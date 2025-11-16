use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
    body::Body,
};
use flow_domain::migration::{Backup, BackupFile};
use flow_service::migration::{BackupService, RestoreService};
use crate::AppState;
use serde::{Deserialize, Serialize};
use axum::body::Bytes;
use futures_util::{Stream, StreamExt, TryStreamExt};
use std::io;

/// 创建备份请求
#[derive(Debug, Deserialize)]
pub struct CreateBackupRequest {
    pub backup: Backup,
}

/// 备份列表响应
#[derive(Debug, Serialize)]
pub struct BackupListResponse {
    pub items: Vec<BackupFile>,
}

/// 创建备份
/// POST /api/v1alpha1/backups
pub async fn create_backup(
    State(state): State<AppState>,
    Json(request): Json<CreateBackupRequest>,
) -> Result<Response, StatusCode> {
    match state.backup_service.backup(request.backup).await {
        Ok(_) => Ok(StatusCode::ACCEPTED.into_response()),
        Err(e) => {
            eprintln!("Failed to create backup: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取备份文件列表
/// GET /api/v1alpha1/backups/files
pub async fn list_backup_files(
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    match state.backup_service.get_backup_files().await {
        Ok(files) => {
            let response = BackupListResponse { items: files };
            Ok(Json(response).into_response())
        }
        Err(e) => {
            eprintln!("Failed to list backup files: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取备份文件信息
/// GET /api/v1alpha1/backups/files/{filename}
pub async fn get_backup_file(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Response, StatusCode> {
    match state.backup_service.get_backup_file(&filename).await {
        Ok(Some(file)) => Ok(Json(file).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to get backup file: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 下载备份文件
/// GET /api/v1alpha1/backups/{name}/download
pub async fn download_backup(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    // 从扩展客户端获取备份对象
    use flow_api::extension::ExtensionClient;
    let backup: Option<Backup> = state.extension_client.fetch(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let backup = backup.ok_or(StatusCode::NOT_FOUND)?;
    
    match state.backup_service.download(&backup).await {
        Ok(path) => {
            // 读取文件并返回
            match tokio::fs::read(&path).await {
                Ok(data) => {
                    use axum::http::header;
                    let headers = [
                        (header::CONTENT_TYPE, "application/zip"),
                        (
                            header::CONTENT_DISPOSITION,
                            &format!("attachment; filename=\"{}\"", 
                                backup.status.filename.as_deref().unwrap_or("backup.zip"))
                        ),
                    ];
                    Ok((headers, data).into_response())
                }
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(e) => {
            eprintln!("Failed to download backup: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 恢复备份
/// POST /api/v1alpha1/backups/restore
pub async fn restore_backup(
    State(state): State<AppState>,
    body: Body,
) -> Result<Response, StatusCode> {
    // 将Body转换为Stream
    use axum::body::HttpBody;
    let mut body_stream = body.into_data_stream();
    
    // 创建一个适配器Stream，将Result<Bytes, axum::Error>转换为Result<Bytes, io::Error>
    let bytes_stream = futures_util::stream::unfold(body_stream, |mut stream| async move {
        match stream.try_next().await {
            Ok(Some(chunk)) => Some((Ok(Bytes::from(chunk.to_vec())), stream)),
            Ok(None) => None,
            Err(e) => Some((Err(io::Error::new(io::ErrorKind::Other, e.to_string())), stream)),
        }
    });
    
    match state.restore_service.restore(bytes_stream).await {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(e) => {
            eprintln!("Failed to restore backup: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 删除备份
/// DELETE /api/v1alpha1/backups/{name}
pub async fn delete_backup(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    // 从扩展客户端获取备份对象
    use flow_api::extension::ExtensionClient;
    let backup: Option<Backup> = state.extension_client.fetch(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let backup = backup.ok_or(StatusCode::NOT_FOUND)?;
    
    match state.backup_service.cleanup(&backup).await {
        Ok(_) => {
            // 删除备份扩展对象
            state.extension_client.delete::<Backup>(&name).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Err(e) => {
            eprintln!("Failed to cleanup backup: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}


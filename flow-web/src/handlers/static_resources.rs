use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use crate::AppState;
use std::path::PathBuf;
use tokio::fs;

/// 提供主题静态资源服务
/// 路径格式: /themes/{theme_name}/**/*.{css,js,png,jpg,svg,woff,woff2,ttf,eot,ico}
/// Path提取器会提取通配符部分，例如：/themes/default/assets/style.css
/// 提取的path是: default/assets/style.css
pub async fn serve_theme_static(
    Path(path): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    serve_theme_static_with_path(Path(path), State(state)).await
}

/// 提供主题静态资源（改进版，支持路径参数）
pub async fn serve_theme_static_with_path(
    Path(path): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    
    // 解析路径: {theme_name}/assets/...
    // Path提取器已经提取了/themes/后面的部分
    let parts: Vec<&str> = path.splitn(2, '/').collect();
    
    if parts.len() < 2 {
        return (StatusCode::BAD_REQUEST, "Invalid theme static resource path").into_response();
    }
    
    let theme_name = parts[0];
    let resource_path = parts[1];
    
    // TODO: 从配置或AppState获取theme_root
    let theme_root = PathBuf::from("./themes"); // 临时路径
    
    // 构建完整文件路径
    let file_path = theme_root.join(theme_name).join(resource_path);
    
    // 验证路径（防止路径遍历攻击）
    // 注意：canonicalize在文件不存在时会失败，所以先检查文件是否存在
    let theme_dir = theme_root.join(theme_name);
    
    // 规范化路径，移除..等危险字符
    let normalized_path = PathBuf::from(resource_path)
        .components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect::<PathBuf>();
    
    let file_path = theme_dir.join(&normalized_path);
    
    // 验证文件路径在主题目录内
    if !file_path.starts_with(&theme_dir) {
        return (StatusCode::FORBIDDEN, "Path traversal detected").into_response();
    }
    
    // 读取文件
    match fs::read(&file_path).await {
        Ok(content) => {
            let content_type = get_content_type(resource_path);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, "public, max-age=31536000") // 1年缓存
                .body(axum::body::Body::from(content))
                .unwrap()
                .into_response()
        }
        Err(_) => {
            (StatusCode::NOT_FOUND, "Resource not found").into_response()
        }
    }
}

/// 根据文件扩展名获取Content-Type
fn get_content_type(path: &str) -> &'static str {
    let path_buf = PathBuf::from(path);
    let ext = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    let ext_lower = ext.to_lowercase();
    match ext_lower.as_str() {
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        "ico" => "image/x-icon",
        "html" => "text/html",
        "xml" => "application/xml",
        _ => "application/octet-stream",
    }
}


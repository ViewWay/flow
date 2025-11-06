use axum::extract::{FromRequest, Multipart};
use axum::http::StatusCode;
use flow_api::security::AuthenticatedUser;

/// Multipart和用户信息的组合提取器
/// 用于在上传文件时同时获取multipart数据和当前用户信息
pub struct MultipartWithUser {
    pub multipart: Multipart,
    pub user: Option<AuthenticatedUser>,
}

#[async_trait::async_trait]
impl<S> FromRequest<S> for MultipartWithUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        // 先提取用户信息（从请求扩展中）
        let user = req.extensions()
            .get::<AuthenticatedUser>()
            .cloned();
        
        // 然后提取Multipart
        let multipart = Multipart::from_request(req, state).await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        Ok(MultipartWithUser {
            multipart,
            user,
        })
    }
}


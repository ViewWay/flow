pub mod multipart_with_user;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use flow_api::security::AuthenticatedUser;

/// 当前用户提取器
/// 从请求扩展中提取已认证的用户信息
pub struct CurrentUser(pub String);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions
            .get::<AuthenticatedUser>()
            .map(|user| CurrentUser(user.username.clone()))
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}


use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flow_api::extension::{ExtensionClient, ListOptions};
use flow_domain::security::{Role, RoleBinding};
use flow_service::security::{RoleBindingService, role_binding_service::DefaultRoleBindingService};
use crate::AppState;
use serde::{Deserialize, Serialize};

// 辅助函数：使用ExtensionClient trait的方法
async fn fetch_role(client: &flow_infra::extension::ReactiveExtensionClient, name: &str) -> Result<Option<Role>, Box<dyn std::error::Error + Send + Sync>> {
    <flow_infra::extension::ReactiveExtensionClient as ExtensionClient>::fetch(client, name).await
}

async fn create_role_helper(client: &flow_infra::extension::ReactiveExtensionClient, role: Role) -> Result<Role, Box<dyn std::error::Error + Send + Sync>> {
    <flow_infra::extension::ReactiveExtensionClient as ExtensionClient>::create(client, role).await
}

async fn list_roles_helper(client: &flow_infra::extension::ReactiveExtensionClient, options: ListOptions) -> Result<flow_api::extension::ListResult<Role>, Box<dyn std::error::Error + Send + Sync>> {
    <flow_infra::extension::ReactiveExtensionClient as ExtensionClient>::list(client, options).await
}

async fn fetch_role_binding(client: &flow_infra::extension::ReactiveExtensionClient, name: &str) -> Result<Option<RoleBinding>, Box<dyn std::error::Error + Send + Sync>> {
    <flow_infra::extension::ReactiveExtensionClient as ExtensionClient>::fetch(client, name).await
}

async fn create_role_binding_helper(client: &flow_infra::extension::ReactiveExtensionClient, binding: RoleBinding) -> Result<RoleBinding, Box<dyn std::error::Error + Send + Sync>> {
    <flow_infra::extension::ReactiveExtensionClient as ExtensionClient>::create(client, binding).await
}

async fn list_role_bindings_helper(client: &flow_infra::extension::ReactiveExtensionClient, options: ListOptions) -> Result<flow_api::extension::ListResult<RoleBinding>, Box<dyn std::error::Error + Send + Sync>> {
    <flow_infra::extension::ReactiveExtensionClient as ExtensionClient>::list(client, options).await
}

async fn delete_role_binding_helper(client: &flow_infra::extension::ReactiveExtensionClient, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    <flow_infra::extension::ReactiveExtensionClient as ExtensionClient>::delete::<RoleBinding>(client, name).await
}

/// 创建角色请求
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub rules: Vec<flow_domain::security::PolicyRule>,
}

/// 创建角色绑定请求
#[derive(Debug, Deserialize)]
pub struct CreateRoleBindingRequest {
    pub username: String,
    pub role_name: String,
}

/// 授予角色请求
#[derive(Debug, Deserialize)]
pub struct GrantRolesRequest {
    pub role_names: Vec<String>,
}

/// 角色列表响应
#[derive(Debug, Serialize)]
pub struct RoleListResponse {
    pub items: Vec<Role>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
}

/// 角色绑定列表响应
#[derive(Debug, Serialize)]
pub struct RoleBindingListResponse {
    pub items: Vec<RoleBinding>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
}

/// 创建角色
/// POST /api/v1alpha1/roles
pub async fn create_role(
    State(state): State<AppState>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<Response, StatusCode> {
    use flow_api::extension::Metadata;

    // 检查角色是否已存在
    if let Ok(Some(_)) = fetch_role(&state.extension_client, &request.name).await {
        return Err(StatusCode::CONFLICT);
    }

    let role = Role {
        metadata: Metadata::new(request.name),
        rules: request.rules,
    };

    match create_role_helper(&state.extension_client, role).await {
        Ok(role) => Ok(Json(role).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取角色
/// GET /api/v1alpha1/roles/{name}
pub async fn get_role(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match fetch_role(&state.extension_client, &name).await {
        Ok(Some(role)) => Ok(Json(role).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出角色
/// GET /api/v1alpha1/roles
pub async fn list_roles(
    State(state): State<AppState>,
    Query(params): Query<ListOptions>,
) -> Result<Response, StatusCode> {
    match list_roles_helper(&state.extension_client, params).await {
        Ok(result) => {
            let response = RoleListResponse {
                items: result.items,
                total: result.total,
                page: result.page as u64,
                size: result.size as u64,
            };
            Ok(Json(response).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 创建角色绑定
/// POST /api/v1alpha1/rolebindings
pub async fn create_role_binding(
    State(state): State<AppState>,
    Json(request): Json<CreateRoleBindingRequest>,
) -> Result<Response, StatusCode> {
    // 检查用户是否存在
    if let Ok(None) = state.user_service.get(&request.username).await {
        return Err(StatusCode::NOT_FOUND);
    }

    // 检查角色是否存在
    if let Ok(None) = fetch_role(&state.extension_client, &request.role_name).await {
        return Err(StatusCode::NOT_FOUND);
    }

    let binding = RoleBinding::create(&request.username, &request.role_name);

    match create_role_binding_helper(&state.extension_client, binding).await {
        Ok(binding) => Ok(Json(binding).into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取角色绑定
/// GET /api/v1alpha1/rolebindings/{name}
pub async fn get_role_binding(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match fetch_role_binding(&state.extension_client, &name).await {
        Ok(Some(binding)) => Ok(Json(binding).into_response()),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 列出角色绑定
/// GET /api/v1alpha1/rolebindings
pub async fn list_role_bindings(
    State(state): State<AppState>,
    Query(params): Query<ListOptions>,
) -> Result<Response, StatusCode> {
    match list_role_bindings_helper(&state.extension_client, params).await {
        Ok(result) => {
            let response = RoleBindingListResponse {
                items: result.items,
                total: result.total,
                page: result.page as u64,
                size: result.size as u64,
            };
            Ok(Json(response).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 为用户授予角色
/// POST /api/v1alpha1/users/{name}/roles
pub async fn grant_user_roles(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<GrantRolesRequest>,
) -> Result<Response, StatusCode> {
    // 检查用户是否存在
    if let Ok(None) = state.user_service.get(&name).await {
        return Err(StatusCode::NOT_FOUND);
    }

    // 使用RoleBindingService授予角色
    let binding_service = DefaultRoleBindingService::new(state.extension_client.clone());

    match <DefaultRoleBindingService<_> as RoleBindingService>::grant_roles(&binding_service, &name, &request.role_names).await {
        Ok(_) => Ok((StatusCode::OK, "Roles granted successfully").into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 删除角色绑定
/// DELETE /api/v1alpha1/rolebindings/{name}
pub async fn delete_role_binding(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    match delete_role_binding_helper(&state.extension_client, &name).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, "").into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}


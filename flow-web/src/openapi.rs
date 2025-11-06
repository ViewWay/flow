use utoipa::OpenApi;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify,
};

/// OpenAPI文档配置
#[derive(OpenApi)]
#[openapi(
    paths(
        // 这里可以添加具体的路径，但由于路由是动态的，我们使用通用配置
    ),
    components(schemas()),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "健康检查端点"),
        (name = "auth", description = "认证相关端点"),
        (name = "users", description = "用户管理端点"),
        (name = "posts", description = "文章管理端点"),
        (name = "uc", description = "用户中心端点"),
        (name = "extensions", description = "扩展对象端点"),
    ),
    info(
        title = "Flow API",
        description = "Flow - Halo的Rust实现",
        version = "1.0.0"
    ),
    servers(
        (url = "http://localhost:8090", description = "本地开发服务器"),
    )
)]
pub struct ApiDoc;

/// 安全配置修改器
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "basicAuth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Basic)),
            );
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}


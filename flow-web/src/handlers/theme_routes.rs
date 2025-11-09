use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    Json,
};
use flow_infra::theme::{ThemeResolver, TemplateEngineManager};
use flow_infra::theme::template_engine::TemplateContext;
use flow_service::theme::finders::{PostFinder, CategoryFinder, TagFinder};
use flow_service::content::PostQuery;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// 渲染主题模板
pub async fn render_theme_template(
    Path(template_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 1. 获取当前主题（支持预览参数）
    let preview_theme = params.get("preview");
    let theme_context = match state.theme_resolver.get_theme_with_preview(preview_theme.map(|s| s.as_str())).await {
        Ok(ctx) => ctx,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get theme context: {}", e)
            ).into_response();
        }
    };
    
    // 2. 获取模板引擎
    let engine = state.template_engine_manager.get_template_engine(&theme_context).await;
    
    // 3. 构建模板上下文（包含Finder数据）
    let mut template_context = TemplateContext::new();
    
    // 添加模型数据（从查询参数中提取）
    let mut model: HashMap<String, serde_json::Value> = HashMap::new();
    for (key, value) in params {
        if key != "preview" {
            model.insert(key, serde_json::Value::String(value));
        }
    }
    template_context = template_context.with_model(model);
    
    // 4. 渲染模板
    match engine.render(&template_name, &template_context) {
        Ok(rendered) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(axum::body::Body::from(rendered))
                .unwrap()
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", e)
            ).into_response()
        }
    }
}

/// 文章页面路由
pub async fn post_page(
    Path(slug): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 1. 根据slug查找Post
    let post_finder = PostFinder::new(state.post_service.clone());
    let post_value = match post_finder.get_by_slug(&slug).await {
        Ok(value) => value,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get post: {}", e)
            ).into_response();
        }
    };
    
    if post_value.is_null() {
        return (
            StatusCode::NOT_FOUND,
            format!("Post not found: {}", slug)
        ).into_response();
    }
    
    // 2. 确定使用的模板（自定义模板或默认模板）
    let template_name = params.get("template")
        .map(|s| s.as_str())
        .unwrap_or("post.html");
    
    // 3. 构建模板上下文
    let theme_context = match state.theme_resolver.get_active_theme_context().await {
        Ok(Some(ctx)) => ctx,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No active theme"
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get theme context: {}", e)
            ).into_response();
        }
    };
    
    let engine = state.template_engine_manager.get_template_engine(&theme_context).await;
    
    let mut template_context = TemplateContext::new();
    let mut model: HashMap<String, serde_json::Value> = HashMap::new();
    model.insert("post".to_string(), post_value);
    template_context = template_context.with_model(model);
    
    // 4. 渲染模板
    match engine.render(template_name, &template_context) {
        Ok(rendered) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(axum::body::Body::from(rendered))
                .unwrap()
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", e)
            ).into_response()
        }
    }
}

/// 分类页面路由
pub async fn category_page(
    Path(name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 1. 根据name查找Category
    let category_finder = CategoryFinder::new(state.category_service.clone());
    let category_value = match category_finder.get_by_name(&name).await {
        Ok(value) => value,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get category: {}", e)
            ).into_response();
        }
    };
    
    if category_value.is_null() {
        return (
            StatusCode::NOT_FOUND,
            format!("Category not found: {}", name)
        ).into_response();
    }
    
    // 2. 查询该分类下的Posts
    let post_finder = PostFinder::new(state.post_service.clone());
    let query = PostQuery {
        category: Some(name.clone()),
        published: Some(true),
        ..Default::default()
    };
    let posts_value = match post_finder.list(Some(query)).await {
        Ok(value) => value,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to list posts: {}", e)
            ).into_response();
        }
    };
    
    // 3. 确定使用的模板
    let template_name = params.get("template")
        .map(|s| s.as_str())
        .unwrap_or("category.html");
    
    // 4. 构建模板上下文并渲染
    let theme_context = match state.theme_resolver.get_active_theme_context().await {
        Ok(Some(ctx)) => ctx,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No active theme"
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get theme context: {}", e)
            ).into_response();
        }
    };
    
    let engine = state.template_engine_manager.get_template_engine(&theme_context).await;
    
    let mut template_context = TemplateContext::new();
    let mut model: HashMap<String, serde_json::Value> = HashMap::new();
    model.insert("category".to_string(), category_value);
    model.insert("posts".to_string(), posts_value);
    template_context = template_context.with_model(model);
    
    match engine.render(template_name, &template_context) {
        Ok(rendered) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(axum::body::Body::from(rendered))
                .unwrap()
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", e)
            ).into_response()
        }
    }
}

/// 标签页面路由
pub async fn tag_page(
    Path(name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 1. 根据name查找Tag
    let tag_finder = TagFinder::new(state.tag_service.clone());
    let tag_value = match tag_finder.get_by_name(&name).await {
        Ok(value) => value,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get tag: {}", e)
            ).into_response();
        }
    };
    
    if tag_value.is_null() {
        return (
            StatusCode::NOT_FOUND,
            format!("Tag not found: {}", name)
        ).into_response();
    }
    
    // 2. 查询该标签下的Posts
    let post_finder = PostFinder::new(state.post_service.clone());
    let query = PostQuery {
        tag: Some(name.clone()),
        published: Some(true),
        ..Default::default()
    };
    let posts_value = match post_finder.list(Some(query)).await {
        Ok(value) => value,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to list posts: {}", e)
            ).into_response();
        }
    };
    
    // 3. 确定使用的模板
    let template_name = params.get("template")
        .map(|s| s.as_str())
        .unwrap_or("tag.html");
    
    // 4. 构建模板上下文并渲染
    let theme_context = match state.theme_resolver.get_active_theme_context().await {
        Ok(Some(ctx)) => ctx,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No active theme"
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get theme context: {}", e)
            ).into_response();
        }
    };
    
    let engine = state.template_engine_manager.get_template_engine(&theme_context).await;
    
    let mut template_context = TemplateContext::new();
    let mut model: HashMap<String, serde_json::Value> = HashMap::new();
    model.insert("tag".to_string(), tag_value);
    model.insert("posts".to_string(), posts_value);
    template_context = template_context.with_model(model);
    
    match engine.render(template_name, &template_context) {
        Ok(rendered) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(axum::body::Body::from(rendered))
                .unwrap()
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", e)
            ).into_response()
        }
    }
}

/// 归档页面路由
pub async fn archive_page(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 1. 查询所有已发布的Posts
    let post_finder = PostFinder::new(state.post_service.clone());
    let query = PostQuery {
        published: Some(true),
        ..Default::default()
    };
    let posts_value = match post_finder.list(Some(query)).await {
        Ok(value) => value,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to list posts: {}", e)
            ).into_response();
        }
    };
    
    // 2. 确定使用的模板
    let template_name = params.get("template")
        .map(|s| s.as_str())
        .unwrap_or("archive.html");
    
    // 3. 构建模板上下文并渲染
    let theme_context = match state.theme_resolver.get_active_theme_context().await {
        Ok(Some(ctx)) => ctx,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No active theme"
            ).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get theme context: {}", e)
            ).into_response();
        }
    };
    
    let engine = state.template_engine_manager.get_template_engine(&theme_context).await;
    
    let mut template_context = TemplateContext::new();
    let mut model: HashMap<String, serde_json::Value> = HashMap::new();
    model.insert("posts".to_string(), posts_value);
    template_context = template_context.with_model(model);
    
    match engine.render(template_name, &template_context) {
        Ok(rendered) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(axum::body::Body::from(rendered))
                .unwrap()
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", e)
            ).into_response()
        }
    }
}


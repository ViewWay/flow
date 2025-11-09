use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use flow_api::search::{SearchOption, SortField, SortOrder};
use flow_service::search::SearchService;
use crate::AppState;
use serde::Deserialize;

/// 搜索请求参数
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub keyword: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default = "default_highlight_pre_tag")]
    #[serde(rename = "highlightPreTag")]
    pub highlight_pre_tag: String,
    #[serde(default = "default_highlight_post_tag")]
    #[serde(rename = "highlightPostTag")]
    pub highlight_post_tag: String,
    #[serde(rename = "filterExposed")]
    pub filter_exposed: Option<bool>,
    #[serde(rename = "filterRecycled")]
    pub filter_recycled: Option<bool>,
    #[serde(rename = "filterPublished")]
    pub filter_published: Option<bool>,
    #[serde(rename = "includeTypes")]
    pub include_types: Option<Vec<String>>,
    #[serde(rename = "includeOwnerNames")]
    pub include_owner_names: Option<Vec<String>>,
    #[serde(rename = "includeCategoryNames")]
    pub include_category_names: Option<Vec<String>>,
    #[serde(rename = "includeTagNames")]
    pub include_tag_names: Option<Vec<String>>,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<SortField>,
    #[serde(rename = "sortOrder")]
    #[serde(default = "default_sort_order")]
    pub sort_order: SortOrder,
}

fn default_limit() -> u32 {
    10
}

fn default_highlight_pre_tag() -> String {
    "<B>".to_string()
}

fn default_highlight_post_tag() -> String {
    "</B>".to_string()
}

fn default_sort_order() -> SortOrder {
    SortOrder::Desc
}

impl From<SearchQuery> for SearchOption {
    fn from(query: SearchQuery) -> Self {
        SearchOption {
            keyword: query.keyword,
            limit: query.limit,
            highlight_pre_tag: query.highlight_pre_tag,
            highlight_post_tag: query.highlight_post_tag,
            filter_exposed: query.filter_exposed,
            filter_recycled: query.filter_recycled,
            filter_published: query.filter_published,
            include_types: query.include_types,
            include_owner_names: query.include_owner_names,
            include_category_names: query.include_category_names,
            include_tag_names: query.include_tag_names,
            sort_by: query.sort_by,
            sort_order: query.sort_order,
            annotations: None,
        }
    }
}

/// 搜索端点
pub async fn search(
    Query(query): Query<SearchQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 验证关键词
    if query.keyword.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "message": "Keyword cannot be empty"
            })),
        ).into_response();
    }
    
    // 验证limit范围
    if query.limit == 0 || query.limit > 1000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "message": "Limit must be between 1 and 1000"
            })),
        ).into_response();
    }
    
    let search_option: SearchOption = query.into();
    
    match state.search_service.search(search_option).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => {
            eprintln!("Search failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Search failed: {}", e)
                })),
            ).into_response()
        }
    }
}


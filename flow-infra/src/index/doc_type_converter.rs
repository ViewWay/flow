use flow_api::extension::Extension;

/// 文档类型提供者trait
/// 为每个Extension类型实现此trait以提供文档类型
/// 文档类型格式：{kind.lowercase()}.{group}
pub trait DocTypeProvider: Extension {
    /// 获取该Extension类型对应的文档类型
    /// 格式：{kind.lowercase()}.{group}
    fn doc_type() -> String;
}

// 为Post实现
impl DocTypeProvider for flow_domain::content::Post {
    fn doc_type() -> String {
        "post.content.halo.run".to_string()
    }
}

// 为SinglePage实现
impl DocTypeProvider for flow_domain::content::SinglePage {
    fn doc_type() -> String {
        "singlepage.content.halo.run".to_string()
    }
}


pub mod post;
pub mod single_page;
pub mod comment;
pub mod snapshot;
pub mod category;
pub mod tag;

pub use post::{Post, PostSpec, PostStatus, PostPhase, VisibleEnum, Excerpt};
pub use single_page::{SinglePage, SinglePageSpec, SinglePageStatus};
pub use comment::{Comment, CommentSpec, CommentStatus, CommentOwner, BaseCommentSpec, SubjectRef};
pub use snapshot::{Snapshot, SnapshotSpec};
pub use category::{Category, CategorySpec, CategoryStatus};
pub use tag::{Tag, TagSpec, TagStatus};

/// 内容管理相关的常量
pub mod constant {
    pub const GROUP: &str = "content.halo.run";
    pub const VERSION: &str = "v1alpha1";
    
    // Post相关
    pub const POST_KIND: &str = "Post";
    pub const POST_DELETED_LABEL: &str = "content.halo.run/deleted";
    pub const POST_PUBLISHED_LABEL: &str = "content.halo.run/published";
    pub const POST_OWNER_LABEL: &str = "content.halo.run/owner";
    pub const POST_VISIBLE_LABEL: &str = "content.halo.run/visible";
    pub const POST_SCHEDULING_PUBLISH_LABEL: &str = "content.halo.run/scheduling-publish";
    pub const POST_ARCHIVE_YEAR_LABEL: &str = "content.halo.run/archive-year";
    pub const POST_ARCHIVE_MONTH_LABEL: &str = "content.halo.run/archive-month";
    pub const POST_ARCHIVE_DAY_LABEL: &str = "content.halo.run/archive-day";
    pub const POST_CATEGORIES_ANNO: &str = "content.halo.run/categories";
    pub const POST_LAST_RELEASED_SNAPSHOT_ANNO: &str = "content.halo.run/last-released-snapshot";
    pub const POST_LAST_ASSOCIATED_TAGS_ANNO: &str = "content.halo.run/last-associated-tags";
    pub const POST_LAST_ASSOCIATED_CATEGORIES_ANNO: &str = "content.halo.run/last-associated-categories";
    pub const POST_STATS_ANNO: &str = "content.halo.run/stats";
    
    // SinglePage相关
    pub const SINGLE_PAGE_KIND: &str = "SinglePage";
    
    // Comment相关
    pub const COMMENT_KIND: &str = "Comment";
    
    // Snapshot相关
    pub const SNAPSHOT_KIND: &str = "Snapshot";
    pub const SNAPSHOT_KEEP_RAW_ANNO: &str = "content.halo.run/keep-raw";
    pub const SNAPSHOT_PATCHED_CONTENT_ANNO: &str = "content.halo.run/patched-content";
    pub const SNAPSHOT_PATCHED_RAW_ANNO: &str = "content.halo.run/patched-raw";
    
    // Category相关
    pub const CATEGORY_KIND: &str = "Category";
    pub const CATEGORY_LAST_HIDDEN_STATE_ANNO: &str = "content.halo.run/last-hidden-state";
    
    // Tag相关
    pub const TAG_KIND: &str = "Tag";
}


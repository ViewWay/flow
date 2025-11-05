use crate::database::extension_store::{Entity as ExtensionStoreEntity, Model as ExtensionStoreModel};
use flow_api::extension::ListOptions;
use sea_orm::{
    DatabaseConnection, EntityTrait, PaginatorTrait,
};
use async_trait::async_trait;
use std::sync::Arc;

/// ExtensionRepository trait 定义扩展对象的数据访问操作
#[async_trait]
pub trait ExtensionRepository: Send + Sync {
    async fn save(&self, store: ExtensionStoreModel) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<ExtensionStoreModel>, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn list(&self, options: ListOptions) -> Result<Vec<ExtensionStoreModel>, Box<dyn std::error::Error + Send + Sync>>;
}

/// SeaOrmExtensionRepository 使用Sea-ORM实现的Repository
pub struct SeaOrmExtensionRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmExtensionRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ExtensionRepository for SeaOrmExtensionRepository {
    async fn save(&self, store: ExtensionStoreModel) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::database::extension_store;
        
        let active_model = extension_store::ActiveModel {
            name: sea_orm::Set(store.name),
            data: sea_orm::Set(store.data),
            version: sea_orm::Set(store.version),
        };

        ExtensionStoreEntity::insert(active_model)
            .exec(&*self.db)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<ExtensionStoreModel>, Box<dyn std::error::Error + Send + Sync>> {
        use sea_orm::EntityTrait;
        
        let result = ExtensionStoreEntity::find_by_id(name.to_string())
            .one(&*self.db)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(result)
    }

    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use sea_orm::EntityTrait;
        
        ExtensionStoreEntity::delete_by_id(name.to_string())
            .exec(&*self.db)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    async fn list(&self, options: ListOptions) -> Result<Vec<ExtensionStoreModel>, Box<dyn std::error::Error + Send + Sync>> {
        use sea_orm::EntityTrait;
        
        let query = ExtensionStoreEntity::find();

        // TODO: 实现label_selector和field_selector过滤
        // TODO: 实现排序

        let page = options.page.unwrap_or(0);
        let size = options.size.unwrap_or(10);

        let paginator = query.paginate(&*self.db, size as u64);
        let items = paginator.fetch_page(page as u64)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(items)
    }
}


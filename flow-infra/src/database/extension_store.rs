use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// ExtensionStore 实体，对应数据库中的extensions表
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "extensions")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "String(Some(255))")]
    pub name: String,
    
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub data: Vec<u8>,
    
    #[sea_orm(column_type = "BigInteger", nullable)]
    pub version: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}


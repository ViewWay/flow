use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250101_000001_create_extensions_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExtensionStore::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExtensionStore::Name)
                            .string()
                            .not_null()
                            .primary_key()
                            .string_len(255),
                    )
                    .col(ColumnDef::new(ExtensionStore::Data).binary().not_null())
                    .col(ColumnDef::new(ExtensionStore::Version).big_integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExtensionStore::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ExtensionStore {
    Table,
    Name,
    Data,
    Version,
}


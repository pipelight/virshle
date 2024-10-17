//!
//! Generate entities.
//!
//! ```sh
//! # on the repo root
//! sea-orm-cli generate entity --output-dir ./entity/src
//! ```
//!

use miette::{IntoDiagnostic, Result};
use sea_orm_migration::prelude::*;
use sea_query::Index;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Virtual machines list
        manager
            .create_table(
                Table::create()
                    .table(Vm::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Vm::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Vm::Uuid).string().not_null().unique_key())
                    .col(ColumnDef::new(Vm::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Vm::Definition).json().not_null())
                    .to_owned(),
            )
            .await?;
        // Networks list
        manager
            .create_table(
                Table::create()
                    .table(Net::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Net::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Net::Uuid).string().not_null().unique_key())
                    .col(ColumnDef::new(Net::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Net::Definition).json().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Vm::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Net::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden, Debug)]
pub enum Vm {
    Table,
    Id,
    Uuid,
    Name,
    Definition,
}

#[derive(DeriveIden, Debug)]
pub enum Net {
    Table,
    Id,
    Uuid,
    Name,
    Definition,
}

#[cfg(test)]
mod tests {
    use crate::{Migrator, MigratorTrait};
    use miette::{IntoDiagnostic, Result};

    #[tokio::test]
    async fn create_db() -> Result<()> {
        let database_url = "sqlite:////var/lib/virshle/virshle.sqlite?mode=rwc";
        let connection = sea_orm::Database::connect(database_url)
            .await
            .into_diagnostic()?;
        // Migrator::fresh(&connection).await.into_diagnostic()?;
        Ok(())
    }
}

//!
//! Generate entities.
//!
//! ```sh
//! # on the repo root
//! sea-orm-cli generate entity --output-dir ./entity/src
//! ```
//!

use miette::{IntoDiagnostic, Result};
use sea_orm_migration::{prelude::*, schema::*};
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
                    .col(pk_auto(Vm::Id))
                    .col(string_uniq(Vm::Uuid))
                    .col(string_uniq(Vm::Name))
                    .col(json(Vm::Definition))
                    // .col(date_time(Vm::CreatedAt))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Vm::Table).to_owned())
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
    CreatedAt,
}

#[derive(DeriveIden, Debug)]
pub enum Account {
    Table,
    Id,
    Uuid,
    Name,
    Definition,
    CreatedAt,
}

#[derive(DeriveIden, Debug)]
pub enum AccountVm {
    Table,
    Id,
    AccountId,
    VmId,
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

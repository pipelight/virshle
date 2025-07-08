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
                    .col(date_time(Vm::CreatedAt))
                    .col(date_time(Vm::UpdatedAt))
                    .to_owned(),
            )
            .await?;
        // Ip leases
        manager
            .create_table(
                Table::create()
                    .table(Lease::Table)
                    .if_not_exists()
                    .col(pk_auto(Lease::Id))
                    .col(integer(Lease::VmId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("vm_id")
                            .from(Lease::Table, Lease::VmId)
                            .to(Vm::Table, Vm::Id),
                    )
                    .col(string_uniq(Lease::Ip))
                    .col(date_time(Lease::CreatedAt))
                    .col(date_time(Lease::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // User account list
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(pk_auto(Account::Id))
                    .col(string_uniq(Account::Uuid))
                    .to_owned(),
            )
            .await?;

        // Junction table Account_Vm
        manager
            .create_table(
                Table::create()
                    .table(AccountVm::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .col(AccountVm::AccountId)
                            .col(AccountVm::VmId),
                    )
                    .col(integer(AccountVm::AccountId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("account_id")
                            .from(AccountVm::Table, AccountVm::AccountId)
                            .to(Account::Table, Account::Id),
                    )
                    .col(integer(AccountVm::VmId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("vm_id")
                            .from(AccountVm::Table, AccountVm::VmId)
                            .to(Vm::Table, Vm::Id),
                    )
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
    UpdatedAt,
}

#[derive(DeriveIden, Debug)]
pub enum Account {
    Table,
    Id,
    Uuid,
}

#[derive(DeriveIden, Debug)]
pub enum Lease {
    Table,
    Id,
    VmId,
    Ip,
    CreatedAt,
    UpdatedAt,
}

/// Junction table
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

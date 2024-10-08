// Sea orm
// use indexmap::IndexMap;
use super::entity::{prelude::*, *};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    error::{ConnAcquireErr, DbErr},
    Database, DatabaseConnection,
};
use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult};

// Error Handling
use virshle_error::VirshleError;
use miette::{Error, IntoDiagnostic, Result};

// Global vars
// use once_cell::sync::Lazy;
// use std::sync::Arc;
// use tokio::sync::Mutex;
const DEFAULT_DATABASE_URL: &'static str = "sqlite:////var/lib/virshle/virshle.sqlite?mode=rwc";

pub async fn connect_db() -> Result<DatabaseConnection, VirshleError> {
    let db = Database::connect(DEFAULT_DATABASE_URL).await;
    match db {
        Err(e) => {
            Err(e.into())
            // let db = fresh_db().await?;
            // Ok(db)
        }
        Ok(db) => Ok(db),
    }
}
pub async fn fresh_db() -> Result<DatabaseConnection, VirshleError> {
    let db = Database::connect(DEFAULT_DATABASE_URL).await?;
    Migrator::fresh(&db).await?;
    Ok(db)
}
#[cfg(test)]
mod test {
    use super::*;
    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[tokio::test]
    async fn connect_to_db() -> Result<()> {
        // connect_db().await?;
        fresh_db().await?;
        Ok(())
    }
}

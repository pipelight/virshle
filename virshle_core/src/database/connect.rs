// Sea orm
// use indexmap::IndexMap;
use super::entity::{prelude::*, *};
use crate::config::MANAGED_DIR;
use sea_orm::{
    error::{ConnAcquireErr, DbErr},
    Database, DatabaseConnection,
};
use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult};
use std::path::Path;
use virshle_migration::{Migrator, MigratorTrait};

// Error Handling
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::VirshleError;

// Global vars
// use once_cell::sync::Lazy;
// use std::sync::Arc;
// use tokio::sync::Mutex;
pub fn get_database_url() -> Result<String, VirshleError> {
    let url = format!("sqlite:///{MANAGED_DIR}/virshle.sqlite?mode=rwc");
    Ok(url)
}

pub async fn connect_db() -> Result<DatabaseConnection, VirshleError> {
    // Ensure database exists with virshle default tables.
    let path = format!("{MANAGED_DIR}/virshle.sqlite");
    let path = Path::new(&path);
    if !path.exists() {
        let db = fresh_db().await?;
    }
    let db = Database::connect(&get_database_url()?).await;
    match db {
        Err(e) => Err(e.into()),
        Ok(db) => Ok(db),
    }
}
/*
* Create a fresh database and ovewrite the old one.
*/
pub async fn fresh_db() -> Result<DatabaseConnection, VirshleError> {
    let db = Database::connect(&get_database_url()?).await?;
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
        connect_db().await?;
        Ok(())
    }
}

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

pub fn get_db_url() -> Result<String, VirshleError> {
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
    let db = Database::connect(&get_db_url()?).await;
    match db {
        Err(e) => Err(e.into()),
        Ok(db) => Ok(db),
    }
}
/// Create a fresh database and ovewrite the old one.
pub async fn fresh_db() -> Result<DatabaseConnection, VirshleError> {
    let db = Database::connect(&get_db_url()?).await?;
    Migrator::fresh(&db).await?;
    Ok(db)
}

pub async fn connect_or_fresh_db() -> Result<DatabaseConnection, VirshleError> {
    match connect_db().await {
        Ok(db) => Ok(db),
        Err(e) => {
            let db = fresh_db().await?;
            Ok(db)
        }
    }
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

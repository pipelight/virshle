use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;

    #[tokio::test]
    async fn migrate() -> Result<()> {
        cli::run_cli(migration::Migrator).await;
        Ok(())
    }
}

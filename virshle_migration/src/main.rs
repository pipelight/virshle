use sea_orm_migration::prelude::*;
use virshle_migration::Migrator;

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;

    #[tokio::test]
    async fn migrate() -> Result<()> {
        cli::run_cli(Migrator).await;
        Ok(())
    }
}

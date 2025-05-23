pub use sea_orm_migration::prelude::*;

mod create_table;
pub use create_table::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(create_table::Migration)]
    }
}

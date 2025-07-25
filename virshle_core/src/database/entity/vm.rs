//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.12

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "vm")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub uuid: String,
    #[sea_orm(unique)]
    pub name: String,
    pub definition: Json,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::account_vm::Entity")]
    AccountVm,
    #[sea_orm(has_many = "super::lease::Entity")]
    Lease,
}

impl Related<super::account_vm::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountVm.def()
    }
}

impl Related<super::lease::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lease.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        super::account_vm::Relation::Account.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::account_vm::Relation::Vm.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

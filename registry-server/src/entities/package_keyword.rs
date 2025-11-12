use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "package_keywords")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub package_id: i64,
    pub keyword: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::package::Entity",
        from = "Column::PackageId",
        to = "super::package::Column::Id"
    )]
    Package,
}

impl Related<super::package::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Package.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

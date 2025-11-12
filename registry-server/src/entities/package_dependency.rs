use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "package_dependencies")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub package_version_id: i64,
    pub dependency_name: String,
    pub version_constraint: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::package_version::Entity",
        from = "Column::PackageVersionId",
        to = "super::package_version::Column::Id"
    )]
    PackageVersion,
}

impl Related<super::package_version::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PackageVersion.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

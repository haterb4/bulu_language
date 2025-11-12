use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "packages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub name: String,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::package_version::Entity")]
    PackageVersions,
    #[sea_orm(has_many = "super::package_keyword::Entity")]
    PackageKeywords,
}

impl Related<super::package_version::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PackageVersions.def()
    }
}

impl Related<super::package_keyword::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PackageKeywords.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

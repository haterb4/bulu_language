use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "package_versions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub package_id: i64,
    pub version: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub checksum: String,
    pub tarball_s3_key: String,
    pub tarball_size: i64,
    pub published_at: DateTimeWithTimeZone,
    pub downloads: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::package::Entity",
        from = "Column::PackageId",
        to = "super::package::Column::Id"
    )]
    Package,
    #[sea_orm(has_many = "super::package_author::Entity")]
    PackageAuthors,
    #[sea_orm(has_many = "super::package_dependency::Entity")]
    PackageDependencies,
    #[sea_orm(has_many = "super::download_stat::Entity")]
    DownloadStats,
}

impl Related<super::package::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Package.def()
    }
}

impl Related<super::package_author::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PackageAuthors.def()
    }
}

impl Related<super::package_dependency::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PackageDependencies.def()
    }
}

impl Related<super::download_stat::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DownloadStats.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub mod package;
pub mod package_version;
pub mod package_author;
pub mod package_keyword;
pub mod package_dependency;
pub mod download_stat;

pub use package::Entity as Package;
pub use package_version::Entity as PackageVersion;
pub use package_author::Entity as PackageAuthor;
pub use package_keyword::Entity as PackageKeyword;
pub use package_dependency::Entity as PackageDependency;
pub use download_stat::Entity as DownloadStat;

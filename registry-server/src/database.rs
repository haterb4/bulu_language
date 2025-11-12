//! Database operations using SeaORM

use sea_orm::*;
use std::collections::HashMap;
use crate::entities::{self, package, package_version, package_author, package_keyword, package_dependency, download_stat};

#[derive(Clone)]
pub struct Database {
    pub db: DatabaseConnection,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self, DbErr> {
        let db = sea_orm::Database::connect(database_url).await?;
        
        // Run migrations
        Self::run_migrations(&db).await?;
        
        Ok(Self { db })
    }
    
    /// Run database migrations
    async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
        tracing::info!("🔄 Running database migrations...");
        let migration_sql = include_str!("../migrations/001_initial_schema.sql");
        
        let statements: Vec<&str> = migration_sql.split(';').collect();
        tracing::info!("📝 Found {} SQL statements", statements.len());
        
        for (i, statement) in statements.iter().enumerate() {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                tracing::debug!("Executing statement {}/{}", i + 1, statements.len());
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    trimmed.to_string(),
                ))
                .await?;
            }
        }
        
        tracing::info!("✅ Migrations completed successfully");
        Ok(())
    }

    /// Create or update a package
    pub async fn upsert_package(
        &self,
        name: &str,
        description: Option<&str>,
        repository: Option<&str>,
    ) -> Result<i64, DbErr> {
        let now = chrono::Utc::now();
        
        // Try to find existing package
        let existing = package::Entity::find()
            .filter(package::Column::Name.eq(name))
            .one(&self.db)
            .await?;
        
        if let Some(pkg) = existing {
            // Update existing package
            let mut active_model: package::ActiveModel = pkg.into();
            active_model.description = Set(description.map(|s| s.to_string()));
            active_model.repository = Set(repository.map(|s| s.to_string()));
            active_model.updated_at = Set(now.into());
            
            let updated = active_model.update(&self.db).await?;
            Ok(updated.id)
        } else {
            // Create new package
            let new_package = package::ActiveModel {
                name: Set(name.to_string()),
                description: Set(description.map(|s| s.to_string())),
                repository: Set(repository.map(|s| s.to_string())),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
                ..Default::default()
            };
            
            let result = new_package.insert(&self.db).await?;
            Ok(result.id)
        }
    }

    /// Create a package version
    pub async fn create_package_version(
        &self,
        package_id: i64,
        version: &str,
        description: Option<&str>,
        license: Option<&str>,
        checksum: &str,
        tarball_s3_key: &str,
        tarball_size: i64,
    ) -> Result<i64, DbErr> {
        let now = chrono::Utc::now();
        
        let new_version = package_version::ActiveModel {
            package_id: Set(package_id),
            version: Set(version.to_string()),
            description: Set(description.map(|s| s.to_string())),
            license: Set(license.map(|s| s.to_string())),
            checksum: Set(checksum.to_string()),
            tarball_s3_key: Set(tarball_s3_key.to_string()),
            tarball_size: Set(tarball_size),
            published_at: Set(now.into()),
            downloads: Set(0),
            ..Default::default()
        };
        
        let result = new_version.insert(&self.db).await?;
        Ok(result.id)
    }

    /// Add authors to a package version
    pub async fn add_authors(&self, version_id: i64, authors: &[String]) -> Result<(), DbErr> {
        for author in authors {
            let new_author = package_author::ActiveModel {
                package_version_id: Set(version_id),
                author: Set(author.clone()),
                ..Default::default()
            };
            new_author.insert(&self.db).await?;
        }
        Ok(())
    }

    /// Add keywords to a package
    pub async fn add_keywords(&self, package_id: i64, keywords: &[String]) -> Result<(), DbErr> {
        // Delete existing keywords
        package_keyword::Entity::delete_many()
            .filter(package_keyword::Column::PackageId.eq(package_id))
            .exec(&self.db)
            .await?;
        
        // Add new keywords
        for keyword in keywords {
            let new_keyword = package_keyword::ActiveModel {
                package_id: Set(package_id),
                keyword: Set(keyword.clone()),
                ..Default::default()
            };
            new_keyword.insert(&self.db).await?;
        }
        Ok(())
    }

    /// Add dependencies to a package version
    pub async fn add_dependencies(
        &self,
        version_id: i64,
        dependencies: &HashMap<String, String>,
    ) -> Result<(), DbErr> {
        for (name, constraint) in dependencies {
            let new_dep = package_dependency::ActiveModel {
                package_version_id: Set(version_id),
                dependency_name: Set(name.clone()),
                version_constraint: Set(constraint.clone()),
                ..Default::default()
            };
            new_dep.insert(&self.db).await?;
        }
        Ok(())
    }

    /// Get all packages
    pub async fn list_packages(&self) -> Result<Vec<package::Model>, DbErr> {
        package::Entity::find()
            .order_by_asc(package::Column::Name)
            .all(&self.db)
            .await
    }

    /// Get a package by name
    pub async fn get_package(&self, name: &str) -> Result<Option<package::Model>, DbErr> {
        package::Entity::find()
            .filter(package::Column::Name.eq(name))
            .one(&self.db)
            .await
    }

    /// Get package versions
    pub async fn get_package_versions(&self, package_id: i64) -> Result<Vec<package_version::Model>, DbErr> {
        package_version::Entity::find()
            .filter(package_version::Column::PackageId.eq(package_id))
            .order_by_desc(package_version::Column::PublishedAt)
            .all(&self.db)
            .await
    }

    /// Get a specific package version
    pub async fn get_package_version(
        &self,
        package_id: i64,
        version: &str,
    ) -> Result<Option<package_version::Model>, DbErr> {
        package_version::Entity::find()
            .filter(package_version::Column::PackageId.eq(package_id))
            .filter(package_version::Column::Version.eq(version))
            .one(&self.db)
            .await
    }

    /// Get authors for a version
    pub async fn get_authors(&self, version_id: i64) -> Result<Vec<String>, DbErr> {
        let authors = package_author::Entity::find()
            .filter(package_author::Column::PackageVersionId.eq(version_id))
            .all(&self.db)
            .await?;
        
        Ok(authors.into_iter().map(|a| a.author).collect())
    }

    /// Get keywords for a package
    pub async fn get_keywords(&self, package_id: i64) -> Result<Vec<String>, DbErr> {
        let keywords = package_keyword::Entity::find()
            .filter(package_keyword::Column::PackageId.eq(package_id))
            .all(&self.db)
            .await?;
        
        Ok(keywords.into_iter().map(|k| k.keyword).collect())
    }

    /// Get dependencies for a version
    pub async fn get_dependencies(&self, version_id: i64) -> Result<HashMap<String, String>, DbErr> {
        let deps = package_dependency::Entity::find()
            .filter(package_dependency::Column::PackageVersionId.eq(version_id))
            .all(&self.db)
            .await?;
        
        let mut dependencies = HashMap::new();
        for dep in deps {
            dependencies.insert(dep.dependency_name, dep.version_constraint);
        }
        
        Ok(dependencies)
    }

    /// Increment download counter
    pub async fn increment_downloads(&self, version_id: i64) -> Result<(), DbErr> {
        // Update downloads count
        let version = package_version::Entity::find_by_id(version_id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Package version not found".to_string()))?;
        
        let mut active_model: package_version::ActiveModel = version.into();
        active_model.downloads = Set(active_model.downloads.unwrap() + 1);
        active_model.update(&self.db).await?;
        
        // Record download stat
        let now = chrono::Utc::now();
        let stat = download_stat::ActiveModel {
            package_version_id: Set(version_id),
            downloaded_at: Set(now.into()),
            ip_address: Set(None),
            user_agent: Set(None),
            ..Default::default()
        };
        stat.insert(&self.db).await?;
        
        Ok(())
    }

    /// Search packages
    pub async fn search_packages(&self, query: &str, limit: u64) -> Result<Vec<package::Model>, DbErr> {
        let search_pattern = format!("%{}%", query);
        
        // Search in package names and descriptions
        let packages = package::Entity::find()
            .filter(
                Condition::any()
                    .add(package::Column::Name.like(&search_pattern))
                    .add(package::Column::Description.like(&search_pattern))
            )
            .order_by_asc(package::Column::Name)
            .limit(limit)
            .all(&self.db)
            .await?;
        
        Ok(packages)
    }

    /// Get total downloads for a package
    pub async fn get_total_downloads(&self, package_id: i64) -> Result<i64, DbErr> {
        let versions = package_version::Entity::find()
            .filter(package_version::Column::PackageId.eq(package_id))
            .all(&self.db)
            .await?;
        
        let total: i64 = versions.iter().map(|v| v.downloads).sum();
        Ok(total)
    }
    
    /// Delete a package version
    pub async fn delete_package_version(&self, version_id: i64) -> Result<(), DbErr> {
        package_version::Entity::delete_by_id(version_id)
            .exec(&self.db)
            .await?;
        Ok(())
    }
}

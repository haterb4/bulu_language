//! Storage abstraction for package tarballs

use crate::error::RegistryError;
use std::path::PathBuf;
use tokio::fs;

/// Storage backend trait for different storage implementations
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    async fn store_tarball(
        &self,
        package_name: &str,
        version: &str,
        tarball_data: &[u8],
    ) -> Result<String, RegistryError>;

    async fn retrieve_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<u8>, RegistryError>;

    async fn delete_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<(), RegistryError>;

    async fn list_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, RegistryError>;
}

/// Local filesystem storage implementation
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

#[async_trait::async_trait]
impl StorageBackend for LocalStorage {
    async fn store_tarball(
        &self,
        package_name: &str,
        version: &str,
        tarball_data: &[u8],
    ) -> Result<String, RegistryError> {
        let package_dir = self.base_path.join("packages").join(package_name);
        fs::create_dir_all(&package_dir).await
            .map_err(|e| RegistryError::StorageError(format!("Failed to create directory: {}", e)))?;

        let tarball_path = package_dir.join(format!("{}.tar.gz", version));
        fs::write(&tarball_path, tarball_data).await
            .map_err(|e| RegistryError::StorageError(format!("Failed to write tarball: {}", e)))?;

        Ok(tarball_path.to_string_lossy().to_string())
    }

    async fn retrieve_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<u8>, RegistryError> {
        let tarball_path = self.base_path
            .join("packages")
            .join(package_name)
            .join(format!("{}.tar.gz", version));

        fs::read(&tarball_path).await
            .map_err(|e| RegistryError::StorageError(format!("Failed to read tarball: {}", e)))
    }

    async fn delete_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<(), RegistryError> {
        let tarball_path = self.base_path
            .join("packages")
            .join(package_name)
            .join(format!("{}.tar.gz", version));

        fs::remove_file(&tarball_path).await
            .map_err(|e| RegistryError::StorageError(format!("Failed to delete tarball: {}", e)))
    }

    async fn list_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, RegistryError> {
        let package_dir = self.base_path.join("packages").join(package_name);
        
        if !package_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&package_dir).await
            .map_err(|e| RegistryError::StorageError(format!("Failed to read directory: {}", e)))?;

        let mut versions = Vec::new();
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| RegistryError::StorageError(format!("Failed to read entry: {}", e)))? {
            
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".tar.gz") {
                    let version = file_name.strip_suffix(".tar.gz").unwrap();
                    versions.push(version.to_string());
                }
            }
        }

        Ok(versions)
    }
}

//! Package registry client for interacting with pkg.lang-lang.org

use super::{PackageConfig, PackageMetadata, VersionConstraint};
use crate::{BuluError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};

/// Registry client for package operations
#[derive(Clone)]
pub struct RegistryClient {
    config: PackageConfig,
    http_client: reqwest::Client,
}

/// Search result from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub packages: Vec<PackageSearchInfo>,
    pub total: usize,
}

/// Package information in search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSearchInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub downloads: u64,
    pub updated_at: String,
}

/// Registry API response for package metadata
#[derive(Debug, Serialize, Deserialize)]
struct RegistryPackageResponse {
    pub package: PackageMetadata,
    pub versions: Vec<String>,
}

/// Registry API response for search
#[derive(Debug, Serialize, Deserialize)]
struct RegistrySearchResponse {
    pub results: Vec<PackageSearchInfo>,
    pub total: usize,
}

/// Publish request to registry
#[derive(Debug, Serialize)]
struct PublishRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub dependencies: HashMap<String, VersionConstraint>,
    pub tarball: Vec<u8>,
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(config: PackageConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("bulu-lang/{}", crate::VERSION))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    /// Search for packages in the registry
    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<SearchResult> {
        let url = format!("{}/api/v1/search", self.config.registry_url);
        let limit = limit.unwrap_or(20);

        let response = self
            .http_client
            .get(&url)
            .query(&[("q", query), ("limit", &limit.to_string())])
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to search packages: {}", e)))?;

        if !response.status().is_success() {
            return Err(BuluError::Other(format!(
                "Registry search failed: {}",
                response.status()
            )));
        }

        let search_response: RegistrySearchResponse = response
            .json()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to parse search response: {}", e)))?;

        Ok(SearchResult {
            packages: search_response.results,
            total: search_response.total,
        })
    }

    /// Get package metadata from registry
    pub async fn get_package(&self, name: &str, version: Option<&str>) -> Result<PackageMetadata> {
        let url = if let Some(version) = version {
            format!("{}/api/v1/packages/{}/{}", self.config.registry_url, name, version)
        } else {
            format!("{}/api/v1/packages/{}", self.config.registry_url, name)
        };

        // Check cache first
        if let Ok(cached) = self.get_cached_package(name, version) {
            return Ok(cached);
        }

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to fetch package {}: {}", name, e)))?;

        if !response.status().is_success() {
            return Err(BuluError::Other(format!(
                "Package {} not found: {}",
                name,
                response.status()
            )));
        }

        let package_response: RegistryPackageResponse = response
            .json()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to parse package response: {}", e)))?;

        // Cache the result
        self.cache_package(&package_response.package)?;

        Ok(package_response.package)
    }

    /// Get all available versions for a package
    pub async fn get_package_versions(&self, name: &str) -> Result<Vec<String>> {
        let url = format!("{}/api/v1/packages/{}/versions", self.config.registry_url, name);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to fetch versions for {}: {}", name, e)))?;

        if !response.status().is_success() {
            return Err(BuluError::Other(format!(
                "Failed to get versions for {}: {}",
                name,
                response.status()
            )));
        }

        let versions: Vec<String> = response
            .json()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to parse versions response: {}", e)))?;

        Ok(versions)
    }

    /// Download a package tarball
    pub async fn download_package(&self, name: &str, version: &str) -> Result<Vec<u8>> {
        let package = self.get_package(name, Some(version)).await?;
        
        let response = self
            .http_client
            .get(&package.download_url)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to download package {}: {}", name, e)))?;

        if !response.status().is_success() {
            return Err(BuluError::Other(format!(
                "Failed to download package {}: {}",
                name,
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to read package data: {}", e)))?;

        // Verify checksum
        let actual_checksum = sha256::digest(bytes.as_ref());
        if actual_checksum != package.checksum {
            return Err(BuluError::Other(format!(
                "Checksum mismatch for package {}: expected {}, got {}",
                name, package.checksum, actual_checksum
            )));
        }

        Ok(bytes.to_vec())
    }

    /// Publish a package to the registry
    pub async fn publish_package(
        &self,
        metadata: &PackageMetadata,
        tarball: Vec<u8>,
    ) -> Result<()> {
        let url = format!("{}/api/v1/packages", self.config.registry_url);

        let auth_token = self.config.auth_token.as_ref()
            .ok_or_else(|| BuluError::Other("No authentication token configured".to_string()))?;

        let publish_request = PublishRequest {
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            description: metadata.description.clone(),
            authors: metadata.authors.clone(),
            license: metadata.license.clone(),
            repository: metadata.repository.clone(),
            keywords: metadata.keywords.clone(),
            categories: metadata.categories.clone(),
            dependencies: metadata.dependencies.clone(),
            tarball,
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .json(&publish_request)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to publish package: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BuluError::Other(format!(
                "Failed to publish package: {} - {}",
                status,
                error_text
            )));
        }

        Ok(())
    }

    /// Get cached package metadata
    fn get_cached_package(&self, name: &str, version: Option<&str>) -> Result<PackageMetadata> {
        let cache_key = if let Some(version) = version {
            format!("{}@{}", name, version)
        } else {
            name.to_string()
        };

        let cache_path = self.config.cache_dir.join("packages").join(format!("{}.json", cache_key));
        
        if !cache_path.exists() {
            return Err(BuluError::Other("Not cached".to_string()));
        }

        // Check if cache is expired (24 hours)
        let metadata = fs::metadata(&cache_path)
            .map_err(|e| BuluError::Other(format!("Failed to read cache metadata: {}", e)))?;
        
        let modified = metadata
            .modified()
            .map_err(|e| BuluError::Other(format!("Failed to get cache modification time: {}", e)))?;
        
        let now = SystemTime::now();
        let age = now.duration_since(modified)
            .map_err(|e| BuluError::Other(format!("Failed to calculate cache age: {}", e)))?;
        
        if age > Duration::from_secs(24 * 60 * 60) {
            return Err(BuluError::Other("Cache expired".to_string()));
        }

        let content = fs::read_to_string(&cache_path)
            .map_err(|e| BuluError::Other(format!("Failed to read cache file: {}", e)))?;
        
        let package: PackageMetadata = serde_json::from_str(&content)
            .map_err(|e| BuluError::Other(format!("Failed to parse cached package: {}", e)))?;

        Ok(package)
    }

    /// Cache package metadata
    fn cache_package(&self, package: &PackageMetadata) -> Result<()> {
        let cache_key = format!("{}@{}", package.name, package.version);
        let cache_dir = self.config.cache_dir.join("packages");
        let cache_path = cache_dir.join(format!("{}.json", cache_key));

        fs::create_dir_all(&cache_dir)
            .map_err(|e| BuluError::Other(format!("Failed to create cache directory: {}", e)))?;

        let content = serde_json::to_string_pretty(package)
            .map_err(|e| BuluError::Other(format!("Failed to serialize package: {}", e)))?;

        fs::write(&cache_path, content)
            .map_err(|e| BuluError::Other(format!("Failed to write cache file: {}", e)))?;

        Ok(())
    }

    /// Clear package cache
    pub fn clear_cache(&self) -> Result<()> {
        let cache_dir = self.config.cache_dir.join("packages");
        
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)
                .map_err(|e| BuluError::Other(format!("Failed to clear cache: {}", e)))?;
        }

        Ok(())
    }
}

// Mock registry client for testing
#[cfg(test)]
pub struct MockRegistryClient {
    packages: HashMap<String, PackageMetadata>,
}

#[cfg(test)]
impl MockRegistryClient {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    pub fn add_package(&mut self, package: PackageMetadata) {
        let key = format!("{}@{}", package.name, package.version);
        self.packages.insert(key, package);
    }

    pub async fn get_package(&self, name: &str, version: Option<&str>) -> Result<PackageMetadata> {
        let key = if let Some(version) = version {
            format!("{}@{}", name, version)
        } else {
            // Find latest version
            let mut latest_version = None;
            let mut latest_package = None;

            for (key, package) in &self.packages {
                if package.name == name {
                    if latest_version.is_none() || 
                       super::compare_versions(&package.version, latest_version.unwrap()) > 0 {
                        latest_version = Some(&package.version);
                        latest_package = Some(package);
                    }
                }
            }

            return latest_package.cloned()
                .ok_or_else(|| BuluError::Other(format!("Package {} not found", name)));
        };

        self.packages.get(&key).cloned()
            .ok_or_else(|| BuluError::Other(format!("Package {} not found", key)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_registry() {
        let mut registry = MockRegistryClient::new();
        
        let package = PackageMetadata {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test package".to_string()),
            authors: vec!["Test Author".to_string()],
            license: Some("MIT".to_string()),
            repository: None,
            keywords: vec![],
            categories: vec![],
            dependencies: HashMap::new(),
            checksum: "abc123".to_string(),
            download_url: "https://example.com/package.tar.gz".to_string(),
        };

        registry.add_package(package.clone());

        // Test async in sync context using block_on
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(registry.get_package("test-package", Some("1.0.0")));
        
        assert!(result.is_ok());
        let retrieved = result.unwrap();
        assert_eq!(retrieved.name, "test-package");
        assert_eq!(retrieved.version, "1.0.0");
    }
}
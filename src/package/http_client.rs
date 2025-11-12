//! HTTP client for communicating with the Bulu package registry

use super::{PackageMetadata, VersionConstraint};
use crate::{BuluError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP client for the package registry
#[derive(Clone)]
pub struct RegistryHttpClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub tarball: Vec<u8>, // Raw bytes
}

#[derive(Debug, Deserialize)]
pub struct PackageListResponse {
    pub packages: Vec<PackageListItem>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct PackageListItem {
    pub name: String,
    pub latest_version: Option<String>,
    pub description: Option<String>,
    pub downloads: u64,
}

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub versions: Vec<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub downloads: u64,
}

#[derive(Debug, Deserialize)]
pub struct PackageVersionInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub checksum: String,
    pub published_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub packages: Vec<SearchPackage>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct SearchPackage {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub downloads: u64,
    pub updated_at: String,
}

impl RegistryHttpClient {
    /// Create a new HTTP client
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        Self { base_url, client }
    }

    /// List all packages
    pub async fn list_packages(&self) -> Result<PackageListResponse> {
        let url = format!("{}/api/packages", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to list packages: {}", e)))?;

        if !response.status().is_success() {
            return Err(BuluError::Other(format!("Registry error: {}", response.status())));
        }

        response
            .json()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to parse response: {}", e)))
    }

    /// Get package information
    pub async fn get_package(&self, name: &str) -> Result<PackageInfo> {
        let url = format!("{}/api/packages/{}", self.base_url, name);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to get package: {}", e)))?;

        if !response.status().is_success() {
            return Err(BuluError::Other(format!("Package not found: {}", name)));
        }

        response
            .json()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to parse response: {}", e)))
    }

    /// Get package versions
    pub async fn get_package_versions(&self, name: &str) -> Result<Vec<String>> {
        let package = self.get_package(name).await?;
        Ok(package.versions)
    }

    /// Get specific package version info
    pub async fn get_package_version(&self, name: &str, version: &str) -> Result<PackageVersionInfo> {
        let url = format!("{}/api/packages/{}/{}", self.base_url, name, version);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to get package version: {}", e)))?;

        if !response.status().is_success() {
            return Err(BuluError::Other(format!("Version not found: {} v{}", name, version)));
        }

        response
            .json()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to parse response: {}", e)))
    }

    /// Download package tarball
    pub async fn download_package(&self, name: &str, version: &str) -> Result<Vec<u8>> {
        let url = format!("{}/api/download/{}/{}", self.base_url, name, version);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to download package: {}", e)))?;

        if !response.status().is_success() {
            return Err(BuluError::Other(format!("Failed to download: {} v{}", name, version)));
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| BuluError::Other(format!("Failed to read package data: {}", e)))
    }

    /// Search for packages
    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<SearchResponse> {
        let limit = limit.unwrap_or(20);
        let url = format!("{}/api/search?q={}&limit={}", self.base_url, query, limit);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to search: {}", e)))?;

        if !response.status().is_success() {
            return Err(BuluError::Other(format!("Search failed: {}", response.status())));
        }

        response
            .json()
            .await
            .map_err(|e| BuluError::Other(format!("Failed to parse response: {}", e)))
    }

    /// Publish a package
    pub async fn publish(&self, request: PublishRequest) -> Result<()> {
        let url = format!("{}/api/packages/{}/{}", self.base_url, request.name, request.version);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| BuluError::Other(format!("Network error while publishing to {}: {}", self.base_url, e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(BuluError::Other(format!(
                "Registry returned error (HTTP {}): {}",
                status.as_u16(),
                error_text
            )));
        }

        Ok(())
    }

    /// Find the latest version matching a constraint
    pub async fn find_matching_version(&self, name: &str, constraint: &VersionConstraint) -> Result<String> {
        let versions = self.get_package_versions(name).await?;
        
        // Find the latest version that satisfies the constraint
        for version in versions.iter().rev() {
            if constraint.satisfies(version) {
                return Ok(version.clone());
            }
        }

        Err(BuluError::Other(format!(
            "No version of {} satisfies constraint {:?}",
            name, constraint
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running registry server
    async fn test_list_packages() {
        let client = RegistryHttpClient::new("http://localhost:3000".to_string());
        let result = client.list_packages().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_search() {
        let client = RegistryHttpClient::new("http://localhost:3000".to_string());
        let result = client.search("math", Some(10)).await;
        assert!(result.is_ok());
    }
}

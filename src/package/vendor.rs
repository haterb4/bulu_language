//! Vendoring support for local dependencies

use super::lockfile::{LockFile, LockedDependency, LockedSource};
use super::registry::RegistryClient;
use crate::{BuluError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Vendor manager for handling local dependency copies
pub struct VendorManager {
    vendor_dir: PathBuf,
    registry: RegistryClient,
}

/// Vendoring options
#[derive(Debug, Clone)]
pub struct VendorOptions {
    /// Whether to update existing vendored dependencies
    pub update_existing: bool,
    /// Whether to verify checksums
    pub verify_checksums: bool,
    /// Whether to include dev dependencies
    pub include_dev_deps: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for VendorOptions {
    fn default() -> Self {
        Self {
            update_existing: false,
            verify_checksums: true,
            include_dev_deps: false,
            verbose: false,
        }
    }
}

impl VendorManager {
    /// Create a new vendor manager
    pub fn new<P: AsRef<Path>>(project_root: P, registry: RegistryClient) -> Self {
        Self {
            vendor_dir: project_root.as_ref().join("vendor"),
            registry,
        }
    }

    /// Vendor all dependencies from lock file
    pub async fn vendor_dependencies(
        &self,
        lock_file: &LockFile,
        options: &VendorOptions,
    ) -> Result<VendorResult> {
        let mut result = VendorResult::default();

        // Create vendor directory
        fs::create_dir_all(&self.vendor_dir)
            .map_err(|e| BuluError::Other(format!("Failed to create vendor directory: {}", e)))?;

        // Get dependencies in resolution order
        let resolution_order = lock_file.get_resolution_order()?;

        for dep_name in resolution_order {
            if let Some(locked_dep) = lock_file.dependencies.get(&dep_name) {
                match self.vendor_single_dependency(locked_dep, options).await {
                    Ok(vendor_info) => {
                        result.vendored.push(vendor_info);
                        if options.verbose {
                            println!("Vendored: {} v{}", locked_dep.name, locked_dep.version);
                        }
                    }
                    Err(e) => {
                        result.errors.push(format!("Failed to vendor {}: {}", dep_name, e));
                        if options.verbose {
                            eprintln!("Error vendoring {}: {}", dep_name, e);
                        }
                    }
                }
            }
        }

        // Clean up unused vendored dependencies
        self.cleanup_unused_dependencies(lock_file, options)?;

        Ok(result)
    }

    /// Vendor a single dependency
    async fn vendor_single_dependency(
        &self,
        locked_dep: &LockedDependency,
        options: &VendorOptions,
    ) -> Result<VendoredDependency> {
        let dep_vendor_dir = self.vendor_dir.join(&locked_dep.name);

        // Check if already vendored and up to date
        if dep_vendor_dir.exists() && !options.update_existing {
            if let Ok(existing_info) = self.read_vendor_info(&dep_vendor_dir) {
                if existing_info.version == locked_dep.version {
                    return Ok(VendoredDependency {
                        name: locked_dep.name.clone(),
                        version: locked_dep.version.clone(),
                        path: dep_vendor_dir,
                        source: locked_dep.source.clone(),
                        checksum: locked_dep.checksum.clone(),
                    });
                }
            }
        }

        // Remove existing if updating
        if dep_vendor_dir.exists() {
            fs::remove_dir_all(&dep_vendor_dir)
                .map_err(|e| BuluError::Other(format!("Failed to remove existing vendor directory: {}", e)))?;
        }

        // Vendor based on source type
        match &locked_dep.source {
            LockedSource::Registry { url: _, checksum: _ } => {
                self.vendor_registry_dependency(locked_dep, &dep_vendor_dir, options).await?;
            }
            LockedSource::Path { path } => {
                self.vendor_path_dependency(locked_dep, &dep_vendor_dir, Path::new(path))?;
            }
            LockedSource::Git { url, commit, branch, tag } => {
                self.vendor_git_dependency(locked_dep, &dep_vendor_dir, url, commit, branch.as_deref(), tag.as_deref()).await?;
            }
        }

        // Write vendor info
        self.write_vendor_info(&dep_vendor_dir, locked_dep)?;

        Ok(VendoredDependency {
            name: locked_dep.name.clone(),
            version: locked_dep.version.clone(),
            path: dep_vendor_dir,
            source: locked_dep.source.clone(),
            checksum: locked_dep.checksum.clone(),
        })
    }

    /// Vendor a registry dependency
    async fn vendor_registry_dependency(
        &self,
        locked_dep: &LockedDependency,
        vendor_path: &Path,
        options: &VendorOptions,
    ) -> Result<()> {
        // Download package tarball
        let tarball = self.registry.download_package(&locked_dep.name, &locked_dep.version).await?;

        // Verify checksum if requested
        if options.verify_checksums {
            if let Some(expected_checksum) = &locked_dep.checksum {
                let actual_checksum = sha256::digest(&tarball);
                if &actual_checksum != expected_checksum {
                    return Err(BuluError::Other(format!(
                        "Checksum mismatch for {}: expected {}, got {}",
                        locked_dep.name, expected_checksum, actual_checksum
                    )));
                }
            }
        }

        // Extract tarball
        self.extract_tarball(&tarball, vendor_path)?;

        Ok(())
    }

    /// Vendor a path dependency
    fn vendor_path_dependency(
        &self,
        _locked_dep: &LockedDependency,
        vendor_path: &Path,
        source_path: &Path,
    ) -> Result<()> {
        if !source_path.exists() {
            return Err(BuluError::Other(format!(
                "Path dependency not found: {}",
                source_path.display()
            )));
        }

        // Copy the entire directory
        self.copy_directory(source_path, vendor_path)?;

        Ok(())
    }

    /// Vendor a git dependency
    async fn vendor_git_dependency(
        &self,
        _locked_dep: &LockedDependency,
        vendor_path: &Path,
        git_url: &str,
        commit: &str,
        branch: Option<&str>,
        tag: Option<&str>,
    ) -> Result<()> {
        // For now, this is a placeholder implementation
        // Full git support would require git2 or similar
        
        // Create a placeholder directory with git info
        fs::create_dir_all(vendor_path)
            .map_err(|e| BuluError::Other(format!("Failed to create vendor directory: {}", e)))?;

        let git_info = format!(
            "# Git dependency: {}\n# Commit: {}\n# Branch: {:?}\n# Tag: {:?}\n",
            git_url, commit, branch, tag
        );

        fs::write(vendor_path.join("GIT_INFO"), git_info)
            .map_err(|e| BuluError::Other(format!("Failed to write git info: {}", e)))?;

        // TODO: Implement actual git cloning and checkout
        Err(BuluError::Other("Git dependencies not yet fully implemented".to_string()))
    }

    /// Extract a tarball to the specified directory
    fn extract_tarball(&self, tarball: &[u8], extract_path: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;
        use std::io::Cursor;

        let cursor = Cursor::new(tarball);
        let decoder = GzDecoder::new(cursor);
        let mut archive = Archive::new(decoder);

        fs::create_dir_all(extract_path)
            .map_err(|e| BuluError::Other(format!("Failed to create extract directory: {}", e)))?;

        archive.unpack(extract_path)
            .map_err(|e| BuluError::Other(format!("Failed to extract tarball: {}", e)))?;

        Ok(())
    }

    /// Copy a directory recursively
    fn copy_directory(&self, source: &Path, dest: &Path) -> Result<()> {
        fs::create_dir_all(dest)
            .map_err(|e| BuluError::Other(format!("Failed to create destination directory: {}", e)))?;

        for entry in fs::read_dir(source)
            .map_err(|e| BuluError::Other(format!("Failed to read source directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| BuluError::Other(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if path.is_dir() {
                self.copy_directory(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)
                    .map_err(|e| BuluError::Other(format!("Failed to copy file: {}", e)))?;
            }
        }

        Ok(())
    }

    /// Write vendor info file
    fn write_vendor_info(&self, vendor_path: &Path, locked_dep: &LockedDependency) -> Result<()> {
        let vendor_info = VendorInfo {
            name: locked_dep.name.clone(),
            version: locked_dep.version.clone(),
            source: locked_dep.source.clone(),
            checksum: locked_dep.checksum.clone(),
            vendored_at: chrono::Utc::now().to_rfc3339(),
        };

        let info_content = toml::to_string_pretty(&vendor_info)
            .map_err(|e| BuluError::Other(format!("Failed to serialize vendor info: {}", e)))?;

        fs::write(vendor_path.join(".vendor_info"), info_content)
            .map_err(|e| BuluError::Other(format!("Failed to write vendor info: {}", e)))?;

        Ok(())
    }

    /// Read vendor info file
    fn read_vendor_info(&self, vendor_path: &Path) -> Result<VendorInfo> {
        let info_path = vendor_path.join(".vendor_info");
        let content = fs::read_to_string(&info_path)
            .map_err(|e| BuluError::Other(format!("Failed to read vendor info: {}", e)))?;

        let vendor_info: VendorInfo = toml::from_str(&content)
            .map_err(|e| BuluError::Other(format!("Failed to parse vendor info: {}", e)))?;

        Ok(vendor_info)
    }

    /// Clean up unused vendored dependencies
    fn cleanup_unused_dependencies(&self, lock_file: &LockFile, options: &VendorOptions) -> Result<()> {
        if !self.vendor_dir.exists() {
            return Ok(());
        }

        let lock_dep_names: std::collections::HashSet<_> = lock_file.dependencies.keys().collect();

        for entry in fs::read_dir(&self.vendor_dir)
            .map_err(|e| BuluError::Other(format!("Failed to read vendor directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| BuluError::Other(format!("Failed to read vendor entry: {}", e)))?;
            
            if entry.path().is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                
                if !lock_dep_names.contains(&dir_name) {
                    if options.verbose {
                        println!("Removing unused vendored dependency: {}", dir_name);
                    }
                    
                    fs::remove_dir_all(entry.path())
                        .map_err(|e| BuluError::Other(format!("Failed to remove unused dependency: {}", e)))?;
                }
            }
        }

        Ok(())
    }

    /// Check if dependencies are vendored
    pub fn check_vendored_status(&self, lock_file: &LockFile) -> Result<VendorStatus> {
        let mut status = VendorStatus {
            total_dependencies: lock_file.dependencies.len(),
            vendored_dependencies: 0,
            missing_dependencies: Vec::new(),
            outdated_dependencies: Vec::new(),
        };

        for (name, locked_dep) in &lock_file.dependencies {
            let dep_vendor_dir = self.vendor_dir.join(name);
            
            if !dep_vendor_dir.exists() {
                status.missing_dependencies.push(name.clone());
                continue;
            }

            match self.read_vendor_info(&dep_vendor_dir) {
                Ok(vendor_info) => {
                    if vendor_info.version != locked_dep.version {
                        status.outdated_dependencies.push(name.clone());
                    } else {
                        status.vendored_dependencies += 1;
                    }
                }
                Err(_) => {
                    status.missing_dependencies.push(name.clone());
                }
            }
        }

        Ok(status)
    }
}

/// Result of vendoring operation
#[derive(Debug, Default)]
pub struct VendorResult {
    pub vendored: Vec<VendoredDependency>,
    pub errors: Vec<String>,
}

/// Information about a vendored dependency
#[derive(Debug, Clone)]
pub struct VendoredDependency {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub source: LockedSource,
    pub checksum: Option<String>,
}

/// Vendor info stored with each vendored dependency
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct VendorInfo {
    pub name: String,
    pub version: String,
    pub source: LockedSource,
    pub checksum: Option<String>,
    pub vendored_at: String,
}

/// Status of vendored dependencies
#[derive(Debug)]
pub struct VendorStatus {
    pub total_dependencies: usize,
    pub vendored_dependencies: usize,
    pub missing_dependencies: Vec<String>,
    pub outdated_dependencies: Vec<String>,
}

impl VendorStatus {
    /// Check if all dependencies are properly vendored
    pub fn is_complete(&self) -> bool {
        self.missing_dependencies.is_empty() && self.outdated_dependencies.is_empty()
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_dependencies == 0 {
            100.0
        } else {
            (self.vendored_dependencies as f64 / self.total_dependencies as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_vendor_status() {
        let status = VendorStatus {
            total_dependencies: 10,
            vendored_dependencies: 8,
            missing_dependencies: vec!["dep1".to_string()],
            outdated_dependencies: vec!["dep2".to_string()],
        };

        assert!(!status.is_complete());
        assert_eq!(status.completion_percentage(), 80.0);
    }

    #[test]
    fn test_vendor_info_serialization() {
        let vendor_info = VendorInfo {
            name: "test-lib".to_string(),
            version: "1.0.0".to_string(),
            source: LockedSource::Registry {
                url: "https://example.com/test-lib".to_string(),
                checksum: "abc123".to_string(),
            },
            checksum: Some("abc123".to_string()),
            vendored_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let serialized = toml::to_string_pretty(&vendor_info).unwrap();
        let deserialized: VendorInfo = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, "test-lib");
        assert_eq!(deserialized.version, "1.0.0");
    }
}
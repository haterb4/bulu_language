///! Local package registry for development and testing
///! Stores packages in ~/.bulu/packages or .bulu/packages

use super::{PackageMetadata, ResolvedDependency, DependencySource, VersionConstraint};
use crate::{BuluError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Local package registry
pub struct LocalRegistry {
    root_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalPackageIndex {
    packages: HashMap<String, Vec<LocalPackageEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalPackageEntry {
    version: String,
    path: PathBuf,
    metadata: PackageMetadata,
}

impl LocalRegistry {
    /// Create a new local registry
    pub fn new() -> Result<Self> {
        let root_dir = if let Some(home) = dirs::home_dir() {
            home.join(".bulu").join("packages")
        } else {
            PathBuf::from(".bulu").join("packages")
        };

        fs::create_dir_all(&root_dir)
            .map_err(|e| BuluError::Other(format!("Failed to create registry directory: {}", e)))?;

        Ok(Self { root_dir })
    }

    /// Install a package from a local path
    pub fn install_from_path(&self, source_path: &Path, name: &str, version: &str) -> Result<()> {
        let package_dir = self.root_dir.join(name).join(version);
        
        if package_dir.exists() {
            return Ok(()); // Already installed
        }

        fs::create_dir_all(&package_dir)
            .map_err(|e| BuluError::Other(format!("Failed to create package directory: {}", e)))?;

        // Copy source files
        self.copy_dir_recursive(source_path, &package_dir)?;

        Ok(())
    }

    /// Get package path
    pub fn get_package_path(&self, name: &str, version: &str) -> Option<PathBuf> {
        let package_dir = self.root_dir.join(name).join(version);
        if package_dir.exists() {
            Some(package_dir)
        } else {
            None
        }
    }

    /// List all installed packages
    pub fn list_packages(&self) -> Result<Vec<(String, String)>> {
        let mut packages = Vec::new();

        if !self.root_dir.exists() {
            return Ok(packages);
        }

        for entry in fs::read_dir(&self.root_dir)
            .map_err(|e| BuluError::Other(format!("Failed to read registry: {}", e)))? 
        {
            let entry = entry.map_err(|e| BuluError::Other(format!("Failed to read entry: {}", e)))?;
            let name = entry.file_name().to_string_lossy().to_string();

            if entry.path().is_dir() {
                for version_entry in fs::read_dir(entry.path())
                    .map_err(|e| BuluError::Other(format!("Failed to read versions: {}", e)))? 
                {
                    let version_entry = version_entry.map_err(|e| BuluError::Other(format!("Failed to read version: {}", e)))?;
                    let version = version_entry.file_name().to_string_lossy().to_string();
                    packages.push((name.clone(), version));
                }
            }
        }

        Ok(packages)
    }

    /// Helper: Copy directory recursively
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)
            .map_err(|e| BuluError::Other(format!("Failed to create directory: {}", e)))?;

        for entry in fs::read_dir(src)
            .map_err(|e| BuluError::Other(format!("Failed to read source directory: {}", e)))? 
        {
            let entry = entry.map_err(|e| BuluError::Other(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dst.join(&file_name);

            // Skip hidden files and build artifacts
            if file_name.to_string_lossy().starts_with('.') || 
               file_name == "target" || 
               file_name == "vendor" {
                continue;
            }

            if path.is_dir() {
                self.copy_dir_recursive(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)
                    .map_err(|e| BuluError::Other(format!("Failed to copy file: {}", e)))?;
            }
        }

        Ok(())
    }
}

impl Default for LocalRegistry {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

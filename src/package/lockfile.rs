//! Lock file generation and management for reproducible builds

use super::{ResolvedDependency, DependencySource};
use crate::{BuluError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Lock file structure for reproducible builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    /// Version of the lock file format
    pub version: String,
    /// Resolved dependencies with exact versions
    pub dependencies: HashMap<String, LockedDependency>,
    /// Metadata about the lock file
    pub metadata: LockFileMetadata,
}

/// Locked dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedDependency {
    /// Package name
    pub name: String,
    /// Exact version
    pub version: String,
    /// Source information
    pub source: LockedSource,
    /// Checksum for integrity verification
    pub checksum: Option<String>,
    /// Direct dependencies of this package
    pub dependencies: Vec<String>,
}

/// Locked source information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LockedSource {
    #[serde(rename = "registry")]
    Registry {
        url: String,
        checksum: String,
    },
    #[serde(rename = "path")]
    Path {
        path: String,
    },
    #[serde(rename = "git")]
    Git {
        url: String,
        commit: String,
        branch: Option<String>,
        tag: Option<String>,
    },
}

/// Lock file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFileMetadata {
    /// When the lock file was generated
    pub generated_at: String,
    /// Version of the package manager that generated this
    pub generator: String,
    /// Root package information
    pub root_package: Option<RootPackageInfo>,
}

/// Root package information in lock file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootPackageInfo {
    pub name: String,
    pub version: String,
}

impl LockFile {
    /// Create a new lock file from resolved dependencies
    pub fn from_resolved_dependencies(
        dependencies: &HashMap<String, ResolvedDependency>,
        root_package: Option<RootPackageInfo>,
    ) -> Self {
        let locked_deps = dependencies
            .iter()
            .map(|(name, resolved)| {
                let locked_source = match &resolved.source {
                    DependencySource::Registry { url } => LockedSource::Registry {
                        url: url.clone(),
                        checksum: resolved.checksum.clone().unwrap_or_default(),
                    },
                    DependencySource::Path { path } => LockedSource::Path {
                        path: path.to_string_lossy().to_string(),
                    },
                    DependencySource::Git { url, branch, tag, commit } => LockedSource::Git {
                        url: url.clone(),
                        commit: commit.clone().unwrap_or_else(|| "HEAD".to_string()),
                        branch: branch.clone(),
                        tag: tag.clone(),
                    },
                };

                let locked_dep = LockedDependency {
                    name: resolved.name.clone(),
                    version: resolved.version.clone(),
                    source: locked_source,
                    checksum: resolved.checksum.clone(),
                    dependencies: resolved.dependencies.keys().cloned().collect(),
                };

                (name.clone(), locked_dep)
            })
            .collect();

        let metadata = LockFileMetadata {
            generated_at: chrono::Utc::now().to_rfc3339(),
            generator: format!("bulu-lang/{}", crate::VERSION),
            root_package,
        };

        Self {
            version: "1".to_string(),
            dependencies: locked_deps,
            metadata,
        }
    }

    /// Load lock file from path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| BuluError::Other(format!("Failed to read lock file: {}", e)))?;

        let lock_file: LockFile = toml::from_str(&content)
            .map_err(|e| BuluError::Other(format!("Failed to parse lock file: {}", e)))?;

        // Validate lock file version
        if lock_file.version != "1" {
            return Err(BuluError::Other(format!(
                "Unsupported lock file version: {}",
                lock_file.version
            )));
        }

        Ok(lock_file)
    }

    /// Save lock file to path
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| BuluError::Other(format!("Failed to serialize lock file: {}", e)))?;

        fs::write(path.as_ref(), content)
            .map_err(|e| BuluError::Other(format!("Failed to write lock file: {}", e)))?;

        Ok(())
    }

    /// Check if lock file is up to date with project dependencies
    pub fn is_up_to_date(&self, project_deps: &HashMap<String, crate::project::DependencySpec>) -> bool {
        // Check if all project dependencies are in the lock file
        for dep_name in project_deps.keys() {
            if !self.dependencies.contains_key(dep_name) {
                return false;
            }
        }

        // Check if lock file has extra dependencies (could indicate removed deps)
        let project_dep_names: std::collections::HashSet<_> = project_deps.keys().collect();
        let lock_dep_names: std::collections::HashSet<_> = self.dependencies.keys().collect();

        // Allow lock file to have more dependencies (transitive deps)
        // but all project deps should be present
        project_dep_names.is_subset(&lock_dep_names)
    }

    /// Get dependency resolution order (topological sort)
    pub fn get_resolution_order(&self) -> Result<Vec<String>> {
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();
        let mut result = Vec::new();

        for dep_name in self.dependencies.keys() {
            if !visited.contains(dep_name) {
                self.visit_dependency(dep_name, &mut visited, &mut temp_visited, &mut result)?;
            }
        }

        Ok(result)
    }

    /// Recursive helper for topological sort
    fn visit_dependency(
        &self,
        dep_name: &str,
        visited: &mut std::collections::HashSet<String>,
        temp_visited: &mut std::collections::HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<()> {
        if temp_visited.contains(dep_name) {
            return Err(BuluError::Other(format!(
                "Circular dependency detected involving: {}",
                dep_name
            )));
        }

        if visited.contains(dep_name) {
            return Ok(());
        }

        temp_visited.insert(dep_name.to_string());

        if let Some(locked_dep) = self.dependencies.get(dep_name) {
            for child_dep in &locked_dep.dependencies {
                self.visit_dependency(child_dep, visited, temp_visited, result)?;
            }
        }

        temp_visited.remove(dep_name);
        visited.insert(dep_name.to_string());
        result.push(dep_name.to_string());

        Ok(())
    }

    /// Validate lock file integrity
    pub fn validate(&self) -> Result<()> {
        // Check that all referenced dependencies exist
        for (name, locked_dep) in &self.dependencies {
            for child_dep in &locked_dep.dependencies {
                if !self.dependencies.contains_key(child_dep) {
                    return Err(BuluError::Other(format!(
                        "Lock file references missing dependency: {} -> {}",
                        name, child_dep
                    )));
                }
            }
        }

        // Check for circular dependencies
        self.get_resolution_order()?;

        Ok(())
    }

    /// Get all registry dependencies that need to be downloaded
    pub fn get_registry_dependencies(&self) -> Vec<&LockedDependency> {
        self.dependencies
            .values()
            .filter(|dep| matches!(dep.source, LockedSource::Registry { .. }))
            .collect()
    }

    /// Get all path dependencies
    pub fn get_path_dependencies(&self) -> Vec<&LockedDependency> {
        self.dependencies
            .values()
            .filter(|dep| matches!(dep.source, LockedSource::Path { .. }))
            .collect()
    }

    /// Get all git dependencies
    pub fn get_git_dependencies(&self) -> Vec<&LockedDependency> {
        self.dependencies
            .values()
            .filter(|dep| matches!(dep.source, LockedSource::Git { .. }))
            .collect()
    }

    /// Update a specific dependency version
    pub fn update_dependency(&mut self, name: &str, new_version: &str, new_checksum: Option<String>) -> Result<()> {
        if let Some(locked_dep) = self.dependencies.get_mut(name) {
            locked_dep.version = new_version.to_string();
            if let Some(checksum) = new_checksum {
                locked_dep.checksum = Some(checksum);
            }
            
            // Update metadata
            self.metadata.generated_at = chrono::Utc::now().to_rfc3339();
            
            Ok(())
        } else {
            Err(BuluError::Other(format!("Dependency {} not found in lock file", name)))
        }
    }

    /// Remove a dependency from the lock file
    pub fn remove_dependency(&mut self, name: &str) -> Result<()> {
        if self.dependencies.remove(name).is_some() {
            // Update metadata
            self.metadata.generated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(BuluError::Other(format!("Dependency {} not found in lock file", name)))
        }
    }
}

/// Lock file manager for handling lock file operations
pub struct LockFileManager {
    lock_file_path: std::path::PathBuf,
}

impl LockFileManager {
    /// Create a new lock file manager
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        Self {
            lock_file_path: project_root.as_ref().join("lang.lock"),
        }
    }

    /// Load existing lock file or create empty one
    pub fn load_or_create(&self) -> Result<LockFile> {
        if self.lock_file_path.exists() {
            LockFile::load(&self.lock_file_path)
        } else {
            Ok(LockFile {
                version: "1".to_string(),
                dependencies: HashMap::new(),
                metadata: LockFileMetadata {
                    generated_at: chrono::Utc::now().to_rfc3339(),
                    generator: format!("bulu-lang/{}", crate::VERSION),
                    root_package: None,
                },
            })
        }
    }

    /// Save lock file
    pub fn save(&self, lock_file: &LockFile) -> Result<()> {
        lock_file.save(&self.lock_file_path)
    }

    /// Check if lock file exists
    pub fn exists(&self) -> bool {
        self.lock_file_path.exists()
    }

    /// Delete lock file
    pub fn delete(&self) -> Result<()> {
        if self.lock_file_path.exists() {
            fs::remove_file(&self.lock_file_path)
                .map_err(|e| BuluError::Other(format!("Failed to delete lock file: {}", e)))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_lock_file_creation() {
        let mut dependencies = HashMap::new();
        
        let resolved_dep = ResolvedDependency {
            name: "test-lib".to_string(),
            version: "1.0.0".to_string(),
            source: DependencySource::Registry {
                url: "https://pkg.lang-lang.org/test-lib/1.0.0".to_string(),
            },
            dependencies: HashMap::new(),
            checksum: Some("abc123".to_string()),
        };
        
        dependencies.insert("test-lib".to_string(), resolved_dep);

        let root_package = RootPackageInfo {
            name: "my-project".to_string(),
            version: "0.1.0".to_string(),
        };

        let lock_file = LockFile::from_resolved_dependencies(&dependencies, Some(root_package));

        assert_eq!(lock_file.version, "1");
        assert_eq!(lock_file.dependencies.len(), 1);
        assert!(lock_file.dependencies.contains_key("test-lib"));
        
        let locked_dep = &lock_file.dependencies["test-lib"];
        assert_eq!(locked_dep.name, "test-lib");
        assert_eq!(locked_dep.version, "1.0.0");
        assert_eq!(locked_dep.checksum, Some("abc123".to_string()));
    }

    #[test]
    fn test_lock_file_serialization() {
        let lock_file = LockFile {
            version: "1".to_string(),
            dependencies: HashMap::new(),
            metadata: LockFileMetadata {
                generated_at: "2023-01-01T00:00:00Z".to_string(),
                generator: "bulu-lang/1.0.0".to_string(),
                root_package: None,
            },
        };

        let serialized = toml::to_string_pretty(&lock_file).unwrap();
        let deserialized: LockFile = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.version, "1");
        assert_eq!(deserialized.metadata.generator, "bulu-lang/1.0.0");
    }

    #[test]
    fn test_resolution_order() {
        let mut dependencies = HashMap::new();
        
        // Create a simple dependency chain: A -> B -> C
        let dep_c = LockedDependency {
            name: "c".to_string(),
            version: "1.0.0".to_string(),
            source: LockedSource::Registry {
                url: "https://example.com/c".to_string(),
                checksum: "c123".to_string(),
            },
            checksum: Some("c123".to_string()),
            dependencies: vec![],
        };
        
        let dep_b = LockedDependency {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            source: LockedSource::Registry {
                url: "https://example.com/b".to_string(),
                checksum: "b123".to_string(),
            },
            checksum: Some("b123".to_string()),
            dependencies: vec!["c".to_string()],
        };
        
        let dep_a = LockedDependency {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            source: LockedSource::Registry {
                url: "https://example.com/a".to_string(),
                checksum: "a123".to_string(),
            },
            checksum: Some("a123".to_string()),
            dependencies: vec!["b".to_string()],
        };

        dependencies.insert("a".to_string(), dep_a);
        dependencies.insert("b".to_string(), dep_b);
        dependencies.insert("c".to_string(), dep_c);

        let lock_file = LockFile {
            version: "1".to_string(),
            dependencies,
            metadata: LockFileMetadata {
                generated_at: "2023-01-01T00:00:00Z".to_string(),
                generator: "bulu-lang/1.0.0".to_string(),
                root_package: None,
            },
        };

        let order = lock_file.get_resolution_order().unwrap();
        
        // C should come before B, B should come before A
        let c_pos = order.iter().position(|x| x == "c").unwrap();
        let b_pos = order.iter().position(|x| x == "b").unwrap();
        let a_pos = order.iter().position(|x| x == "a").unwrap();
        
        assert!(c_pos < b_pos);
        assert!(b_pos < a_pos);
    }
}
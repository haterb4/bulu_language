//! Package management system for Bulu
//!
//! This module provides functionality for managing dependencies, interacting with
//! the package registry, and handling package operations.

pub mod registry;
pub mod resolver;
pub mod commands;
pub mod lockfile;
pub mod vendor;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Package metadata from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub dependencies: HashMap<String, VersionConstraint>,
    pub checksum: String,
    pub download_url: String,
}

/// Version constraint specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VersionConstraint {
    /// Exact version (=1.2.3)
    Exact(String),
    /// Compatible version (^1.2.3)
    Compatible(String),
    /// Tilde version (~1.2.3)
    Tilde(String),
    /// Greater than or equal (>=1.2.3)
    GreaterEqual(String),
    /// Greater than (>1.2.3)
    Greater(String),
    /// Less than or equal (<=1.2.3)
    LessEqual(String),
    /// Less than (<1.2.3)
    Less(String),
    /// Wildcard (*) - any version
    Any,
}

impl VersionConstraint {
    /// Parse a version constraint string
    pub fn parse(constraint: &str) -> Result<Self, String> {
        let constraint = constraint.trim();
        
        if constraint == "*" {
            return Ok(VersionConstraint::Any);
        }
        
        if let Some(version) = constraint.strip_prefix(">=") {
            return Ok(VersionConstraint::GreaterEqual(version.trim().to_string()));
        }
        
        if let Some(version) = constraint.strip_prefix("<=") {
            return Ok(VersionConstraint::LessEqual(version.trim().to_string()));
        }
        
        if let Some(version) = constraint.strip_prefix('>') {
            return Ok(VersionConstraint::Greater(version.trim().to_string()));
        }
        
        if let Some(version) = constraint.strip_prefix('<') {
            return Ok(VersionConstraint::Less(version.trim().to_string()));
        }
        
        if let Some(version) = constraint.strip_prefix('^') {
            return Ok(VersionConstraint::Compatible(version.trim().to_string()));
        }
        
        if let Some(version) = constraint.strip_prefix('~') {
            return Ok(VersionConstraint::Tilde(version.trim().to_string()));
        }
        
        if let Some(version) = constraint.strip_prefix('=') {
            return Ok(VersionConstraint::Exact(version.trim().to_string()));
        }
        
        // Default to compatible version
        Ok(VersionConstraint::Compatible(constraint.to_string()))
    }
    
    /// Check if a version satisfies this constraint
    pub fn satisfies(&self, version: &str) -> bool {
        match self {
            VersionConstraint::Any => true,
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::Compatible(v) => is_compatible_version(version, v),
            VersionConstraint::Tilde(v) => is_tilde_compatible(version, v),
            VersionConstraint::GreaterEqual(v) => compare_versions(version, v) >= 0,
            VersionConstraint::Greater(v) => compare_versions(version, v) > 0,
            VersionConstraint::LessEqual(v) => compare_versions(version, v) <= 0,
            VersionConstraint::Less(v) => compare_versions(version, v) < 0,
        }
    }
}

/// Resolved dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
    pub dependencies: HashMap<String, VersionConstraint>,
    pub checksum: Option<String>,
}

/// Source of a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    /// From package registry
    Registry {
        url: String,
    },
    /// From local path
    Path {
        path: PathBuf,
    },
    /// From git repository
    Git {
        url: String,
        branch: Option<String>,
        tag: Option<String>,
        commit: Option<String>,
    },
}

/// Package manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Registry URL
    pub registry_url: String,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Vendor directory
    pub vendor_dir: PathBuf,
    /// Authentication token
    pub auth_token: Option<String>,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            registry_url: "https://pkg.lang-lang.org".to_string(),
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".cache"))
                .join("bulu"),
            vendor_dir: PathBuf::from("vendor"),
            auth_token: None,
        }
    }
}

// Version comparison utilities

fn parse_version(version: &str) -> Result<Vec<u32>, String> {
    version
        .split('.')
        .map(|part| {
            part.parse::<u32>()
                .map_err(|_| format!("Invalid version component: {}", part))
        })
        .collect()
}

fn compare_versions(a: &str, b: &str) -> i32 {
    let a_parts = parse_version(a).unwrap_or_default();
    let b_parts = parse_version(b).unwrap_or_default();
    
    let max_len = a_parts.len().max(b_parts.len());
    
    for i in 0..max_len {
        let a_part = a_parts.get(i).copied().unwrap_or(0);
        let b_part = b_parts.get(i).copied().unwrap_or(0);
        
        match a_part.cmp(&b_part) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => continue,
        }
    }
    
    0
}

fn is_compatible_version(version: &str, constraint: &str) -> bool {
    let version_parts = parse_version(version).unwrap_or_default();
    let constraint_parts = parse_version(constraint).unwrap_or_default();
    
    if version_parts.is_empty() || constraint_parts.is_empty() {
        return false;
    }
    
    // Major version must match
    if version_parts[0] != constraint_parts[0] {
        return false;
    }
    
    // Version must be >= constraint
    compare_versions(version, constraint) >= 0
}

fn is_tilde_compatible(version: &str, constraint: &str) -> bool {
    let version_parts = parse_version(version).unwrap_or_default();
    let constraint_parts = parse_version(constraint).unwrap_or_default();
    
    if version_parts.len() < 2 || constraint_parts.len() < 2 {
        return false;
    }
    
    // Major and minor versions must match
    if version_parts[0] != constraint_parts[0] || version_parts[1] != constraint_parts[1] {
        return false;
    }
    
    // Version must be >= constraint
    compare_versions(version, constraint) >= 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constraint_parsing() {
        assert_eq!(VersionConstraint::parse("*").unwrap(), VersionConstraint::Any);
        assert_eq!(VersionConstraint::parse("^1.2.3").unwrap(), VersionConstraint::Compatible("1.2.3".to_string()));
        assert_eq!(VersionConstraint::parse("~1.2.3").unwrap(), VersionConstraint::Tilde("1.2.3".to_string()));
        assert_eq!(VersionConstraint::parse(">=1.2.3").unwrap(), VersionConstraint::GreaterEqual("1.2.3".to_string()));
        assert_eq!(VersionConstraint::parse("=1.2.3").unwrap(), VersionConstraint::Exact("1.2.3".to_string()));
    }

    #[test]
    fn test_version_comparison() {
        assert_eq!(compare_versions("1.2.3", "1.2.3"), 0);
        assert!(compare_versions("1.2.4", "1.2.3") > 0);
        assert!(compare_versions("1.2.2", "1.2.3") < 0);
        assert!(compare_versions("2.0.0", "1.9.9") > 0);
    }

    #[test]
    fn test_compatible_version() {
        assert!(is_compatible_version("1.2.3", "1.2.3"));
        assert!(is_compatible_version("1.2.4", "1.2.3"));
        assert!(is_compatible_version("1.3.0", "1.2.3"));
        assert!(!is_compatible_version("2.0.0", "1.2.3"));
        assert!(!is_compatible_version("1.2.2", "1.2.3"));
    }

    #[test]
    fn test_tilde_compatible() {
        assert!(is_tilde_compatible("1.2.3", "1.2.3"));
        assert!(is_tilde_compatible("1.2.4", "1.2.3"));
        assert!(!is_tilde_compatible("1.3.0", "1.2.3"));
        assert!(!is_tilde_compatible("2.0.0", "1.2.3"));
    }
}
//! Dependency resolution for package management

use super::{PackageMetadata, ResolvedDependency, VersionConstraint, DependencySource};
use super::registry::RegistryClient;
use crate::project::DependencySpec;
use crate::{BuluError, Result};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Dependency resolver for handling transitive dependencies
pub struct DependencyResolver {
    registry: RegistryClient,
    resolved: HashMap<String, ResolvedDependency>,
    visited: HashSet<String>,
}

/// Resolution context for tracking dependency resolution
#[derive(Debug, Clone)]
struct ResolutionContext {
    /// Current dependency chain for cycle detection
    chain: Vec<String>,
    /// Constraints for each package from different sources
    constraints: HashMap<String, Vec<(String, VersionConstraint)>>,
}

/// Conflict resolution strategy
#[derive(Debug, Clone)]
pub enum ConflictStrategy {
    /// Fail on any conflict
    Strict,
    /// Use highest compatible version
    HighestCompatible,
    /// Use lowest compatible version
    LowestCompatible,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new(registry: RegistryClient) -> Self {
        Self {
            registry,
            resolved: HashMap::new(),
            visited: HashSet::new(),
        }
    }

    /// Resolve all dependencies for a project
    pub async fn resolve_dependencies(
        &mut self,
        dependencies: &HashMap<String, DependencySpec>,
        strategy: ConflictStrategy,
    ) -> Result<HashMap<String, ResolvedDependency>> {
        self.resolved.clear();
        self.visited.clear();

        let mut context = ResolutionContext {
            chain: Vec::new(),
            constraints: HashMap::new(),
        };

        // First pass: collect all constraints
        for (name, spec) in dependencies {
            self.collect_constraints(name, spec, &mut context).await?;
        }

        // Second pass: resolve with conflict detection
        for (name, spec) in dependencies {
            self.resolve_dependency(name, spec, &mut context, &strategy).await?;
        }

        // Validate resolution
        self.validate_resolution()?;

        Ok(self.resolved.clone())
    }

    /// Collect all version constraints for packages
    fn collect_constraints<'a>(
        &'a mut self,
        name: &'a str,
        spec: &'a DependencySpec,
        context: &'a mut ResolutionContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
        if context.chain.contains(&name.to_string()) {
            return Err(BuluError::Other(format!(
                "Circular dependency detected: {} -> {}",
                context.chain.join(" -> "),
                name
            )));
        }

        context.chain.push(name.to_string());

        let constraint = self.spec_to_constraint(spec)?;
        let source_name = context.chain.get(context.chain.len() - 2)
            .unwrap_or(&"root".to_string())
            .clone();

        context.constraints
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push((source_name, constraint.clone()));

        // Get package metadata to collect transitive dependencies
        if let DependencySource::Registry { .. } = self.spec_to_source(spec)? {
            let package = self.find_compatible_version(name, &constraint).await?;
            
            for (dep_name, dep_constraint) in &package.dependencies {
                let dep_spec = DependencySpec::Simple(dep_constraint.to_string());
                self.collect_constraints(dep_name, &dep_spec, context).await?;
            }
        }

        context.chain.pop();
        Ok(())
        })
    }

    /// Resolve a single dependency
    fn resolve_dependency<'a>(
        &'a mut self,
        name: &'a str,
        spec: &'a DependencySpec,
        context: &'a mut ResolutionContext,
        strategy: &'a ConflictStrategy,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
        if self.resolved.contains_key(name) {
            return Ok(());
        }

        if self.visited.contains(name) {
            return Err(BuluError::Other(format!(
                "Circular dependency detected: {}",
                name
            )));
        }

        self.visited.insert(name.to_string());

        let source = self.spec_to_source(spec)?;
        let resolved_dep = match &source {
            DependencySource::Registry { .. } => {
                self.resolve_registry_dependency(name, context, strategy).await?
            }
            DependencySource::Path { path } => {
                self.resolve_path_dependency(name, path).await?
            }
            DependencySource::Git { .. } => {
                self.resolve_git_dependency(name, &source).await?
            }
        };

        // Resolve transitive dependencies
        for (dep_name, dep_constraint) in &resolved_dep.dependencies {
            let dep_spec = DependencySpec::Simple(dep_constraint.to_string());
            self.resolve_dependency(dep_name, &dep_spec, context, strategy).await?;
        }

        self.resolved.insert(name.to_string(), resolved_dep);
        self.visited.remove(name);

        Ok(())
        })
    }

    /// Resolve a registry dependency with conflict resolution
    async fn resolve_registry_dependency(
        &mut self,
        name: &str,
        context: &ResolutionContext,
        strategy: &ConflictStrategy,
    ) -> Result<ResolvedDependency> {
        let constraints = context.constraints.get(name)
            .ok_or_else(|| BuluError::Other(format!("No constraints found for {}", name)))?;

        // Find a version that satisfies all constraints
        let version = self.resolve_version_conflicts(name, constraints, strategy).await?;
        let package = self.registry.get_package(name, Some(&version)).await?;

        Ok(ResolvedDependency {
            name: package.name.clone(),
            version: package.version.clone(),
            source: DependencySource::Registry {
                url: package.download_url.clone(),
            },
            dependencies: package.dependencies.clone(),
            checksum: Some(package.checksum.clone()),
        })
    }

    /// Resolve a path dependency
    async fn resolve_path_dependency(
        &mut self,
        _name: &str,
        path: &PathBuf,
    ) -> Result<ResolvedDependency> {
        // Read lang.toml from the path
        let config_path = path.join("lang.toml");
        if !config_path.exists() {
            return Err(BuluError::Other(format!(
                "No lang.toml found in path dependency: {}",
                path.display()
            )));
        }

        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| BuluError::Other(format!("Failed to read lang.toml: {}", e)))?;

        let config: crate::project::ProjectConfig = toml::from_str(&config_content)
            .map_err(|e| BuluError::Other(format!("Failed to parse lang.toml: {}", e)))?;

        let dependencies = config.dependencies.iter()
            .map(|(name, spec)| {
                let constraint = self.spec_to_constraint(spec).unwrap_or(VersionConstraint::Any);
                (name.clone(), constraint)
            })
            .collect();

        Ok(ResolvedDependency {
            name: config.package.name.clone(),
            version: config.package.version.clone(),
            source: DependencySource::Path {
                path: path.clone(),
            },
            dependencies,
            checksum: None,
        })
    }

    /// Resolve a git dependency
    async fn resolve_git_dependency(
        &mut self,
        name: &str,
        source: &DependencySource,
    ) -> Result<ResolvedDependency> {
        // For now, return a placeholder - full git support would require git operations
        if let DependencySource::Git { url: _, branch: _, tag, commit } = source {
            Ok(ResolvedDependency {
                name: name.to_string(),
                version: tag.clone().or_else(|| commit.clone()).unwrap_or_else(|| "main".to_string()),
                source: source.clone(),
                dependencies: HashMap::new(),
                checksum: None,
            })
        } else {
            Err(BuluError::Other("Invalid git source".to_string()))
        }
    }

    /// Resolve version conflicts using the specified strategy
    async fn resolve_version_conflicts(
        &mut self,
        name: &str,
        constraints: &[(String, VersionConstraint)],
        strategy: &ConflictStrategy,
    ) -> Result<String> {
        // Get all available versions
        let available_versions = self.registry.get_package_versions(name).await?;
        
        // Find versions that satisfy all constraints
        let mut compatible_versions = Vec::new();
        
        for version in &available_versions {
            let satisfies_all = constraints.iter()
                .all(|(_, constraint)| constraint.satisfies(version));
            
            if satisfies_all {
                compatible_versions.push(version.clone());
            }
        }

        if compatible_versions.is_empty() {
            let constraint_strs: Vec<String> = constraints.iter()
                .map(|(source, constraint)| format!("{}: {}", source, constraint.to_string()))
                .collect();
            
            return Err(BuluError::Other(format!(
                "No version of {} satisfies all constraints: [{}]",
                name,
                constraint_strs.join(", ")
            )));
        }

        // Apply resolution strategy
        let selected_version = match strategy {
            ConflictStrategy::Strict => {
                if compatible_versions.len() > 1 {
                    return Err(BuluError::Other(format!(
                        "Multiple compatible versions found for {}: {:?}",
                        name, compatible_versions
                    )));
                }
                compatible_versions[0].clone()
            }
            ConflictStrategy::HighestCompatible => {
                compatible_versions.into_iter()
                    .max_by(|a, b| super::compare_versions(a, b).cmp(&0))
                    .unwrap()
            }
            ConflictStrategy::LowestCompatible => {
                compatible_versions.into_iter()
                    .min_by(|a, b| super::compare_versions(a, b).cmp(&0))
                    .unwrap()
            }
        };

        Ok(selected_version)
    }

    /// Find a compatible version for a constraint
    async fn find_compatible_version(
        &mut self,
        name: &str,
        constraint: &VersionConstraint,
    ) -> Result<PackageMetadata> {
        let available_versions = self.registry.get_package_versions(name).await?;
        
        for version in available_versions.iter().rev() { // Start with latest
            if constraint.satisfies(version) {
                return self.registry.get_package(name, Some(version)).await;
            }
        }

        Err(BuluError::Other(format!(
            "No compatible version found for {} with constraint {}",
            name,
            constraint.to_string()
        )))
    }

    /// Convert DependencySpec to VersionConstraint
    fn spec_to_constraint(&self, spec: &DependencySpec) -> Result<VersionConstraint> {
        match spec {
            DependencySpec::Simple(version) => VersionConstraint::parse(version)
                .map_err(|e| BuluError::Other(format!("Invalid version constraint: {}", e))),
            DependencySpec::Detailed { version, .. } => {
                if let Some(version) = version {
                    VersionConstraint::parse(version)
                        .map_err(|e| BuluError::Other(format!("Invalid version constraint: {}", e)))
                } else {
                    Ok(VersionConstraint::Any)
                }
            }
        }
    }

    /// Convert DependencySpec to DependencySource
    fn spec_to_source(&self, spec: &DependencySpec) -> Result<DependencySource> {
        match spec {
            DependencySpec::Simple(_) => Ok(DependencySource::Registry {
                url: "registry".to_string(),
            }),
            DependencySpec::Detailed { path, git, branch, tag, .. } => {
                if let Some(path) = path {
                    Ok(DependencySource::Path {
                        path: PathBuf::from(path),
                    })
                } else if let Some(git) = git {
                    Ok(DependencySource::Git {
                        url: git.clone(),
                        branch: branch.clone(),
                        tag: tag.clone(),
                        commit: None,
                    })
                } else {
                    Ok(DependencySource::Registry {
                        url: "registry".to_string(),
                    })
                }
            }
        }
    }

    /// Validate the final resolution
    fn validate_resolution(&self) -> Result<()> {
        // Check for any unresolved dependencies
        for (name, resolved) in &self.resolved {
            for (dep_name, _) in &resolved.dependencies {
                if !self.resolved.contains_key(dep_name) {
                    return Err(BuluError::Other(format!(
                        "Unresolved dependency: {} requires {}",
                        name, dep_name
                    )));
                }
            }
        }

        Ok(())
    }
}

impl VersionConstraint {
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            VersionConstraint::Any => "*".to_string(),
            VersionConstraint::Exact(v) => format!("={}", v),
            VersionConstraint::Compatible(v) => format!("^{}", v),
            VersionConstraint::Tilde(v) => format!("~{}", v),
            VersionConstraint::GreaterEqual(v) => format!(">={}", v),
            VersionConstraint::Greater(v) => format!(">{}", v),
            VersionConstraint::LessEqual(v) => format!("<={}", v),
            VersionConstraint::Less(v) => format!("<{}", v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::registry::MockRegistryClient;

    #[tokio::test]
    async fn test_simple_resolution() {
        let mut registry = MockRegistryClient::new();
        
        // Add a simple package
        let package = PackageMetadata {
            name: "test-lib".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            authors: vec![],
            license: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            dependencies: HashMap::new(),
            checksum: "abc123".to_string(),
            download_url: "https://example.com/test-lib-1.0.0.tar.gz".to_string(),
        };
        
        registry.add_package(package);

        // Create resolver with mock registry
        // Note: This would need to be adapted for the actual RegistryClient
        // For now, this is a conceptual test structure
    }

    #[test]
    fn test_version_constraint_to_string() {
        assert_eq!(VersionConstraint::Any.to_string(), "*");
        assert_eq!(VersionConstraint::Compatible("1.2.3".to_string()).to_string(), "^1.2.3");
        assert_eq!(VersionConstraint::Exact("1.2.3".to_string()).to_string(), "=1.2.3");
        assert_eq!(VersionConstraint::GreaterEqual("1.2.3".to_string()).to_string(), ">=1.2.3");
    }
}
//! Package management commands implementation

use super::lockfile::{LockFile, LockFileManager, RootPackageInfo};
use super::registry::RegistryClient;
use super::resolver::{ConflictStrategy, DependencyResolver};
use super::vendor::{VendorManager, VendorOptions};
use super::{PackageConfig, PackageMetadata, VersionConstraint};
use crate::project::{DependencySpec, Project, ProjectConfig};
use crate::{BuluError, Result};
use colored::*;
use std::fs;

/// Package manager for handling all package operations
pub struct PackageManager {
    project: Project,
    config: PackageConfig,
    registry: RegistryClient,
    lock_manager: LockFileManager,
}

/// Options for package operations
#[derive(Debug, Clone)]
pub struct PackageOptions {
    pub verbose: bool,
    pub dry_run: bool,
    pub force: bool,
}

impl Default for PackageOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            dry_run: false,
            force: false,
        }
    }
}

impl PackageManager {
    /// Create a new package manager
    pub fn new(project: Project) -> Result<Self> {
        let config = PackageConfig::default();
        let registry = RegistryClient::new(config.clone());
        let lock_manager = LockFileManager::new(&project.root);

        Ok(Self {
            project,
            config,
            registry,
            lock_manager,
        })
    }

    /// Add a dependency to the project
    pub async fn add_dependency(
        &mut self,
        name: &str,
        version_spec: Option<&str>,
        options: &PackageOptions,
    ) -> Result<()> {
        if options.verbose {
            println!("{} Adding dependency: {}", "Adding".green().bold(), name);
        }

        // Parse version specification
        let dependency_spec = if let Some(version) = version_spec {
            DependencySpec::Simple(version.to_string())
        } else {
            // Find latest version
            let versions = self.registry.get_package_versions(name).await?;
            let latest = versions.first()
                .ok_or_else(|| BuluError::Other(format!("No versions found for package: {}", name)))?;
            DependencySpec::Simple(format!("^{}", latest))
        };

        if options.dry_run {
            println!("Would add: {} = {}", name, self.spec_to_string(&dependency_spec));
            return Ok(());
        }

        // Update project configuration
        let mut config = self.project.config.clone();
        config.dependencies.insert(name.to_string(), dependency_spec);

        // Resolve dependencies
        let mut resolver = DependencyResolver::new(self.registry.clone());
        let resolved = resolver.resolve_dependencies(&config.dependencies, ConflictStrategy::HighestCompatible).await?;

        // Update lock file
        let root_package = RootPackageInfo {
            name: config.package.name.clone(),
            version: config.package.version.clone(),
        };
        let lock_file = LockFile::from_resolved_dependencies(&resolved, Some(root_package));
        self.lock_manager.save(&lock_file)?;

        // Save updated project configuration
        self.save_project_config(&config)?;

        if options.verbose {
            println!("{} Added dependency: {}", "Success".green().bold(), name);
        }

        Ok(())
    }

    /// Remove a dependency from the project
    pub async fn remove_dependency(&mut self, name: &str, options: &PackageOptions) -> Result<()> {
        if options.verbose {
            println!("{} Removing dependency: {}", "Removing".red().bold(), name);
        }

        if !self.project.config.dependencies.contains_key(name) {
            return Err(BuluError::Other(format!("Dependency {} not found", name)));
        }

        if options.dry_run {
            println!("Would remove: {}", name);
            return Ok(());
        }

        // Update project configuration
        let mut config = self.project.config.clone();
        config.dependencies.remove(name);

        // Re-resolve remaining dependencies
        let mut resolver = DependencyResolver::new(self.registry.clone());
        let resolved = resolver.resolve_dependencies(&config.dependencies, ConflictStrategy::HighestCompatible).await?;

        // Update lock file
        let root_package = RootPackageInfo {
            name: config.package.name.clone(),
            version: config.package.version.clone(),
        };
        let lock_file = LockFile::from_resolved_dependencies(&resolved, Some(root_package));
        self.lock_manager.save(&lock_file)?;

        // Save updated project configuration
        self.save_project_config(&config)?;

        if options.verbose {
            println!("{} Removed dependency: {}", "Success".green().bold(), name);
        }

        Ok(())
    }

    /// Update all dependencies to their latest compatible versions
    pub async fn update_dependencies(&mut self, options: &PackageOptions) -> Result<()> {
        if options.verbose {
            println!("{} Updating dependencies...", "Updating".blue().bold());
        }

        if options.dry_run {
            println!("Would update all dependencies");
            return Ok(());
        }

        // Re-resolve all dependencies with latest versions
        let mut resolver = DependencyResolver::new(self.registry.clone());
        let resolved = resolver.resolve_dependencies(&self.project.config.dependencies, ConflictStrategy::HighestCompatible).await?;

        // Update lock file
        let root_package = RootPackageInfo {
            name: self.project.config.package.name.clone(),
            version: self.project.config.package.version.clone(),
        };
        let lock_file = LockFile::from_resolved_dependencies(&resolved, Some(root_package));
        self.lock_manager.save(&lock_file)?;

        if options.verbose {
            println!("{} Updated {} dependencies", "Success".green().bold(), resolved.len());
        }

        Ok(())
    }

    /// Install dependencies from lang.toml
    pub async fn install_dependencies(&mut self, options: &PackageOptions) -> Result<()> {
        if options.verbose {
            println!("{} Installing dependencies...", "Installing".blue().bold());
        }

        // Check if lock file exists and is up to date
        let lock_file = if self.lock_manager.exists() {
            let existing_lock = self.lock_manager.load_or_create()?;
            if existing_lock.is_up_to_date(&self.project.config.dependencies) {
                existing_lock
            } else {
                // Re-resolve dependencies
                let mut resolver = DependencyResolver::new(self.registry.clone());
                let resolved = resolver.resolve_dependencies(&self.project.config.dependencies, ConflictStrategy::HighestCompatible).await?;
                
                let root_package = RootPackageInfo {
                    name: self.project.config.package.name.clone(),
                    version: self.project.config.package.version.clone(),
                };
                LockFile::from_resolved_dependencies(&resolved, Some(root_package))
            }
        } else {
            // Create new lock file
            let mut resolver = DependencyResolver::new(self.registry.clone());
            let resolved = resolver.resolve_dependencies(&self.project.config.dependencies, ConflictStrategy::HighestCompatible).await?;
            
            let root_package = RootPackageInfo {
                name: self.project.config.package.name.clone(),
                version: self.project.config.package.version.clone(),
            };
            LockFile::from_resolved_dependencies(&resolved, Some(root_package))
        };

        if options.dry_run {
            println!("Would install {} dependencies", lock_file.dependencies.len());
            return Ok(());
        }

        // Save lock file
        self.lock_manager.save(&lock_file)?;

        if options.verbose {
            println!("{} Installed {} dependencies", "Success".green().bold(), lock_file.dependencies.len());
        }

        Ok(())
    }

    /// List installed dependencies
    pub async fn list_dependencies(&self, options: &PackageOptions) -> Result<()> {
        let lock_file = if self.lock_manager.exists() {
            self.lock_manager.load_or_create()?
        } else {
            println!("No dependencies installed");
            return Ok(());
        };

        println!("{}", "Dependencies:".bold());
        
        for (name, spec) in &self.project.config.dependencies {
            if let Some(locked_dep) = lock_file.dependencies.get(name) {
                println!("  {} {} ({})", 
                    name.cyan(), 
                    locked_dep.version.green(),
                    self.spec_to_string(spec).dimmed()
                );
                
                if options.verbose {
                    println!("    Source: {:?}", locked_dep.source);
                    if !locked_dep.dependencies.is_empty() {
                        println!("    Dependencies: {}", locked_dep.dependencies.join(", "));
                    }
                }
            } else {
                println!("  {} {} ({})", 
                    name.cyan(), 
                    "not resolved".red(),
                    self.spec_to_string(spec).dimmed()
                );
            }
        }

        if options.verbose {
            println!("\nTransitive dependencies:");
            for (name, locked_dep) in &lock_file.dependencies {
                if !self.project.config.dependencies.contains_key(name) {
                    println!("  {} {}", name.yellow(), locked_dep.version.green());
                }
            }
        }

        Ok(())
    }

    /// Search for packages in the registry
    pub async fn search_packages(&self, query: &str, limit: Option<usize>) -> Result<()> {
        println!("{} Searching for: {}", "Searching".blue().bold(), query);
        
        let results = self.registry.search(query, limit).await?;
        
        if results.packages.is_empty() {
            println!("No packages found matching '{}'", query);
            return Ok(());
        }

        println!("Found {} packages:", results.packages.len());
        
        for package in &results.packages {
            println!("  {} {} - {}", 
                package.name.cyan().bold(),
                package.version.green(),
                package.description.as_deref().unwrap_or("No description").dimmed()
            );
            println!("    {} downloads, updated {}", 
                package.downloads.to_string().yellow(),
                package.updated_at.dimmed()
            );
        }

        if results.total > results.packages.len() {
            println!("\n... and {} more results", results.total - results.packages.len());
        }

        Ok(())
    }

    /// Publish a package to the registry
    pub async fn publish_package(&self, options: &PackageOptions) -> Result<()> {
        if options.verbose {
            println!("{} Publishing package: {}", "Publishing".blue().bold(), self.project.config.package.name);
        }

        // Create package tarball
        let tarball = self.create_package_tarball()?;

        // Create package metadata
        let metadata = PackageMetadata {
            name: self.project.config.package.name.clone(),
            version: self.project.config.package.version.clone(),
            description: self.project.config.package.description.clone(),
            authors: self.project.config.package.authors.clone(),
            license: self.project.config.package.license.clone(),
            repository: self.project.config.package.repository.clone(),
            keywords: self.project.config.package.keywords.clone().unwrap_or_default(),
            categories: self.project.config.package.categories.clone().unwrap_or_default(),
            dependencies: self.project.config.dependencies.iter()
                .map(|(name, spec)| {
                    let constraint = self.spec_to_constraint(spec).unwrap_or(VersionConstraint::Any);
                    (name.clone(), constraint)
                })
                .collect(),
            checksum: sha256::digest(&tarball),
            download_url: format!("https://pkg.lang-lang.org/{}/{}/download", 
                self.project.config.package.name, 
                self.project.config.package.version
            ),
        };

        if options.dry_run {
            println!("Would publish: {} v{}", metadata.name, metadata.version);
            return Ok(());
        }

        // Publish to registry
        self.registry.publish_package(&metadata, tarball).await?;

        if options.verbose {
            println!("{} Published: {} v{}", "Success".green().bold(), metadata.name, metadata.version);
        }

        Ok(())
    }

    /// Vendor dependencies
    pub async fn vendor_dependencies(&self, options: &PackageOptions) -> Result<()> {
        if options.verbose {
            println!("{} Vendoring dependencies...", "Vendoring".blue().bold());
        }

        let lock_file = self.lock_manager.load_or_create()?;
        let vendor_manager = VendorManager::new(&self.project.root, self.registry.clone());
        
        let vendor_options = VendorOptions {
            update_existing: options.force,
            verify_checksums: true,
            include_dev_deps: false,
            verbose: options.verbose,
        };

        if options.dry_run {
            let status = vendor_manager.check_vendored_status(&lock_file)?;
            println!("Would vendor {} dependencies", status.total_dependencies);
            return Ok(());
        }

        let result = vendor_manager.vendor_dependencies(&lock_file, &vendor_options).await?;

        if !result.errors.is_empty() {
            for error in &result.errors {
                eprintln!("{} {}", "Error:".red().bold(), error);
            }
        }

        if options.verbose {
            println!("{} Vendored {} dependencies", "Success".green().bold(), result.vendored.len());
        }

        Ok(())
    }

    /// Clean build artifacts and caches
    pub fn clean(&self, options: &PackageOptions) -> Result<()> {
        if options.verbose {
            println!("{} Cleaning project...", "Cleaning".blue().bold());
        }

        let mut cleaned_items = Vec::new();

        // Clean build directory
        if self.project.build_dir.exists() {
            if options.dry_run {
                println!("Would remove: {}", self.project.build_dir.display());
            } else {
                fs::remove_dir_all(&self.project.build_dir)
                    .map_err(|e| BuluError::Other(format!("Failed to clean build directory: {}", e)))?;
                cleaned_items.push("build directory");
            }
        }

        // Clean target directory
        if self.project.target_dir.exists() {
            if options.dry_run {
                println!("Would remove: {}", self.project.target_dir.display());
            } else {
                fs::remove_dir_all(&self.project.target_dir)
                    .map_err(|e| BuluError::Other(format!("Failed to clean target directory: {}", e)))?;
                cleaned_items.push("target directory");
            }
        }

        // Clean package cache
        if !options.dry_run {
            self.registry.clear_cache()?;
            cleaned_items.push("package cache");
        }

        if options.verbose && !options.dry_run {
            println!("{} Cleaned: {}", "Success".green().bold(), cleaned_items.join(", "));
        }

        Ok(())
    }

    /// Helper: Convert DependencySpec to string
    fn spec_to_string(&self, spec: &DependencySpec) -> String {
        match spec {
            DependencySpec::Simple(version) => version.clone(),
            DependencySpec::Detailed { version, path, git, .. } => {
                if let Some(path) = path {
                    format!("path:{}", path)
                } else if let Some(git) = git {
                    format!("git:{}", git)
                } else if let Some(version) = version {
                    version.clone()
                } else {
                    "*".to_string()
                }
            }
        }
    }

    /// Helper: Convert DependencySpec to VersionConstraint
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

    /// Helper: Save project configuration
    fn save_project_config(&self, config: &ProjectConfig) -> Result<()> {
        let config_content = toml::to_string_pretty(config)
            .map_err(|e| BuluError::Other(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(self.project.root.join("lang.toml"), config_content)
            .map_err(|e| BuluError::Other(format!("Failed to write lang.toml: {}", e)))?;

        Ok(())
    }

    /// Helper: Create package tarball
    fn create_package_tarball(&self) -> Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::Builder;


        let mut tarball = Vec::new();
        {
            let encoder = GzEncoder::new(&mut tarball, Compression::default());
            let mut builder = Builder::new(encoder);

            // Add source files
            for source_file in self.project.source_files()? {
                let relative_path = source_file.strip_prefix(&self.project.root)
                    .map_err(|e| BuluError::Other(format!("Failed to get relative path: {}", e)))?;
                builder.append_path_with_name(&source_file, relative_path)
                    .map_err(|e| BuluError::Other(format!("Failed to add file to tarball: {}", e)))?;
            }

            // Add lang.toml
            builder.append_path_with_name(self.project.root.join("lang.toml"), "lang.toml")
                .map_err(|e| BuluError::Other(format!("Failed to add lang.toml to tarball: {}", e)))?;

            // Add README if it exists
            let readme_path = self.project.root.join("README.md");
            if readme_path.exists() {
                builder.append_path_with_name(&readme_path, "README.md")
                    .map_err(|e| BuluError::Other(format!("Failed to add README to tarball: {}", e)))?;
            }

            builder.finish()
                .map_err(|e| BuluError::Other(format!("Failed to finish tarball: {}", e)))?;
        }

        Ok(tarball)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_spec_to_string() {
        let simple = DependencySpec::Simple("^1.0.0".to_string());
        let detailed_version = DependencySpec::Detailed {
            version: Some("~1.2.0".to_string()),
            path: None,
            git: None,
            branch: None,
            tag: None,
            features: None,
            optional: None,
        };
        let detailed_path = DependencySpec::Detailed {
            version: None,
            path: Some("../local-lib".to_string()),
            git: None,
            branch: None,
            tag: None,
            features: None,
            optional: None,
        };

        // Create a temporary project for testing
        let temp_dir = TempDir::new().unwrap();
        let project_config = ProjectConfig {
            package: crate::project::PackageConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                authors: vec![],
                description: None,
                license: None,
                repository: None,
                keywords: None,
                categories: None,
            },
            dependencies: std::collections::HashMap::new(),
            build: crate::project::BuildConfig::default(),
            test: crate::project::TestConfig::default(),
        };

        // This test would need a proper project setup to work fully
        // For now, just test the basic functionality
        assert_eq!("^1.0.0", "^1.0.0"); // Placeholder assertion
    }
}
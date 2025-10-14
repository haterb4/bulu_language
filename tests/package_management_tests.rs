//! Integration tests for package management system

use bulu::package::commands::PackageOptions;
use bulu::package::lockfile::{LockFile, LockFileManager, RootPackageInfo};
use bulu::package::{PackageMetadata, VersionConstraint, DependencySource, ResolvedDependency};
use bulu::project::{create_project, Project, DependencySpec};
use std::collections::HashMap;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_version_constraint_parsing() {
    // Test various version constraint formats
    assert_eq!(VersionConstraint::parse("*").unwrap(), VersionConstraint::Any);
    assert_eq!(VersionConstraint::parse("^1.2.3").unwrap(), VersionConstraint::Compatible("1.2.3".to_string()));
    assert_eq!(VersionConstraint::parse("~1.2.3").unwrap(), VersionConstraint::Tilde("1.2.3".to_string()));
    assert_eq!(VersionConstraint::parse(">=1.2.3").unwrap(), VersionConstraint::GreaterEqual("1.2.3".to_string()));
    assert_eq!(VersionConstraint::parse("=1.2.3").unwrap(), VersionConstraint::Exact("1.2.3".to_string()));
    
    // Test default behavior (should be compatible)
    assert_eq!(VersionConstraint::parse("1.2.3").unwrap(), VersionConstraint::Compatible("1.2.3".to_string()));
}

#[tokio::test]
async fn test_version_constraint_satisfaction() {
    let compatible = VersionConstraint::Compatible("1.2.0".to_string());
    assert!(compatible.satisfies("1.2.0"));
    assert!(compatible.satisfies("1.2.1"));
    assert!(compatible.satisfies("1.3.0"));
    assert!(!compatible.satisfies("2.0.0"));
    assert!(!compatible.satisfies("1.1.9"));

    let tilde = VersionConstraint::Tilde("1.2.0".to_string());
    assert!(tilde.satisfies("1.2.0"));
    assert!(tilde.satisfies("1.2.1"));
    assert!(!tilde.satisfies("1.3.0"));
    assert!(!tilde.satisfies("2.0.0"));

    let exact = VersionConstraint::Exact("1.2.3".to_string());
    assert!(exact.satisfies("1.2.3"));
    assert!(!exact.satisfies("1.2.4"));
    assert!(!exact.satisfies("1.2.2"));
}

#[tokio::test]
async fn test_lock_file_creation_and_serialization() {
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

    // Test basic properties
    assert_eq!(lock_file.version, "1");
    assert_eq!(lock_file.dependencies.len(), 1);
    assert!(lock_file.dependencies.contains_key("test-lib"));
    
    let locked_dep = &lock_file.dependencies["test-lib"];
    assert_eq!(locked_dep.name, "test-lib");
    assert_eq!(locked_dep.version, "1.0.0");
    assert_eq!(locked_dep.checksum, Some("abc123".to_string()));

    // Test serialization/deserialization
    let serialized = toml::to_string_pretty(&lock_file).unwrap();
    let deserialized: LockFile = toml::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.version, lock_file.version);
    assert_eq!(deserialized.dependencies.len(), lock_file.dependencies.len());
}

#[tokio::test]
async fn test_lock_file_resolution_order() {
    let mut dependencies = HashMap::new();
    
    // Create a dependency chain: A -> B -> C
    let dep_c = bulu::package::lockfile::LockedDependency {
        name: "c".to_string(),
        version: "1.0.0".to_string(),
        source: bulu::package::lockfile::LockedSource::Registry {
            url: "https://example.com/c".to_string(),
            checksum: "c123".to_string(),
        },
        checksum: Some("c123".to_string()),
        dependencies: vec![],
    };
    
    let dep_b = bulu::package::lockfile::LockedDependency {
        name: "b".to_string(),
        version: "1.0.0".to_string(),
        source: bulu::package::lockfile::LockedSource::Registry {
            url: "https://example.com/b".to_string(),
            checksum: "b123".to_string(),
        },
        checksum: Some("b123".to_string()),
        dependencies: vec!["c".to_string()],
    };
    
    let dep_a = bulu::package::lockfile::LockedDependency {
        name: "a".to_string(),
        version: "1.0.0".to_string(),
        source: bulu::package::lockfile::LockedSource::Registry {
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
        metadata: bulu::package::lockfile::LockFileMetadata {
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

#[tokio::test]
async fn test_lock_file_validation() {
    let mut dependencies = HashMap::new();
    
    // Create a valid dependency
    let dep_a = bulu::package::lockfile::LockedDependency {
        name: "a".to_string(),
        version: "1.0.0".to_string(),
        source: bulu::package::lockfile::LockedSource::Registry {
            url: "https://example.com/a".to_string(),
            checksum: "a123".to_string(),
        },
        checksum: Some("a123".to_string()),
        dependencies: vec!["b".to_string()], // References missing dependency
    };

    dependencies.insert("a".to_string(), dep_a);

    let lock_file = LockFile {
        version: "1".to_string(),
        dependencies,
        metadata: bulu::package::lockfile::LockFileMetadata {
            generated_at: "2023-01-01T00:00:00Z".to_string(),
            generator: "bulu-lang/1.0.0".to_string(),
            root_package: None,
        },
    };

    // Validation should fail due to missing dependency
    assert!(lock_file.validate().is_err());
}

#[tokio::test]
async fn test_mock_registry_operations() {
    // This test would require the MockRegistryClient which is only available in the registry module's tests
    // For now, we'll test the basic functionality that's available
    
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

    // Test package metadata creation
    assert_eq!(package.name, "test-package");
    assert_eq!(package.version, "1.0.0");
    assert_eq!(package.description, Some("Test package".to_string()));
}

#[tokio::test]
async fn test_project_integration() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-project");
    
    // Create a test project
    create_project("test-project", Some(temp_dir.path())).unwrap();
    
    // Load the project
    let project = Project::load_from_path(&project_path).unwrap();
    
    // Verify project structure
    assert_eq!(project.config.package.name, "test-project");
    assert_eq!(project.config.package.version, "0.1.0");
    assert!(project.src_dir.exists());
    assert!(project.main_source_file().exists());
    
    // Test lock file manager
    let lock_manager = LockFileManager::new(&project.root);
    assert!(!lock_manager.exists());
    
    let empty_lock = lock_manager.load_or_create().unwrap();
    assert_eq!(empty_lock.dependencies.len(), 0);
}

#[tokio::test]
async fn test_vendor_status_calculation() {
    use bulu::package::vendor::VendorStatus;
    
    let status = VendorStatus {
        total_dependencies: 10,
        vendored_dependencies: 8,
        missing_dependencies: vec!["dep1".to_string()],
        outdated_dependencies: vec!["dep2".to_string()],
    };

    assert!(!status.is_complete());
    assert_eq!(status.completion_percentage(), 80.0);
    
    let complete_status = VendorStatus {
        total_dependencies: 5,
        vendored_dependencies: 5,
        missing_dependencies: vec![],
        outdated_dependencies: vec![],
    };
    
    assert!(complete_status.is_complete());
    assert_eq!(complete_status.completion_percentage(), 100.0);
}

#[tokio::test]
async fn test_dependency_spec_parsing() {
    use bulu::project::DependencySpec;
    
    // Test simple dependency spec
    let simple = DependencySpec::Simple("^1.0.0".to_string());
    match simple {
        DependencySpec::Simple(version) => assert_eq!(version, "^1.0.0"),
        _ => panic!("Expected simple dependency spec"),
    }
    
    // Test detailed dependency spec
    let detailed = DependencySpec::Detailed {
        version: Some("~1.2.0".to_string()),
        path: None,
        git: Some("https://github.com/example/repo.git".to_string()),
        branch: Some("main".to_string()),
        tag: None,
        features: Some(vec!["feature1".to_string(), "feature2".to_string()]),
        optional: Some(true),
    };
    
    match detailed {
        DependencySpec::Detailed { version, git, branch, features, optional, .. } => {
            assert_eq!(version, Some("~1.2.0".to_string()));
            assert_eq!(git, Some("https://github.com/example/repo.git".to_string()));
            assert_eq!(branch, Some("main".to_string()));
            assert_eq!(features, Some(vec!["feature1".to_string(), "feature2".to_string()]));
            assert_eq!(optional, Some(true));
        }
        _ => panic!("Expected detailed dependency spec"),
    }
}

#[tokio::test]
async fn test_package_options() {
    let default_options = PackageOptions::default();
    assert!(!default_options.verbose);
    assert!(!default_options.dry_run);
    assert!(!default_options.force);
    
    let custom_options = PackageOptions {
        verbose: true,
        dry_run: true,
        force: true,
    };
    assert!(custom_options.verbose);
    assert!(custom_options.dry_run);
    assert!(custom_options.force);
}

// Integration test for the complete package management workflow
#[tokio::test]
async fn test_complete_package_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("workflow-test");
    
    // 1. Create a new project
    create_project("workflow-test", Some(temp_dir.path())).unwrap();
    let project = Project::load_from_path(&project_path).unwrap();
    
    // 2. Test lock file operations
    let lock_manager = LockFileManager::new(&project.root);
    
    // Create a mock resolved dependency
    let mut resolved_deps = HashMap::new();
    let resolved_dep = ResolvedDependency {
        name: "example-lib".to_string(),
        version: "1.0.0".to_string(),
        source: DependencySource::Registry {
            url: "https://pkg.lang-lang.org/example-lib/1.0.0".to_string(),
        },
        dependencies: HashMap::new(),
        checksum: Some("def456".to_string()),
    };
    resolved_deps.insert("example-lib".to_string(), resolved_dep);
    
    let root_package = RootPackageInfo {
        name: project.config.package.name.clone(),
        version: project.config.package.version.clone(),
    };
    
    let lock_file = LockFile::from_resolved_dependencies(&resolved_deps, Some(root_package));
    
    // 3. Save and load lock file
    lock_manager.save(&lock_file).unwrap();
    assert!(lock_manager.exists());
    
    let loaded_lock = lock_manager.load_or_create().unwrap();
    assert_eq!(loaded_lock.dependencies.len(), 1);
    assert!(loaded_lock.dependencies.contains_key("example-lib"));
    
    // 4. Validate lock file
    loaded_lock.validate().unwrap();
    
    // 5. Test resolution order
    let order = loaded_lock.get_resolution_order().unwrap();
    assert_eq!(order.len(), 1);
    assert_eq!(order[0], "example-lib");
    
    // 6. Test up-to-date check
    let mut project_deps = HashMap::new();
    project_deps.insert("example-lib".to_string(), DependencySpec::Simple("^1.0.0".to_string()));
    
    assert!(loaded_lock.is_up_to_date(&project_deps));
    
    // 7. Test with missing dependency
    project_deps.insert("missing-lib".to_string(), DependencySpec::Simple("^2.0.0".to_string()));
    assert!(!loaded_lock.is_up_to_date(&project_deps));
}
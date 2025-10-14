//! Project configuration and management for Bulu projects

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::{BuluError, Result};

/// Project configuration loaded from lang.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub package: PackageConfig,
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub test: TestConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    Simple(String),
    Detailed {
        version: Option<String>,
        path: Option<String>,
        git: Option<String>,
        branch: Option<String>,
        tag: Option<String>,
        features: Option<Vec<String>>,
        optional: Option<bool>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_optimization")]
    pub optimization: String,
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub incremental: bool,
    #[serde(default)]
    pub parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    #[serde(default)]
    pub parallel: bool,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub coverage: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            optimization: default_optimization(),
            target: default_target(),
            features: Vec::new(),
            incremental: true,
            parallel: true,
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            timeout: Some(30),
            coverage: false,
        }
    }
}

fn default_optimization() -> String {
    "2".to_string()
}

fn default_target() -> String {
    "native".to_string()
}

/// Represents a Bulu project
#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub config: ProjectConfig,
    pub src_dir: PathBuf,
    pub build_dir: PathBuf,
    pub target_dir: PathBuf,
}

impl Project {
    /// Load a project from the current directory
    pub fn load_current() -> Result<Self> {
        Self::load_from_path(".")
    }

    /// Load a project from the specified path
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let root = path.as_ref().canonicalize()
            .map_err(|e| BuluError::Other(format!("Failed to resolve project path: {}", e)))?;
        
        let config_path = root.join("lang.toml");
        if !config_path.exists() {
            return Err(BuluError::Other(
                "No lang.toml found. This doesn't appear to be a Bulu project.".to_string()
            ));
        }

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| BuluError::Other(format!("Failed to read lang.toml: {}", e)))?;
        
        let config: ProjectConfig = toml::from_str(&config_content)
            .map_err(|e| BuluError::Other(format!("Failed to parse lang.toml: {}", e)))?;

        let src_dir = root.join("src");
        let build_dir = root.join("build");
        let target_dir = root.join("target");

        Ok(Self {
            root,
            config,
            src_dir,
            build_dir,
            target_dir,
        })
    }

    /// Get the main source file path
    pub fn main_source_file(&self) -> PathBuf {
        self.src_dir.join("main.bu")
    }

    /// Get all source files in the project
    pub fn source_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.collect_source_files(&self.src_dir, &mut files)?;
        Ok(files)
    }

    fn collect_source_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)
            .map_err(|e| BuluError::Other(format!("Failed to read directory {}: {}", dir.display(), e)))?
        {
            let entry = entry
                .map_err(|e| BuluError::Other(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.is_dir() {
                self.collect_source_files(&path, files)?;
            } else if let Some(ext) = path.extension() {
                if ext == "bu" {
                    files.push(path);
                }
            }
        }

        Ok(())
    }

    /// Get test files in the project
    pub fn test_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        // Look for test files in src/ (files ending with _test.bu)
        for file in self.source_files()? {
            if let Some(stem) = file.file_stem() {
                if stem.to_string_lossy().ends_with("_test") {
                    files.push(file);
                }
            }
        }

        // Look for test files in tests/ directory
        let tests_dir = self.root.join("tests");
        if tests_dir.exists() {
            self.collect_source_files(&tests_dir, &mut files)?;
        }

        Ok(files)
    }

    /// Create necessary directories for the project
    pub fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.build_dir)
            .map_err(|e| BuluError::Other(format!("Failed to create build directory: {}", e)))?;
        fs::create_dir_all(&self.target_dir)
            .map_err(|e| BuluError::Other(format!("Failed to create target directory: {}", e)))?;
        Ok(())
    }

    /// Get the output executable path
    pub fn executable_path(&self, release: bool) -> PathBuf {
        let profile = if release { "release" } else { "debug" };
        self.target_dir.join(profile).join(&self.config.package.name)
    }

    /// Check if the project needs rebuilding
    pub fn needs_rebuild(&self, release: bool) -> Result<bool> {
        let exe_path = self.executable_path(release);
        
        if !exe_path.exists() {
            return Ok(true);
        }

        let exe_modified = fs::metadata(&exe_path)
            .map_err(|e| BuluError::Other(format!("Failed to get executable metadata: {}", e)))?
            .modified()
            .map_err(|e| BuluError::Other(format!("Failed to get executable modification time: {}", e)))?;

        // Check if any source file is newer than the executable
        for source_file in self.source_files()? {
            let source_modified = fs::metadata(&source_file)
                .map_err(|e| BuluError::Other(format!("Failed to get source file metadata: {}", e)))?
                .modified()
                .map_err(|e| BuluError::Other(format!("Failed to get source modification time: {}", e)))?;

            if source_modified > exe_modified {
                return Ok(true);
            }
        }

        // Check if lang.toml is newer than the executable
        let config_modified = fs::metadata(self.root.join("lang.toml"))
            .map_err(|e| BuluError::Other(format!("Failed to get config metadata: {}", e)))?
            .modified()
            .map_err(|e| BuluError::Other(format!("Failed to get config modification time: {}", e)))?;

        if config_modified > exe_modified {
            return Ok(true);
        }

        Ok(false)
    }
}

/// Create a new Bulu project
pub fn create_project(name: &str, path: Option<&Path>) -> Result<()> {
    let project_path = if let Some(path) = path {
        path.join(name)
    } else {
        PathBuf::from(name)
    };

    if project_path.exists() {
        return Err(BuluError::Other(format!("Directory '{}' already exists", project_path.display())));
    }

    // Create project directory structure
    fs::create_dir_all(&project_path)
        .map_err(|e| BuluError::Other(format!("Failed to create project directory: {}", e)))?;
    
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| BuluError::Other(format!("Failed to create src directory: {}", e)))?;

    // Create lang.toml
    let config = ProjectConfig {
        package: PackageConfig {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["Your Name <your.email@example.com>".to_string()],
            description: Some(format!("A Bulu project named {}", name)),
            license: None,
            repository: None,
            keywords: None,
            categories: None,
        },
        dependencies: HashMap::new(),
        build: BuildConfig::default(),
        test: TestConfig::default(),
    };

    let config_content = toml::to_string_pretty(&config)
        .map_err(|e| BuluError::Other(format!("Failed to serialize config: {}", e)))?;
    
    fs::write(project_path.join("lang.toml"), config_content)
        .map_err(|e| BuluError::Other(format!("Failed to write lang.toml: {}", e)))?;

    // Create main.bu
    let main_content = r#"// Main entry point for the Bulu program

func main() {
    println("Hello, Bulu!")
}
"#;
    
    fs::write(src_dir.join("main.bu"), main_content)
        .map_err(|e| BuluError::Other(format!("Failed to write main.bu: {}", e)))?;

    // Create .gitignore
    let gitignore_content = r#"# Build artifacts
/target/
/build/

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db
"#;
    
    fs::write(project_path.join(".gitignore"), gitignore_content)
        .map_err(|e| BuluError::Other(format!("Failed to write .gitignore: {}", e)))?;

    // Create README.md
    let readme_content = format!(r#"# {}

{}

## Getting Started

To run this project:

```bash
lang run
```

To build this project:

```bash
lang build
```

To run tests:

```bash
lang test
```

## Project Structure

- `src/` - Source code
- `tests/` - Test files
- `lang.toml` - Project configuration
"#, name, config.package.description.as_deref().unwrap_or("A Bulu project"));

    fs::write(project_path.join("README.md"), readme_content)
        .map_err(|e| BuluError::Other(format!("Failed to write README.md: {}", e)))?;

    Ok(())
}
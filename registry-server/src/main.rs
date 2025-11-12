mod database;
mod storage;
mod error;
mod cloudflare_storage;
mod entities;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber;
use sha2::Digest;

use database::Database;
use storage::StorageBackend;
use error::RegistryError;

#[derive(Clone)]
struct AppState {
    db: Database,
    storage: Arc<dyn StorageBackend + Send + Sync>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PublishRequest {
    name: String,
    version: String,
    description: Option<String>,
    license: Option<String>,
    repository: Option<String>,
    authors: Vec<String>,
    keywords: Vec<String>,
    dependencies: std::collections::HashMap<String, String>,
    tarball: Vec<u8>,
}

#[derive(Debug, Serialize)]
struct PackageInfo {
    name: String,
    description: Option<String>,
    repository: Option<String>,
    versions: Vec<VersionInfo>,
    keywords: Vec<String>,
    total_downloads: i64,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    packages: Vec<PackageInfo>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct VersionInfo {
    version: String,
    description: Option<String>,
    license: Option<String>,
    authors: Vec<String>,
    dependencies: std::collections::HashMap<String, String>,
    published_at: chrono::DateTime<chrono::FixedOffset>,
    downloads: i64,
    checksum: String,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    20
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://registry.db".to_string());
    
    info!("📊 Connecting to database...");
    let db = match Database::new(&database_url).await {
        Ok(db) => {
            info!("✅ Database connected successfully");
            db
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {}", e);
            return Err(e.into());
        }
    };

    // Initialize storage
    let storage: Arc<dyn StorageBackend + Send + Sync> = if let Ok(account_id) = std::env::var("CLOUDFLARE_ACCOUNT_ID") {
        // Use Cloudflare R2 storage
        let bucket_name = std::env::var("CLOUDFLARE_BUCKET_NAME")
            .expect("CLOUDFLARE_BUCKET_NAME must be set when using Cloudflare storage");
        let access_key_id = std::env::var("CLOUDFLARE_ACCESS_KEY_ID")
            .expect("CLOUDFLARE_ACCESS_KEY_ID must be set when using Cloudflare storage");
        let secret_access_key = std::env::var("CLOUDFLARE_SECRET_ACCESS_KEY")
            .expect("CLOUDFLARE_SECRET_ACCESS_KEY must be set when using Cloudflare storage");
        
        info!("☁️  Using Cloudflare R2 storage with bucket: {}", bucket_name);
        Arc::new(cloudflare_storage::CloudflareStorage::new(
            account_id,
            bucket_name,
            access_key_id,
            secret_access_key,
        ))
    } else {
        // Use local storage as fallback
        let storage_path = std::env::var("STORAGE_PATH")
            .unwrap_or_else(|_| "./storage".to_string());
        info!("💾 Using local storage at: {}", storage_path);
        Arc::new(storage::LocalStorage::new(std::path::PathBuf::from(storage_path)))
    };

    // Create application state
    let state = Arc::new(AppState { db, storage });

    // Build our application with routes
    let app = Router::new()
        .route("/api/packages", get(list_packages))
        .route("/api/packages/:name", get(get_package_info))
        .route("/api/packages/:name/:version", post(publish_package))
        .route("/api/packages/:name/:version", delete(delete_package))
        .route("/api/download/:name/:version", get(download_package))
        .route("/api/search", get(search_packages))
        .route("/health", get(health_check))
        .with_state(state);

    // Start the server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("🚀 Registry server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn list_packages(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PackageInfo>>, (StatusCode, String)> {
    let packages = state.db.list_packages().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let mut result = Vec::new();
    for pkg in packages {
        let versions = state.db.get_package_versions(pkg.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        let keywords = state.db.get_keywords(pkg.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        let total_downloads = state.db.get_total_downloads(pkg.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        let mut version_infos = Vec::new();
        for v in versions {
            let authors = state.db.get_authors(v.id).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let dependencies = state.db.get_dependencies(v.id).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
            version_infos.push(VersionInfo {
                version: v.version,
                description: v.description,
                license: v.license,
                authors,
                dependencies,
                published_at: v.published_at,
                downloads: v.downloads,
                checksum: v.checksum,
            });
        }
        
        result.push(PackageInfo {
            name: pkg.name,
            description: pkg.description,
            repository: pkg.repository,
            versions: version_infos,
            keywords,
            total_downloads,
        });
    }
    
    Ok(Json(result))
}

async fn get_package_info(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<PackageInfo>, (StatusCode, String)> {
    let package = state.db.get_package(&name).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Package not found".to_string()))?;
    
    let versions = state.db.get_package_versions(package.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let keywords = state.db.get_keywords(package.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let total_downloads = state.db.get_total_downloads(package.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let mut version_infos = Vec::new();
    for v in versions {
        let authors = state.db.get_authors(v.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let dependencies = state.db.get_dependencies(v.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        version_infos.push(VersionInfo {
            version: v.version,
            description: v.description,
            license: v.license,
            authors,
            dependencies,
            published_at: v.published_at,
            downloads: v.downloads,
            checksum: v.checksum,
        });
    }
    
    Ok(Json(PackageInfo {
        name: package.name,
        description: package.description,
        repository: package.repository,
        versions: version_infos,
        keywords,
        total_downloads,
    }))
}

async fn publish_package(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
    Json(req): Json<PublishRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("📦 Publishing package: {} v{}", name, version);
    
    // Validate package name and version match
    if req.name != name || req.version != version {
        return Err((StatusCode::BAD_REQUEST, "Package name or version mismatch".to_string()));
    }
    
    // Calculate checksum
    let checksum = format!("{:x}", sha2::Sha256::digest(&req.tarball));
    
    // Upload tarball to storage
    let tarball_key = format!("packages/{}/{}.tar.gz", name, version);
    state.storage.store_tarball(&name, &version, &req.tarball).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error: {}", e)))?;
    
    // Create or update package in database
    let package_id = state.db.upsert_package(&name, req.description.as_deref(), req.repository.as_deref()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Create package version
    let version_id = state.db.create_package_version(
        package_id,
        &version,
        req.description.as_deref(),
        req.license.as_deref(),
        &checksum,
        &tarball_key,
        req.tarball.len() as i64,
    ).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Add authors
    state.db.add_authors(version_id, &req.authors).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Add keywords
    state.db.add_keywords(package_id, &req.keywords).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Add dependencies
    state.db.add_dependencies(version_id, &req.dependencies).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    info!("✅ Published: {} v{}", name, version);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Package {} v{} published successfully", name, version)
    })))
}

async fn download_package(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("📥 Download request: {} v{}", name, version);
    
    // Get package from database
    let package = state.db.get_package(&name).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Package not found".to_string()))?;
    
    // Get specific version
    let pkg_version = state.db.get_package_version(package.id, &version).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Version not found".to_string()))?;
    
    // Download tarball from storage
    let tarball_data = state.storage.retrieve_tarball(&name, &version).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error: {}", e)))?;
    
    // Increment download counter
    state.db.increment_downloads(pkg_version.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    info!("✅ Downloaded: {} v{} ({} bytes)", name, version, tarball_data.len());
    
    Ok((
        StatusCode::OK,
        [("Content-Type", "application/gzip")],
        tarball_data,
    ))
}

async fn delete_package(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("🗑️  Delete request: {} v{}", name, version);
    
    // Get package from database
    let package = state.db.get_package(&name).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Package not found".to_string()))?;
    
    // Get specific version
    let pkg_version = state.db.get_package_version(package.id, &version).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Version not found".to_string()))?;
    
    // Delete from storage
    state.storage.delete_tarball(&name, &version).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error: {}", e)))?;
    
    // Delete from database (cascade will handle related records)
    state.db.delete_package_version(pkg_version.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    info!("✅ Deleted: {} v{}", name, version);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Package {} v{} deleted successfully", name, version)
    })))
}

async fn search_packages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    info!("🔍 Search query: {}", query.q);
    
    let packages = state.db.search_packages(&query.q, query.limit as u64).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let mut result = Vec::new();
    for pkg in packages {
        let versions = state.db.get_package_versions(pkg.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        let keywords = state.db.get_keywords(pkg.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        let total_downloads = state.db.get_total_downloads(pkg.id).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        let mut version_infos = Vec::new();
        for v in versions {
            let authors = state.db.get_authors(v.id).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let dependencies = state.db.get_dependencies(v.id).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
            version_infos.push(VersionInfo {
                version: v.version,
                description: v.description,
                license: v.license,
                authors,
                dependencies,
                published_at: v.published_at,
                downloads: v.downloads,
                checksum: v.checksum,
            });
        }
        
        result.push(PackageInfo {
            name: pkg.name,
            description: pkg.description,
            repository: pkg.repository,
            versions: version_infos,
            keywords,
            total_downloads,
        });
    }
    
    let total = result.len();
    Ok(Json(SearchResponse {
        packages: result,
        total,
    }))
}

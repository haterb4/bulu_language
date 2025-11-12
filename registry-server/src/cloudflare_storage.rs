//! Cloudflare R2 storage backend using AWS SDK
//! 
//! R2 is S3-compatible, so we use the official AWS SDK

use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client,
};
use crate::error::RegistryError;

pub struct CloudflareStorage {
    client: Client,
    bucket_name: String,
}

impl CloudflareStorage {
    pub fn new(
        account_id: String,
        bucket_name: String,
        access_key_id: String,
        secret_access_key: String,
    ) -> Self {
        // Cloudflare R2 endpoint
        let endpoint = format!("https://{}.r2.cloudflarestorage.com", account_id);
        
        // Create credentials
        let credentials = Credentials::new(
            access_key_id,
            secret_access_key,
            None, // session token
            None, // expiry
            "cloudflare-r2",
        );
        
        // Create S3 config for R2
        let config = aws_sdk_s3::Config::builder()
            .credentials_provider(credentials)
            .region(Region::new("auto")) // R2 uses "auto" region
            .endpoint_url(endpoint)
            .force_path_style(true) // Required for R2
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .build();
        
        let client = Client::from_conf(config);
        
        Self {
            client,
            bucket_name,
        }
    }

    /// Upload tarball to R2
    pub async fn upload_tarball(
        &self,
        package_name: &str,
        version: &str,
        tarball_data: &[u8],
    ) -> Result<String, RegistryError> {
        let key = format!("packages/{}/{}.tar.gz", package_name, version);
        
        let body = ByteStream::from(tarball_data.to_vec());
        
        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(body)
            .content_type("application/gzip")
            .send()
            .await
            .map_err(|e| RegistryError::StorageError(format!("R2 upload failed: {}", e)))?;
        
        let url = format!("https://{}.r2.cloudflarestorage.com/{}/{}", 
            self.bucket_name, self.bucket_name, key);
        
        Ok(url)
    }

    /// Download tarball from R2
    pub async fn download_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<u8>, RegistryError> {
        let key = format!("packages/{}/{}.tar.gz", package_name, version);
        
        let response = self.client
            .get_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await
            .map_err(|e| RegistryError::StorageError(format!("R2 download failed: {}", e)))?;
        
        let bytes = response.body.collect().await
            .map_err(|e| RegistryError::StorageError(format!("Failed to read R2 response: {}", e)))?;
        
        Ok(bytes.to_vec())
    }

    /// Delete tarball from R2
    pub async fn delete_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<(), RegistryError> {
        let key = format!("packages/{}/{}.tar.gz", package_name, version);
        
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await
            .map_err(|e| RegistryError::StorageError(format!("R2 delete failed: {}", e)))?;
        
        Ok(())
    }

    /// List versions for a package
    pub async fn list_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, RegistryError> {
        let prefix = format!("packages/{}/", package_name);
        
        let response = self.client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .prefix(&prefix)
            .send()
            .await
            .map_err(|e| RegistryError::StorageError(format!("R2 list failed: {}", e)))?;
        
        let mut versions = Vec::new();
        
        if let Some(contents) = response.contents {
            for object in contents {
                if let Some(key) = object.key {
                    // Extract version from key: packages/name/version.tar.gz
                    if let Some(filename) = key.strip_prefix(&prefix) {
                        if let Some(version) = filename.strip_suffix(".tar.gz") {
                            versions.push(version.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(versions)
    }
}

// Implement StorageBackend trait
#[async_trait::async_trait]
impl crate::storage::StorageBackend for CloudflareStorage {
    async fn store_tarball(
        &self,
        package_name: &str,
        version: &str,
        tarball_data: &[u8],
    ) -> Result<String, RegistryError> {
        self.upload_tarball(package_name, version, tarball_data).await
    }

    async fn retrieve_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<u8>, RegistryError> {
        self.download_tarball(package_name, version).await
    }

    async fn delete_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<(), RegistryError> {
        self.delete_tarball(package_name, version).await
    }

    async fn list_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, RegistryError> {
        self.list_versions(package_name).await
    }
}

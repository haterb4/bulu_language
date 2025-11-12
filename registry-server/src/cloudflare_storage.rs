//! Cloudflare R2 storage backend for package tarballs
//! 
//! R2 is Cloudflare's S3-compatible object storage service.
//! This module handles uploading and downloading package tarballs to/from R2.

use reqwest::Client;
use std::collections::HashMap;
use crate::error::RegistryError;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};

type HmacSha256 = Hmac<Sha256>;

pub struct CloudflareStorage {
    client: Client,
    account_id: String,
    bucket_name: String,
    access_key_id: String,
    secret_access_key: String,
    endpoint: String,
}

impl CloudflareStorage {
    pub fn new(
        account_id: String,
        bucket_name: String,
        access_key_id: String,
        secret_access_key: String,
    ) -> Self {
        let endpoint = format!("https://{}.r2.cloudflarestorage.com", account_id);
        
        Self {
            client: Client::new(),
            account_id,
            bucket_name,
            access_key_id,
            secret_access_key,
            endpoint,
        }
    }

    /// Generate AWS Signature V4 for Cloudflare R2
    fn generate_signature(
        &self,
        method: &str,
        path: &str,
        query_params: &HashMap<String, String>,
        headers: &HashMap<String, String>,
        payload_hash: &str,
        timestamp: &DateTime<Utc>,
    ) -> Result<String, RegistryError> {
        let date = timestamp.format("%Y%m%d").to_string();
        let datetime = timestamp.format("%Y%m%dT%H%M%SZ").to_string();
        let region = "auto"; // Cloudflare R2 uses "auto" region
        let service = "s3";
        
        // Create canonical request
        let mut canonical_headers = String::new();
        let mut signed_headers = Vec::new();
        
        for (key, value) in headers {
            canonical_headers.push_str(&format!("{}:{}\n", key.to_lowercase(), value));
            signed_headers.push(key.to_lowercase());
        }
        signed_headers.sort();
        let signed_headers_str = signed_headers.join(";");
        
        let mut query_string = String::new();
        if !query_params.is_empty() {
            let mut params: Vec<_> = query_params.iter().collect();
            params.sort_by_key(|(k, _)| *k);
            query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
        }
        
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method, path, query_string, canonical_headers, signed_headers_str, payload_hash
        );
        
        // Create string to sign
        let credential_scope = format!("{}/{}/{}/aws4_request", date, region, service);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            datetime,
            credential_scope,
            hex::encode(sha2::Sha256::digest(canonical_request.as_bytes()))
        );
        
        // Calculate signature
        let mut mac = HmacSha256::new_from_slice(
            format!("AWS4{}", self.secret_access_key).as_bytes()
        ).map_err(|e| RegistryError::StorageError(format!("HMAC error: {}", e)))?;
        
        mac.update(date.as_bytes());
        let date_key = mac.finalize().into_bytes();
        
        let mut mac = HmacSha256::new_from_slice(&date_key)
            .map_err(|e| RegistryError::StorageError(format!("HMAC error: {}", e)))?;
        mac.update(region.as_bytes());
        let region_key = mac.finalize().into_bytes();
        
        let mut mac = HmacSha256::new_from_slice(&region_key)
            .map_err(|e| RegistryError::StorageError(format!("HMAC error: {}", e)))?;
        mac.update(service.as_bytes());
        let service_key = mac.finalize().into_bytes();
        
        let mut mac = HmacSha256::new_from_slice(&service_key)
            .map_err(|e| RegistryError::StorageError(format!("HMAC error: {}", e)))?;
        mac.update(b"aws4_request");
        let signing_key = mac.finalize().into_bytes();
        
        let mut mac = HmacSha256::new_from_slice(&signing_key)
            .map_err(|e| RegistryError::StorageError(format!("HMAC error: {}", e)))?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        
        Ok(signature)
    }

    /// Upload tarball to Cloudflare R2
    pub async fn upload_tarball(
        &self,
        package_name: &str,
        version: &str,
        tarball_data: &[u8],
    ) -> Result<String, RegistryError> {
        let key = format!("packages/{}/{}.tar.gz", package_name, version);
        let url = format!("{}/{}/{}", self.endpoint, self.bucket_name, key);
        
        let timestamp = Utc::now();
        let payload_hash = hex::encode(sha2::Sha256::digest(tarball_data));
        
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), format!("{}.r2.cloudflarestorage.com", self.account_id));
        headers.insert("x-amz-date".to_string(), timestamp.format("%Y%m%dT%H%M%SZ").to_string());
        headers.insert("x-amz-content-sha256".to_string(), payload_hash.clone());
        headers.insert("content-type".to_string(), "application/gzip".to_string());
        
        let signature = self.generate_signature(
            "PUT",
            &format!("/{}/{}", self.bucket_name, key),
            &HashMap::new(),
            &headers,
            &payload_hash,
            &timestamp,
        )?;
        
        let credential = format!(
            "{}/{}/auto/s3/aws4_request",
            self.access_key_id,
            timestamp.format("%Y%m%d")
        );
        
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}, SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date, Signature={}",
            credential, signature
        );
        
        let response = self.client
            .put(&url)
            .header("Authorization", authorization)
            .header("x-amz-date", timestamp.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-content-sha256", payload_hash)
            .header("Content-Type", "application/gzip")
            .body(tarball_data.to_vec())
            .send()
            .await
            .map_err(|e| RegistryError::StorageError(format!("Upload failed: {}", e)))?;
        
        if response.status().is_success() {
            Ok(url)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(RegistryError::StorageError(format!("Upload failed: {}", error_text)))
        }
    }

    /// Download tarball from Cloudflare R2
    pub async fn download_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<u8>, RegistryError> {
        let key = format!("packages/{}/{}.tar.gz", package_name, version);
        let url = format!("{}/{}/{}", self.endpoint, self.bucket_name, key);
        
        let timestamp = Utc::now();
        let payload_hash = hex::encode(sha2::Sha256::digest(b""));
        
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), format!("{}.r2.cloudflarestorage.com", self.account_id));
        headers.insert("x-amz-date".to_string(), timestamp.format("%Y%m%dT%H%M%SZ").to_string());
        headers.insert("x-amz-content-sha256".to_string(), payload_hash.clone());
        
        let signature = self.generate_signature(
            "GET",
            &format!("/{}/{}", self.bucket_name, key),
            &HashMap::new(),
            &headers,
            &payload_hash,
            &timestamp,
        )?;
        
        let credential = format!(
            "{}/{}/auto/s3/aws4_request",
            self.access_key_id,
            timestamp.format("%Y%m%d")
        );
        
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature={}",
            credential, signature
        );
        
        let response = self.client
            .get(&url)
            .header("Authorization", authorization)
            .header("x-amz-date", timestamp.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-content-sha256", payload_hash)
            .send()
            .await
            .map_err(|e| RegistryError::StorageError(format!("Download failed: {}", e)))?;
        
        if response.status().is_success() {
            let bytes = response.bytes().await
                .map_err(|e| RegistryError::StorageError(format!("Failed to read response: {}", e)))?;
            Ok(bytes.to_vec())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(RegistryError::StorageError(format!("Download failed: {}", error_text)))
        }
    }

    /// Delete tarball from Cloudflare R2
    pub async fn delete_tarball(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<(), RegistryError> {
        let key = format!("packages/{}/{}.tar.gz", package_name, version);
        let url = format!("{}/{}/{}", self.endpoint, self.bucket_name, key);
        
        let timestamp = Utc::now();
        let payload_hash = hex::encode(sha2::Sha256::digest(b""));
        
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), format!("{}.r2.cloudflarestorage.com", self.account_id));
        headers.insert("x-amz-date".to_string(), timestamp.format("%Y%m%dT%H%M%SZ").to_string());
        headers.insert("x-amz-content-sha256".to_string(), payload_hash.clone());
        
        let signature = self.generate_signature(
            "DELETE",
            &format!("/{}/{}", self.bucket_name, key),
            &HashMap::new(),
            &headers,
            &payload_hash,
            &timestamp,
        )?;
        
        let credential = format!(
            "{}/{}/auto/s3/aws4_request",
            self.access_key_id,
            timestamp.format("%Y%m%d")
        );
        
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature={}",
            credential, signature
        );
        
        let response = self.client
            .delete(&url)
            .header("Authorization", authorization)
            .header("x-amz-date", timestamp.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-content-sha256", payload_hash)
            .send()
            .await
            .map_err(|e| RegistryError::StorageError(format!("Delete failed: {}", e)))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(RegistryError::StorageError(format!("Delete failed: {}", error_text)))
        }
    }

    /// List versions for a package
    pub async fn list_versions(
        &self,
        package_name: &str,
    ) -> Result<Vec<String>, RegistryError> {
        let prefix = format!("packages/{}/", package_name);
        let mut query_params = HashMap::new();
        query_params.insert("list-type".to_string(), "2".to_string());
        query_params.insert("prefix".to_string(), prefix.clone());
        
        let url = format!("{}/{}", self.endpoint, self.bucket_name);
        let timestamp = Utc::now();
        let payload_hash = hex::encode(sha2::Sha256::digest(b""));
        
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), format!("{}.r2.cloudflarestorage.com", self.account_id));
        headers.insert("x-amz-date".to_string(), timestamp.format("%Y%m%dT%H%M%SZ").to_string());
        headers.insert("x-amz-content-sha256".to_string(), payload_hash.clone());
        
        let signature = self.generate_signature(
            "GET",
            &format!("/{}", self.bucket_name),
            &query_params,
            &headers,
            &payload_hash,
            &timestamp,
        )?;
        
        let credential = format!(
            "{}/{}/auto/s3/aws4_request",
            self.access_key_id,
            timestamp.format("%Y%m%d")
        );
        
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature={}",
            credential, signature
        );
        
        let response = self.client
            .get(&url)
            .query(&query_params)
            .header("Authorization", authorization)
            .header("x-amz-date", timestamp.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-content-sha256", payload_hash)
            .send()
            .await
            .map_err(|e| RegistryError::StorageError(format!("List failed: {}", e)))?;
        
        if response.status().is_success() {
            let xml_text = response.text().await
                .map_err(|e| RegistryError::StorageError(format!("Failed to read response: {}", e)))?;
            
            // Parse XML to extract versions (simplified)
            let mut versions = Vec::new();
            for line in xml_text.lines() {
                if line.contains("<Key>") && line.contains(&prefix) {
                    if let Some(start) = line.find(&prefix) {
                        if let Some(end) = line.find(".tar.gz</Key>") {
                            let version_start = start + prefix.len();
                            let version = &line[version_start..end];
                            versions.push(version.to_string());
                        }
                    }
                }
            }
            
            Ok(versions)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(RegistryError::StorageError(format!("List failed: {}", error_text)))
        }
    }
}


// Implement StorageBackend trait for CloudflareStorage
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

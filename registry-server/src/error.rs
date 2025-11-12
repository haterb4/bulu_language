//! Error types for the registry

use std::fmt;

#[derive(Debug)]
pub enum RegistryError {
    StorageError(String),
    DatabaseError(String),
    NotFound(String),
    InvalidRequest(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            RegistryError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            RegistryError::NotFound(msg) => write!(f, "Not found: {}", msg),
            RegistryError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}

impl From<sea_orm::DbErr> for RegistryError {
    fn from(err: sea_orm::DbErr) -> Self {
        RegistryError::DatabaseError(err.to_string())
    }
}

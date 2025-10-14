// Database module for the Bulu programming language
// Provides SQL database operations with connection pooling and transaction support

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Database connection configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub driver: DatabaseDriver,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
}

/// Supported database drivers
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseDriver {
    PostgreSQL,
    MySQL,
    SQLite,
}

impl DatabaseDriver {
    pub fn from_string(driver: &str) -> Result<Self, String> {
        match driver.to_lowercase().as_str() {
            "postgresql" | "postgres" => Ok(DatabaseDriver::PostgreSQL),
            "mysql" => Ok(DatabaseDriver::MySQL),
            "sqlite" => Ok(DatabaseDriver::SQLite),
            _ => Err(format!("Unsupported database driver: {}", driver)),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DatabaseDriver::PostgreSQL => "postgresql".to_string(),
            DatabaseDriver::MySQL => "mysql".to_string(),
            DatabaseDriver::SQLite => "sqlite".to_string(),
        }
    }
}

/// Database connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    InTransaction,
    Error(String),
}

/// Database connection
#[derive(Debug)]
pub struct DatabaseConnection {
    pub id: String,
    pub config: DatabaseConfig,
    pub state: ConnectionState,
    pub created_at: Instant,
    pub last_used: Instant,
    pub query_count: u64,
}

impl DatabaseConnection {
    /// Create a new database connection
    pub fn new(id: String, config: DatabaseConfig) -> Self {
        let now = Instant::now();
        Self {
            id,
            config,
            state: ConnectionState::Disconnected,
            created_at: now,
            last_used: now,
            query_count: 0,
        }
    }

    /// Connect to the database
    pub fn connect(&mut self) -> Result<(), String> {
        // Simulate connection logic based on driver
        match self.config.driver {
            DatabaseDriver::PostgreSQL => {
                // PostgreSQL connection logic would go here
                self.state = ConnectionState::Connected;
                Ok(())
            }
            DatabaseDriver::MySQL => {
                // MySQL connection logic would go here
                self.state = ConnectionState::Connected;
                Ok(())
            }
            DatabaseDriver::SQLite => {
                // SQLite connection logic would go here
                self.state = ConnectionState::Connected;
                Ok(())
            }
        }
    }

    /// Disconnect from the database
    pub fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    /// Execute a SQL query
    pub fn execute(&mut self, sql: &str, _params: Vec<SqlValue>) -> Result<QueryResult, String> {
        if self.state != ConnectionState::Connected && self.state != ConnectionState::InTransaction {
            return Err("Connection is not active".to_string());
        }

        self.last_used = Instant::now();
        self.query_count += 1;

        // Simulate query execution
        let affected_rows = if sql.trim().to_lowercase().starts_with("select") {
            0 // SELECT queries don't affect rows
        } else {
            1 // Simulate 1 affected row for other queries
        };

        Ok(QueryResult {
            affected_rows,
            rows: vec![], // Would contain actual result rows
            columns: vec![], // Would contain column metadata
        })
    }

    /// Begin a transaction
    pub fn begin_transaction(&mut self) -> Result<Transaction, String> {
        if self.state != ConnectionState::Connected {
            return Err("Connection is not active".to_string());
        }

        self.state = ConnectionState::InTransaction;
        Ok(Transaction {
            connection_id: self.id.clone(),
            started_at: Instant::now(),
            is_active: true,
        })
    }

    /// Check if connection is idle
    pub fn is_idle(&self) -> bool {
        self.last_used.elapsed() > self.config.idle_timeout
    }

    /// Check if connection is expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > Duration::from_secs(3600) // 1 hour max lifetime
    }
}

/// SQL value types
#[derive(Debug, Clone)]
pub enum SqlValue {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Bytes(Vec<u8>),
}

/// Query result
#[derive(Debug)]
pub struct QueryResult {
    pub affected_rows: u64,
    pub rows: Vec<HashMap<String, SqlValue>>,
    pub columns: Vec<ColumnInfo>,
}

/// Column metadata
#[derive(Debug)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// Database transaction
#[derive(Debug)]
pub struct Transaction {
    pub connection_id: String,
    pub started_at: Instant,
    pub is_active: bool,
}

impl Transaction {
    /// Commit the transaction
    pub fn commit(&mut self) -> Result<(), String> {
        if !self.is_active {
            return Err("Transaction is not active".to_string());
        }
        
        self.is_active = false;
        // Simulate commit logic
        Ok(())
    }

    /// Rollback the transaction
    pub fn rollback(&mut self) -> Result<(), String> {
        if !self.is_active {
            return Err("Transaction is not active".to_string());
        }
        
        self.is_active = false;
        // Simulate rollback logic
        Ok(())
    }
}

/// Connection pool for managing database connections
#[derive(Debug)]
pub struct ConnectionPool {
    config: DatabaseConfig,
    connections: Arc<Mutex<Vec<DatabaseConnection>>>,
    active_connections: Arc<Mutex<HashMap<String, DatabaseConnection>>>,
    connection_counter: Arc<Mutex<u64>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            connections: Arc::new(Mutex::new(Vec::new())),
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            connection_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Get a connection from the pool
    pub fn get_connection(&self) -> Result<String, String> {
        let mut connections = self.connections.lock().unwrap();
        let mut active = self.active_connections.lock().unwrap();

        // Try to reuse an existing idle connection
        if let Some(mut conn) = connections.pop() {
            if !conn.is_expired() {
                conn.last_used = Instant::now();
                let conn_id = conn.id.clone();
                active.insert(conn_id.clone(), conn);
                return Ok(conn_id);
            }
        }

        // Create a new connection if under the limit
        if active.len() < self.config.max_connections {
            let mut counter = self.connection_counter.lock().unwrap();
            *counter += 1;
            let conn_id = format!("conn_{}", *counter);
            
            let mut conn = DatabaseConnection::new(conn_id.clone(), self.config.clone());
            conn.connect()?;
            
            active.insert(conn_id.clone(), conn);
            Ok(conn_id)
        } else {
            Err("Connection pool exhausted".to_string())
        }
    }

    /// Return a connection to the pool
    pub fn return_connection(&self, connection_id: &str) -> Result<(), String> {
        let mut active = self.active_connections.lock().unwrap();
        let mut connections = self.connections.lock().unwrap();

        if let Some(mut conn) = active.remove(connection_id) {
            // Reset connection state
            if conn.state == ConnectionState::InTransaction {
                conn.state = ConnectionState::Connected;
            }
            
            // Return to pool if not expired
            if !conn.is_expired() {
                connections.push(conn);
            } else {
                conn.disconnect();
            }
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    /// Execute a query using a pooled connection
    pub fn execute(&self, sql: &str, params: Vec<SqlValue>) -> Result<QueryResult, String> {
        let conn_id = self.get_connection()?;
        
        let result = {
            let mut active = self.active_connections.lock().unwrap();
            if let Some(conn) = active.get_mut(&conn_id) {
                conn.execute(sql, params)
            } else {
                Err("Connection not found".to_string())
            }
        };

        self.return_connection(&conn_id)?;
        result
    }

    /// Begin a transaction
    pub fn begin_transaction(&self) -> Result<(String, Transaction), String> {
        let conn_id = self.get_connection()?;
        
        let transaction = {
            let mut active = self.active_connections.lock().unwrap();
            if let Some(conn) = active.get_mut(&conn_id) {
                conn.begin_transaction()
            } else {
                return Err("Connection not found".to_string());
            }
        }?;

        Ok((conn_id, transaction))
    }

    /// Commit a transaction
    pub fn commit_transaction(&self, connection_id: &str, mut transaction: Transaction) -> Result<(), String> {
        transaction.commit()?;
        
        // Update connection state before returning to pool
        {
            let mut active = self.active_connections.lock().unwrap();
            if let Some(conn) = active.get_mut(connection_id) {
                conn.state = ConnectionState::Connected;
            }
        }
        
        self.return_connection(connection_id)?;
        Ok(())
    }

    /// Rollback a transaction
    pub fn rollback_transaction(&self, connection_id: &str, mut transaction: Transaction) -> Result<(), String> {
        transaction.rollback()?;
        
        // Update connection state before returning to pool
        {
            let mut active = self.active_connections.lock().unwrap();
            if let Some(conn) = active.get_mut(connection_id) {
                conn.state = ConnectionState::Connected;
            }
        }
        
        self.return_connection(connection_id)?;
        Ok(())
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        let connections = self.connections.lock().unwrap();
        let active = self.active_connections.lock().unwrap();
        
        PoolStats {
            total_connections: connections.len() + active.len(),
            active_connections: active.len(),
            idle_connections: connections.len(),
            max_connections: self.config.max_connections,
        }
    }

    /// Clean up expired connections
    pub fn cleanup(&self) {
        let mut connections = self.connections.lock().unwrap();
        connections.retain(|conn| !conn.is_expired());
    }
}

/// Connection pool statistics
#[derive(Debug)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub max_connections: usize,
}

/// Built-in functions for database operations
pub mod builtins {
    use super::*;
    use std::sync::Mutex;

    /// Global connection pools
    static mut CONNECTION_POOLS: Option<Mutex<HashMap<String, ConnectionPool>>> = None;

    /// Initialize the database system
    pub fn init_database() {
        unsafe {
            CONNECTION_POOLS = Some(Mutex::new(HashMap::new()));
        }
    }

    /// Get the global connection pools
    fn get_pools() -> &'static Mutex<HashMap<String, ConnectionPool>> {
        unsafe {
            CONNECTION_POOLS.as_ref().expect("Database system not initialized")
        }
    }

    /// Create a database connection pool
    pub fn db_create_pool(name: &str, config: DatabaseConfig) -> Result<(), String> {
        let mut pools = get_pools().lock().unwrap();
        let pool = ConnectionPool::new(config);
        pools.insert(name.to_string(), pool);
        Ok(())
    }

    /// Execute a SQL query
    pub fn db_execute(pool_name: &str, sql: &str, params: Vec<SqlValue>) -> Result<QueryResult, String> {
        let pools = get_pools().lock().unwrap();
        if let Some(pool) = pools.get(pool_name) {
            pool.execute(sql, params)
        } else {
            Err(format!("Connection pool '{}' not found", pool_name))
        }
    }

    /// Begin a database transaction
    pub fn db_begin_transaction(pool_name: &str) -> Result<String, String> {
        let pools = get_pools().lock().unwrap();
        if let Some(pool) = pools.get(pool_name) {
            let (conn_id, _transaction) = pool.begin_transaction()?;
            Ok(conn_id)
        } else {
            Err(format!("Connection pool '{}' not found", pool_name))
        }
    }

    /// Get connection pool statistics
    pub fn db_get_stats(pool_name: &str) -> Result<PoolStats, String> {
        let pools = get_pools().lock().unwrap();
        if let Some(pool) = pools.get(pool_name) {
            Ok(pool.get_stats())
        } else {
            Err(format!("Connection pool '{}' not found", pool_name))
        }
    }

    /// Clean up expired connections
    pub fn db_cleanup(pool_name: &str) -> Result<(), String> {
        let pools = get_pools().lock().unwrap();
        if let Some(pool) = pools.get(pool_name) {
            pool.cleanup();
            Ok(())
        } else {
            Err(format!("Connection pool '{}' not found", pool_name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> DatabaseConfig {
        DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "test_db".to_string(),
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            driver: DatabaseDriver::PostgreSQL,
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
        }
    }

    #[test]
    fn test_database_driver_from_string() {
        assert_eq!(DatabaseDriver::from_string("postgresql").unwrap(), DatabaseDriver::PostgreSQL);
        assert_eq!(DatabaseDriver::from_string("mysql").unwrap(), DatabaseDriver::MySQL);
        assert_eq!(DatabaseDriver::from_string("sqlite").unwrap(), DatabaseDriver::SQLite);
        assert!(DatabaseDriver::from_string("invalid").is_err());
    }

    #[test]
    fn test_database_connection() {
        let config = create_test_config();
        let mut conn = DatabaseConnection::new("test_conn".to_string(), config);
        
        assert_eq!(conn.state, ConnectionState::Disconnected);
        assert!(conn.connect().is_ok());
        assert_eq!(conn.state, ConnectionState::Connected);
        
        conn.disconnect();
        assert_eq!(conn.state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_connection_execute() {
        let config = create_test_config();
        let mut conn = DatabaseConnection::new("test_conn".to_string(), config);
        conn.connect().unwrap();
        
        let result = conn.execute("SELECT * FROM users", vec![]);
        assert!(result.is_ok());
        assert_eq!(conn.query_count, 1);
    }

    #[test]
    fn test_transaction() {
        let config = create_test_config();
        let mut conn = DatabaseConnection::new("test_conn".to_string(), config);
        conn.connect().unwrap();
        
        let mut transaction = conn.begin_transaction().unwrap();
        assert_eq!(conn.state, ConnectionState::InTransaction);
        assert!(transaction.is_active);
        
        assert!(transaction.commit().is_ok());
        assert!(!transaction.is_active);
    }

    #[test]
    fn test_connection_pool() {
        let config = create_test_config();
        let pool = ConnectionPool::new(config);
        
        let conn_id = pool.get_connection().unwrap();
        assert!(!conn_id.is_empty());
        
        let stats = pool.get_stats();
        assert_eq!(stats.active_connections, 1);
        
        assert!(pool.return_connection(&conn_id).is_ok());
        
        let stats = pool.get_stats();
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.idle_connections, 1);
    }

    #[test]
    fn test_pool_execute() {
        let config = create_test_config();
        let pool = ConnectionPool::new(config);
        
        let result = pool.execute("SELECT 1", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pool_transaction() {
        let config = create_test_config();
        let pool = ConnectionPool::new(config);
        
        let (conn_id, transaction) = pool.begin_transaction().unwrap();
        assert!(!conn_id.is_empty());
        assert!(transaction.is_active);
        
        assert!(pool.commit_transaction(&conn_id, transaction).is_ok());
    }

    #[test]
    fn test_sql_value() {
        let values = vec![
            SqlValue::Null,
            SqlValue::Integer(42),
            SqlValue::Float(3.14),
            SqlValue::Text("hello".to_string()),
            SqlValue::Boolean(true),
            SqlValue::Bytes(vec![1, 2, 3]),
        ];
        
        assert_eq!(values.len(), 6);
    }
}
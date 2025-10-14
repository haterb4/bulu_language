// Unit tests for the database module

use bulu::std::db::*;
use std::time::Duration;

fn create_test_config() -> DatabaseConfig {
    DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "test_db".to_string(),
        username: "test_user".to_string(),
        password: "test_pass".to_string(),
        driver: DatabaseDriver::PostgreSQL,
        max_connections: 5,
        connection_timeout: Duration::from_secs(30),
        idle_timeout: Duration::from_secs(300),
    }
}

#[test]
fn test_database_driver_parsing() {
    // Test valid drivers
    assert_eq!(DatabaseDriver::from_string("postgresql").unwrap(), DatabaseDriver::PostgreSQL);
    assert_eq!(DatabaseDriver::from_string("postgres").unwrap(), DatabaseDriver::PostgreSQL);
    assert_eq!(DatabaseDriver::from_string("mysql").unwrap(), DatabaseDriver::MySQL);
    assert_eq!(DatabaseDriver::from_string("sqlite").unwrap(), DatabaseDriver::SQLite);
    
    // Test case insensitive
    assert_eq!(DatabaseDriver::from_string("POSTGRESQL").unwrap(), DatabaseDriver::PostgreSQL);
    assert_eq!(DatabaseDriver::from_string("MySQL").unwrap(), DatabaseDriver::MySQL);
    
    // Test invalid driver
    assert!(DatabaseDriver::from_string("invalid").is_err());
    assert!(DatabaseDriver::from_string("oracle").is_err());
}

#[test]
fn test_database_driver_to_string() {
    assert_eq!(DatabaseDriver::PostgreSQL.to_string(), "postgresql");
    assert_eq!(DatabaseDriver::MySQL.to_string(), "mysql");
    assert_eq!(DatabaseDriver::SQLite.to_string(), "sqlite");
}

#[test]
fn test_database_connection_lifecycle() {
    let config = create_test_config();
    let mut conn = DatabaseConnection::new("test_conn".to_string(), config);
    
    // Initial state
    assert_eq!(conn.id, "test_conn");
    assert_eq!(conn.state, ConnectionState::Disconnected);
    assert_eq!(conn.query_count, 0);
    
    // Connect
    assert!(conn.connect().is_ok());
    assert_eq!(conn.state, ConnectionState::Connected);
    
    // Disconnect
    conn.disconnect();
    assert_eq!(conn.state, ConnectionState::Disconnected);
}

#[test]
fn test_database_connection_execute() {
    let config = create_test_config();
    let mut conn = DatabaseConnection::new("test_conn".to_string(), config);
    conn.connect().unwrap();
    
    // Test SELECT query
    let result = conn.execute("SELECT * FROM users", vec![]);
    assert!(result.is_ok());
    assert_eq!(conn.query_count, 1);
    
    let query_result = result.unwrap();
    assert_eq!(query_result.affected_rows, 0); // SELECT doesn't affect rows
    
    // Test INSERT query
    let result = conn.execute("INSERT INTO users (name) VALUES ('test')", vec![]);
    assert!(result.is_ok());
    assert_eq!(conn.query_count, 2);
    
    let query_result = result.unwrap();
    assert_eq!(query_result.affected_rows, 1); // INSERT affects 1 row
    
    // Test query on disconnected connection
    conn.disconnect();
    let result = conn.execute("SELECT 1", vec![]);
    assert!(result.is_err());
}

#[test]
fn test_database_connection_with_parameters() {
    let config = create_test_config();
    let mut conn = DatabaseConnection::new("test_conn".to_string(), config);
    conn.connect().unwrap();
    
    let params = vec![
        SqlValue::Text("John".to_string()),
        SqlValue::Integer(30),
        SqlValue::Boolean(true),
    ];
    
    let result = conn.execute("INSERT INTO users (name, age, active) VALUES (?, ?, ?)", params);
    assert!(result.is_ok());
}

#[test]
fn test_database_transaction() {
    let config = create_test_config();
    let mut conn = DatabaseConnection::new("test_conn".to_string(), config);
    conn.connect().unwrap();
    
    // Begin transaction
    let mut transaction = conn.begin_transaction().unwrap();
    assert_eq!(conn.state, ConnectionState::InTransaction);
    assert!(transaction.is_active);
    
    // Commit transaction
    assert!(transaction.commit().is_ok());
    assert!(!transaction.is_active);
    
    // Reset connection state for next test
    conn.state = ConnectionState::Connected;
    
    // Test rollback
    let mut transaction2 = conn.begin_transaction().unwrap();
    assert!(transaction2.rollback().is_ok());
    assert!(!transaction2.is_active);
    
    // Test transaction on disconnected connection
    conn.disconnect();
    let result = conn.begin_transaction();
    assert!(result.is_err());
}

#[test]
fn test_connection_idle_and_expiry() {
    let mut config = create_test_config();
    config.idle_timeout = Duration::from_millis(1); // Very short timeout for testing
    
    let conn = DatabaseConnection::new("test_conn".to_string(), config);
    
    // Should not be idle immediately
    assert!(!conn.is_idle());
    
    // Wait for idle timeout
    std::thread::sleep(Duration::from_millis(10));
    assert!(conn.is_idle());
    
    // Test expiry (connections expire after 1 hour by default)
    assert!(!conn.is_expired()); // Should not be expired immediately
}

#[test]
fn test_connection_pool_creation() {
    let config = create_test_config();
    let pool = ConnectionPool::new(config.clone());
    
    let stats = pool.get_stats();
    assert_eq!(stats.total_connections, 0);
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.idle_connections, 0);
    assert_eq!(stats.max_connections, config.max_connections);
}

#[test]
fn test_connection_pool_get_and_return() {
    let config = create_test_config();
    let pool = ConnectionPool::new(config);
    
    // Get connection
    let conn_id = pool.get_connection().unwrap();
    assert!(!conn_id.is_empty());
    
    let stats = pool.get_stats();
    assert_eq!(stats.active_connections, 1);
    assert_eq!(stats.idle_connections, 0);
    
    // Return connection
    assert!(pool.return_connection(&conn_id).is_ok());
    
    let stats = pool.get_stats();
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.idle_connections, 1);
    
    // Get connection again (should reuse)
    let conn_id2 = pool.get_connection().unwrap();
    assert!(!conn_id2.is_empty());
    
    let stats = pool.get_stats();
    assert_eq!(stats.active_connections, 1);
    assert_eq!(stats.idle_connections, 0);
}

#[test]
fn test_connection_pool_max_connections() {
    let mut config = create_test_config();
    config.max_connections = 2; // Limit to 2 connections
    
    let pool = ConnectionPool::new(config);
    
    // Get maximum connections
    let conn1 = pool.get_connection().unwrap();
    let conn2 = pool.get_connection().unwrap();
    
    let stats = pool.get_stats();
    assert_eq!(stats.active_connections, 2);
    
    // Try to get one more (should fail)
    let result = pool.get_connection();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exhausted"));
    
    // Return one connection
    assert!(pool.return_connection(&conn1).is_ok());
    
    // Now should be able to get another
    let conn3 = pool.get_connection().unwrap();
    assert!(!conn3.is_empty());
}

#[test]
fn test_connection_pool_execute() {
    let config = create_test_config();
    let pool = ConnectionPool::new(config);
    
    // Execute query through pool
    let result = pool.execute("SELECT 1", vec![]);
    assert!(result.is_ok());
    
    let query_result = result.unwrap();
    assert_eq!(query_result.affected_rows, 0); // SELECT doesn't affect rows
    
    // Pool should handle connection automatically
    let stats = pool.get_stats();
    assert_eq!(stats.active_connections, 0); // Connection returned to pool
    assert_eq!(stats.idle_connections, 1);
}

#[test]
fn test_connection_pool_transactions() {
    let config = create_test_config();
    let pool = ConnectionPool::new(config);
    
    // Begin transaction
    let (conn_id, transaction) = pool.begin_transaction().unwrap();
    assert!(!conn_id.is_empty());
    assert!(transaction.is_active);
    
    let stats = pool.get_stats();
    assert_eq!(stats.active_connections, 1);
    
    // Commit transaction
    assert!(pool.commit_transaction(&conn_id, transaction).is_ok());
    
    let stats = pool.get_stats();
    assert_eq!(stats.active_connections, 0); // Connection returned to pool
    
    // Test rollback
    let (conn_id2, transaction2) = pool.begin_transaction().unwrap();
    assert!(pool.rollback_transaction(&conn_id2, transaction2).is_ok());
}

#[test]
fn test_sql_value_types() {
    let values = vec![
        SqlValue::Null,
        SqlValue::Integer(42),
        SqlValue::Float(3.14159),
        SqlValue::Text("Hello, World!".to_string()),
        SqlValue::Boolean(true),
        SqlValue::Bytes(vec![1, 2, 3, 4, 5]),
    ];
    
    assert_eq!(values.len(), 6);
    
    // Test cloning
    let cloned_values = values.clone();
    assert_eq!(cloned_values.len(), values.len());
}

#[test]
fn test_query_result() {
    use std::collections::HashMap;
    
    let mut row = HashMap::new();
    row.insert("id".to_string(), SqlValue::Integer(1));
    row.insert("name".to_string(), SqlValue::Text("John".to_string()));
    row.insert("active".to_string(), SqlValue::Boolean(true));
    
    let column = ColumnInfo {
        name: "id".to_string(),
        data_type: "INTEGER".to_string(),
        nullable: false,
    };
    
    let result = QueryResult {
        affected_rows: 1,
        rows: vec![row],
        columns: vec![column],
    };
    
    assert_eq!(result.affected_rows, 1);
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.columns.len(), 1);
    assert_eq!(result.columns[0].name, "id");
}

#[test]
fn test_builtin_functions() {
    use bulu::std::db::builtins::*;
    
    // Initialize database system
    init_database();
    
    // Create connection pool
    let config = create_test_config();
    assert!(db_create_pool("test_pool", config).is_ok());
    
    // Execute query
    let result = db_execute("test_pool", "SELECT 1", vec![]);
    assert!(result.is_ok());
    
    // Get stats
    let stats = db_get_stats("test_pool").unwrap();
    assert_eq!(stats.max_connections, 5);
    
    // Begin transaction
    let conn_id = db_begin_transaction("test_pool").unwrap();
    assert!(!conn_id.is_empty());
    
    // Cleanup
    assert!(db_cleanup("test_pool").is_ok());
    
    // Test with non-existent pool
    let result = db_execute("non_existent", "SELECT 1", vec![]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_multiple_database_drivers() {
    let mut pg_config = create_test_config();
    pg_config.driver = DatabaseDriver::PostgreSQL;
    
    let mut mysql_config = create_test_config();
    mysql_config.driver = DatabaseDriver::MySQL;
    mysql_config.port = 3306;
    
    let mut sqlite_config = create_test_config();
    sqlite_config.driver = DatabaseDriver::SQLite;
    sqlite_config.database = "/tmp/test.db".to_string();
    
    // Test connections for different drivers
    let mut pg_conn = DatabaseConnection::new("pg_conn".to_string(), pg_config);
    let mut mysql_conn = DatabaseConnection::new("mysql_conn".to_string(), mysql_config);
    let mut sqlite_conn = DatabaseConnection::new("sqlite_conn".to_string(), sqlite_config);
    
    assert!(pg_conn.connect().is_ok());
    assert!(mysql_conn.connect().is_ok());
    assert!(sqlite_conn.connect().is_ok());
    
    assert_eq!(pg_conn.state, ConnectionState::Connected);
    assert_eq!(mysql_conn.state, ConnectionState::Connected);
    assert_eq!(sqlite_conn.state, ConnectionState::Connected);
}

#[test]
fn test_concurrent_pool_access() {
    use std::thread;
    use std::sync::Arc;
    
    let config = create_test_config();
    let pool = Arc::new(ConnectionPool::new(config));
    let mut handles = vec![];
    
    // Spawn multiple threads to test concurrent access
    for i in 0..5 {
        let pool_clone = Arc::clone(&pool);
        let handle = thread::spawn(move || {
            let sql = format!("SELECT {}", i);
            pool_clone.execute(&sql, vec![])
        });
        handles.push(handle);
    }
    
    // Collect results
    let mut results = vec![];
    for handle in handles {
        results.push(handle.join().unwrap());
    }
    
    // All queries should succeed
    for result in results {
        assert!(result.is_ok());
    }
}

#[test]
fn test_connection_pool_cleanup() {
    let config = create_test_config();
    let pool = ConnectionPool::new(config);
    
    // Get and return some connections to populate the pool
    let conn1 = pool.get_connection().unwrap();
    let conn2 = pool.get_connection().unwrap();
    
    pool.return_connection(&conn1).unwrap();
    pool.return_connection(&conn2).unwrap();
    
    let stats_before = pool.get_stats();
    assert_eq!(stats_before.idle_connections, 2);
    
    // Cleanup should not remove non-expired connections
    pool.cleanup();
    
    let stats_after = pool.get_stats();
    assert_eq!(stats_after.idle_connections, 2); // Should remain the same
}

#[test]
fn test_transaction_inactive_operations() {
    let mut transaction = Transaction {
        connection_id: "test_conn".to_string(),
        started_at: std::time::Instant::now(),
        is_active: false, // Already inactive
    };
    
    // Operations on inactive transaction should fail
    assert!(transaction.commit().is_err());
    assert!(transaction.rollback().is_err());
}

#[test]
fn test_database_config_clone() {
    let config = create_test_config();
    let cloned_config = config.clone();
    
    assert_eq!(config.host, cloned_config.host);
    assert_eq!(config.port, cloned_config.port);
    assert_eq!(config.database, cloned_config.database);
    assert_eq!(config.driver, cloned_config.driver);
    assert_eq!(config.max_connections, cloned_config.max_connections);
}
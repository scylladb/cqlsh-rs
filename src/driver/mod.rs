//! Driver abstraction layer for CQL database connectivity.
//!
//! Provides a trait-based abstraction over the underlying database driver,
//! enabling testability and future flexibility. The primary implementation
//! uses the `scylla` crate for Cassandra/ScyllaDB connectivity.
//!
//! Many types and trait methods are defined ahead of their use in later
//! development phases (REPL, DESCRIBE, COPY, etc.).

pub mod scylla_driver;
pub mod types;

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

pub use scylla_driver::ScyllaDriver;
#[allow(unused_imports)]
pub use types::{CqlColumn, CqlResult, CqlRow, CqlValue};

/// Configuration for establishing a database connection.
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Contact point host (e.g., "127.0.0.1").
    pub host: String,
    /// Native transport port (default: 9042).
    pub port: u16,
    /// Optional username for authentication.
    pub username: Option<String>,
    /// Optional password for authentication.
    pub password: Option<String>,
    /// Optional default keyspace.
    pub keyspace: Option<String>,
    /// Connection timeout in seconds.
    pub connect_timeout: u64,
    /// Per-request timeout in seconds.
    pub request_timeout: u64,
    /// Whether to use SSL/TLS.
    pub ssl: bool,
    /// SSL/TLS configuration.
    pub ssl_config: Option<SslConfig>,
    /// Protocol version (None = auto-negotiate).
    pub protocol_version: Option<u8>,
}

/// SSL/TLS configuration options.
#[derive(Debug, Clone)]
pub struct SslConfig {
    /// Path to CA certificate file for server verification.
    pub certfile: Option<String>,
    /// Whether to validate the server certificate.
    pub validate: bool,
    /// Path to client private key file (for mutual TLS).
    pub userkey: Option<String>,
    /// Path to client certificate file (for mutual TLS).
    pub usercert: Option<String>,
    /// Per-host certificate files.
    pub host_certfiles: HashMap<String, String>,
}

/// Metadata about a column in a result set.
#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    pub type_name: String,
}

/// Metadata about a keyspace.
#[derive(Debug, Clone)]
pub struct KeyspaceMetadata {
    pub name: String,
    pub replication: HashMap<String, String>,
    pub durable_writes: bool,
}

/// Metadata about a table.
#[derive(Debug, Clone)]
pub struct TableMetadata {
    pub keyspace: String,
    pub name: String,
    pub columns: Vec<ColumnMetadata>,
    pub partition_key: Vec<String>,
    pub clustering_key: Vec<String>,
}

/// Consistency levels matching CQL specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consistency {
    Any,
    One,
    Two,
    Three,
    Quorum,
    All,
    LocalQuorum,
    EachQuorum,
    Serial,
    LocalSerial,
    LocalOne,
}

impl Consistency {
    /// Parse a consistency level from a string (case-insensitive).
    pub fn from_str_cql(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ANY" => Some(Self::Any),
            "ONE" => Some(Self::One),
            "TWO" => Some(Self::Two),
            "THREE" => Some(Self::Three),
            "QUORUM" => Some(Self::Quorum),
            "ALL" => Some(Self::All),
            "LOCAL_QUORUM" => Some(Self::LocalQuorum),
            "EACH_QUORUM" => Some(Self::EachQuorum),
            "SERIAL" => Some(Self::Serial),
            "LOCAL_SERIAL" => Some(Self::LocalSerial),
            "LOCAL_ONE" => Some(Self::LocalOne),
            _ => None,
        }
    }

    /// Return the CQL string representation.
    pub fn as_cql_str(&self) -> &'static str {
        match self {
            Self::Any => "ANY",
            Self::One => "ONE",
            Self::Two => "TWO",
            Self::Three => "THREE",
            Self::Quorum => "QUORUM",
            Self::All => "ALL",
            Self::LocalQuorum => "LOCAL_QUORUM",
            Self::EachQuorum => "EACH_QUORUM",
            Self::Serial => "SERIAL",
            Self::LocalSerial => "LOCAL_SERIAL",
            Self::LocalOne => "LOCAL_ONE",
        }
    }
}

impl std::fmt::Display for Consistency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_cql_str())
    }
}

/// The core driver trait abstracting database operations.
///
/// All methods are async and return `Result` for proper error propagation.
/// Implementations must be `Send + Sync` for use across async tasks.
#[async_trait]
pub trait CqlDriver: Send + Sync {
    /// Establish a connection to the database cluster.
    async fn connect(config: &ConnectionConfig) -> Result<Self>
    where
        Self: Sized;

    /// Execute a raw CQL query string without parameters.
    async fn execute_unpaged(&self, query: &str) -> Result<CqlResult>;

    /// Execute a CQL query with automatic paging, returning all rows.
    async fn execute_paged(&self, query: &str, page_size: i32) -> Result<CqlResult>;

    /// Prepare a CQL statement for repeated execution.
    async fn prepare(&self, query: &str) -> Result<PreparedId>;

    /// Execute a previously prepared statement with the given values.
    async fn execute_prepared(
        &self,
        prepared_id: &PreparedId,
        values: &[CqlValue],
    ) -> Result<CqlResult>;

    /// Switch the current keyspace (USE <keyspace>).
    async fn use_keyspace(&self, keyspace: &str) -> Result<()>;

    /// Get the current consistency level.
    fn get_consistency(&self) -> Consistency;

    /// Set the consistency level for subsequent queries.
    fn set_consistency(&self, consistency: Consistency);

    /// Get the current serial consistency level.
    fn get_serial_consistency(&self) -> Option<Consistency>;

    /// Set the serial consistency level for subsequent queries.
    fn set_serial_consistency(&self, consistency: Option<Consistency>);

    /// Enable or disable request tracing.
    fn set_tracing(&self, enabled: bool);

    /// Check if tracing is currently enabled.
    fn is_tracing_enabled(&self) -> bool;

    /// Get the last tracing session ID (if tracing was enabled).
    fn last_trace_id(&self) -> Option<uuid::Uuid>;

    /// Retrieve tracing session data for a given trace ID.
    async fn get_trace_session(&self, trace_id: uuid::Uuid) -> Result<Option<TracingSession>>;

    /// Get metadata for all keyspaces.
    async fn get_keyspaces(&self) -> Result<Vec<KeyspaceMetadata>>;

    /// Get metadata for all tables in a keyspace.
    async fn get_tables(&self, keyspace: &str) -> Result<Vec<TableMetadata>>;

    /// Get metadata for a specific table.
    async fn get_table_metadata(
        &self,
        keyspace: &str,
        table: &str,
    ) -> Result<Option<TableMetadata>>;

    /// Get the cluster name.
    async fn get_cluster_name(&self) -> Result<Option<String>>;

    /// Get the CQL version from the connected node.
    async fn get_cql_version(&self) -> Result<Option<String>>;

    /// Get the release version of the connected node.
    async fn get_release_version(&self) -> Result<Option<String>>;

    /// Get the ScyllaDB version (None if not ScyllaDB).
    async fn get_scylla_version(&self) -> Result<Option<String>>;

    /// Check if the connection is still alive.
    async fn is_connected(&self) -> bool;
}

/// Opaque handle for a prepared statement.
#[derive(Debug, Clone)]
pub struct PreparedId {
    /// Internal identifier (implementation-specific).
    pub(crate) inner: Vec<u8>,
}

/// Tracing session data returned by the database.
#[derive(Debug, Clone)]
pub struct TracingSession {
    pub trace_id: uuid::Uuid,
    pub client: Option<String>,
    pub command: Option<String>,
    pub coordinator: Option<String>,
    pub duration: Option<i32>,
    pub parameters: HashMap<String, String>,
    pub request: Option<String>,
    pub started_at: Option<String>,
    pub events: Vec<TracingEvent>,
}

/// A single event within a tracing session.
#[derive(Debug, Clone)]
pub struct TracingEvent {
    pub activity: Option<String>,
    pub source: Option<String>,
    pub source_elapsed: Option<i32>,
    pub thread: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistency_from_str() {
        assert_eq!(
            Consistency::from_str_cql("QUORUM"),
            Some(Consistency::Quorum)
        );
        assert_eq!(
            Consistency::from_str_cql("local_quorum"),
            Some(Consistency::LocalQuorum)
        );
        assert_eq!(
            Consistency::from_str_cql("LOCAL_SERIAL"),
            Some(Consistency::LocalSerial)
        );
        assert_eq!(Consistency::from_str_cql("INVALID"), None);
    }

    #[test]
    fn consistency_display() {
        assert_eq!(Consistency::One.to_string(), "ONE");
        assert_eq!(Consistency::LocalQuorum.to_string(), "LOCAL_QUORUM");
    }

    #[test]
    fn consistency_roundtrip() {
        let levels = [
            Consistency::Any,
            Consistency::One,
            Consistency::Two,
            Consistency::Three,
            Consistency::Quorum,
            Consistency::All,
            Consistency::LocalQuorum,
            Consistency::EachQuorum,
            Consistency::Serial,
            Consistency::LocalSerial,
            Consistency::LocalOne,
        ];
        for level in &levels {
            let s = level.as_cql_str();
            assert_eq!(Consistency::from_str_cql(s), Some(*level));
        }
    }
}

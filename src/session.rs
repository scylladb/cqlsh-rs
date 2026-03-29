//! CQL session management layer.
//!
//! Wraps the driver with higher-level session state management including
//! keyspace tracking, consistency level management, and tracing control.
//! This mirrors the Python cqlsh `Shell` session state.

use anyhow::{bail, Result};

use crate::config::MergedConfig;
use crate::driver::types::CqlValue;
use crate::driver::{
    AggregateMetadata, ConnectionConfig, Consistency, CqlDriver, CqlResult, FunctionMetadata,
    KeyspaceMetadata, PreparedId, ScyllaDriver, SslConfig, TableMetadata, TracingSession,
    UdtMetadata,
};

/// High-level CQL session managing driver state and user preferences.
pub struct CqlSession {
    driver: ScyllaDriver,
    /// Current keyspace (updated on USE commands).
    current_keyspace: Option<String>,
    /// Display name for the connection (host:port).
    pub connection_display: String,
    /// Cluster name retrieved after connecting.
    pub cluster_name: Option<String>,
    /// CQL version from the connected node.
    pub cql_version: Option<String>,
    /// Release version of the connected node.
    pub release_version: Option<String>,
    /// ScyllaDB version (None if connected to Apache Cassandra).
    pub scylla_version: Option<String>,
}

impl CqlSession {
    /// Create a new session by connecting using the merged configuration.
    pub async fn connect(config: &MergedConfig) -> Result<Self> {
        let ssl_config = if config.ssl {
            Some(SslConfig {
                certfile: config.cqlshrc.ssl.certfile.clone(),
                validate: config.cqlshrc.ssl.validate.unwrap_or(false),
                userkey: config.cqlshrc.ssl.userkey.clone(),
                usercert: config.cqlshrc.ssl.usercert.clone(),
                host_certfiles: config.cqlshrc.certfiles.clone(),
            })
        } else {
            None
        };

        let conn_config = ConnectionConfig {
            host: config.host.clone(),
            port: config.port,
            username: config.username.clone(),
            password: config.password.clone(),
            keyspace: config.keyspace.clone(),
            connect_timeout: config.connect_timeout,
            request_timeout: config.request_timeout,
            ssl: config.ssl,
            ssl_config,
            protocol_version: config.protocol_version,
        };

        let driver = ScyllaDriver::connect(&conn_config).await?;

        let connection_display = format!("{}:{}", config.host, config.port);

        // Fetch cluster metadata after connecting
        let cluster_name = driver.get_cluster_name().await.ok().flatten();
        let cql_version = driver.get_cql_version().await.ok().flatten();
        let release_version = driver.get_release_version().await.ok().flatten();
        let scylla_version = driver.get_scylla_version().await.ok().flatten();

        // Set initial consistency from config
        if let Some(cl_str) = &config.consistency_level {
            if let Some(cl) = Consistency::from_str_cql(cl_str) {
                driver.set_consistency(cl);
            }
        }

        // Set initial serial consistency from config
        if let Some(scl_str) = &config.serial_consistency_level {
            if let Some(scl) = Consistency::from_str_cql(scl_str) {
                driver.set_serial_consistency(Some(scl));
            }
        }

        Ok(CqlSession {
            driver,
            current_keyspace: config.keyspace.clone(),
            connection_display,
            cluster_name,
            cql_version,
            release_version,
            scylla_version,
        })
    }

    /// Execute a CQL statement. Handles USE keyspace commands specially.
    pub async fn execute(&mut self, query: &str) -> Result<CqlResult> {
        let trimmed = query.trim();

        // Detect USE keyspace commands
        if let Some(keyspace) = parse_use_command(trimmed) {
            self.use_keyspace(&keyspace).await?;
            return Ok(CqlResult::empty());
        }

        self.driver.execute_unpaged(query).await
    }

    /// Execute a raw CQL query without USE interception.
    ///
    /// Used by DESCRIBE and other internal commands that need to query
    /// system tables directly.
    pub async fn execute_query(&self, query: &str) -> Result<CqlResult> {
        self.driver.execute_unpaged(query).await
    }

    /// Execute a CQL statement with paging.
    pub async fn execute_paged(&self, query: &str, page_size: i32) -> Result<CqlResult> {
        self.driver.execute_paged(query, page_size).await
    }

    /// Prepare a CQL statement.
    pub async fn prepare(&self, query: &str) -> Result<PreparedId> {
        self.driver.prepare(query).await
    }

    /// Execute a previously prepared statement with typed bound values.
    pub async fn execute_prepared(
        &self,
        id: &PreparedId,
        values: &[CqlValue],
    ) -> Result<CqlResult> {
        self.driver.execute_prepared(id, values).await
    }

    /// Switch to a different keyspace.
    pub async fn use_keyspace(&mut self, keyspace: &str) -> Result<()> {
        self.driver.use_keyspace(keyspace).await?;
        self.current_keyspace = Some(keyspace.to_string());
        Ok(())
    }

    /// Get the current keyspace.
    pub fn current_keyspace(&self) -> Option<&str> {
        self.current_keyspace.as_deref()
    }

    /// Get the current consistency level.
    pub fn get_consistency(&self) -> Consistency {
        self.driver.get_consistency()
    }

    /// Set the consistency level.
    pub fn set_consistency(&self, consistency: Consistency) {
        self.driver.set_consistency(consistency);
    }

    /// Set the consistency level from a string. Returns error if invalid.
    pub fn set_consistency_str(&self, level: &str) -> Result<()> {
        let consistency = Consistency::from_str_cql(level)
            .ok_or_else(|| anyhow::anyhow!("invalid consistency level: {level}"))?;
        self.driver.set_consistency(consistency);
        Ok(())
    }

    /// Get the current serial consistency level.
    pub fn get_serial_consistency(&self) -> Option<Consistency> {
        self.driver.get_serial_consistency()
    }

    /// Set the serial consistency level.
    pub fn set_serial_consistency(&self, consistency: Option<Consistency>) {
        self.driver.set_serial_consistency(consistency);
    }

    /// Set the serial consistency level from a string. Returns error if invalid.
    pub fn set_serial_consistency_str(&self, level: &str) -> Result<()> {
        let consistency = Consistency::from_str_cql(level)
            .ok_or_else(|| anyhow::anyhow!("invalid serial consistency level: {level}"))?;
        match consistency {
            Consistency::Serial | Consistency::LocalSerial => {
                self.driver.set_serial_consistency(Some(consistency));
                Ok(())
            }
            _ => bail!("serial consistency must be SERIAL or LOCAL_SERIAL, got {level}"),
        }
    }

    /// Enable or disable tracing.
    pub fn set_tracing(&self, enabled: bool) {
        self.driver.set_tracing(enabled);
    }

    /// Check if tracing is enabled.
    pub fn is_tracing_enabled(&self) -> bool {
        self.driver.is_tracing_enabled()
    }

    /// Get the last tracing session ID.
    pub fn last_trace_id(&self) -> Option<uuid::Uuid> {
        self.driver.last_trace_id()
    }

    /// Retrieve tracing session data.
    pub async fn get_trace_session(&self, trace_id: uuid::Uuid) -> Result<Option<TracingSession>> {
        self.driver.get_trace_session(trace_id).await
    }

    /// Get metadata for all keyspaces.
    pub async fn get_keyspaces(&self) -> Result<Vec<KeyspaceMetadata>> {
        self.driver.get_keyspaces().await
    }

    /// Get metadata for tables in a keyspace.
    pub async fn get_tables(&self, keyspace: &str) -> Result<Vec<TableMetadata>> {
        self.driver.get_tables(keyspace).await
    }

    /// Get metadata for a specific table.
    pub async fn get_table_metadata(
        &self,
        keyspace: &str,
        table: &str,
    ) -> Result<Option<TableMetadata>> {
        self.driver.get_table_metadata(keyspace, table).await
    }

    /// Get metadata for all user-defined types in a keyspace.
    pub async fn get_udts(&self, keyspace: &str) -> Result<Vec<UdtMetadata>> {
        self.driver.get_udts(keyspace).await
    }

    /// Get metadata for all user-defined functions in a keyspace.
    pub async fn get_functions(&self, keyspace: &str) -> Result<Vec<FunctionMetadata>> {
        self.driver.get_functions(keyspace).await
    }

    /// Get metadata for all user-defined aggregates in a keyspace.
    pub async fn get_aggregates(&self, keyspace: &str) -> Result<Vec<AggregateMetadata>> {
        self.driver.get_aggregates(keyspace).await
    }

    /// Check if the connection is still alive.
    pub async fn is_connected(&self) -> bool {
        self.driver.is_connected().await
    }
}

/// Parse a USE keyspace command, returning the keyspace name if matched.
fn parse_use_command(query: &str) -> Option<String> {
    let upper = query.to_uppercase();
    let trimmed = upper.trim().trim_end_matches(';').trim();

    if !trimmed.starts_with("USE ") {
        return None;
    }

    let keyspace = query
        .trim()
        .trim_end_matches(';')
        .trim()
        .strip_prefix("USE ")
        .or_else(|| {
            query
                .trim()
                .trim_end_matches(';')
                .trim()
                .strip_prefix("use ")
        })
        .map(|s| s.trim())?;

    // Remove quotes if present
    let keyspace = if (keyspace.starts_with('"') && keyspace.ends_with('"'))
        || (keyspace.starts_with('\'') && keyspace.ends_with('\''))
    {
        &keyspace[1..keyspace.len() - 1]
    } else {
        keyspace
    };

    if keyspace.is_empty() {
        None
    } else {
        Some(keyspace.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_use_simple() {
        assert_eq!(
            parse_use_command("USE my_keyspace"),
            Some("my_keyspace".to_string())
        );
    }

    #[test]
    fn parse_use_semicolon() {
        assert_eq!(
            parse_use_command("USE my_keyspace;"),
            Some("my_keyspace".to_string())
        );
    }

    #[test]
    fn parse_use_lowercase() {
        assert_eq!(
            parse_use_command("use test_ks"),
            Some("test_ks".to_string())
        );
    }

    #[test]
    fn parse_use_quoted() {
        assert_eq!(
            parse_use_command("USE \"MyKeyspace\""),
            Some("MyKeyspace".to_string())
        );
    }

    #[test]
    fn parse_use_single_quoted() {
        assert_eq!(parse_use_command("USE 'my_ks'"), Some("my_ks".to_string()));
    }

    #[test]
    fn parse_use_with_whitespace() {
        assert_eq!(
            parse_use_command("  USE  my_keyspace  ;  "),
            Some("my_keyspace".to_string())
        );
    }

    #[test]
    fn parse_not_use_command() {
        assert_eq!(parse_use_command("SELECT * FROM table"), None);
        assert_eq!(parse_use_command("INSERT INTO users"), None);
    }

    #[test]
    fn parse_use_empty() {
        assert_eq!(parse_use_command("USE "), None);
        assert_eq!(parse_use_command("USE ;"), None);
    }
}

//! ScyllaDriver — CqlDriver implementation using the `scylla` crate.
//!
//! Provides connectivity to Apache Cassandra and ScyllaDB clusters using
//! the scylla-rust-driver, with support for authentication, SSL/TLS,
//! prepared statements, paging, and schema metadata queries.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use futures::TryStreamExt;
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use scylla::response::query_result::QueryResult;
use scylla::statement::prepared::PreparedStatement;
use scylla::statement::Statement;
use scylla::value::{CqlValue as ScyllaCqlValue, Row};
use uuid::Uuid;

use super::types::{CqlColumn, CqlResult, CqlRow, CqlValue};
use super::{
    ColumnMetadata, ConnectionConfig, Consistency, CqlDriver, KeyspaceMetadata, PreparedId,
    SslConfig, TableMetadata, TracingEvent, TracingSession,
};

/// ScyllaDriver wraps a scylla `Session` and provides the `CqlDriver` trait.
pub struct ScyllaDriver {
    session: Session,
    /// Cache of prepared statements keyed by internal ID.
    prepared_cache: Mutex<HashMap<Vec<u8>, PreparedStatement>>,
    /// Current consistency level.
    consistency: Mutex<Consistency>,
    /// Current serial consistency level.
    serial_consistency: Mutex<Option<Consistency>>,
    /// Whether tracing is enabled for queries.
    tracing_enabled: AtomicBool,
    /// Last tracing session ID.
    last_trace_id: Mutex<Option<Uuid>>,
}

impl ScyllaDriver {
    /// Build the TLS configuration from SslConfig.
    fn build_rustls_config(ssl_config: &SslConfig) -> Result<Arc<rustls::ClientConfig>> {
        use rustls::pki_types::CertificateDer;
        use std::fs::File;
        use std::io::BufReader;

        let mut root_store = rustls::RootCertStore::empty();

        // Load CA certificate if provided
        if let Some(certfile) = &ssl_config.certfile {
            let file = File::open(certfile)
                .with_context(|| format!("opening CA certificate: {certfile}"))?;
            let mut reader = BufReader::new(file);
            let certs = rustls_pemfile::certs(&mut reader)
                .collect::<std::result::Result<Vec<_>, _>>()
                .with_context(|| format!("parsing CA certificate: {certfile}"))?;
            for cert in certs {
                root_store
                    .add(cert)
                    .context("adding CA certificate to root store")?;
            }
        }

        let builder = rustls::ClientConfig::builder().with_root_certificates(root_store);

        // Client certificate authentication (mutual TLS)
        let config = if let (Some(usercert_path), Some(userkey_path)) =
            (&ssl_config.usercert, &ssl_config.userkey)
        {
            let cert_file = File::open(usercert_path)
                .with_context(|| format!("opening client certificate: {usercert_path}"))?;
            let mut cert_reader = BufReader::new(cert_file);
            let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
                .collect::<std::result::Result<Vec<_>, _>>()
                .with_context(|| format!("parsing client certificate: {usercert_path}"))?;

            let key_file = File::open(userkey_path)
                .with_context(|| format!("opening client key: {userkey_path}"))?;
            let mut key_reader = BufReader::new(key_file);
            let key = rustls_pemfile::private_key(&mut key_reader)
                .with_context(|| format!("parsing client key: {userkey_path}"))?
                .ok_or_else(|| anyhow!("no private key found in {userkey_path}"))?;

            builder
                .with_client_auth_cert(certs, key)
                .context("configuring mutual TLS")?
        } else {
            builder.with_no_client_auth()
        };

        Ok(Arc::new(config))
    }

    /// Convert a scylla QueryResult into our CqlResult type.
    fn convert_query_result(result: QueryResult) -> Result<CqlResult> {
        let tracing_id = result.tracing_id();
        let warnings: Vec<String> = result.warnings().map(|s| s.to_string()).collect();

        // Check if this is a non-row result (DDL/DML)
        if !result.is_rows() {
            return Ok(CqlResult {
                columns: Vec::new(),
                rows: Vec::new(),
                has_rows: false,
                tracing_id,
                warnings,
            });
        }

        // Convert to QueryRowsResult to access typed rows
        let rows_result = result
            .into_rows_result()
            .context("converting query result to rows")?;

        // Extract column metadata
        let col_specs = rows_result.column_specs();
        let columns: Vec<CqlColumn> = col_specs
            .iter()
            .map(|spec| CqlColumn {
                name: spec.name().to_string(),
                type_name: format!("{:?}", spec.typ()),
            })
            .collect();

        // Deserialize rows as untyped Row (Vec<Option<CqlValue>>)
        let typed_rows = rows_result.rows::<Row>().context("deserializing rows")?;

        let mut cql_rows = Vec::new();
        for row_result in typed_rows {
            let row = row_result.context("deserializing row")?;
            let values: Vec<CqlValue> = row
                .columns
                .into_iter()
                .enumerate()
                .map(|(col_idx, opt_val)| match opt_val {
                    Some(v) => {
                        tracing::debug!(
                            column = col_idx,
                            variant = ?std::mem::discriminant(&v),
                            "converting ScyllaCqlValue: {v:?}"
                        );
                        Self::convert_scylla_value(v)
                    }
                    None => {
                        tracing::debug!(column = col_idx, "column value is None (null)");
                        CqlValue::Null
                    }
                })
                .collect();
            cql_rows.push(CqlRow { values });
        }

        Ok(CqlResult {
            columns,
            rows: cql_rows,
            has_rows: true,
            tracing_id,
            warnings,
        })
    }

    /// Convert a scylla CqlValue to our CqlValue type.
    fn convert_scylla_value(value: ScyllaCqlValue) -> CqlValue {
        match value {
            ScyllaCqlValue::Ascii(s) => CqlValue::Ascii(s),
            ScyllaCqlValue::Boolean(b) => CqlValue::Boolean(b),
            ScyllaCqlValue::Blob(bytes) => CqlValue::Blob(bytes),
            ScyllaCqlValue::Counter(c) => CqlValue::Counter(c.0),
            ScyllaCqlValue::Decimal(d) => {
                let (int_val, scale) = d.as_signed_be_bytes_slice_and_exponent();
                let big_int = num_bigint::BigInt::from_signed_bytes_be(int_val);
                CqlValue::Decimal(bigdecimal::BigDecimal::new(big_int, scale.into()))
            }
            ScyllaCqlValue::Date(d) => {
                // scylla CqlDate wraps u32 days since epoch center (2^31)
                let days = d.0;
                let epoch_offset = days as i64 - (1i64 << 31);
                match chrono::NaiveDate::from_num_days_from_ce_opt((epoch_offset + 719_163) as i32)
                {
                    Some(date) => CqlValue::Date(date),
                    None => CqlValue::Text(format!("<invalid date: {days}>")),
                }
            }
            ScyllaCqlValue::Double(d) => CqlValue::Double(d),
            ScyllaCqlValue::Duration(d) => CqlValue::Duration {
                months: d.months,
                days: d.days,
                nanoseconds: d.nanoseconds,
            },
            ScyllaCqlValue::Empty => CqlValue::Null,
            ScyllaCqlValue::Float(f) => CqlValue::Float(f),
            ScyllaCqlValue::Int(i) => CqlValue::Int(i),
            ScyllaCqlValue::BigInt(i) => CqlValue::BigInt(i),
            ScyllaCqlValue::Text(s) => CqlValue::Text(s),
            ScyllaCqlValue::Timestamp(t) => CqlValue::Timestamp(t.0),
            ScyllaCqlValue::Inet(addr) => CqlValue::Inet(addr),
            ScyllaCqlValue::List(items) => {
                CqlValue::List(items.into_iter().map(Self::convert_scylla_value).collect())
            }
            ScyllaCqlValue::Map(entries) => CqlValue::Map(
                entries
                    .into_iter()
                    .map(|(k, v)| (Self::convert_scylla_value(k), Self::convert_scylla_value(v)))
                    .collect(),
            ),
            ScyllaCqlValue::Set(items) => {
                CqlValue::Set(items.into_iter().map(Self::convert_scylla_value).collect())
            }
            ScyllaCqlValue::UserDefinedType {
                keyspace,
                name,
                fields,
            } => CqlValue::UserDefinedType {
                keyspace,
                type_name: name,
                fields: fields
                    .into_iter()
                    .map(|(n, val)| (n, val.map(Self::convert_scylla_value)))
                    .collect(),
            },
            ScyllaCqlValue::SmallInt(i) => CqlValue::SmallInt(i),
            ScyllaCqlValue::TinyInt(i) => CqlValue::TinyInt(i),
            ScyllaCqlValue::Time(t) => {
                let nanos = t.0;
                let secs = (nanos / 1_000_000_000) as u32;
                let nano_part = (nanos % 1_000_000_000) as u32;
                match chrono::NaiveTime::from_num_seconds_from_midnight_opt(secs, nano_part) {
                    Some(time) => CqlValue::Time(time),
                    None => CqlValue::Text(format!("<invalid time: {nanos}>")),
                }
            }
            ScyllaCqlValue::Timeuuid(u) => CqlValue::TimeUuid(u.into()),
            ScyllaCqlValue::Tuple(items) => CqlValue::Tuple(
                items
                    .into_iter()
                    .map(|v| v.map(Self::convert_scylla_value))
                    .collect(),
            ),
            ScyllaCqlValue::Uuid(u) => CqlValue::Uuid(u),
            ScyllaCqlValue::Varint(v) => {
                let big_int =
                    num_bigint::BigInt::from_signed_bytes_be(v.as_signed_bytes_be_slice());
                CqlValue::Varint(big_int)
            }
            // CqlValue is non-exhaustive; handle future variants gracefully
            _ => {
                tracing::warn!("unhandled ScyllaCqlValue variant: {value:?}");
                CqlValue::Text(format!("{value:?}"))
            }
        }
    }

    /// Convert our Consistency to scylla's Consistency.
    fn to_scylla_consistency(c: Consistency) -> scylla::statement::Consistency {
        use scylla::statement::Consistency as SC;
        match c {
            Consistency::Any => SC::Any,
            Consistency::One => SC::One,
            Consistency::Two => SC::Two,
            Consistency::Three => SC::Three,
            Consistency::Quorum => SC::Quorum,
            Consistency::All => SC::All,
            Consistency::LocalQuorum => SC::LocalQuorum,
            Consistency::EachQuorum => SC::EachQuorum,
            Consistency::Serial => SC::Serial,
            Consistency::LocalSerial => SC::LocalSerial,
            Consistency::LocalOne => SC::LocalOne,
        }
    }

    /// Convert our Consistency to scylla's SerialConsistency.
    fn to_scylla_serial_consistency(
        c: Consistency,
    ) -> Option<scylla::statement::SerialConsistency> {
        use scylla::statement::SerialConsistency as SC;
        match c {
            Consistency::Serial => Some(SC::Serial),
            Consistency::LocalSerial => Some(SC::LocalSerial),
            _ => None,
        }
    }

    /// Build a Statement with the current consistency and tracing settings.
    fn build_query(&self, cql: &str) -> Statement {
        let mut stmt = Statement::new(cql);

        let consistency = *self.consistency.lock().unwrap();
        stmt.set_consistency(Self::to_scylla_consistency(consistency));

        let serial = *self.serial_consistency.lock().unwrap();
        if let Some(sc) = serial {
            if let Some(sc) = Self::to_scylla_serial_consistency(sc) {
                stmt.set_serial_consistency(Some(sc));
            }
        }

        if self.tracing_enabled.load(Ordering::Relaxed) {
            stmt.set_tracing(true);
        }

        stmt
    }

    /// Store tracing ID from a result if present.
    fn store_trace_id(&self, result: &QueryResult) {
        if let Some(trace_id) = result.tracing_id() {
            *self.last_trace_id.lock().unwrap() = Some(trace_id);
        }
    }
}

#[async_trait]
impl CqlDriver for ScyllaDriver {
    async fn connect(config: &ConnectionConfig) -> Result<Self> {
        let addr = format!("{}:{}", config.host, config.port);

        let mut builder = SessionBuilder::new().known_node(&addr);

        // Authentication
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            builder = builder.user(username, password);
        }

        // Connection timeout
        builder = builder.connection_timeout(Duration::from_secs(config.connect_timeout));

        // Default keyspace
        if let Some(keyspace) = &config.keyspace {
            builder = builder.use_keyspace(keyspace, false);
        }

        // SSL/TLS
        if config.ssl {
            let tls_config = if let Some(ssl_config) = &config.ssl_config {
                Self::build_rustls_config(ssl_config)?
            } else {
                // SSL enabled but no config — use default (no validation)
                let root_store = rustls::RootCertStore::empty();
                Arc::new(
                    rustls::ClientConfig::builder()
                        .with_root_certificates(root_store)
                        .with_no_client_auth(),
                )
            };
            builder = builder.tls_context(Some(tls_config));
        }

        let session = builder.build().await.context("connecting to cluster")?;

        Ok(ScyllaDriver {
            session,
            prepared_cache: Mutex::new(HashMap::new()),
            consistency: Mutex::new(Consistency::One),
            serial_consistency: Mutex::new(None),
            tracing_enabled: AtomicBool::new(false),
            last_trace_id: Mutex::new(None),
        })
    }

    async fn execute_unpaged(&self, query: &str) -> Result<CqlResult> {
        let stmt = self.build_query(query);

        let result = self.session.query_unpaged(stmt, ()).await?;

        self.store_trace_id(&result);
        Self::convert_query_result(result)
    }

    async fn execute_paged(&self, query: &str, page_size: i32) -> Result<CqlResult> {
        let mut stmt = self.build_query(query);
        stmt.set_page_size(page_size);

        let query_pager = self
            .session
            .query_iter(stmt, ())
            .await
            .context("starting paged query")?;

        // Get column metadata from the pager
        let col_specs = query_pager.column_specs();
        let columns: Vec<CqlColumn> = col_specs
            .iter()
            .map(|spec| CqlColumn {
                name: spec.name().to_string(),
                type_name: format!("{:?}", spec.typ()),
            })
            .collect();

        // Stream all rows using the untyped Row type
        let mut rows_stream = query_pager.rows_stream::<Row>()?;
        let mut cql_rows = Vec::new();

        while let Some(row) = rows_stream.try_next().await? {
            let values: Vec<CqlValue> = row
                .columns
                .into_iter()
                .map(|opt_val| match opt_val {
                    Some(v) => Self::convert_scylla_value(v),
                    None => CqlValue::Null,
                })
                .collect();
            cql_rows.push(CqlRow { values });
        }

        Ok(CqlResult {
            columns,
            rows: cql_rows,
            has_rows: true,
            tracing_id: None,
            warnings: Vec::new(),
        })
    }

    async fn prepare(&self, query: &str) -> Result<PreparedId> {
        let prepared = self
            .session
            .prepare(query)
            .await
            .context("preparing CQL statement")?;

        let id = prepared.get_id().to_vec();
        self.prepared_cache
            .lock()
            .unwrap()
            .insert(id.clone(), prepared);

        Ok(PreparedId { inner: id })
    }

    async fn execute_prepared(
        &self,
        prepared_id: &PreparedId,
        _values: &[CqlValue],
    ) -> Result<CqlResult> {
        let prepared = self
            .prepared_cache
            .lock()
            .unwrap()
            .get(&prepared_id.inner)
            .cloned()
            .ok_or_else(|| anyhow!("prepared statement not found in cache"))?;

        let result = self
            .session
            .execute_unpaged(&prepared, ())
            .await
            .context("executing prepared statement")?;

        self.store_trace_id(&result);
        Self::convert_query_result(result)
    }

    async fn use_keyspace(&self, keyspace: &str) -> Result<()> {
        self.session
            .use_keyspace(keyspace, false)
            .await
            .with_context(|| format!("switching to keyspace: {keyspace}"))?;
        Ok(())
    }

    fn get_consistency(&self) -> Consistency {
        *self.consistency.lock().unwrap()
    }

    fn set_consistency(&self, consistency: Consistency) {
        *self.consistency.lock().unwrap() = consistency;
    }

    fn get_serial_consistency(&self) -> Option<Consistency> {
        *self.serial_consistency.lock().unwrap()
    }

    fn set_serial_consistency(&self, consistency: Option<Consistency>) {
        *self.serial_consistency.lock().unwrap() = consistency;
    }

    fn set_tracing(&self, enabled: bool) {
        self.tracing_enabled.store(enabled, Ordering::Relaxed);
    }

    fn is_tracing_enabled(&self) -> bool {
        self.tracing_enabled.load(Ordering::Relaxed)
    }

    fn last_trace_id(&self) -> Option<Uuid> {
        *self.last_trace_id.lock().unwrap()
    }

    async fn get_trace_session(&self, trace_id: Uuid) -> Result<Option<TracingSession>> {
        let query = format!(
            "SELECT client, command, coordinator, duration, parameters, request, started_at \
             FROM system_traces.sessions WHERE session_id = {}",
            trace_id
        );
        let result = self.execute_unpaged(&query).await?;

        if result.rows.is_empty() {
            return Ok(None);
        }

        let events_query = format!(
            "SELECT activity, source, source_elapsed, thread \
             FROM system_traces.events WHERE session_id = {}",
            trace_id
        );
        let events_result = self.execute_unpaged(&events_query).await?;

        let events: Vec<TracingEvent> = events_result
            .rows
            .iter()
            .map(|row| TracingEvent {
                activity: row.get(0).and_then(cql_value_to_string),
                source: row.get(1).and_then(cql_value_to_string),
                source_elapsed: row.get(2).and_then(cql_value_to_i32),
                thread: row.get(3).and_then(cql_value_to_string),
            })
            .collect();

        let session_row = &result.rows[0];
        Ok(Some(TracingSession {
            trace_id,
            client: session_row.get(0).and_then(cql_value_to_string),
            command: session_row.get(1).and_then(cql_value_to_string),
            coordinator: session_row.get(2).and_then(cql_value_to_string),
            duration: session_row.get(3).and_then(cql_value_to_i32),
            parameters: HashMap::new(),
            request: session_row.get(5).and_then(cql_value_to_string),
            started_at: session_row.get(6).and_then(cql_value_to_string),
            events,
        }))
    }

    async fn get_keyspaces(&self) -> Result<Vec<KeyspaceMetadata>> {
        let result = self
            .execute_unpaged(
                "SELECT keyspace_name, replication, durable_writes \
                 FROM system_schema.keyspaces",
            )
            .await?;

        let mut keyspaces = Vec::new();
        for row in &result.rows {
            let name = row.get(0).and_then(cql_value_to_string).unwrap_or_default();
            let durable_writes = match row.get(2) {
                Some(CqlValue::Boolean(b)) => *b,
                _ => true,
            };

            keyspaces.push(KeyspaceMetadata {
                name,
                replication: HashMap::new(),
                durable_writes,
            });
        }

        Ok(keyspaces)
    }

    async fn get_tables(&self, keyspace: &str) -> Result<Vec<TableMetadata>> {
        let result = self
            .execute_unpaged(&format!(
                "SELECT table_name FROM system_schema.tables WHERE keyspace_name = '{}'",
                keyspace.replace('\'', "''")
            ))
            .await?;

        let mut tables = Vec::new();
        for row in &result.rows {
            let table_name = row.get(0).and_then(cql_value_to_string).unwrap_or_default();

            let col_result = self
                .execute_unpaged(&format!(
                    "SELECT column_name, type, kind \
                     FROM system_schema.columns \
                     WHERE keyspace_name = '{}' AND table_name = '{}'",
                    keyspace.replace('\'', "''"),
                    table_name.replace('\'', "''")
                ))
                .await?;

            let mut columns = Vec::new();
            let mut partition_key = Vec::new();
            let mut clustering_key = Vec::new();

            for col_row in &col_result.rows {
                let col_name = col_row
                    .get(0)
                    .and_then(cql_value_to_string)
                    .unwrap_or_default();
                let col_type = col_row
                    .get(1)
                    .and_then(cql_value_to_string)
                    .unwrap_or_default();
                let kind = col_row
                    .get(2)
                    .and_then(cql_value_to_string)
                    .unwrap_or_default();

                columns.push(ColumnMetadata {
                    name: col_name.clone(),
                    type_name: col_type,
                });

                match kind.as_str() {
                    "partition_key" => partition_key.push(col_name),
                    "clustering" => clustering_key.push(col_name),
                    _ => {}
                }
            }

            tables.push(TableMetadata {
                keyspace: keyspace.to_string(),
                name: table_name,
                columns,
                partition_key,
                clustering_key,
            });
        }

        Ok(tables)
    }

    async fn get_table_metadata(
        &self,
        keyspace: &str,
        table: &str,
    ) -> Result<Option<TableMetadata>> {
        let tables = self.get_tables(keyspace).await?;
        Ok(tables.into_iter().find(|t| t.name == table))
    }

    async fn get_cluster_name(&self) -> Result<Option<String>> {
        let result = self
            .execute_unpaged("SELECT cluster_name FROM system.local")
            .await?;
        Ok(result
            .rows
            .first()
            .and_then(|row| row.get(0))
            .and_then(cql_value_to_string))
    }

    async fn get_cql_version(&self) -> Result<Option<String>> {
        let result = self
            .execute_unpaged("SELECT cql_version FROM system.local")
            .await?;
        Ok(result
            .rows
            .first()
            .and_then(|row| row.get(0))
            .and_then(cql_value_to_string))
    }

    async fn get_release_version(&self) -> Result<Option<String>> {
        let result = self
            .execute_unpaged("SELECT release_version FROM system.local")
            .await?;
        Ok(result
            .rows
            .first()
            .and_then(|row| row.get(0))
            .and_then(cql_value_to_string))
    }

    async fn is_connected(&self) -> bool {
        self.execute_unpaged("SELECT key FROM system.local LIMIT 1")
            .await
            .is_ok()
    }
}

/// Helper: extract a string from a CqlValue.
fn cql_value_to_string(v: &CqlValue) -> Option<String> {
    match v {
        CqlValue::Text(s) | CqlValue::Ascii(s) => Some(s.clone()),
        CqlValue::Inet(addr) => Some(addr.to_string()),
        CqlValue::Null => None,
        other => Some(other.to_string()),
    }
}

/// Helper: extract an i32 from a CqlValue.
fn cql_value_to_i32(v: &CqlValue) -> Option<i32> {
    match v {
        CqlValue::Int(i) => Some(*i),
        CqlValue::BigInt(i) => Some(*i as i32),
        CqlValue::SmallInt(i) => Some(*i as i32),
        CqlValue::TinyInt(i) => Some(*i as i32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_scylla_value_text() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Text("hello".to_string()));
        assert_eq!(v, CqlValue::Text("hello".to_string()));
    }

    #[test]
    fn convert_scylla_value_int() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Int(42));
        assert_eq!(v, CqlValue::Int(42));
    }

    #[test]
    fn convert_scylla_value_boolean() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Boolean(true));
        assert_eq!(v, CqlValue::Boolean(true));
    }

    #[test]
    fn convert_scylla_value_null() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Empty);
        assert_eq!(v, CqlValue::Null);
    }

    #[test]
    fn convert_scylla_value_list() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::List(vec![
            ScyllaCqlValue::Int(1),
            ScyllaCqlValue::Int(2),
        ]));
        assert_eq!(v, CqlValue::List(vec![CqlValue::Int(1), CqlValue::Int(2)]));
    }

    #[test]
    fn convert_scylla_value_uuid() {
        let id = Uuid::nil();
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Uuid(id));
        assert_eq!(v, CqlValue::Uuid(id));
    }

    #[test]
    fn convert_scylla_value_blob() {
        let v =
            ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Blob(vec![0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(v, CqlValue::Blob(vec![0xde, 0xad, 0xbe, 0xef]));
    }

    #[test]
    fn convert_scylla_value_float() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Float(1.5));
        assert_eq!(v, CqlValue::Float(1.5));
    }

    #[test]
    fn convert_scylla_value_double() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Double(1.5));
        assert_eq!(v, CqlValue::Double(1.5));
    }

    #[test]
    fn convert_scylla_value_map() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Map(vec![(
            ScyllaCqlValue::Text("key".to_string()),
            ScyllaCqlValue::Int(42),
        )]));
        assert_eq!(
            v,
            CqlValue::Map(vec![(CqlValue::Text("key".to_string()), CqlValue::Int(42))])
        );
    }

    #[test]
    fn convert_scylla_value_set() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::Set(vec![
            ScyllaCqlValue::Int(1),
            ScyllaCqlValue::Int(2),
        ]));
        assert_eq!(v, CqlValue::Set(vec![CqlValue::Int(1), CqlValue::Int(2)]));
    }

    #[test]
    fn convert_scylla_value_udt() {
        let v = ScyllaDriver::convert_scylla_value(ScyllaCqlValue::UserDefinedType {
            keyspace: "ks".to_string(),
            name: "my_type".to_string(),
            fields: vec![
                ("f1".to_string(), Some(ScyllaCqlValue::Int(1))),
                ("f2".to_string(), None),
            ],
        });
        assert_eq!(
            v,
            CqlValue::UserDefinedType {
                keyspace: "ks".to_string(),
                type_name: "my_type".to_string(),
                fields: vec![
                    ("f1".to_string(), Some(CqlValue::Int(1))),
                    ("f2".to_string(), None),
                ],
            }
        );
    }

    #[test]
    fn to_scylla_consistency_mapping() {
        use scylla::statement::Consistency as SC;
        assert!(matches!(
            ScyllaDriver::to_scylla_consistency(Consistency::One),
            SC::One
        ));
        assert!(matches!(
            ScyllaDriver::to_scylla_consistency(Consistency::Quorum),
            SC::Quorum
        ));
        assert!(matches!(
            ScyllaDriver::to_scylla_consistency(Consistency::LocalQuorum),
            SC::LocalQuorum
        ));
        assert!(matches!(
            ScyllaDriver::to_scylla_consistency(Consistency::All),
            SC::All
        ));
    }

    #[test]
    fn to_scylla_serial_consistency_mapping() {
        use scylla::statement::SerialConsistency as SC;
        assert!(matches!(
            ScyllaDriver::to_scylla_serial_consistency(Consistency::Serial),
            Some(SC::Serial)
        ));
        assert!(matches!(
            ScyllaDriver::to_scylla_serial_consistency(Consistency::LocalSerial),
            Some(SC::LocalSerial)
        ));
        assert!(ScyllaDriver::to_scylla_serial_consistency(Consistency::One).is_none());
    }

    #[test]
    fn cql_value_to_string_helper() {
        assert_eq!(
            cql_value_to_string(&CqlValue::Text("hello".to_string())),
            Some("hello".to_string())
        );
        assert_eq!(
            cql_value_to_string(&CqlValue::Int(42)),
            Some("42".to_string())
        );
        assert_eq!(cql_value_to_string(&CqlValue::Null), None);
    }

    #[test]
    fn cql_value_to_i32_helper() {
        assert_eq!(cql_value_to_i32(&CqlValue::Int(42)), Some(42));
        assert_eq!(cql_value_to_i32(&CqlValue::BigInt(100)), Some(100));
        assert_eq!(cql_value_to_i32(&CqlValue::Text("x".to_string())), None);
    }
}

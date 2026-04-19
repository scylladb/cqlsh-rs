//! Schema metadata cache for tab completion.
//!
//! Provides a `SchemaCache` struct that caches CQL schema metadata (keyspaces,
//! tables, columns, UDTs, functions, aggregates) fetched from the database.
//! The cache supports TTL-based staleness checks so it refreshes periodically.
//!
//! The REPL wraps `SchemaCache` in an `Arc<RwLock<>>` for shared async access.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::driver::{
    AggregateMetadata, FunctionMetadata, KeyspaceMetadata, TableMetadata, UdtMetadata,
};
use crate::session::CqlSession;

/// Default TTL for cached schema data (30 seconds).
const DEFAULT_TTL: Duration = Duration::from_secs(30);

/// Cached schema metadata for a CQL cluster.
///
/// Stores keyspaces, tables, UDTs, functions, and aggregates fetched from the
/// cluster. Lookup methods are synchronous — callers must call `refresh()` to
/// populate or update the cache.
pub struct SchemaCache {
    /// All keyspaces in the cluster.
    keyspaces: Vec<KeyspaceMetadata>,
    /// Tables indexed by keyspace name.
    tables: HashMap<String, Vec<TableMetadata>>,
    /// UDTs indexed by keyspace name.
    udts: HashMap<String, Vec<UdtMetadata>>,
    /// Functions indexed by keyspace name.
    functions: HashMap<String, Vec<FunctionMetadata>>,
    /// Aggregates indexed by keyspace name.
    aggregates: HashMap<String, Vec<AggregateMetadata>>,
    /// When the cache was last successfully refreshed.
    last_refresh: Option<Instant>,
    /// How long before the cache is considered stale.
    ttl: Duration,
}

impl SchemaCache {
    /// Create a new, empty cache with the default TTL of 30 seconds.
    pub fn new() -> Self {
        Self::with_ttl(DEFAULT_TTL)
    }

    /// Create a new, empty cache with a custom TTL.
    pub fn with_ttl(ttl: Duration) -> Self {
        SchemaCache {
            keyspaces: Vec::new(),
            tables: HashMap::new(),
            udts: HashMap::new(),
            functions: HashMap::new(),
            aggregates: HashMap::new(),
            last_refresh: None,
            ttl,
        }
    }

    /// Returns `true` if the cache has never been refreshed or its TTL has elapsed.
    pub fn is_stale(&self) -> bool {
        match self.last_refresh {
            None => true,
            Some(refreshed_at) => refreshed_at.elapsed() >= self.ttl,
        }
    }

    /// Force the cache to appear stale so the next access triggers a refresh.
    pub fn invalidate(&mut self) {
        self.last_refresh = None;
    }

    /// Refresh the cache by fetching all schema metadata from the cluster.
    ///
    /// Fetches all keyspaces first, then tables, UDTs, functions, and
    /// aggregates for each keyspace in parallel (sequentially per keyspace for
    /// simplicity — a future optimisation could use `join_all`).
    pub async fn refresh(&mut self, session: &CqlSession) -> Result<()> {
        let keyspaces = session.get_keyspaces().await?;

        let mut tables: HashMap<String, Vec<TableMetadata>> = HashMap::new();
        let mut udts: HashMap<String, Vec<UdtMetadata>> = HashMap::new();
        let mut functions: HashMap<String, Vec<FunctionMetadata>> = HashMap::new();
        let mut aggregates: HashMap<String, Vec<AggregateMetadata>> = HashMap::new();

        for ks in &keyspaces {
            let ks_name = ks.name.as_str();

            // Ignore errors for individual keyspaces — best-effort population.
            if let Ok(t) = session.get_tables(ks_name).await {
                tables.insert(ks_name.to_string(), t);
            }
            if let Ok(u) = session.get_udts(ks_name).await {
                udts.insert(ks_name.to_string(), u);
            }
            if let Ok(f) = session.get_functions(ks_name).await {
                functions.insert(ks_name.to_string(), f);
            }
            if let Ok(a) = session.get_aggregates(ks_name).await {
                aggregates.insert(ks_name.to_string(), a);
            }
        }

        self.keyspaces = keyspaces;
        self.tables = tables;
        self.udts = udts;
        self.functions = functions;
        self.aggregates = aggregates;
        self.last_refresh = Some(Instant::now());

        Ok(())
    }

    // ── Lookup helpers ────────────────────────────────────────────────────────

    /// Return the names of all cached keyspaces.
    pub fn keyspace_names(&self) -> Vec<&str> {
        self.keyspaces.iter().map(|ks| ks.name.as_str()).collect()
    }

    /// Return the names of all tables in `keyspace`.
    ///
    /// Returns an empty `Vec` if the keyspace is unknown.
    pub fn table_names(&self, keyspace: &str) -> Vec<&str> {
        self.tables
            .get(keyspace)
            .map(|tables| tables.iter().map(|t| t.name.as_str()).collect())
            .unwrap_or_default()
    }

    /// Return the names of all columns in `keyspace.table`.
    ///
    /// Returns an empty `Vec` if the keyspace or table is unknown.
    pub fn column_names(&self, keyspace: &str, table: &str) -> Vec<&str> {
        self.tables
            .get(keyspace)
            .and_then(|tables| tables.iter().find(|t| t.name == table))
            .map(|t| t.columns.iter().map(|c| c.name.as_str()).collect())
            .unwrap_or_default()
    }

    /// Return the names of all UDTs in `keyspace`.
    ///
    /// Returns an empty `Vec` if the keyspace is unknown.
    pub fn udt_names(&self, keyspace: &str) -> Vec<&str> {
        self.udts
            .get(keyspace)
            .map(|udts| udts.iter().map(|u| u.name.as_str()).collect())
            .unwrap_or_default()
    }

    /// Return the names of all functions in `keyspace`.
    ///
    /// Returns an empty `Vec` if the keyspace is unknown.
    pub fn function_names(&self, keyspace: &str) -> Vec<&str> {
        self.functions
            .get(keyspace)
            .map(|fns| fns.iter().map(|f| f.name.as_str()).collect())
            .unwrap_or_default()
    }

    /// Return the names of all aggregates in `keyspace`.
    ///
    /// Returns an empty `Vec` if the keyspace is unknown.
    pub fn aggregate_names(&self, keyspace: &str) -> Vec<&str> {
        self.aggregates
            .get(keyspace)
            .map(|aggs| aggs.iter().map(|a| a.name.as_str()).collect())
            .unwrap_or_default()
    }

    /// Build a pre-populated cache for testing and benchmarking.
    ///
    /// Not part of the public API — may change without notice.
    #[doc(hidden)]
    pub fn from_test_data(
        keyspaces: Vec<crate::driver::KeyspaceMetadata>,
        tables: std::collections::HashMap<String, Vec<crate::driver::TableMetadata>>,
    ) -> Self {
        SchemaCache {
            keyspaces,
            tables,
            udts: Default::default(),
            functions: Default::default(),
            aggregates: Default::default(),
            last_refresh: Some(std::time::Instant::now()),
            ttl: DEFAULT_TTL,
        }
    }
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{ColumnMetadata, KeyspaceMetadata, TableMetadata, UdtMetadata};

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_keyspace(name: &str) -> KeyspaceMetadata {
        KeyspaceMetadata {
            name: name.to_string(),
            replication: HashMap::new(),
            durable_writes: true,
        }
    }

    fn make_table(keyspace: &str, name: &str, columns: &[(&str, &str)]) -> TableMetadata {
        TableMetadata {
            keyspace: keyspace.to_string(),
            name: name.to_string(),
            columns: columns
                .iter()
                .map(|(col_name, col_type)| ColumnMetadata {
                    name: col_name.to_string(),
                    type_name: col_type.to_string(),
                })
                .collect(),
            partition_key: vec![],
            clustering_key: vec![],
        }
    }

    fn make_udt(keyspace: &str, name: &str) -> UdtMetadata {
        UdtMetadata {
            keyspace: keyspace.to_string(),
            name: name.to_string(),
            field_names: vec!["street".to_string()],
            field_types: vec!["text".to_string()],
        }
    }

    fn make_function(keyspace: &str, name: &str) -> FunctionMetadata {
        FunctionMetadata {
            keyspace: keyspace.to_string(),
            name: name.to_string(),
            argument_types: vec![],
            return_type: "text".to_string(),
        }
    }

    fn make_aggregate(keyspace: &str, name: &str) -> AggregateMetadata {
        AggregateMetadata {
            keyspace: keyspace.to_string(),
            name: name.to_string(),
            argument_types: vec![],
            return_type: "int".to_string(),
        }
    }

    /// Build a pre-populated `SchemaCache` without going through `refresh()`.
    fn populated_cache() -> SchemaCache {
        let mut cache = SchemaCache::new();

        cache.keyspaces = vec![make_keyspace("ks1"), make_keyspace("ks2")];

        cache.tables.insert(
            "ks1".to_string(),
            vec![
                make_table("ks1", "users", &[("id", "uuid"), ("name", "text")]),
                make_table(
                    "ks1",
                    "orders",
                    &[("order_id", "uuid"), ("total", "decimal")],
                ),
            ],
        );
        cache.tables.insert(
            "ks2".to_string(),
            vec![make_table("ks2", "events", &[("event_id", "uuid")])],
        );

        cache
            .udts
            .insert("ks1".to_string(), vec![make_udt("ks1", "address")]);

        cache
            .functions
            .insert("ks1".to_string(), vec![make_function("ks1", "my_func")]);

        cache
            .aggregates
            .insert("ks1".to_string(), vec![make_aggregate("ks1", "my_agg")]);

        // Mark as freshly refreshed so staleness tests work correctly.
        cache.last_refresh = Some(Instant::now());

        cache
    }

    // ── Constructor tests ─────────────────────────────────────────────────────

    #[test]
    fn new_cache_is_empty_and_stale() {
        let cache = SchemaCache::new();
        assert!(
            cache.is_stale(),
            "fresh cache should be stale (never refreshed)"
        );
        assert!(cache.keyspace_names().is_empty());
    }

    #[test]
    fn default_ttl_is_thirty_seconds() {
        let cache = SchemaCache::new();
        assert_eq!(cache.ttl, Duration::from_secs(30));
    }

    #[test]
    fn with_ttl_stores_custom_ttl() {
        let cache = SchemaCache::with_ttl(Duration::from_secs(60));
        assert_eq!(cache.ttl, Duration::from_secs(60));
    }

    #[test]
    fn default_impl_equals_new() {
        let a = SchemaCache::new();
        let b = SchemaCache::default();
        assert_eq!(a.ttl, b.ttl);
        assert!(a.keyspace_names().is_empty());
        assert!(b.keyspace_names().is_empty());
    }

    // ── Staleness / TTL tests ─────────────────────────────────────────────────

    #[test]
    fn freshly_refreshed_cache_is_not_stale() {
        let cache = populated_cache();
        assert!(!cache.is_stale());
    }

    #[test]
    fn expired_cache_is_stale() {
        let mut cache = SchemaCache::with_ttl(Duration::from_millis(1));
        // Simulate a refresh that happened long enough ago.
        cache.last_refresh = Some(Instant::now() - Duration::from_millis(10));
        assert!(cache.is_stale());
    }

    #[test]
    fn invalidate_marks_cache_stale() {
        let mut cache = populated_cache();
        assert!(!cache.is_stale());
        cache.invalidate();
        assert!(cache.is_stale());
    }

    #[test]
    fn invalidate_preserves_cached_data() {
        let mut cache = populated_cache();
        cache.invalidate();
        // Data is still present even though the cache is stale.
        assert!(!cache.keyspace_names().is_empty());
        assert!(!cache.table_names("ks1").is_empty());
    }

    // ── keyspace_names tests ──────────────────────────────────────────────────

    #[test]
    fn keyspace_names_returns_all_keyspaces() {
        let cache = populated_cache();
        let mut names = cache.keyspace_names();
        names.sort();
        assert_eq!(names, vec!["ks1", "ks2"]);
    }

    #[test]
    fn keyspace_names_empty_when_no_data() {
        let cache = SchemaCache::new();
        assert!(cache.keyspace_names().is_empty());
    }

    // ── table_names tests ─────────────────────────────────────────────────────

    #[test]
    fn table_names_returns_tables_for_keyspace() {
        let cache = populated_cache();
        let mut tables = cache.table_names("ks1");
        tables.sort();
        assert_eq!(tables, vec!["orders", "users"]);
    }

    #[test]
    fn table_names_empty_for_unknown_keyspace() {
        let cache = populated_cache();
        assert!(cache.table_names("nonexistent").is_empty());
    }

    #[test]
    fn table_names_single_table_keyspace() {
        let cache = populated_cache();
        assert_eq!(cache.table_names("ks2"), vec!["events"]);
    }

    // ── column_names tests ────────────────────────────────────────────────────

    #[test]
    fn column_names_returns_columns_for_table() {
        let cache = populated_cache();
        let mut cols = cache.column_names("ks1", "users");
        cols.sort();
        assert_eq!(cols, vec!["id", "name"]);
    }

    #[test]
    fn column_names_empty_for_unknown_table() {
        let cache = populated_cache();
        assert!(cache.column_names("ks1", "nonexistent").is_empty());
    }

    #[test]
    fn column_names_empty_for_unknown_keyspace() {
        let cache = populated_cache();
        assert!(cache.column_names("nonexistent", "users").is_empty());
    }

    #[test]
    fn column_names_orders_table() {
        let cache = populated_cache();
        let mut cols = cache.column_names("ks1", "orders");
        cols.sort();
        assert_eq!(cols, vec!["order_id", "total"]);
    }

    // ── udt_names tests ───────────────────────────────────────────────────────

    #[test]
    fn udt_names_returns_udts_for_keyspace() {
        let cache = populated_cache();
        assert_eq!(cache.udt_names("ks1"), vec!["address"]);
    }

    #[test]
    fn udt_names_empty_for_keyspace_with_no_udts() {
        let cache = populated_cache();
        assert!(cache.udt_names("ks2").is_empty());
    }

    #[test]
    fn udt_names_empty_for_unknown_keyspace() {
        let cache = populated_cache();
        assert!(cache.udt_names("nonexistent").is_empty());
    }

    // ── function_names tests ──────────────────────────────────────────────────

    #[test]
    fn function_names_returns_functions_for_keyspace() {
        let cache = populated_cache();
        assert_eq!(cache.function_names("ks1"), vec!["my_func"]);
    }

    #[test]
    fn function_names_empty_for_keyspace_with_no_functions() {
        let cache = populated_cache();
        assert!(cache.function_names("ks2").is_empty());
    }

    #[test]
    fn function_names_empty_for_unknown_keyspace() {
        let cache = populated_cache();
        assert!(cache.function_names("nonexistent").is_empty());
    }

    // ── aggregate_names tests ─────────────────────────────────────────────────

    #[test]
    fn aggregate_names_returns_aggregates_for_keyspace() {
        let cache = populated_cache();
        assert_eq!(cache.aggregate_names("ks1"), vec!["my_agg"]);
    }

    #[test]
    fn aggregate_names_empty_for_keyspace_with_no_aggregates() {
        let cache = populated_cache();
        assert!(cache.aggregate_names("ks2").is_empty());
    }

    #[test]
    fn aggregate_names_empty_for_unknown_keyspace() {
        let cache = populated_cache();
        assert!(cache.aggregate_names("nonexistent").is_empty());
    }

    // ── Multi-keyspace isolation tests ────────────────────────────────────────

    #[test]
    fn tables_are_isolated_per_keyspace() {
        let cache = populated_cache();
        assert!(!cache.table_names("ks1").contains(&"events"));
        assert!(!cache.table_names("ks2").contains(&"users"));
    }

    #[test]
    fn udts_are_isolated_per_keyspace() {
        let mut cache = populated_cache();
        cache
            .udts
            .insert("ks2".to_string(), vec![make_udt("ks2", "location")]);

        assert_eq!(cache.udt_names("ks1"), vec!["address"]);
        assert_eq!(cache.udt_names("ks2"), vec!["location"]);
    }

    // ── Edge-case tests ───────────────────────────────────────────────────────

    #[test]
    fn multiple_functions_returned_in_order() {
        let mut cache = SchemaCache::new();
        cache.keyspaces = vec![make_keyspace("ks1")];
        cache.functions.insert(
            "ks1".to_string(),
            vec![
                make_function("ks1", "alpha"),
                make_function("ks1", "beta"),
                make_function("ks1", "gamma"),
            ],
        );
        cache.last_refresh = Some(Instant::now());

        assert_eq!(cache.function_names("ks1"), vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn multiple_aggregates_returned_in_order() {
        let mut cache = SchemaCache::new();
        cache.keyspaces = vec![make_keyspace("ks1")];
        cache.aggregates.insert(
            "ks1".to_string(),
            vec![
                make_aggregate("ks1", "agg_a"),
                make_aggregate("ks1", "agg_b"),
            ],
        );
        cache.last_refresh = Some(Instant::now());

        assert_eq!(cache.aggregate_names("ks1"), vec!["agg_a", "agg_b"]);
    }

    #[test]
    fn table_with_no_columns_returns_empty_column_list() {
        let mut cache = SchemaCache::new();
        cache.keyspaces = vec![make_keyspace("ks1")];
        cache.tables.insert(
            "ks1".to_string(),
            vec![make_table("ks1", "empty_table", &[])],
        );
        cache.last_refresh = Some(Instant::now());

        assert!(cache.column_names("ks1", "empty_table").is_empty());
    }

    #[test]
    fn zero_ttl_cache_is_immediately_stale_after_refresh() {
        let mut cache = SchemaCache::with_ttl(Duration::ZERO);
        // Simulate a past refresh.
        cache.last_refresh = Some(Instant::now() - Duration::from_nanos(1));
        assert!(cache.is_stale());
    }
}

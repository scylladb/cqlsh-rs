//! DESCRIBE command implementations for cqlsh-rs.
//!
//! Provides schema introspection commands matching Python cqlsh:
//! - DESCRIBE CLUSTER
//! - DESCRIBE KEYSPACES
//! - DESCRIBE KEYSPACE [name]
//! - DESCRIBE TABLES
//! - DESCRIBE TABLE <name>
//! - DESCRIBE SCHEMA

use std::io::Write;

use anyhow::Result;

use crate::session::CqlSession;

/// Execute a DESCRIBE command and write output.
///
/// Parses the DESCRIBE subcommand and dispatches to the appropriate handler.
pub async fn execute(session: &CqlSession, args: &str, writer: &mut dyn Write) -> Result<()> {
    let args = args.trim();
    let upper = args.to_uppercase();

    // DESCRIBE with no args — show help
    if args.is_empty() {
        writeln!(
            writer,
            "Usage: DESCRIBE [CLUSTER | KEYSPACES | KEYSPACE [name] | TABLES | TABLE <name> | SCHEMA]"
        )?;
        return Ok(());
    }

    if upper == "CLUSTER" {
        describe_cluster(session, writer).await
    } else if upper == "KEYSPACES" {
        describe_keyspaces(session, writer).await
    } else if upper == "TABLES" {
        describe_tables(session, writer).await
    } else if upper == "SCHEMA" {
        describe_schema(session, writer).await
    } else if upper == "KEYSPACE" {
        // DESCRIBE KEYSPACE with no name → current keyspace
        describe_keyspace(session, session.current_keyspace(), writer).await
    } else if upper.starts_with("KEYSPACE ") {
        // DESCRIBE KEYSPACE <name>
        let ks_name = args["KEYSPACE ".len()..].trim();
        let ks_name = strip_quotes(ks_name);
        describe_keyspace(session, Some(ks_name), writer).await
    } else if upper == "TABLE" {
        writeln!(writer, "DESCRIBE TABLE requires a table name.")?;
        Ok(())
    } else if upper.starts_with("TABLE ") {
        // DESCRIBE TABLE <name>
        let table_spec = args["TABLE ".len()..].trim();
        let table_spec = strip_quotes(table_spec);
        describe_table(session, table_spec, writer).await
    } else {
        // Try to guess: could be a keyspace name, table name, or keyspace.table
        // Check if it matches a keyspace first, then a table in current keyspace
        let name = strip_quotes(args);
        if name.contains('.') {
            // Qualified table name: keyspace.table
            describe_table(session, name, writer).await
        } else {
            // Try as keyspace first
            let keyspaces = session.get_keyspaces().await?;
            if keyspaces.iter().any(|ks| ks.name == name) {
                describe_keyspace(session, Some(name), writer).await
            } else {
                // Try as table in current keyspace
                describe_table(session, name, writer).await
            }
        }
    }
}

/// DESCRIBE CLUSTER — show cluster name and partitioner.
async fn describe_cluster(session: &CqlSession, writer: &mut dyn Write) -> Result<()> {
    let cluster_name = session
        .cluster_name
        .as_deref()
        .unwrap_or("Unknown Cluster");

    writeln!(writer)?;
    writeln!(writer, "Cluster: {cluster_name}")?;
    writeln!(writer, "Partitioner: Murmur3Partitioner")?;

    // Fetch snitch info from system.local
    match session
        .execute_query("SELECT snitch FROM system.local")
        .await
    {
        Ok(result) => {
            if let Some(row) = result.rows.first() {
                if let Some(snitch) = row.get(0) {
                    writeln!(writer, "Snitch: {snitch}")?;
                }
            }
        }
        Err(_) => {
            // Snitch info may not be available in all configurations
        }
    }
    writeln!(writer)?;
    Ok(())
}

/// DESCRIBE KEYSPACES — list all keyspace names.
async fn describe_keyspaces(session: &CqlSession, writer: &mut dyn Write) -> Result<()> {
    let keyspaces = session.get_keyspaces().await?;
    writeln!(writer)?;
    for ks in &keyspaces {
        write!(writer, "{}", ks.name)?;
        // Add spaces between keyspace names (Python cqlsh style)
        write!(writer, "  ")?;
    }
    writeln!(writer)?;
    writeln!(writer)?;
    Ok(())
}

/// DESCRIBE KEYSPACE [name] — show CREATE KEYSPACE statement.
async fn describe_keyspace(
    session: &CqlSession,
    keyspace: Option<&str>,
    writer: &mut dyn Write,
) -> Result<()> {
    let ks_name = match keyspace {
        Some(name) => name,
        None => {
            writeln!(
                writer,
                "No keyspace specified and no current keyspace. Use DESCRIBE KEYSPACE <name>."
            )?;
            return Ok(());
        }
    };

    // Fetch keyspace details from system_schema
    let query = format!(
        "SELECT replication FROM system_schema.keyspaces WHERE keyspace_name = '{}'",
        ks_name.replace('\'', "''")
    );
    let result = session.execute_query(&query).await?;

    if result.rows.is_empty() {
        writeln!(writer, "Keyspace '{ks_name}' not found.")?;
        return Ok(());
    }

    // Fetch durable_writes
    let dw_query = format!(
        "SELECT durable_writes FROM system_schema.keyspaces WHERE keyspace_name = '{}'",
        ks_name.replace('\'', "''")
    );
    let dw_result = session.execute_query(&dw_query).await?;
    let durable_writes = dw_result
        .rows
        .first()
        .and_then(|r| r.get(0))
        .map(|v| v.to_string() == "True")
        .unwrap_or(true);

    // Build replication string from the map value
    let replication_str = result
        .rows
        .first()
        .and_then(|r| r.get(0))
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());

    writeln!(writer)?;
    writeln!(
        writer,
        "CREATE KEYSPACE {ks_name} WITH replication = {replication_str} AND durable_writes = {durable_writes};"
    )?;
    writeln!(writer)?;
    Ok(())
}

/// DESCRIBE TABLES — list tables in the current keyspace.
async fn describe_tables(session: &CqlSession, writer: &mut dyn Write) -> Result<()> {
    let keyspace = match session.current_keyspace() {
        Some(ks) => ks.to_string(),
        None => {
            writeln!(
                writer,
                "No keyspace selected. Use USE <keyspace> first, or DESCRIBE KEYSPACE <name>."
            )?;
            return Ok(());
        }
    };

    let tables = session.get_tables(&keyspace).await?;
    if tables.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "Keyspace '{keyspace}' has no tables.")?;
        writeln!(writer)?;
        return Ok(());
    }

    writeln!(writer)?;
    for table in &tables {
        write!(writer, "{}", table.name)?;
        write!(writer, "  ")?;
    }
    writeln!(writer)?;
    writeln!(writer)?;
    Ok(())
}

/// DESCRIBE TABLE <name> — show CREATE TABLE statement.
async fn describe_table(
    session: &CqlSession,
    table_spec: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    let (keyspace, table_name) = if table_spec.contains('.') {
        let parts: Vec<&str> = table_spec.splitn(2, '.').collect();
        (parts[0].to_string(), parts[1].to_string())
    } else {
        match session.current_keyspace() {
            Some(ks) => (ks.to_string(), table_spec.to_string()),
            None => {
                writeln!(
                    writer,
                    "No keyspace selected. Use a fully qualified name: DESCRIBE TABLE keyspace.table"
                )?;
                return Ok(());
            }
        }
    };

    let table = session.get_table_metadata(&keyspace, &table_name).await?;

    match table {
        Some(meta) => {
            writeln!(writer)?;
            write_create_table(writer, &meta)?;
            writeln!(writer)?;
        }
        None => {
            writeln!(writer, "Table '{keyspace}.{table_name}' not found.")?;
        }
    }

    Ok(())
}

/// DESCRIBE SCHEMA — show CREATE statements for all user keyspaces and their tables.
async fn describe_schema(session: &CqlSession, writer: &mut dyn Write) -> Result<()> {
    let keyspaces = session.get_keyspaces().await?;

    let user_keyspaces: Vec<_> = keyspaces
        .iter()
        .filter(|ks| !is_system_keyspace(&ks.name))
        .collect();

    if user_keyspaces.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "No user-defined keyspaces found.")?;
        writeln!(writer)?;
        return Ok(());
    }

    for ks in user_keyspaces {
        // Print DESCRIBE KEYSPACE
        describe_keyspace(session, Some(&ks.name), writer).await?;

        // Print all tables in this keyspace
        let tables = session.get_tables(&ks.name).await?;
        for table in &tables {
            writeln!(writer)?;
            write_create_table(writer, table)?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Write a CREATE TABLE statement for the given table metadata.
fn write_create_table(
    writer: &mut dyn Write,
    meta: &crate::driver::TableMetadata,
) -> Result<()> {
    writeln!(
        writer,
        "CREATE TABLE {}.{} (",
        quote_if_needed(&meta.keyspace),
        quote_if_needed(&meta.name)
    )?;

    // Print columns
    for col in &meta.columns {
        writeln!(
            writer,
            "    {} {},",
            quote_if_needed(&col.name),
            col.type_name
        )?;
    }

    // Print PRIMARY KEY
    if !meta.partition_key.is_empty() {
        let pk_str = if meta.partition_key.len() == 1 {
            quote_if_needed(&meta.partition_key[0])
        } else {
            format!(
                "({})",
                meta.partition_key
                    .iter()
                    .map(|k| quote_if_needed(k))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        if meta.clustering_key.is_empty() {
            writeln!(writer, "    PRIMARY KEY ({pk_str})")?;
        } else {
            let ck_str = meta
                .clustering_key
                .iter()
                .map(|k| quote_if_needed(k))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(writer, "    PRIMARY KEY ({pk_str}, {ck_str})")?;
        }
    }

    writeln!(writer, ");")?;
    Ok(())
}

/// Check if a keyspace is a system keyspace.
fn is_system_keyspace(name: &str) -> bool {
    name.starts_with("system")
        || name == "dse_system"
        || name == "dse_perf"
        || name == "dse_security"
        || name == "dse_leases"
        || name == "dse_system_local"
        || name == "dse_insights"
        || name == "solr_admin"
}

/// Quote an identifier if it needs quoting (contains uppercase, spaces, or reserved words).
fn quote_if_needed(name: &str) -> String {
    // Simple heuristic: quote if not all lowercase alphanumeric + underscore
    if name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !name.is_empty()
        && !name.starts_with(|c: char| c.is_ascii_digit())
    {
        name.to_string()
    } else {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

/// Strip surrounding single or double quotes from a string.
fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_keyspace_detection() {
        assert!(is_system_keyspace("system"));
        assert!(is_system_keyspace("system_schema"));
        assert!(is_system_keyspace("system_auth"));
        assert!(is_system_keyspace("system_traces"));
        assert!(!is_system_keyspace("my_keyspace"));
        assert!(!is_system_keyspace("users"));
    }

    #[test]
    fn quote_simple_identifier() {
        assert_eq!(quote_if_needed("users"), "users");
        assert_eq!(quote_if_needed("my_table"), "my_table");
    }

    #[test]
    fn quote_mixed_case_identifier() {
        assert_eq!(quote_if_needed("MyTable"), "\"MyTable\"");
    }

    #[test]
    fn quote_identifier_with_spaces() {
        assert_eq!(quote_if_needed("my table"), "\"my table\"");
    }

    #[test]
    fn quote_identifier_starting_with_digit() {
        assert_eq!(quote_if_needed("1table"), "\"1table\"");
    }

    #[test]
    fn strip_quotes_test() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
        assert_eq!(strip_quotes("'hello'"), "hello");
        assert_eq!(strip_quotes("hello"), "hello");
    }

    #[test]
    fn write_create_table_simple() {
        use crate::driver::{ColumnMetadata, TableMetadata};

        let meta = TableMetadata {
            keyspace: "test_ks".to_string(),
            name: "users".to_string(),
            columns: vec![
                ColumnMetadata {
                    name: "id".to_string(),
                    type_name: "uuid".to_string(),
                },
                ColumnMetadata {
                    name: "name".to_string(),
                    type_name: "text".to_string(),
                },
                ColumnMetadata {
                    name: "age".to_string(),
                    type_name: "int".to_string(),
                },
            ],
            partition_key: vec!["id".to_string()],
            clustering_key: vec![],
        };

        let mut buf = Vec::new();
        write_create_table(&mut buf, &meta).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("CREATE TABLE test_ks.users"));
        assert!(output.contains("id uuid"));
        assert!(output.contains("name text"));
        assert!(output.contains("PRIMARY KEY (id)"));
    }

    #[test]
    fn write_create_table_composite_key() {
        use crate::driver::{ColumnMetadata, TableMetadata};

        let meta = TableMetadata {
            keyspace: "ks".to_string(),
            name: "events".to_string(),
            columns: vec![
                ColumnMetadata {
                    name: "user_id".to_string(),
                    type_name: "uuid".to_string(),
                },
                ColumnMetadata {
                    name: "event_time".to_string(),
                    type_name: "timestamp".to_string(),
                },
                ColumnMetadata {
                    name: "data".to_string(),
                    type_name: "text".to_string(),
                },
            ],
            partition_key: vec!["user_id".to_string()],
            clustering_key: vec!["event_time".to_string()],
        };

        let mut buf = Vec::new();
        write_create_table(&mut buf, &meta).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("PRIMARY KEY (user_id, event_time)"));
    }

    #[test]
    fn write_create_table_compound_partition_key() {
        use crate::driver::{ColumnMetadata, TableMetadata};

        let meta = TableMetadata {
            keyspace: "ks".to_string(),
            name: "metrics".to_string(),
            columns: vec![
                ColumnMetadata {
                    name: "host".to_string(),
                    type_name: "text".to_string(),
                },
                ColumnMetadata {
                    name: "metric".to_string(),
                    type_name: "text".to_string(),
                },
                ColumnMetadata {
                    name: "ts".to_string(),
                    type_name: "timestamp".to_string(),
                },
                ColumnMetadata {
                    name: "value".to_string(),
                    type_name: "double".to_string(),
                },
            ],
            partition_key: vec!["host".to_string(), "metric".to_string()],
            clustering_key: vec!["ts".to_string()],
        };

        let mut buf = Vec::new();
        write_create_table(&mut buf, &meta).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("PRIMARY KEY ((host, metric), ts)"));
    }
}

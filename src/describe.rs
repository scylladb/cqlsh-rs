//! DESCRIBE command implementations for cqlsh-rs.
//!
//! Provides schema introspection commands matching Python cqlsh:
//! - DESCRIBE CLUSTER
//! - DESCRIBE KEYSPACES
//! - DESCRIBE KEYSPACE [name]
//! - DESCRIBE TABLES
//! - DESCRIBE TABLE <name>
//! - DESCRIBE SCHEMA
//! - DESCRIBE FULL SCHEMA
//! - DESCRIBE INDEX <name>
//! - DESCRIBE MATERIALIZED VIEW <name>
//! - DESCRIBE TYPE <name> / DESCRIBE TYPES
//! - DESCRIBE FUNCTION <name> / DESCRIBE FUNCTIONS
//! - DESCRIBE AGGREGATE <name> / DESCRIBE AGGREGATES

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
            "Usage: DESCRIBE [CLUSTER | KEYSPACES | KEYSPACE [name] | TABLES | TABLE <name> | SCHEMA | FULL SCHEMA | INDEX <name> | MATERIALIZED VIEW <name> | TYPES | TYPE <name> | FUNCTIONS | FUNCTION <name> | AGGREGATES | AGGREGATE <name>]"
        )?;
        return Ok(());
    }

    if upper == "CLUSTER" {
        describe_cluster(session, writer).await
    } else if upper == "KEYSPACES" {
        describe_keyspaces(session, writer).await
    } else if upper == "TABLES" {
        describe_tables(session, writer).await
    } else if upper == "FULL SCHEMA" {
        describe_full_schema(session, writer).await
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
    } else if upper == "INDEX" {
        writeln!(writer, "DESCRIBE INDEX requires an index name.")?;
        Ok(())
    } else if upper.starts_with("INDEX ") {
        let index_spec = args["INDEX ".len()..].trim();
        let index_spec = strip_quotes(index_spec);
        describe_index(session, index_spec, writer).await
    } else if upper == "MATERIALIZED VIEW" {
        writeln!(writer, "DESCRIBE MATERIALIZED VIEW requires a view name.")?;
        Ok(())
    } else if upper.starts_with("MATERIALIZED VIEW ") {
        let view_spec = args["MATERIALIZED VIEW ".len()..].trim();
        let view_spec = strip_quotes(view_spec);
        describe_materialized_view(session, view_spec, writer).await
    } else if upper == "TYPES" {
        describe_types(session, writer).await
    } else if upper == "TYPE" {
        writeln!(writer, "DESCRIBE TYPE requires a type name.")?;
        Ok(())
    } else if upper.starts_with("TYPE ") {
        let type_spec = args["TYPE ".len()..].trim();
        let type_spec = strip_quotes(type_spec);
        describe_type(session, type_spec, writer).await
    } else if upper == "FUNCTIONS" {
        describe_functions(session, writer).await
    } else if upper == "FUNCTION" {
        writeln!(writer, "DESCRIBE FUNCTION requires a function name.")?;
        Ok(())
    } else if upper.starts_with("FUNCTION ") {
        let func_spec = args["FUNCTION ".len()..].trim();
        let func_spec = strip_quotes(func_spec);
        describe_function(session, func_spec, writer).await
    } else if upper == "AGGREGATES" {
        describe_aggregates(session, writer).await
    } else if upper == "AGGREGATE" {
        writeln!(writer, "DESCRIBE AGGREGATE requires an aggregate name.")?;
        Ok(())
    } else if upper.starts_with("AGGREGATE ") {
        let agg_spec = args["AGGREGATE ".len()..].trim();
        let agg_spec = strip_quotes(agg_spec);
        describe_aggregate(session, agg_spec, writer).await
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
    let cluster_name = session.cluster_name.as_deref().unwrap_or("Unknown Cluster");

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

/// DESCRIBE KEYSPACE [name] — show CREATE KEYSPACE and all objects within it.
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

    // Print all tables and their indexes in this keyspace
    let tables = session.get_tables(ks_name).await?;
    for table in &tables {
        writeln!(writer)?;
        write_create_table(writer, table)?;
        write_table_indexes(session, ks_name, &table.name, writer).await?;
    }

    write_keyspace_materialized_views(session, ks_name, writer).await?;

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
    describe_schema_inner(session, writer, false).await
}

/// DESCRIBE FULL SCHEMA — show CREATE statements for ALL keyspaces (including system).
async fn describe_full_schema(session: &CqlSession, writer: &mut dyn Write) -> Result<()> {
    describe_schema_inner(session, writer, true).await
}

/// Shared implementation for DESCRIBE SCHEMA and DESCRIBE FULL SCHEMA.
async fn describe_schema_inner(
    session: &CqlSession,
    writer: &mut dyn Write,
    include_system: bool,
) -> Result<()> {
    let keyspaces = session.get_keyspaces().await?;

    let filtered_keyspaces: Vec<_> = if include_system {
        keyspaces.iter().collect()
    } else {
        keyspaces
            .iter()
            .filter(|ks| !is_system_keyspace(&ks.name))
            .collect()
    };

    if filtered_keyspaces.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "No user-defined keyspaces found.")?;
        writeln!(writer)?;
        return Ok(());
    }

    for ks in filtered_keyspaces {
        // Print DESCRIBE KEYSPACE
        describe_keyspace(session, Some(&ks.name), writer).await?;

        // Print all tables in this keyspace
        let tables = session.get_tables(&ks.name).await?;
        for table in &tables {
            writeln!(writer)?;
            write_create_table(writer, table)?;
            write_table_indexes(session, &ks.name, &table.name, writer).await?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// DESCRIBE INDEX <name> — show CREATE INDEX statement.
async fn describe_index(
    session: &CqlSession,
    index_spec: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    let (keyspace, index_name) = resolve_qualified_name(session, index_spec, writer)?;
    let keyspace = match keyspace {
        Some(ks) => ks,
        None => return Ok(()),
    };

    // system_schema.indexes PK is (keyspace_name, table_name, index_name).
    // We cannot filter on index_name without table_name, so we scan the whole
    // keyspace partition and match in Rust.
    let query = format!(
        "SELECT index_name, table_name, kind, options FROM system_schema.indexes WHERE keyspace_name = '{}'",
        keyspace.replace('\'', "''"),
    );
    let result = session.execute_query(&query).await?;

    let row = result.rows.iter().find(|r| {
        r.get_by_name("index_name", &result.columns)
            .map(|v| v.to_string().to_lowercase())
            .as_deref()
            == Some(index_name.to_lowercase().as_str())
    });

    let row = match row {
        Some(r) => r,
        None => {
            writeln!(writer, "Index '{keyspace}.{index_name}' not found.")?;
            return Ok(());
        }
    };

    let idx_name = row
        .get_by_name("index_name", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let table_name = row
        .get_by_name("table_name", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let options = row
        .get_by_name("options", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();

    // Extract target column from options map
    // The options map contains 'target' key with the indexed column
    let target = extract_map_value(&options, "target").unwrap_or_else(|| "unknown".to_string());

    write!(
        writer,
        "{}",
        format_index_ddl(&keyspace, &idx_name, &table_name, &target)
    )?;
    Ok(())
}

/// DESCRIBE MATERIALIZED VIEW <name> — show CREATE MATERIALIZED VIEW statement.
async fn describe_materialized_view(
    session: &CqlSession,
    view_spec: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    let (keyspace, view_name) = resolve_qualified_name(session, view_spec, writer)?;
    let keyspace = match keyspace {
        Some(ks) => ks,
        None => return Ok(()),
    };

    let query = format!(
        "SELECT view_name, base_table_name, where_clause, include_all_columns FROM system_schema.views WHERE keyspace_name = '{}' AND view_name = '{}'",
        keyspace.replace('\'', "''"),
        view_name.replace('\'', "''")
    );
    let result = session.execute_query(&query).await?;

    if result.rows.is_empty() {
        writeln!(
            writer,
            "Materialized view '{keyspace}.{view_name}' not found."
        )?;
        return Ok(());
    }

    let row = &result.rows[0];
    let mv_name = row
        .get_by_name("view_name", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let base_table = row
        .get_by_name("base_table_name", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let where_clause = row
        .get_by_name("where_clause", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "IS NOT NULL".to_string());
    let include_all = row
        .get_by_name("include_all_columns", &result.columns)
        .map(|v| v.to_string() == "True")
        .unwrap_or(false);

    // Fetch columns for the view
    // system_schema.columns is clustered by column_name, not position, so we
    // cannot ORDER BY position in CQL — fetch all and sort in Rust.
    let col_query = format!(
        "SELECT column_name, type, kind, position, clustering_order FROM system_schema.columns WHERE keyspace_name = '{}' AND table_name = '{}'",
        keyspace.replace('\'', "''"),
        mv_name.replace('\'', "''")
    );
    let col_result = session.execute_query(&col_query).await?;

    let mut select_columns = Vec::new();
    let mut partition_keys: Vec<(i32, String)> = Vec::new();
    let mut clustering_keys: Vec<(i32, String, String)> = Vec::new();

    for col_row in &col_result.rows {
        let col_name = col_row
            .get_by_name("column_name", &col_result.columns)
            .map(|v| v.to_string())
            .unwrap_or_default();
        let kind = col_row
            .get_by_name("kind", &col_result.columns)
            .map(|v| v.to_string())
            .unwrap_or_default();
        let position = col_row
            .get_by_name("position", &col_result.columns)
            .and_then(|v| v.to_string().parse::<i32>().ok())
            .unwrap_or(0);
        let clustering_order = col_row
            .get_by_name("clustering_order", &col_result.columns)
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string());

        select_columns.push(col_name.clone());

        if kind == "partition_key" {
            partition_keys.push((position, col_name));
        } else if kind == "clustering" {
            clustering_keys.push((position, col_name, clustering_order));
        }
    }

    partition_keys.sort_by_key(|k| k.0);
    clustering_keys.sort_by_key(|k| k.0);

    let sorted_pk: Vec<String> = partition_keys
        .iter()
        .map(|(_, name)| name.clone())
        .collect();
    let sorted_ck: Vec<(String, String)> = clustering_keys
        .iter()
        .map(|(_, name, order)| (name.clone(), order.clone()))
        .collect();

    let props_query = format!(
        "SELECT bloom_filter_fp_chance, caching, comment, compaction, compression, \
         crc_check_chance, default_time_to_live, gc_grace_seconds, \
         max_index_interval, memtable_flush_period_in_ms, min_index_interval, \
         speculative_retry \
         FROM system_schema.views \
         WHERE keyspace_name = '{}' AND view_name = '{}'",
        keyspace.replace('\'', "''"),
        mv_name.replace('\'', "''")
    );
    let props_result = session.execute_query(&props_query).await?;

    let mut properties = std::collections::BTreeMap::new();
    if let Some(props_row) = props_result.rows.first() {
        let prop_names = [
            "bloom_filter_fp_chance",
            "caching",
            "comment",
            "compaction",
            "compression",
            "crc_check_chance",
            "default_time_to_live",
            "gc_grace_seconds",
            "max_index_interval",
            "memtable_flush_period_in_ms",
            "min_index_interval",
            "speculative_retry",
        ];
        for prop_name in &prop_names {
            if let Some(val) = props_row.get_by_name(prop_name, &props_result.columns) {
                properties.insert(prop_name.to_string(), val.to_string());
            }
        }
    }

    let parts = MvDdlParts {
        keyspace: &keyspace,
        view_name: &mv_name,
        base_table: &base_table,
        include_all,
        select_columns: &select_columns,
        where_clause: &where_clause,
        partition_keys: &sorted_pk,
        clustering_keys: &sorted_ck,
        properties: &properties,
    };
    write!(writer, "{}", format_create_mv_ddl(&parts))?;
    Ok(())
}

/// DESCRIBE TYPES — list all UDT names in the current keyspace.
async fn describe_types(session: &CqlSession, writer: &mut dyn Write) -> Result<()> {
    let keyspace = match session.current_keyspace() {
        Some(ks) => ks.to_string(),
        None => {
            writeln!(writer, "No keyspace selected. Use USE <keyspace> first.")?;
            return Ok(());
        }
    };

    let udts = session.get_udts(&keyspace).await?;
    if udts.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "Keyspace '{keyspace}' has no user-defined types.")?;
        writeln!(writer)?;
        return Ok(());
    }

    writeln!(writer)?;
    for udt in &udts {
        write!(writer, "{}  ", udt.name)?;
    }
    writeln!(writer)?;
    writeln!(writer)?;
    Ok(())
}

/// DESCRIBE TYPE <name> — show CREATE TYPE statement.
async fn describe_type(
    session: &CqlSession,
    type_spec: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    let (keyspace, type_name) = resolve_qualified_name(session, type_spec, writer)?;
    let keyspace = match keyspace {
        Some(ks) => ks,
        None => return Ok(()),
    };

    let query = format!(
        "SELECT type_name, field_names, field_types FROM system_schema.types WHERE keyspace_name = '{}' AND type_name = '{}'",
        keyspace.replace('\'', "''"),
        type_name.replace('\'', "''")
    );
    let result = session.execute_query(&query).await?;

    if result.rows.is_empty() {
        writeln!(writer, "Type '{keyspace}.{type_name}' not found.")?;
        return Ok(());
    }

    let row = &result.rows[0];
    let udt_name = row
        .get_by_name("type_name", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let field_names_str = row
        .get_by_name("field_names", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let field_types_str = row
        .get_by_name("field_types", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();

    let field_names = parse_list_value(&field_names_str);
    let field_types = parse_list_value(&field_types_str);

    let field_count = field_names.len().min(field_types.len());
    let fields: Vec<(String, String)> = field_names
        .into_iter()
        .take(field_count)
        .zip(field_types)
        .collect();
    write!(
        writer,
        "{}",
        format_create_type_ddl(&keyspace, &udt_name, &fields)
    )?;
    Ok(())
}

/// DESCRIBE FUNCTIONS — list all UDF names in the current keyspace.
async fn describe_functions(session: &CqlSession, writer: &mut dyn Write) -> Result<()> {
    let keyspace = match session.current_keyspace() {
        Some(ks) => ks.to_string(),
        None => {
            writeln!(writer, "No keyspace selected. Use USE <keyspace> first.")?;
            return Ok(());
        }
    };

    let functions = session.get_functions(&keyspace).await?;
    if functions.is_empty() {
        writeln!(writer)?;
        writeln!(
            writer,
            "Keyspace '{keyspace}' has no user-defined functions."
        )?;
        writeln!(writer)?;
        return Ok(());
    }

    writeln!(writer)?;
    for func in &functions {
        write!(writer, "{}  ", func.name)?;
    }
    writeln!(writer)?;
    writeln!(writer)?;
    Ok(())
}

/// DESCRIBE FUNCTION <name> — show CREATE FUNCTION statement.
async fn describe_function(
    session: &CqlSession,
    func_spec: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    let (keyspace, func_name) = resolve_qualified_name(session, func_spec, writer)?;
    let keyspace = match keyspace {
        Some(ks) => ks,
        None => return Ok(()),
    };

    let query = format!(
        "SELECT function_name, argument_names, argument_types, return_type, language, body, called_on_null_input FROM system_schema.functions WHERE keyspace_name = '{}' AND function_name = '{}'",
        keyspace.replace('\'', "''"),
        func_name.replace('\'', "''")
    );
    let result = session.execute_query(&query).await?;

    if result.rows.is_empty() {
        writeln!(writer, "Function '{keyspace}.{func_name}' not found.")?;
        return Ok(());
    }

    let row = &result.rows[0];
    let fn_name = row
        .get_by_name("function_name", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let arg_names_str = row
        .get_by_name("argument_names", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let arg_types_str = row
        .get_by_name("argument_types", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let return_type = row
        .get_by_name("return_type", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let language = row
        .get_by_name("language", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let body = row
        .get_by_name("body", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let called_on_null = row
        .get_by_name("called_on_null_input", &result.columns)
        .map(|v| v.to_string() == "True")
        .unwrap_or(false);

    let arg_names = parse_list_value(&arg_names_str);
    let arg_types = parse_list_value(&arg_types_str);

    let args_str = arg_names
        .iter()
        .zip(arg_types.iter())
        .map(|(name, typ)| format!("{} {}", quote_if_needed(name), typ))
        .collect::<Vec<_>>()
        .join(", ");

    let null_handling = if called_on_null {
        "CALLED ON NULL INPUT"
    } else {
        "RETURNS NULL ON NULL INPUT"
    };

    write!(
        writer,
        "{}",
        format_create_function_ddl(
            &keyspace,
            &fn_name,
            &args_str,
            null_handling,
            &return_type,
            &language,
            &body,
        )
    )?;
    Ok(())
}

/// DESCRIBE AGGREGATES — list all UDA names in the current keyspace.
async fn describe_aggregates(session: &CqlSession, writer: &mut dyn Write) -> Result<()> {
    let keyspace = match session.current_keyspace() {
        Some(ks) => ks.to_string(),
        None => {
            writeln!(writer, "No keyspace selected. Use USE <keyspace> first.")?;
            return Ok(());
        }
    };

    let aggregates = session.get_aggregates(&keyspace).await?;
    if aggregates.is_empty() {
        writeln!(writer)?;
        writeln!(
            writer,
            "Keyspace '{keyspace}' has no user-defined aggregates."
        )?;
        writeln!(writer)?;
        return Ok(());
    }

    writeln!(writer)?;
    for agg in &aggregates {
        write!(writer, "{}  ", agg.name)?;
    }
    writeln!(writer)?;
    writeln!(writer)?;
    Ok(())
}

/// DESCRIBE AGGREGATE <name> — show CREATE AGGREGATE statement.
async fn describe_aggregate(
    session: &CqlSession,
    agg_spec: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    let (keyspace, agg_name) = resolve_qualified_name(session, agg_spec, writer)?;
    let keyspace = match keyspace {
        Some(ks) => ks,
        None => return Ok(()),
    };

    let query = format!(
        "SELECT aggregate_name, argument_types, state_func, state_type, final_func, initcond FROM system_schema.aggregates WHERE keyspace_name = '{}' AND aggregate_name = '{}'",
        keyspace.replace('\'', "''"),
        agg_name.replace('\'', "''")
    );
    let result = session.execute_query(&query).await?;

    if result.rows.is_empty() {
        writeln!(writer, "Aggregate '{keyspace}.{agg_name}' not found.")?;
        return Ok(());
    }

    let row = &result.rows[0];
    let ag_name = row
        .get_by_name("aggregate_name", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let arg_types_str = row
        .get_by_name("argument_types", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let state_func = row
        .get_by_name("state_func", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let state_type = row
        .get_by_name("state_type", &result.columns)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let final_func = row
        .get_by_name("final_func", &result.columns)
        .map(|v| v.to_string());
    let initcond = row
        .get_by_name("initcond", &result.columns)
        .map(|v| v.to_string());

    let arg_types = parse_list_value(&arg_types_str);
    let args_str = arg_types.join(", ");

    write!(
        writer,
        "{}",
        format_create_aggregate_ddl(
            &keyspace,
            &ag_name,
            &args_str,
            &state_func,
            &state_type,
            final_func.as_deref(),
            initcond.as_deref(),
        )
    )?;
    Ok(())
}

/// Write a CREATE TABLE statement for the given table metadata.
fn write_create_table(writer: &mut dyn Write, meta: &crate::driver::TableMetadata) -> Result<()> {
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

    writeln!(writer, ")")?;

    let mut first_with = true;

    if !meta.clustering_key.is_empty() {
        let order_parts: Vec<String> = meta
            .clustering_key
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let order = meta
                    .clustering_order
                    .get(i)
                    .map(|s| s.as_str())
                    .unwrap_or("ASC");
                format!("{} {}", quote_if_needed(name), order)
            })
            .collect();
        write!(
            writer,
            " WITH CLUSTERING ORDER BY ({})",
            order_parts.join(", ")
        )?;
        first_with = false;
    }

    let prop_order = [
        "bloom_filter_fp_chance",
        "caching",
        "comment",
        "compaction",
        "compression",
        "crc_check_chance",
        "default_time_to_live",
        "gc_grace_seconds",
        "max_index_interval",
        "memtable_flush_period_in_ms",
        "min_index_interval",
        "speculative_retry",
    ];

    for prop_name in &prop_order {
        if let Some(value) = meta.properties.get(*prop_name) {
            let formatted_value = format_property_value(prop_name, value);
            if first_with {
                write!(writer, " WITH {} = {}", prop_name, formatted_value)?;
                first_with = false;
            } else {
                write!(writer, "\n    AND {} = {}", prop_name, formatted_value)?;
            }
        }
    }

    writeln!(writer, ";")?;
    Ok(())
}

/// Fetch indexes for a table from system_schema.indexes and write CREATE INDEX statements.
async fn write_table_indexes(
    session: &CqlSession,
    keyspace: &str,
    table_name: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    let query = format!(
        "SELECT index_name, table_name, kind, options FROM system_schema.indexes WHERE keyspace_name = '{}' AND table_name = '{}'",
        keyspace.replace('\'', "''"),
        table_name.replace('\'', "''"),
    );
    let result = session.execute_query(&query).await?;

    for row in &result.rows {
        let idx_name = row
            .get_by_name("index_name", &result.columns)
            .map(|v| v.to_string())
            .unwrap_or_default();
        let tbl_name = row
            .get_by_name("table_name", &result.columns)
            .map(|v| v.to_string())
            .unwrap_or_default();
        let options = row
            .get_by_name("options", &result.columns)
            .map(|v| v.to_string())
            .unwrap_or_default();

        let target = extract_map_value(&options, "target").unwrap_or_else(|| "unknown".to_string());

        writeln!(
            writer,
            "CREATE INDEX {} ON {}.{} ({});",
            quote_if_needed(&idx_name),
            quote_if_needed(keyspace),
            quote_if_needed(&tbl_name),
            target
        )?;
    }

    Ok(())
}

async fn write_keyspace_materialized_views(
    session: &CqlSession,
    keyspace: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    let query = format!(
        "SELECT view_name FROM system_schema.views WHERE keyspace_name = '{}'",
        keyspace.replace('\'', "''")
    );
    let result = session.execute_query(&query).await?;

    for row in &result.rows {
        if let Some(view_name) = row.get(0) {
            let view_name = view_name.to_string();
            writeln!(writer)?;
            describe_materialized_view(session, &format!("{keyspace}.{view_name}"), writer).await?;
        }
    }

    Ok(())
}

/// Resolve a potentially qualified name (ks.name or just name) into (keyspace, name).
///
/// If no keyspace prefix is given, uses the session's current keyspace.
/// Returns `(None, name)` if no keyspace can be determined (and prints an error).
fn resolve_qualified_name(
    session: &CqlSession,
    spec: &str,
    writer: &mut dyn Write,
) -> Result<(Option<String>, String)> {
    if spec.contains('.') {
        let parts: Vec<&str> = spec.splitn(2, '.').collect();
        Ok((Some(parts[0].to_string()), parts[1].to_string()))
    } else {
        match session.current_keyspace() {
            Some(ks) => Ok((Some(ks.to_string()), spec.to_string())),
            None => {
                writeln!(
                    writer,
                    "No keyspace selected. Use a fully qualified name (keyspace.name) or USE <keyspace> first."
                )?;
                Ok((None, spec.to_string()))
            }
        }
    }
}

/// Parse a CQL list string like `['a', 'b', 'c']` or `[a, b, c]` into a Vec of strings.
fn parse_list_value(s: &str) -> Vec<String> {
    let trimmed = s.trim();
    // Handle empty list
    if trimmed == "[]" || trimmed.is_empty() {
        return Vec::new();
    }
    // Strip surrounding brackets
    let inner = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    if inner.trim().is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .map(|s| {
            let s = s.trim();
            // Strip surrounding quotes
            if (s.starts_with('\'') && s.ends_with('\''))
                || (s.starts_with('"') && s.ends_with('"'))
            {
                s[1..s.len() - 1].to_string()
            } else {
                s.to_string()
            }
        })
        .collect()
}

/// Extract a value from a CQL map string like `{'key': 'value', ...}`.
fn extract_map_value(map_str: &str, key: &str) -> Option<String> {
    let trimmed = map_str.trim();
    // Strip surrounding braces
    let inner = if trimmed.starts_with('{') && trimmed.ends_with('}') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    // Simple parsing: split on commas, then on ':'
    for entry in inner.split(',') {
        let parts: Vec<&str> = entry.splitn(2, ':').collect();
        if parts.len() == 2 {
            let k = parts[0].trim().trim_matches('\'').trim_matches('"');
            let v = parts[1].trim().trim_matches('\'').trim_matches('"');
            if k == key {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn format_property_value(name: &str, value: &str) -> String {
    match name {
        "comment" | "speculative_retry" => format!("'{}'", value.replace('\'', "''")),
        "caching" | "compaction" | "compression" => value.to_string(),
        _ => value.to_string(),
    }
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

// --- Pure DDL formatters (testable without a session) ---

/// Format a CREATE INDEX DDL string.
fn format_index_ddl(keyspace: &str, index_name: &str, table_name: &str, target: &str) -> String {
    format!(
        "\nCREATE INDEX {} ON {}.{} ({});\n\n",
        quote_if_needed(index_name),
        quote_if_needed(keyspace),
        quote_if_needed(table_name),
        target
    )
}

/// Format a CREATE TYPE DDL string.
fn format_create_type_ddl(keyspace: &str, type_name: &str, fields: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!(
        "CREATE TYPE {}.{} (\n",
        quote_if_needed(keyspace),
        quote_if_needed(type_name)
    ));
    let field_count = fields.len();
    for (i, (name, typ)) in fields.iter().enumerate() {
        let comma = if i < field_count - 1 { "," } else { "" };
        out.push_str(&format!("    {} {}{}\n", quote_if_needed(name), typ, comma));
    }
    out.push_str(");\n\n");
    out
}

/// Format a CREATE FUNCTION DDL string.
fn format_create_function_ddl(
    keyspace: &str,
    func_name: &str,
    args_str: &str,
    null_handling: &str,
    return_type: &str,
    language: &str,
    body: &str,
) -> String {
    format!(
        "\nCREATE OR REPLACE FUNCTION {}.{} ({})\n    {}\n    RETURNS {}\n    LANGUAGE {}\n    AS $$ {} $$;\n\n",
        quote_if_needed(keyspace),
        quote_if_needed(func_name),
        args_str,
        null_handling,
        return_type,
        language,
        body
    )
}

/// Format a CREATE AGGREGATE DDL string.
fn format_create_aggregate_ddl(
    keyspace: &str,
    agg_name: &str,
    args_str: &str,
    state_func: &str,
    state_type: &str,
    final_func: Option<&str>,
    initcond: Option<&str>,
) -> String {
    let mut out = format!(
        "\nCREATE OR REPLACE AGGREGATE {}.{} ({})\n    SFUNC {}\n    STYPE {}",
        quote_if_needed(keyspace),
        quote_if_needed(agg_name),
        args_str,
        state_func,
        state_type
    );
    if let Some(ff) = final_func {
        if !ff.is_empty() && ff != "null" {
            out.push_str(&format!("\n    FINALFUNC {ff}"));
        }
    }
    if let Some(ic) = initcond {
        if !ic.is_empty() && ic != "null" {
            out.push_str(&format!("\n    INITCOND {ic}"));
        }
    }
    out.push_str("\n;\n\n");
    out
}

/// Parts needed to format a CREATE MATERIALIZED VIEW DDL string.
struct MvDdlParts<'a> {
    keyspace: &'a str,
    view_name: &'a str,
    base_table: &'a str,
    include_all: bool,
    select_columns: &'a [String],
    where_clause: &'a str,
    partition_keys: &'a [String],            // sorted by position
    clustering_keys: &'a [(String, String)], // (name, order), sorted by position
    properties: &'a std::collections::BTreeMap<String, String>,
}

/// Format a CREATE MATERIALIZED VIEW DDL string.
fn format_create_mv_ddl(parts: &MvDdlParts<'_>) -> String {
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!(
        "CREATE MATERIALIZED VIEW {}.{} AS\n",
        quote_if_needed(parts.keyspace),
        quote_if_needed(parts.view_name)
    ));

    let columns_str = if parts.include_all {
        "*".to_string()
    } else {
        parts
            .select_columns
            .iter()
            .map(|c| quote_if_needed(c))
            .collect::<Vec<_>>()
            .join(", ")
    };
    out.push_str(&format!("    SELECT {columns_str}\n"));
    out.push_str(&format!(
        "    FROM {}.{}\n",
        quote_if_needed(parts.keyspace),
        quote_if_needed(parts.base_table)
    ));
    out.push_str(&format!("    WHERE {}\n", parts.where_clause));

    let pk_str = if parts.partition_keys.len() == 1 {
        quote_if_needed(&parts.partition_keys[0])
    } else {
        format!(
            "({})",
            parts
                .partition_keys
                .iter()
                .map(|k| quote_if_needed(k))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    if parts.clustering_keys.is_empty() {
        out.push_str(&format!("    PRIMARY KEY ({pk_str})\n"));
    } else {
        let ck_str = parts
            .clustering_keys
            .iter()
            .map(|(name, _)| quote_if_needed(name))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("    PRIMARY KEY ({pk_str}, {ck_str})\n"));
    }

    // Python cqlsh always emits CLUSTERING ORDER BY for MVs (even when all ASC)
    let mut first_with = true;
    if !parts.clustering_keys.is_empty() {
        let order_str = parts
            .clustering_keys
            .iter()
            .map(|(name, order)| format!("{} {}", quote_if_needed(name), order.to_uppercase()))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("    WITH CLUSTERING ORDER BY ({order_str})"));
        first_with = false;
    }

    let prop_order = [
        "bloom_filter_fp_chance",
        "caching",
        "comment",
        "compaction",
        "compression",
        "crc_check_chance",
        "default_time_to_live",
        "gc_grace_seconds",
        "max_index_interval",
        "memtable_flush_period_in_ms",
        "min_index_interval",
        "speculative_retry",
    ];

    for prop_name in &prop_order {
        if let Some(value) = parts.properties.get(*prop_name) {
            let formatted_value = format_property_value(prop_name, value);
            if first_with {
                out.push_str(&format!("    WITH {} = {}", prop_name, formatted_value));
                first_with = false;
            } else {
                out.push_str(&format!("\n    AND {} = {}", prop_name, formatted_value));
            }
        }
    }

    out.push_str(";\n");

    out.push('\n');
    out
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
    fn parse_list_value_test() {
        assert_eq!(parse_list_value("[]"), Vec::<String>::new());
        assert_eq!(parse_list_value(""), Vec::<String>::new());
        assert_eq!(parse_list_value("['a', 'b', 'c']"), vec!["a", "b", "c"]);
        assert_eq!(
            parse_list_value("[int, text, uuid]"),
            vec!["int", "text", "uuid"]
        );
    }

    #[test]
    fn extract_map_value_test() {
        assert_eq!(
            extract_map_value("{'target': 'email', 'class_name': 'foo'}", "target"),
            Some("email".to_string())
        );
        assert_eq!(extract_map_value("{'target': 'email'}", "missing"), None);
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
            clustering_order: vec![],
            properties: std::collections::BTreeMap::new(),
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
            clustering_order: vec!["ASC".to_string()],
            properties: std::collections::BTreeMap::new(),
        };

        let mut buf = Vec::new();
        write_create_table(&mut buf, &meta).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("PRIMARY KEY (user_id, event_time)"));
        assert!(
            output.contains("WITH CLUSTERING ORDER BY (event_time ASC)"),
            "expected CLUSTERING ORDER BY: {output}"
        );
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
            clustering_order: vec!["ASC".to_string()],
            properties: std::collections::BTreeMap::new(),
        };

        let mut buf = Vec::new();
        write_create_table(&mut buf, &meta).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("PRIMARY KEY ((host, metric), ts)"));
    }

    // --- DDL formatter tests ---

    #[test]
    fn format_index_ddl_simple() {
        let ddl = format_index_ddl("my_ks", "email_idx", "users", "email");
        assert!(ddl.contains("CREATE INDEX email_idx ON my_ks.users (email);"));
    }

    #[test]
    fn format_index_ddl_quoted_names() {
        let ddl = format_index_ddl("MyKs", "MyIdx", "MyTable", "email");
        assert!(ddl.contains("\"MyKs\""));
        assert!(ddl.contains("\"MyIdx\""));
        assert!(ddl.contains("\"MyTable\""));
    }

    #[test]
    fn format_create_type_ddl_single_field() {
        let fields = vec![("street".to_string(), "text".to_string())];
        let ddl = format_create_type_ddl("ks1", "address", &fields);
        assert!(ddl.contains("CREATE TYPE ks1.address ("));
        assert!(ddl.contains("street text"));
        assert!(ddl.contains(");"));
    }

    #[test]
    fn format_create_type_ddl_multiple_fields() {
        let fields = vec![
            ("street".to_string(), "text".to_string()),
            ("city".to_string(), "text".to_string()),
            ("zip".to_string(), "int".to_string()),
        ];
        let ddl = format_create_type_ddl("ks1", "address", &fields);
        assert!(
            ddl.contains("street text,"),
            "expected trailing comma: {ddl}"
        );
        assert!(ddl.contains("city text,"), "expected trailing comma: {ddl}");
        // last field has no trailing comma
        assert!(
            !ddl.contains("int,"),
            "last field should not have comma: {ddl}"
        );
    }

    #[test]
    fn format_create_function_ddl_called_on_null() {
        let ddl = format_create_function_ddl(
            "ks1",
            "add_one",
            "val int",
            "CALLED ON NULL INPUT",
            "int",
            "java",
            "return val + 1;",
        );
        assert!(ddl.contains("CREATE OR REPLACE FUNCTION ks1.add_one (val int)"));
        assert!(ddl.contains("CALLED ON NULL INPUT"));
        assert!(ddl.contains("RETURNS int"));
        assert!(ddl.contains("LANGUAGE java"));
        assert!(ddl.contains("AS $$ return val + 1; $$;"));
    }

    #[test]
    fn format_create_function_ddl_returns_null() {
        let ddl = format_create_function_ddl(
            "ks1",
            "my_func",
            "x text",
            "RETURNS NULL ON NULL INPUT",
            "text",
            "lua",
            "return x",
        );
        assert!(ddl.contains("RETURNS NULL ON NULL INPUT"));
        assert!(!ddl.contains("CALLED ON NULL INPUT"));
    }

    #[test]
    fn format_create_aggregate_ddl_minimal() {
        let ddl =
            format_create_aggregate_ddl("ks1", "my_sum", "int", "state_add", "int", None, None);
        assert!(ddl.contains("CREATE OR REPLACE AGGREGATE ks1.my_sum (int)"));
        assert!(ddl.contains("SFUNC state_add"));
        assert!(ddl.contains("STYPE int"));
        assert!(!ddl.contains("FINALFUNC"));
        assert!(!ddl.contains("INITCOND"));
        assert!(ddl.contains(';'));
    }

    #[test]
    fn format_create_aggregate_ddl_with_optional() {
        let ddl = format_create_aggregate_ddl(
            "ks1",
            "my_avg",
            "int",
            "state_avg",
            "tuple<int,int>",
            Some("final_avg"),
            Some("0"),
        );
        assert!(ddl.contains("FINALFUNC final_avg"));
        assert!(ddl.contains("INITCOND 0"));
    }

    #[test]
    fn format_create_aggregate_ddl_empty_optional_skipped() {
        let ddl = format_create_aggregate_ddl(
            "ks1",
            "my_agg",
            "int",
            "sf",
            "int",
            Some(""),
            Some("null"),
        );
        assert!(
            !ddl.contains("FINALFUNC"),
            "empty FINALFUNC should be omitted: {ddl}"
        );
        assert!(
            !ddl.contains("INITCOND"),
            "'null' INITCOND should be omitted: {ddl}"
        );
    }

    #[test]
    fn format_create_mv_ddl_simple() {
        let cols = vec!["id".to_string(), "email".to_string()];
        let properties = std::collections::BTreeMap::new();
        let parts = MvDdlParts {
            keyspace: "ks1",
            view_name: "user_by_email",
            base_table: "users",
            include_all: false,
            select_columns: &cols,
            where_clause: "email IS NOT NULL",
            partition_keys: &["email".to_string()],
            clustering_keys: &[],
            properties: &properties,
        };
        let ddl = format_create_mv_ddl(&parts);
        assert!(ddl.contains("CREATE MATERIALIZED VIEW ks1.user_by_email AS"));
        assert!(ddl.contains("SELECT id, email"));
        assert!(ddl.contains("FROM ks1.users"));
        assert!(ddl.contains("WHERE email IS NOT NULL"));
        assert!(ddl.contains("PRIMARY KEY (email)"));
    }

    #[test]
    fn format_create_mv_ddl_include_all() {
        let properties = std::collections::BTreeMap::new();
        let parts = MvDdlParts {
            keyspace: "ks1",
            view_name: "mv_all",
            base_table: "base",
            include_all: true,
            select_columns: &["id".to_string()],
            where_clause: "id IS NOT NULL",
            partition_keys: &["id".to_string()],
            clustering_keys: &[],
            properties: &properties,
        };
        let ddl = format_create_mv_ddl(&parts);
        assert!(
            ddl.contains("SELECT *"),
            "include_all should emit SELECT *: {ddl}"
        );
    }

    #[test]
    fn format_create_mv_ddl_with_clustering_desc() {
        let cols = vec!["user_id".to_string(), "ts".to_string()];
        let ck = vec![("ts".to_string(), "DESC".to_string())];
        let properties = std::collections::BTreeMap::new();
        let parts = MvDdlParts {
            keyspace: "ks1",
            view_name: "mv_ordered",
            base_table: "events",
            include_all: false,
            select_columns: &cols,
            where_clause: "ts IS NOT NULL",
            partition_keys: &["user_id".to_string()],
            clustering_keys: &ck,
            properties: &properties,
        };
        let ddl = format_create_mv_ddl(&parts);
        assert!(ddl.contains("PRIMARY KEY (user_id, ts)"));
        assert!(ddl.contains("WITH CLUSTERING ORDER BY (ts DESC)"));
    }

    #[test]
    fn format_create_mv_ddl_with_properties() {
        let cols = vec!["state".to_string(), "username".to_string()];
        let ck = vec![("username".to_string(), "ASC".to_string())];
        let mut properties = std::collections::BTreeMap::new();
        properties.insert("bloom_filter_fp_chance".to_string(), "0.01".to_string());
        properties.insert("comment".to_string(), "".to_string());
        properties.insert("gc_grace_seconds".to_string(), "864000".to_string());
        let parts = MvDdlParts {
            keyspace: "test",
            view_name: "users_by_state",
            base_table: "users",
            include_all: true,
            select_columns: &cols,
            where_clause: "state IS NOT null AND username IS NOT null",
            partition_keys: &["state".to_string()],
            clustering_keys: &ck,
            properties: &properties,
        };
        let ddl = format_create_mv_ddl(&parts);
        assert!(
            ddl.contains("WITH CLUSTERING ORDER BY (username ASC)"),
            "should always emit CLUSTERING ORDER BY for MVs: {ddl}"
        );
        assert!(ddl.contains("AND bloom_filter_fp_chance = 0.01"));
        assert!(ddl.contains("AND comment = ''"));
        assert!(ddl.contains("AND gc_grace_seconds = 864000"));
    }
}

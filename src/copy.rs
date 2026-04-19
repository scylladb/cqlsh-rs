//! COPY TO and COPY FROM implementation — exports/imports CSV data.
//!
//! Supports:
//!   `COPY [ks.]table [(col1, col2, ...)] TO 'filename'|STDOUT [WITH options...]`
//!   `COPY [ks.]table [(col1, col2, ...)] FROM 'filename'|STDIN [WITH options...]`

use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use futures::StreamExt;

use crate::driver::types::CqlValue;
use crate::driver::PreparedId;
use crate::session::CqlSession;

/// Where to write/read the CSV data.
#[derive(Debug, Clone, PartialEq)]
pub enum CopyTarget {
    /// Write to / read from a file at the given path.
    File(PathBuf),
    /// Write to standard output.
    Stdout,
    /// Read from standard input.
    Stdin,
}

/// All options controlling CSV export behavior.
#[derive(Debug, Clone)]
pub struct CopyOptions {
    pub delimiter: char,
    pub quote: char,
    pub escape: char,
    pub header: bool,
    pub null_val: String,
    pub datetime_format: Option<String>,
    pub encoding: String,
    pub float_precision: usize,
    pub double_precision: usize,
    pub decimal_sep: char,
    pub thousands_sep: Option<char>,
    pub bool_style: (String, String),
    pub page_size: usize,
    pub max_output_size: Option<usize>,
    pub report_frequency: Option<usize>,
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            delimiter: ',',
            quote: '"',
            escape: '\\',
            header: false,
            null_val: String::new(),
            datetime_format: None,
            encoding: "utf-8".to_string(),
            float_precision: 5,
            double_precision: 12,
            decimal_sep: '.',
            thousands_sep: None,
            bool_style: ("True".to_string(), "False".to_string()),
            page_size: 1000,
            max_output_size: None,
            report_frequency: None,
        }
    }
}

/// A parsed COPY TO command.
#[derive(Debug, Clone)]
pub struct CopyToCommand {
    pub keyspace: Option<String>,
    pub table: String,
    pub columns: Option<Vec<String>>,
    pub filename: CopyTarget,
    pub options: CopyOptions,
}

/// Parse a `COPY ... TO ...` statement.
///
/// Format: `COPY [ks.]table [(col1, col2)] TO 'filename'|STDOUT [WITH opt=val AND ...]`
pub fn parse_copy_to(input: &str) -> Result<CopyToCommand> {
    let trimmed = input.trim().trim_end_matches(';').trim();

    // Must start with COPY (case-insensitive)
    let upper = trimmed.to_uppercase();
    if !upper.starts_with("COPY ") {
        bail!("not a COPY statement");
    }

    // Find the TO keyword (case-insensitive), but not inside parentheses
    let to_pos =
        find_keyword_outside_parens(trimmed, "TO").context("COPY statement missing TO keyword")?;

    let before_to = trimmed[4..to_pos].trim(); // skip "COPY"
    let after_to = trimmed[to_pos + 2..].trim(); // skip "TO"

    // Parse table spec (before TO): [ks.]table [(col1, col2)]
    let (keyspace, table, columns) = parse_table_spec(before_to)?;

    // Parse target and options (after TO): 'filename'|STDOUT [WITH ...]
    let (filename, options_str) = parse_target_and_options(after_to)?;

    let options = if let Some(opts) = options_str {
        parse_options(&opts)?
    } else {
        CopyOptions::default()
    };

    Ok(CopyToCommand {
        keyspace,
        table,
        columns,
        filename,
        options,
    })
}

/// Execute a COPY TO command, writing results as CSV.
pub async fn execute_copy_to(
    session: &CqlSession,
    cmd: &CopyToCommand,
    current_keyspace: Option<&str>,
) -> Result<()> {
    // Build SELECT query
    let col_spec = match &cmd.columns {
        Some(cols) => cols.join(", "),
        None => "*".to_string(),
    };

    let table_spec = match (&cmd.keyspace, current_keyspace) {
        (Some(ks), _) => format!("{}.{}", ks, cmd.table),
        (None, Some(ks)) => format!("{}.{}", ks, cmd.table),
        (None, None) => cmd.table.clone(),
    };

    let query = format!("SELECT {} FROM {}", col_spec, table_spec);

    let result = session.execute_query(&query).await?;

    // Set up CSV writer
    let mut row_count: usize = 0;

    match &cmd.filename {
        CopyTarget::File(path) => {
            let file = std::fs::File::create(path)
                .with_context(|| format!("failed to create file: {}", path.display()))?;
            let buf = std::io::BufWriter::new(file);
            let mut wtr = build_csv_writer(&cmd.options, buf);

            if cmd.options.header {
                let headers: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
                wtr.write_record(&headers)?;
            }

            for row in &result.rows {
                if let Some(max) = cmd.options.max_output_size {
                    if row_count >= max {
                        break;
                    }
                }
                let fields: Vec<String> = row
                    .values
                    .iter()
                    .map(|v| format_value_for_csv(v, &cmd.options))
                    .collect();
                wtr.write_record(&fields)?;
                row_count += 1;

                if let Some(freq) = cmd.options.report_frequency {
                    if freq > 0 && row_count.is_multiple_of(freq) {
                        eprintln!("Processed {} rows...", row_count);
                    }
                }
            }

            wtr.flush()?;
            println!("{} rows exported to '{}'.", row_count, path.display());
        }
        CopyTarget::Stdout => {
            let stdout = std::io::stdout();
            let handle = stdout.lock();
            let mut wtr = build_csv_writer(&cmd.options, handle);

            if cmd.options.header {
                let headers: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
                wtr.write_record(&headers)?;
            }

            for row in &result.rows {
                if let Some(max) = cmd.options.max_output_size {
                    if row_count >= max {
                        break;
                    }
                }
                let fields: Vec<String> = row
                    .values
                    .iter()
                    .map(|v| format_value_for_csv(v, &cmd.options))
                    .collect();
                wtr.write_record(&fields)?;
                row_count += 1;

                if let Some(freq) = cmd.options.report_frequency {
                    if freq > 0 && row_count.is_multiple_of(freq) {
                        eprintln!("Processed {} rows...", row_count);
                    }
                }
            }

            wtr.flush()?;
            eprintln!("{} rows exported to STDOUT.", row_count);
        }
        CopyTarget::Stdin => {
            bail!("COPY TO cannot write to STDIN");
        }
    }

    Ok(())
}

/// Format a single CQL value for CSV output according to the given options.
pub fn format_value_for_csv(value: &CqlValue, options: &CopyOptions) -> String {
    match value {
        CqlValue::Null | CqlValue::Unset => options.null_val.clone(),
        CqlValue::Text(s) | CqlValue::Ascii(s) => s.clone(),
        CqlValue::Boolean(b) => {
            if *b {
                options.bool_style.0.clone()
            } else {
                options.bool_style.1.clone()
            }
        }
        CqlValue::Int(v) => v.to_string(),
        CqlValue::BigInt(v) => v.to_string(),
        CqlValue::SmallInt(v) => v.to_string(),
        CqlValue::TinyInt(v) => v.to_string(),
        CqlValue::Counter(v) => v.to_string(),
        CqlValue::Varint(v) => v.to_string(),
        CqlValue::Float(v) => format_float(*v as f64, options.float_precision, options),
        CqlValue::Double(v) => format_float(*v, options.double_precision, options),
        CqlValue::Decimal(v) => {
            let s = v.to_string();
            if options.decimal_sep != '.' {
                s.replace('.', &options.decimal_sep.to_string())
            } else {
                s
            }
        }
        CqlValue::Timestamp(millis) => format_timestamp(*millis, options),
        CqlValue::Uuid(u) | CqlValue::TimeUuid(u) => u.to_string(),
        CqlValue::Blob(bytes) => {
            let mut s = String::with_capacity(2 + bytes.len() * 2);
            s.push_str("0x");
            for b in bytes {
                s.push_str(&format!("{b:02x}"));
            }
            s
        }
        CqlValue::Date(d) => d.to_string(),
        CqlValue::Time(t) => t.to_string(),
        CqlValue::Duration {
            months,
            days,
            nanoseconds,
        } => format!("{months}mo{days}d{nanoseconds}ns"),
        CqlValue::Inet(addr) => addr.to_string(),
        // Collection types: use CQL literal format via Display
        CqlValue::List(_)
        | CqlValue::Set(_)
        | CqlValue::Map(_)
        | CqlValue::Tuple(_)
        | CqlValue::UserDefinedType { .. } => value.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build a csv::Writer with the configured options.
fn build_csv_writer<W: Write>(options: &CopyOptions, writer: W) -> csv::Writer<W> {
    csv::WriterBuilder::new()
        .delimiter(options.delimiter as u8)
        .quote(options.quote as u8)
        .escape(options.escape as u8)
        .double_quote(false)
        .from_writer(writer)
}

/// Format a floating-point number with given precision and decimal separator.
fn format_float(v: f64, precision: usize, options: &CopyOptions) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    let s = format!("{v:.prec$}", prec = precision);
    if options.decimal_sep != '.' {
        s.replace('.', &options.decimal_sep.to_string())
    } else {
        s
    }
}

/// Format a timestamp (millis since epoch) using the configured format.
fn format_timestamp(millis: i64, options: &CopyOptions) -> String {
    match DateTime::from_timestamp_millis(millis) {
        Some(dt) => {
            let utc: DateTime<Utc> = dt;
            match &options.datetime_format {
                Some(fmt) => utc.format(fmt).to_string(),
                None => utc.format("%Y-%m-%d %H:%M:%S%.3f%z").to_string(),
            }
        }
        None => format!("<invalid timestamp: {millis}>"),
    }
}

/// Find a keyword in the string that is not inside parentheses.
/// Returns the byte offset of the keyword start.
fn find_keyword_outside_parens(s: &str, keyword: &str) -> Option<usize> {
    let upper = s.to_uppercase();
    let kw_upper = keyword.to_uppercase();
    let kw_len = kw_upper.len();
    let mut depth: i32 = 0;
    let mut in_quote = false;
    let mut quote_char: char = '\'';
    let bytes = s.as_bytes();

    for (i, ch) in s.char_indices() {
        if in_quote {
            if ch == quote_char {
                in_quote = false;
            }
            continue;
        }
        match ch {
            '\'' | '"' => {
                in_quote = true;
                quote_char = ch;
            }
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if depth == 0 && !in_quote {
            // Check if keyword matches at this position, surrounded by word boundaries
            if i + kw_len <= upper.len() && upper[i..i + kw_len] == *kw_upper {
                // Check word boundaries
                let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
                let after_ok = i + kw_len >= s.len() || !bytes[i + kw_len].is_ascii_alphanumeric();
                if before_ok && after_ok {
                    return Some(i);
                }
            }
        }
    }
    None
}

/// Parse the table spec: `[ks.]table [(col1, col2, ...)]`
fn parse_table_spec(spec: &str) -> Result<(Option<String>, String, Option<Vec<String>>)> {
    let spec = spec.trim();

    // Split off column list if present
    let (table_part, columns) = if let Some(paren_start) = spec.find('(') {
        let paren_end = spec
            .rfind(')')
            .context("unmatched parenthesis in column list")?;
        let cols_str = &spec[paren_start + 1..paren_end];
        let cols: Vec<String> = cols_str
            .split(',')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        (spec[..paren_start].trim(), Some(cols))
    } else {
        (spec, None)
    };

    // Split keyspace.table
    let (keyspace, table) = if let Some(dot_pos) = table_part.find('.') {
        let ks = table_part[..dot_pos].trim().to_string();
        let tbl = table_part[dot_pos + 1..].trim().to_string();
        (Some(ks), tbl)
    } else {
        (None, table_part.trim().to_string())
    };

    if table.is_empty() {
        bail!("missing table name in COPY statement");
    }

    Ok((keyspace, table, columns))
}

/// Parse the target and WITH options after the TO keyword.
/// Returns `(CopyTarget, Option<options_string>)`.
fn parse_target_and_options(after_to: &str) -> Result<(CopyTarget, Option<String>)> {
    let after_to = after_to.trim();

    // Find WITH keyword (case-insensitive) outside of quotes
    let with_pos = find_keyword_outside_parens(after_to, "WITH");

    let (target_str, options_str) = match with_pos {
        Some(pos) => {
            let target = after_to[..pos].trim();
            let opts = after_to[pos + 4..].trim(); // skip "WITH"
            (target, Some(opts.to_string()))
        }
        None => (after_to, None),
    };

    let target_str = target_str.trim();

    let target = if target_str.eq_ignore_ascii_case("STDOUT") {
        CopyTarget::Stdout
    } else {
        // Strip surrounding quotes (single quotes)
        let path_str = if (target_str.starts_with('\'') && target_str.ends_with('\''))
            || (target_str.starts_with('"') && target_str.ends_with('"'))
        {
            &target_str[1..target_str.len() - 1]
        } else {
            target_str
        };
        CopyTarget::File(PathBuf::from(path_str))
    };

    Ok((target, options_str))
}

/// Parse `opt1=val1 AND opt2=val2 ...` pairs into `CopyOptions`.
fn parse_options(options_str: &str) -> Result<CopyOptions> {
    let mut opts = CopyOptions::default();

    // Split on AND (case-insensitive)
    let parts = split_on_and(options_str);

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let eq_pos = part
            .find('=')
            .with_context(|| format!("invalid option (missing '='): {part}"))?;
        let key = part[..eq_pos].trim().to_uppercase();
        let val = unquote(part[eq_pos + 1..].trim());

        match key.as_str() {
            "DELIMITER" => {
                opts.delimiter = val
                    .chars()
                    .next()
                    .context("DELIMITER must be a single character")?;
            }
            "QUOTE" => {
                opts.quote = val
                    .chars()
                    .next()
                    .context("QUOTE must be a single character")?;
            }
            "ESCAPE" => {
                opts.escape = val
                    .chars()
                    .next()
                    .context("ESCAPE must be a single character")?;
            }
            "HEADER" => {
                opts.header = parse_bool_option(&val)?;
            }
            "NULL" | "NULLVAL" => {
                opts.null_val = val;
            }
            "DATETIMEFORMAT" => {
                opts.datetime_format = if val.is_empty() { None } else { Some(val) };
            }
            "ENCODING" => {
                opts.encoding = val;
            }
            "FLOATPRECISION" => {
                opts.float_precision = val.parse().context("FLOATPRECISION must be an integer")?;
            }
            "DOUBLEPRECISION" => {
                opts.double_precision =
                    val.parse().context("DOUBLEPRECISION must be an integer")?;
            }
            "DECIMALSEP" => {
                opts.decimal_sep = val
                    .chars()
                    .next()
                    .context("DECIMALSEP must be a single character")?;
            }
            "THOUSANDSSEP" => {
                opts.thousands_sep = val.chars().next();
            }
            "BOOLSTYLE" => {
                // Format: "True:False"
                let parts: Vec<&str> = val.splitn(2, ':').collect();
                if parts.len() == 2 {
                    opts.bool_style = (parts[0].to_string(), parts[1].to_string());
                } else {
                    bail!("BOOLSTYLE must be in format 'TrueVal:FalseVal'");
                }
            }
            "PAGESIZE" => {
                opts.page_size = val.parse().context("PAGESIZE must be an integer")?;
            }
            "MAXOUTPUTSIZE" => {
                let n: usize = val.parse().context("MAXOUTPUTSIZE must be an integer")?;
                opts.max_output_size = Some(n);
            }
            "REPORTFREQUENCY" => {
                let n: usize = val.parse().context("REPORTFREQUENCY must be an integer")?;
                opts.report_frequency = if n == 0 { None } else { Some(n) };
            }
            other => {
                bail!("unknown COPY option: {other}");
            }
        }
    }

    Ok(opts)
}

/// Split a string on `AND` keywords (case-insensitive), not inside quotes.
fn split_on_and(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let upper = s.to_uppercase();
    let chars: Vec<char> = s.chars().collect();
    let upper_chars: Vec<char> = upper.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_quote = false;
    let mut quote_char = '\'';

    while i < len {
        if in_quote {
            if chars[i] == quote_char {
                in_quote = false;
            }
            current.push(chars[i]);
            i += 1;
            continue;
        }

        if chars[i] == '\'' || chars[i] == '"' {
            in_quote = true;
            quote_char = chars[i];
            current.push(chars[i]);
            i += 1;
            continue;
        }

        // Check for " AND " pattern
        if i + 5 <= len
            && (i == 0 || chars[i].is_whitespace())
            && upper_chars[i..].iter().collect::<String>().starts_with(
                if chars[i].is_whitespace() {
                    " AND "
                } else {
                    "AND "
                },
            )
        {
            // More precise check
            let remaining: String = upper_chars[i..].iter().collect();
            if remaining.starts_with(" AND ") {
                parts.push(current.clone());
                current.clear();
                i += 5; // skip " AND "
                continue;
            }
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

/// Remove surrounding single or double quotes from a value.
fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2
        && ((s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')))
    {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

/// Parse a boolean option value (true/false, yes/no, 1/0).
fn parse_bool_option(val: &str) -> Result<bool> {
    match val.to_lowercase().as_str() {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => bail!("invalid boolean value: {val}"),
    }
}

// ===========================================================================
// COPY FROM implementation — imports CSV data into a table.
// ===========================================================================

/// Options specific to COPY FROM (CSV import).
#[derive(Debug, Clone)]
pub struct CopyFromOptions {
    // Shared format options
    pub delimiter: char,
    pub quote: char,
    pub escape: char,
    pub header: bool,
    pub null_val: String,
    pub datetime_format: Option<String>,
    pub encoding: String,
    // COPY FROM specific
    pub chunk_size: usize,
    pub max_batch_size: usize,
    pub min_batch_size: usize,
    pub prepared_statements: bool,
    pub ttl: Option<u64>,
    pub max_attempts: usize,
    pub max_parse_errors: Option<usize>,
    pub max_insert_errors: Option<usize>,
    pub err_file: Option<PathBuf>,
    pub report_frequency: Option<usize>,
    pub ingest_rate: Option<usize>,
    pub num_processes: usize,
}

impl Default for CopyFromOptions {
    fn default() -> Self {
        Self {
            delimiter: ',',
            quote: '"',
            escape: '\\',
            header: false,
            null_val: String::new(),
            datetime_format: None,
            encoding: "utf-8".to_string(),
            chunk_size: 5000,
            max_batch_size: 20,
            min_batch_size: 2,
            prepared_statements: true,
            ttl: None,
            max_attempts: 5,
            max_parse_errors: None,
            max_insert_errors: None,
            err_file: None,
            report_frequency: None,
            ingest_rate: None,
            num_processes: 1,
        }
    }
}

/// A parsed COPY FROM command.
#[derive(Debug, Clone)]
pub struct CopyFromCommand {
    pub keyspace: Option<String>,
    pub table: String,
    pub columns: Option<Vec<String>>,
    pub source: CopyTarget,
    pub options: CopyFromOptions,
}

/// Parse a `COPY ... FROM ...` statement.
///
/// Format: `COPY [ks.]table [(col1, col2)] FROM 'filename'|STDIN [WITH opt=val AND ...]`
pub fn parse_copy_from(input: &str) -> Result<CopyFromCommand> {
    let trimmed = input.trim().trim_end_matches(';').trim();

    let upper = trimmed.to_uppercase();
    if !upper.starts_with("COPY ") {
        bail!("not a COPY statement");
    }

    // Find the FROM keyword (case-insensitive), but not inside parentheses
    let from_pos = find_keyword_outside_parens(trimmed, "FROM")
        .context("COPY statement missing FROM keyword")?;

    let before_from = trimmed[4..from_pos].trim(); // skip "COPY"
    let after_from = trimmed[from_pos + 4..].trim(); // skip "FROM"

    // Parse table spec (before FROM): [ks.]table [(col1, col2)]
    let (keyspace, table, columns) = parse_table_spec(before_from)?;

    // Parse source and options (after FROM): 'filename'|STDIN [WITH ...]
    let (source, options_str) = parse_source_and_options(after_from)?;

    let options = if let Some(opts) = options_str {
        parse_copy_from_options(&opts)?
    } else {
        CopyFromOptions::default()
    };

    Ok(CopyFromCommand {
        keyspace,
        table,
        columns,
        source,
        options,
    })
}

/// Parse the source and WITH options after the FROM keyword.
/// Returns `(CopyTarget, Option<options_string>)`.
fn parse_source_and_options(after_from: &str) -> Result<(CopyTarget, Option<String>)> {
    let after_from = after_from.trim();

    let with_pos = find_keyword_outside_parens(after_from, "WITH");

    let (source_str, options_str) = match with_pos {
        Some(pos) => {
            let source = after_from[..pos].trim();
            let opts = after_from[pos + 4..].trim();
            (source, Some(opts.to_string()))
        }
        None => (after_from, None),
    };

    let source_str = source_str.trim();

    let source = if source_str.eq_ignore_ascii_case("STDIN") {
        CopyTarget::Stdin
    } else {
        let path_str = if (source_str.starts_with('\'') && source_str.ends_with('\''))
            || (source_str.starts_with('"') && source_str.ends_with('"'))
        {
            &source_str[1..source_str.len() - 1]
        } else {
            source_str
        };
        CopyTarget::File(PathBuf::from(path_str))
    };

    Ok((source, options_str))
}

/// Parse `opt1=val1 AND opt2=val2 ...` pairs into `CopyFromOptions`.
fn parse_copy_from_options(options_str: &str) -> Result<CopyFromOptions> {
    let mut opts = CopyFromOptions::default();

    let parts = split_on_and(options_str);

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let eq_pos = part
            .find('=')
            .with_context(|| format!("invalid option (missing '='): {part}"))?;
        let key = part[..eq_pos].trim().to_uppercase();
        let val = unquote(part[eq_pos + 1..].trim());

        match key.as_str() {
            "DELIMITER" => {
                opts.delimiter = val
                    .chars()
                    .next()
                    .context("DELIMITER must be a single character")?;
            }
            "QUOTE" => {
                opts.quote = val
                    .chars()
                    .next()
                    .context("QUOTE must be a single character")?;
            }
            "ESCAPE" => {
                opts.escape = val
                    .chars()
                    .next()
                    .context("ESCAPE must be a single character")?;
            }
            "HEADER" => {
                opts.header = parse_bool_option(&val)?;
            }
            "NULL" | "NULLVAL" => {
                opts.null_val = val;
            }
            "DATETIMEFORMAT" => {
                opts.datetime_format = if val.is_empty() { None } else { Some(val) };
            }
            "ENCODING" => {
                opts.encoding = val;
            }
            "CHUNKSIZE" => {
                opts.chunk_size = val.parse().context("CHUNKSIZE must be an integer")?;
            }
            "MAXBATCHSIZE" => {
                opts.max_batch_size = val.parse().context("MAXBATCHSIZE must be an integer")?;
            }
            "MINBATCHSIZE" => {
                opts.min_batch_size = val.parse().context("MINBATCHSIZE must be an integer")?;
            }
            "PREPAREDSTATEMENTS" => {
                opts.prepared_statements = parse_bool_option(&val)?;
            }
            "TTL" => {
                let n: u64 = val.parse().context("TTL must be a positive integer")?;
                opts.ttl = Some(n);
            }
            "MAXATTEMPTS" => {
                opts.max_attempts = val.parse().context("MAXATTEMPTS must be an integer")?;
            }
            "MAXPARSEERRORS" => {
                if val == "-1" {
                    opts.max_parse_errors = None;
                } else {
                    let n: usize = val.parse().context("MAXPARSEERRORS must be an integer")?;
                    opts.max_parse_errors = Some(n);
                }
            }
            "MAXINSERTERRORS" => {
                if val == "-1" {
                    opts.max_insert_errors = None;
                } else {
                    let n: usize = val.parse().context("MAXINSERTERRORS must be an integer")?;
                    opts.max_insert_errors = Some(n);
                }
            }
            "ERRFILE" | "ERRORSFILE" => {
                opts.err_file = if val.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(val))
                };
            }
            "REPORTFREQUENCY" => {
                let n: usize = val.parse().context("REPORTFREQUENCY must be an integer")?;
                opts.report_frequency = if n == 0 { None } else { Some(n) };
            }
            "INGESTRATE" => {
                if val == "-1" || val == "0" {
                    opts.ingest_rate = None;
                } else {
                    let n: usize = val.parse().context("INGESTRATE must be an integer")?;
                    opts.ingest_rate = Some(n);
                }
            }
            "NUMPROCESSES" => {
                let n: usize = val.parse().context("NUMPROCESSES must be an integer")?;
                opts.num_processes = n.max(1);
            }
            other => {
                bail!("unknown COPY FROM option: {other}");
            }
        }
    }

    Ok(opts)
}

// ---------------------------------------------------------------------------
// Type-aware CSV ↔ CQL conversion
// ---------------------------------------------------------------------------

/// Convert a CSV string field to a typed `CqlValue` based on the CQL column type.
///
/// `type_name` is the raw CQL type string from `system_schema.columns.type`
/// (e.g. `"int"`, `"text"`, `"list<int>"`, `"frozen<set<uuid>>"`).
/// Complex nested types (list, set, map, tuple, frozen, udt) are preserved as
/// `CqlValue::Text` literals — the database will parse them via the unprepared path.
pub fn csv_str_to_cql_value(field: &str, type_name: &str, null_val: &str) -> Result<CqlValue> {
    // Null check (exact match with configured null_val, or empty string when null_val is empty)
    if field == null_val || (null_val.is_empty() && field.is_empty()) {
        return Ok(CqlValue::Null);
    }

    let base_type = strip_frozen(type_name).to_lowercase();
    let base_type = base_type.as_str();

    match base_type {
        "ascii" => Ok(CqlValue::Ascii(field.to_string())),
        "text" | "varchar" => Ok(CqlValue::Text(field.to_string())),
        "boolean" => {
            let b = match field.to_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => true,
                "false" | "no" | "off" | "0" => false,
                _ => bail!("invalid boolean value: {field:?}"),
            };
            Ok(CqlValue::Boolean(b))
        }
        "int" => Ok(CqlValue::Int(
            field
                .parse::<i32>()
                .with_context(|| format!("invalid int: {field:?}"))?,
        )),
        "bigint" | "counter" => Ok(CqlValue::BigInt(
            field
                .parse::<i64>()
                .with_context(|| format!("invalid bigint: {field:?}"))?,
        )),
        "smallint" => Ok(CqlValue::SmallInt(
            field
                .parse::<i16>()
                .with_context(|| format!("invalid smallint: {field:?}"))?,
        )),
        "tinyint" => Ok(CqlValue::TinyInt(
            field
                .parse::<i8>()
                .with_context(|| format!("invalid tinyint: {field:?}"))?,
        )),
        "float" => Ok(CqlValue::Float(
            field
                .parse::<f32>()
                .with_context(|| format!("invalid float: {field:?}"))?,
        )),
        "double" => Ok(CqlValue::Double(
            field
                .parse::<f64>()
                .with_context(|| format!("invalid double: {field:?}"))?,
        )),
        "uuid" => {
            let u =
                uuid::Uuid::parse_str(field).with_context(|| format!("invalid uuid: {field:?}"))?;
            Ok(CqlValue::Uuid(u))
        }
        "timeuuid" => {
            let u = uuid::Uuid::parse_str(field)
                .with_context(|| format!("invalid timeuuid: {field:?}"))?;
            Ok(CqlValue::TimeUuid(u))
        }
        "timestamp" => {
            // Try RFC 3339 first (handles 'Z' and offset formats)
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(field) {
                return Ok(CqlValue::Timestamp(dt.timestamp_millis()));
            }
            // Try space-separated ISO-8601 with numeric offset
            let formats = [
                "%Y-%m-%d %H:%M:%S%.f%z",
                "%Y-%m-%dT%H:%M:%S%.f%z",
                "%Y-%m-%dT%H:%M:%S%z",
                "%Y-%m-%d %H:%M:%S%z",
                "%Y-%m-%d %H:%M:%S%.3f+0000",
            ];
            for fmt in &formats {
                if let Ok(dt) = DateTime::parse_from_str(field, fmt) {
                    return Ok(CqlValue::Timestamp(dt.timestamp_millis()));
                }
            }
            // Try plain date (midnight UTC)
            if let Ok(d) = NaiveDate::parse_from_str(field, "%Y-%m-%d") {
                let dt = d.and_hms_opt(0, 0, 0).unwrap();
                return Ok(CqlValue::Timestamp(dt.and_utc().timestamp_millis()));
            }
            // Try milliseconds since epoch as a number
            if let Ok(ms) = field.parse::<i64>() {
                return Ok(CqlValue::Timestamp(ms));
            }
            bail!("invalid timestamp: {field:?}")
        }
        "date" => {
            let d = NaiveDate::parse_from_str(field, "%Y-%m-%d")
                .with_context(|| format!("invalid date (expected YYYY-MM-DD): {field:?}"))?;
            Ok(CqlValue::Date(d))
        }
        "time" => {
            // Accept HH:MM:SS, HH:MM:SS.nnn, HH:MM:SS.nnnnnnnnn
            let formats = ["%H:%M:%S%.f", "%H:%M:%S"];
            for fmt in &formats {
                if let Ok(t) = NaiveTime::parse_from_str(field, fmt) {
                    return Ok(CqlValue::Time(t));
                }
            }
            bail!("invalid time (expected HH:MM:SS[.nnn]): {field:?}")
        }
        "inet" => {
            let addr = field
                .parse::<IpAddr>()
                .with_context(|| format!("invalid inet: {field:?}"))?;
            Ok(CqlValue::Inet(addr))
        }
        "blob" => {
            // Accept "0x..." hex or plain hex
            let hex = field.strip_prefix("0x").unwrap_or(field);
            if !hex.len().is_multiple_of(2) {
                bail!("invalid blob (odd number of hex digits): {field:?}");
            }
            let bytes = (0..hex.len())
                .step_by(2)
                .map(|i| {
                    u8::from_str_radix(&hex[i..i + 2], 16)
                        .with_context(|| format!("invalid hex byte at offset {i}: {field:?}"))
                })
                .collect::<Result<Vec<u8>>>()?;
            Ok(CqlValue::Blob(bytes))
        }
        "varint" => {
            let n = field
                .parse::<num_bigint::BigInt>()
                .with_context(|| format!("invalid varint: {field:?}"))?;
            Ok(CqlValue::Varint(n))
        }
        "decimal" => {
            let d = field
                .parse::<bigdecimal::BigDecimal>()
                .with_context(|| format!("invalid decimal: {field:?}"))?;
            Ok(CqlValue::Decimal(d))
        }
        // duration, list<*>, set<*>, map<*>, tuple<*>, and unknown types:
        // pass through as Text; the database parses the CQL literal.
        _ => Ok(CqlValue::Text(field.to_string())),
    }
}

/// Strip the `frozen<...>` wrapper from a CQL type name, if present.
fn strip_frozen(type_name: &str) -> &str {
    let lower = type_name.to_lowercase();
    if lower.starts_with("frozen<") && type_name.ends_with('>') {
        &type_name[7..type_name.len() - 1]
    } else {
        type_name
    }
}

/// Convert a `CqlValue` to a CQL insert literal string.
///
/// Used in the unprepared (string-based) INSERT path. Produces values that can
/// be embedded directly into a CQL statement without further quoting by the
/// caller.
fn cql_value_to_insert_literal(v: &CqlValue) -> String {
    match v {
        CqlValue::Null | CqlValue::Unset => "null".to_string(),
        CqlValue::Text(s) | CqlValue::Ascii(s) => {
            format!("'{}'", s.replace('\'', "''"))
        }
        CqlValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
        CqlValue::Int(n) => n.to_string(),
        CqlValue::BigInt(n) | CqlValue::Counter(n) => n.to_string(),
        CqlValue::SmallInt(n) => n.to_string(),
        CqlValue::TinyInt(n) => n.to_string(),
        CqlValue::Float(f) => {
            if f.is_nan() {
                "NaN".to_string()
            } else if f.is_infinite() {
                if f.is_sign_positive() {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else {
                f.to_string()
            }
        }
        CqlValue::Double(d) => {
            if d.is_nan() {
                "NaN".to_string()
            } else if d.is_infinite() {
                if d.is_sign_positive() {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else {
                d.to_string()
            }
        }
        CqlValue::Varint(n) => n.to_string(),
        CqlValue::Decimal(d) => d.to_string(),
        CqlValue::Uuid(u) | CqlValue::TimeUuid(u) => u.to_string(),
        CqlValue::Timestamp(ms) => {
            // Format as ISO-8601 string, quoted
            match DateTime::from_timestamp_millis(*ms) {
                Some(dt) => {
                    let utc: DateTime<Utc> = dt;
                    format!("'{}'", utc.format("%Y-%m-%d %H:%M:%S%.3f+0000"))
                }
                None => format!("{ms}"),
            }
        }
        CqlValue::Date(d) => format!("'{d}'"),
        CqlValue::Time(t) => format!("'{t}'"),
        CqlValue::Inet(addr) => format!("'{addr}'"),
        CqlValue::Blob(bytes) => {
            let mut s = String::with_capacity(2 + bytes.len() * 2);
            s.push_str("0x");
            for b in bytes {
                s.push_str(&format!("{b:02x}"));
            }
            s
        }
        CqlValue::Duration {
            months,
            days,
            nanoseconds,
        } => {
            format!("{months}mo{days}d{nanoseconds}ns")
        }
        // Collections and UDTs: use Display which outputs CQL literal format
        CqlValue::List(_)
        | CqlValue::Set(_)
        | CqlValue::Map(_)
        | CqlValue::Tuple(_)
        | CqlValue::UserDefinedType { .. } => v.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Token bucket rate limiter for INGESTRATE
// ---------------------------------------------------------------------------

/// Simple token bucket for rate limiting row inserts.
struct TokenBucket {
    rate: f64,
    tokens: f64,
    last: Instant,
}

impl TokenBucket {
    fn new(rows_per_second: usize) -> Self {
        Self {
            rate: rows_per_second as f64,
            tokens: rows_per_second as f64,
            last: Instant::now(),
        }
    }

    async fn acquire(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.rate);
        self.last = now;

        if self.tokens < 1.0 {
            let wait_secs = (1.0 - self.tokens) / self.rate;
            tokio::time::sleep(Duration::from_secs_f64(wait_secs)).await;
            self.tokens = 0.0;
        } else {
            self.tokens -= 1.0;
        }
    }
}

/// Execute a COPY FROM command, importing CSV data into a table.
pub async fn execute_copy_from(
    session: &CqlSession,
    cmd: &CopyFromCommand,
    current_keyspace: Option<&str>,
) -> Result<()> {
    let start = Instant::now();

    // Resolve keyspace and table spec
    let table_spec = match (&cmd.keyspace, current_keyspace) {
        (Some(ks), _) => format!("{}.{}", ks, cmd.table),
        (None, Some(ks)) => format!("{}.{}", ks, cmd.table),
        (None, None) => cmd.table.clone(),
    };

    let source_name = match &cmd.source {
        CopyTarget::File(path) => format!("'{}'", path.display()),
        CopyTarget::Stdin => "STDIN".to_string(),
        CopyTarget::Stdout => unreachable!("COPY FROM cannot use STDOUT"),
    };

    let ttl_clause = match cmd.options.ttl {
        Some(ttl) => format!(" USING TTL {ttl}"),
        None => String::new(),
    };

    // Query schema for column metadata: (name, kind, position, type_name)
    let ks_for_schema = cmd
        .keyspace
        .as_deref()
        .or(current_keyspace)
        .context("no keyspace specified and no current keyspace set")?;
    let schema_query = format!(
        "SELECT column_name, kind, position, type FROM system_schema.columns \
         WHERE keyspace_name = '{}' AND table_name = '{}'",
        ks_for_schema, cmd.table
    );
    let schema_result = session.execute_query(&schema_query).await?;

    // Collect and sort into CREATE TABLE order
    let mut schema_cols: Vec<(String, String, i32, String)> = Vec::new();
    for row in &schema_result.rows {
        let name = match row.values.first() {
            Some(CqlValue::Text(n)) => n.clone(),
            _ => continue,
        };
        let kind = match row.values.get(1) {
            Some(CqlValue::Text(k)) => k.clone(),
            _ => "regular".to_string(),
        };
        let position = match row.values.get(2) {
            Some(CqlValue::Int(p)) => *p,
            _ => -1,
        };
        let type_name = match row.values.get(3) {
            Some(CqlValue::Text(t)) => t.clone(),
            _ => "text".to_string(),
        };
        schema_cols.push((name, kind, position, type_name));
    }
    if schema_cols.is_empty() {
        bail!(
            "could not determine columns for table '{}' — table may not exist",
            table_spec
        );
    }
    schema_cols.sort_by(|a, b| {
        let kind_order = |k: &str| -> i32 {
            match k {
                "partition_key" => 0,
                "clustering" => 1,
                "static" => 2,
                _ => 3,
            }
        };
        kind_order(&a.1)
            .cmp(&kind_order(&b.1))
            .then(a.2.cmp(&b.2))
            .then(a.0.cmp(&b.0))
    });

    // type lookup: column_name → type_name (for header-derived ordering)
    let type_map: std::collections::HashMap<String, String> = schema_cols
        .iter()
        .map(|(n, _, _, t)| (n.clone(), t.clone()))
        .collect();

    // Preliminary column list: explicit columns or schema order
    let prelim_columns: Vec<(String, String)> = match &cmd.columns {
        Some(explicit) => explicit
            .iter()
            .map(|n| {
                let t = type_map
                    .get(n)
                    .cloned()
                    .unwrap_or_else(|| "text".to_string());
                (n.clone(), t)
            })
            .collect(),
        None => schema_cols.into_iter().map(|(n, _, _, t)| (n, t)).collect(),
    };

    // Open CSV reader
    let reader: Box<dyn std::io::Read> = match &cmd.source {
        CopyTarget::File(path) => {
            let file = std::fs::File::open(path)
                .with_context(|| format!("failed to open file: {}", path.display()))?;
            Box::new(std::io::BufReader::new(file))
        }
        CopyTarget::Stdin => Box::new(std::io::stdin().lock()),
        CopyTarget::Stdout => bail!("COPY FROM cannot read from STDOUT"),
    };
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(cmd.options.delimiter as u8)
        .quote(cmd.options.quote as u8)
        .escape(Some(cmd.options.escape as u8))
        .has_headers(cmd.options.header)
        .flexible(true)
        .from_reader(reader);

    // When HEADER=true and no explicit columns, column order comes from CSV header
    let columns: Vec<(String, String)> = if cmd.options.header && cmd.columns.is_none() {
        let headers = csv_reader
            .headers()
            .context("failed to read CSV header row")?;
        headers
            .iter()
            .map(|h| {
                let name = h.trim().to_string();
                let t = type_map
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| "text".to_string());
                (name, t)
            })
            .collect()
    } else {
        prelim_columns
    };

    let col_list: String = columns
        .iter()
        .map(|(n, _)| n.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let col_type_names: Vec<String> = columns.iter().map(|(_, t)| t.clone()).collect();

    // Prepare INSERT statement (done after finalizing column list)
    let prepared_id = if cmd.options.prepared_statements {
        let placeholders = vec!["?"; columns.len()].join(", ");
        let insert_template =
            format!("INSERT INTO {table_spec} ({col_list}) VALUES ({placeholders}){ttl_clause}");
        Some(
            session
                .prepare(&insert_template)
                .await
                .with_context(|| format!("failed to prepare: {insert_template}"))?,
        )
    } else {
        None
    };

    // Open error file
    let mut err_writer: Option<std::io::BufWriter<std::fs::File>> = match &cmd.options.err_file {
        Some(path) => {
            let file = std::fs::File::create(path)
                .with_context(|| format!("failed to create error file: {}", path.display()))?;
            Some(std::io::BufWriter::new(file))
        }
        None => None,
    };

    let mut row_count: usize = 0;
    let mut parse_errors: usize = 0;
    let mut insert_errors: usize = 0;
    let mut rate_limiter = cmd.options.ingest_rate.map(TokenBucket::new);
    let num_processes = cmd.options.num_processes.max(1);
    let chunk_size = cmd.options.chunk_size.max(1);

    let max_attempts = cmd.options.max_attempts;
    let max_parse_errors = cmd.options.max_parse_errors;
    let max_insert_errors = cmd.options.max_insert_errors;
    let report_frequency = cmd.options.report_frequency;
    let null_val = &cmd.options.null_val;

    let mut csv_records = csv_reader.records();

    'outer: loop {
        // --- Parse phase: fill a chunk of CHUNKSIZE typed rows ---
        let mut chunk: Vec<Vec<CqlValue>> = Vec::with_capacity(chunk_size);

        'fill: loop {
            if chunk.len() >= chunk_size {
                break 'fill;
            }
            let record = match csv_records.next() {
                None => break 'fill,
                Some(Err(e)) => {
                    parse_errors += 1;
                    let msg = format!("CSV parse error on row {}: {e}", row_count + parse_errors);
                    eprintln!("{msg}");
                    if let Some(ref mut w) = err_writer {
                        let _ = writeln!(w, "{msg}");
                    }
                    if let Some(max) = max_parse_errors {
                        if parse_errors > max {
                            bail!("Exceeded maximum parse errors ({max}). Aborting.");
                        }
                    }
                    continue 'fill;
                }
                Some(Ok(r)) => r,
            };

            if record.len() != col_type_names.len() {
                parse_errors += 1;
                let msg = format!(
                    "Row {}: expected {} columns but got {}",
                    row_count + parse_errors,
                    col_type_names.len(),
                    record.len()
                );
                eprintln!("{msg}");
                if let Some(ref mut w) = err_writer {
                    let _ = writeln!(w, "{msg}");
                }
                if let Some(max) = max_parse_errors {
                    if parse_errors > max {
                        bail!("Exceeded maximum number of parse errors ({max}). Aborting import.");
                    }
                }
                continue 'fill;
            }

            let mut row_values: Vec<CqlValue> = Vec::with_capacity(col_type_names.len());
            let mut row_ok = true;
            for (field, type_name) in record.iter().zip(col_type_names.iter()) {
                match csv_str_to_cql_value(field, type_name, null_val) {
                    Ok(v) => row_values.push(v),
                    Err(e) => {
                        parse_errors += 1;
                        let msg = format!(
                            "Row {}: type error for '{}': {e}",
                            row_count + parse_errors,
                            type_name
                        );
                        eprintln!("{msg}");
                        if let Some(ref mut w) = err_writer {
                            let _ = writeln!(w, "{msg}");
                        }
                        if let Some(max) = max_parse_errors {
                            if parse_errors > max {
                                bail!("Exceeded maximum parse errors ({max}). Aborting.");
                            }
                        }
                        row_ok = false;
                        break;
                    }
                }
            }
            if row_ok {
                chunk.push(row_values);
            }
        }

        if chunk.is_empty() {
            break 'outer;
        }

        // --- Rate limiting ---
        if let Some(ref mut bucket) = rate_limiter {
            for _ in 0..chunk.len() {
                bucket.acquire().await;
            }
        }

        // --- Insert phase: execute chunk concurrently ---
        let insert_results: Vec<Result<()>> = futures::stream::iter(chunk)
            .map(|values| {
                let ts = table_spec.as_str();
                let cl = col_list.as_str();
                let ttl = ttl_clause.as_str();
                let pid = prepared_id.as_ref();
                async move {
                    insert_row_with_retry(session, pid, ts, cl, ttl, &values, max_attempts).await
                }
            })
            .buffer_unordered(num_processes)
            .collect()
            .await;

        for result in insert_results {
            match result {
                Ok(()) => row_count += 1,
                Err(e) => {
                    insert_errors += 1;
                    let msg = format!("Insert error on row {}: {e}", row_count + insert_errors);
                    eprintln!("{msg}");
                    if let Some(ref mut w) = err_writer {
                        let _ = writeln!(w, "{msg}");
                    }
                    if let Some(max) = max_insert_errors {
                        if insert_errors > max {
                            bail!("Exceeded maximum number of insert errors ({max}). Aborting import.");
                        }
                    }
                }
            }
        }

        // --- Progress report ---
        if let Some(freq) = report_frequency {
            let total = row_count + insert_errors + parse_errors;
            if freq > 0 && total > 0 && total.is_multiple_of(freq) {
                eprintln!("Processed {} rows...", row_count);
            }
        }
    }

    if let Some(ref mut w) = err_writer {
        w.flush()?;
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!("{row_count} rows imported from {source_name} in {elapsed:.3}s.");
    if parse_errors > 0 {
        eprintln!("{parse_errors} parse error(s) encountered.");
    }
    if insert_errors > 0 {
        eprintln!("{insert_errors} insert error(s) encountered.");
    }

    Ok(())
}

/// Execute a single row INSERT with retry on failure.
///
/// When `prepared_id` is `Some`, uses the prepared statement path with typed
/// bound values. Otherwise builds a literal-value INSERT string.
async fn insert_row_with_retry(
    session: &CqlSession,
    prepared_id: Option<&PreparedId>,
    table_spec: &str,
    col_list: &str,
    ttl_clause: &str,
    values: &[CqlValue],
    max_attempts: usize,
) -> Result<()> {
    let max = max_attempts.max(1);
    let mut last_err = anyhow::anyhow!("no attempts made");

    for attempt in 1..=max {
        let result = if let Some(id) = prepared_id {
            session.execute_prepared(id, values).await
        } else {
            let literals: Vec<String> = values.iter().map(cql_value_to_insert_literal).collect();
            let insert = format!(
                "INSERT INTO {} ({}) VALUES ({}){};",
                table_spec,
                col_list,
                literals.join(", "),
                ttl_clause
            );
            session.execute_query(&insert).await
        };

        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                last_err = e;
                if attempt < max {
                    // Exponential backoff: 100ms * 2^(attempt-1), capped at 2s
                    let wait_ms = (100u64 * (1u64 << (attempt - 1).min(4))).min(2000);
                    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                }
            }
        }
    }

    Err(last_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_copy_to_basic() {
        let cmd = parse_copy_to("COPY ks.table TO '/tmp/out.csv'").unwrap();
        assert_eq!(cmd.keyspace, Some("ks".to_string()));
        assert_eq!(cmd.table, "table");
        assert_eq!(cmd.columns, None);
        assert_eq!(
            cmd.filename,
            CopyTarget::File(PathBuf::from("/tmp/out.csv"))
        );
    }

    #[test]
    fn parse_copy_to_with_columns() {
        let cmd = parse_copy_to("COPY ks.table (col1, col2) TO '/tmp/out.csv'").unwrap();
        assert_eq!(cmd.keyspace, Some("ks".to_string()));
        assert_eq!(cmd.table, "table");
        assert_eq!(
            cmd.columns,
            Some(vec!["col1".to_string(), "col2".to_string()])
        );
        assert_eq!(
            cmd.filename,
            CopyTarget::File(PathBuf::from("/tmp/out.csv"))
        );
    }

    #[test]
    fn parse_copy_to_stdout() {
        let cmd = parse_copy_to("COPY ks.table TO STDOUT").unwrap();
        assert_eq!(cmd.filename, CopyTarget::Stdout);
    }

    #[test]
    fn parse_copy_to_with_options() {
        let cmd =
            parse_copy_to("COPY ks.table TO '/tmp/out.csv' WITH DELIMITER='|' AND HEADER=true")
                .unwrap();
        assert_eq!(cmd.options.delimiter, '|');
        assert!(cmd.options.header);
    }

    #[test]
    fn format_value_null() {
        let opts = CopyOptions::default();
        assert_eq!(format_value_for_csv(&CqlValue::Null, &opts), "");
    }

    #[test]
    fn format_value_text() {
        let opts = CopyOptions::default();
        assert_eq!(
            format_value_for_csv(&CqlValue::Text("hello".to_string()), &opts),
            "hello"
        );
    }

    #[test]
    fn format_value_boolean() {
        let opts = CopyOptions::default();
        assert_eq!(
            format_value_for_csv(&CqlValue::Boolean(true), &opts),
            "True"
        );
        assert_eq!(
            format_value_for_csv(&CqlValue::Boolean(false), &opts),
            "False"
        );
    }

    #[test]
    fn format_value_float_precision() {
        let opts = CopyOptions {
            float_precision: 3,
            ..Default::default()
        };
        assert_eq!(
            format_value_for_csv(&CqlValue::Float(1.23456), &opts),
            "1.235"
        );
    }

    #[test]
    fn default_options() {
        let opts = CopyOptions::default();
        assert_eq!(opts.delimiter, ',');
        assert_eq!(opts.quote, '"');
        assert_eq!(opts.escape, '\\');
        assert!(!opts.header);
        assert_eq!(opts.null_val, "");
        assert_eq!(opts.datetime_format, None);
        assert_eq!(opts.encoding, "utf-8");
        assert_eq!(opts.float_precision, 5);
        assert_eq!(opts.double_precision, 12);
        assert_eq!(opts.decimal_sep, '.');
        assert_eq!(opts.thousands_sep, None);
        assert_eq!(opts.bool_style, ("True".to_string(), "False".to_string()));
        assert_eq!(opts.page_size, 1000);
        assert_eq!(opts.max_output_size, None);
        assert_eq!(opts.report_frequency, None);
    }

    // -----------------------------------------------------------------------
    // COPY FROM tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_copy_from_basic() {
        let cmd = parse_copy_from("COPY ks.table FROM '/tmp/in.csv'").unwrap();
        assert_eq!(cmd.keyspace, Some("ks".to_string()));
        assert_eq!(cmd.table, "table");
        assert_eq!(cmd.columns, None);
        assert_eq!(cmd.source, CopyTarget::File(PathBuf::from("/tmp/in.csv")));
    }

    #[test]
    fn parse_copy_from_with_columns() {
        let cmd = parse_copy_from("COPY ks.table (col1, col2) FROM '/tmp/in.csv'").unwrap();
        assert_eq!(cmd.keyspace, Some("ks".to_string()));
        assert_eq!(cmd.table, "table");
        assert_eq!(
            cmd.columns,
            Some(vec!["col1".to_string(), "col2".to_string()])
        );
    }

    #[test]
    fn parse_copy_from_stdin() {
        let cmd = parse_copy_from("COPY ks.table FROM STDIN").unwrap();
        assert_eq!(cmd.source, CopyTarget::Stdin);
    }

    #[test]
    fn parse_copy_from_stdin_case_insensitive() {
        let cmd = parse_copy_from("COPY ks.table FROM stdin").unwrap();
        assert_eq!(cmd.source, CopyTarget::Stdin);
    }

    #[test]
    fn parse_copy_from_no_keyspace() {
        let cmd = parse_copy_from("COPY mytable FROM '/data/file.csv'").unwrap();
        assert_eq!(cmd.keyspace, None);
        assert_eq!(cmd.table, "mytable");
    }

    #[test]
    fn parse_copy_from_with_options() {
        let cmd = parse_copy_from(
            "COPY ks.table FROM '/tmp/in.csv' WITH TTL=3600 AND HEADER=true AND CHUNKSIZE=1000 AND DELIMITER='|'",
        )
        .unwrap();
        assert_eq!(cmd.options.ttl, Some(3600));
        assert!(cmd.options.header);
        assert_eq!(cmd.options.chunk_size, 1000);
        assert_eq!(cmd.options.delimiter, '|');
    }

    #[test]
    fn parse_copy_from_with_error_options() {
        let cmd = parse_copy_from(
            "COPY ks.table FROM '/tmp/in.csv' WITH MAXPARSEERRORS=100 AND MAXINSERTERRORS=50 AND ERRFILE='/tmp/err.log'",
        )
        .unwrap();
        assert_eq!(cmd.options.max_parse_errors, Some(100));
        assert_eq!(cmd.options.max_insert_errors, Some(50));
        assert_eq!(cmd.options.err_file, Some(PathBuf::from("/tmp/err.log")));
    }

    #[test]
    fn parse_copy_from_with_batch_options() {
        let cmd = parse_copy_from(
            "COPY ks.table FROM '/tmp/in.csv' WITH MAXBATCHSIZE=50 AND MINBATCHSIZE=5 AND MAXATTEMPTS=10",
        )
        .unwrap();
        assert_eq!(cmd.options.max_batch_size, 50);
        assert_eq!(cmd.options.min_batch_size, 5);
        assert_eq!(cmd.options.max_attempts, 10);
    }

    #[test]
    fn parse_copy_from_semicolon() {
        let cmd = parse_copy_from("COPY ks.table FROM '/tmp/in.csv';").unwrap();
        assert_eq!(cmd.source, CopyTarget::File(PathBuf::from("/tmp/in.csv")));
    }

    #[test]
    fn default_copy_from_options() {
        let opts = CopyFromOptions::default();
        assert_eq!(opts.delimiter, ',');
        assert_eq!(opts.quote, '"');
        assert_eq!(opts.escape, '\\');
        assert!(!opts.header);
        assert_eq!(opts.null_val, "");
        assert_eq!(opts.datetime_format, None);
        assert_eq!(opts.encoding, "utf-8");
        assert_eq!(opts.chunk_size, 5000);
        assert_eq!(opts.max_batch_size, 20);
        assert_eq!(opts.min_batch_size, 2);
        assert!(opts.prepared_statements);
        assert_eq!(opts.ttl, None);
        assert_eq!(opts.max_attempts, 5);
        assert_eq!(opts.max_parse_errors, None);
        assert_eq!(opts.max_insert_errors, None);
        assert_eq!(opts.err_file, None);
        assert_eq!(opts.report_frequency, None);
        assert_eq!(opts.ingest_rate, None);
        assert_eq!(opts.num_processes, 1);
    }

    // -----------------------------------------------------------------------
    // csv_str_to_cql_value unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn csv_to_cql_text_types() {
        let v = csv_str_to_cql_value("hello", "text", "").unwrap();
        assert_eq!(v, CqlValue::Text("hello".to_string()));

        let v = csv_str_to_cql_value("hi", "ascii", "").unwrap();
        assert_eq!(v, CqlValue::Ascii("hi".to_string()));

        let v = csv_str_to_cql_value("world", "varchar", "").unwrap();
        assert_eq!(v, CqlValue::Text("world".to_string())); // varchar → Text
    }

    #[test]
    fn csv_to_cql_int_types() {
        assert_eq!(
            csv_str_to_cql_value("42", "int", "").unwrap(),
            CqlValue::Int(42)
        );
        assert_eq!(
            csv_str_to_cql_value("-100", "bigint", "").unwrap(),
            CqlValue::BigInt(-100)
        );
        assert_eq!(
            csv_str_to_cql_value("1000", "counter", "").unwrap(),
            CqlValue::BigInt(1000)
        );
        assert_eq!(
            csv_str_to_cql_value("32767", "smallint", "").unwrap(),
            CqlValue::SmallInt(32767)
        );
        assert_eq!(
            csv_str_to_cql_value("127", "tinyint", "").unwrap(),
            CqlValue::TinyInt(127)
        );
    }

    #[test]
    fn csv_to_cql_float_types() {
        match csv_str_to_cql_value("1.5", "float", "").unwrap() {
            CqlValue::Float(f) => assert!((f - 1.5f32).abs() < 1e-5),
            other => panic!("expected Float, got {other:?}"),
        }
        match csv_str_to_cql_value("1.5", "double", "").unwrap() {
            CqlValue::Double(d) => assert!((d - 1.5f64).abs() < 1e-9),
            other => panic!("expected Double, got {other:?}"),
        }
        // Scientific notation
        assert!(matches!(
            csv_str_to_cql_value("1e10", "double", "").unwrap(),
            CqlValue::Double(_)
        ));
    }

    #[test]
    fn csv_to_cql_boolean() {
        for t in &["true", "True", "TRUE", "yes", "YES", "on", "ON", "1"] {
            assert_eq!(
                csv_str_to_cql_value(t, "boolean", "").unwrap(),
                CqlValue::Boolean(true),
                "expected true for {t:?}"
            );
        }
        for f in &["false", "False", "FALSE", "no", "NO", "off", "OFF", "0"] {
            assert_eq!(
                csv_str_to_cql_value(f, "boolean", "").unwrap(),
                CqlValue::Boolean(false),
                "expected false for {f:?}"
            );
        }
    }

    #[test]
    fn csv_to_cql_uuid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        assert!(matches!(
            csv_str_to_cql_value(uuid_str, "uuid", "").unwrap(),
            CqlValue::Uuid(_)
        ));
        assert!(matches!(
            csv_str_to_cql_value(uuid_str, "timeuuid", "").unwrap(),
            CqlValue::TimeUuid(_)
        ));
        // Invalid UUID
        assert!(csv_str_to_cql_value("not-a-uuid", "uuid", "").is_err());
    }

    #[test]
    fn csv_to_cql_timestamp() {
        // ISO-8601 with timezone
        let v = csv_str_to_cql_value("2024-01-15T12:34:56Z", "timestamp", "").unwrap();
        assert!(matches!(v, CqlValue::Timestamp(_)));

        // Milliseconds as integer
        let v = csv_str_to_cql_value("1705318496000", "timestamp", "").unwrap();
        assert_eq!(v, CqlValue::Timestamp(1705318496000));
    }

    #[test]
    fn csv_to_cql_date() {
        use chrono::NaiveDate;
        let v = csv_str_to_cql_value("2024-01-15", "date", "").unwrap();
        assert_eq!(
            v,
            CqlValue::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );

        assert!(csv_str_to_cql_value("not-a-date", "date", "").is_err());
    }

    #[test]
    fn csv_to_cql_time() {
        let v = csv_str_to_cql_value("12:34:56", "time", "").unwrap();
        assert!(matches!(v, CqlValue::Time(_)));

        let v = csv_str_to_cql_value("12:34:56.789", "time", "").unwrap();
        assert!(matches!(v, CqlValue::Time(_)));

        assert!(csv_str_to_cql_value("not-a-time", "time", "").is_err());
    }

    #[test]
    fn csv_to_cql_inet() {
        let v = csv_str_to_cql_value("127.0.0.1", "inet", "").unwrap();
        assert!(matches!(v, CqlValue::Inet(_)));

        let v = csv_str_to_cql_value("::1", "inet", "").unwrap();
        assert!(matches!(v, CqlValue::Inet(_)));

        assert!(csv_str_to_cql_value("not.an.ip", "inet", "").is_err());
    }

    #[test]
    fn csv_to_cql_blob() {
        let v = csv_str_to_cql_value("0xdeadbeef", "blob", "").unwrap();
        assert_eq!(v, CqlValue::Blob(vec![0xde, 0xad, 0xbe, 0xef]));

        // Without 0x prefix
        let v = csv_str_to_cql_value("deadbeef", "blob", "").unwrap();
        assert_eq!(v, CqlValue::Blob(vec![0xde, 0xad, 0xbe, 0xef]));

        // Invalid hex
        assert!(csv_str_to_cql_value("0xgg", "blob", "").is_err());
        // Odd number of digits
        assert!(csv_str_to_cql_value("0xabc", "blob", "").is_err());
    }

    #[test]
    fn csv_to_cql_null_handling() {
        // Empty field with empty null_val → Null
        assert_eq!(csv_str_to_cql_value("", "int", "").unwrap(), CqlValue::Null);
        assert_eq!(
            csv_str_to_cql_value("", "text", "").unwrap(),
            CqlValue::Null
        );
    }

    #[test]
    fn csv_to_cql_null_custom() {
        // Custom null_val
        assert_eq!(
            csv_str_to_cql_value("NULL", "int", "NULL").unwrap(),
            CqlValue::Null
        );
        assert_eq!(
            csv_str_to_cql_value("N/A", "text", "N/A").unwrap(),
            CqlValue::Null
        );
        // Non-null value with custom null_val
        assert!(matches!(
            csv_str_to_cql_value("42", "int", "NULL").unwrap(),
            CqlValue::Int(42)
        ));
    }

    #[test]
    fn csv_to_cql_unknown_type_fallback() {
        // Unknown types fall back to Text
        let v = csv_str_to_cql_value("hello", "customtype", "").unwrap();
        assert_eq!(v, CqlValue::Text("hello".to_string()));

        // Collection types also fall through to Text
        let v = csv_str_to_cql_value("[1, 2, 3]", "list<int>", "").unwrap();
        assert_eq!(v, CqlValue::Text("[1, 2, 3]".to_string()));
    }

    #[test]
    fn csv_to_cql_parse_error_int() {
        // Non-numeric for int type → error
        assert!(csv_str_to_cql_value("notanint", "int", "").is_err());
        assert!(csv_str_to_cql_value("3.14", "int", "").is_err());
        assert!(csv_str_to_cql_value("notanint", "bigint", "").is_err());
    }

    #[test]
    fn csv_to_cql_varint_and_decimal() {
        let v = csv_str_to_cql_value("123456789012345678901234567890", "varint", "").unwrap();
        assert!(matches!(v, CqlValue::Varint(_)));

        let v = csv_str_to_cql_value("3.141592653589793", "decimal", "").unwrap();
        assert!(matches!(v, CqlValue::Decimal(_)));
    }

    #[test]
    fn csv_to_cql_frozen_stripped() {
        // frozen<set<uuid>> should strip frozen wrapper → set<uuid> → Text fallback
        let v = csv_str_to_cql_value("{uuid1, uuid2}", "frozen<set<uuid>>", "").unwrap();
        assert!(matches!(v, CqlValue::Text(_)));
    }

    #[test]
    fn parse_copy_from_numprocesses() {
        let cmd = parse_copy_from("COPY ks.table FROM '/tmp/in.csv' WITH NUMPROCESSES=4").unwrap();
        assert_eq!(cmd.options.num_processes, 4);
    }
}

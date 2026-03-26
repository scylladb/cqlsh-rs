//! COPY TO implementation — exports query results to CSV.
//!
//! Supports `COPY [ks.]table [(col1, col2, ...)] TO 'filename'|STDOUT [WITH options...]`.

use std::io::Write;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};

use crate::driver::types::CqlValue;
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
    let to_pos = find_keyword_outside_parens(trimmed, "TO")
        .context("COPY statement missing TO keyword")?;

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
                let headers: Vec<String> =
                    result.columns.iter().map(|c| c.name.clone()).collect();
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
                    if freq > 0 && row_count % freq == 0 {
                        eprintln!("Processed {} rows...", row_count);
                    }
                }
            }

            wtr.flush()?;
            println!(
                "{} rows exported to '{}'.",
                row_count,
                path.display()
            );
        }
        CopyTarget::Stdout => {
            let stdout = std::io::stdout();
            let handle = stdout.lock();
            let mut wtr = build_csv_writer(&cmd.options, handle);

            if cmd.options.header {
                let headers: Vec<String> =
                    result.columns.iter().map(|c| c.name.clone()).collect();
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
                    if freq > 0 && row_count % freq == 0 {
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
            if i + kw_len <= upper.len() && &upper[i..i + kw_len] == kw_upper {
                // Check word boundaries
                let before_ok =
                    i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
                let after_ok = i + kw_len >= s.len()
                    || !bytes[i + kw_len].is_ascii_alphanumeric();
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
                opts.float_precision = val
                    .parse()
                    .context("FLOATPRECISION must be an integer")?;
            }
            "DOUBLEPRECISION" => {
                opts.double_precision = val
                    .parse()
                    .context("DOUBLEPRECISION must be an integer")?;
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
                let n: usize = val
                    .parse()
                    .context("REPORTFREQUENCY must be an integer")?;
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
    if s.len() >= 2 {
        if (s.starts_with('\'') && s.ends_with('\''))
            || (s.starts_with('"') && s.ends_with('"'))
        {
            return s[1..s.len() - 1].to_string();
        }
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
            other => {
                bail!("unknown COPY FROM option: {other}");
            }
        }
    }

    Ok(opts)
}

/// Format a CSV field value for insertion into a CQL statement.
///
/// - If it matches null_val → `null`
/// - If it looks like a number, UUID, or boolean → use as-is
/// - Otherwise → wrap in single quotes with internal quotes escaped
fn format_csv_value_for_cql(field: &str, null_val: &str) -> String {
    if field == null_val {
        return "null".to_string();
    }

    let trimmed = field.trim();

    if trimmed.is_empty() && null_val.is_empty() {
        return "null".to_string();
    }

    // Check for boolean
    if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("false") {
        return trimmed.to_lowercase();
    }

    // Check for integer
    if trimmed.parse::<i64>().is_ok() {
        return trimmed.to_string();
    }

    // Check for float
    if trimmed.parse::<f64>().is_ok() {
        return trimmed.to_string();
    }

    // Check for UUID (simple pattern: 8-4-4-4-12 hex digits)
    if trimmed.len() == 36 && is_uuid_like(trimmed) {
        return trimmed.to_string();
    }

    // Default: wrap in single quotes, escaping internal single quotes
    format!("'{}'", trimmed.replace('\'', "''"))
}

/// Quick check if a string looks like a UUID.
fn is_uuid_like(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    let expected_lens = [8, 4, 4, 4, 12];
    for (part, &expected) in parts.iter().zip(&expected_lens) {
        if part.len() != expected || !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }
    true
}

/// Execute a COPY FROM command, importing CSV data into a table.
pub async fn execute_copy_from(
    session: &CqlSession,
    cmd: &CopyFromCommand,
    current_keyspace: Option<&str>,
) -> Result<()> {
    // Resolve keyspace
    let table_spec = match (&cmd.keyspace, current_keyspace) {
        (Some(ks), _) => format!("{}.{}", ks, cmd.table),
        (None, Some(ks)) => format!("{}.{}", ks, cmd.table),
        (None, None) => cmd.table.clone(),
    };

    // Determine column names
    let column_names = match &cmd.columns {
        Some(cols) => cols.clone(),
        None => {
            // Query system_schema.columns to get all column names
            let ks = cmd
                .keyspace
                .as_deref()
                .or(current_keyspace)
                .context("no keyspace specified and no current keyspace set")?;
            let query = format!(
                "SELECT column_name, kind, position FROM system_schema.columns WHERE keyspace_name = '{}' AND table_name = '{}'",
                ks, cmd.table
            );
            let result = session.execute_query(&query).await?;
            // Collect (name, kind, position) and sort by CREATE TABLE order:
            // partition_key (by position), clustering (by position), regular (alphabetical)
            let mut cols: Vec<(String, String, i32)> = Vec::new();
            for row in &result.rows {
                let name = match row.values.first() {
                    Some(crate::driver::types::CqlValue::Text(n)) => n.clone(),
                    _ => continue,
                };
                let kind = match row.values.get(1) {
                    Some(crate::driver::types::CqlValue::Text(k)) => k.clone(),
                    _ => "regular".to_string(),
                };
                let position = match row.values.get(2) {
                    Some(crate::driver::types::CqlValue::Int(p)) => *p,
                    _ => -1,
                };
                cols.push((name, kind, position));
            }
            cols.sort_by(|a, b| {
                let kind_order = |k: &str| -> i32 {
                    match k {
                        "partition_key" => 0,
                        "clustering" => 1,
                        "static" => 2,
                        _ => 3, // regular
                    }
                };
                kind_order(&a.1)
                    .cmp(&kind_order(&b.1))
                    .then(a.2.cmp(&b.2))
                    .then(a.0.cmp(&b.0))
            });
            let names: Vec<String> = cols.into_iter().map(|(n, _, _)| n).collect();
            if names.is_empty() {
                bail!(
                    "could not determine columns for table '{}' — table may not exist",
                    table_spec
                );
            }
            names
        }
    };

    let ttl_clause = match cmd.options.ttl {
        Some(ttl) => format!(" USING TTL {ttl}"),
        None => String::new(),
    };

    let source_name = match &cmd.source {
        CopyTarget::File(path) => format!("'{}'", path.display()),
        CopyTarget::Stdin => "STDIN".to_string(),
        CopyTarget::Stdout => unreachable!("COPY FROM cannot use STDOUT"),
    };

    // Build CSV reader
    let reader_result: Result<Box<dyn std::io::Read>> = match &cmd.source {
        CopyTarget::File(path) => {
            let file = std::fs::File::open(path)
                .with_context(|| format!("failed to open file: {}", path.display()))?;
            Ok(Box::new(std::io::BufReader::new(file)))
        }
        CopyTarget::Stdin => Ok(Box::new(std::io::stdin().lock())),
        CopyTarget::Stdout => bail!("COPY FROM cannot read from STDOUT"),
    };

    let reader = reader_result?;
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(cmd.options.delimiter as u8)
        .quote(cmd.options.quote as u8)
        .escape(Some(cmd.options.escape as u8))
        .has_headers(cmd.options.header)
        .flexible(true)
        .from_reader(reader);

    // When HEADER=true and no explicit columns, use CSV header for column order
    let column_names = if cmd.options.header && cmd.columns.is_none() {
        let headers = csv_reader.headers()
            .context("failed to read CSV header row")?;
        headers.iter().map(|h| h.trim().to_string()).collect::<Vec<_>>()
    } else {
        column_names
    };
    let col_list = column_names.join(", ");

    let mut row_count: usize = 0;
    let mut parse_errors: usize = 0;
    let mut insert_errors: usize = 0;
    let mut err_writer: Option<std::io::BufWriter<std::fs::File>> = match &cmd.options.err_file {
        Some(path) => {
            let file = std::fs::File::create(path)
                .with_context(|| format!("failed to create error file: {}", path.display()))?;
            Some(std::io::BufWriter::new(file))
        }
        None => None,
    };

    for result in csv_reader.records() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                parse_errors += 1;
                let msg = format!("CSV parse error on row {}: {e}", row_count + parse_errors);
                eprintln!("{msg}");
                if let Some(ref mut w) = err_writer {
                    let _ = writeln!(w, "{msg}");
                }
                if let Some(max) = cmd.options.max_parse_errors {
                    if parse_errors > max {
                        bail!(
                            "Exceeded maximum number of parse errors ({max}). Aborting import."
                        );
                    }
                }
                continue;
            }
        };

        // Build values list from CSV fields
        let values: Vec<String> = record
            .iter()
            .map(|field| format_csv_value_for_cql(field, &cmd.options.null_val))
            .collect();

        if values.len() != column_names.len() {
            parse_errors += 1;
            let msg = format!(
                "Row {}: expected {} columns but got {}",
                row_count + parse_errors,
                column_names.len(),
                values.len()
            );
            eprintln!("{msg}");
            if let Some(ref mut w) = err_writer {
                let _ = writeln!(w, "{msg}");
            }
            if let Some(max) = cmd.options.max_parse_errors {
                if parse_errors > max {
                    bail!("Exceeded maximum number of parse errors ({max}). Aborting import.");
                }
            }
            continue;
        }

        let insert = format!(
            "INSERT INTO {} ({}) VALUES ({}){};",
            table_spec,
            col_list,
            values.join(", "),
            ttl_clause
        );

        match session.execute_query(&insert).await {
            Ok(_) => {
                row_count += 1;
            }
            Err(e) => {
                insert_errors += 1;
                let msg = format!("Insert error on row {}: {e}", row_count + insert_errors);
                eprintln!("{msg}");
                if let Some(ref mut w) = err_writer {
                    let _ = writeln!(w, "{msg}");
                }
                if let Some(max) = cmd.options.max_insert_errors {
                    if insert_errors > max {
                        bail!(
                            "Exceeded maximum number of insert errors ({max}). Aborting import."
                        );
                    }
                }
            }
        }

        if let Some(freq) = cmd.options.report_frequency {
            if freq > 0 && (row_count + insert_errors + parse_errors) % freq == 0 {
                eprintln!("Processed {} rows...", row_count);
            }
        }
    }

    if let Some(ref mut w) = err_writer {
        w.flush()?;
    }

    println!("{row_count} rows imported from {source_name}.");
    if parse_errors > 0 {
        eprintln!("{parse_errors} parse error(s) encountered.");
    }
    if insert_errors > 0 {
        eprintln!("{insert_errors} insert error(s) encountered.");
    }

    Ok(())
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
        assert_eq!(cmd.filename, CopyTarget::File(PathBuf::from("/tmp/out.csv")));
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
        assert_eq!(cmd.filename, CopyTarget::File(PathBuf::from("/tmp/out.csv")));
    }

    #[test]
    fn parse_copy_to_stdout() {
        let cmd = parse_copy_to("COPY ks.table TO STDOUT").unwrap();
        assert_eq!(cmd.filename, CopyTarget::Stdout);
    }

    #[test]
    fn parse_copy_to_with_options() {
        let cmd = parse_copy_to(
            "COPY ks.table TO '/tmp/out.csv' WITH DELIMITER='|' AND HEADER=true",
        )
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
            format_value_for_csv(&CqlValue::Float(3.14159), &opts),
            "3.142"
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
        assert_eq!(
            cmd.options.err_file,
            Some(PathBuf::from("/tmp/err.log"))
        );
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
    }

    #[test]
    fn format_csv_value_null_val() {
        assert_eq!(format_csv_value_for_cql("", ""), "null");
        assert_eq!(format_csv_value_for_cql("N/A", "N/A"), "null");
    }

    #[test]
    fn format_csv_value_numbers() {
        assert_eq!(format_csv_value_for_cql("42", ""), "42");
        assert_eq!(format_csv_value_for_cql("-7", ""), "-7");
        assert_eq!(format_csv_value_for_cql("3.14", ""), "3.14");
    }

    #[test]
    fn format_csv_value_boolean() {
        assert_eq!(format_csv_value_for_cql("true", ""), "true");
        assert_eq!(format_csv_value_for_cql("False", ""), "false");
        assert_eq!(format_csv_value_for_cql("TRUE", ""), "true");
    }

    #[test]
    fn format_csv_value_uuid() {
        assert_eq!(
            format_csv_value_for_cql("550e8400-e29b-41d4-a716-446655440000", ""),
            "550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn format_csv_value_string() {
        assert_eq!(format_csv_value_for_cql("Alice", ""), "'Alice'");
        assert_eq!(format_csv_value_for_cql("O'Brien", ""), "'O''Brien'");
    }
}

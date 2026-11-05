//! CQL value types and result set representations.
//!
//! Provides an intermediate type layer between the scylla driver's native types
//! and the cqlsh-rs formatting/display layer. This decouples the driver
//! implementation from the rest of the application.

use std::fmt;
use std::net::IpAddr;
use std::sync::atomic::{AtomicUsize, Ordering};

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveTime};
use num_bigint::BigInt;
use uuid::Uuid;

/// A single CQL value, mirroring all CQL data types.
#[derive(Debug, Clone, PartialEq)]
pub enum CqlValue {
    /// ASCII string.
    Ascii(String),
    /// Boolean value.
    Boolean(bool),
    /// Arbitrary-precision integer.
    BigInt(i64),
    /// Arbitrary blob of bytes.
    Blob(Vec<u8>),
    /// Counter value.
    Counter(i64),
    /// Arbitrary-precision decimal.
    Decimal(BigDecimal),
    /// Double-precision floating point.
    Double(f64),
    /// Duration (months, days, nanoseconds).
    Duration {
        months: i32,
        days: i32,
        nanoseconds: i64,
    },
    /// Single-precision floating point.
    Float(f32),
    /// 32-bit integer.
    Int(i32),
    /// 16-bit integer (smallint).
    SmallInt(i16),
    /// 8-bit integer (tinyint).
    TinyInt(i8),
    /// Timestamp (milliseconds since Unix epoch).
    Timestamp(i64),
    /// UUID.
    Uuid(Uuid),
    /// TimeUUID (v1).
    TimeUuid(Uuid),
    /// IP address (inet).
    Inet(IpAddr),
    /// Date (days since epoch: 2^31).
    Date(NaiveDate),
    /// Time of day (nanoseconds since midnight).
    Time(NaiveTime),
    /// UTF-8 string.
    Text(String),
    /// Arbitrary-precision integer.
    Varint(BigInt),
    /// Ordered list of values.
    List(Vec<CqlValue>),
    /// Set of values.
    Set(Vec<CqlValue>),
    /// Map of key-value pairs.
    Map(Vec<(CqlValue, CqlValue)>),
    /// Tuple of values.
    Tuple(Vec<Option<CqlValue>>),
    /// User-defined type.
    UserDefinedType {
        keyspace: String,
        type_name: String,
        fields: Vec<(String, Option<CqlValue>)>,
    },
    /// Null/empty value.
    Null,
    /// Unset value (for prepared statement bindings).
    Unset,
}

impl fmt::Display for CqlValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CqlValue::Ascii(s) | CqlValue::Text(s) => write!(f, "{s}"),
            CqlValue::Boolean(b) => {
                if *b {
                    write!(f, "True")
                } else {
                    write!(f, "False")
                }
            }
            CqlValue::BigInt(v) => write!(f, "{v}"),
            CqlValue::Blob(bytes) => {
                write!(f, "0x")?;
                for b in bytes {
                    write!(f, "{b:02x}")?;
                }
                Ok(())
            }
            CqlValue::Counter(v) => write!(f, "{v}"),
            CqlValue::Decimal(v) => write!(f, "{v}"),
            CqlValue::Double(v) => {
                write!(
                    f,
                    "{}",
                    format_float_py(*v, DOUBLE_PRECISION.load(Ordering::Relaxed))
                )
            }
            CqlValue::Duration {
                months,
                days,
                nanoseconds,
            } => write!(f, "{months}mo{days}d{nanoseconds}ns"),
            CqlValue::Float(v) => {
                write!(
                    f,
                    "{}",
                    format_float_py(*v as f64, FLOAT_PRECISION.load(Ordering::Relaxed))
                )
            }
            CqlValue::Int(v) => write!(f, "{v}"),
            CqlValue::SmallInt(v) => write!(f, "{v}"),
            CqlValue::TinyInt(v) => write!(f, "{v}"),
            CqlValue::Timestamp(millis) => format_timestamp(f, *millis),
            CqlValue::Uuid(u) | CqlValue::TimeUuid(u) => write!(f, "{u}"),
            CqlValue::Inet(addr) => write!(f, "{addr}"),
            CqlValue::Date(d) => write!(f, "{d}"),
            CqlValue::Time(t) => write!(f, "{t}"),
            CqlValue::Varint(v) => write!(f, "{v}"),
            CqlValue::List(items) | CqlValue::Set(items) => {
                let is_set = matches!(self, CqlValue::Set(_));
                let (open, close) = if is_set { ('{', '}') } else { ('[', ']') };
                write!(f, "{open}")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write_cql_literal(f, item)?;
                }
                write!(f, "{close}")
            }
            CqlValue::Map(entries) => {
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write_cql_literal(f, k)?;
                    write!(f, ": ")?;
                    write_cql_literal(f, v)?;
                }
                write!(f, "}}")
            }
            CqlValue::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match item {
                        Some(v) => write_cql_literal(f, v)?,
                        None => write!(f, "null")?,
                    }
                }
                write!(f, ")")
            }
            CqlValue::UserDefinedType { fields, .. } => {
                write!(f, "{{")?;
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}: ")?;
                    match value {
                        Some(v) => write_cql_literal(f, v)?,
                        None => write!(f, "null")?,
                    }
                }
                write!(f, "}}")
            }
            CqlValue::Null => Ok(()),
            CqlValue::Unset => write!(f, "<unset>"),
        }
    }
}

/// Write a CQL literal value, quoting strings.
fn write_cql_literal(f: &mut fmt::Formatter<'_>, value: &CqlValue) -> fmt::Result {
    match value {
        CqlValue::Ascii(s) | CqlValue::Text(s) => {
            write!(f, "'{}'", s.replace('\'', "''"))
        }
        CqlValue::Double(v) => write!(f, "{}", format_float_py(*v, nested_double_precision())),
        other => write!(f, "{other}"),
    }
}

/// Display precision for `float` values and anything nested in a collection
/// (cqlshrc `[ui] float_precision`, Python cqlsh default 5).
static FLOAT_PRECISION: AtomicUsize = AtomicUsize::new(5);
/// Display precision for top-level `double` columns
/// (cqlshrc `[ui] double_precision`; scylla-cqlsh's display default is 5, not 12).
static DOUBLE_PRECISION: AtomicUsize = AtomicUsize::new(5);

/// Set the display precisions from cqlshrc. Called once at startup.
// ponytail: process-wide setting, like Python cqlsh's shell attributes; pass it
// through the formatter if one process ever needs two different precisions.
pub fn set_display_precision(float_precision: Option<u32>, double_precision: Option<u32>) {
    if let Some(p) = float_precision {
        FLOAT_PRECISION.store(p as usize, Ordering::Relaxed);
    }
    if let Some(p) = double_precision {
        DOUBLE_PRECISION.store(p as usize, Ordering::Relaxed);
    }
}

/// Display precision for a nested `double` (inside a collection, tuple or UDT).
/// Python cqlsh picks the precision from the top-level column type, so nested
/// doubles get `float_precision`.
pub fn nested_double_precision() -> usize {
    FLOAT_PRECISION.load(Ordering::Relaxed)
}

/// Format a floating-point value exactly like Python cqlsh's
/// `formatting.format_floating_point_type` (without thousands separator).
///
/// `precision` is a number of significant digits, widened by the number of
/// integer digits when the value would print in fixed notation, then fed to
/// C's `%.*g` (trailing zeros stripped, scientific notation for large/small
/// exponents).
pub fn format_float_py(val: f64, precision: usize) -> String {
    if val.is_nan() {
        return "NaN".to_string();
    }
    if val.is_infinite() {
        return if val > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    let mut precision = precision as i64;
    if val.abs() > f64::EPSILON {
        // Python: int(math.log10(abs(val))) truncates toward zero.
        let exponent = val.abs().log10().trunc() as i64;
        if (-4..precision).contains(&exponent) {
            precision += exponent + 1;
        }
    }
    format_g(val, precision.max(0) as usize)
}

/// C/Python `'%.*g' % (precision, val)` for a finite value.
fn format_g(val: f64, precision: usize) -> String {
    let p = precision.max(1);
    // Round to `p` significant digits first; the resulting exponent decides
    // between fixed and scientific notation, as in C's %g.
    let sci = format!("{:.*e}", p - 1, val);
    let (mantissa, exp) = sci.split_once('e').expect("{:e} always has an exponent");
    let exp: i64 = exp.parse().expect("{:e} exponent is an integer");
    if exp < -4 || exp >= p as i64 {
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{}e{sign}{:02}", strip_fraction_zeros(mantissa), exp.abs())
    } else {
        let decimals = (p as i64 - 1 - exp) as usize;
        strip_fraction_zeros(&format!("{val:.decimals$}")).to_string()
    }
}

/// Drop trailing zeros after the decimal point, and the point itself if bare.
fn strip_fraction_zeros(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}

/// Format a CQL timestamp (milliseconds since Unix epoch).
fn format_timestamp(f: &mut fmt::Formatter<'_>, millis: i64) -> fmt::Result {
    use chrono::{DateTime, Utc};
    let dt = DateTime::from_timestamp_millis(millis);
    match dt {
        Some(dt) => {
            let utc: DateTime<Utc> = dt;
            write!(f, "{}", utc.format("%Y-%m-%d %H:%M:%S%.6f%z"))
        }
        None => write!(f, "<invalid timestamp: {millis}>"),
    }
}

/// A column descriptor in a result set.
#[derive(Debug, Clone)]
pub struct CqlColumn {
    /// Column name.
    pub name: String,
    /// CQL type name (e.g., "text", "int", "uuid").
    pub type_name: String,
}

/// A single row in a result set.
#[derive(Debug, Clone)]
pub struct CqlRow {
    /// The values in this row, one per column.
    pub values: Vec<CqlValue>,
}

impl CqlRow {
    /// Get a value by column index.
    pub fn get(&self, index: usize) -> Option<&CqlValue> {
        self.values.get(index)
    }

    /// Get a value by column name (requires column metadata).
    pub fn get_by_name<'a>(&'a self, name: &str, columns: &[CqlColumn]) -> Option<&'a CqlValue> {
        columns
            .iter()
            .position(|c| c.name == name)
            .and_then(|idx| self.values.get(idx))
    }
}

use std::pin::Pin;

use futures::Stream;

/// A streaming result set that yields rows lazily from the database.
///
/// Rows are fetched page-by-page from the server as they are consumed,
/// keeping memory usage bounded for large result sets.
pub struct CqlRowStream {
    /// Column metadata (available immediately after query starts).
    pub columns: Vec<CqlColumn>,
    /// The underlying async stream of rows.
    pub rows: Pin<Box<dyn Stream<Item = anyhow::Result<CqlRow>> + Send>>,
}

/// The result of executing a CQL query.
#[derive(Debug, Clone)]
pub struct CqlResult {
    /// Column metadata for the result set.
    pub columns: Vec<CqlColumn>,
    /// The rows returned by the query.
    pub rows: Vec<CqlRow>,
    /// Whether the query returned rows (SELECT) vs. was a schema/DML change.
    pub has_rows: bool,
    /// Tracing ID if tracing was enabled.
    pub tracing_id: Option<uuid::Uuid>,
    /// Warnings from the database.
    pub warnings: Vec<String>,
}

impl CqlResult {
    /// Create an empty result (for DML/DDL statements).
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            has_rows: false,
            tracing_id: None,
            warnings: Vec::new(),
        }
    }

    /// Number of rows in the result.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Number of columns in the result.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cql_value_display_text() {
        assert_eq!(CqlValue::Text("hello".to_string()).to_string(), "hello");
    }

    #[test]
    fn cql_value_display_boolean() {
        assert_eq!(CqlValue::Boolean(true).to_string(), "True");
        assert_eq!(CqlValue::Boolean(false).to_string(), "False");
    }

    #[test]
    fn cql_value_display_int() {
        assert_eq!(CqlValue::Int(42).to_string(), "42");
        assert_eq!(CqlValue::BigInt(-100).to_string(), "-100");
        assert_eq!(CqlValue::SmallInt(7).to_string(), "7");
        assert_eq!(CqlValue::TinyInt(-1).to_string(), "-1");
    }

    #[test]
    fn cql_value_display_blob() {
        assert_eq!(
            CqlValue::Blob(vec![0xde, 0xad, 0xbe, 0xef]).to_string(),
            "0xdeadbeef"
        );
    }

    #[test]
    fn cql_value_display_uuid() {
        let id = Uuid::nil();
        assert_eq!(
            CqlValue::Uuid(id).to_string(),
            "00000000-0000-0000-0000-000000000000"
        );
    }

    #[test]
    fn cql_value_display_null() {
        assert_eq!(CqlValue::Null.to_string(), "");
    }

    #[test]
    fn cql_value_display_list() {
        let list = CqlValue::List(vec![CqlValue::Int(1), CqlValue::Int(2), CqlValue::Int(3)]);
        assert_eq!(list.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn cql_value_display_set() {
        let set = CqlValue::Set(vec![
            CqlValue::Text("a".to_string()),
            CqlValue::Text("b".to_string()),
        ]);
        assert_eq!(set.to_string(), "{'a', 'b'}");
    }

    #[test]
    fn cql_value_display_map() {
        let map = CqlValue::Map(vec![(CqlValue::Text("key".to_string()), CqlValue::Int(42))]);
        assert_eq!(map.to_string(), "{'key': 42}");
    }

    #[test]
    fn cql_value_display_tuple() {
        let tuple = CqlValue::Tuple(vec![
            Some(CqlValue::Int(1)),
            None,
            Some(CqlValue::Text("x".to_string())),
        ]);
        assert_eq!(tuple.to_string(), "(1, null, 'x')");
    }

    #[test]
    fn cql_value_display_udt() {
        let udt = CqlValue::UserDefinedType {
            keyspace: "ks".to_string(),
            type_name: "my_type".to_string(),
            fields: vec![
                (
                    "name".to_string(),
                    Some(CqlValue::Text("Alice".to_string())),
                ),
                ("age".to_string(), Some(CqlValue::Int(30))),
            ],
        };
        assert_eq!(udt.to_string(), "{name: 'Alice', age: 30}");
    }

    #[test]
    fn cql_value_display_float_special() {
        assert_eq!(CqlValue::Float(f32::NAN).to_string(), "NaN");
        assert_eq!(CqlValue::Float(f32::INFINITY).to_string(), "Infinity");
        assert_eq!(CqlValue::Float(f32::NEG_INFINITY).to_string(), "-Infinity");
        assert_eq!(CqlValue::Double(f64::NAN).to_string(), "NaN");
    }

    /// Expected strings come from Python cqlsh's `format_floating_point_type`
    /// run on the same f64 values (f32 inputs widened the way the driver does).
    #[test]
    fn format_float_py_matches_python_cqlsh() {
        let cases = [
            (0.5_f64, 5, "0.5"),
            (9999.5_f64, 5, "9999.5"),
            (11.333333015441895_f64, 5, "11.33333"),
            (76571.34375_f64, 5, "76571.34375"),
            (7.657139128428829e+18_f64, 5, "7.6571e+18"),
            (7.657139163564605e+18_f64, 5, "7.6571e+18"),
            (-254.256987_f64, 5, "-254.25699"),
            (-254.256987_f64, 12, "-254.256987"),
            (3.2767456456456457e+52_f64, 5, "3.2767e+52"),
            (-2044.4000244140625_f64, 5, "-2044.40002"),
            (0.0_f64, 5, "0"),
            (-0.0_f64, 5, "-0"),
            (6e-08_f64, 5, "6e-08"),
            (6e-07_f64, 5, "6e-07"),
            (6e-06_f64, 5, "6e-06"),
            (6e-05_f64, 5, "6e-05"),
            (0.0006_f64, 5, "0.0006"),
            (0.06_f64, 5, "0.06"),
            (0.6_f64, 5, "0.6"),
            (6.0_f64, 5, "6"),
            (1.23456_f64, 3, "1.235"),
            (1000000000000000.0_f64, 5, "1e+15"),
            (123456.0_f64, 5, "1.2346e+05"),
            (99999.99999_f64, 5, "99999.99999"),
            (0.0001_f64, 5, "0.0001"),
            (0.00012345678_f64, 5, "0.000123"),
            (1.0_f64, 0, "1"),
            (2.5e-300_f64, 12, "2.5e-300"),
            (1.7976931348623157e+308_f64, 12, "1.79769313486e+308"),
            (76571.345_f64, 12, "76571.345000000001"),
        ];
        for (val, precision, expected) in cases {
            assert_eq!(
                format_float_py(val, precision),
                expected,
                "format_float_py({val:?}, {precision})"
            );
        }
    }

    #[test]
    fn cql_result_empty() {
        let result = CqlResult::empty();
        assert!(!result.has_rows);
        assert_eq!(result.row_count(), 0);
        assert_eq!(result.column_count(), 0);
    }

    #[test]
    fn cql_row_get_by_name() {
        let columns = vec![
            CqlColumn {
                name: "id".to_string(),
                type_name: "int".to_string(),
            },
            CqlColumn {
                name: "name".to_string(),
                type_name: "text".to_string(),
            },
        ];
        let row = CqlRow {
            values: vec![CqlValue::Int(1), CqlValue::Text("Alice".to_string())],
        };
        assert_eq!(
            row.get_by_name("name", &columns),
            Some(&CqlValue::Text("Alice".to_string()))
        );
        assert_eq!(row.get_by_name("missing", &columns), None);
    }
}

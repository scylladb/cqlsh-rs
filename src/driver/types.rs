//! CQL value types and result set representations.
//!
//! Provides an intermediate type layer between the scylla driver's native types
//! and the cqlsh-rs formatting/display layer. This decouples the driver
//! implementation from the rest of the application.

use std::fmt;
use std::net::IpAddr;

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
            CqlValue::Double(v) => format_float64(f, *v),
            CqlValue::Duration {
                months,
                days,
                nanoseconds,
            } => write!(f, "{months}mo{days}d{nanoseconds}ns"),
            CqlValue::Float(v) => format_float32(f, *v),
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
        other => write!(f, "{other}"),
    }
}

/// Format a float64 matching Python cqlsh output style.
fn format_float64(f: &mut fmt::Formatter<'_>, v: f64) -> fmt::Result {
    if v.is_nan() {
        write!(f, "NaN")
    } else if v.is_infinite() {
        if v.is_sign_positive() {
            write!(f, "Infinity")
        } else {
            write!(f, "-Infinity")
        }
    } else if v == v.trunc() && v.abs() < 1e15 {
        // Show as integer-like when possible, matching Python behavior
        write!(f, "{v}")
    } else {
        write!(f, "{v}")
    }
}

/// Format a float32 matching Python cqlsh output style.
fn format_float32(f: &mut fmt::Formatter<'_>, v: f32) -> fmt::Result {
    if v.is_nan() {
        write!(f, "NaN")
    } else if v.is_infinite() {
        if v.is_sign_positive() {
            write!(f, "Infinity")
        } else {
            write!(f, "-Infinity")
        }
    } else {
        write!(f, "{v}")
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

    #[test]
    fn cql_result_empty() {
        let result = CqlResult::empty();
        assert!(!result.has_rows);
        assert_eq!(result.row_count(), 0);
        assert_eq!(result.column_count(), 0);
    }

    #[test]
    fn cql_value_display_counter() {
        assert_eq!(CqlValue::Counter(99).to_string(), "99");
    }

    #[test]
    fn cql_value_display_decimal() {
        let d = BigDecimal::from(12345);
        assert_eq!(CqlValue::Decimal(d).to_string(), "12345");
    }

    #[test]
    fn cql_value_display_duration() {
        let d = CqlValue::Duration {
            months: 1,
            days: 2,
            nanoseconds: 3000000000,
        };
        assert_eq!(d.to_string(), "1mo2d3000000000ns");
    }

    #[test]
    fn cql_value_display_timestamp() {
        assert_eq!(
            CqlValue::Timestamp(0).to_string(),
            "1970-01-01 00:00:00.000000+0000"
        );
    }

    #[test]
    fn cql_value_display_timestamp_invalid() {
        let s = CqlValue::Timestamp(i64::MIN).to_string();
        assert!(s.contains("invalid timestamp"));
    }

    #[test]
    fn cql_value_display_inet() {
        let addr: IpAddr = "127.0.0.1".parse().unwrap();
        assert_eq!(CqlValue::Inet(addr).to_string(), "127.0.0.1");
    }

    #[test]
    fn cql_value_display_inet_v6() {
        let addr: IpAddr = "::1".parse().unwrap();
        assert_eq!(CqlValue::Inet(addr).to_string(), "::1");
    }

    #[test]
    fn cql_value_display_date() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(CqlValue::Date(d).to_string(), "2024-01-15");
    }

    #[test]
    fn cql_value_display_time() {
        let t = NaiveTime::from_hms_opt(13, 45, 30).unwrap();
        assert_eq!(CqlValue::Time(t).to_string(), "13:45:30");
    }

    #[test]
    fn cql_value_display_varint() {
        let v = BigInt::from(123456789i64);
        assert_eq!(CqlValue::Varint(v).to_string(), "123456789");
    }

    #[test]
    fn cql_value_display_ascii() {
        assert_eq!(CqlValue::Ascii("test".to_string()).to_string(), "test");
    }

    #[test]
    fn cql_value_display_timeuuid() {
        let id = Uuid::nil();
        assert_eq!(
            CqlValue::TimeUuid(id).to_string(),
            "00000000-0000-0000-0000-000000000000"
        );
    }

    #[test]
    fn cql_value_display_unset() {
        assert_eq!(CqlValue::Unset.to_string(), "<unset>");
    }

    #[test]
    fn cql_value_display_float_neg_infinity() {
        assert_eq!(CqlValue::Float(f32::NEG_INFINITY).to_string(), "-Infinity");
        assert_eq!(CqlValue::Double(f64::NEG_INFINITY).to_string(), "-Infinity");
    }

    #[test]
    fn cql_value_display_double_infinity() {
        assert_eq!(CqlValue::Double(f64::INFINITY).to_string(), "Infinity");
    }

    #[test]
    fn cql_value_display_text_with_quotes() {
        let list = CqlValue::List(vec![CqlValue::Text("it's".to_string())]);
        assert_eq!(list.to_string(), "['it''s']");
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

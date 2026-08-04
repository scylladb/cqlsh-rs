//! Configuration file parsing and merged configuration for cqlsh-rs.
//!
//! Handles `~/.cassandra/cqlshrc` (INI format) parsing, environment variable loading,
//! and merging with CLI arguments following the precedence rule:
//! CLI > environment variables > cqlshrc > defaults.
//!
//! Many fields are defined ahead of their use in later development phases.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use configparser::ini::Ini;
use thiserror::Error;

use crate::cli::CliArgs;
use crate::client_routes::{parse_client_routes, ClientRoutesSettings};
use crate::driver::join_host_port;

/// Errors specific to configuration loading.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to parse cqlshrc file at {path}: {reason}")]
    ParseError { path: String, reason: String },

    #[error("invalid value for {key}: {reason}")]
    InvalidValue { key: String, reason: String },
}

/// Represents the parsed contents of a cqlshrc INI file.
#[derive(Debug, Clone, Default)]
pub struct CqlshrcConfig {
    pub authentication: AuthenticationSection,
    pub connection: ConnectionSection,
    pub ssl: SslSection,
    pub certfiles: HashMap<String, String>,
    pub ui: UiSection,
    pub cql: CqlSection,
    pub csv: CsvSection,
    pub copy: CopySection,
    pub copy_to: CopyToSection,
    pub copy_from: CopyFromSection,
    pub tracing: TracingSection,
    pub client_routes: ClientRoutesSection,
}

#[derive(Debug, Clone, Default)]
pub struct AuthenticationSection {
    pub credentials: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keyspace: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionSection {
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub factory: Option<String>,
    pub timeout: Option<u64>,
    pub request_timeout: Option<u64>,
    pub connect_timeout: Option<u64>,
    pub client_timeout: Option<u64>,
    pub default_fetch_size: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct SslSection {
    pub certfile: Option<String>,
    pub validate: Option<bool>,
    pub userkey: Option<String>,
    pub usercert: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UiSection {
    pub color: Option<bool>,
    pub datetimeformat: Option<String>,
    pub timezone: Option<String>,
    pub float_precision: Option<u32>,
    pub double_precision: Option<u32>,
    pub max_trace_wait: Option<f64>,
    pub encoding: Option<String>,
    pub completekey: Option<String>,
    pub browser: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CqlSection {
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CsvSection {
    pub field_size_limit: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct CopySection {
    pub numprocesses: Option<u32>,
    pub maxattempts: Option<u32>,
    pub reportfrequency: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct CopyToSection {
    pub pagesize: Option<u32>,
    pub pagetimeout: Option<u64>,
    pub begintoken: Option<String>,
    pub endtoken: Option<String>,
    pub maxrequests: Option<u32>,
    pub maxoutputsize: Option<i64>,
    pub floatprecision: Option<u32>,
    pub doubleprecision: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct CopyFromSection {
    pub maxbatchsize: Option<u32>,
    pub minbatchsize: Option<u32>,
    pub chunksize: Option<u32>,
    pub ingestrate: Option<u64>,
    pub maxparseerrors: Option<i64>,
    pub maxinserterrors: Option<i64>,
    pub preparedstatements: Option<bool>,
    pub ttl: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct TracingSection {
    pub max_trace_wait: Option<f64>,
}

/// The `[client_routes]` section (PrivateLink / Private Service Connect).
#[derive(Debug, Clone, Default)]
pub struct ClientRoutesSection {
    /// Comma- or newline-separated `CONNECTION_ID[=ADDRESS]` specifications.
    pub proxies: Option<String>,
    /// Whether the driver may use shard-aware ports with client routes.
    pub advanced_shard_awareness: Option<bool>,
}

impl CqlshrcConfig {
    /// Build the INI parser used for cqlshrc files.
    ///
    /// Case-sensitive, and with multi-line values enabled so indented
    /// continuation lines behave as they do in Python's `configparser` — the
    /// form Python cqlsh documents for `[client_routes] proxies`.
    fn new_ini() -> Ini {
        let mut ini = Ini::new_cs();
        ini.set_multiline(true);
        ini
    }

    /// Load a cqlshrc file from the given path. Returns default config if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let mut ini = Self::new_ini();
        ini.load(path).map_err(|e| ConfigError::ParseError {
            path: path.display().to_string(),
            reason: e,
        })?;

        Ok(Self::from_ini(&ini))
    }

    /// Parse a cqlshrc from a string (useful for testing).
    pub fn parse(content: &str) -> Result<Self> {
        let mut ini = Self::new_ini();
        ini.read(content.to_string())
            .map_err(|e| ConfigError::ParseError {
                path: "<string>".to_string(),
                reason: e,
            })?;

        Ok(Self::from_ini(&ini))
    }

    fn from_ini(ini: &Ini) -> Self {
        Self {
            authentication: AuthenticationSection {
                credentials: ini.get("authentication", "credentials"),
                username: ini.get("authentication", "username"),
                password: ini.get("authentication", "password"),
                keyspace: ini.get("authentication", "keyspace"),
            },
            connection: ConnectionSection {
                hostname: ini.get("connection", "hostname"),
                port: ini.get("connection", "port").and_then(|v| v.parse().ok()),
                factory: ini.get("connection", "factory"),
                timeout: ini
                    .get("connection", "timeout")
                    .and_then(|v| v.parse().ok()),
                request_timeout: ini
                    .get("connection", "request_timeout")
                    .and_then(|v| v.parse().ok()),
                connect_timeout: ini
                    .get("connection", "connect_timeout")
                    .and_then(|v| v.parse().ok()),
                client_timeout: ini
                    .get("connection", "client_timeout")
                    .and_then(|v| v.parse().ok()),
                default_fetch_size: ini
                    .get("connection", "default_fetch_size")
                    .and_then(|v| v.parse().ok()),
            },
            ssl: SslSection {
                certfile: ini.get("ssl", "certfile"),
                validate: ini.get("ssl", "validate").map(|v| parse_bool(&v)),
                userkey: ini.get("ssl", "userkey"),
                usercert: ini.get("ssl", "usercert"),
                version: ini.get("ssl", "version"),
            },
            certfiles: ini
                .get_map()
                .and_then(|m| m.get("certfiles").cloned())
                .unwrap_or_default()
                .into_iter()
                .filter_map(|(k, v)| v.map(|val| (k, val)))
                .collect(),
            ui: UiSection {
                color: ini.get("ui", "color").map(|v| parse_bool(&v)),
                datetimeformat: ini.get("ui", "datetimeformat"),
                timezone: ini.get("ui", "timezone"),
                float_precision: ini
                    .get("ui", "float_precision")
                    .and_then(|v| v.parse().ok()),
                double_precision: ini
                    .get("ui", "double_precision")
                    .and_then(|v| v.parse().ok()),
                max_trace_wait: ini.get("ui", "max_trace_wait").and_then(|v| v.parse().ok()),
                encoding: ini.get("ui", "encoding"),
                completekey: ini.get("ui", "completekey"),
                browser: ini.get("ui", "browser"),
            },
            cql: CqlSection {
                version: ini.get("cql", "version"),
            },
            csv: CsvSection {
                field_size_limit: ini
                    .get("csv", "field_size_limit")
                    .and_then(|v| v.parse().ok()),
            },
            copy: CopySection {
                numprocesses: ini.get("copy", "numprocesses").and_then(|v| v.parse().ok()),
                maxattempts: ini.get("copy", "maxattempts").and_then(|v| v.parse().ok()),
                reportfrequency: ini
                    .get("copy", "reportfrequency")
                    .and_then(|v| v.parse().ok()),
            },
            copy_to: CopyToSection {
                pagesize: ini.get("copy-to", "pagesize").and_then(|v| v.parse().ok()),
                pagetimeout: ini
                    .get("copy-to", "pagetimeout")
                    .and_then(|v| v.parse().ok()),
                begintoken: ini.get("copy-to", "begintoken").filter(|s| !s.is_empty()),
                endtoken: ini.get("copy-to", "endtoken").filter(|s| !s.is_empty()),
                maxrequests: ini
                    .get("copy-to", "maxrequests")
                    .and_then(|v| v.parse().ok()),
                maxoutputsize: ini
                    .get("copy-to", "maxoutputsize")
                    .and_then(|v| v.parse().ok()),
                floatprecision: ini
                    .get("copy-to", "floatprecision")
                    .and_then(|v| v.parse().ok()),
                doubleprecision: ini
                    .get("copy-to", "doubleprecision")
                    .and_then(|v| v.parse().ok()),
            },
            copy_from: CopyFromSection {
                maxbatchsize: ini
                    .get("copy-from", "maxbatchsize")
                    .and_then(|v| v.parse().ok()),
                minbatchsize: ini
                    .get("copy-from", "minbatchsize")
                    .and_then(|v| v.parse().ok()),
                chunksize: ini
                    .get("copy-from", "chunksize")
                    .and_then(|v| v.parse().ok()),
                ingestrate: ini
                    .get("copy-from", "ingestrate")
                    .and_then(|v| v.parse().ok()),
                maxparseerrors: ini
                    .get("copy-from", "maxparseerrors")
                    .and_then(|v| v.parse().ok()),
                maxinserterrors: ini
                    .get("copy-from", "maxinserterrors")
                    .and_then(|v| v.parse().ok()),
                preparedstatements: ini
                    .get("copy-from", "preparedstatements")
                    .map(|v| parse_bool(&v)),
                ttl: ini.get("copy-from", "ttl").and_then(|v| v.parse().ok()),
            },
            tracing: TracingSection {
                max_trace_wait: ini
                    .get("tracing", "max_trace_wait")
                    .and_then(|v| v.parse().ok()),
            },
            client_routes: ClientRoutesSection {
                proxies: ini.get("client_routes", "proxies"),
                advanced_shard_awareness: ini
                    .get("client_routes", "advanced_shard_awareness")
                    .map(|v| parse_bool(&v)),
            },
        }
    }
}

/// Parse boolean values in the same way Python cqlsh does:
/// "true", "yes", "on", "1" → true, everything else → false.
fn parse_bool(val: &str) -> bool {
    matches!(val.to_lowercase().as_str(), "true" | "yes" | "on" | "1")
}

/// The fully resolved configuration after merging CLI args, environment variables,
/// cqlshrc file, and defaults. Follows precedence: CLI > env > cqlshrc > defaults.
#[derive(Debug, Clone)]
pub struct MergedConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keyspace: Option<String>,
    pub ssl: bool,
    pub color: ColorMode,
    pub debug: bool,
    pub tty: bool,
    pub no_file_io: bool,
    pub no_compact: bool,
    pub disable_history: bool,
    pub execute: Option<String>,
    pub file: Option<String>,
    pub connect_timeout: u64,
    pub request_timeout: u64,
    pub encoding: String,
    pub cqlversion: Option<String>,
    pub protocol_version: Option<u8>,
    pub consistency_level: Option<String>,
    pub serial_consistency_level: Option<String>,
    pub browser: Option<String>,
    pub secure_connect_bundle: Option<String>,
    pub cqlshrc_path: PathBuf,
    pub cqlshrc: CqlshrcConfig,
    pub fetch_size: i32,
    /// Resolved client-routes (PrivateLink) settings.
    pub client_routes: ClientRoutesSettings,
    /// Additional contact points as `host:port` strings. Empty means the single
    /// `host`/`port` pair above; only client routes populate this today.
    pub contact_points: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorMode {
    On,
    Off,
    Auto,
}

/// Default connect timeout in seconds (matches Python cqlsh).
const DEFAULT_CONNECT_TIMEOUT: u64 = 5;
/// Default request timeout in seconds (matches Python cqlsh).
const DEFAULT_REQUEST_TIMEOUT: u64 = 10;
/// Default host.
const DEFAULT_HOST: &str = "127.0.0.1";
/// Default port.
const DEFAULT_PORT: u16 = 9042;
const DEFAULT_FETCH_SIZE: i32 = 100;

/// Load environment variables relevant to cqlsh.
#[derive(Debug, Clone, Default)]
pub struct EnvConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub ssl_certfile: Option<String>,
    pub ssl_validate: Option<bool>,
    pub connect_timeout: Option<u64>,
    pub request_timeout: Option<u64>,
    pub history_file: Option<String>,
}

impl EnvConfig {
    /// Read cqlsh-related environment variables.
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("CQLSH_HOST").ok(),
            port: std::env::var("CQLSH_PORT")
                .ok()
                .and_then(|v| v.parse().ok()),
            ssl_certfile: std::env::var("SSL_CERTFILE").ok(),
            ssl_validate: std::env::var("SSL_VALIDATE").ok().map(|v| parse_bool(&v)),
            connect_timeout: std::env::var("CQLSH_DEFAULT_CONNECT_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok()),
            request_timeout: std::env::var("CQLSH_DEFAULT_REQUEST_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok()),
            history_file: std::env::var("CQL_HISTORY").ok(),
        }
    }
}

impl MergedConfig {
    /// Build a merged configuration from CLI args, environment, and cqlshrc.
    ///
    /// Precedence: CLI > environment > cqlshrc > defaults.
    pub fn build(
        cli: &CliArgs,
        env: &EnvConfig,
        cqlshrc: CqlshrcConfig,
        cqlshrc_path: PathBuf,
    ) -> Self {
        // Host: CLI > env > cqlshrc > default
        let explicit_host = cli
            .host
            .clone()
            .or_else(|| env.host.clone())
            .or_else(|| cqlshrc.connection.hostname.clone());
        let host_was_explicit = explicit_host.is_some();
        let mut host = explicit_host.unwrap_or_else(|| DEFAULT_HOST.to_string());

        // Port: CLI > env > cqlshrc > default
        let port = cli
            .port
            .or(env.port)
            .or(cqlshrc.connection.port)
            .unwrap_or(DEFAULT_PORT);

        // Client routes: CLI replaces cqlshrc wholesale, as in Python cqlsh.
        // Invalid specs are reported by CliArgs::validate and load_config; here
        // they degrade to "no routes" so build() stays infallible.
        let client_routes = ClientRoutesSettings {
            routes: if cli.client_route.is_empty() {
                cqlshrc
                    .client_routes
                    .proxies
                    .as_deref()
                    .and_then(|v| parse_client_routes([v]).ok())
                    .unwrap_or_default()
            } else {
                parse_client_routes(cli.client_route.iter().map(String::as_str)).unwrap_or_default()
            },
            advanced_shard_awareness: if cli.no_client_routes_advanced_shard_awareness {
                false
            } else if cli.client_routes_advanced_shard_awareness {
                true
            } else {
                cqlshrc
                    .client_routes
                    .advanced_shard_awareness
                    .unwrap_or(false)
            },
        };

        // With client routes and no explicit host, the route address overrides
        // are the only contact points the client can reach — use them, as
        // Python cqlsh does. `host` follows the first one so the prompt and
        // connection errors name something meaningful.
        let mut contact_points = Vec::new();
        if client_routes.is_enabled() && !host_was_explicit {
            let addresses: Vec<&str> = client_routes
                .routes
                .iter()
                .filter_map(|route| route.address.as_deref())
                .collect();
            if let Some(first) = addresses.first() {
                host = (*first).to_string();
                contact_points = addresses
                    .iter()
                    .map(|address| join_host_port(address, port))
                    .collect();
            }
        }

        // Username: CLI > cqlshrc
        let username = cli
            .username
            .clone()
            .or_else(|| cqlshrc.authentication.username.clone());

        // Password: CLI > cqlshrc
        let password = cli
            .password
            .clone()
            .or_else(|| cqlshrc.authentication.password.clone());

        // Keyspace: CLI > cqlshrc
        let keyspace = cli
            .keyspace
            .clone()
            .or_else(|| cqlshrc.authentication.keyspace.clone());

        // Color: CLI flags > cqlshrc > auto
        let color = if cli.color {
            ColorMode::On
        } else if cli.no_color {
            ColorMode::Off
        } else {
            match &cqlshrc.ui.color {
                Some(true) => ColorMode::On,
                Some(false) => ColorMode::Off,
                None => ColorMode::Auto,
            }
        };

        // Connect timeout: CLI > env > cqlshrc > default
        let connect_timeout = cli
            .connect_timeout
            .or(env.connect_timeout)
            .or(cqlshrc.connection.connect_timeout)
            .unwrap_or(DEFAULT_CONNECT_TIMEOUT);

        // Request timeout: CLI > env > cqlshrc > default
        let request_timeout = cli
            .request_timeout
            .or(env.request_timeout)
            .or(cqlshrc.connection.request_timeout)
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT);

        // Encoding: CLI > cqlshrc > default
        let encoding = cli
            .encoding
            .clone()
            .or_else(|| cqlshrc.ui.encoding.clone())
            .unwrap_or_else(|| "utf-8".to_string());

        // CQL version: CLI > cqlshrc
        let cqlversion = cli
            .cqlversion
            .clone()
            .or_else(|| cqlshrc.cql.version.clone());

        // Browser: CLI > cqlshrc
        let browser = cli.browser.clone().or_else(|| cqlshrc.ui.browser.clone());

        let fetch_size = cqlshrc
            .connection
            .default_fetch_size
            .unwrap_or(DEFAULT_FETCH_SIZE);

        MergedConfig {
            host,
            port,
            username,
            password,
            keyspace,
            ssl: cli.ssl,
            color,
            debug: cli.debug,
            tty: cli.tty,
            no_file_io: cli.no_file_io,
            no_compact: cli.no_compact,
            disable_history: cli.disable_history,
            execute: cli.execute.clone(),
            file: cli.file.clone(),
            connect_timeout,
            request_timeout,
            encoding,
            cqlversion,
            protocol_version: cli.protocol_version,
            consistency_level: cli.consistency_level.clone(),
            serial_consistency_level: cli.serial_consistency_level.clone(),
            browser,
            secure_connect_bundle: cli.secure_connect_bundle.clone(),
            cqlshrc_path,
            cqlshrc,
            fetch_size,
            client_routes,
            contact_points,
        }
    }
}

/// Resolve the cqlshrc file path based on CLI flag or default location.
pub fn resolve_cqlshrc_path(cli_path: Option<&str>) -> PathBuf {
    if let Some(path) = cli_path {
        PathBuf::from(path)
    } else {
        default_cqlshrc_path()
    }
}

/// Return the default cqlshrc path: ~/.cassandra/cqlshrc
pub fn default_cqlshrc_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cassandra")
        .join("cqlshrc")
}

/// Load the full configuration pipeline: resolve path → load cqlshrc → read env → merge.
pub fn load_config(cli: &CliArgs) -> Result<MergedConfig> {
    let cqlshrc_path = resolve_cqlshrc_path(cli.cqlshrc.as_deref());
    let cqlshrc = CqlshrcConfig::load(&cqlshrc_path)
        .with_context(|| format!("loading cqlshrc from {}", cqlshrc_path.display()))?;

    // Routes from cqlshrc bypass CliArgs::validate, so re-check them here.
    if let Some(proxies) = cqlshrc.client_routes.proxies.as_deref() {
        if cli.client_route.is_empty() {
            let routes =
                parse_client_routes([proxies]).map_err(|reason| ConfigError::InvalidValue {
                    key: "[client_routes] proxies".to_string(),
                    reason,
                })?;
            if !routes.is_empty() && cli.ssl {
                anyhow::bail!(crate::cli::CLIENT_ROUTES_SSL_CONFLICT);
            }
        }
    }

    let env = EnvConfig::from_env();
    Ok(MergedConfig::build(cli, &env, cqlshrc, cqlshrc_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_bool tests ---

    #[test]
    fn parse_bool_true_variants() {
        assert!(parse_bool("true"));
        assert!(parse_bool("True"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("Yes"));
        assert!(parse_bool("on"));
        assert!(parse_bool("ON"));
        assert!(parse_bool("1"));
    }

    #[test]
    fn parse_bool_false_variants() {
        assert!(!parse_bool("false"));
        assert!(!parse_bool("False"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool("off"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool(""));
        assert!(!parse_bool("anything"));
    }

    // --- CqlshrcConfig parsing tests ---

    #[test]
    fn parse_empty_config() {
        let config = CqlshrcConfig::parse("").unwrap();
        assert!(config.authentication.username.is_none());
        assert!(config.connection.hostname.is_none());
        assert!(config.ui.color.is_none());
    }

    #[test]
    fn parse_authentication_section() {
        let config = CqlshrcConfig::parse(
            "[authentication]\nusername = admin\npassword = secret\nkeyspace = test_ks\n",
        )
        .unwrap();
        assert_eq!(config.authentication.username.as_deref(), Some("admin"));
        assert_eq!(config.authentication.password.as_deref(), Some("secret"));
        assert_eq!(config.authentication.keyspace.as_deref(), Some("test_ks"));
    }

    #[test]
    fn parse_connection_section() {
        let config = CqlshrcConfig::parse(
            "[connection]\nhostname = 10.0.0.1\nport = 9043\ntimeout = 30\nrequest_timeout = 60\nconnect_timeout = 15\n",
        )
        .unwrap();
        assert_eq!(config.connection.hostname.as_deref(), Some("10.0.0.1"));
        assert_eq!(config.connection.port, Some(9043));
        assert_eq!(config.connection.timeout, Some(30));
        assert_eq!(config.connection.request_timeout, Some(60));
        assert_eq!(config.connection.connect_timeout, Some(15));
    }

    #[test]
    fn parse_ssl_section() {
        let config = CqlshrcConfig::parse(
            "[ssl]\ncertfile = /path/to/cert.pem\nvalidate = true\nuserkey = /path/to/key.pem\nusercert = /path/to/usercert.pem\nversion = TLSv1_2\n",
        )
        .unwrap();
        assert_eq!(config.ssl.certfile.as_deref(), Some("/path/to/cert.pem"));
        assert_eq!(config.ssl.validate, Some(true));
        assert_eq!(config.ssl.userkey.as_deref(), Some("/path/to/key.pem"));
        assert_eq!(config.ssl.version.as_deref(), Some("TLSv1_2"));
    }

    #[test]
    fn parse_ui_section() {
        let config = CqlshrcConfig::parse(
            "[ui]\ncolor = on\ndatetimeformat = %Y-%m-%d %H:%M:%S%z\ntimezone = UTC\nfloat_precision = 5\ndouble_precision = 12\nmax_trace_wait = 10.0\nencoding = utf-8\ncompletekey = tab\n",
        )
        .unwrap();
        assert_eq!(config.ui.color, Some(true));
        assert_eq!(
            config.ui.datetimeformat.as_deref(),
            Some("%Y-%m-%d %H:%M:%S%z")
        );
        assert_eq!(config.ui.timezone.as_deref(), Some("UTC"));
        assert_eq!(config.ui.float_precision, Some(5));
        assert_eq!(config.ui.double_precision, Some(12));
        assert_eq!(config.ui.max_trace_wait, Some(10.0));
        assert_eq!(config.ui.encoding.as_deref(), Some("utf-8"));
        assert_eq!(config.ui.completekey.as_deref(), Some("tab"));
    }

    #[test]
    fn parse_cql_section() {
        let config = CqlshrcConfig::parse("[cql]\nversion = 3.4.7\n").unwrap();
        assert_eq!(config.cql.version.as_deref(), Some("3.4.7"));
    }

    #[test]
    fn parse_csv_section() {
        let config = CqlshrcConfig::parse("[csv]\nfield_size_limit = 131072\n").unwrap();
        assert_eq!(config.csv.field_size_limit, Some(131072));
    }

    #[test]
    fn parse_copy_section() {
        let config = CqlshrcConfig::parse(
            "[copy]\nnumprocesses = 4\nmaxattempts = 5\nreportfrequency = 0.25\n",
        )
        .unwrap();
        assert_eq!(config.copy.numprocesses, Some(4));
        assert_eq!(config.copy.maxattempts, Some(5));
        assert_eq!(config.copy.reportfrequency, Some(0.25));
    }

    #[test]
    fn parse_copy_to_section() {
        let config = CqlshrcConfig::parse(
            "[copy-to]\npagesize = 1000\npagetimeout = 10\nmaxrequests = 6\nmaxoutputsize = -1\nfloatprecision = 5\ndoubleprecision = 12\n",
        )
        .unwrap();
        assert_eq!(config.copy_to.pagesize, Some(1000));
        assert_eq!(config.copy_to.pagetimeout, Some(10));
        assert_eq!(config.copy_to.maxrequests, Some(6));
        assert_eq!(config.copy_to.maxoutputsize, Some(-1));
        assert_eq!(config.copy_to.floatprecision, Some(5));
        assert_eq!(config.copy_to.doubleprecision, Some(12));
    }

    #[test]
    fn parse_copy_from_section() {
        let config = CqlshrcConfig::parse(
            "[copy-from]\nmaxbatchsize = 20\nminbatchsize = 10\nchunksize = 5000\ningestrate = 100000\nmaxparseerrors = -1\nmaxinserterrors = 1000\npreparedstatements = true\nttl = 3600\n",
        )
        .unwrap();
        assert_eq!(config.copy_from.maxbatchsize, Some(20));
        assert_eq!(config.copy_from.minbatchsize, Some(10));
        assert_eq!(config.copy_from.chunksize, Some(5000));
        assert_eq!(config.copy_from.ingestrate, Some(100000));
        assert_eq!(config.copy_from.maxparseerrors, Some(-1));
        assert_eq!(config.copy_from.maxinserterrors, Some(1000));
        assert_eq!(config.copy_from.preparedstatements, Some(true));
        assert_eq!(config.copy_from.ttl, Some(3600));
    }

    #[test]
    fn parse_tracing_section() {
        let config = CqlshrcConfig::parse("[tracing]\nmax_trace_wait = 10.0\n").unwrap();
        assert_eq!(config.tracing.max_trace_wait, Some(10.0));
    }

    #[test]
    fn parse_certfiles_section() {
        let config = CqlshrcConfig::parse(
            "[certfiles]\n172.31.10.22 = ~/keys/node0.cer.pem\n172.31.8.141 = ~/keys/node1.cer.pem\n",
        )
        .unwrap();
        assert_eq!(
            config.certfiles.get("172.31.10.22").map(|s| s.as_str()),
            Some("~/keys/node0.cer.pem")
        );
        assert_eq!(
            config.certfiles.get("172.31.8.141").map(|s| s.as_str()),
            Some("~/keys/node1.cer.pem")
        );
    }

    #[test]
    fn parse_full_sample_config() {
        let content = r#"
[authentication]
username = cassandra
password = cassandra
keyspace = my_keyspace

[connection]
hostname = 127.0.0.1
port = 9042
timeout = 10
request_timeout = 10
connect_timeout = 5

[ssl]
certfile = /path/to/ca-cert.pem
validate = true

[ui]
color = on
datetimeformat = %Y-%m-%d %H:%M:%S%z
timezone = UTC
float_precision = 5
double_precision = 12

[cql]
version = 3.4.7

[csv]
field_size_limit = 131072

[copy]
numprocesses = 4
maxattempts = 5

[copy-to]
pagesize = 1000
floatprecision = 5

[copy-from]
maxbatchsize = 20
chunksize = 5000
preparedstatements = true
ttl = 3600

[tracing]
max_trace_wait = 10.0
"#;
        let config = CqlshrcConfig::parse(content).unwrap();
        assert_eq!(config.authentication.username.as_deref(), Some("cassandra"));
        assert_eq!(config.connection.port, Some(9042));
        assert_eq!(config.ui.color, Some(true));
        assert_eq!(config.cql.version.as_deref(), Some("3.4.7"));
        assert_eq!(config.copy.numprocesses, Some(4));
        assert_eq!(config.copy_to.pagesize, Some(1000));
        assert_eq!(config.copy_from.ttl, Some(3600));
        assert_eq!(config.tracing.max_trace_wait, Some(10.0));
    }

    #[test]
    fn parse_unknown_keys_ignored() {
        let config =
            CqlshrcConfig::parse("[authentication]\nunknown_key = value\nusername = test\n")
                .unwrap();
        assert_eq!(config.authentication.username.as_deref(), Some("test"));
    }

    #[test]
    fn load_nonexistent_file_returns_default() {
        let config = CqlshrcConfig::load(Path::new("/nonexistent/path/cqlshrc")).unwrap();
        assert!(config.authentication.username.is_none());
        assert!(config.connection.hostname.is_none());
    }

    // --- MergedConfig precedence tests ---

    fn default_cli() -> CliArgs {
        CliArgs {
            host: None,
            port: None,
            color: false,
            no_color: false,
            browser: None,
            ssl: false,
            no_file_io: false,
            debug: false,
            coverage: false,
            execute: None,
            file: None,
            keyspace: None,
            username: None,
            password: None,
            connect_timeout: None,
            request_timeout: None,
            tty: false,
            encoding: None,
            cqlshrc: None,
            cqlversion: None,
            protocol_version: None,
            consistency_level: None,
            serial_consistency_level: None,
            no_compact: false,
            disable_history: false,
            secure_connect_bundle: None,
            completions: None,
            generate_man: false,
            client_route: Vec::new(),
            client_routes_advanced_shard_awareness: false,
            no_client_routes_advanced_shard_awareness: false,
        }
    }

    #[test]
    fn merged_defaults() {
        let cli = default_cli();
        let env = EnvConfig::default();
        let cqlshrc = CqlshrcConfig::default();
        let config = MergedConfig::build(&cli, &env, cqlshrc, default_cqlshrc_path());

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 9042);
        assert!(config.username.is_none());
        assert!(config.password.is_none());
        assert!(config.keyspace.is_none());
        assert!(!config.ssl);
        assert_eq!(config.color, ColorMode::Auto);
        assert_eq!(config.connect_timeout, DEFAULT_CONNECT_TIMEOUT);
        assert_eq!(config.request_timeout, DEFAULT_REQUEST_TIMEOUT);
        assert_eq!(config.encoding, "utf-8");
    }

    #[test]
    fn cli_overrides_everything() {
        let cli = CliArgs {
            host: Some("cli-host".to_string()),
            port: Some(9999),
            username: Some("cli-user".to_string()),
            connect_timeout: Some(99),
            ..default_cli()
        };
        let env = EnvConfig {
            host: Some("env-host".to_string()),
            port: Some(8888),
            connect_timeout: Some(88),
            ..EnvConfig::default()
        };
        let mut cqlshrc = CqlshrcConfig::default();
        cqlshrc.connection.hostname = Some("cqlshrc-host".to_string());
        cqlshrc.connection.port = Some(7777);
        cqlshrc.connection.connect_timeout = Some(77);
        cqlshrc.authentication.username = Some("cqlshrc-user".to_string());

        let config = MergedConfig::build(&cli, &env, cqlshrc, default_cqlshrc_path());

        assert_eq!(config.host, "cli-host");
        assert_eq!(config.port, 9999);
        assert_eq!(config.username.as_deref(), Some("cli-user"));
        assert_eq!(config.connect_timeout, 99);
    }

    #[test]
    fn env_overrides_cqlshrc() {
        let cli = default_cli();
        let env = EnvConfig {
            host: Some("env-host".to_string()),
            port: Some(8888),
            connect_timeout: Some(88),
            ..EnvConfig::default()
        };
        let mut cqlshrc = CqlshrcConfig::default();
        cqlshrc.connection.hostname = Some("cqlshrc-host".to_string());
        cqlshrc.connection.port = Some(7777);
        cqlshrc.connection.connect_timeout = Some(77);

        let config = MergedConfig::build(&cli, &env, cqlshrc, default_cqlshrc_path());

        assert_eq!(config.host, "env-host");
        assert_eq!(config.port, 8888);
        assert_eq!(config.connect_timeout, 88);
    }

    #[test]
    fn cqlshrc_overrides_defaults() {
        let cli = default_cli();
        let env = EnvConfig::default();
        let mut cqlshrc = CqlshrcConfig::default();
        cqlshrc.connection.hostname = Some("cqlshrc-host".to_string());
        cqlshrc.connection.port = Some(7777);
        cqlshrc.connection.connect_timeout = Some(77);
        cqlshrc.connection.request_timeout = Some(99);
        cqlshrc.authentication.username = Some("cqlshrc-user".to_string());
        cqlshrc.authentication.keyspace = Some("cqlshrc-ks".to_string());

        let config = MergedConfig::build(&cli, &env, cqlshrc, default_cqlshrc_path());

        assert_eq!(config.host, "cqlshrc-host");
        assert_eq!(config.port, 7777);
        assert_eq!(config.connect_timeout, 77);
        assert_eq!(config.request_timeout, 99);
        assert_eq!(config.username.as_deref(), Some("cqlshrc-user"));
        assert_eq!(config.keyspace.as_deref(), Some("cqlshrc-ks"));
    }

    #[test]
    fn color_mode_cli_on() {
        let cli = CliArgs {
            color: true,
            ..default_cli()
        };
        let config = MergedConfig::build(
            &cli,
            &EnvConfig::default(),
            CqlshrcConfig::default(),
            default_cqlshrc_path(),
        );
        assert_eq!(config.color, ColorMode::On);
    }

    #[test]
    fn color_mode_cli_off() {
        let cli = CliArgs {
            no_color: true,
            ..default_cli()
        };
        let config = MergedConfig::build(
            &cli,
            &EnvConfig::default(),
            CqlshrcConfig::default(),
            default_cqlshrc_path(),
        );
        assert_eq!(config.color, ColorMode::Off);
    }

    #[test]
    fn color_mode_cqlshrc_on() {
        let mut cqlshrc = CqlshrcConfig::default();
        cqlshrc.ui.color = Some(true);
        let config = MergedConfig::build(
            &default_cli(),
            &EnvConfig::default(),
            cqlshrc,
            default_cqlshrc_path(),
        );
        assert_eq!(config.color, ColorMode::On);
    }

    #[test]
    fn color_mode_cqlshrc_off() {
        let mut cqlshrc = CqlshrcConfig::default();
        cqlshrc.ui.color = Some(false);
        let config = MergedConfig::build(
            &default_cli(),
            &EnvConfig::default(),
            cqlshrc,
            default_cqlshrc_path(),
        );
        assert_eq!(config.color, ColorMode::Off);
    }

    #[test]
    fn color_mode_auto_when_unset() {
        let config = MergedConfig::build(
            &default_cli(),
            &EnvConfig::default(),
            CqlshrcConfig::default(),
            default_cqlshrc_path(),
        );
        assert_eq!(config.color, ColorMode::Auto);
    }

    #[test]
    fn encoding_precedence() {
        // CLI > cqlshrc > default
        let mut cqlshrc = CqlshrcConfig::default();
        cqlshrc.ui.encoding = Some("latin-1".to_string());

        // With only cqlshrc set
        let config = MergedConfig::build(
            &default_cli(),
            &EnvConfig::default(),
            cqlshrc.clone(),
            default_cqlshrc_path(),
        );
        assert_eq!(config.encoding, "latin-1");

        // CLI overrides
        let cli = CliArgs {
            encoding: Some("utf-16".to_string()),
            ..default_cli()
        };
        let config =
            MergedConfig::build(&cli, &EnvConfig::default(), cqlshrc, default_cqlshrc_path());
        assert_eq!(config.encoding, "utf-16");
    }

    #[test]
    fn cqlversion_precedence() {
        let mut cqlshrc = CqlshrcConfig::default();
        cqlshrc.cql.version = Some("3.4.5".to_string());

        let config = MergedConfig::build(
            &default_cli(),
            &EnvConfig::default(),
            cqlshrc.clone(),
            default_cqlshrc_path(),
        );
        assert_eq!(config.cqlversion.as_deref(), Some("3.4.5"));

        let cli = CliArgs {
            cqlversion: Some("3.4.7".to_string()),
            ..default_cli()
        };
        let config =
            MergedConfig::build(&cli, &EnvConfig::default(), cqlshrc, default_cqlshrc_path());
        assert_eq!(config.cqlversion.as_deref(), Some("3.4.7"));
    }

    #[test]
    fn resolve_cqlshrc_path_custom() {
        let path = resolve_cqlshrc_path(Some("/etc/custom/cqlshrc"));
        assert_eq!(path, PathBuf::from("/etc/custom/cqlshrc"));
    }

    #[test]
    fn resolve_cqlshrc_path_default() {
        let path = resolve_cqlshrc_path(None);
        assert!(path.ends_with(".cassandra/cqlshrc"));
    }

    // --- File loading tests ---

    #[test]
    fn load_config_from_tempfile() {
        let dir = tempfile::tempdir().unwrap();
        let cqlshrc_path = dir.path().join("cqlshrc");
        std::fs::write(
            &cqlshrc_path,
            "[authentication]\nusername = file_user\n[connection]\nport = 9999\n",
        )
        .unwrap();

        let config = CqlshrcConfig::load(&cqlshrc_path).unwrap();
        assert_eq!(config.authentication.username.as_deref(), Some("file_user"));
        assert_eq!(config.connection.port, Some(9999));
    }

    #[test]
    fn ssl_validate_false() {
        let config = CqlshrcConfig::parse("[ssl]\nvalidate = false\n").unwrap();
        assert_eq!(config.ssl.validate, Some(false));
    }

    #[test]
    fn copy_from_preparedstatements_false() {
        let config = CqlshrcConfig::parse("[copy-from]\npreparedstatements = false\n").unwrap();
        assert_eq!(config.copy_from.preparedstatements, Some(false));
    }

    #[test]
    fn invalid_numeric_ignored() {
        let config = CqlshrcConfig::parse("[connection]\nport = not_a_number\n").unwrap();
        assert!(config.connection.port.is_none());
    }

    #[test]
    fn copy_to_empty_begintoken() {
        let config = CqlshrcConfig::parse("[copy-to]\nbegintoken = \n").unwrap();
        assert!(config.copy_to.begintoken.is_none());
    }

    // --- [client_routes] section ---

    #[test]
    fn client_routes_section_parsed() {
        let config = CqlshrcConfig::parse(
            "[client_routes]\nproxies = conn-a=proxy-a.example.com,conn-b\n\
             advanced_shard_awareness = true\n",
        )
        .unwrap();
        assert_eq!(
            config.client_routes.proxies.as_deref(),
            Some("conn-a=proxy-a.example.com,conn-b")
        );
        assert_eq!(config.client_routes.advanced_shard_awareness, Some(true));
    }

    #[test]
    fn client_routes_multiline_proxies_parsed() {
        // Python's configparser joins indented continuation lines; the
        // multi-line form is what Python cqlsh documents for `proxies`.
        let config = CqlshrcConfig::parse(
            "[client_routes]\nproxies = conn-a=proxy-a.example.com,\n          conn-b\n",
        )
        .unwrap();
        let routes =
            parse_client_routes([config.client_routes.proxies.as_deref().unwrap()]).unwrap();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].connection_id, "conn-a");
        assert_eq!(routes[0].address.as_deref(), Some("proxy-a.example.com"));
        assert_eq!(routes[1].connection_id, "conn-b");
        assert!(routes[1].address.is_none());
    }

    #[test]
    fn multiline_does_not_disturb_other_sections() {
        let config = CqlshrcConfig::parse(
            "[connection]\nhostname = 10.0.0.1\nport = 9043\n\n[ui]\ncolor = on\n",
        )
        .unwrap();
        assert_eq!(config.connection.hostname.as_deref(), Some("10.0.0.1"));
        assert_eq!(config.connection.port, Some(9043));
        assert_eq!(config.ui.color, Some(true));
    }

    #[test]
    fn client_routes_from_cqlshrc_merged() {
        let cli = default_cli();
        let cqlshrc =
            CqlshrcConfig::parse("[client_routes]\nproxies = conn-a=proxy-a.example.com\n")
                .unwrap();
        let config =
            MergedConfig::build(&cli, &EnvConfig::default(), cqlshrc, default_cqlshrc_path());

        assert!(config.client_routes.is_enabled());
        assert_eq!(config.client_routes.routes.len(), 1);
        assert_eq!(config.client_routes.routes[0].connection_id, "conn-a");
        assert!(!config.client_routes.advanced_shard_awareness);
    }

    #[test]
    fn cli_client_routes_replace_cqlshrc() {
        let mut cli = default_cli();
        cli.client_route = vec!["cli-conn".to_string()];
        let cqlshrc =
            CqlshrcConfig::parse("[client_routes]\nproxies = file-conn=proxy.example.com\n")
                .unwrap();
        let config =
            MergedConfig::build(&cli, &EnvConfig::default(), cqlshrc, default_cqlshrc_path());

        assert_eq!(config.client_routes.routes.len(), 1);
        assert_eq!(config.client_routes.routes[0].connection_id, "cli-conn");
        // The cqlshrc route provided an address; the CLI one did not, so no
        // contact point is derived from it.
        assert!(config.contact_points.is_empty());
        assert_eq!(config.host, "127.0.0.1");
    }

    #[test]
    fn advanced_shard_awareness_precedence() {
        let cqlshrc =
            CqlshrcConfig::parse("[client_routes]\nadvanced_shard_awareness = true\n").unwrap();

        let config = MergedConfig::build(
            &default_cli(),
            &EnvConfig::default(),
            cqlshrc.clone(),
            default_cqlshrc_path(),
        );
        assert!(config.client_routes.advanced_shard_awareness);

        let mut cli = default_cli();
        cli.no_client_routes_advanced_shard_awareness = true;
        let config =
            MergedConfig::build(&cli, &EnvConfig::default(), cqlshrc, default_cqlshrc_path());
        assert!(!config.client_routes.advanced_shard_awareness);

        let mut cli = default_cli();
        cli.client_routes_advanced_shard_awareness = true;
        let config = MergedConfig::build(
            &cli,
            &EnvConfig::default(),
            CqlshrcConfig::default(),
            default_cqlshrc_path(),
        );
        assert!(config.client_routes.advanced_shard_awareness);
    }

    #[test]
    fn contact_points_derived_from_route_addresses() {
        let mut cli = default_cli();
        cli.client_route =
            vec!["conn-a=proxy-a.example.com,conn-b=proxy-b.example.com".to_string()];
        let config = MergedConfig::build(
            &cli,
            &EnvConfig::default(),
            CqlshrcConfig::default(),
            default_cqlshrc_path(),
        );

        assert_eq!(config.host, "proxy-a.example.com");
        assert_eq!(
            config.contact_points,
            ["proxy-a.example.com:9042", "proxy-b.example.com:9042"]
        );
    }

    #[test]
    fn explicit_host_wins_over_route_addresses() {
        let mut cli = default_cli();
        cli.host = Some("seed.example.com".to_string());
        cli.client_route = vec!["conn-a=proxy-a.example.com".to_string()];
        let config = MergedConfig::build(
            &cli,
            &EnvConfig::default(),
            CqlshrcConfig::default(),
            default_cqlshrc_path(),
        );
        assert_eq!(config.host, "seed.example.com");
        assert!(config.contact_points.is_empty());

        // Same for a host from the environment...
        let mut cli = default_cli();
        cli.client_route = vec!["conn-a=proxy-a.example.com".to_string()];
        let env = EnvConfig {
            host: Some("env.example.com".to_string()),
            ..EnvConfig::default()
        };
        let config =
            MergedConfig::build(&cli, &env, CqlshrcConfig::default(), default_cqlshrc_path());
        assert_eq!(config.host, "env.example.com");
        assert!(config.contact_points.is_empty());

        // ...and from cqlshrc.
        let mut cli = default_cli();
        cli.client_route = vec!["conn-a=proxy-a.example.com".to_string()];
        let cqlshrc = CqlshrcConfig::parse("[connection]\nhostname = file.example.com\n").unwrap();
        let config =
            MergedConfig::build(&cli, &EnvConfig::default(), cqlshrc, default_cqlshrc_path());
        assert_eq!(config.host, "file.example.com");
        assert!(config.contact_points.is_empty());
    }

    #[test]
    fn contact_points_bracket_ipv6_literals() {
        let mut cli = default_cli();
        cli.client_route = vec!["conn-a=::1".to_string()];
        let config = MergedConfig::build(
            &cli,
            &EnvConfig::default(),
            CqlshrcConfig::default(),
            default_cqlshrc_path(),
        );
        assert_eq!(config.contact_points, ["[::1]:9042"]);
        // `host` keeps the unbracketed form — it is only displayed.
        assert_eq!(config.host, "::1");
    }

    #[test]
    fn join_host_port_forms() {
        assert_eq!(
            join_host_port("proxy.example.com", 9042),
            "proxy.example.com:9042"
        );
        assert_eq!(join_host_port("10.0.0.1", 9042), "10.0.0.1:9042");
        assert_eq!(join_host_port("::1", 9042), "[::1]:9042");
        assert_eq!(join_host_port("[::1]", 9042), "[::1]:9042");
    }

    #[test]
    fn contact_points_use_resolved_port() {
        let mut cli = default_cli();
        cli.port = Some(19042);
        cli.client_route = vec!["conn-a=proxy-a.example.com".to_string()];
        let config = MergedConfig::build(
            &cli,
            &EnvConfig::default(),
            CqlshrcConfig::default(),
            default_cqlshrc_path(),
        );
        assert_eq!(config.contact_points, ["proxy-a.example.com:19042"]);
    }

    #[test]
    fn load_config_rejects_cqlshrc_routes_with_ssl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cqlshrc");
        std::fs::write(&path, "[client_routes]\nproxies = conn-a\n").unwrap();

        let mut cli = default_cli();
        cli.cqlshrc = Some(path.display().to_string());
        cli.ssl = true;

        let err = load_config(&cli).unwrap_err().to_string();
        assert_eq!(err, crate::cli::CLIENT_ROUTES_SSL_CONFLICT);
    }

    #[test]
    fn load_config_rejects_invalid_cqlshrc_route_spec() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cqlshrc");
        std::fs::write(&path, "[client_routes]\nproxies = =proxy-a.example.com\n").unwrap();

        let mut cli = default_cli();
        cli.cqlshrc = Some(path.display().to_string());

        let err = load_config(&cli).unwrap_err().to_string();
        assert!(
            err.contains("empty connection id"),
            "unexpected error: {err}"
        );
    }
}

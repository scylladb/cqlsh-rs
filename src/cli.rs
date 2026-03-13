//! Command-line argument parsing for cqlsh-rs.
//!
//! Implements 100% CLI compatibility with Python cqlsh, accepting all flags
//! from `cqlsh --help` across Cassandra 3.11, 4.x, and 5.x.

use clap::Parser;
use clap_complete::Shell;

/// The Apache Cassandra interactive CQL shell (Rust implementation).
///
/// Connects to a Cassandra cluster and provides an interactive shell
/// for executing CQL statements.
#[derive(Parser, Debug, Clone)]
#[command(name = "cqlsh", version, about, disable_help_flag = false)]
pub struct CliArgs {
    /// Contact point hostname (default: 127.0.0.1)
    #[arg(value_name = "host")]
    pub host: Option<String>,

    /// Native transport port (default: 9042)
    #[arg(value_name = "port")]
    pub port: Option<u16>,

    /// Force colored output
    #[arg(short = 'C', long = "color")]
    pub color: bool,

    /// Disable colored output
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Browser for CQL HELP (unused in modern cqlsh)
    #[arg(long = "browser", value_name = "BROWSER")]
    pub browser: Option<String>,

    /// Enable SSL/TLS connection
    #[arg(long = "ssl")]
    pub ssl: bool,

    /// Disable file I/O commands (COPY, SOURCE, CAPTURE)
    #[arg(long = "no-file-io")]
    pub no_file_io: bool,

    /// Show additional debug info
    #[arg(long = "debug")]
    pub debug: bool,

    /// Collect coverage (internal, accepted but ignored)
    #[arg(long = "coverage", hide = true)]
    pub coverage: bool,

    /// Execute a CQL statement and exit
    #[arg(short = 'e', long = "execute", value_name = "STATEMENT")]
    pub execute: Option<String>,

    /// Execute statements from a file
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    pub file: Option<String>,

    /// Default keyspace
    #[arg(short = 'k', long = "keyspace", value_name = "KEYSPACE")]
    pub keyspace: Option<String>,

    /// Authentication username
    #[arg(short = 'u', long = "username", value_name = "USERNAME")]
    pub username: Option<String>,

    /// Authentication password
    #[arg(short = 'p', long = "password", value_name = "PASSWORD")]
    pub password: Option<String>,

    /// Connection timeout in seconds
    #[arg(long = "connect-timeout", value_name = "SECONDS")]
    pub connect_timeout: Option<u64>,

    /// Per-request timeout in seconds
    #[arg(long = "request-timeout", value_name = "SECONDS")]
    pub request_timeout: Option<u64>,

    /// Force TTY mode
    #[arg(short = 't', long = "tty")]
    pub tty: bool,

    /// Set character encoding (default: utf-8)
    #[arg(long = "encoding", value_name = "ENCODING")]
    pub encoding: Option<String>,

    /// Path to cqlshrc file (default: ~/.cassandra/cqlshrc)
    #[arg(long = "cqlshrc", value_name = "FILE")]
    pub cqlshrc: Option<String>,

    /// CQL version to use
    #[arg(long = "cqlversion", value_name = "VERSION")]
    pub cqlversion: Option<String>,

    /// Native protocol version
    #[arg(long = "protocol-version", value_name = "VERSION")]
    pub protocol_version: Option<u8>,

    /// Initial consistency level
    #[arg(long = "consistency-level", value_name = "LEVEL")]
    pub consistency_level: Option<String>,

    /// Initial serial consistency level
    #[arg(long = "serial-consistency-level", value_name = "LEVEL")]
    pub serial_consistency_level: Option<String>,

    /// Disable compact storage interpretation
    #[arg(long = "no_compact")]
    pub no_compact: bool,

    /// Disable saving of command history
    #[arg(long = "disable-history")]
    pub disable_history: bool,

    /// Secure connect bundle for Astra DB
    #[arg(short = 'b', long = "secure-connect-bundle", value_name = "BUNDLE")]
    pub secure_connect_bundle: Option<String>,

    /// Generate shell completion script for the given shell (bash, zsh, fish, elvish, powershell)
    #[arg(long = "completions", value_name = "SHELL")]
    pub completions: Option<Shell>,
}

impl CliArgs {
    /// Validate CLI arguments for mutual exclusivity and ranges.
    pub fn validate(&self) -> Result<(), String> {
        if self.color && self.no_color {
            return Err("Cannot use both --color and --no-color".to_string());
        }

        if self.execute.is_some() && self.file.is_some() {
            return Err("Cannot use both --execute and --file".to_string());
        }

        if let Some(pv) = self.protocol_version {
            if !(1..=6).contains(&pv) {
                return Err(format!(
                    "Protocol version must be between 1 and 6, got {}",
                    pv
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> CliArgs {
        let mut full_args = vec!["cqlsh-rs"];
        full_args.extend_from_slice(args);
        CliArgs::parse_from(full_args)
    }

    #[test]
    fn no_args_defaults() {
        let args = parse(&[]);
        assert!(args.host.is_none());
        assert!(args.port.is_none());
        assert!(!args.color);
        assert!(!args.no_color);
        assert!(!args.ssl);
        assert!(!args.debug);
        assert!(!args.tty);
        assert!(!args.no_file_io);
        assert!(!args.no_compact);
        assert!(!args.disable_history);
        assert!(args.execute.is_none());
        assert!(args.file.is_none());
        assert!(args.keyspace.is_none());
        assert!(args.username.is_none());
        assert!(args.password.is_none());
        assert!(args.connect_timeout.is_none());
        assert!(args.request_timeout.is_none());
        assert!(args.encoding.is_none());
        assert!(args.cqlshrc.is_none());
        assert!(args.cqlversion.is_none());
        assert!(args.protocol_version.is_none());
        assert!(args.consistency_level.is_none());
        assert!(args.serial_consistency_level.is_none());
        assert!(args.browser.is_none());
        assert!(args.secure_connect_bundle.is_none());
    }

    #[test]
    fn positional_host() {
        let args = parse(&["192.168.1.1"]);
        assert_eq!(args.host.as_deref(), Some("192.168.1.1"));
        assert!(args.port.is_none());
    }

    #[test]
    fn positional_host_and_port() {
        let args = parse(&["192.168.1.1", "9043"]);
        assert_eq!(args.host.as_deref(), Some("192.168.1.1"));
        assert_eq!(args.port, Some(9043));
    }

    #[test]
    fn execute_flag_short() {
        let args = parse(&["-e", "SELECT * FROM system.local"]);
        assert_eq!(args.execute.as_deref(), Some("SELECT * FROM system.local"));
    }

    #[test]
    fn execute_flag_long() {
        let args = parse(&["--execute", "DESC KEYSPACES"]);
        assert_eq!(args.execute.as_deref(), Some("DESC KEYSPACES"));
    }

    #[test]
    fn file_flag() {
        let args = parse(&["-f", "/tmp/schema.cql"]);
        assert_eq!(args.file.as_deref(), Some("/tmp/schema.cql"));
    }

    #[test]
    fn keyspace_flag() {
        let args = parse(&["-k", "my_keyspace"]);
        assert_eq!(args.keyspace.as_deref(), Some("my_keyspace"));
    }

    #[test]
    fn auth_flags() {
        let args = parse(&["-u", "admin", "-p", "secret"]);
        assert_eq!(args.username.as_deref(), Some("admin"));
        assert_eq!(args.password.as_deref(), Some("secret"));
    }

    #[test]
    fn ssl_flag() {
        let args = parse(&["--ssl"]);
        assert!(args.ssl);
    }

    #[test]
    fn color_flag() {
        let args = parse(&["-C"]);
        assert!(args.color);
    }

    #[test]
    fn no_color_flag() {
        let args = parse(&["--no-color"]);
        assert!(args.no_color);
    }

    #[test]
    fn debug_flag() {
        let args = parse(&["--debug"]);
        assert!(args.debug);
    }

    #[test]
    fn tty_flag_short() {
        let args = parse(&["-t"]);
        assert!(args.tty);
    }

    #[test]
    fn tty_flag_long() {
        let args = parse(&["--tty"]);
        assert!(args.tty);
    }

    #[test]
    fn timeout_flags() {
        let args = parse(&["--connect-timeout", "30", "--request-timeout", "60"]);
        assert_eq!(args.connect_timeout, Some(30));
        assert_eq!(args.request_timeout, Some(60));
    }

    #[test]
    fn encoding_flag() {
        let args = parse(&["--encoding", "latin-1"]);
        assert_eq!(args.encoding.as_deref(), Some("latin-1"));
    }

    #[test]
    fn cqlshrc_flag() {
        let args = parse(&["--cqlshrc", "/etc/cqlshrc"]);
        assert_eq!(args.cqlshrc.as_deref(), Some("/etc/cqlshrc"));
    }

    #[test]
    fn cqlversion_flag() {
        let args = parse(&["--cqlversion", "3.4.5"]);
        assert_eq!(args.cqlversion.as_deref(), Some("3.4.5"));
    }

    #[test]
    fn protocol_version_flag() {
        let args = parse(&["--protocol-version", "4"]);
        assert_eq!(args.protocol_version, Some(4));
    }

    #[test]
    fn consistency_level_flag() {
        let args = parse(&["--consistency-level", "QUORUM"]);
        assert_eq!(args.consistency_level.as_deref(), Some("QUORUM"));
    }

    #[test]
    fn serial_consistency_level_flag() {
        let args = parse(&["--serial-consistency-level", "LOCAL_SERIAL"]);
        assert_eq!(
            args.serial_consistency_level.as_deref(),
            Some("LOCAL_SERIAL")
        );
    }

    #[test]
    fn no_file_io_flag() {
        let args = parse(&["--no-file-io"]);
        assert!(args.no_file_io);
    }

    #[test]
    fn no_compact_flag() {
        let args = parse(&["--no_compact"]);
        assert!(args.no_compact);
    }

    #[test]
    fn disable_history_flag() {
        let args = parse(&["--disable-history"]);
        assert!(args.disable_history);
    }

    #[test]
    fn secure_connect_bundle_flag() {
        let args = parse(&["-b", "/path/to/bundle.zip"]);
        assert_eq!(
            args.secure_connect_bundle.as_deref(),
            Some("/path/to/bundle.zip")
        );
    }

    #[test]
    fn browser_flag() {
        let args = parse(&["--browser", "firefox"]);
        assert_eq!(args.browser.as_deref(), Some("firefox"));
    }

    #[test]
    fn combined_flags() {
        let args = parse(&[
            "10.0.0.1",
            "9142",
            "-u",
            "admin",
            "-p",
            "pass",
            "-k",
            "test_ks",
            "--ssl",
            "-C",
            "--connect-timeout",
            "15",
        ]);
        assert_eq!(args.host.as_deref(), Some("10.0.0.1"));
        assert_eq!(args.port, Some(9142));
        assert_eq!(args.username.as_deref(), Some("admin"));
        assert_eq!(args.password.as_deref(), Some("pass"));
        assert_eq!(args.keyspace.as_deref(), Some("test_ks"));
        assert!(args.ssl);
        assert!(args.color);
        assert_eq!(args.connect_timeout, Some(15));
    }

    // Validation tests

    #[test]
    fn validate_color_conflict() {
        let args = parse(&["-C", "--no-color"]);
        let result = args.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--color"));
    }

    #[test]
    fn validate_execute_and_file_conflict() {
        let args = parse(&["-e", "SELECT 1", "-f", "test.cql"]);
        let result = args.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--execute"));
    }

    #[test]
    fn validate_protocol_version_range() {
        let args = parse(&["--protocol-version", "4"]);
        assert!(args.validate().is_ok());
    }

    #[test]
    fn validate_valid_args() {
        let args = parse(&["-u", "admin", "--ssl", "-k", "test"]);
        assert!(args.validate().is_ok());
    }

    #[test]
    fn completions_flag() {
        let args = parse(&["--completions", "bash"]);
        assert_eq!(args.completions, Some(Shell::Bash));
    }

    #[test]
    fn completions_flag_zsh() {
        let args = parse(&["--completions", "zsh"]);
        assert_eq!(args.completions, Some(Shell::Zsh));
    }

    #[test]
    fn unknown_flag_produces_error() {
        let result = CliArgs::try_parse_from(["cqlsh-rs", "--nonexistent"]);
        assert!(result.is_err());
    }
}

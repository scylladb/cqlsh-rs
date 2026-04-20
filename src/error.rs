//! Error classification and formatting for user-facing error display.
//!
//! Maps scylla driver error types to Python cqlsh-compatible error names
//! and strips verbose driver boilerplate to produce clean messages.

use scylla::errors::{DbError, ExecutionError, RequestAttemptError, RequestError};

/// Error categories matching Python cqlsh error display names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    SyntaxException,
    InvalidRequest,
    Unauthorized,
    Unavailable,
    ReadTimeout,
    WriteTimeout,
    ConfigurationException,
    AlreadyExists,
    Overloaded,
    IsBootstrapping,
    TruncateError,
    ReadFailure,
    WriteFailure,
    FunctionFailure,
    AuthenticationError,
    ServerError,
    ProtocolError,
    ConnectionError,
}

impl ErrorCategory {
    /// CQL protocol error code for this category.
    pub fn error_code(&self) -> Option<u32> {
        match self {
            Self::ServerError => Some(0x0000),
            Self::ProtocolError => Some(0x000A),
            Self::AuthenticationError => Some(0x0100),
            Self::Unavailable => Some(0x1000),
            Self::Overloaded => Some(0x1001),
            Self::IsBootstrapping => Some(0x1002),
            Self::TruncateError => Some(0x1003),
            Self::WriteTimeout => Some(0x1100),
            Self::ReadTimeout => Some(0x1200),
            Self::ReadFailure => Some(0x1300),
            Self::FunctionFailure => Some(0x1400),
            Self::WriteFailure => Some(0x1500),
            Self::SyntaxException => Some(0x2000),
            Self::Unauthorized => Some(0x2100),
            Self::InvalidRequest => Some(0x2200),
            Self::ConfigurationException => Some(0x2300),
            Self::AlreadyExists => Some(0x2400),
            Self::ConnectionError => None,
        }
    }

    /// Human-readable category label used in `Error from server` messages.
    fn server_label(&self) -> &'static str {
        match self {
            Self::ServerError => "Server error",
            Self::ProtocolError => "Protocol error",
            Self::AuthenticationError => "Bad credentials",
            Self::Unavailable => "Unavailable exception",
            Self::Overloaded => "Overloaded",
            Self::IsBootstrapping => "Is bootstrapping",
            Self::TruncateError => "Truncate error",
            Self::WriteTimeout => "Write timeout",
            Self::ReadTimeout => "Read timeout",
            Self::ReadFailure => "Read failure",
            Self::FunctionFailure => "Function failure",
            Self::WriteFailure => "Write failure",
            Self::SyntaxException => "Syntax error",
            Self::Unauthorized => "Unauthorized",
            Self::InvalidRequest => "Invalid query",
            Self::ConfigurationException => "Configuration error",
            Self::AlreadyExists => "Already exists",
            Self::ConnectionError => "Connection error",
        }
    }
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntaxException => write!(f, "SyntaxException"),
            Self::InvalidRequest => write!(f, "InvalidRequest"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::Unavailable => write!(f, "Unavailable"),
            Self::ReadTimeout => write!(f, "ReadTimeout"),
            Self::WriteTimeout => write!(f, "WriteTimeout"),
            Self::ConfigurationException => write!(f, "ConfigurationException"),
            Self::AlreadyExists => write!(f, "AlreadyExists"),
            Self::Overloaded => write!(f, "Overloaded"),
            Self::IsBootstrapping => write!(f, "IsBootstrapping"),
            Self::TruncateError => write!(f, "TruncateError"),
            Self::ReadFailure => write!(f, "ReadFailure"),
            Self::WriteFailure => write!(f, "WriteFailure"),
            Self::FunctionFailure => write!(f, "FunctionFailure"),
            Self::AuthenticationError => write!(f, "AuthenticationError"),
            Self::ServerError => write!(f, "ServerError"),
            Self::ProtocolError => write!(f, "ProtocolError"),
            Self::ConnectionError => write!(f, "ConnectionError"),
        }
    }
}

/// Classified error with category and cleaned message.
pub struct ClassifiedError {
    pub category: ErrorCategory,
    pub message: String,
}

/// Classify an anyhow error by walking the chain to find a DbError.
pub fn classify_error(error: &anyhow::Error) -> ClassifiedError {
    // Try direct downcast first, then walk the chain
    for cause in error.chain() {
        if let Some(exec_err) = cause.downcast_ref::<ExecutionError>() {
            if let Some(classified) = classify_execution_error(exec_err) {
                return classified;
            }
        }
        if let Some(req_err) = cause.downcast_ref::<RequestError>() {
            if let Some(classified) = classify_request_error(req_err) {
                return classified;
            }
        }
        if let Some(attempt_err) = cause.downcast_ref::<RequestAttemptError>() {
            if let Some(classified) = classify_attempt_error(attempt_err) {
                return classified;
            }
        }
    }

    // Fallback: use root cause message
    ClassifiedError {
        category: ErrorCategory::ServerError,
        message: error.root_cause().to_string(),
    }
}

/// Format a classified error for display matching Python cqlsh output.
pub fn format_error(error: &anyhow::Error) -> String {
    let classified = classify_error(error);
    match classified.category.error_code() {
        Some(code) => format!(
            "{}: Error from server: code={:04X} [{}] message=\"{}\"",
            classified.category,
            code,
            classified.category.server_label(),
            classified.message
        ),
        None => format!("{}: {}", classified.category, classified.message),
    }
}

/// Format a classified error with optional color (red bold when enabled).
pub fn format_error_colored(
    error: &anyhow::Error,
    colorizer: &crate::colorizer::CqlColorizer,
) -> String {
    let plain = format_error(error);
    colorizer.colorize_error(&plain)
}

fn categorize_db_error(db_error: &DbError) -> ErrorCategory {
    match db_error {
        DbError::SyntaxError => ErrorCategory::SyntaxException,
        DbError::Invalid => ErrorCategory::InvalidRequest,
        DbError::Unauthorized => ErrorCategory::Unauthorized,
        DbError::Unavailable { .. } => ErrorCategory::Unavailable,
        DbError::ReadTimeout { .. } => ErrorCategory::ReadTimeout,
        DbError::WriteTimeout { .. } => ErrorCategory::WriteTimeout,
        DbError::ConfigError => ErrorCategory::ConfigurationException,
        DbError::AlreadyExists { .. } => ErrorCategory::AlreadyExists,
        DbError::Overloaded => ErrorCategory::Overloaded,
        DbError::IsBootstrapping => ErrorCategory::IsBootstrapping,
        DbError::TruncateError => ErrorCategory::TruncateError,
        DbError::ReadFailure { .. } => ErrorCategory::ReadFailure,
        DbError::WriteFailure { .. } => ErrorCategory::WriteFailure,
        DbError::FunctionFailure { .. } => ErrorCategory::FunctionFailure,
        DbError::AuthenticationError => ErrorCategory::AuthenticationError,
        DbError::ServerError => ErrorCategory::ServerError,
        DbError::ProtocolError => ErrorCategory::ProtocolError,
        _ => ErrorCategory::ServerError,
    }
}

/// Clean the reason string from a DbError, stripping driver boilerplate.
fn clean_db_message(reason: &str) -> String {
    let cleaned = reason;
    // Strip nested prefixes — apply each in sequence
    let cleaned = cleaned
        .strip_prefix("The submitted query has a syntax error, ")
        .unwrap_or(cleaned);
    let cleaned = cleaned
        .strip_prefix("The query is syntactically correct but invalid, ")
        .unwrap_or(cleaned);
    let cleaned = cleaned.strip_prefix("Error message: ").unwrap_or(cleaned);
    cleaned.to_string()
}

fn classify_execution_error(err: &ExecutionError) -> Option<ClassifiedError> {
    match err {
        ExecutionError::LastAttemptError(attempt) => classify_attempt_error(attempt),
        ExecutionError::EmptyPlan => Some(ClassifiedError {
            category: ErrorCategory::ConnectionError,
            message: "No nodes available for query execution".to_string(),
        }),
        ExecutionError::RequestTimeout(dur) => Some(ClassifiedError {
            category: ErrorCategory::ReadTimeout,
            message: format!("Request timed out after {dur:?}"),
        }),
        _ => None,
    }
}

fn classify_request_error(err: &RequestError) -> Option<ClassifiedError> {
    match err {
        RequestError::LastAttemptError(attempt) => classify_attempt_error(attempt),
        RequestError::EmptyPlan => Some(ClassifiedError {
            category: ErrorCategory::ConnectionError,
            message: "No nodes available for query execution".to_string(),
        }),
        RequestError::RequestTimeout(dur) => Some(ClassifiedError {
            category: ErrorCategory::ReadTimeout,
            message: format!("Request timed out after {dur:?}"),
        }),
        _ => None,
    }
}

fn classify_attempt_error(err: &RequestAttemptError) -> Option<ClassifiedError> {
    match err {
        RequestAttemptError::DbError(db_error, reason) => {
            let category = categorize_db_error(db_error);
            let message = clean_db_message(reason);
            Some(ClassifiedError { category, message })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_display_names() {
        assert_eq!(
            ErrorCategory::SyntaxException.to_string(),
            "SyntaxException"
        );
        assert_eq!(ErrorCategory::InvalidRequest.to_string(), "InvalidRequest");
        assert_eq!(ErrorCategory::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(ErrorCategory::ServerError.to_string(), "ServerError");
        assert_eq!(
            ErrorCategory::ConfigurationException.to_string(),
            "ConfigurationException"
        );
    }

    #[test]
    fn categorize_syntax_error() {
        assert_eq!(
            categorize_db_error(&DbError::SyntaxError),
            ErrorCategory::SyntaxException
        );
    }

    #[test]
    fn categorize_invalid() {
        assert_eq!(
            categorize_db_error(&DbError::Invalid),
            ErrorCategory::InvalidRequest
        );
    }

    #[test]
    fn clean_strips_syntax_prefix() {
        let msg = clean_db_message(
            "The submitted query has a syntax error, Error message: line 1:0 no viable alternative at input 'SELEC'",
        );
        assert_eq!(msg, "line 1:0 no viable alternative at input 'SELEC'");
    }

    #[test]
    fn clean_strips_invalid_prefix() {
        let msg = clean_db_message(
            "The query is syntactically correct but invalid, Error message: unconfigured table foo",
        );
        assert_eq!(msg, "unconfigured table foo");
    }

    #[test]
    fn clean_preserves_already_clean() {
        let msg = clean_db_message("table foo does not exist");
        assert_eq!(msg, "table foo does not exist");
    }

    #[test]
    fn classify_syntax_from_execution_error() {
        let attempt = RequestAttemptError::DbError(
            DbError::SyntaxError,
            "Error message: line 1:0 no viable alternative at input 'SELEC'".to_string(),
        );
        let exec = ExecutionError::LastAttemptError(attempt);
        let err = anyhow::Error::new(exec);

        let classified = classify_error(&err);
        assert_eq!(classified.category, ErrorCategory::SyntaxException);
        assert_eq!(
            classified.message,
            "line 1:0 no viable alternative at input 'SELEC'"
        );
    }

    #[test]
    fn classify_invalid_from_execution_error() {
        let attempt = RequestAttemptError::DbError(
            DbError::Invalid,
            "Error message: unconfigured table no_such_table".to_string(),
        );
        let exec = ExecutionError::LastAttemptError(attempt);
        let err = anyhow::Error::new(exec);

        let classified = classify_error(&err);
        assert_eq!(classified.category, ErrorCategory::InvalidRequest);
        assert_eq!(classified.message, "unconfigured table no_such_table");
    }

    #[test]
    fn format_syntax_error() {
        let attempt = RequestAttemptError::DbError(
            DbError::SyntaxError,
            "Error message: line 1:0 bad input".to_string(),
        );
        let exec = ExecutionError::LastAttemptError(attempt);
        let err = anyhow::Error::new(exec);

        assert_eq!(format_error(&err), "SyntaxException: Error from server: code=2000 [Syntax error] message=\"line 1:0 bad input\"");
    }

    #[test]
    fn classify_through_anyhow_context() {
        let attempt = RequestAttemptError::DbError(
            DbError::SyntaxError,
            "Error message: line 1:0 bad input".to_string(),
        );
        let exec = ExecutionError::LastAttemptError(attempt);
        let err = anyhow::Error::new(exec).context("executing CQL query");

        let classified = classify_error(&err);
        assert_eq!(classified.category, ErrorCategory::SyntaxException);
    }

    #[test]
    fn classify_fallback_unknown() {
        let err = anyhow::anyhow!("something went wrong");
        let classified = classify_error(&err);
        assert_eq!(classified.category, ErrorCategory::ServerError);
        assert_eq!(classified.message, "something went wrong");
    }

    // --- Additional categorize_db_error variant tests ---

    #[test]
    fn categorize_unauthorized() {
        assert_eq!(
            categorize_db_error(&DbError::Unauthorized),
            ErrorCategory::Unauthorized
        );
    }

    #[test]
    fn categorize_unavailable() {
        assert_eq!(
            categorize_db_error(&DbError::Unavailable {
                consistency: scylla::frame::types::Consistency::Quorum,
                required: 2,
                alive: 1,
            }),
            ErrorCategory::Unavailable
        );
    }

    #[test]
    fn categorize_read_timeout() {
        assert_eq!(
            categorize_db_error(&DbError::ReadTimeout {
                consistency: scylla::frame::types::Consistency::One,
                received: 0,
                required: 1,
                data_present: false,
            }),
            ErrorCategory::ReadTimeout
        );
    }

    #[test]
    fn categorize_write_timeout() {
        assert_eq!(
            categorize_db_error(&DbError::WriteTimeout {
                consistency: scylla::frame::types::Consistency::Quorum,
                received: 1,
                required: 2,
                write_type: scylla::errors::WriteType::Simple,
            }),
            ErrorCategory::WriteTimeout
        );
    }

    #[test]
    fn categorize_config_error() {
        assert_eq!(
            categorize_db_error(&DbError::ConfigError),
            ErrorCategory::ConfigurationException
        );
    }

    #[test]
    fn categorize_already_exists() {
        assert_eq!(
            categorize_db_error(&DbError::AlreadyExists {
                keyspace: "ks".to_string(),
                table: "tbl".to_string(),
            }),
            ErrorCategory::AlreadyExists
        );
    }

    #[test]
    fn categorize_overloaded() {
        assert_eq!(
            categorize_db_error(&DbError::Overloaded),
            ErrorCategory::Overloaded
        );
    }

    #[test]
    fn categorize_is_bootstrapping() {
        assert_eq!(
            categorize_db_error(&DbError::IsBootstrapping),
            ErrorCategory::IsBootstrapping
        );
    }

    #[test]
    fn categorize_truncate_error() {
        assert_eq!(
            categorize_db_error(&DbError::TruncateError),
            ErrorCategory::TruncateError
        );
    }

    #[test]
    fn categorize_read_failure() {
        assert_eq!(
            categorize_db_error(&DbError::ReadFailure {
                consistency: scylla::frame::types::Consistency::One,
                received: 1,
                required: 1,
                numfailures: 1,
                data_present: false,
            }),
            ErrorCategory::ReadFailure
        );
    }

    #[test]
    fn categorize_write_failure() {
        assert_eq!(
            categorize_db_error(&DbError::WriteFailure {
                consistency: scylla::frame::types::Consistency::Quorum,
                received: 1,
                required: 2,
                numfailures: 1,
                write_type: scylla::errors::WriteType::Simple,
            }),
            ErrorCategory::WriteFailure
        );
    }

    #[test]
    fn categorize_function_failure() {
        assert_eq!(
            categorize_db_error(&DbError::FunctionFailure {
                keyspace: "ks".to_string(),
                function: "fn".to_string(),
                arg_types: vec!["int".to_string()],
            }),
            ErrorCategory::FunctionFailure
        );
    }

    #[test]
    fn categorize_authentication_error() {
        assert_eq!(
            categorize_db_error(&DbError::AuthenticationError),
            ErrorCategory::AuthenticationError
        );
    }

    #[test]
    fn categorize_server_error() {
        assert_eq!(
            categorize_db_error(&DbError::ServerError),
            ErrorCategory::ServerError
        );
    }

    #[test]
    fn categorize_protocol_error() {
        assert_eq!(
            categorize_db_error(&DbError::ProtocolError),
            ErrorCategory::ProtocolError
        );
    }

    // --- classify_execution_error / classify_request_error paths ---

    #[test]
    fn classify_empty_plan_execution() {
        let exec = ExecutionError::EmptyPlan;
        let err = anyhow::Error::new(exec);
        let classified = classify_error(&err);
        assert_eq!(classified.category, ErrorCategory::ConnectionError);
        assert!(classified.message.contains("No nodes available"));
    }

    #[test]
    fn classify_request_timeout_execution() {
        let exec = ExecutionError::RequestTimeout(std::time::Duration::from_secs(10));
        let err = anyhow::Error::new(exec);
        let classified = classify_error(&err);
        assert_eq!(classified.category, ErrorCategory::ReadTimeout);
        assert!(classified.message.contains("timed out"));
    }

    #[test]
    fn classify_empty_plan_request() {
        let req = RequestError::EmptyPlan;
        let err = anyhow::Error::new(req);
        let classified = classify_error(&err);
        assert_eq!(classified.category, ErrorCategory::ConnectionError);
        assert!(classified.message.contains("No nodes available"));
    }

    #[test]
    fn classify_request_timeout_request() {
        let req = RequestError::RequestTimeout(std::time::Duration::from_secs(5));
        let err = anyhow::Error::new(req);
        let classified = classify_error(&err);
        assert_eq!(classified.category, ErrorCategory::ReadTimeout);
        assert!(classified.message.contains("timed out"));
    }

    // --- format_error with no error code (ConnectionError) ---

    #[test]
    fn format_connection_error() {
        let exec = ExecutionError::EmptyPlan;
        let err = anyhow::Error::new(exec);
        let formatted = format_error(&err);
        assert!(formatted.starts_with("ConnectionError:"));
        assert!(!formatted.contains("code="));
    }

    // --- error_code tests ---

    #[test]
    fn error_codes_some_known() {
        assert_eq!(ErrorCategory::SyntaxException.error_code(), Some(0x2000));
        assert_eq!(ErrorCategory::InvalidRequest.error_code(), Some(0x2200));
        assert_eq!(ErrorCategory::Unavailable.error_code(), Some(0x1000));
        assert_eq!(ErrorCategory::ReadTimeout.error_code(), Some(0x1200));
        assert_eq!(ErrorCategory::WriteTimeout.error_code(), Some(0x1100));
        assert_eq!(ErrorCategory::ConnectionError.error_code(), None);
    }

    // --- server_label tests ---

    #[test]
    fn server_labels() {
        assert_eq!(
            ErrorCategory::SyntaxException.server_label(),
            "Syntax error"
        );
        assert_eq!(
            ErrorCategory::InvalidRequest.server_label(),
            "Invalid query"
        );
        assert_eq!(ErrorCategory::Unauthorized.server_label(), "Unauthorized");
        assert_eq!(
            ErrorCategory::Unavailable.server_label(),
            "Unavailable exception"
        );
        assert_eq!(ErrorCategory::Overloaded.server_label(), "Overloaded");
        assert_eq!(
            ErrorCategory::IsBootstrapping.server_label(),
            "Is bootstrapping"
        );
        assert_eq!(
            ErrorCategory::TruncateError.server_label(),
            "Truncate error"
        );
        assert_eq!(ErrorCategory::ReadFailure.server_label(), "Read failure");
        assert_eq!(ErrorCategory::WriteFailure.server_label(), "Write failure");
        assert_eq!(
            ErrorCategory::FunctionFailure.server_label(),
            "Function failure"
        );
        assert_eq!(
            ErrorCategory::AuthenticationError.server_label(),
            "Bad credentials"
        );
        assert_eq!(ErrorCategory::ServerError.server_label(), "Server error");
        assert_eq!(
            ErrorCategory::ProtocolError.server_label(),
            "Protocol error"
        );
        assert_eq!(
            ErrorCategory::ConnectionError.server_label(),
            "Connection error"
        );
    }

    // --- Display trait for all variants ---

    #[test]
    fn display_all_categories() {
        let categories = vec![
            (ErrorCategory::Unavailable, "Unavailable"),
            (ErrorCategory::ReadTimeout, "ReadTimeout"),
            (ErrorCategory::WriteTimeout, "WriteTimeout"),
            (ErrorCategory::Overloaded, "Overloaded"),
            (ErrorCategory::IsBootstrapping, "IsBootstrapping"),
            (ErrorCategory::TruncateError, "TruncateError"),
            (ErrorCategory::ReadFailure, "ReadFailure"),
            (ErrorCategory::WriteFailure, "WriteFailure"),
            (ErrorCategory::FunctionFailure, "FunctionFailure"),
            (ErrorCategory::AuthenticationError, "AuthenticationError"),
            (ErrorCategory::ProtocolError, "ProtocolError"),
            (ErrorCategory::ConnectionError, "ConnectionError"),
            (ErrorCategory::AlreadyExists, "AlreadyExists"),
        ];
        for (cat, expected) in categories {
            assert_eq!(cat.to_string(), expected);
        }
    }

    // --- classify_attempt_error with RequestError wrapping ---

    #[test]
    fn classify_db_error_through_request_error() {
        let attempt = RequestAttemptError::DbError(
            DbError::Unauthorized,
            "User has no permission".to_string(),
        );
        let req = RequestError::LastAttemptError(attempt);
        let err = anyhow::Error::new(req);
        let classified = classify_error(&err);
        assert_eq!(classified.category, ErrorCategory::Unauthorized);
        assert_eq!(classified.message, "User has no permission");
    }
}

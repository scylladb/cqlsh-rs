//! Integration test harness for cqlsh-rs.
//!
//! All integration tests are compiled as a single test binary to share
//! a single ScyllaDB container instance across all tests.
//!
//! Run with: cargo test --test integration -- --ignored
//! Or with nextest: cargo nextest run --test integration --run-ignored ignored-only
//!
//! Run a specific category:
//!   cargo test --test integration --features test-plain -- --ignored --test-threads=1
//!   cargo test --test integration --features test-ssl -- --ignored --test-threads=1

#[allow(dead_code)]
mod helpers;

#[cfg(feature = "test-plain")]
mod copy_from_tests;
#[cfg(feature = "test-plain")]
mod copy_tests;
#[cfg(feature = "test-plain")]
mod core_tests;
#[cfg(feature = "test-plain")]
mod describe_tests;
#[cfg(feature = "test-plain")]
mod dtest_copy_tests;
#[cfg(feature = "test-plain")]
mod dtest_cqlsh_tests;
#[cfg(feature = "test-plain")]
mod escape_tests;
#[cfg(feature = "test-plain")]
mod login_tests;
#[cfg(feature = "test-plain")]
mod output_tests;
#[cfg(feature = "test-plain")]
mod proxy_tests;
#[cfg(feature = "test-plain")]
mod schema_agreement_tests;
#[cfg(feature = "test-plain")]
mod unicode_tests;

#[cfg(feature = "test-auth")]
mod auth_tests;

#[cfg(feature = "test-ssl")]
mod ssl_tests;

#[cfg(feature = "test-maintenance")]
mod uds_tests;

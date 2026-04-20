//! Integration test harness for cqlsh-rs.
//!
//! All integration tests are compiled as a single test binary to share
//! a single ScyllaDB container instance across all tests.
//!
//! Run with: cargo test --test integration -- --ignored
//! Or with nextest: cargo nextest run --test integration --run-ignored ignored-only

mod copy_from_tests;
mod copy_tests;
mod core_tests;
mod describe_tests;
mod escape_tests;
mod helpers;
mod login_tests;
mod output_tests;
mod ssl_tests;
mod unicode_tests;

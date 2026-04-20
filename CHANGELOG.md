# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/scylladb/cqlsh-rs/compare/v0.2.0...v0.2.1) - 2026-04-20

### Fixed

- add WITH clause to DESCRIBE MATERIALIZED VIEW output

## [0.2.0](https://github.com/scylladb/cqlsh-rs/compare/v0.1.0...v0.2.0) - 2026-04-19

### Added

- add --cqlversion and --protocol-version compatibility warnings

### Fixed

- *(ci)* use RELEASE_PLZ_TOKEN for release-pr step
- *(ci)* remove broken release-pr step from release job
- *(ci)* handle 403 error in release-plz release-pr step
- correct DESCRIBE TABLE column ordering, add WITH clause, and include materialized views in DESCRIBE KEYSPACE
- handle SOURCE command client-side in non-interactive mode
- correct debug output assertion for encoding in ui_encoding_from_cqlshrc test
- *(deps)* update rust dependencies
- gracefully skip SSL tests when TLS container fails to start
- *(ci)* address code review feedback on release pipeline
- suppress tracing output for queries against system_traces
- include tables and indexes in DESCRIBE KEYSPACE output
- display null values as blank and trim trailing whitespace
- emit ANSI clear sequences for CLEAR/CLS in non-interactive mode
- print connect/request timeout and ssl in --debug mode
- use microsecond precision (6 digits) for timestamp formatting
- remove unnecessary .into_iter() calls to fix clippy warnings
- *(ci)* exclude benchmark dashboard URL from link checker
- update all fruch/cqlsh-rs links to point to scylladb/cqlsh-rs
- *(ci)* fix GitHub Pages docs deployment conflicts
- *(deps)* update rust crate rustyline to v18
- *(docs)* fix link checker failures in CI
- *(ci)* remove deprecated --exclude-mail flag from lychee link checker

### Other

- add cqlshrc configuration integration tests
- add TLS integration tests with real encrypted ScyllaDB container
- add CONTRIBUTING.md with release process and crates.io setup
- add automated release pipeline with release-plz
- add integration tests for all remaining CLI flags
- implement COPY FROM integration test stubs with real test logic
- update progress roadmap SVG [skip ci]
- *(cql_lexer)* replace tokenize-based strip_comments with zero-alloc scanner
- apply rustfmt formatting to cql_lexer, colorizer, completer
- *(SP18)* add unified CQL lexer with grammar-aware tokenization
- *(ci)* reduce benchmark CI time by lowering criterion warmup/measurement
- *(ci)* optimize benchmark workflow from ~21min to ~5-8min
- *(deps)* update alpine docker tag to v3.23
- *(deps)* update dependency python to 3.14
- configure Renovate for Rust and GitHub Actions dependency updates
- update progress roadmap SVG [skip ci]
- *(SP14)* add mdBook documentation site, CI workflow, and LLM files
- facelift README with community-standard style
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- release-plz: next-release -->

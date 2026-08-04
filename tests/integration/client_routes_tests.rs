//! Client-routes (PrivateLink) integration tests.
//!
//! These tests need a ScyllaDB build that ships `system.client_routes` and the
//! `/v2/client-routes` REST endpoints — ScyllaDB 2026.1 or later. Anything
//! older (and Apache Cassandra) is detected and skipped rather than failed.
//!
//! ## What is verified
//!
//! ```text
//! cqlsh-rs → forwarder A (contact point)          → ScyllaDB CQL port
//!            forwarder B (address in the table)   → ScyllaDB CQL port
//! ```
//!
//! The route entry published for the node points at **forwarder B** while
//! cqlsh-rs is told to contact **forwarder A**. Traffic arriving at B can only
//! be the result of the driver translating the node address through
//! `system.client_routes`; a single-forwarder setup would pass even with
//! routing disabled, because the node's own address is directly reachable from
//! the test host.
//!
//! Routes are cluster-wide state and the container is shared with other tests,
//! so every test uses a unique connection ID and deletes its entries at the end.
//!
//! Environment variables (used by CI instead of testcontainers):
//! - `CQLSH_TEST_HOST` / `CQLSH_TEST_PORT` — CQL endpoint
//! - `CQLSH_TEST_API_PORT` — ScyllaDB REST API port (default 10000)

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use predicates::prelude::*;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::SyncRunner;
use testcontainers::{Container, GenericImage, ImageExt};

use super::tcp_forwarder::TcpForwarder;

/// The native CQL transport port inside the container.
const CQL_PORT: u16 = 9042;
/// The ScyllaDB REST API port inside the container.
const API_PORT: u16 = 10000;
/// Image that ships `system.client_routes` (scylladb/scylladb#27323).
const IMAGE_TAG: &str = "2026.1";

// ---------------------------------------------------------------------------
// Container fixture
// ---------------------------------------------------------------------------

struct ClientRoutesScylla {
    _container: Option<Container<GenericImage>>,
    host: String,
    port: u16,
    api_port: u16,
}

impl ClientRoutesScylla {
    /// `host:port` of the real CQL endpoint, used as the forwarders' backend.
    fn cql_backend(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

type StartResult = Result<ClientRoutesScylla, String>;
static SCYLLA: OnceLock<StartResult> = OnceLock::new();

fn get_scylla() -> Option<&'static ClientRoutesScylla> {
    SCYLLA.get_or_init(start_scylla).as_ref().ok()
}

fn start_scylla() -> StartResult {
    // CI starts the container itself (testcontainers-rs cannot map ports
    // reliably on GitHub Actions runners) and points us at it via env vars.
    if let Ok(host) = std::env::var("CQLSH_TEST_HOST") {
        return Ok(ClientRoutesScylla {
            _container: None,
            port: env_port("CQLSH_TEST_PORT", CQL_PORT),
            api_port: env_port("CQLSH_TEST_API_PORT", API_PORT),
            host,
        });
    }

    let container = GenericImage::new("scylladb/scylla", IMAGE_TAG)
        .with_wait_for(WaitFor::message_on_stderr("serving"))
        .with_exposed_port(CQL_PORT.tcp())
        .with_exposed_port(API_PORT.tcp())
        .with_cmd(vec![
            "--smp".to_string(),
            "1".to_string(),
            "--memory".to_string(),
            "512M".to_string(),
            "--overprovisioned".to_string(),
            "1".to_string(),
            "--skip-wait-for-gossip-to-settle".to_string(),
            "0".to_string(),
            // The REST API binds to localhost by default, which is unreachable
            // from outside the container.
            "--api-address".to_string(),
            "0.0.0.0".to_string(),
        ])
        .with_startup_timeout(Duration::from_secs(180))
        .start()
        .map_err(|e| format!("failed to start ScyllaDB container: {e}"))?;

    let port = container
        .get_host_port_ipv4(CQL_PORT)
        .map_err(|e| format!("failed to get mapped CQL port: {e}"))?;
    let api_port = container
        .get_host_port_ipv4(API_PORT)
        .map_err(|e| format!("failed to get mapped API port: {e}"))?;
    let host = container
        .get_host()
        .map_err(|e| format!("failed to get container host: {e}"))?
        .to_string();

    // CQL becomes usable slightly after the "serving" log line.
    std::thread::sleep(Duration::from_secs(5));

    Ok(ClientRoutesScylla {
        _container: Some(container),
        host,
        port,
        api_port,
    })
}

fn env_port(name: &str, default: u16) -> u16 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Skip the test unless a cluster with `system.client_routes` is available.
macro_rules! require_client_routes {
    () => {
        match get_scylla() {
            Some(scylla) if client_routes_supported(scylla) => scylla,
            Some(_) => {
                eprintln!(
                    "Skipping test: cluster has no system.client_routes table \
                     (needs ScyllaDB {}+)",
                    IMAGE_TAG
                );
                return;
            }
            None => {
                eprintln!("Skipping test: ScyllaDB container unavailable (Docker issue)");
                return;
            }
        }
    };
}

// ---------------------------------------------------------------------------
// CQL + REST helpers
// ---------------------------------------------------------------------------

static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn tokio_rt() -> &'static tokio::runtime::Runtime {
    TOKIO_RT.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime")
    })
}

/// Run a query against the cluster directly (no client routes involved).
fn run_cql(scylla: &ClientRoutesScylla, cql: &str) -> String {
    tokio_rt()
        .block_on(cqlsh_rs::run_cql_in_process(
            &scylla.host,
            scylla.port,
            None,
            cql,
        ))
        .unwrap_or_else(|e| panic!("in-process cqlsh execution failed: {e}"))
}

fn client_routes_supported(scylla: &ClientRoutesScylla) -> bool {
    let output = run_cql(
        scylla,
        "SELECT table_name FROM system_schema.tables \
         WHERE keyspace_name = 'system' AND table_name = 'client_routes';",
    );
    output.contains("client_routes")
}

/// Read this node's `host_id`, which is the key routes are published under.
fn host_id(scylla: &ClientRoutesScylla) -> String {
    let output = run_cql(scylla, "SELECT host_id FROM system.local;");
    extract_uuid(&output).unwrap_or_else(|| panic!("no host_id in system.local output:\n{output}"))
}

/// Pull the first UUID out of formatted cqlsh output.
fn extract_uuid(text: &str) -> Option<String> {
    text.split(|c: char| !(c.is_ascii_hexdigit() || c == '-'))
        .find(|token| {
            token.len() == 36
                && token.as_bytes().iter().enumerate().all(|(i, b)| {
                    if matches!(i, 8 | 13 | 18 | 23) {
                        *b == b'-'
                    } else {
                        b.is_ascii_hexdigit()
                    }
                })
        })
        .map(str::to_string)
}

/// A `system.client_routes` entry, published and removed via the REST API.
struct Route {
    connection_id: String,
    host_id: String,
    address: String,
    port: u16,
}

impl Route {
    fn to_json(&self) -> String {
        format!(
            r#"{{"connection_id":"{}","host_id":"{}","address":"{}","port":{},"tls_port":{}}}"#,
            self.connection_id,
            self.host_id,
            self.address,
            self.port,
            // The REST API rejects an entry with neither port set; TLS is not
            // supported with client routes, so the value itself is unused.
            self.port
        )
    }

    fn key_json(&self) -> String {
        format!(
            r#"{{"connection_id":"{}","host_id":"{}"}}"#,
            self.connection_id, self.host_id
        )
    }
}

/// Publishes routes on construction and deletes them on drop.
struct PublishedRoutes<'a> {
    scylla: &'a ClientRoutesScylla,
    routes: Vec<Route>,
}

impl<'a> PublishedRoutes<'a> {
    fn publish(scylla: &'a ClientRoutesScylla, routes: Vec<Route>) -> Self {
        let body = format!(
            "[{}]",
            routes
                .iter()
                .map(Route::to_json)
                .collect::<Vec<_>>()
                .join(",")
        );
        rest_request(scylla, "POST", &body).expect("publishing client routes");
        Self { scylla, routes }
    }
}

impl Drop for PublishedRoutes<'_> {
    fn drop(&mut self) {
        let body = format!(
            "[{}]",
            self.routes
                .iter()
                .map(Route::key_json)
                .collect::<Vec<_>>()
                .join(",")
        );
        if let Err(e) = rest_request(self.scylla, "DELETE", &body) {
            eprintln!("Warning: could not delete client routes: {e}");
        }
    }
}

/// Send a request to `/v2/client-routes`, retrying while the API warms up.
///
/// Hand-rolled HTTP/1.1 keeps the test suite free of an HTTP client
/// dependency; the request and response are both trivial.
fn rest_request(scylla: &ClientRoutesScylla, method: &str, body: &str) -> Result<(), String> {
    const ATTEMPTS: u32 = 10;
    let mut last_err = String::new();

    for attempt in 1..=ATTEMPTS {
        match rest_request_once(scylla, method, body) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = e;
                if attempt < ATTEMPTS {
                    std::thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }

    Err(format!("{method} /v2/client-routes failed: {last_err}"))
}

fn rest_request_once(scylla: &ClientRoutesScylla, method: &str, body: &str) -> Result<(), String> {
    let addr = format!("{}:{}", scylla.host, scylla.api_port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("connect {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;

    let request = format!(
        "{method} /v2/client-routes HTTP/1.1\r\n\
         Host: {addr}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("send: {e}"))?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|e| format!("read: {e}"))?;
    let response = String::from_utf8_lossy(&response);
    let status = response.lines().next().unwrap_or("(empty response)");

    if status.contains("200") || status.contains("201") {
        Ok(())
    } else {
        Err(format!("unexpected status '{status}'\n{response}"))
    }
}

/// A connection ID unique to each test, so tests do not see each other's routes.
fn unique_connection_id(prefix: &str) -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    format!(
        "cqlsh-rs-{prefix}-{:x}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    )
}

fn cqlsh() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("cqlsh-rs").unwrap()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn routes_traffic_through_configured_proxy() {
    let scylla = require_client_routes!();
    let connection_id = unique_connection_id("route");

    let contact = TcpForwarder::start(scylla.cql_backend()).unwrap();
    let routed = TcpForwarder::start(scylla.cql_backend()).unwrap();

    let _routes = PublishedRoutes::publish(
        scylla,
        vec![Route {
            connection_id: connection_id.clone(),
            host_id: host_id(scylla),
            address: "127.0.0.1".to_string(),
            port: routed.port(),
        }],
    );

    cqlsh()
        .args([
            "127.0.0.1",
            &contact.port().to_string(),
            "--client-route",
            &connection_id,
            "-e",
            "SELECT release_version FROM system.local;",
        ])
        .timeout(Duration::from_secs(60))
        .assert()
        .success()
        .stdout(predicate::str::contains("release_version"));

    assert!(
        routed.connections() >= 1,
        "no traffic reached the routed endpoint — address translation did not happen"
    );
}

#[test]
#[ignore = "requires Docker"]
fn routes_configured_via_cqlshrc() {
    let scylla = require_client_routes!();
    let connection_id = unique_connection_id("cqlshrc");

    let contact = TcpForwarder::start(scylla.cql_backend()).unwrap();
    let routed = TcpForwarder::start(scylla.cql_backend()).unwrap();

    let _routes = PublishedRoutes::publish(
        scylla,
        vec![Route {
            connection_id: connection_id.clone(),
            host_id: host_id(scylla),
            address: "127.0.0.1".to_string(),
            port: routed.port(),
        }],
    );

    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    std::fs::write(
        &cqlshrc,
        format!(
            "[connection]\nhostname = 127.0.0.1\nport = {}\n\
             [client_routes]\nproxies = {connection_id}\n",
            contact.port()
        ),
    )
    .unwrap();

    cqlsh()
        .args([
            "--cqlshrc",
            cqlshrc.to_str().unwrap(),
            "-e",
            "SELECT release_version FROM system.local;",
        ])
        .timeout(Duration::from_secs(60))
        .assert()
        .success()
        .stdout(predicate::str::contains("release_version"));

    assert!(
        routed.connections() >= 1,
        "no traffic reached the routed endpoint configured via cqlshrc"
    );
}

#[test]
#[ignore = "requires Docker"]
fn address_override_replaces_table_hostname() {
    let scylla = require_client_routes!();
    let connection_id = unique_connection_id("override");

    let contact = TcpForwarder::start(scylla.cql_backend()).unwrap();
    let routed = TcpForwarder::start(scylla.cql_backend()).unwrap();

    // The hostname published in the table does not resolve; only the override
    // passed on the command line can make this connection work. The port still
    // comes from the table, which is why the override is just a hostname.
    let _routes = PublishedRoutes::publish(
        scylla,
        vec![Route {
            connection_id: connection_id.clone(),
            host_id: host_id(scylla),
            address: "unreachable.invalid".to_string(),
            port: routed.port(),
        }],
    );

    cqlsh()
        .args([
            "127.0.0.1",
            &contact.port().to_string(),
            "--client-route",
            &format!("{connection_id}=127.0.0.1"),
            "-e",
            "SELECT release_version FROM system.local;",
        ])
        .timeout(Duration::from_secs(60))
        .assert()
        .success()
        .stdout(predicate::str::contains("release_version"));

    assert!(
        routed.connections() >= 1,
        "no traffic reached the routed endpoint via the hostname override"
    );
}

#[test]
#[ignore = "requires Docker"]
fn contact_point_derived_from_route_address() {
    let scylla = require_client_routes!();
    let connection_id = unique_connection_id("contact");

    let routed = TcpForwarder::start(scylla.cql_backend()).unwrap();

    let _routes = PublishedRoutes::publish(
        scylla,
        vec![Route {
            connection_id: connection_id.clone(),
            host_id: host_id(scylla),
            address: "127.0.0.1".to_string(),
            port: routed.port(),
        }],
    );

    // With no host argument, the route's address override becomes the contact
    // point (as in Python cqlsh). The port is not part of the override, so the
    // attempt targets the default 9042 and only the derivation is asserted.
    cqlsh()
        .args([
            "--client-route",
            &format!("{connection_id}=127.0.0.1"),
            "--debug",
            "--connect-timeout",
            "2",
            "-e",
            "SELECT release_version FROM system.local;",
        ])
        .timeout(Duration::from_secs(60))
        .assert()
        .stderr(predicate::str::contains("Using client routes: true"))
        .stderr(predicate::str::contains("Debug: resolved host=127.0.0.1"))
        .stderr(predicate::str::contains(
            "Debug: contact points=127.0.0.1:9042",
        ));
}

#[test]
#[ignore = "requires Docker"]
fn unknown_connection_id_cannot_reach_nodes() {
    let scylla = require_client_routes!();
    let contact = TcpForwarder::start(scylla.cql_backend()).unwrap();

    // No route is published for this connection ID. The control connection to
    // the contact point still succeeds, so the session builds; the failure
    // surfaces on the first query, when the driver has no translated address
    // for the node. That is a CQL-level error (exit 1), not a connection
    // failure (exit 2).
    cqlsh()
        .args([
            "127.0.0.1",
            &contact.port().to_string(),
            "--client-route",
            &unique_connection_id("missing"),
            "--connect-timeout",
            "5",
            "-e",
            "SELECT release_version FROM system.local;",
        ])
        .timeout(Duration::from_secs(60))
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Address translation failed"));
}

#[test]
#[ignore = "requires Docker"]
fn login_preserves_client_routes() {
    let scylla = require_client_routes!();
    let connection_id = unique_connection_id("login");

    let contact = TcpForwarder::start(scylla.cql_backend()).unwrap();
    let routed = TcpForwarder::start(scylla.cql_backend()).unwrap();

    let _routes = PublishedRoutes::publish(
        scylla,
        vec![Route {
            connection_id: connection_id.clone(),
            host_id: host_id(scylla),
            address: "127.0.0.1".to_string(),
            port: routed.port(),
        }],
    );

    let before = routed.connections();

    // LOGIN rebuilds the session from the merged config; if client routes were
    // dropped there, the reconnect could not reach the node at all.
    cqlsh()
        .args([
            "127.0.0.1",
            &contact.port().to_string(),
            "--client-route",
            &connection_id,
        ])
        .write_stdin("LOGIN cassandra cassandra\nSELECT release_version FROM system.local;\n")
        .timeout(Duration::from_secs(60))
        .assert()
        .success()
        .stdout(predicate::str::contains("release_version"));

    assert!(
        routed.connections() > before,
        "reconnect after LOGIN did not go through the routed endpoint"
    );
}

#[test]
#[ignore = "requires Docker"]
fn advanced_shard_awareness_flag_connects() {
    let scylla = require_client_routes!();
    let connection_id = unique_connection_id("shardaware");

    let contact = TcpForwarder::start(scylla.cql_backend()).unwrap();
    let routed = TcpForwarder::start(scylla.cql_backend()).unwrap();

    let _routes = PublishedRoutes::publish(
        scylla,
        vec![Route {
            connection_id: connection_id.clone(),
            host_id: host_id(scylla),
            address: "127.0.0.1".to_string(),
            port: routed.port(),
        }],
    );

    // Re-enabling the shard-aware port must not break a plain connection; the
    // driver falls back to the non-shard-aware port when it is unusable.
    cqlsh()
        .args([
            "127.0.0.1",
            &contact.port().to_string(),
            "--client-route",
            &connection_id,
            "--client-routes-advanced-shard-awareness",
            "-e",
            "SELECT release_version FROM system.local;",
        ])
        .timeout(Duration::from_secs(60))
        .assert()
        .success()
        .stdout(predicate::str::contains("release_version"));
}

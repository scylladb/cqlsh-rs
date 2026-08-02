//! Integration tests for Unix domain socket (UDS) proxy support.
//!
//! These tests start a separate ScyllaDB container with `--maintenance-socket workdir`
//! and a bind mount, then connect cqlsh-rs via the exposed `cql.m` socket file.
//! Set `CQLSH_TEST_MAINTENANCE_SOCKET` to a socket path to run against an
//! externally provisioned instance instead (no Docker required).
//!
//! Key lessons from development:
//! - Use `--maintenance-socket workdir` (not `listen`) — `listen` is misinterpreted as a path.
//! - The socket file `cql.m` is created in ScyllaDB's workdir (`/var/lib/scylla/` by default).
//! - Bind-mount the workdir to a host temp directory to access `cql.m`.
//! - Wait for "Starting listening for maintenance CQL clients" in stderr before connecting.
//! - The `ProxyAddressTranslator` is required because the driver discovers nodes by `rpc_address`
//!   (container-internal IP) and would try to connect there instead of through the proxy.

#[cfg(unix)]
mod unix {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::Duration;

    use assert_cmd::Command;
    use predicates::prelude::*;
    use testcontainers::core::{ExecCommand, Mount, WaitFor};
    use testcontainers::runners::SyncRunner;
    use testcontainers::{GenericImage, ImageExt};

    fn socket_path(host_dir: &std::path::Path) -> PathBuf {
        host_dir.join("cql.m")
    }

    fn wait_for_socket(path: &std::path::Path, timeout: Duration) {
        use std::os::unix::fs::FileTypeExt;
        let start = std::time::Instant::now();
        loop {
            if let Ok(meta) = std::fs::metadata(path) {
                if meta.file_type().is_socket() {
                    return;
                }
            }
            if start.elapsed() > timeout {
                panic!("Timed out waiting for socket at {}", path.display());
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    struct UdsScylla {
        /// `Some` when this fixture started its own container; `None` when an
        /// externally provisioned socket is used.
        _container: Option<testcontainers::Container<GenericImage>>,
        _socket_dir: Option<tempfile::TempDir>,
        socket: PathBuf,
    }

    impl UdsScylla {
        /// Use the socket provided via `CQLSH_TEST_MAINTENANCE_SOCKET` (the
        /// `[maintenance]` category contract in `tests/test_categories.toml`)
        /// when set, otherwise start a ScyllaDB container with a maintenance
        /// socket exposed through a bind mount.
        fn start() -> Self {
            if let Ok(path) = std::env::var("CQLSH_TEST_MAINTENANCE_SOCKET") {
                let socket = PathBuf::from(path);
                wait_for_socket(&socket, Duration::from_secs(30));
                return UdsScylla {
                    _container: None,
                    _socket_dir: None,
                    socket,
                };
            }

            let socket_dir = tempfile::TempDir::new().expect("create temp dir for UDS");
            // Make writable by container process (runs as root or uid 999)
            std::fs::set_permissions(socket_dir.path(), std::fs::Permissions::from_mode(0o777))
                .expect("chmod temp dir");
            let host_path = socket_dir.path().to_str().unwrap().to_string();

            // Pinned like the other fixtures (tests/integration/helpers.rs) so
            // image updates can't silently change maintenance-socket behavior.
            // 2025.1 matches the CI integration matrix.
            let container = GenericImage::new("scylladb/scylla", "2025.1")
                .with_wait_for(WaitFor::message_on_stderr(
                    "Starting listening for maintenance CQL clients",
                ))
                .with_mount(Mount::bind_mount(host_path, "/var/lib/scylla"))
                .with_cmd(vec![
                    "--smp".to_string(),
                    "1".to_string(),
                    "--memory".to_string(),
                    "512M".to_string(),
                    "--overprovisioned".to_string(),
                    "1".to_string(),
                    "--skip-wait-for-gossip-to-settle".to_string(),
                    "0".to_string(),
                    "--maintenance-socket".to_string(),
                    "workdir".to_string(),
                ])
                .with_startup_timeout(Duration::from_secs(120))
                .start()
                .expect("start ScyllaDB with maintenance socket");

            let sock = socket_path(socket_dir.path());
            wait_for_socket(&sock, Duration::from_secs(30));
            // Scylla creates cql.m under the container's uid/gid; make it
            // connectable from the host test process regardless of the
            // image's default socket mode.
            container
                .exec(ExecCommand::new(["chmod", "0666", "/var/lib/scylla/cql.m"]))
                .expect("chmod maintenance socket");
            std::thread::sleep(Duration::from_secs(5));

            UdsScylla {
                _container: Some(container),
                _socket_dir: Some(socket_dir),
                socket: sock,
            }
        }

        fn socket(&self) -> String {
            self.socket.to_str().unwrap().to_string()
        }
    }

    #[test]
    #[ignore = "requires Docker"]
    fn test_uds_select_system_local() {
        let scylla = UdsScylla::start();

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                &scylla.socket(),
                "-e",
                "SELECT release_version FROM system.local;",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("release_version"));
    }

    #[test]
    #[ignore = "requires Docker"]
    fn test_uds_ddl_dml_query() {
        let scylla = UdsScylla::start();
        let sock = scylla.socket();

        // Unique per-run keyspace so runs against a persistent external
        // instance (CQLSH_TEST_MAINTENANCE_SOCKET) never collide with —
        // or drop — pre-existing data.
        let ks = format!(
            "uds_test_{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % 0xFFFFFF
        );

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                &sock,
                "-e",
                &format!(
                    "CREATE KEYSPACE IF NOT EXISTS {ks} \
                     WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}};"
                ),
            ])
            .assert()
            .success();

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                &sock,
                "-e",
                &format!("CREATE TABLE {ks}.items (id int PRIMARY KEY, val text);"),
            ])
            .assert()
            .success();

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                &sock,
                "-e",
                &format!("INSERT INTO {ks}.items (id, val) VALUES (42, 'hello-uds');"),
            ])
            .assert()
            .success();

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                &sock,
                "-e",
                &format!("SELECT * FROM {ks}.items WHERE id = 42;"),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("hello-uds"));

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([&sock, "-e", &format!("DROP KEYSPACE IF EXISTS {ks};")])
            .assert()
            .success();
    }

    #[test]
    #[ignore = "integration"]
    fn test_uds_rejects_ssl_combination() {
        // The SSL+UDS combination is rejected client-side before any
        // connection is attempted, so a bare listening socket is enough —
        // no ScyllaDB container (or Docker) needed.
        let dir = tempfile::TempDir::new().expect("create temp dir");
        let sock = dir.path().join("cql.m");
        let _listener =
            std::os::unix::net::UnixListener::bind(&sock).expect("bind placeholder UDS");

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                sock.to_str().unwrap(),
                "--ssl",
                "-e",
                "SELECT * FROM system.local;",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "SSL is not supported with Unix domain socket connections",
            ));
    }
}

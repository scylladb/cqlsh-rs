
//! Integration tests for Unix domain socket (UDS) proxy support.
//!
//! These tests start a separate ScyllaDB container with `--maintenance-socket workdir`
//! and a bind mount, then connect cqlsh-rs via the exposed `cql.m` socket file.
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
    use testcontainers::core::{Mount, WaitFor};
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
        _container: testcontainers::Container<GenericImage>,
        socket_dir: tempfile::TempDir,
    }

    impl UdsScylla {
        fn start() -> Self {
            let socket_dir = tempfile::TempDir::new().expect("create temp dir for UDS");
            // Make writable by container process (runs as root or uid 999)
            std::fs::set_permissions(socket_dir.path(), std::fs::Permissions::from_mode(0o777))
                .expect("chmod temp dir");
            let host_path = socket_dir.path().to_str().unwrap().to_string();

            let container = GenericImage::new("scylladb/scylla", "latest")
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
            std::thread::sleep(Duration::from_secs(5));

            UdsScylla {
                _container: container,
                socket_dir,
            }
        }

        fn socket(&self) -> String {
            socket_path(self.socket_dir.path())
                .to_str()
                .unwrap()
                .to_string()
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

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                &sock,
                "-e",
                "CREATE KEYSPACE IF NOT EXISTS uds_test \
                 WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};",
            ])
            .assert()
            .success();

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                &sock,
                "-e",
                "CREATE TABLE uds_test.items (id int PRIMARY KEY, val text);",
            ])
            .assert()
            .success();

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                &sock,
                "-e",
                "INSERT INTO uds_test.items (id, val) VALUES (42, 'hello-uds');",
            ])
            .assert()
            .success();

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([&sock, "-e", "SELECT * FROM uds_test.items WHERE id = 42;"])
            .assert()
            .success()
            .stdout(predicate::str::contains("hello-uds"));

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([&sock, "-e", "DROP KEYSPACE IF EXISTS uds_test;"])
            .assert()
            .success();
    }

    #[test]
    #[ignore = "requires Docker"]
    fn test_uds_rejects_ssl_combination() {
        let scylla = UdsScylla::start();

        Command::cargo_bin("cqlsh-rs")
            .unwrap()
            .args([
                &scylla.socket(),
                "--ssl",
                "-e",
                "SELECT * FROM system.local;",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("SSL").or(predicate::str::contains("ssl")));
    }
}

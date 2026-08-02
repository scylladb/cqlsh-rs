//! Unix Domain Socket (UDS) detection and TCP-to-UDS proxy.
//!
//! When cqlsh-rs is given a path to a Unix domain socket instead of a
//! hostname, this module transparently proxies TCP connections (which the
//! Scylla driver always opens) to the UDS endpoint.
//!
//! # Platform support
//! All UDS-specific code is guarded with `#[cfg(unix)]`.  Non-Unix builds
//! compile the stubs that always return `false` / an error.
//!
//! # Security
//! The proxy listens on an ephemeral TCP port bound to `127.0.0.1`.  While the
//! session is open, any local process that can connect to loopback can reach
//! the Unix socket through the proxy, bypassing the socket file's ownership
//! and mode checks.  This matches the trust model of an interactive
//! single-user shell, but is a real widening of the socket's trust boundary
//! on multi-user hosts — don't rely on socket file permissions for isolation
//! there.  Native UDS support in the driver (SP21b, tracked in
//! [scylladb/scylla-rust-driver#1616]) will remove the proxy and this caveat.
//!
//! [scylladb/scylla-rust-driver#1616]: https://github.com/scylladb/scylla-rust-driver/issues/1616

#[cfg(unix)]
use anyhow::Context as _;

/// Errors for user-visible UDS misconfiguration, distinguishable from generic
/// connection failures (which cqlsh reports with its own compatible message).
#[derive(Debug, thiserror::Error)]
pub enum UdsError {
    /// SSL/TLS cannot be layered over the TCP-to-UDS proxy.
    #[error("SSL is not supported with Unix domain socket connections")]
    SslNotSupported,
    /// UDS connections require a Unix platform.
    #[error("Unix domain sockets are not supported on this platform")]
    NotSupportedOnPlatform,
}

/// Returns `true` if `host` is written like a filesystem path (`/…` or `./…`)
/// rather than a hostname.  Used to give a clear error on platforms without
/// Unix socket support, where [`is_unix_socket`] can never return `true`.
pub fn looks_like_uds_path(host: &str) -> bool {
    host.starts_with('/') || host.starts_with("./")
}

/// Returns `true` if `path` refers to a Unix domain socket on the filesystem.
///
/// Uses `std::fs::metadata` which follows symlinks, so a symlink pointing at
/// a socket will return `true`.  Always returns `false` on non-Unix platforms.
pub fn is_unix_socket(path: &str) -> bool {
    cfg_select! {
        unix => {
            use std::os::unix::fs::FileTypeExt;
            std::fs::metadata(path)
                .map(|m| m.file_type().is_socket())
                .unwrap_or(false)
        }
        _ => {
            let _ = path;
            false
        }
    }
}

// ── UdsProxy (unix) ────────────────────────────────────────────────────────

/// RAII handle that aborts the background proxy listener task — and with it
/// every active relay connection — when dropped.
///
/// Obtain one via [`start_uds_proxy`].
#[cfg(unix)]
pub struct UdsProxy {
    abort_handle: tokio::task::AbortHandle,
}

#[cfg(unix)]
impl Drop for UdsProxy {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}

// ── start_uds_proxy (unix) ─────────────────────────────────────────────────

/// Bind an ephemeral TCP port on `127.0.0.1` and forward every connection to
/// the Unix domain socket at `socket_path`.
///
/// Returns the bound `SocketAddr` (pass it to the driver as the contact point)
/// and a [`UdsProxy`] RAII guard.  Dropping the guard aborts the listener and
/// every relay connection it has accepted, so access to the socket ends with
/// the session.
///
/// The proxy accepts connections in a loop and spawns a bidirectional copy
/// task per connection using [`tokio::io::copy_bidirectional`], which handles
/// half-close correctly.
#[cfg(unix)]
pub async fn start_uds_proxy(
    socket_path: &str,
) -> anyhow::Result<(std::net::SocketAddr, UdsProxy)> {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("binding proxy TCP listener")?;
    let local_addr = listener.local_addr()?;
    let socket_path = socket_path.to_owned();
    let socket_path_for_log = socket_path.clone();

    let join_handle = tokio::spawn(accept_loop(listener, socket_path));

    let abort_handle = join_handle.abort_handle();
    drop(join_handle);

    tracing::debug!("UDS proxy started on {local_addr} → UDS {socket_path_for_log}");
    Ok((local_addr, UdsProxy { abort_handle }))
}

#[cfg(unix)]
async fn accept_loop(listener: tokio::net::TcpListener, socket_path: String) {
    // Relays are owned by this JoinSet rather than detached with
    // `tokio::spawn`: aborting the accept loop drops the set, which aborts
    // every in-flight relay, so dropping `UdsProxy` cuts off established
    // connections too — not just new ones.
    let mut relays = tokio::task::JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((tcp_stream, _peer)) => {
                    relays.spawn(relay_connection(tcp_stream, socket_path.clone()));
                }
                Err(e) => {
                    tracing::debug!("UDS proxy listener error: {e}");
                    break;
                }
            },
            // Reap finished relays so the set doesn't grow with connection
            // count over the session's lifetime.
            Some(_) = relays.join_next() => {}
        }
    }
}

#[cfg(unix)]
async fn relay_connection(mut tcp_stream: tokio::net::TcpStream, path: String) {
    use tokio::io::copy_bidirectional;
    use tokio::net::UnixStream;

    match UnixStream::connect(&path).await {
        Ok(mut uds_stream) => {
            if let Err(e) = copy_bidirectional(&mut tcp_stream, &mut uds_stream).await {
                tracing::debug!("UDS proxy connection closed: {e}");
            }
        }
        Err(e) => {
            tracing::warn!("UDS proxy: failed to connect to {path}: {e}");
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    // ── Helpers ────────────────────────────────────────────────────────────

    /// Create a unique temp path for a UDS without an external crate.
    fn temp_socket_path(suffix: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("cqlsh_rs_test_{}_{suffix}", std::process::id()));
        p
    }

    // ── looks_like_uds_path tests ──────────────────────────────────────────

    #[test]
    fn test_looks_like_uds_path() {
        assert!(looks_like_uds_path("/var/run/scylla/cql.m"));
        assert!(looks_like_uds_path("./cql.m"));
        assert!(!looks_like_uds_path("localhost"));
        assert!(!looks_like_uds_path("192.168.1.1"));
        assert!(!looks_like_uds_path("scylla.example.com"));
    }

    // ── is_unix_socket tests ───────────────────────────────────────────────

    #[test]
    fn test_is_unix_socket_with_real_socket() {
        let path = temp_socket_path("uds_real");
        let _ = std::fs::remove_file(&path);

        let _listener = UnixListener::bind(&path).expect("failed to bind test UDS");

        assert!(
            is_unix_socket(path.to_str().unwrap()),
            "should return true for a real Unix domain socket"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_is_unix_socket_regular_file() {
        let path = temp_socket_path("regular_file");
        std::fs::write(&path, b"hello").expect("failed to create temp file");

        assert!(
            !is_unix_socket(path.to_str().unwrap()),
            "should return false for a regular file"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_is_unix_socket_nonexistent() {
        assert!(
            !is_unix_socket("/tmp/definitely_does_not_exist_cqlsh_rs_xyz"),
            "should return false for a nonexistent path"
        );
    }

    #[test]
    fn test_is_unix_socket_directory() {
        assert!(
            !is_unix_socket("/tmp"),
            "should return false for a directory"
        );
    }

    // ── proxy tests ────────────────────────────────────────────────────────

    /// Bind a UDS at `path` and echo bytes back on every connection, using
    /// the blocking std API in detached background threads.
    fn spawn_echo_server(path: &std::path::Path) {
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path).expect("echo server bind");

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut s) => {
                        std::thread::spawn(move || {
                            let mut buf = [0u8; 4096];
                            loop {
                                match s.read(&mut buf) {
                                    Ok(0) | Err(_) => break,
                                    Ok(n) => {
                                        if s.write_all(&buf[..n]).is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(_) => break,
                }
            }
        });
    }

    #[tokio::test]
    async fn test_proxy_concurrent_connections() {
        let socket_path = temp_socket_path("echo_srv");
        spawn_echo_server(&socket_path);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let (addr, _proxy) = start_uds_proxy(socket_path.to_str().unwrap())
            .await
            .expect("start_uds_proxy");

        let payloads: &[&[u8]] = &[b"hello", b"world!", b"rust42"];
        let mut handles = Vec::new();

        for &payload in payloads {
            let owned = payload.to_vec();
            handles.push(tokio::spawn(async move {
                let mut tcp = TcpStream::connect(addr)
                    .await
                    .expect("tcp connect to proxy");
                tcp.write_all(&owned).await.expect("write");
                tcp.shutdown().await.expect("shutdown write");

                let mut response = Vec::new();
                tcp.read_to_end(&mut response).await.expect("read");
                assert_eq!(response, owned, "echo mismatch");
            }));
        }

        for h in handles {
            h.await.expect("client task panicked");
        }

        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn test_proxy_cleanup_on_drop() {
        let socket_path = temp_socket_path("cleanup_srv");
        spawn_echo_server(&socket_path);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let (addr, proxy) = start_uds_proxy(socket_path.to_str().unwrap())
            .await
            .expect("start_uds_proxy");

        TcpStream::connect(addr).await.expect("proxy should be up");

        drop(proxy);

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let result = TcpStream::connect(addr).await;
        assert!(result.is_err(), "proxy should be stopped after drop");

        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn test_proxy_drop_closes_established_relays() {
        let socket_path = temp_socket_path("relay_drop_srv");
        spawn_echo_server(&socket_path);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let (addr, proxy) = start_uds_proxy(socket_path.to_str().unwrap())
            .await
            .expect("start_uds_proxy");

        let mut tcp = TcpStream::connect(addr).await.expect("connect to proxy");
        tcp.write_all(b"ping").await.expect("write");
        let mut buf = [0u8; 4];
        tcp.read_exact(&mut buf)
            .await
            .expect("relay should echo while proxy is alive");
        assert_eq!(&buf, b"ping");

        drop(proxy);

        let mut rest = Vec::new();
        let read = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tcp.read_to_end(&mut rest),
        )
        .await
        .expect("established relay should be severed by drop, not left alive");
        // EOF (`Ok(0)`) and connection reset (`Err`) both prove the relay died.
        if let Ok(n) = read {
            assert_eq!(n, 0, "expected EOF on the severed relay");
        }

        let _ = std::fs::remove_file(&socket_path);
    }
}

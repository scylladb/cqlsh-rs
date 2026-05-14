//! Unix Domain Socket (UDS) detection and TCP-to-UDS proxy.
//!
//! When cqlsh-rs is given a path to a Unix domain socket instead of a
//! hostname, this module transparently proxies TCP connections (which the
//! Scylla driver always opens) to the UDS endpoint.
//!
//! # Platform support
//! All UDS-specific code is guarded with `#[cfg(unix)]`.  Non-Unix builds
//! compile the stubs that always return `false` / an error.

#[cfg(unix)]
use anyhow::Context as _;

/// Returns `true` if `path` refers to a Unix domain socket on the filesystem.
///
/// Uses `std::fs::metadata` which follows symlinks, so a symlink pointing at
/// a socket will return `true`.  Always returns `false` on non-Unix platforms.
#[cfg(unix)]
pub fn is_unix_socket(path: &str) -> bool {
    use std::os::unix::fs::FileTypeExt;
    std::fs::metadata(path)
        .map(|m| m.file_type().is_socket())
        .unwrap_or(false)
}

/// Non-Unix stub — always returns `false`.
#[cfg(not(unix))]
pub fn is_unix_socket(_path: &str) -> bool {
    false
}

// ── UdsProxy (unix) ────────────────────────────────────────────────────────

/// RAII handle that aborts the background proxy listener task when dropped.
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

// ── UdsProxy (non-unix stub) ───────────────────────────────────────────────

/// Stub type so that `Option<UdsProxy>` compiles on all platforms.
#[cfg(not(unix))]
pub struct UdsProxy;

// ── start_uds_proxy (unix) ─────────────────────────────────────────────────

/// Bind an ephemeral TCP port on `127.0.0.1` and forward every connection to
/// the Unix domain socket at `socket_path`.
///
/// Returns the bound `SocketAddr` (pass it to the driver as the contact point)
/// and a [`UdsProxy`] RAII guard.  Dropping the guard aborts the listener.
///
/// The proxy accepts connections in a loop and spawns a bidirectional copy
/// task per connection using [`tokio::io::copy_bidirectional`], which handles
/// half-close correctly.
#[cfg(unix)]
pub async fn start_uds_proxy(
    socket_path: &str,
) -> anyhow::Result<(std::net::SocketAddr, UdsProxy)> {
    use tokio::io::copy_bidirectional;
    use tokio::net::{TcpListener, UnixStream};

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("binding proxy TCP listener")?;
    let local_addr = listener.local_addr()?;
    let socket_path = socket_path.to_owned();
    let socket_path_for_log = socket_path.clone();

    let join_handle = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut tcp_stream, _peer)) => {
                    let path = socket_path.clone();
                    tokio::spawn(async move {
                        match UnixStream::connect(&path).await {
                            Ok(mut uds_stream) => {
                                if let Err(e) =
                                    copy_bidirectional(&mut tcp_stream, &mut uds_stream).await
                                {
                                    tracing::debug!("UDS proxy connection closed: {e}");
                                }
                            }
                            Err(e) => {
                                tracing::warn!("UDS proxy: failed to connect to {path}: {e}");
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::debug!("UDS proxy listener error: {e}");
                    break;
                }
            }
        }
    });

    let abort_handle = join_handle.abort_handle();
    // Dropping a JoinHandle does NOT abort the task — the task keeps running.
    // We only need the AbortHandle for cleanup on drop.
    drop(join_handle);

    tracing::debug!("UDS proxy started on {local_addr} → UDS {socket_path_for_log}");
    Ok((local_addr, UdsProxy { abort_handle }))
}

/// Address translator that redirects all driver connections to the local proxy.
///
/// The scylla-rust-driver discovers cluster topology and opens connections to
/// each node's `rpc_address`.  When connecting through a UDS proxy, all those
/// addresses must be redirected to the proxy's local TCP port.
#[cfg(unix)]
#[derive(Debug, Clone)]
pub struct ProxyAddressTranslator {
    proxy_addr: std::net::SocketAddr,
}

#[cfg(unix)]
impl ProxyAddressTranslator {
    pub fn new(proxy_addr: std::net::SocketAddr) -> Self {
        Self { proxy_addr }
    }
}

#[cfg(unix)]
#[async_trait::async_trait]
impl scylla::policies::address_translator::AddressTranslator for ProxyAddressTranslator {
    async fn translate_address(
        &self,
        _untranslated_peer: &scylla::policies::address_translator::UntranslatedPeer,
    ) -> Result<std::net::SocketAddr, scylla::errors::TranslationError> {
        Ok(self.proxy_addr)
    }
}

// ── start_uds_proxy (non-unix stub) ───────────────────────────────────────

/// Non-Unix stub — always returns an error.
#[cfg(not(unix))]
pub async fn start_uds_proxy(
    _socket_path: &str,
) -> anyhow::Result<(std::net::SocketAddr, UdsProxy)> {
    anyhow::bail!("Unix domain sockets are not supported on this platform")
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

    /// Spawn a simple UDS echo server using the blocking std API in a thread.
    /// Returns the socket path and a drop-guard that removes it on cleanup.
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
}

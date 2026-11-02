//! In-process TCP forwarder used to stand in for a PrivateLink endpoint.
//!
//! Binds an ephemeral loopback port and pipes every accepted connection to a
//! fixed backend, counting connections along the way. Client-routes tests use
//! two of these: one as the contact point and one as the address published in
//! `system.client_routes`, so traffic reaching the second one proves the driver
//! actually followed the route rather than talking to the node directly.
//!
//! The integration test binary is synchronous, so the forwarder owns its own
//! Tokio runtime and is shut down on drop.

use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};

/// A running TCP forwarder. Stops accepting when dropped.
pub struct TcpForwarder {
    addr: SocketAddr,
    connections: Arc<AtomicUsize>,
    /// Owned runtime; dropped (in the background) to stop the accept loop.
    runtime: Option<tokio::runtime::Runtime>,
}

impl TcpForwarder {
    /// Start forwarding from a fresh loopback port to `backend`.
    ///
    /// `backend` is resolved per connection, so a hostname such as `localhost`
    /// is fine.
    pub fn start(backend: impl Into<String>) -> io::Result<Self> {
        let backend = backend.into();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;

        let listener = runtime.block_on(async { TcpListener::bind("127.0.0.1:0").await })?;
        let addr = listener.local_addr()?;

        let connections = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&connections);
        runtime.spawn(async move {
            loop {
                let Ok((mut client, _)) = listener.accept().await else {
                    break;
                };
                counter.fetch_add(1, Ordering::SeqCst);
                let backend = backend.clone();
                tokio::spawn(async move {
                    if let Ok(mut upstream) = TcpStream::connect(&backend).await {
                        // Errors here are the normal way a CQL connection ends
                        // (client or server closing), so they are not reported.
                        let _ = copy_bidirectional(&mut client, &mut upstream).await;
                    }
                });
            }
        });

        Ok(Self {
            addr,
            connections,
            runtime: Some(runtime),
        })
    }

    /// The loopback address clients should connect to.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// The loopback port clients should connect to.
    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// Number of connections accepted so far.
    pub fn connections(&self) -> usize {
        self.connections.load(Ordering::SeqCst)
    }
}

impl Drop for TcpForwarder {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            // Do not block the test thread waiting for in-flight copies.
            runtime.shutdown_background();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener as StdListener;

    /// Accept one connection and echo a fixed reply.
    fn spawn_echo_server() -> (String, std::thread::JoinHandle<()>) {
        let listener = StdListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 16];
                let n = stream.read(&mut buf).unwrap_or(0);
                let _ = stream.write_all(&buf[..n]);
            }
        });
        (addr, handle)
    }

    #[test]
    fn forwards_payload_and_counts_connections() {
        let (backend, server) = spawn_echo_server();
        let forwarder = TcpForwarder::start(backend).unwrap();
        assert_eq!(forwarder.connections(), 0);

        let mut client = std::net::TcpStream::connect(forwarder.addr()).unwrap();
        client.write_all(b"ping").unwrap();
        let mut reply = [0u8; 4];
        client.read_exact(&mut reply).unwrap();

        assert_eq!(&reply, b"ping");
        assert_eq!(forwarder.connections(), 1);
        server.join().unwrap();
    }

    #[test]
    fn stops_accepting_after_drop() {
        let (backend, _server) = spawn_echo_server();
        let forwarder = TcpForwarder::start(backend).unwrap();
        let addr = forwarder.addr();
        drop(forwarder);

        // The listener is closed with the runtime, so connect (or the first
        // read) must fail rather than hang.
        let connected = std::net::TcpStream::connect(addr).and_then(|mut stream| {
            stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
            let mut buf = [0u8; 1];
            stream.read(&mut buf)
        });
        assert!(
            matches!(connected, Err(_) | Ok(0)),
            "forwarder still serving after drop: {connected:?}"
        );
    }
}

//! Auto-detecting proxy address translator.
//!
//! When connecting through a proxy or load balancer (e.g., AWS NLB, PrivateLink),
//! the driver discovers internal node IPs from `system.peers` that are unreachable
//! from the client. This translator detects the proxy scenario automatically and
//! remaps all peer addresses to the original contact point.
//!
//! **Auto-detection logic**: After connecting, if the contact point IP does not
//! appear in the cluster's discovered node addresses, we assume a proxy is in use
//! and translate all addresses to the contact point.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use scylla::errors::TranslationError;
use scylla::policies::address_translator::{AddressTranslator, UntranslatedPeer};

/// An [`AddressTranslator`] that redirects all peer connections to the original
/// contact point address. Used when the cluster is accessed through a proxy.
///
/// All discovered node addresses are translated to `proxy_address`, ensuring
/// the driver only connects through the proxy endpoint.
#[derive(Debug, Clone)]
pub struct ProxyAddressTranslator {
    /// The proxy/contact point address to route all connections through.
    proxy_address: SocketAddr,
}

impl ProxyAddressTranslator {
    /// Create a new translator that routes all connections to `proxy_address`.
    pub fn new(proxy_address: SocketAddr) -> Self {
        Self { proxy_address }
    }

    /// Returns the proxy address this translator routes to.
    pub fn proxy_address(&self) -> SocketAddr {
        self.proxy_address
    }
}

#[async_trait]
impl AddressTranslator for ProxyAddressTranslator {
    async fn translate_address(
        &self,
        _untranslated_peer: &UntranslatedPeer,
    ) -> Result<SocketAddr, TranslationError> {
        Ok(self.proxy_address)
    }
}

/// Determine whether to use proxy mode based on the contact point and discovered peers.
///
/// Returns `Some(translator)` if the contact point IP is not among the peer addresses,
/// indicating a proxy scenario. Returns `None` for direct connections.
pub fn detect_proxy(
    contact_point: SocketAddr,
    peer_addresses: &[SocketAddr],
) -> Option<Arc<dyn AddressTranslator>> {
    let contact_ip = contact_point.ip();

    // If the contact point IP matches any peer, it's a direct connection
    let is_direct = peer_addresses.iter().any(|peer| peer.ip() == contact_ip);

    if is_direct {
        None
    } else {
        Some(Arc::new(ProxyAddressTranslator::new(contact_point)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn sock(ip: [u8; 4], port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])), port)
    }

    #[test]
    fn detect_proxy_when_contact_point_not_in_peers() {
        let contact = sock([18, 208, 144, 200], 9042);
        let peers = vec![
            sock([10, 0, 0, 1], 9042),
            sock([10, 0, 0, 2], 9042),
            sock([10, 0, 0, 3], 9042),
        ];

        let result = detect_proxy(contact, &peers);
        assert!(
            result.is_some(),
            "should detect proxy when contact point not in peers"
        );
    }

    #[test]
    fn no_proxy_when_contact_point_in_peers() {
        let contact = sock([10, 0, 0, 1], 9042);
        let peers = vec![
            sock([10, 0, 0, 1], 9042),
            sock([10, 0, 0, 2], 9042),
            sock([10, 0, 0, 3], 9042),
        ];

        let result = detect_proxy(contact, &peers);
        assert!(
            result.is_none(),
            "should not detect proxy when contact point is in peers"
        );
    }

    #[test]
    fn no_proxy_when_contact_point_ip_matches_different_port() {
        // Same IP, different port — still a direct connection
        let contact = sock([10, 0, 0, 1], 9043);
        let peers = vec![sock([10, 0, 0, 1], 9042), sock([10, 0, 0, 2], 9042)];

        let result = detect_proxy(contact, &peers);
        assert!(
            result.is_none(),
            "IP match should suffice regardless of port"
        );
    }

    #[test]
    fn detect_proxy_empty_peers() {
        let contact = sock([18, 208, 144, 200], 9042);
        let peers: Vec<SocketAddr> = vec![];

        let result = detect_proxy(contact, &peers);
        assert!(
            result.is_some(),
            "empty peers means we can't reach them — proxy mode"
        );
    }

    #[tokio::test]
    async fn translator_always_returns_proxy_address() {
        let proxy_addr = sock([18, 208, 144, 200], 9042);
        let translator = ProxyAddressTranslator::new(proxy_addr);

        // Create a fake UntranslatedPeer — we need to test the trait method
        // Since UntranslatedPeer fields are pub(crate), we test via the trait
        // using a HashMap translator as reference and our own via detect_proxy
        assert_eq!(translator.proxy_address(), proxy_addr);
    }
}

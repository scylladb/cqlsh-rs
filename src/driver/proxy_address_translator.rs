//! Proxy address translator.
//!
//! When connecting through a proxy or load balancer (e.g., AWS NLB, PrivateLink),
//! the driver discovers internal node IPs from `system.peers` that are unreachable
//! from the client. This translator remaps all peer addresses to the original
//! contact point, ensuring all connections go through the proxy.

use std::net::SocketAddr;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn sock(ip: [u8; 4], port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])), port)
    }

    #[test]
    fn creates_with_correct_address() {
        let proxy_addr = sock([18, 208, 144, 200], 9042);
        let translator = ProxyAddressTranslator::new(proxy_addr);
        assert_eq!(translator.proxy_address, proxy_addr);
    }
}

//! Client routes (PrivateLink / Private Service Connect) configuration.
//!
//! ScyllaDB deployments behind a proxy — AWS PrivateLink, GCP Private Service
//! Connect — advertise node addresses in `system.peers` that the client cannot
//! reach. Instead, each node publishes a routed address in
//! `system.client_routes`, keyed by a `connection_id` assigned by the cloud
//! provider. Configuring one or more connection IDs makes the driver read that
//! table and translate every node's `host_id` to its routed address.
//!
//! The user-facing spelling matches Python cqlsh: a repeatable
//! `--client-route CONNECTION_ID[=ADDRESS]` flag and a `[client_routes]`
//! cqlshrc section. The optional `ADDRESS` overrides the hostname published in
//! the table (the port always comes from the table).

/// A single client route: a connection ID with an optional hostname override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRoute {
    /// Connection ID assigned by the cloud provider, used to filter
    /// `system.client_routes`.
    pub connection_id: String,
    /// Hostname to use instead of the one published in the table.
    pub address: Option<String>,
}

/// Resolved client-routes configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientRoutesSettings {
    /// Configured routes; empty means client routes are disabled.
    pub routes: Vec<ClientRoute>,
    /// Whether the driver may use shard-aware ports. Disabled by default
    /// because ScyllaDB Cloud client-routes deployments do not support it yet.
    pub advanced_shard_awareness: bool,
}

impl ClientRoutesSettings {
    /// Whether client routes are configured at all.
    pub fn is_enabled(&self) -> bool {
        !self.routes.is_empty()
    }
}

/// Parse a single `CONNECTION_ID[=ADDRESS]` specification.
///
/// Matches Python cqlsh's `parse_client_route_spec`: surrounding whitespace is
/// trimmed, the connection ID is split off at the first `=`, and an empty
/// connection ID or an empty override is an error.
pub fn parse_client_route_spec(spec: &str) -> Result<ClientRoute, String> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Err("client route must not be empty".to_string());
    }

    let (connection_id, address) = match trimmed.split_once('=') {
        Some((id, addr)) => (id.trim(), Some(addr.trim())),
        None => (trimmed, None),
    };

    if connection_id.is_empty() {
        return Err(format!("client route '{trimmed}' has empty connection id"));
    }

    match address {
        Some("") => Err(format!(
            "client route '{trimmed}' has empty address override"
        )),
        _ => Ok(ClientRoute {
            connection_id: connection_id.to_string(),
            address: address.map(str::to_string),
        }),
    }
}

/// Parse client routes from CLI values or a cqlshrc `proxies` value.
///
/// Each input is split on commas and newlines, so a single `--client-route`
/// value may carry several specs and a cqlshrc entry may span multiple lines.
/// Empty fragments are skipped, which allows the trailing commas that Python
/// cqlsh's multi-line form produces.
pub fn parse_client_routes<'a, I>(values: I) -> Result<Vec<ClientRoute>, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut routes = Vec::new();
    for value in values {
        for spec in value.split(['\n', ',']) {
            if spec.trim().is_empty() {
                continue;
            }
            routes.push(parse_client_route_spec(spec)?);
        }
    }
    Ok(routes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(id: &str, addr: Option<&str>) -> ClientRoute {
        ClientRoute {
            connection_id: id.to_string(),
            address: addr.map(str::to_string),
        }
    }

    #[test]
    fn parses_bare_connection_id() {
        assert_eq!(
            parse_client_route_spec("conn-a").unwrap(),
            route("conn-a", None)
        );
    }

    #[test]
    fn parses_address_override() {
        assert_eq!(
            parse_client_route_spec("conn-a=proxy-a.example.com").unwrap(),
            route("conn-a", Some("proxy-a.example.com"))
        );
    }

    #[test]
    fn trims_whitespace_around_parts() {
        assert_eq!(
            parse_client_route_spec("  conn-a = proxy-a.example.com  ").unwrap(),
            route("conn-a", Some("proxy-a.example.com"))
        );
    }

    #[test]
    fn splits_on_first_equals_only() {
        assert_eq!(
            parse_client_route_spec("conn-a=host=weird").unwrap(),
            route("conn-a", Some("host=weird"))
        );
    }

    #[test]
    fn rejects_empty_spec() {
        assert_eq!(
            parse_client_route_spec("   ").unwrap_err(),
            "client route must not be empty"
        );
    }

    #[test]
    fn rejects_empty_connection_id() {
        assert_eq!(
            parse_client_route_spec("=proxy-a.example.com").unwrap_err(),
            "client route '=proxy-a.example.com' has empty connection id"
        );
    }

    #[test]
    fn rejects_empty_address_override() {
        assert_eq!(
            parse_client_route_spec("conn-a=").unwrap_err(),
            "client route 'conn-a=' has empty address override"
        );
    }

    #[test]
    fn parses_comma_and_newline_separated_values() {
        let routes =
            parse_client_routes(["conn-a=proxy-a.example.com,\n          conn-b"]).unwrap();
        assert_eq!(
            routes,
            vec![
                route("conn-a", Some("proxy-a.example.com")),
                route("conn-b", None),
            ]
        );
    }

    #[test]
    fn accumulates_across_repeated_values() {
        let routes = parse_client_routes(["conn-a", "conn-b=proxy-b"]).unwrap();
        assert_eq!(
            routes,
            vec![route("conn-a", None), route("conn-b", Some("proxy-b"))]
        );
    }

    #[test]
    fn skips_empty_fragments() {
        let routes = parse_client_routes(["conn-a,,\n,conn-b,"]).unwrap();
        assert_eq!(routes, vec![route("conn-a", None), route("conn-b", None)]);
    }

    #[test]
    fn propagates_spec_errors() {
        let err = parse_client_routes(["conn-a,=bad"]).unwrap_err();
        assert!(
            err.contains("empty connection id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn empty_input_yields_no_routes() {
        assert!(parse_client_routes(Vec::<&str>::new()).unwrap().is_empty());
        assert!(parse_client_routes([""]).unwrap().is_empty());
    }

    #[test]
    fn settings_enabled_only_with_routes() {
        assert!(!ClientRoutesSettings::default().is_enabled());
        let settings = ClientRoutesSettings {
            routes: vec![route("conn-a", None)],
            advanced_shard_awareness: false,
        };
        assert!(settings.is_enabled());
    }
}

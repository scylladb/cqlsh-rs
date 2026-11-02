# [client_routes]

Client routes for ScyllaDB deployments behind a proxy — AWS PrivateLink, GCP
Private Service Connect, or a similar setup where the node addresses advertised
in `system.peers` are not reachable from the client.

```ini
[client_routes]
proxies = conn-a=proxy-a.example.com,
          conn-b
advanced_shard_awareness = false
```

## Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `proxies` | string | | Comma- or newline-separated `CONNECTION_ID[=ADDRESS]` list. `ADDRESS` overrides the hostname published in `system.client_routes`. |
| `advanced_shard_awareness` | boolean | `false` | Allow the driver to use shard-aware ports. ScyllaDB Cloud client-routes deployments do not support this yet. |

## How it works

Each node in a client-routes deployment publishes a reachable address in
`system.client_routes`, keyed by its `host_id` and by a `connection_id` that the
cloud provider assigns to your connection. Given one or more connection IDs,
the driver reads that table and translates every node address it discovers,
following `CLIENT_ROUTES_CHANGE` events as the deployment changes.

Requires ScyllaDB 2026.1 or later on the server side.

## Command line

The `--client-route` flag takes the same `CONNECTION_ID[=ADDRESS]` form and may
be repeated. When given, it replaces the `proxies` value from `cqlshrc` rather
than adding to it.

```bash
cqlsh-rs --client-route conn-a my-endpoint.example.com
cqlsh-rs --client-route conn-a=proxy-a.example.com --client-route conn-b
```

Shard awareness has an explicit flag pair:
`--client-routes-advanced-shard-awareness` and
`--no-client-routes-advanced-shard-awareness`.

## Contact points

If no host is given on the command line, in `CQLSH_HOST`, or in
`[connection] hostname`, the address overrides from the configured routes become
the contact points and the first of them becomes the displayed host:

```bash
# Connects to proxy-a.example.com:9042
cqlsh-rs --client-route conn-a=proxy-a.example.com
```

An explicitly configured host always wins.

## Limitations

These come from the driver and the deployment model, not from cqlsh-rs:

- **TLS is not supported** with client routes. Combining `--client-route` with
  `--ssl` (or with `[client_routes] proxies` in `cqlshrc`) is rejected at
  startup.
- **Mixed clusters are not supported.** Every node must have an entry in
  `system.client_routes`; nodes reachable only directly cannot be contacted.
- Advanced shard awareness is off by default, as noted above.

## Example

```ini
[connection]
hostname = my-endpoint.example.com

[client_routes]
proxies = my-connection-id
```

```bash
cqlsh-rs --debug          # "Using client routes: true" confirms the setup
```

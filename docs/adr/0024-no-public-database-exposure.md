# ADR-0024: No public database exposure — access via Fly proxy only

Date: 2026-05-26
Status: Accepted

## Context

SurrealDB runs on Fly.io alongside the backend. By default, Fly apps can communicate internally via `*.internal` DNS (private network). Exposing the database port publicly would allow anyone on the internet to attempt connections, even with credentials in place.

We need a way for developers to access the database for debugging without permanently exposing it.

## Decision

The database apps (`sitehub-db-staging`, `sitehub-db-production`) are not publicly reachable. This is enforced at two levels:

1. **Private IPs only** — each DB app has a Flycast private IPv6 address (`fly ips allocate-v6 --private`). No public IPv4 or IPv6 IPs are allocated. Even though `[[services]]` is defined in the deploy config, Fly Proxy only routes traffic from the private network when no public IPs exist.
2. **Internal DNS** — the backend connects via `sitehub-db-staging.internal:8000` (Fly's 6PN private network). This DNS name is only resolvable from within the same Fly organization.

The `[[services]]` block is retained in the deploy configs because `flyctl proxy` requires it to route traffic. Without it, the proxy command cannot connect.

For developer access, use Fly's proxy command to open a temporary local tunnel:

```bash
flyctl proxy 8000 --app sitehub-db-staging
```

This makes the DB available at `localhost:8000` for the duration of the session. Closing the terminal closes the tunnel.

## Alternatives considered

- **Public port with IP allowlist** — expose port 8000 publicly but restrict by source IP. Rejected because IP allowlists are fragile (developer IPs change), and Fly.io's free tier has limited networking controls.
- **Public port with credentials only** — rely solely on `SURREAL_USER`/`SURREAL_PASS` for protection. Rejected because credentials can be brute-forced, leaked, or guessed (especially on staging). Defense-in-depth requires network-level isolation.
- **VPN / WireGuard** — Fly.io supports WireGuard tunnels to the private network. More robust than proxy but requires each developer to configure a WireGuard peer. Overkill for a solo developer; revisit when the team grows.
- **Fly Machines SSH** — `flyctl ssh console` to the DB machine. Works for CLI access but doesn't expose a local port for GUI tools like Surrealist. Proxy is more versatile.

## Consequences

- **Positive**: zero attack surface on the database. No public port to scan, brute-force, or exploit. Credentials are a second layer of defense, not the only one.
- **Costs accepted**: developers must run `flyctl proxy` to connect — an extra step compared to a permanently open endpoint. Cannot connect from CI or external monitoring tools without a tunnel.
- **Mitigations**: the proxy command is documented in the README. A `just` recipe could wrap it for convenience. The backend connects internally and is unaffected.

## Revisit when

- The team grows and WireGuard becomes worth the per-developer setup cost.
- We need external monitoring or backup tools to connect to the DB directly.
- Fly.io adds managed database access controls (IP allowlists, IAM-style policies).

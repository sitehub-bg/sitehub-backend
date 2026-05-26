# ADR-0007: Host-based routing for multi-adapter single binary

Date: 2026-05-20
Status: Accepted — partially superseded by ADR-0010 (mobile-api removed, mobile routes merged into public-api as path-based)

## Context

Four driving adapters (`public-api`, `admin-api`, `mobile-api`, `auth-api`)
share a single binary in `sitehub-bin` (see [ADR-0003](0003-eight-package-hexagonal-layout.md)).
We need a request-dispatch scheme that:

1. Routes each request to exactly one adapter.
2. Makes the public API addressable per-tenant (each school's static website
   talks to its own subdomain).
3. Is friendly to CDN / reverse-proxy rules (per-subdomain caching, rate
   limits, ACLs).
4. Makes future splitting into multiple binaries (one per adapter) a
   mechanical operation, not a redesign.

## Decision

Dispatch by `Host` header / subdomain:

| URL pattern | Adapter | Tenant identified by |
|---|---|---|
| `{slug}.sitehub.bg/api/...` | `public-api` | subdomain (`{slug}`) |
| `admin.sitehub.bg/...` | `admin-api` | JWT claim |
| `mobile.sitehub.bg/...` | `mobile-api` | JWT claim |
| `auth.sitehub.bg/...` | `auth-api` | login flow |

The composition root in `sitehub-bin` parses the request's `Host` header,
extracts the subdomain, and dispatches:

- A short fixed set (`admin`, `mobile`, `auth`) routes to the corresponding
  adapter's `router()`.
- Anything else is treated as a tenant slug and routed to `public-api`.
- A request with an unknown or missing host returns 404.

Each driving adapter exposes `pub fn router(state: AppState) -> axum::Router`;
`sitehub-bin` stitches them under a single `axum::serve`.

## Alternatives considered

- **Path-prefix routing** (`/admin/...`, `/api/public/...`, `/mobile/...`,
  `/auth/...` on a single host). Rejected — uglier URLs, harder to apply
  per-adapter network-layer policy (rate limits, CORS, certificates),
  and the public API loses the natural per-tenant subdomain structure.
- **Port-based routing** (different TCP ports per adapter). Rejected —
  needs additional firewall configuration, and tenant subdomains for the
  public API still require host inspection.
- **Separate binaries from day one.** Rejected for now — adds deployment
  complexity (four processes to orchestrate) that buys nothing while the
  user is one developer. Splitting later is mechanical: each adapter's
  `router()` becomes the basis of its own `bin` package.

## Consequences

- **Positive**: production routing mirrors local routing logic. The
  public API gets first-class per-tenant addressing for free (DNS, SSL,
  CDN all work without backend changes). Adapter concerns
  (auth middleware, CORS) stay isolated to their respective routers.
- **Costs accepted**: requires a wildcard TLS certificate
  (`*.sitehub.bg`) and matching DNS. Single point of failure: one binary
  going down takes down all four APIs. Memory footprint of the single
  process is the sum of all adapters.
- **Mitigations**: wildcard TLS via Let's Encrypt is standard.
  Once any adapter outgrows shared deployment, [ADR-0003](0003-eight-package-hexagonal-layout.md)'s
  per-adapter packages let us extract it into its own binary without a
  redesign.

## Revisit when

- One adapter's traffic, deploy cadence, or failure profile diverges
  enough from the others to justify a separate binary.
- We need per-adapter horizontal scaling.

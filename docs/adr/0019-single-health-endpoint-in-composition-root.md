# ADR-0019: Single health endpoint in composition root

Date: 2026-05-26
Status: Accepted

## Context

The initial scaffold had each driving adapter (`sitehub-public-api`, `sitehub-admin-api`, `sitehub-auth-api`) exposing its own `/api/health` endpoint. This caused a route collision when merging routers in `sitehub-bin`, since all three registered the same path.

We need a health endpoint for:
1. Fly.io health checks (configured in `fly.toml` at `/api/health`).
2. Local development verification.
3. Potential future load balancer probes.

## Decision

A single `GET /api/health` handler lives in `sitehub-bin` (the composition root). Individual adapters do not expose health endpoints. The handler returns `{"status": "ok"}` with HTTP 200.

## Alternatives considered

- **Per-adapter health endpoints at different paths** (e.g., `/api/public/health`, `/api/admin/health`) — eliminates the collision. Rejected because all adapters run in the same process — if one is up, all are up. Per-adapter endpoints provide no additional signal and complicate the health check configuration.
- **Health endpoint in a shared utility crate** — adds a cross-cutting dependency between adapters. Rejected per the "no shared utilities package" rule (see CLAUDE.md) and because the handler is 3 lines of code.
- **Per-adapter readiness checks** (e.g., "can storage reach SurrealDB?") — useful but a different concern from liveness. Not needed yet. When added, they would be separate readiness endpoints, not replacements for the liveness check.

## Consequences

- **Positive**: no route collision. Single configuration point for Fly.io. Clear ownership (composition root owns infrastructure concerns).
- **Costs accepted**: if we later need adapter-specific readiness probes (e.g., "is the DB connection pool healthy?"), they'll need their own paths and a way to compose into the root router.
- **Mitigations**: adapter routers already use path prefixes (`/web/v1/...`, `/admin/v1/...`) so future readiness endpoints naturally namespace themselves (e.g., `/admin/v1/ready`).

## Revisit when

- We need to distinguish liveness from readiness (e.g., Kubernetes-style probes).
- An adapter has its own dependency that can fail independently (e.g., a dedicated cache or external API).

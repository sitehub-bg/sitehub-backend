# ADR-0023: SurrealDB on Fly.io colocated with backend

Date: 2026-05-26
Status: Accepted

## Context

The backend deploys to Fly.io in Frankfurt (fra). SurrealDB needs to be accessible from the backend with minimal latency. Each page load involves 3-5 database queries, so per-query latency directly impacts user-facing response times.

We evaluated three hosting options for SurrealDB:

1. Fly.io with persistent volumes (same region as backend)
2. SurrealDB Cloud (managed, free tier available)
3. External VPS (e.g., Hetzner)

## Decision

Run SurrealDB on Fly.io with persistent volumes, colocated in the same region (fra) as the backend. Four apps total:

| App | Purpose | Volume |
|---|---|---|
| `sitehub-backend-staging` | Backend — staging | — |
| `sitehub-backend-production` | Backend — production | — |
| `sitehub-db-staging` | SurrealDB — staging | 1GB |
| `sitehub-db-production` | SurrealDB — production | 3GB |

DB apps run as single instances (`max_machines = 1`) since SurrealDB is not designed for multi-instance load balancing.

## Alternatives considered

- **SurrealDB Cloud** — managed service with a 1GB free tier. Rejected because the cloud instances are hosted on AWS/GCP, adding 5-20ms of network latency per query compared to ~0.5-1ms when colocated on Fly.io. For 3-5 queries per page load, this adds 15-100ms to every response. The free tier is also limited to read-only when storage exceeds 1GB.
- **External VPS (Hetzner)** — cheapest long-term option (~€4/month). Rejected for now because it adds operational overhead (managing the server, backups, TLS) and introduces cross-network latency unless the VPS is in the same datacenter as Fly.io's Frankfurt region.
- **Fly.io Postgres (managed)** — Fly offers managed Postgres but not managed SurrealDB. Not applicable.

## Consequences

- **Positive**: sub-millisecond DB latency. Single platform for all infrastructure. Encrypted volumes with automatic snapshots. Deploy configs version-controlled in `deploy/`.
- **Costs accepted**: Fly.io volumes are tied to a single machine and region — no automatic replication. Volume data survives restarts but is lost if the volume itself is deleted. More expensive than a VPS at scale.
- **Mitigations**: Fly.io takes automatic volume snapshots (retained for 5 days). Production volume is 3GB with room to grow. Migration to SurrealDB Cloud or a VPS is straightforward — change `SURREAL_URL` and redeploy.

## Revisit when

- Volume storage costs exceed SurrealDB Cloud pricing at equivalent usage.
- We need database replication or multi-region failover.
- Fly.io volume reliability becomes a concern (data loss incident).
- SurrealDB Cloud adds a Frankfurt region with competitive latency.

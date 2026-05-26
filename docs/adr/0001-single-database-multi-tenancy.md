# ADR-0001: Single-database multi-tenancy with `tenant_id` scoping

Date: 2026-05-20
Status: Accepted

## Context

sitehub.bg is a multi-tenant platform projected to host ~5000 schools (phase 1)
and other organization types later. Per-tenant data volume is expected to be
modest (likely <1000 records per school, dominated by pages, news, staff, and
schedules). We need to choose how tenants are isolated in SurrealDB.

The decision affects: query patterns, schema migration strategy, backups,
connection pooling, blast radius of bugs, and operational complexity.

## Decision

Use a **single SurrealDB namespace and database**. Every record carries a
`tenant_id` field. The application enforces scoping at two layers:

1. **Repository ports** in `sitehub-app` require a `TenantCtx` parameter on
   every method. A use case cannot compile without supplying tenant context.
2. The storage adapter in `sitehub-storage` injects `WHERE tenant = $tenant`
   into every query.

Defense-in-depth via SurrealDB `DEFINE ACCESS` rules is planned as a follow-up
so the database itself rejects unscoped queries.

## Alternatives considered

- **Database per tenant.** Cleanest isolation, simpler queries, smaller per-DB
  working sets. Rejected because at 5000 tenants every schema change must apply
  to 5000 databases — requiring migration orchestration, drift detection,
  partial-failure retry, and 5000-way backups. Cross-tenant analytics
  (e.g., "which schools use feature X?") also become 5000× harder.
- **Namespace per tenant.** Same operational burden as DB-per-tenant.
- **Connection per tenant.** Stateful SurrealDB connections per tenant scale
  poorly when most tenants are idle.

## Consequences

- **Positive**: one migration path, one backup, one connection pool. Easy
  cross-tenant analytics and feature rollouts. Trivial new-tenant onboarding
  (insert rows, no schema work). Migration to DB-per-tenant remains possible
  later by exporting per-tenant subsets.
- **Costs accepted**: a missing `WHERE tenant_id = ?` would leak data across
  tenants. Noisy-neighbor risk: one tenant's heavy load can affect others.
  Restore granularity is whole-DB, not per-tenant.
- **Mitigations**: repository signatures make forgetting the scope a compile
  error. Cross-tenant isolation tests in the integration suite. DB-level
  `DEFINE ACCESS` planned as a second layer of enforcement.

## Revisit when

- A single tenant exceeds ~1M records or generates load that materially affects
  others.
- We onboard a customer with regulatory requirements demanding physical
  isolation.
- Cross-tenant query patterns dry up (i.e., we stop benefiting from a shared DB).

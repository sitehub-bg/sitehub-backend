# ADR-0004: SurrealDB as the data store

Date: 2026-05-20
Status: Accepted

## Context

sitehub-backend stores tenant content (pages, news, staff, schedules), users,
memberships, and audit records. We needed to pick a primary data store. The
constraints:

1. Solid Rust async client.
2. Indexed lookups under millisecond latency for typical web API loads.
3. Tenant-scoped queries are the dominant access pattern.
4. The data model is part document (page body, structured content) and part
   relational (user → membership → school).
5. Single-instance operability for a side project.

## Decision

Use **SurrealDB** as the primary data store.

- Single namespace, single database (see [ADR-0001](0001-single-database-multi-tenancy.md)).
- Schema-full table definitions enforced via `DEFINE TABLE` / `DEFINE FIELD`.
- Indexes on `tenant_id` plus query-key combinations.
- SurrealDB `DEFINE ACCESS` rules planned for DB-level tenant isolation.

## Alternatives considered

- **PostgreSQL + sqlx.** The conventional choice. Rejected for this project
  primarily because the content model has graph/document characteristics
  (pages embedding structured content, user-school-role relations) that
  SurrealQL expresses more directly. Postgres remains the safer choice for a
  production-critical system; SurrealDB suits this project's scope and serves
  as a distinguishing technical talking point.
- **MongoDB.** Schemaless makes tenant isolation harder to enforce; mature
  async Rust client but ecosystem fit is weaker for our access patterns.
- **MySQL / MariaDB.** No advantage over Postgres for our case.
- **Embedded options (SQLite, sled).** Rejected for multi-tenant write
  workloads.

## Consequences

- **Positive**: first-party Rust SDK (SurrealDB is itself written in Rust).
  Native graph + document + relational in one engine. `DEFINE ACCESS`
  provides record-level security enforced at the database, which is a
  strong defense-in-depth story for multi-tenancy. Live queries available
  if we want server-push later.
- **Costs accepted**: smaller ecosystem than Postgres — fewer tutorials,
  fewer ORMs/migration tools, fewer answered Stack Overflow questions.
  SurrealQL is a learning curve. Operational tooling (backup, monitoring,
  HA) is less mature than the Postgres equivalent.
- **Mitigations**: keep the SurrealDB-specific code confined to
  `sitehub-storage`. `sitehub-app` knows only about port traits — swapping
  to Postgres later is mechanical from the domain's perspective.

## Revisit when

- We need features SurrealDB lacks or hits a maturity wall (e.g.,
  point-in-time recovery, mature HA setup).
- A single-tenant load profile exposes performance problems that don't
  reproduce on Postgres.

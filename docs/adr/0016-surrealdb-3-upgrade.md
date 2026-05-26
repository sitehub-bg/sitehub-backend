# ADR-0016: SurrealDB 3.x as data store version

Date: 2026-05-26
Status: Accepted

## Context

ADR-0004 chose SurrealDB as the data store. At project start (May 2026), two major versions are available:

- **v2.6.5** — stable, well-documented, large dependency tree (includes WASM bindings, GraphQL, many backends).
- **v3.0.5** — stable release of the 3.x line. Slimmer dependency tree, `surrealkv` replaces the deprecated `file:` storage backend, updated SDK API.

Since we have zero production data and no existing schema, there's no migration cost. Starting on v3 avoids a future breaking upgrade.

## Decision

Use `surrealdb = "3.1"` (currently resolves to 3.1.0). Docker images pinned to `surrealdb/surrealdb:v3.1.0`. Local storage backend is `surrealkv:` (replaces the v2 `file:` backend). Dependabot bumps the Docker image tag when new patches are released.

## Alternatives considered

- **Stay on v2.6.5** — maximum stability and documentation coverage. Rejected because starting a greenfield project on a version that will eventually require a breaking migration is technical debt from day one.
- **Use v3.1.0-beta** — latest features. Rejected because beta versions may have breaking changes between releases and limited documentation.

## Consequences

- **Positive**: we start on the latest stable major version with no migration debt. Smaller dependency tree means faster compiles and smaller binaries. `surrealkv` is the recommended storage backend going forward.
- **Costs accepted**: v3 documentation may lag behind v2 in some areas. Community examples often target v2 and may need adaptation.
- **Mitigations**: the SurrealDB Rust SDK API is similar between v2 and v3. We have no stored data yet, so any breaking discovery is cheap to fix.

## Revisit when

- SurrealDB 4.x is released and offers compelling features.
- We discover a v3-specific bug that blocks a feature and has no workaround.

# ADR-0027: Staging mirrors production configuration

Date: 2026-05-26
Status: Accepted

## Context

Configuration differences between staging and production are a common source of "works on staging, fails on production" bugs. Every divergence is a blind spot — a feature that passes staging validation but breaks in production because the environment behaves differently.

SurrealDB's `SURREAL_CAPS_ALLOW_ALL` flag was initially enabled on staging for development convenience but disabled on production for security. This created exactly such a blind spot: queries using capabilities that staging allowed would silently fail in production.

## Decision

Staging and production configurations must be identical except for:

1. **Scale** — `min_machines_running` (0 for staging, 1 for production)
2. **Volume size** — (1GB staging, 3GB production)
3. **Credentials** — different passwords per environment (via Fly secrets)

Everything else — SurrealDB flags, capability settings, Fly.io config structure, environment variables — must match between `deploy/*-staging.toml` and `deploy/*-production.toml`.

When adding a new config flag, add it to both environments simultaneously.

## Alternatives considered

- **Relaxed staging** — enable convenience flags (all capabilities, debug modes, permissive CORS) on staging to speed up development. Rejected because every relaxed setting is a bug that staging can't catch. The cost of debugging a capability error locally is minutes; the cost of discovering it in production is an incident.
- **Three environments (dev/staging/production)** — add a separate "dev" environment with relaxed settings, keep staging strict. Rejected as over-engineering for a solo project. Local development (`docker-compose.yml`) already serves as the relaxed environment.

## Consequences

- **Positive**: staging is a reliable preview of production. A deploy that succeeds on staging will succeed on production (modulo scale). No "works on staging" surprises.
- **Costs accepted**: developers may hit capability or configuration errors earlier (on staging instead of only in production). This is the desired behavior — fail early.
- **Mitigations**: local development via `docker-compose.yml` remains unrestricted for experimentation. Staging catches config issues before production.

## Revisit when

- A staging-specific feature is needed (e.g., debug endpoints, test data seeding) that cannot exist in production.
- The team grows and needs a separate "dev" environment for experimentation.

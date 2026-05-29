# ADR-0027: Staging mirrors production, with stricter performance bounds

Date: 2026-05-26
Last updated: 2026-05-29
Status: Accepted

## Context

Configuration differences between staging and production are a common source of "works on staging, fails on production" bugs. Every divergence is a blind spot — a feature that passes staging validation but breaks in production because the environment behaves differently.

SurrealDB's `SURREAL_CAPS_ALLOW_ALL` flag was initially enabled on staging for development convenience but disabled on production for security. This created exactly such a blind spot: queries using capabilities that staging allowed would silently fail in production.

A second concern emerged later: when staging matches production exactly on performance bounds (timeouts, resource limits), slow code paths produced by careless changes go unnoticed on staging because they still fit within the production budget. Discovering them only in production — under real traffic — turns ordinary regressions into incidents.

## Decision

Staging shares production's **structure** (every flag, capability, env var, service shape) but applies **stricter performance bounds** so developers notice slow paths before they reach real users.

### Identical between staging and production

- All SurrealDB capability flags
- Fly.io config structure: same `[http_service]`, `[checks]`, `[env]` shape and keys
- Set of env vars (key names match; values may differ where noted below)
- Health check paths
- The same Docker image promoted from staging to production (per ADR-0030)

### Allowed to differ

| Dimension | Staging | Production | Why |
|---|---|---|---|
| `min_machines_running` | 0 | 0–1 | Cost: staging idles when unused. Production scales to keep cold-starts off the user path once we have traffic. |
| Volume size | smaller | larger | Cost-shaped, not behavior-shaped. |
| Credentials | distinct per env | distinct per env | Compromise isolation. |
| `request_timeout_secs` | **stricter (smaller)** | larger | Force slow endpoints to fail in staging before they meet a real user. |
| `shutdown_grace_secs` | **stricter (smaller)** | larger | Surface laggy graceful-shutdown paths before they slow production rollouts. |
| Other performance/resource bounds | **stricter where possible** | normal | Same principle: if a code path can violate the production budget, the staging budget should catch it first. |

The rule is directional: staging's performance bounds are **tighter or equal** to production's, never looser. A change that runs cleanly in production must also run cleanly in staging — but a change that *only* runs cleanly in production is a bug waiting to ship.

When adding a new flag, add it to both environments simultaneously. When adding a new performance bound, pick the production value first and then choose a staging value that is at most that strict, ideally tighter.

## Alternatives considered

- **Strictly identical staging and production** (the original 2026-05-26 form of this ADR) — has the predictability benefit but lets performance regressions ride to production unchallenged. The cost of an incident outweighs the cost of seeing the regression in staging first.
- **Relaxed staging** (faster timeouts, all capabilities, debug modes) — every relaxed setting is a class of bugs staging can't catch. Rejected.
- **Three environments (dev/staging/production)** — local `docker-compose.yml` already plays the "relaxed for experimentation" role. Adding a fourth environment for one team is over-engineering.

## Consequences

- **Positive**: regressions in performance show up in staging while they are still cheap to fix. Configuration drift (flags, capabilities) is still impossible because the structural surface is identical.
- **Costs accepted**: developers may see staging request-timeouts trigger on slow code paths that would still be within production's budget. This is the desired feedback signal, not a problem to suppress.
- **Mitigations**: documented numeric values (timeouts, etc.) sit in committed config files (`config/staging.toml`, `config/production.toml`) so the divergence is reviewable, not implicit. Local development via the dev config remains unrestricted for experimentation.

## Revisit when

- A staging-specific feature is needed (e.g., debug endpoints, test data seeding) that cannot exist in production.
- The team grows and needs a separate "dev" environment for experimentation.
- We adopt load-testing infrastructure that catches perf regressions earlier than staging traffic — at which point stricter staging bounds may become redundant.

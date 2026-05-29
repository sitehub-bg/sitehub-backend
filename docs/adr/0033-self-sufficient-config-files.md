# ADR-0033: Config files are self-sufficient

Date: 2026-05-29
Status: Accepted

## Context

ADR-0032 introduced layered configuration via `figment`, with the original layering:

1. Struct defaults
2. `config/base.toml` (shared base)
3. `config/<env>.toml` (env-specific overlay)
4. `SITEHUB_` env vars

A shared `base.toml` file means the effective config for any environment requires mentally merging two files. When debugging "what's the actual timeout in staging?", you have to read `base.toml`, then `staging.toml`, and reason about which values win. This makes the deployed configuration harder to reason about and easier to get wrong (e.g., forgetting that a field exists in `base.toml` and accidentally not overriding it).

## Decision

Each environment-specific config file (`staging.toml`, `production.toml`, etc.) must contain **all** configuration values for that environment. There is no `base.toml`.

Layering order is now:

1. Struct defaults (compiled-in fallbacks, used when no config file is loaded — primarily for local dev).
2. `config/<env>.toml` (path from `SITEHUB_CONFIG` env var) — the complete configuration for that environment.
3. `SITEHUB_` env vars — runtime overrides (primarily for secrets and per-deploy ad-hoc tuning).

Each TOML file should be readable top-to-bottom as the complete deployed configuration. No "merge math" required.

## Alternatives considered

- **Shared `base.toml` + per-env overrides** (original ADR-0032 design) — DRY but obscures the effective config.
- **Single file with sections per environment** — works for small configs but doesn't compose well as the number of environments grows.
- **No struct defaults; every value must come from a file** — strictly self-sufficient but breaks local dev where no file is set. Rejected because struct defaults give a useful "it just works" experience for `cargo run`.

## Consequences

- **Positive**:
  - One file = one environment's full config. Easy to audit, easy to diff.
  - No silent inheritance of values across environments.
  - Reduces cognitive load when troubleshooting production behavior.
- **Costs accepted**:
  - Some duplication across env files (e.g., both staging and production likely have `port = 3000`).
  - Adding a new config key requires touching every env file, not just one base file.
- **Mitigations**:
  - Struct defaults provide a safety net: if a key is missing from a file, the compiled-in default applies. Adding a new key with a sensible default doesn't break existing deploys.
  - PR review can enforce that env files stay in sync when new keys are added.

## Revisit when

- The duplication across env files becomes a real source of bugs (e.g., updating one env and forgetting another led to an incident).
- We have so many environments that maintaining N copies of the same value is materially painful.

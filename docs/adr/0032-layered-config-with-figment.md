# ADR-0032: Layered config with figment (exception to 12-factor factor 3)

Date: 2026-05-29
Status: Accepted

## Context

ADR-0030 commits the project to 12-factor principles. Factor 3 — "store config in the environment" — pushes back hard against configuration files. As the application grows (timeouts, host/port, DB URL, JWT settings, feature flags, logging levels, tenant defaults, etc.), the number of env vars to manage per deploy becomes unwieldy and error-prone:

- Flat env vars don't express nested structure (e.g., `[surreal] url = ... namespace = ...`).
- Adding a new config key requires touching every Fly.io TOML file individually.
- No single source of truth for "what config does this service consume?"
- Defaults end up scattered across code or duplicated across env files.

We want a config system that is **structured**, **typed**, **environment-aware**, and still **12-factor-compliant for the values that matter** (secrets, per-deploy overrides).

## Decision

Use [`figment`](https://docs.rs/figment) for layered configuration. Sources, in priority order (later layers override earlier):

1. **Struct defaults** — sane fallbacks defined alongside the type.
2. **Base TOML** — `config/base.toml`, committed, shared across all environments.
3. **Environment-specific TOML** — `config/staging.toml`, `config/production.toml`. Path selected by `SITEHUB_CONFIG` env var.
4. **Env vars with `SITEHUB_` prefix** — final override, used for secrets and per-deploy tuning.

Configuration is deserialized into a typed `Config` struct at startup. Failures abort the process before serving traffic.

This is an **explicit exception** to ADR-0030 factor 3, justified by:

- Number of config values is expected to grow well past the threshold where flat env vars stay readable.
- Nested structure (e.g., per-API timeouts, per-tenant defaults) maps poorly to flat env keys.
- Layered overrides preserve the 12-factor benefit (env-var-overridable) without forcing every value into the env namespace.

## Alternatives considered

- **Flat env vars only** — pure 12-factor but doesn't scale to >20 config keys; nested structures become awkward (`SITEHUB_SURREAL_URL`, `SITEHUB_SURREAL_NS`, `SITEHUB_SURREAL_DB`, …).
- **`config-rs`** — solid, more format support (INI, RON, JSON5), but clunkier API and less helpful error messages. We don't need extra formats; TOML is enough.
- **Hand-rolled `serde` + TOML** — viable but reinvents the layering and env-override logic that figment already provides.
- **Hardcoded const in `main.rs`** — what we had. Doesn't differentiate environments without recompiling, breaking ADR-0030 factor 5 (build/release/run separation).

## Consequences

- **Positive**:
  - Single typed config struct documents every value the service consumes.
  - Per-environment overrides via small TOML files, not a sea of env vars.
  - Secrets and per-deploy overrides still flow through env vars, preserving 12-factor where it matters most.
  - Startup-time validation: malformed config fails fast.
- **Costs accepted**:
  - Adds a dependency (`figment`).
  - Two extra files per environment (`config/staging.toml`, `config/production.toml`) to keep in sync.
  - Deviation from 12-factor purity — documented here as a deliberate trade-off.
- **Mitigations**:
  - `config/*.toml` files committed to git; no secrets in them — secrets always come from env vars at the top layer.
  - Required env vars (e.g., `SURREAL_PASS`) fail loudly at startup if missing.
  - Treat `config/base.toml` as authoritative for "what keys exist"; env files only override.

## Revisit when

- Config grows large enough to need a remote config service (Consul, etcd, AWS AppConfig).
- We move to a runtime that natively supports typed config (e.g., NixOS modules).
- Number of config keys drops back to <10, at which point flat env vars would be simpler.

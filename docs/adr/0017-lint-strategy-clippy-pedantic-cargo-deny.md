# ADR-0017: Lint strategy — clippy pedantic + cargo-deny + rustfmt

Date: 2026-05-26
Status: Accepted

## Context

We need consistent code quality enforcement across the workspace. Rust's ecosystem provides official tooling (clippy, rustfmt) and community tools (cargo-deny) but they require deliberate configuration to be useful beyond defaults.

The project is also a portfolio piece — demonstrating strict but pragmatic lint configuration signals engineering maturity.

## Decision

Three layers of enforcement, all running in CI with `-D warnings` (warnings are errors):

**1. Clippy pedantic (code quality)**

Configured via `[workspace.lints.clippy]` in root `Cargo.toml`, inherited by all packages via `[lints] workspace = true`. Settings:

- `pedantic = warn` (priority -1, so individual overrides work)
- `unsafe_code = forbid` (workspace-wide, no exceptions)
- Allowed pedantic lints: `missing_errors_doc`, `missing_panics_doc`, `module_name_repetitions`, `must_use_candidate` — too noisy for the value they provide.

**2. rustfmt (formatting)**

Configured via `rustfmt.toml`:
- `edition = "2024"` — pinned to prevent drift if rustfmt defaults change.
- `max_width = 100` — community standard.
- `use_field_init_shorthand = true` and `use_try_shorthand = true` — idiomatic Rust.
- `imports_granularity = "Module"` and `group_imports = "StdExternalCrate"` — requires nightly rustfmt.

**3. cargo-deny (dependency hygiene)**

Configured via `deny.toml`:
- License allowlist (permissive licenses + BSL-1.1 exception for SurrealDB and our crates).
- Security advisory checking (RUSTSEC database).
- Duplicate crate detection (warn level).
- Banned crates (`dotenv`, `openssl`, `openssl-sys`).
- Unknown registry/git source warnings.

## Alternatives considered

- **Default clippy only (no pedantic)** — catches bugs but not stylistic issues. Rejected because pedantic lints enforce idiomatic patterns that improve readability.
- **clippy::restriction group** — extremely opinionated, requires individual lint selection. Rejected as too high-maintenance for the benefit.
- **No cargo-deny** — dependencies unchecked for licenses or vulnerabilities. Rejected because a single transitive dep with a copyleft license could create legal issues.

## Consequences

- **Positive**: consistent code style enforced at compile time. Known vulnerabilities caught before merge. License compliance automated.
- **Costs accepted**: pedantic lints occasionally require `#[allow(...)]` annotations. cargo-deny config needs updating when new transitive dependencies bring new licenses.
- **Mitigations**: allowed lints are documented in Cargo.toml. CI runs deny checks in parallel with other jobs.

## Revisit when

- Pedantic lints become more noisy than valuable.
- A new Rust edition changes default lint behavior.
- We need stricter security scanning.

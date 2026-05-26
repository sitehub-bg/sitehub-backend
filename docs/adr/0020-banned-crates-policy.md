# ADR-0020: Banned crates policy via cargo-deny

Date: 2026-05-26
Status: Accepted

## Context

Rust's ecosystem has cases where multiple crates solve the same problem, with one being unmaintained, insecure, or carrying undesirable properties (C dependencies, platform-specific behavior). Without an explicit deny list, a transitive dependency can silently introduce an unwanted crate into the tree.

Two specific cases motivated this decision:

1. **`dotenv` vs `dotenvy`** — `dotenv` is unmaintained since 2020. `dotenvy` is its maintained fork with the same API. A transitive dep pulling in `dotenv` would add dead code and a potential security liability.
2. **`openssl`/`openssl-sys` vs `rustls`** — OpenSSL requires a system C library, introduces version drift across environments, and has a long history of CVEs. `rustls` is a pure-Rust TLS implementation with no C dependencies, used by SurrealDB's default features.

## Decision

We maintain a `[bans.deny]` list in `deny.toml` that explicitly forbids crates that must never appear in the dependency tree:

```toml
deny = [
    { crate = "dotenv", reason = "Use dotenvy (maintained fork) instead" },
    { crate = "openssl", reason = "Use rustls — no C dependency, no system OpenSSL version drift" },
    { crate = "openssl-sys", reason = "Use rustls — no C dependency" },
]
```

`cargo deny check bans` runs in CI and will fail the build if any of these crates appear, even transitively.

## Alternatives considered

- **No deny list, rely on code review** — a transitive dep from a legitimate crate (e.g., `reqwest` with default features) could silently pull in `openssl-sys`. Human review is unreliable for transitive deps. Rejected.
- **Use `[patch]` in Cargo.toml to redirect** — only works for direct deps, not transitive. Also modifies resolution in ways that are hard to reason about. Rejected.
- **Feature flags only (e.g., `default-features = false`)** — helps for direct deps but doesn't protect against transitive pulls. Complements the deny list but doesn't replace it.

## Consequences

- **Positive**: build fails immediately if a banned crate enters the tree. Forces explicit decisions when adding deps that might pull in OpenSSL. Documents which crates are unwanted and why.
- **Costs accepted**: if a legitimate dependency requires `openssl-sys` (e.g., a future crate with no rustls support), we must either find an alternative, contribute a rustls feature upstream, or explicitly override the ban with documentation.
- **Mitigations**: the deny list entries include `reason` fields so the CI failure message explains what to do. Adding an exception is a one-line change with a PR that documents why.

## Revisit when

- A critical dependency has no rustls support and the OpenSSL ban becomes blocking.
- `dotenv` is revived or merged into `dotenvy`.
- New categories of unwanted crates emerge (e.g., if we decide to ban `tokio-compat` or similar shims).

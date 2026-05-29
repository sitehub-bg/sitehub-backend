# ADR-0031: Unix-only target

Date: 2026-05-29
Status: Accepted

## Context

`sitehub-backend` runs on Fly.io (Firecracker VMs, Linux only) and is developed primarily on Linux workstations. Contributors may also use macOS. Windows is not used by anyone on the project and has no path to production.

Cross-platform Rust code that supports Windows adds `#[cfg(windows)]` / `#[cfg(not(unix))]` branches that exist only to satisfy a compiler check that will never run in CI or production.

The 12-factor app principle of dev/prod parity (ADR-0030, factor 10) is best served by acknowledging the platforms we actually target rather than pretending to be portable.

## Decision

`sitehub-backend` targets Unix only:

- **Production**: Linux (Fly.io Firecracker VMs).
- **Development**: Linux or macOS native; both can use `tokio::signal::unix`, file paths, and other Unix APIs without `#[cfg]` gating.
- **Windows**: not supported. Windows contributors must use WSL or Docker.

## Alternatives considered

- **Maintain cross-platform code paths including Windows** — adds `#[cfg(windows)]` boilerplate that nobody benefits from. Increases surface area for bugs never caught in CI (which runs only on Linux).
- **Linux only** — slightly simpler but excludes macOS contributors from native development. Most Rust developers use macOS, so this raises onboarding friction without operational benefit.

## Consequences

- **Positive**:
  - Simpler code: signal handling, file paths, and process management can use Unix APIs directly.
  - macOS contributors can `cargo run` natively without Docker.
- **Costs accepted**:
  - Contributors on Windows can only build via WSL or Docker.
  - If we adopt Linux-specific APIs (e.g., `epoll` directly, `io_uring`), macOS support becomes a code-level concern again.
- **Mitigations**:
  - Docker-based local development is documented in the README for Windows contributors.
  - If a Linux-specific API becomes necessary, gate it with `#[cfg(target_os = "linux")]` and provide a macOS fallback only if the affected code path is dev-relevant.

## Revisit when

- The hosting platform changes to something that isn't Linux (extremely unlikely).
- We adopt a Linux-only kernel API that doesn't work on macOS and the affected code matters for local dev.

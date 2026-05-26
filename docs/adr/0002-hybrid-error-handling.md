# ADR-0002: Hybrid error handling (`AppError` + `anyhow`)

Date: 2026-05-20
Status: Accepted

## Context

The workspace has eight packages, several of which produce errors at different
layers (storage I/O, domain validation, HTTP request parsing, auth verification).
We need an error strategy that:

1. Carries enough information to debug failures in production.
2. Lets the HTTP layer map errors to status codes by pattern matching.
3. Does not require deep `From` chains that lose context.

A prior attempt by the author used per-layer typed errors (one enum per crate,
with `#[from]` propagating between them). The result was deep nested error
chains where the original cause and the operation being attempted were both
hidden under several layers of "internal error: …", and debugging required
unwrapping multiple variants by hand.

## Decision

Use a **two-tool hybrid**:

1. **One `AppError` enum** in `sitehub-app/src/error.rs`, defined with
   `thiserror`. Roughly 5–7 semantically meaningful variants
   (`NotFound`, `Conflict`, `Validation`, `Unauthorized`, `Forbidden`, …)
   plus `Internal(#[from] anyhow::Error)` as the catch-all.
2. Adapters use **`anyhow::Context`** internally. Every meaningful step
   attaches a context string (`.with_context(|| ...)`). The `?` operator
   converts any `anyhow::Error` into `AppError::Internal`, preserving the
   full context chain.
3. **One mapping point per HTTP adapter** — an `IntoResponse for AppError`
   implementation maps semantic variants to status codes; `Internal` logs
   the full chain server-side via `tracing::error!` and returns a generic
   message to clients.

`AppResult<T>` = `Result<T, AppError>` is the conventional return type
across `sitehub-app`.

## Alternatives considered

- **Per-layer typed errors with `#[from]` chains.** The pattern the author
  tried previously. Rejected — context was lost, chains became opaque,
  pattern matching at the HTTP edge required cascading match arms.
- **Single `anyhow::Error` everywhere.** Rejected — the HTTP layer needs to
  match on error kinds to choose status codes, and pure `anyhow` errors
  don't carry the type information cleanly.
- **Shared `sitehub-errors` package.** Rejected — would become a god-crate
  every package depends on, and would couple the domain to error
  representations of unrelated layers.

## Consequences

- **Positive**: error chains stay flat and readable. New "internal" failures
  cost nothing (just `?`). New semantic failures require one enum variant.
  Pattern matching at the HTTP edge is concentrated in one place per adapter.
- **Costs accepted**: `sitehub-app` depends on `anyhow`, slightly polluting
  the "pure" hexagonal core. `anyhow` is a generic carrying utility (not
  a framework or I/O library), so the leak is conceptual rather than
  practical.
- **Mitigations**: `AppError::Internal` is opaque to consumers — clients see
  a generic message; the chain is only visible in server logs. If purist
  isolation becomes important, replace the `anyhow::Error` field with
  `Box<dyn std::error::Error + Send + Sync>`.

## Revisit when

- We need machine-readable error codes shared with external clients
  (e.g., a stable gRPC error schema across services).
- The `Internal` variant accumulates so much that we want subcategories.

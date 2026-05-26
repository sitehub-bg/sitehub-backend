# ADR-0009: Rename sitehub-jwt to sitehub-tokens

Date: 2026-05-25
Status: Accepted

## Context

The driven adapter for token issuance and verification was named `sitehub-jwt`, after the implementation technology (JSON Web Tokens). The other driven adapter, `sitehub-storage`, is named after the concern (data storage), not the technology (SurrealDB). This inconsistency caused confusion with `sitehub-auth-api` — both contained "auth" or "jwt" in casual conversation, making it unclear which crate was being discussed.

## Decision

Rename `sitehub-jwt` to `sitehub-tokens`. The package is named after the concern (token management), not the technology (JWT). This matches the pattern established by `sitehub-storage`.

## Alternatives considered

- **Keep `sitehub-jwt`** — workable but inconsistent with `sitehub-storage` naming pattern. Breaks the convention.
- **`sitehub-auth`** — too close to `sitehub-auth-api`. Ambiguous in conversation and import paths.
- **`sitehub-auth-core`** — implies this crate owns auth logic. It doesn't — it's just the crypto/encoding adapter. Domain auth logic lives in `sitehub-app` use cases.
- **`sitehub-identity`** — identity is a domain concept (users, memberships). This crate doesn't own identity; it serializes tokens.

## Consequences

- **Positive**: consistent concern-based naming across all driven adapters. Unambiguous in conversation.
- **Costs accepted**: minor rename effort across Cargo.toml files and imports.
- **Mitigations**: none needed — blast radius is small (directory, Cargo.toml references, one lib.rs doc comment).

## Revisit when

If we add a second token technology (e.g., Paseto, opaque tokens), the name `sitehub-tokens` still works. No revisit anticipated.

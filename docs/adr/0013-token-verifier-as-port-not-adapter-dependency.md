# ADR-0013: Token verification via port trait, not adapter dependency

Date: 2026-05-25
Status: Accepted

## Context

The authenticated driving adapters (`sitehub-admin-api`, `sitehub-auth-api`) need to verify JWT tokens in middleware. The token verification logic lives in `sitehub-tokens` (driven adapter). However, the hexagonal architecture rule says "no adapter depends on another adapter." If admin-api depends on sitehub-tokens directly, this rule is violated.

## Decision

Define a `TokenVerifier` trait (port) in `sitehub-app`. The concrete JWT implementation lives in `sitehub-tokens`. The composition root (`sitehub-bin`) injects the concrete verifier into the application state (`AppState`). Driving adapters access it as `Arc<dyn TokenVerifier>` from the shared state.

```
sitehub-app:     defines TokenVerifier trait
sitehub-tokens:  implements TokenVerifier for JwtVerifier
sitehub-bin:     creates JwtVerifier, wraps in Arc, puts in AppState
admin-api:       extracts Arc<dyn TokenVerifier> from AppState
```

## Alternatives considered

- **admin-api depends on sitehub-tokens directly** — simpler wiring but violates the "no adapter depends on another adapter" rule. Creates a coupling path between driving and driven adapters.
- **Inline JWT verification in each driving adapter** — duplicates crypto logic across packages. Defeats the purpose of having a dedicated token adapter.
- **Verification middleware in sitehub-bin** — possible but pushes HTTP concerns (extracting headers, returning 401) into the composition root, which should only wire things together.

## Consequences

- **Positive**: hexagonal rules maintained. Token technology can be swapped by changing only `sitehub-tokens` and `sitehub-bin`. Driving adapters are unaware of JWT.
- **Costs accepted**: slightly more indirection. AppState must carry the verifier.
- **Mitigations**: the pattern is standard in hexagonal Rust projects. One trait, one impl, one injection point.

## Revisit when

If native async fn in traits (AFIT) becomes fully dyn-compatible on stable Rust, the `#[async_trait]` annotation on `TokenVerifier` can be removed.

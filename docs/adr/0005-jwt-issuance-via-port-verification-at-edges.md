# ADR-0005: JWT issuance via port, verification at adapter edges

Date: 2026-05-20
Status: Accepted

## Context

Three driving adapters (`sitehub-admin-api`, `sitehub-mobile-api`, and
authenticated routes within `sitehub-public-api` if any are added) need to
authenticate callers. `sitehub-auth-api` issues tokens after a login flow.

Two operations span the system:

- **Issuance** — happens during login. It involves domain logic
  (credential verification, audit logging, possibly rate limiting per actor)
  and produces a signed token. This is a business-level operation.
- **Verification** — happens on every authenticated request. It is pure
  cryptography (signature check + expiry + issuer claim). It does not
  consult the domain or the database.

Treating these uniformly (everything goes through the app) would couple every
request to the domain crate for what is, on the verification path, just a
crypto operation.

## Decision

Split the two operations:

- **Issuance** is exposed in `sitehub-app` as a `TokenIssuer` port (trait).
  The login use case orchestrates credential checking, then asks the
  `TokenIssuer` to mint a token. The concrete implementation lives in
  `sitehub-jwt`. `sitehub-auth-api` calls the use case; it does not touch
  JWT primitives directly.
- **Verification** is a middleware concern. Each driving adapter that
  requires authentication uses a thin middleware that calls a verifier
  (from `sitehub-jwt`) to check the signature and expiry of the token,
  decode the claims, and attach them to the request as an extractor.
  Verification does **not** call into `sitehub-app`.

## Alternatives considered

- **All auth operations through `sitehub-app`.** Rejected — verification
  would add a function-call layer (and potentially a database hit if claims
  were re-fetched) on every request, with no business logic to justify it.
- **JWT primitives duplicated in each adapter.** Rejected — drift risk,
  inconsistent claim handling, duplicate key management.
- **JWT logic embedded in `sitehub-storage`.** Rejected — coupling crypto
  to the DB adapter is conceptually wrong and slows incremental compiles.

## Consequences

- **Positive**: clean separation between domain-relevant token issuance and
  framework-level verification. `sitehub-app` defines what claims mean
  (the `TokenIssuer` trait + claims types); `sitehub-jwt` handles
  signature/algorithm specifics. Verification is fast (no app-layer
  round-trip) and uniform across adapters.
- **Costs accepted**: claim types (e.g., `Claims { actor_id, tenant_id, role }`)
  must be defined in a place all driving adapters can import. They live in
  `sitehub-app` (so they're part of the domain contract).
- **Mitigations**: a single source of truth for claim shape in
  `sitehub-app::auth`. Key rotation handled by `sitehub-jwt` with config
  from the composition root.

## Revisit when

- We need to revoke tokens before expiry (would require a verifier that
  consults a revocation store — i.e., adapter-layer state).
- Claims grow beyond what JWT comfortably carries (consider opaque tokens
  backed by a session store).

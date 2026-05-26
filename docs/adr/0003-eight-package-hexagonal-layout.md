# ADR-0003: Eight-package hexagonal workspace layout

Date: 2026-05-20
Status: Accepted — partially superseded by ADR-0009 (jwt renamed to tokens) and ADR-0010 (mobile-api removed; now 7 packages)

## Context

We chose hexagonal architecture (ports & adapters) for sitehub-backend. The
question was how to map the architecture onto Cargo packages within a single
workspace. Goals:

1. The dependency rule (domain depends on nothing; adapters depend on the
   domain; domain never depends on adapters) should be **enforced by the
   compiler**, not by convention.
2. Independent compile units for faster incremental builds — changing the
   HTTP framework should not recompile storage, and vice versa.
3. The architecture should remain legible: each package should answer one
   clear question.

## Decision

Eight packages in a **flat workspace layout** (no `crates/` or `packages/`
subfolder):

| Package | Role |
|---|---|
| `sitehub-app` | Domain, ports (traits), use cases, `AppError`. Zero I/O. |
| `sitehub-storage` | Driven adapter: SurrealDB repository implementations. |
| `sitehub-jwt` | Driven adapter: JWT issuance and verification. |
| `sitehub-public-api` | Driving adapter: REST for per-tenant static sites. |
| `sitehub-admin-api` | Driving adapter: REST for school staff. |
| `sitehub-mobile-api` | Driving adapter: REST for the mobile app. |
| `sitehub-auth-api` | Driving adapter: login, refresh, password reset. |
| `sitehub-bin` | Composition root, single binary. |

Dependency rules:

- `sitehub-app` depends on nothing internal.
- Driven and driving adapters each depend on `sitehub-app` only.
- `sitehub-bin` is the only package that imports both adapter sides.

## Alternatives considered

- **Three packages (`app`, `storage`, `api` — with `main.rs` inside `api`).**
  Rejected — fewer compile boundaries, the api package can accidentally import
  storage, and changing axum forces storage recompilation.
- **Strict DDD: separate `domain` and `application` packages.** Rejected —
  domain types and use cases are tightly coupled today; the split adds two
  more `Cargo.toml`s without earning a real compile boundary. Can be done
  later if business logic outgrows the merged package.
- **Single driving-adapter package with route modules for public/admin/mobile/auth.**
  Rejected — would compile axum, tower-http, and (eventually) auth middleware
  for every API, even when only one changed.

## Consequences

- **Positive**: the package graph physically prevents `sitehub-app` from
  importing a framework or DB driver. Each driving adapter recompiles
  independently of the others. The composition root is unambiguous.
- **Costs accepted**: eight `Cargo.toml` files to keep aligned. More upfront
  ceremony than a single-package or three-package layout. Some packages
  start small (`sitehub-mobile-api` is a stub) and risk looking like
  scaffolding.
- **Mitigations**: shared dependencies are declared once in
  `[workspace.dependencies]` so packages reference with `{ workspace = true }`,
  keeping per-package manifests short. If `sitehub-mobile-api` is not built
  out, it will be deleted rather than left as dead weight.

## Revisit when

- Business logic in `sitehub-app` outgrows the merged domain+application
  layout (consider splitting then).
- A driving adapter is never used (delete it rather than keep a stub).
- Compile-time benefits stop materializing in practice.

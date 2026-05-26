# ADR-0025: Per-API test projects instead of shared integration tests

Date: 2026-05-26
Status: Accepted

## Context

The backend exposes three distinct APIs — public, admin, and auth — each as a separate driving adapter package. These APIs differ in authentication, tenancy model, and routing. In the future, they may be deployed as separate services.

We need an API testing strategy that:

1. Tests each API in isolation via real HTTP requests (not unit tests with mocked handlers).
2. Enforces separation between APIs from the start, so tests don't accidentally couple across API boundaries.
3. Scales naturally if the APIs are later split into separate binaries or services.

The common Rust pattern of `#[ignore]`-annotated tests mixed into production crates was rejected as an anti-pattern — it conflates unit and integration tests, requires flag discipline, and doesn't enforce API boundary separation.

## Decision

One dedicated test project per API, living at the workspace root:

| Test project | Tests against | Auth model |
|---|---|---|
| `sitehub-public-api-tests` | Public API (`*.sitehub.bg`) | None — tenant from subdomain |
| `sitehub-admin-api-tests` | Admin API (`admin.sitehub.bg`) | JWT required |
| `sitehub-auth-api-tests` | Auth API (`auth.sitehub.bg`) | Login flow |

Each test project:

- Depends only on `reqwest`, `serde_json`, and `tokio` — no internal crate dependencies.
- Sends real HTTP requests to a running server (started in CI or locally).
- Reads `SITEHUB_TEST_URL` from the environment to locate the server.
- Runs in CI as a parallel matrix job (`test-api`) after the `build` job warms the cache.

The test projects are workspace members but are excluded from the production Docker build via `.dockerignore`.

## Alternatives considered

- **`#[ignore]` tests in production crates** — mixes integration tests with unit tests. Requires developers to remember `--run-ignored`. Doesn't enforce API boundary separation. A test in `sitehub-public-api` could import `sitehub-admin-api` internals. Rejected.
- **Single `sitehub-api-tests` project for all APIs** — simpler to manage but allows tests to share helpers that cross API boundaries. When APIs diverge (different auth, different deployment), the shared project becomes a coupling point. Rejected.
- **`tests/` directory in each adapter crate** — Cargo's built-in integration test pattern. These run in the same process as the crate and can access `pub(crate)` items, which defeats the purpose of black-box API testing. Rejected.
- **External test framework (e.g., Hurl, Bruno, Postman)** — non-Rust tooling adds a dependency and can't share types or test infrastructure with the Rust codebase. May be added later as a complement, not a replacement. Rejected as primary approach.

## Consequences

- **Positive**: each API's tests are completely isolated — different auth setup, different fixtures, different assertions. Tests are black-box by design (HTTP in, JSON out). The structure naturally scales to separate services. CI runs all three in parallel via matrix strategy.
- **Costs accepted**: three test projects means three sets of boilerplate (Cargo.toml, CI setup). Each CI matrix job starts its own SurrealDB + server instance, consuming more runner minutes than a single combined job. Test helpers can't be shared across projects without a fourth shared crate.
- **Mitigations**: matrix strategy keeps the CI definition DRY (one job definition, three executions). Boilerplate is minimal (each project is ~20 lines of Cargo.toml). A shared `sitehub-test-utils` crate can be added later if helper duplication becomes a problem.

## Revisit when

- APIs are deployed as separate services — test projects can point `SITEHUB_TEST_URL` at different endpoints.
- Test helper duplication exceeds ~50 lines — add a shared `sitehub-test-utils` crate.
- CI runner costs become a concern — merge back into a single job that runs all three sequentially.

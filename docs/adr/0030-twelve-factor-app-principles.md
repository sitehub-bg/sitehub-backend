# ADR-0030: Adopt 12-Factor App principles

Date: 2026-05-29
Status: Accepted

## Context

The project deploys to multiple environments (staging, production) on Fly.io and will continue to grow as more APIs and backing services are added. Without a consistent deployment philosophy, the codebase tends to accumulate per-environment branches, baked-in config, and divergent build artifacts — all of which break "staging mirrors production" (ADR-0027) and make incident response harder.

The [12-Factor App](https://12factor.net) is a well-established methodology for building deployable backend services. It is widely understood in the industry, which makes it a good reference point for both onboarding and interview discussions.

## Decision

Adopt 12-Factor App principles as the deployment and operational philosophy for sitehub-backend. The factors apply as follows:

| Factor | How it applies here |
|---|---|
| **1. Codebase** | One Git repo (`sitehub-backend`), many deploys (staging, production). |
| **2. Dependencies** | Cargo workspace pins all crate versions. No system-level dependencies leaked through. |
| **3. Config** | All environment-specific values live in env vars / Fly secrets. Nothing baked into images. |
| **4. Backing services** | SurrealDB is an attached resource referenced by URL. Swappable without code changes. |
| **5. Build, release, run** | One build produces one image. Releases combine that image with env config. Runs execute releases. No rebuilds per environment. |
| **6. Processes** | Backend is stateless. State lives in SurrealDB. |
| **7. Port binding** | The binary self-binds to a port (3000). Fly maps external traffic to it. |
| **8. Concurrency** | Horizontal scaling via Fly machine count. No threading tricks for scale. |
| **9. Disposability** | Fast startup, graceful SIGTERM shutdown. |
| **10. Dev/prod parity** | Staging mirrors production (ADR-0027). Same SurrealDB version, same Fly config shape, same region (`ams`). |
| **11. Logs** | `tracing` writes to stdout. Fly captures the stream. No log files. |
| **12. Admin processes** | Migrations and one-off tasks run as separate invocations of the same binary, not as web requests. |

Factor 5 (build/release/run separation) is the most operationally significant: it means **one image is built per commit, then deployed unchanged to staging and later promoted to production**. Production deploys do not rebuild — they re-tag and release the staging-tested image with production config.

## Alternatives considered

- **No formal methodology** — what we had. Works at small scale but tends to drift; each new service adds inconsistency.
- **GitOps (ArgoCD / Flux)** — more rigorous but oversized for a single-app side project. Re-evaluate if we operate multiple services.
- **Distroless / scratch images only** — orthogonal; we already use a minimal `debian:slim` base. Can adopt later without conflict.

## Consequences

- **Positive**:
  - Production deploys go from ~5 min (rebuild) to ~30 sec (re-tag) — see ADR-0028 for build cost.
  - Strong guarantee that production runs the bits that passed staging.
  - Easier to onboard contributors familiar with the 12-factor model.
- **Costs accepted**:
  - One promoted image means staging and production can't use different build profiles. Both use `release`.
  - Requires discipline: no env-specific code paths, no baked-in secrets, no log files.
- **Mitigations**:
  - Lint/check during code review for env-specific branches.
  - CI fails if `.env` files are committed (already enforced via `.gitignore` + dependabot).

## Revisit when

- The app outgrows a single binary and we genuinely need multiple services with independent lifecycles.
- A specific factor becomes a constraint (e.g., we need to store state locally — would break factor 6).

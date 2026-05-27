# ADR-0028: CI artifact passing for test jobs

Date: 2026-05-27
Status: Accepted

## Context

Our CI pipeline has a `build` job that compiles the entire workspace, followed by parallel `test-unit`, `test-api`, and `clippy` jobs. The test jobs were taking 6–8 minutes each despite depending on the build job, because restoring the Rust cache and re-linking binaries added significant overhead. The actual tests (currently dummy/placeholder) run in seconds.

## Decision

The `build` job creates a nextest archive (`cargo nextest archive`) containing all pre-built test binaries and uploads it alongside the server binary as GitHub Actions artifacts. Downstream test jobs download these artifacts and run tests directly — no Rust toolchain installation, no cache restoration, no compilation.

Clippy still uses the shared Rust cache because it needs to invoke the compiler.

## Alternatives considered

- **Shared Rust cache only** — what we had before. Cache restoration works but still triggers freshness checks and re-linking, adding 3–5 minutes per job.
- **Single job for build + test** — eliminates artifact passing but removes parallelism between clippy, unit tests, and API tests.
- **Uploading `target/` as artifact** — the directory is multiple GB; upload/download time would exceed the compilation time saved.

## Consequences

- **Positive**: test jobs drop from ~8 minutes to ~2 minutes. No Rust toolchain needed in test runners.
- **Costs accepted**: build job takes ~1 minute longer to create and upload archives. Artifact storage uses ~50–100 MB with 1-day retention.
- **Mitigations**: retention set to 1 day to minimize storage cost.

## Revisit when

- Nextest archive format changes in a breaking way.
- Test binaries grow large enough that artifact upload/download becomes the bottleneck.

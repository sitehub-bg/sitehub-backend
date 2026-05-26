# ADR-0018: Release profile — size-optimized with safety checks

Date: 2026-05-26
Status: Accepted

## Context

The binary deploys to Fly.io containers. Relevant constraints:

- Container images should be small (faster pulls, less registry storage).
- The service handles HTTP requests — throughput matters but is bounded by network and database I/O, not CPU-bound computation.
- Production debugging relies on structured logging (`tracing`), not debug symbols or stack inspection.
- Integer overflow bugs should never silently produce wrong data in production.

We also configure the dev profile for fast iteration: dependencies optimized but workspace code unoptimized.

## Decision

```toml
[profile.dev.package."*"]
opt-level = 2

[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
strip = "debuginfo"
overflow-checks = true
```

Rationale for each setting:

- **`opt-level = "z"`** — optimize for binary size. For an I/O-bound HTTP backend, the ~5-10% runtime speed difference vs `opt-level = 3` is negligible, but the binary size reduction (often 30-50%) meaningfully shrinks Docker images.
- **`lto = "fat"`** — full link-time optimization across all crates. Maximizes dead code elimination and cross-crate inlining. Slow link time is acceptable since release builds happen only in CI.
- **`codegen-units = 1`** — single compilation unit per crate. Enables maximum optimization opportunities. Combined with fat LTO, this gives the optimizer complete program visibility.
- **`strip = "debuginfo"`** — removes debug info (DWARF) but keeps symbol names. Production stack traces still show function names (useful in panic messages and tracing spans), but binary is significantly smaller. Full `strip = "symbols"` would lose function names in backtraces.
- **`overflow-checks = true`** — arithmetic overflow panics rather than wrapping silently. Default is off in release. We enable it because silent overflow in business logic (e.g., calculating quotas, counting items) is worse than a loud panic that gets logged and investigated.
- **`panic = "unwind"` (default, not changed)** — we keep unwinding rather than abort. The marginal size/speed benefit of abort isn't worth losing the ability to run destructors and produce full panic backtraces.

Dev profile:
- **`[profile.dev.package."*"] opt-level = 2`** — all dependencies compile with optimizations, but our workspace code stays at `opt-level = 0`. This means `cargo run` in dev is fast to recompile after code changes, but the app runs at near-release speed (since most CPU time is in deps like surrealdb, tokio, serde).

## Alternatives considered

- **`opt-level = 3` for release** — maximum runtime speed. Rejected because the binary size increase is significant and the speed difference is unmeasurable for an I/O-bound service.
- **`strip = "symbols"` (full strip)** — smallest possible binary. Rejected because panic backtraces would show only addresses, not function names, making production incident investigation harder.
- **`strip = "none"`** — keeps everything. Rejected because debug info in the Docker image adds 50-100MB with no benefit (we don't attach debuggers to production containers).
- **`panic = "abort"`** — slightly smaller binary, no unwind tables. Rejected because the 5-10% size benefit isn't worth losing proper cleanup behavior and readable panic output.
- **`overflow-checks = false` (release default)** — wrapping arithmetic is faster by a few nanoseconds per operation. Rejected because correctness matters more than throughput for business logic, and the performance cost is negligible at the HTTP request scale.

## Consequences

- **Positive**: small Docker images (~50-70% smaller than unoptimized). Production overflow bugs are caught immediately as panics rather than producing silent corruption. Dev builds iterate quickly while running at realistic speed.
- **Costs accepted**: release builds are slow (~5-10 minutes due to fat LTO). First dev build is slow (deps compile at opt-level 2). Overflow panics in production require that arithmetic edge cases are handled in code rather than relying on wrapping.
- **Mitigations**: release builds only happen in CI (developers never wait for them). Dev dep compilation is a one-time cost cached by cargo. Overflow-sensitive code should use `saturating_*` or `checked_*` methods explicitly where wrapping is intentional.

## Revisit when

- The service becomes CPU-bound (e.g., heavy computation in request path) — switch to `opt-level = 3`.
- Docker image size is no longer a concern (e.g., move to a platform with pre-cached layers).
- Release CI time becomes a bottleneck — consider `lto = "thin"` as a compromise.

# ADR-0036: Forbid `unsafe` Rust workspace-wide

Date: 2026-05-30
Status: Accepted

## Context

Rust's `unsafe` keyword unlocks memory-unsafe operations (raw pointer dereference, FFI calls, mutable static access, unchecked type conversions) that the compiler cannot verify for correctness. Misuse silently introduces undefined behavior — use-after-free, data races, invalid type punning — that produces the worst class of bugs: ones the compiler usually catches.

For a backend service like sitehub, none of the legitimate reasons to reach for `unsafe` apply:

- We do not implement low-level data structures whose performance requires raw pointers.
- We do not link against C libraries that require FFI (we use `rustls` instead of `openssl`, per ADR-0020).
- We do not write SIMD intrinsics or platform-specific micro-optimisations.
- We do not maintain mutable global state.

Standard library functions that became `unsafe` in Rust 2024 (notably `std::env::set_var` and `std::env::remove_var`, marked unsafe because they're not thread-safe) need handling somewhere — but that handling can live in dedicated test-helper crates rather than in our code.

## Decision

The workspace forbids `unsafe` code via the workspace-wide lint:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
```

`forbid` is intentionally stricter than `deny`: it cannot be overridden with `#[allow(unsafe_code)]` anywhere in the workspace. There is no path to introducing `unsafe` short of changing this ADR and the lint level.

When a use case appears to require `unsafe` (e.g., manipulating process-wide env vars in tests), the resolution is:

1. **Find a third-party crate that already wraps the unsafe operation safely.** For env-var tests, `temp-env` does this — it provides a closure-scoped API that handles env restoration and contains the unsafe internally. The unsafe stays out of our code.
2. **If no such crate exists**, ask whether the operation is really needed. Most "need unsafe" intuitions in application code are wrong — there's usually a safe abstraction available.
3. **If the operation is genuinely necessary** and no wrapper exists, this ADR is the document to revise. Don't sneak `unsafe` in by changing `forbid` to `deny`.

## Alternatives considered

- **`unsafe_code = "deny"`** — allows `#[allow(unsafe_code)]` overrides at the call site. Rejected because it makes "just allow it here" too easy in code review. `forbid` requires explicit conversation.
- **Allow `unsafe` only in tests** — the lint system doesn't cleanly express this; we'd need to split test code into separate crates or relax the lint workspace-wide. Better to use safe wrapper crates instead.
- **Allow `unsafe` workspace-wide** — defeats the purpose. The reason to use Rust here is to inherit the safety guarantees by default.

## Consequences

- **Positive**:
  - The class of bugs `unsafe` enables (UB, data races, type confusion) cannot appear in our codebase.
  - Reviewers don't have to evaluate `unsafe` blocks for correctness; the compiler does it for us.
  - Reinforces a discipline of reaching for safe abstractions first.
- **Costs accepted**:
  - Tests that touch process-wide state (env vars, signal handlers in non-tokio paths, etc.) need third-party helper crates like `temp-env`. Small extra dependency cost.
  - Some performance optimisations that genuinely require `unsafe` are off the table. For a backend serving HTTP requests, this is unlikely to matter.
- **Mitigations**:
  - When the right wrapper crate doesn't exist, the conversation about adding one is itself useful — it surfaces what the project actually needs from its dependencies.

## Revisit when

- A genuine performance requirement appears that cannot be met without `unsafe` (e.g., a hot path that needs SIMD or `MaybeUninit` tricks) AND profiling has confirmed the gain is material.
- We need to wrap a C library that has no Rust-native alternative.
- Rust's standard library evolves in a way that makes our current safe abstractions unavailable.

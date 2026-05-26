# ADR-0014: Conventional commit version bumping via git hook

Date: 2026-05-26
Status: Accepted

## Context

We need automatic version incrementing for staging deploys (every merge to main) while keeping production releases manual (via git tag). The version must live in `Cargo.toml` so the binary can report it, and the version bump should not create extra commits or PRs that pollute the git history.

Tools like release-plz and cargo-release create dedicated "release PRs" or separate bump commits. For a small team (currently one developer), this overhead adds noise without value.

## Decision

A `commit-msg` git hook (`.githooks/commit-msg`) parses the conventional commit prefix and bumps the workspace version in-place, amending the result into the original commit. The mapping:

- `feat!:` or `BREAKING CHANGE` → major bump
- `feat:` → minor bump
- `fix:` → patch bump
- Anything else → no bump

Production deploys are triggered by manually pushing a `v*` tag pointing at the desired commit.

Contributors must run `git config core.hooksPath .githooks` after cloning (documented in Quick Start).

## Alternatives considered

- **release-plz** — creates a "Release PR" on every push to main with bumped versions and a changelog. Rejected because it adds merge commits and PRs that pollute history for a single-developer project.
- **cargo-release** — manual CLI command (`cargo release patch`). Rejected because it requires remembering to run it and doesn't integrate with the staging auto-deploy flow.
- **CI-based bump commit** — a GitHub Action that bumps and pushes a commit to main after merge. Rejected because it creates an extra commit per merge and can trigger CI loops.
- **Git tag-only versioning (vergen/shadow-rs)** — Cargo.toml stays static, binary reports version from git tag at build time. Rejected because it means `Cargo.toml` never reflects the actual deployed version, making it harder to reason about what's running.

## Consequences

- **Positive**: every commit with a `feat:` or `fix:` prefix carries its own unique version. No extra commits, PRs, or CI steps. The version in `Cargo.toml` always matches what's deployed.
- **Costs accepted**: contributors must configure the hooks path after cloning. Commits without conventional prefixes don't bump — discipline required. Rebasing or squashing can produce unexpected version sequences (gaps are harmless).
- **Mitigations**: hook path setup is a single command documented in README step 1. CI could optionally validate that the version in `Cargo.toml` is monotonically increasing on main (not implemented yet).

## Revisit when

- The team grows beyond 2-3 developers and hook discipline becomes unreliable.
- We need a generated CHANGELOG for external stakeholders (release-plz would then be worth the history noise).
- We publish crates to a registry and need coordinated multi-package versioning.

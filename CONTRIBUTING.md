# Contributing to sitehub-backend

## Setup

```bash
git clone <repo-url>
cd sitehub-backend
git config core.hooksPath .githooks
cp .env.example .env
docker compose up -d
cargo build
```

Optional tools for a better dev experience:

```bash
cargo install just cargo-watch cargo-nextest
sudo apt install mold clang   # faster linking (see README)
```

## Making changes

### Branch workflow

1. Create a branch from `main`.
2. Make your changes in small, focused commits.
3. Use [conventional commit](https://www.conventionalcommits.org/) prefixes — the git hook auto-bumps the version based on them:
   - `feat:` — new functionality (minor version bump)
   - `fix:` — bug fix (patch version bump)
   - `feat!:` or `BREAKING CHANGE` in body — breaking change (major version bump)
   - `chore:`, `docs:`, `refactor:`, `test:`, `ci:` — no version bump
4. Run checks before pushing:
   ```bash
   just check    # or: cargo +nightly fmt --all && cargo clippy --locked --workspace --all-targets -- -D warnings && cargo nextest run --locked --workspace && cargo deny check
   ```
5. Open a PR using the template.

### Architecture changes

If your change affects package structure, dependency rules, error handling patterns, or any decision documented in an [ADR](docs/adr/), file a new ADR before or alongside the code change. Copy `docs/adr/0000-template.md` and use the next available number.

### Code conventions

- **No `unsafe` code** — `unsafe_code = "forbid"` is enforced workspace-wide.
- **`sitehub-app` has no I/O dependencies** — no `axum`, `surrealdb`, or `tokio`. If you need one, the design is wrong.
- **No adapter depends on another adapter** — they share only `sitehub-app`.
- **Overflow-safe arithmetic** — `overflow-checks = true` in release. Use `checked_*` or `saturating_*` methods where wrapping is intentional.
- **Formatting** — we use nightly rustfmt for import grouping. Run `cargo +nightly fmt --all` or `just fmt`.

### Tests

- Unit tests live next to the code they test (`#[cfg(test)]` modules).
- Integration tests (anything that needs SurrealDB) are marked with `#[ignore]` and run separately via `just test-integration`.
- SurrealDB must be running for integration tests: `just db`.

## PR checklist

- [ ] Conventional commit prefix on all commits
- [ ] `just check` passes locally
- [ ] Tests added or updated for the change
- [ ] ADR filed if the change is architectural
- [ ] No secrets or credentials in the diff

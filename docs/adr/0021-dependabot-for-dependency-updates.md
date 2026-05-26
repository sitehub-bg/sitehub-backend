# ADR-0021: Dependabot for automated dependency updates

Date: 2026-05-26
Status: Accepted

## Context

Dependencies (Rust crates, GitHub Actions, Docker base images) receive security patches, bug fixes, and new features over time. Without automation, dependency updates happen only when a developer remembers to check — which means security patches can sit unmerged for weeks.

We need to decide how to handle updates, and whether they should trigger version bumps.

## Decision

We use GitHub Dependabot (`.github/dependabot.yml`) to automatically open PRs for dependency updates:

- **Cargo crates** — weekly on Mondays, max 5 open PRs.
- **GitHub Actions** — weekly.
- **Docker base images** — monthly.

All Dependabot PRs use the `chore:` conventional commit prefix, which means **no version bump** on merge. The rationale:

1. CI (clippy, tests, integration tests, cargo-deny) runs on every Dependabot PR. If the update breaks anything, the PR fails and is never merged.
2. A dependency update that passes CI is functionally transparent to the application — it changes no behavior that our tests can observe.
3. If a dependency update does introduce a behavioral change we care about, we handle it manually with a `fix:` or `feat:` commit that describes the actual change, not the dep bump.
4. Staging deploys automatically on merge to main. If a dep update causes a runtime issue not caught by tests, we observe it on staging and roll back.

## Alternatives considered

- **`fix:` prefix for all dep updates** — triggers a patch bump on every merge. Rejected because it inflates version numbers without corresponding functional changes. A week with 5 dep updates would bump 5 patch versions with no user-visible difference.
- **Manual dependency updates only** — requires developer discipline to check regularly. Rejected because security patches would be delayed indefinitely on a side project with sporadic development.
- **Renovate** — third-party alternative to Dependabot with more configuration options (grouping, auto-merge). Rejected for now because Dependabot is built into GitHub (zero setup beyond the YAML) and sufficient for this project's scale.
- **Auto-merge Dependabot PRs** — removes the human review step. Rejected because even with CI, a quick scan of changelogs is valuable for breaking-change awareness.

## Consequences

- **Positive**: security patches arrive as PRs within a week. No manual effort to track updates. CI validates each update before merge.
- **Costs accepted**: up to 5 open Cargo PRs per week adds review overhead. Dependabot PRs can be noisy during major ecosystem updates (e.g., a new tokio minor version cascading through many crates).
- **Mitigations**: `open-pull-requests-limit: 5` caps the noise. Labels (`dependencies`, `ci`, `docker`) allow filtering. Monthly schedule for Docker reduces base image churn.

## Revisit when

- PR noise becomes unmanageable — consider Renovate's grouping feature to batch related updates.
- We want zero-touch updates — enable auto-merge for patch-level updates that pass CI.
- The project goes dormant — disable Dependabot to avoid stale open PRs accumulating.

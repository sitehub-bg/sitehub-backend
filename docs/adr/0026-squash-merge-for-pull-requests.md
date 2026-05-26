# ADR-0026: Squash merge for pull requests

Date: 2026-05-26
Status: Accepted

## Context

When merging a PR to `main`, GitHub offers three strategies: merge commit, rebase and merge, and squash and merge. Each produces a different git history shape and affects contribution tracking (GitHub green squares), readability of `git log`, and bisect usability.

## Decision

Use **squash and merge** as the default PR merge strategy. All commits on a feature branch are collapsed into a single commit on `main`.

The squash commit message should follow the conventional commit format (`feat:`, `fix:`, `chore:`) so the version-bump git hook works correctly on the resulting commit.

## Alternatives considered

- **Merge commit** — preserves all branch commits plus an extra merge commit. Produces a noisy history with "Merge branch X into main" entries interleaved with feature work. Every branch commit appears in `git log --first-parent` output. Rejected because the history becomes hard to read for a reviewer or interviewer scanning `git log`.
- **Rebase and merge** — replays each branch commit onto `main` individually. Clean linear history but preserves every intermediate commit ("wip", "fix typo", "actually fix it"). Rejected because intermediate commits are noise — the PR review already validated the final result, not each step.

## Consequences

- **Positive**: every commit on `main` represents one reviewed, complete change. `git log` reads as a clean changelog. `git bisect` is effective because each commit is a coherent unit. The version-bump hook fires once per merged PR with the correct conventional prefix.
- **Costs accepted**: individual commit history from branches is lost on `main` (still visible on the PR page in GitHub). Each PR produces one green square on the contribution graph, not one per branch commit.
- **Mitigations**: detailed work history is preserved in the PR description and on the branch (GitHub retains closed PR branches). The PR template captures the "what" and "why" that individual commits would have shown.

## Revisit when

- The team grows and needs to attribute specific changes within a PR to different authors.
- We need to cherry-pick individual changes from a large PR (squash makes this impossible).

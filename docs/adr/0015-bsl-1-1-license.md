# ADR-0015: BSL-1.1 license with MIT change license

Date: 2026-05-26
Status: Accepted

## Context

The project is a commercial SaaS backend owned by Eliseus Tech Ltd (Bulgaria). It also serves as a portfolio piece for senior engineering interviews. We need a license that:

1. Allows the source code to be publicly visible on GitHub (portfolio value).
2. Prevents competitors from running the code in production without permission.
3. Is well-understood in the industry (not a custom or obscure license).

## Decision

We use the Business Source License 1.1 (BSL-1.1) with the following parameters:

- **Licensor**: Eliseus Tech Ltd (Bulgaria)
- **Change Date**: Four years from each file's most recent modification
- **Change License**: MIT License
- **Additional Use Grant**: None

After 4 years from last modification, code converts to MIT automatically.

## Alternatives considered

- **Proprietary (All Rights Reserved)** — maximum protection but zero portfolio value if the repo is private. If public, "All Rights Reserved" with visible source is legally ambiguous and less industry-recognized than BSL.
- **AGPL-3.0** — strong copyleft that forces network users to open-source modifications. Rejected because a competitor could still legally run it if they comply with the AGPL terms, and AGPL scares away potential enterprise partners.
- **MIT / Apache-2.0** — fully permissive. Maximum portfolio visibility but zero commercial protection. Anyone could fork and compete immediately.
- **Apache-2.0 as change license** — considered over MIT for the patent grant. Rejected because a web backend is unlikely to involve patented algorithms, and MIT is simpler and more recognizable.

## Consequences

- **Positive**: code is publicly readable for interviews and portfolio. Legally prevents production use by others. Well-precedented (SurrealDB, HashiCorp, CockroachDB, MariaDB all use BSL variants). Converts to MIT eventually, contributing to open-source commons.
- **Costs accepted**: BSL is not OSI-approved, so the project cannot be called "open source." Some developers may be unfamiliar with BSL terms. The 4-year window means old code becomes MIT even if still commercially valuable.
- **Mitigations**: the LICENSE file is clear and self-contained. The 4-year window is per-file, so actively maintained files stay protected.

## Revisit when

- We take on investors who prefer a different licensing model.
- We want to publish internal crates to crates.io (requires a permissive license).
- The project is no longer commercially viable and could be fully open-sourced.

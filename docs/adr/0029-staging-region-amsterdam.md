# ADR-0029: Primary region moved to Amsterdam

Date: 2026-05-27
Status: Accepted

## Context

The Frankfurt (`fra`) region on Fly.io has persistent capacity constraints. Deploying the staging SurrealDB app repeatedly failed with "insufficient resources to create new machine with existing volume" — the physical hosts backing newly created volumes could not allocate VMs. Deleting and recreating volumes in different zones within `fra` did not resolve the issue. Fly.io community forums confirm ongoing capacity problems in Frankfurt.

## Decision

Move all Fly.io apps (staging and production) to Amsterdam (`ams`). All four apps (`sitehub-backend-staging`, `sitehub-db-staging`, `sitehub-backend-production`, `sitehub-db-production`) use `primary_region = "ams"`.

Backend and DB remain colocated within each environment per ADR-0023.

## Alternatives considered

- **Keep retrying Frankfurt** — no guarantee of resolution; multiple community reports confirm ongoing `fra` capacity issues.
- **Paris (`cdg`) or Warsaw (`waw`)** — viable, but Amsterdam has the most community-reported availability among EU regions.
- **Move only staging to Amsterdam** — rejected; keeping both environments in the same region simplifies operations and avoids Frankfurt capacity issues hitting production later.

## Consequences

- **Positive**: deploys unblocked. Consistent region across all environments.
- **Costs accepted**: latency from Bulgaria is ~5ms higher than Frankfurt (~25ms vs ~20ms). Negligible for the expected user base.
- **Mitigations**: if Amsterdam develops similar capacity issues, any EU region can be substituted by changing `primary_region` in the deploy configs.

## Revisit when

- Frankfurt capacity improves and lower latency to Bulgaria becomes important.
- User base expands to regions where Amsterdam is suboptimal.

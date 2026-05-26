# ADR-0011: Fly.io for backend deployment

Date: 2026-05-20
Status: Accepted

## Context

We need a deployment platform for the sitehub-backend binary. Requirements: simple Docker-based deploys, HTTPS termination, European regions (GDPR), auto-scaling for a side project budget, and minimal operational overhead for a solo developer.

## Decision

Deploy to Fly.io using Docker images built in CI. Two apps: `sitehub-backend-staging` (auto-deploy from main) and `sitehub-backend` (manual deploy via `v*` tags). Fly.io provides TLS termination, health checks, and machine auto-stop/start for cost efficiency.

## Alternatives considered

- **AWS ECS / Fargate** — more control, but significantly more configuration (VPC, ALB, task definitions, IAM roles). Overkill for a side project with one service.
- **Railway / Render** — simpler than AWS but less control over regions and networking. Limited European region support at the time of evaluation.
- **Self-hosted VPS (Hetzner)** — cheapest at scale, but requires managing TLS, reverse proxy, deploys, and monitoring manually. Too much operational burden for a solo developer.
- **Kubernetes (GKE/EKS)** — maximum flexibility, but the operational complexity is unjustifiable for a single binary serving school websites.

## Consequences

- **Positive**: zero-config TLS, Frankfurt region (GDPR-friendly), Docker-native deploys, pay-per-use with auto-stop. Deployment is a single `flyctl deploy` command.
- **Costs accepted**: vendor lock-in on Fly.io's platform. Limited to Fly.io's machine types and networking model. Cold starts when machines scale to zero.
- **Mitigations**: the app is a standard Docker image — migration to another platform requires only changing the deploy step, not the build. Cold starts are mitigated by setting `min_machines_running = 1` for production.

## Revisit when

- Monthly costs exceed what a dedicated VPS would cost at equivalent traffic.
- Fly.io has reliability issues or removes European regions.
- The project needs capabilities Fly.io doesn't support (e.g., GPU, persistent disk beyond volumes).

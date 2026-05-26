# ADR-0022: Official base images only — no third-party Docker images

Date: 2026-05-26
Status: Accepted

## Context

Docker base images are the foundation of the build and runtime environment. A compromised or unmaintained base image can introduce vulnerabilities, unexpected behavior, or supply-chain risk into every build.

The initial Dockerfile used `lukemathwalker/cargo-chef:0.1.71-rust-1.87-bookworm` — a pre-built image from a community maintainer's personal Docker Hub account. While the author is reputable and the tool is widely used, the image is:

1. Controlled by a single individual, not an organization with security processes.
2. Not subject to Docker Official Images review or Verified Publisher auditing.
3. A transitive trust dependency — we trust both the tool (cargo-chef) and the image build pipeline.

## Decision

Use only official or verified publisher Docker images as base images:

- **Build stage**: `rust:1.95-bookworm` (official Rust image maintained by the Docker community and Rust project).
- **Runtime stage**: `debian:bookworm-YYYYMMDD-slim` (official Debian image with pinned date tag).

Third-party tools needed during the build (e.g., `cargo-chef`) are installed from crates.io via `cargo install` inside the official base image, not pulled as pre-built Docker images.

## Alternatives considered

- **Use `lukemathwalker/cargo-chef` pre-built image** — faster first build (cargo-chef is pre-compiled). Rejected because it introduces a supply-chain dependency on a personal Docker Hub account. The time saved (~60s on first build, cached thereafter) does not justify the trust surface.
- **Use Docker Hub Verified Publisher images only** — stricter than "official only" but excludes useful images like the Rust official image (which is community-maintained, not a verified publisher). Too restrictive.
- **Build from scratch (distroless/alpine)** — minimal attack surface but complicates debugging and requires static linking or musl. Revisit if image size becomes critical.

## Consequences

- **Positive**: every base image is traceable to an official, audited source. No dependency on personal Docker Hub accounts. Tools are installed from crates.io (the canonical Rust package registry) with `--locked` for reproducibility.
- **Costs accepted**: first Docker build is slower (~60s to compile cargo-chef). Pinning Rust version in the Dockerfile requires manual updates when upgrading (Dependabot's Docker ecosystem check handles this).
- **Mitigations**: the `cargo install` layer is cached by Docker — only the first build or a Rust version bump pays the compilation cost.

## Revisit when

- A tool we need is only available as a Docker image (no crates.io/apt equivalent).
- Docker Official Images adds a cargo-chef variant.
- Build times become a bottleneck and the caching layer is insufficient.

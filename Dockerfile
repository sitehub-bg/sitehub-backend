# ---------------------------------------------------------------------------
# Stage 1: cargo-chef planner — captures dependency recipe from the workspace
# ---------------------------------------------------------------------------
FROM rust:1.95-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------------------------------------------------------------------------
# Stage 2: dependency cache — builds deps once, re-used until Cargo.toml changes
# ---------------------------------------------------------------------------
FROM chef AS builder

# System deps for compilation only — this layer is discarded in the final image.
# Note: libssl-dev is the C library needed by transitive build scripts, not the
# Rust openssl crate (which is banned in deny.toml — we use rustls for TLS).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json

# Now copy the full source and build the actual binary.
# Because deps are cached, only workspace code is recompiled.
COPY . .
RUN cargo build --release --locked --bin sitehub

# ---------------------------------------------------------------------------
# Stage 3: runtime — minimal image, no compiler, no source
# ---------------------------------------------------------------------------
FROM debian:bookworm-20260518-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Non-root user for defense-in-depth
RUN groupadd --gid 1001 sitehub && \
    useradd --uid 1001 --gid sitehub --shell /bin/false sitehub

COPY --from=builder /app/target/release/sitehub /usr/local/bin/sitehub
COPY --from=builder /app/config /config

WORKDIR /

USER sitehub

EXPOSE 3000

ENTRYPOINT ["sitehub"]

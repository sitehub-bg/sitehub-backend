# sitehub-backend

Multi-tenant backend for sitehub.bg — a platform serving school websites (and later other organizations).

## Architecture

Hexagonal (ports & adapters). Each package compiles independently.

```
sitehub-bin ──► sitehub-public-api ─┐
            ├─► sitehub-admin-api  ─┤
            ├─► sitehub-auth-api   ─┼──► sitehub-app
            ├─► sitehub-storage    ─┤
            └─► sitehub-tokens     ─┘
```

| Package | Role |
|---|---|
| `sitehub-app` | Domain, ports (traits), use cases, `AppError`. No I/O. |
| `sitehub-storage` | Driven adapter: SurrealDB repositories. |
| `sitehub-tokens` | Driven adapter: token issuance and verification (currently JWT). |
| `sitehub-public-api` | Driving adapter: REST for school sites. |
| `sitehub-admin-api` | Driving adapter: REST for school staff (JWT-authenticated). |
| `sitehub-auth-api` | Driving adapter: login, refresh, password reset. |
| `sitehub-bin` | Composition root. Single binary, host-based routing. |

## Prerequisites

- Rust (1.95, managed via `rust-toolchain.toml`)
- Docker & Docker Compose (for SurrealDB)
- [just](https://github.com/casey/just) (`cargo install just`)

## Quick start

```bash
# 1. Bootstrap (installs tools, configures hooks, creates .env)
./scripts/bootstrap.sh

# 2. Start SurrealDB
just db

# 3. Run the server
just run

# 4. Verify
curl http://localhost:3000/api/health
```

## Development commands

```bash
just check              # fmt + clippy + deny + test (run before pushing)
just build              # compile workspace
just run                # start the server
just watch              # start with auto-reload
just test               # unit tests
just test-api           # API tests (requires server + SurrealDB running)
just db                 # start SurrealDB
just db-stop            # stop SurrealDB
just up                 # full stack in Docker (app + SurrealDB)
just down               # stop full stack
just db-proxy           # connect to staging DB via Fly proxy
just                    # list all commands
```

### Manual commands

```bash
# Build
cargo build --locked --workspace
cargo build --release --locked

# Lint & format
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo +nightly fmt --all
cargo +nightly fmt --all -- --check

# Test
cargo nextest run --locked --workspace
cargo deny check

# Docker
docker build -t sitehub .
```

### Faster linking (optional)

Install `mold` and `clang` for 5-10x faster incremental builds, then create `.cargo/config.toml`:

```bash
sudo apt install mold clang
```

```toml
# .cargo/config.toml (not committed — gitignored)
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

## Tenancy

Tenants are identified by subdomain (e.g., `mg-geo-milev.sitehub.bg`). Storage is single-database with `tenant_id` scoping enforced at the repository layer and (later) via SurrealDB record-level access rules.

## API routing

```
admin.sitehub.bg/...    → admin API (JWT required)
auth.sitehub.bg/...     → auth API (login, tokens)
*.sitehub.bg/...        → public API (tenant resolved from subdomain)

Reserved subdomains: admin, auth, api
```

## Deployment secrets

Before the first deploy, set these Fly secrets per app:

```bash
# Database apps
flyctl secrets set SURREAL_USER=... SURREAL_PASS=... --app sitehub-db-staging
flyctl secrets set SURREAL_USER=... SURREAL_PASS=... --app sitehub-db-production

# Backend apps
flyctl secrets set \
  SURREAL_USER=... \
  SURREAL_PASS=... \
  SURREAL_URL=ws://sitehub-db-staging.internal:8000 \
  SURREAL_NS=sitehub \
  SURREAL_DB=main \
  --app sitehub-backend-staging

flyctl secrets set \
  SURREAL_USER=... \
  SURREAL_PASS=... \
  SURREAL_URL=ws://sitehub-db-production.internal:8000 \
  SURREAL_NS=sitehub \
  SURREAL_DB=main \
  --app sitehub-backend-production
```

JWT secrets will be added when the auth adapter is implemented.

## Accessing the staging database

The database is not publicly exposed. To connect for debugging, open a temporary tunnel via Fly proxy:

```bash
flyctl proxy 8000 --app sitehub-db-staging
# DB is now available at localhost:8000 — close the terminal to disconnect
```

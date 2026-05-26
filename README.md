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
| `sitehub-public-api` | Driving adapter: REST for school sites and mobile app (`/web/v1/...`, `/mobile/v1/...`). |
| `sitehub-admin-api` | Driving adapter: REST for school staff (JWT-authenticated). |
| `sitehub-auth-api` | Driving adapter: login, refresh, password reset. |
| `sitehub-bin` | Composition root. Single binary, host-based routing. |

## Prerequisites

- Rust (stable, edition 2024)
- Docker & Docker Compose (for SurrealDB)
- [cargo-watch](https://crates.io/crates/cargo-watch) (optional, for auto-reload)

## Quick start

```bash
# 1. Start SurrealDB
docker compose up -d

# 2. Copy environment config
cp .env.example .env

# 3. Run the server
cargo run -p sitehub-bin

# 4. Verify
curl http://localhost:3000/api/health
```

## Development (Zed editor)

This project includes Zed task definitions in `.zed/tasks.json`. Open the task picker with `ctrl+shift+t` and select a task. Rerun the last task with `ctrl+shift+r`.

### Available tasks

| Task | What it does | When to use |
|---|---|---|
| **DB (start)** | `docker compose up -d` | Start SurrealDB before running the app |
| **DB (stop)** | `docker compose down` | Stop SurrealDB when done |
| **Run** | `cargo run -p sitehub-bin` | Start the server |
| **Run (watch)** | `cargo watch -x 'run -p sitehub-bin'` | Start the server with auto-reload on file es |
| **Build (workspace)** | `cargo build --workspace` | Compile everything |
| **Build (release)** | `cargo build --release` | Compile optimized binary |
| **Build (current package)** | `cargo build -p <current>` | Compile only the package of the file you're editing |
| **Test (workspace)** | `cargo test --workspace` | Run all unit tests |
| **Test (current package)** | `cargo test -p <current>` | Run tests for the package you're editing |
| **Test (integration)** | `cargo test --workspace -- --ignored` | Run integration tests (requires SurrealDB running) |
| **Clippy** | `cargo clippy --workspace --all-targets` | Lint the entire workspace |
| **Format** | `cargo fmt --all` | Auto-format all code |
| **Format (check)** | `cargo fmt --all -- --check` | Check formatting without changing files |
| **Docker (build)** | `docker build -t sitehub .` | Build the production Docker image |

### Typical workflow

```
ctrl+shift+t → "DB (start)"        # start SurrealDB
ctrl+shift+t → "Run (watch)"       # start server with auto-reload
                                    # edit code, server restarts automatically
ctrl+shift+t → "Test (workspace)"  # run tests
ctrl+shift+t → "Clippy"            # lint before committing
```

## Tenancy

Tenants are identified by subdomain (e.g., `mg-geo-milev.sitehub.bg`). Storage is single-database with `tenant_id` scoping enforced at the repository layer and (later) via SurrealDB record-level access rules.

## API routing

```
api.sitehub.bg/web/v1/...       → public API (SSG builds)
api.sitehub.bg/mobile/v1/...    → public API (mobile app)
admin.sitehub.bg/v1/...         → admin API (JWT required)
auth.sitehub.bg/v1/...          → auth API (login, tokens)
```

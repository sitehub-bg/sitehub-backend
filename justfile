# Run all checks (what CI runs)
check: fmt build clippy deny test

# Format code (nightly rustfmt for import grouping)
fmt:
    cargo +nightly fmt --all

# Format check (CI mode)
fmt-check:
    cargo +nightly fmt --all -- --check

# Lint
clippy:
    cargo clippy --locked --workspace --all-targets -- -D warnings

# Dependency audit
deny:
    cargo deny check

# Unit tests (excludes API test projects)
test:
    cargo nextest run --locked --workspace --exclude sitehub-public-api-tests --exclude sitehub-admin-api-tests --exclude sitehub-auth-api-tests

# API tests (requires server + SurrealDB running)
test-api:
    cargo nextest run --locked -p sitehub-public-api-tests -p sitehub-admin-api-tests -p sitehub-auth-api-tests

# Run the server
run:
    cargo run -p sitehub-bin

# Run with auto-reload
watch:
    cargo watch -x 'run -p sitehub-bin'

# Build workspace
build:
    cargo build --locked --workspace

# Build release binary
release:
    cargo build --release --locked

# Start SurrealDB
db:
    docker compose up -d

# Stop SurrealDB
db-stop:
    docker compose down

# Run full stack in Docker (app + SurrealDB)
up:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build

# Stop full stack
down:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml down

# Connect to staging DB via Fly proxy (localhost:8000)
db-proxy:
    flyctl proxy 8000 --app sitehub-db-staging

# Build Docker image
docker:
    docker build -t sitehub .

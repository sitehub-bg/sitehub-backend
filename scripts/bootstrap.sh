#!/usr/bin/env bash
set -euo pipefail

echo "=== sitehub-backend bootstrap ==="

# Git hooks
echo "Configuring git hooks..."
git config core.hooksPath .githooks

# Rust toolchain (rust-toolchain.toml handles stable + components)
echo "Installing nightly toolchain (for rustfmt)..."
rustup toolchain install nightly --component rustfmt

# Cargo tools (cargo-chef is only used inside Docker — see Dockerfile)
echo "Installing cargo tools..."
cargo install just cargo-watch cargo-nextest cargo-deny --locked 2>/dev/null || {
    echo "Some tools may already be installed, continuing..."
}

# Environment
if [ ! -f .env ]; then
    echo "Creating .env from .env.example..."
    cp .env.example .env
else
    echo ".env already exists, skipping."
fi

# Optional: mold linker
if command -v mold &>/dev/null; then
    echo "mold linker detected."
else
    echo ""
    echo "Optional: install mold + clang for faster incremental builds:"
    echo "  sudo apt install mold clang"
    echo "Then create .cargo/config.toml (see README)."
fi

echo ""
echo "=== Done! Next steps: ==="
echo "  just db       # start SurrealDB"
echo "  just run      # start the server"
echo "  just check    # run all checks"

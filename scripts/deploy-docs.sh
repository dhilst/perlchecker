#!/usr/bin/env bash
# Build Sphinx docs and deploy to gh-pages using ghp-import.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
SPHINX_DIR="$ROOT/docs/sphinx"

echo "Building Sphinx docs..."
cd "$SPHINX_DIR"
uv run sphinx-build -b html . _build/html

echo "Deploying to gh-pages..."
uv run ghp-import -n -p -m "Update docs (auto-deploy)" _build/html

echo "Docs deployed to gh-pages."

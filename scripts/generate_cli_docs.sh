#!/usr/bin/env bash
set -euo pipefail

# Generate docs/CLI.md from the compiled binary's --help outputs
# Usage: scripts/generate_cli_docs.sh

ROOT_DIR="$(git rev-parse --show-toplevel)"
OUT_DIR="$ROOT_DIR/docs"
OUT_FILE="$OUT_DIR/CLI.md"
BIN="$ROOT_DIR/target/debug/devkit"

mkdir -p "$OUT_DIR"

# Build once
cargo build -q

# Helper to append a section
append_section() {
  local title="$1"; shift
  local cmd=("$@")
  echo "" >> "$OUT_FILE"
  echo "### $title" >> "$OUT_FILE"
  echo '```text' >> "$OUT_FILE"
  if ! "${cmd[@]}" --color never --help >> "$OUT_FILE" 2>/dev/null; then
    echo "[ERROR] Failed to get help for: ${cmd[*]}" >&2
  fi
  echo '```' >> "$OUT_FILE"
}

# Start file
cat > "$OUT_FILE" <<'HEADER'
# DevKit CLI Reference (Source of Truth)

This document is generated from the CLI's built-in help texts to ensure it always matches the implementation.

## devkit (top-level)

```text
HEADER
"$BIN" --color never --help >> "$OUT_FILE"
echo '```' >> "$OUT_FILE"

echo "" >> "$OUT_FILE"
echo "## Subcommands" >> "$OUT_FILE"

# List of subcommands to document (keep in sync with src/cli/mod.rs)
SUBCMDS=(
  init interactive analyze generate agent config inspect profile template status shell demo
  blueprint plugin chat session visualize dashboard analytics monitor export behavior diagnose
)

for sub in "${SUBCMDS[@]}"; do
  append_section "devkit $sub" "$BIN" "$sub"
done

echo "Wrote $OUT_FILE"

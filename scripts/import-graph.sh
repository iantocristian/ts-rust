#!/usr/bin/env bash
# Regenerates data/import-graph.txt and data/topological-order.txt from a
# microsoft/TypeScript checkout (the Go module under tsc/). Requires go and tsort.
#   scripts/import-graph.sh [path-to-TypeScript-checkout]
set -euo pipefail
TS_REPO="${1:-$HOME/git/TypeScript}"
OUT="$(cd "$(dirname "$0")/.." && pwd)/data"
mkdir -p "$OUT"
cd "$TS_REPO/tsc"
go list -f '{{.ImportPath}} {{join .Imports " "}}' ./internal/... ./cmd/... \
  | awk '{imp=$1; for (i = 2; i <= NF; i++) if ($i ~ /^github.com\/microsoft\/TypeScript\/tsc\//) print imp, $i}' \
  | sed 's#github.com/microsoft/TypeScript/tsc/##g' | sort -u > "$OUT/import-graph.txt"
awk '{print $2, $1}' "$OUT/import-graph.txt" | tsort > "$OUT/topological-order.txt"
echo "measured at microsoft/TypeScript $(git -C "$TS_REPO" rev-parse --short HEAD) on $(date +%F) with $(go version)" > "$OUT/MEASURED.txt"
wc -l "$OUT/import-graph.txt" "$OUT/topological-order.txt"

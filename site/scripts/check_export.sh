#!/usr/bin/env bash
set -euo pipefail

DIST_DIR="${DX_DIST_DIR:-./dist}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LLMS_FILE="$DIST_DIR/llms.txt"
if [ ! -f "$LLMS_FILE" ] && [ -f "$SCRIPT_DIR/../public/llms.txt" ]; then
  LLMS_FILE="$SCRIPT_DIR/../public/llms.txt"
fi

if [ ! -d "$DIST_DIR" ]; then
  echo "error: export directory '$DIST_DIR' does not exist" >&2
  exit 1
fi

if [ -e "$DIST_DIR/_redirects" ]; then
  echo "error: '$DIST_DIR/_redirects' must not exist (redirects are handled by worker/redirects.ts)" >&2
  exit 1
fi

REQUIRED_FILES=(
  "index.html"
  "_headers"
  "llms.txt"
  "search-index.json"
  "docs/overview/introduction.md"
)

if [ ! -f "$LLMS_FILE" ]; then
  echo "error: missing llms.txt to derive component export checks" >&2
  exit 1
fi

component_count=0
while IFS= read -r comp_md; do
  if [ -n "$comp_md" ]; then
    REQUIRED_FILES+=("$comp_md")
    component_count=$((component_count + 1))
  fi
done < <(grep -oE 'docs/components/[a-z0-9_-]+\.md' "$LLMS_FILE" | sort -u)

if [ "$component_count" -eq 0 ]; then
  echo "error: no component markdown entries found in '$LLMS_FILE'" >&2
  exit 1
fi

missing=0
for rel in "${REQUIRED_FILES[@]}"; do
  if [ ! -f "$DIST_DIR/$rel" ]; then
    echo "error: missing required export artifact '$DIST_DIR/$rel'" >&2
    missing=1
  fi
done

if [ "$missing" -ne 0 ]; then
  exit 1
fi

echo "check_export: verified ${#REQUIRED_FILES[@]} required artifacts ($component_count components) in '$DIST_DIR'"

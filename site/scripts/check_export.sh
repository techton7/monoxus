#!/usr/bin/env bash
set -euo pipefail

DIST_DIR="${DX_DIST_DIR:-./dist}"

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
  "docs/components/button.md"
  "docs/components/dialog.md"
  "docs/components/select.md"
  "docs/components/tabs.md"
  "docs/components/tooltip.md"
)

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

echo "check_export: verified ${#REQUIRED_FILES[@]} required artifacts in '$DIST_DIR'"

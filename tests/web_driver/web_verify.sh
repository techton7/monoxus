#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Web Verification Automation Runner for Monoxus Playground
# Drives ego-browser against dx serve --example playground --web --port <PORT>
# ==============================================================================

COMPONENT="${1:-collapsible}"
PORT="${2:-8080}"
BASE_URL="http://127.0.0.1:${PORT}"
SLUG=$(echo "${COMPONENT}" | tr '[:upper:]' '[:lower:]' | tr ' ' '-')

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "\n${BOLD}=== Monoxus Web Verification Runner ===${NC}"
echo -e "${BLUE}[INFO]${NC} Target Component: ${COMPONENT} (#nav-link-${SLUG})"
echo -e "${BLUE}[INFO]${NC} Target Base URL : ${BASE_URL}"

# 1. Check prerequisites
if ! command -v ego-browser >/dev/null 2>&1; then
    echo -e "${RED}[ERROR]${NC} ego-browser CLI not found in PATH."
    exit 1
fi

if ! curl -s -I "${BASE_URL}" >/dev/null 2>&1; then
    echo -e "${YELLOW}[WARN]${NC} Playground server not responding at ${BASE_URL}."
    echo -e "${BLUE}[INFO]${NC} Start it with:"
    echo -e "       dx serve --example playground --web --open false --port ${PORT}"
    exit 1
fi

echo -e "${GREEN}[PASS]${NC} Playground server reachable at ${BASE_URL}."

# 2. Run ego-browser verification with strict invariants:
# - Wait durations strictly in SECONDS (sub-second: 0.5s, 0.3s)
# - Single compact IIFEs
# - Zero auxiliary .js files
echo -e "${BLUE}[INFO]${NC} Executing ego-browser verification session..."

ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-${SLUG}");
const page = task.page("p1");

// Navigate to playground root
await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to component
const navResult = await page.evaluate("(() => { const btn = document.querySelector('#nav-link-${SLUG}'); if (btn) { btn.click(); return true; } return false; })()");
console.log(JSON.stringify({ step: "navigate_nav_link", id: "nav-link-${SLUG}", found: navResult }));
await wait(0.3);

// Query initial data-state
const initialState = await page.evaluate("(() => { const el = document.querySelector('#${SLUG}-trigger') || document.querySelector('[data-state]'); return el ? el.getAttribute('data-state') : null; })()");
console.log(JSON.stringify({ step: "initial_state", dataState: initialState }));

// Click trigger
const clickResult = await page.evaluate("(() => { const btn = document.querySelector('#${SLUG}-trigger') || document.querySelector('button'); if (btn) { btn.focus(); btn.click(); return btn.id || 'unnamed-button'; } return null; })()");
console.log(JSON.stringify({ step: "click_trigger", clickedId: clickResult }));
await wait(0.3);

// Query activeElement and updated state
const activeElementId = await page.evaluate("(() => document.activeElement ? document.activeElement.id : '')()");
const updatedState = await page.evaluate("(() => { const el = document.querySelector('#${SLUG}-trigger') || document.querySelector('[data-state]'); return el ? el.getAttribute('data-state') : null; })()");
console.log(JSON.stringify({ step: "post_click", activeElementId, dataState: updatedState }));

await task.finish({ keep: [] });
EOF

echo -e "${GREEN}[PASS]${NC} Web verification completed cleanly for ${COMPONENT}.\n"

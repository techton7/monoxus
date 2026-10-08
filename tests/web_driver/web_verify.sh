#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Web Verification Automation Runner for Monoxus Playground
# Drives ego-browser against dx serve --example playground --web --port <PORT>
# ==============================================================================

TARGET="${1:-all}"
PORT="${2:-8080}"
BASE_URL="http://127.0.0.1:${PORT}"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "\n${BOLD}=== Monoxus Web Verification Runner ===${NC}"
echo -e "${BLUE}[INFO]${NC} Target         : ${TARGET}"
echo -e "${BLUE}[INFO]${NC} Target Base URL: ${BASE_URL}"

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

verify_component() {
    local comp="$1"
    echo -e "\n${BLUE}[INFO]${NC} Verifying component: ${BOLD}${comp}${NC}..."

    case "${comp}" in
        collapsible)
            ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-collapsible");
const page = task.page("p1");

await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to Collapsible
await page.evaluate("(() => { const btn = document.querySelector('#nav-link-collapsible'); if (btn) btn.click(); })()");
await wait(0.3);

// Click trigger
await page.evaluate("(() => { const btn = document.querySelector('#collapsible-trigger'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify state and focus
const res = await page.evaluate("(() => { \
    const el = document.querySelector('#collapsible-trigger'); \
    const state = el ? el.getAttribute('data-state') : null; \
    const activeId = document.activeElement ? document.activeElement.id : ''; \
    return { state, activeId }; \
})()");
console.log(JSON.stringify(res));

if (res.state !== "open") {
    throw new Error(\`Collapsible state assertion failed: expected 'open', got '\${res.state}'\`);
}
if (res.activeId !== "collapsible-trigger") {
    throw new Error(\`Collapsible focus assertion failed: expected 'collapsible-trigger', got '\${res.activeId}'\`);
}

await task.finish({ keep: [] });
EOF
            echo -e "${GREEN}[PASS]${NC} Collapsible verified (data-state=open, focused)."
            ;;

        accordion)
            ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-accordion");
const page = task.page("p1");

await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to Accordion
await page.evaluate("(() => { const btn = document.querySelector('#nav-link-accordion'); if (btn) btn.click(); })()");
await wait(0.3);

// Click item 1 trigger
await page.evaluate("(() => { const btn = document.querySelector('#accordion-trigger-1'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify state
const res = await page.evaluate("(() => { \
    const el = document.querySelector('#accordion-trigger-1'); \
    const state = el ? el.getAttribute('data-state') : null; \
    return { state }; \
})()");
console.log(JSON.stringify(res));

if (res.state !== "open") {
    throw new Error(\`Accordion state assertion failed: expected 'open', got '\${res.state}'\`);
}

await task.finish({ keep: [] });
EOF
            echo -e "${GREEN}[PASS]${NC} Accordion verified (data-state=open)."
            ;;

        tabs)
            ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-tabs");
const page = task.page("p1");

await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to Tabs
await page.evaluate("(() => { const btn = document.querySelector('#nav-link-tabs'); if (btn) btn.click(); })()");
await wait(0.3);

// Click Tab 2 trigger
await page.evaluate("(() => { const btn = document.querySelector('#tabs-trigger-2'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify state, activeElement, and content
const res = await page.evaluate("(() => { \
    const trigger = document.querySelector('#tabs-trigger-2'); \
    const state = trigger ? trigger.getAttribute('data-state') : null; \
    const activeId = document.activeElement ? document.activeElement.id : ''; \
    const content = document.querySelector('#tabs-content-2'); \
    const contentState = content ? content.getAttribute('data-state') : null; \
    return { state, activeId, contentExists: !!content, contentState }; \
})()");
console.log(JSON.stringify(res));

if (res.state !== "active") {
    throw new Error(\`Tabs trigger state assertion failed: expected 'active', got '\${res.state}'\`);
}
if (res.activeId !== "tabs-trigger-2") {
    throw new Error(\`Tabs focus assertion failed: expected 'tabs-trigger-2', got '\${res.activeId}'\`);
}
if (!res.contentExists || res.contentState !== "active") {
    throw new Error(\`Tabs content assertion failed: contentExists=\${res.contentExists}, contentState=\${res.contentState}\`);
}

await task.finish({ keep: [] });
EOF
            echo -e "${GREEN}[PASS]${NC} Tabs verified (data-state=active, focused, content mounted)."
            ;;

        *)
            echo -e "${RED}[ERROR]${NC} Unknown component: ${comp}"
            exit 1
            ;;
    esac
}

if [ "${TARGET}" = "all" ]; then
    verify_component collapsible
    verify_component accordion
    verify_component tabs
else
    verify_component "${TARGET}"
fi

echo -e "\n${GREEN}[PASS]${NC} All requested web verifications completed cleanly.\n"

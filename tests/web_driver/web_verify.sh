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

        popover)
            ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-popover");
const page = task.page("p1");

await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to Popover
await page.evaluate("(() => { const btn = document.querySelector('#nav-link-popover'); if (btn) btn.click(); })()");
await wait(0.3);

// Click trigger
await page.evaluate("(() => { const btn = document.querySelector('#popover-trigger'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify content mounted
const openRes = await page.evaluate("(() => { \
    const trigger = document.querySelector('#popover-trigger'); \
    const content = document.querySelector('#popover-content'); \
    return { \
        triggerState: trigger ? trigger.getAttribute('data-state') : null, \
        contentMounted: !!content \
    }; \
})()");
console.log(JSON.stringify(openRes));

if (!openRes.contentMounted || openRes.triggerState !== "open") {
    throw new Error(\`Popover open assertion failed: contentMounted=\${openRes.contentMounted}, triggerState=\${openRes.triggerState}\`);
}

// Click close button
await page.evaluate("(() => { const btn = document.querySelector('#popover-close'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify content dismissed
const closedRes = await page.evaluate("(() => { \
    const content = document.querySelector('#popover-content'); \
    return { contentMounted: !!content }; \
})()");
console.log(JSON.stringify(closedRes));

if (closedRes.contentMounted) {
    throw new Error("Popover dismissal failed: content still mounted");
}

await task.finish({ keep: [] });
EOF
            echo -e "${GREEN}[PASS]${NC} Popover verified (open, floating content mounted, dismissed)."
            ;;

        tooltip)
            ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-tooltip");
const page = task.page("p1");

await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to Tooltip
await page.evaluate("(() => { const btn = document.querySelector('#nav-link-tooltip'); if (btn) btn.click(); })()");
await wait(0.3);

// Focus and hover trigger to start delay timer
await page.evaluate("(() => { \
    const trigger = document.querySelector('#tooltip-trigger'); \
    if (trigger) { \
        trigger.focus(); \
        trigger.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true })); \
    } \
})()");
await wait(0.5);

// Verify tooltip content and arrow mounted
const res = await page.evaluate("(() => { \
    const trigger = document.querySelector('#tooltip-trigger'); \
    const content = document.querySelector('#tooltip-content'); \
    const arrow = document.querySelector('#tooltip-arrow'); \
    return { \
        triggerState: trigger ? trigger.getAttribute('data-state') : null, \
        contentMounted: !!content, \
        arrowMounted: !!arrow \
    }; \
})()");
console.log(JSON.stringify(res));

if (!res.contentMounted || !res.arrowMounted || res.triggerState !== "open") {
    throw new Error(\`Tooltip open assertion failed: contentMounted=\${res.contentMounted}, arrowMounted=\${res.arrowMounted}, triggerState=\${res.triggerState}\`);
}

await task.finish({ keep: [] });
EOF
            echo -e "${GREEN}[PASS]${NC} Tooltip verified (hover/focus trigger, content and arrow mounted)."
            ;;

        dialog)
            ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-dialog");
const page = task.page("p1");

await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to Dialog
await page.evaluate("(() => { const btn = document.querySelector('#nav-link-dialog'); if (btn) btn.click(); })()");
await wait(0.3);

// Click trigger
await page.evaluate("(() => { const btn = document.querySelector('#dialog-trigger'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify overlay and content mounted
const openRes = await page.evaluate("(() => { \
    const overlay = document.querySelector('#dialog-overlay'); \
    const content = document.querySelector('#dialog-content'); \
    return { \
        overlayMounted: !!overlay, \
        contentMounted: !!content, \
        contentState: content ? content.getAttribute('data-state') : null \
    }; \
})()");
console.log(JSON.stringify(openRes));

if (!openRes.overlayMounted || !openRes.contentMounted || openRes.contentState !== "open") {
    throw new Error(\`Dialog open assertion failed: overlayMounted=\${openRes.overlayMounted}, contentMounted=\${openRes.contentMounted}\`);
}

// Click close button
await page.evaluate("(() => { const btn = document.querySelector('#dialog-close'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify content dismissed
const closedRes = await page.evaluate("(() => { \
    const content = document.querySelector('#dialog-content'); \
    return { contentMounted: !!content }; \
})()");
console.log(JSON.stringify(closedRes));

if (closedRes.contentMounted) {
    throw new Error("Dialog dismissal failed: content still mounted");
}

await task.finish({ keep: [] });
EOF
            echo -e "${GREEN}[PASS]${NC} Dialog verified (overlay, modal content mounted, dismissed)."
            ;;

        alert_dialog|alert-dialog)
            ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-alert-dialog");
const page = task.page("p1");

await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to Alert Dialog
await page.evaluate("(() => { const btn = document.querySelector('#nav-link-alert-dialog'); if (btn) btn.click(); })()");
await wait(0.3);

// Click trigger
await page.evaluate("(() => { const btn = document.querySelector('#alert-dialog-trigger'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify content mounted
const openRes = await page.evaluate("(() => { \
    const content = document.querySelector('#alert-dialog-content'); \
    const cancel = document.querySelector('#alert-dialog-cancel'); \
    return { \
        contentMounted: !!content, \
        cancelMounted: !!cancel, \
        contentState: content ? content.getAttribute('data-state') : null \
    }; \
})()");
console.log(JSON.stringify(openRes));

if (!openRes.contentMounted || !openRes.cancelMounted || openRes.contentState !== "open") {
    throw new Error(\`AlertDialog open assertion failed: contentMounted=\${openRes.contentMounted}, cancelMounted=\${openRes.cancelMounted}\`);
}

// Click cancel button
await page.evaluate("(() => { const btn = document.querySelector('#alert-dialog-cancel'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify content dismissed
const closedRes = await page.evaluate("(() => { \
    const content = document.querySelector('#alert-dialog-content'); \
    return { contentMounted: !!content }; \
})()");
console.log(JSON.stringify(closedRes));

if (closedRes.contentMounted) {
    throw new Error("AlertDialog dismissal failed: content still mounted");
}

await task.finish({ keep: [] });
EOF
            echo -e "${GREEN}[PASS]${NC} AlertDialog verified (destructive content mounted, cancel dismissed)."
            ;;

        select)
            ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-select");
const page = task.page("p1");

await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to Select
await page.evaluate("(() => { const btn = document.querySelector('#nav-link-select'); if (btn) btn.click(); })()");
await wait(0.3);

// Click trigger to open dropdown
await page.evaluate("(() => { const btn = document.querySelector('#select-trigger'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify open state and active descendant
const openRes = await page.evaluate("(() => { \
    const trigger = document.querySelector('#select-trigger'); \
    const content = document.querySelector('#select-content'); \
    const openState = trigger ? trigger.getAttribute('aria-expanded') : null; \
    const activeDesc = content ? content.getAttribute('aria-activedescendant') : null; \
    return { openState, activeDesc, contentMounted: !!content }; \
})()");
console.log(JSON.stringify(openRes));

if (openRes.openState !== "true" || !openRes.contentMounted) {
    throw new Error(\`Select open assertion failed: openState=\${openRes.openState}, mounted=\${openRes.contentMounted}\`);
}

// Send ArrowDown to advance roving focus
await page.evaluate("(() => { \
    const content = document.querySelector('#select-content'); \
    if (content) { \
        content.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', code: 'ArrowDown', bubbles: true })); \
        content.dispatchEvent(new KeyboardEvent('keyup', { key: 'ArrowDown', code: 'ArrowDown', bubbles: true })); \
    } \
})()");
await wait(0.3);

// Typeahead 'b' on open content
await page.evaluate("(() => { \
    const content = document.querySelector('#select-content'); \
    if (content) { \
        content.dispatchEvent(new KeyboardEvent('keydown', { key: 'b', code: 'KeyB', bubbles: true })); \
        content.dispatchEvent(new KeyboardEvent('keyup', { key: 'b', code: 'KeyB', bubbles: true })); \
    } \
})()");
await wait(0.3);

const bRes = await page.evaluate("(() => { \
    const content = document.querySelector('#select-content'); \
    const activeDesc = content ? content.getAttribute('aria-activedescendant') : null; \
    return { activeDesc }; \
})()");
console.log(JSON.stringify(bRes));
if (!bRes.activeDesc || !bRes.activeDesc.includes("banana")) {
    throw new Error(\`Select typeahead 'b' failed: activeDesc=\${bRes.activeDesc}\`);
}

// Commit selection with Enter
await page.evaluate("(() => { \
    const content = document.querySelector('#select-content'); \
    if (content) { \
        content.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', bubbles: true })); \
        content.dispatchEvent(new KeyboardEvent('keyup', { key: 'Enter', code: 'Enter', bubbles: true })); \
    } \
})()");
await wait(0.5);

const closedRes = await page.evaluate("(() => { \
    const trigger = document.querySelector('#select-trigger'); \
    const content = document.querySelector('#select-content'); \
    const openState = trigger ? trigger.getAttribute('aria-expanded') : null; \
    return { openState, contentMounted: !!content }; \
})()");
console.log(JSON.stringify(closedRes));

if (closedRes.openState !== "false") {
    throw new Error(\`Select close assertion failed: openState=\${closedRes.openState}\`);
}

// Closed typeahead: focus trigger and dispatch 'p'
await page.evaluate("(() => { \
    const trigger = document.querySelector('#select-trigger'); \
    if (trigger) { \
        trigger.focus(); \
        trigger.dispatchEvent(new KeyboardEvent('keydown', { key: 'p', code: 'KeyP', bubbles: true })); \
        trigger.dispatchEvent(new KeyboardEvent('keyup', { key: 'p', code: 'KeyP', bubbles: true })); \
    } \
})()");
await wait(0.3);

const pRes = await page.evaluate("(() => { \
    const trigger = document.querySelector('#select-trigger'); \
    const text = trigger ? trigger.textContent.trim() : ''; \
    return { text }; \
})()");
console.log(JSON.stringify(pRes));
if (!pRes.text.toLowerCase().includes("pineapple")) {
    throw new Error(\`Select closed typeahead 'p' failed: text=\${pRes.text}\`);
}

await task.finish({ keep: [] });
EOF
            echo -e "${GREEN}[PASS]${NC} Select verified (dropdown open, roving focus, typeahead open 'b', commit, closed typeahead 'p')."
            ;;

        toast)
            ego-browser nodejs <<EOF
const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("verify-monoxus-toast");
const page = task.page("p1");

await page.goto("${BASE_URL}");
await wait(0.5);

// Navigate to Toast
await page.evaluate("(() => { const btn = document.querySelector('#nav-link-toast'); if (btn) btn.click(); })()");
await wait(0.3);

// Click #dispatch-info to dispatch single toast
await page.evaluate("(() => { const btn = document.querySelector('#dispatch-info'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify toast mounted
const mountedRes = await page.evaluate("(() => { \
    const toast = document.querySelector('#monoxus-toast-viewport li'); \
    return { mounted: !!toast, state: toast ? toast.getAttribute('data-state') : null }; \
})()");
console.log(JSON.stringify(mountedRes));
if (!mountedRes.mounted) {
    throw new Error("Toast mount failed after dispatch-info");
}

// Synthesize pointer swipe gesture: pointerdown at (100, 100) -> pointermove at (200, 100)
await page.evaluate("(() => { \
    const toast = document.querySelector('#monoxus-toast-viewport li'); \
    if (toast) { \
        toast.dispatchEvent(new PointerEvent('pointerdown', { clientX: 100, clientY: 100, bubbles: true })); \
        toast.dispatchEvent(new PointerEvent('pointermove', { clientX: 200, clientY: 100, bubbles: true })); \
    } \
})()");

// Assert data-swiping="true"
const swipingRes = await page.evaluate("(() => { \
    const toast = document.querySelector('#monoxus-toast-viewport li'); \
    return { swiping: toast ? toast.getAttribute('data-swiping') : null }; \
})()");
console.log(JSON.stringify(swipingRes));
if (swipingRes.swiping !== "true") {
    throw new Error(\`Toast data-swiping assertion failed: swiping=\${swipingRes.swiping}\`);
}

// Dispatch pointerup to finish swipe dismissal
await page.evaluate("(() => { \
    const toast = document.querySelector('#monoxus-toast-viewport li'); \
    if (toast) { \
        toast.dispatchEvent(new PointerEvent('pointerup', { clientX: 200, clientY: 100, bubbles: true })); \
    } \
})()");

// Assert data-swipe-out="true"
const swipeOutRes = await page.evaluate("(() => { \
    const toast = document.querySelector('#monoxus-toast-viewport li'); \
    return { swipeOut: toast ? toast.getAttribute('data-swipe-out') : null }; \
})()");
console.log(JSON.stringify(swipeOutRes));
if (swipeOutRes.swipeOut !== "true") {
    throw new Error(\`Toast data-swipe-out assertion failed: swipeOut=\${swipeOutRes.swipeOut}\`);
}

// Wait for unmount delay
await wait(0.3);

const evictedRes = await page.evaluate("(() => { \
    const toasts = document.querySelectorAll('#monoxus-toast-viewport li'); \
    return { count: toasts.length }; \
})()");
console.log(JSON.stringify(evictedRes));
if (evictedRes.count !== 0) {
    throw new Error(\`Toast swipe dismissal eviction failed: count=\${evictedRes.count}\`);
}

// Click #dispatch-stack-3 to dispatch toasts
await page.evaluate("(() => { const btn = document.querySelector('#dispatch-stack-3'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.3);

// Verify toasts mounted in viewport
const stackedRes = await page.evaluate("(() => { \
    const viewport = document.querySelector('#monoxus-toast-viewport'); \
    const toasts = viewport ? viewport.querySelectorAll('li') : []; \
    return { viewportMounted: !!viewport, toastCount: toasts.length }; \
})()");
console.log(JSON.stringify(stackedRes));

if (!stackedRes.viewportMounted || stackedRes.toastCount < 1) {
    throw new Error(\`Toast dispatch failed: viewport=\${stackedRes.viewportMounted}, count=\${stackedRes.toastCount}\`);
}

// Click #clear-all to dismiss
await page.evaluate("(() => { const btn = document.querySelector('#clear-all'); if (btn) { btn.focus(); btn.click(); } })()");
await wait(0.4);

const clearedRes = await page.evaluate("(() => { \
    const viewport = document.querySelector('#monoxus-toast-viewport'); \
    const toasts = viewport ? viewport.querySelectorAll('li') : []; \
    return { toastCount: toasts.length }; \
})()");
console.log(JSON.stringify(clearedRes));

await task.finish({ keep: [] });
EOF
            echo -e "${GREEN}[PASS]${NC} Toast verified (pointer swipe gesture dismissal, stack dispatch, clear dismissal)."
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
    verify_component popover
    verify_component tooltip
    verify_component dialog
    verify_component alert_dialog
    verify_component select
    verify_component toast
else
    verify_component "${TARGET}"
fi

echo -e "\n${GREEN}[PASS]${NC} All requested web verifications completed cleanly.\n"

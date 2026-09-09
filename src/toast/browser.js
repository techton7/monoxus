if (typeof window.__monoxus_toast_cleanup === "function") {
    try { window.__monoxus_toast_cleanup(); } catch (_) {}
}

let init = await dioxus.recv();
let cfg = {};
try {
    cfg = JSON.parse(init);
} catch (_) {
    let parts = (init || "").split("\n");
    cfg = {
        hotkey: parts[0] || "F8",
        viewport_id: parts[1] || "monoxus-toast-viewport",
        gap: 14,
        swipe_threshold: 45,
        position: "bottom-right"
    };
}

let hotkey = cfg.hotkey || "F8";
let viewportId = cfg.viewport_id || "monoxus-toast-viewport";
let gap = typeof cfg.gap === "number" ? cfg.gap : 14;
let swipeThreshold = typeof cfg.swipe_threshold === "number" ? cfg.swipe_threshold : 45;
let position = cfg.position || "bottom-right";
let dir = cfg.dir || "auto";
let swipeDirections = Array.isArray(cfg.swipe_directions) && cfg.swipe_directions.length > 0
    ? cfg.swipe_directions
    : (position.startsWith("top") ? ["top", "right"] : ["bottom", "right"]);
let offset = cfg.offset || {};
let mobileOffset = cfg.mobile_offset || {};

const getResolvedDir = () => {
    if (dir !== "auto") return dir;
    const docDir = document.documentElement.getAttribute("dir");
    if (docDir && docDir !== "auto") return docDir;
    return window.getComputedStyle(document.documentElement).direction || "ltr";
};

const applyViewportStyles = (viewport) => {
    if (!viewport) return;
    const resolvedDir = getResolvedDir();
    if (!viewport.getAttribute("dir")) {
        viewport.setAttribute("dir", resolvedDir);
    }
    const defaultOffset = { top: "32px", right: "32px", bottom: "32px", left: "32px" };
    const defaultMobileOffset = { top: "16px", right: "16px", bottom: "16px", left: "16px" };

    ["top", "right", "bottom", "left"].forEach((key) => {
        const offVal = offset[key] || defaultOffset[key];
        viewport.style.setProperty(`--offset-${key}`, offVal);
        const mobVal = mobileOffset[key] || defaultMobileOffset[key];
        viewport.style.setProperty(`--mobile-offset-${key}`, mobVal);
    });
};

const handleVisibilityChange = () => {
    dioxus.send(`visibility:${document.visibilityState}`);
};

const handleKeyDown = (event) => {
    if (event.defaultPrevented) return;
    if (event.key === hotkey) {
        event.preventDefault();
        const el = document.getElementById(viewportId);
        if (el) {
            el.focus();
        }
        dioxus.send(`hotkey:${event.key}`);
    }
};

document.addEventListener("visibilitychange", handleVisibilityChange);
document.addEventListener("keydown", handleKeyDown);

// --- Stacking Geometry & Gestures Engine ---
const observedElements = new Set();
const resizeObserver = new ResizeObserver(() => {
    updateStackGeometry();
});

const updateStackGeometry = () => {
    const viewport = document.getElementById(viewportId);
    if (!viewport) return;

    const toasts = Array.from(
        viewport.querySelectorAll('[data-sonner-toast], li[role="status"], li[role="alert"]')
    );
    if (toasts.length === 0) {
        viewport.style.removeProperty("--front-toast-height");
        return;
    }

    const frontToast = toasts[0];
    const frontRect = frontToast.getBoundingClientRect();
    const frontH = Math.round(frontRect.height);
    viewport.style.setProperty("--front-toast-height", `${frontH}px`);

    let accumulatedH = 0;
    for (let i = 0; i < toasts.length; i++) {
        const toast = toasts[i];
        const rect = toast.getBoundingClientRect();
        const h = Math.round(rect.height);
        const offset = accumulatedH + (i * gap);
        const scale = Number((Math.max(0.7, 1.0 - (i * 0.05))).toFixed(3));

        toast.style.setProperty("--height", `${h}px`);
        toast.style.setProperty("--offset", `${offset}px`);
        toast.style.setProperty("--scale", `${scale}`);
        toast.style.setProperty("--index", `${i}`);
        toast.style.setProperty("--toasts-before", `${i}`);
        toast.style.setProperty("--z-index", `${toasts.length - i}`);
        accumulatedH += h;
    }
};

const syncObservedToasts = () => {
    const viewport = document.getElementById(viewportId);
    if (!viewport) return;
    const toasts = Array.from(
        viewport.querySelectorAll('[data-sonner-toast], li[role="status"], li[role="alert"]')
    );
    const currentSet = new Set(toasts);
    for (const el of observedElements) {
        if (!currentSet.has(el)) {
            resizeObserver.unobserve(el);
            observedElements.delete(el);
        }
    }
    for (const el of toasts) {
        if (!observedElements.has(el)) {
            resizeObserver.observe(el);
            observedElements.add(el);
        }
    }
};

let isHovered = false;

const handleMouseEnter = () => {
    if (!isHovered) {
        isHovered = true;
        dioxus.send("hover:enter");
    }
};

const handleMouseMove = () => {
    if (!isHovered) {
        isHovered = true;
        dioxus.send("hover:enter");
    }
};

const handleMouseLeave = (e) => {
    if (e && e.relatedTarget && currentBoundViewport && currentBoundViewport.contains(e.relatedTarget)) {
        return;
    }
    if (isHovered) {
        isHovered = false;
        dioxus.send("hover:leave");
    }
};

const handleFocusIn = () => dioxus.send("focus:enter");
const handleFocusOut = () => dioxus.send("focus:leave");

let dragToast = null;
let dragStart = null;
let dragStartTime = 0;
let swipeDirection = null; // 'x' or 'y' or null
let isDragging = false;

const getDampening = (delta) => {
    const factor = Math.abs(delta) / 20;
    return 1 / (1.5 + factor);
};

const handlePointerDown = (e) => {
    if (e.button === 2) return;
    const toast = e.target.closest('[data-sonner-toast], li[role="status"], li[role="alert"]');
    if (!toast) return;
    if (e.target.tagName === "BUTTON" || e.target.closest("button")) return;

    dragToast = toast;
    dragStart = { x: e.clientX, y: e.clientY };
    dragStartTime = Date.now();
    swipeDirection = null;
    isDragging = false;
    toast.removeAttribute("data-swipe-out");
    toast.removeAttribute("data-swipe-direction");
    toast.removeAttribute("data-swiped");
    toast.removeAttribute("data-swiping");
    try {
        toast.setPointerCapture(e.pointerId);
    } catch (_) {}
};

const getEffectiveSwipeDirections = (toast) => {
    if (toast && toast.getAttribute("data-swipe-directions")) {
        return toast.getAttribute("data-swipe-directions").split(",").map((s) => s.trim());
    }
    const toastPos =
        (toast && toast.getAttribute("data-position")) ||
        (currentBoundViewport && currentBoundViewport.getAttribute("data-position")) ||
        position;
    const [y, x] = toastPos.split("-");
    const dirs = [];
    if (y) dirs.push(y);
    if (x && x !== "center") dirs.push(x);
    return dirs.length > 0 ? dirs : swipeDirections;
};

const handlePointerMove = (e) => {
    if (!dragToast || !dragStart) return;
    if (!isDragging && window.getSelection()?.toString().length > 0) return;

    const xDelta = e.clientX - dragStart.x;
    const yDelta = e.clientY - dragStart.y;

    if (!swipeDirection && (Math.abs(xDelta) > 1 || Math.abs(yDelta) > 1)) {
        swipeDirection = Math.abs(xDelta) > Math.abs(yDelta) ? 'x' : 'y';
    }

    let swipeAmount = { x: 0, y: 0 };
    const activeSwipeDirs = getEffectiveSwipeDirections(dragToast);

    if (swipeDirection === 'y') {
        const isAllowed = (activeSwipeDirs.includes('top') && yDelta < 0) || (activeSwipeDirs.includes('bottom') && yDelta > 0);
        if (isAllowed) {
            swipeAmount.y = yDelta;
        } else {
            const dampenedDelta = yDelta * getDampening(yDelta);
            swipeAmount.y = Math.abs(dampenedDelta) < Math.abs(yDelta) ? dampenedDelta : yDelta;
        }
    } else if (swipeDirection === 'x') {
        const isAllowed = (activeSwipeDirs.includes('left') && xDelta < 0) || (activeSwipeDirs.includes('right') && xDelta > 0);
        if (isAllowed) {
            swipeAmount.x = xDelta;
        } else {
            const dampenedDelta = xDelta * getDampening(xDelta);
            swipeAmount.x = Math.abs(dampenedDelta) < Math.abs(xDelta) ? dampenedDelta : xDelta;
        }
    }

    if (Math.abs(swipeAmount.x) > 0 || Math.abs(swipeAmount.y) > 0) {
        if (!isDragging) {
            isDragging = true;
            dragToast.setAttribute("data-swiping", "true");
            dragToast.setAttribute("data-swiped", "true");
            try {
                window.getSelection()?.removeAllRanges();
            } catch (_) {}
            dioxus.send("swipe:start");
        }
        dragToast.style.setProperty("--swipe-amount-x", `${swipeAmount.x}px`);
        dragToast.style.setProperty("--swipe-amount-y", `${swipeAmount.y}px`);
        dragToast.style.setProperty("--drag-offset", `${swipeDirection === 'x' ? swipeAmount.x : swipeAmount.y}px`);
    }
};

const handlePointerEnd = (e) => {
    if (!dragToast) return;
    const toast = dragToast;
    const elapsed = Date.now() - dragStartTime;

    const swipeAmountX = Number(
        toast.style.getPropertyValue('--swipe-amount-x').replace('px', '') || 0
    );
    const swipeAmountY = Number(
        toast.style.getPropertyValue('--swipe-amount-y').replace('px', '') || 0
    );

    const activeSwipeDirection = swipeDirection;
    dragToast = null;
    dragStart = null;
    swipeDirection = null;

    toast.removeAttribute("data-swiping");
    toast.removeAttribute("data-swiped");

    if (isDragging) {
        isDragging = false;
        dioxus.send("swipe:end");

        const activeSwipeDirs = getEffectiveSwipeDirections(toast);
        const swipeAmount = activeSwipeDirection === 'x' ? swipeAmountX : swipeAmountY;
        const velocity = Math.abs(swipeAmount) / (elapsed || 1);

        const isAllowedDirection =
            activeSwipeDirection === 'x'
                ? activeSwipeDirs.includes(swipeAmountX > 0 ? 'right' : 'left')
                : activeSwipeDirs.includes(swipeAmountY > 0 ? 'bottom' : 'top');

        if (isAllowedDirection && (Math.abs(swipeAmount) >= swipeThreshold || velocity > 0.11)) {
            const idStr = toast.getAttribute("data-id") || toast.getAttribute("id") || "";
            const match = idStr.match(/\d+/);
            const toastId = match ? match[0] : "0";

            const swipeOutDir =
                activeSwipeDirection === 'x'
                    ? (swipeAmountX > 0 ? 'right' : 'left')
                    : (swipeAmountY > 0 ? 'down' : 'up');

            toast.setAttribute("data-swipe-out", "true");
            toast.setAttribute("data-swipe-direction", swipeOutDir);

            dioxus.send(`swipe:dismiss:${toastId}`);
        } else {
            toast.style.setProperty("--swipe-amount-x", "0px");
            toast.style.setProperty("--swipe-amount-y", "0px");
            toast.style.setProperty("--drag-offset", "0px");
            toast.removeAttribute("data-swipe-out");
            toast.removeAttribute("data-swipe-direction");
        }
    }
};

window.addEventListener("pointermove", handlePointerMove);
window.addEventListener("pointerup", handlePointerEnd);
window.addEventListener("pointercancel", handlePointerEnd);

let viewportObserver = null;
let currentBoundViewport = null;

const bindViewport = () => {
    const viewport = document.getElementById(viewportId);
    if (!viewport) return false;
    if (currentBoundViewport === viewport) return true;

    if (currentBoundViewport && viewportObserver) {
        viewportObserver.disconnect();
        currentBoundViewport.removeEventListener("mouseenter", handleMouseEnter);
        currentBoundViewport.removeEventListener("mouseleave", handleMouseLeave);
        currentBoundViewport.removeEventListener("focusin", handleFocusIn);
        currentBoundViewport.removeEventListener("focusout", handleFocusOut);
        currentBoundViewport.removeEventListener("pointerdown", handlePointerDown);
    }

    currentBoundViewport = viewport;
    viewportObserver = new MutationObserver(() => {
        syncObservedToasts();
        updateStackGeometry();
    });
    viewportObserver.observe(viewport, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["data-expanded"]
    });

    viewport.addEventListener("mouseenter", handleMouseEnter);
    viewport.addEventListener("mousemove", handleMouseMove);
    viewport.addEventListener("mouseleave", handleMouseLeave);
    viewport.addEventListener("focusin", handleFocusIn);
    viewport.addEventListener("focusout", handleFocusOut);
    viewport.addEventListener("pointerdown", handlePointerDown);

    applyViewportStyles(viewport);
    syncObservedToasts();
    updateStackGeometry();
    return true;
};

bindViewport();

const bodyObserver = new MutationObserver(() => {
    bindViewport();
});
bodyObserver.observe(document.body, { childList: true, subtree: true });

const cleanup = () => {
    document.removeEventListener("visibilitychange", handleVisibilityChange);
    document.removeEventListener("keydown", handleKeyDown);
    window.removeEventListener("pointermove", handlePointerMove);
    window.removeEventListener("pointerup", handlePointerEnd);
    window.removeEventListener("pointercancel", handlePointerEnd);

    bodyObserver.disconnect();
    if (viewportObserver) viewportObserver.disconnect();
    resizeObserver.disconnect();

    if (currentBoundViewport) {
        currentBoundViewport.removeEventListener("mouseenter", handleMouseEnter);
        currentBoundViewport.removeEventListener("mousemove", handleMouseMove);
        currentBoundViewport.removeEventListener("mouseleave", handleMouseLeave);
        currentBoundViewport.removeEventListener("focusin", handleFocusIn);
        currentBoundViewport.removeEventListener("focusout", handleFocusOut);
        currentBoundViewport.removeEventListener("pointerdown", handlePointerDown);
    }
    window.__monoxus_toast_cleanup = null;
};
window.__monoxus_toast_cleanup = cleanup;

while (true) {
    const cmd = await dioxus.recv();
    if (cmd === "stop") {
        break;
    }
}

cleanup();
dioxus.send("stopped");


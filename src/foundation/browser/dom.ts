// DOM and Viewport helper bridge for monoxus primitives

/**
 * Focus element by ID with options
 */
export function focusElementByIdWithOptions(targetId: string, preventScroll: boolean): void {
    const tryFocus = () => {
        const target = document.getElementById(targetId);
        if (target instanceof HTMLElement) {
            target.focus({ preventScroll });
            return true;
        }
        return false;
    };
    if (!tryFocus() && typeof window !== "undefined" && window.requestAnimationFrame) {
        window.requestAnimationFrame(() => {
            tryFocus();
        });
    }
}

/**
 * Focus first focusable element inside container
 */
export function focusFirstFocusable(contentId: string, selector: string): void {
    const root = document.getElementById(contentId);
    if (!(root instanceof HTMLElement)) {
        return;
    }

    const candidate = root.matches(selector) ? root : root.querySelector(selector);
    if (candidate instanceof HTMLElement) {
        if (candidate === root && !root.hasAttribute("tabindex")) {
            root.setAttribute("tabindex", "-1");
        }
        candidate.focus();
        return;
    }

    if (!root.hasAttribute("tabindex")) {
        root.setAttribute("tabindex", "-1");
    }
    root.focus();
}

/**
 * Get viewport width and height
 */
export function getViewportSize(): [number, number] {
    return [window.innerWidth, window.innerHeight];
}

/**
 * Check if target element matches activeElement
 */
export function isElementActive(targetId: string): boolean {
    const target = document.getElementById(targetId);
    return target instanceof HTMLElement && document.activeElement === target;
}

/**
 * Check if reference element is hidden outside viewport
 */
export function isReferenceHidden(anchorIds: string[]): boolean | null {
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    for (const id of anchorIds) {
        const element = document.getElementById(id);
        if (!(element instanceof HTMLElement)) {
            continue;
        }

        const rect = element.getBoundingClientRect();
        return rect.width <= 0 ||
            rect.height <= 0 ||
            rect.right <= 0 ||
            rect.bottom <= 0 ||
            rect.left >= viewportWidth ||
            rect.top >= viewportHeight;
    }

    return null;
}

/**
 * Acquire modal scroll lock
 */
export function acquireScrollLock(lockId: string): void {
    const state = ((window as any).__monoxusScrollLockRuntime ??= ((window as any).__monoxusDialogRuntime ??= {
        scrollLocks: new Set(),
        previousBodyOverflow: null,
        previousBodyPaddingRight: null,
        previousDocumentOverflow: null,
        restoreToken: 0,
    }));
    (window as any).__monoxusDialogRuntime = state;

    state.restoreToken += 1;
    if (state.scrollLocks.has(lockId)) {
        return;
    }

    if (state.scrollLocks.size === 0) {
        const body = document.body;
        const documentElement = document.documentElement;
        const computedBodyStyle = window.getComputedStyle(body);
        const bodyPaddingRight = Number.parseFloat(computedBodyStyle.paddingRight || "0") || 0;
        const scrollbarWidth = Math.max(0, window.innerWidth - documentElement.clientWidth);

        state.previousBodyOverflow = body.style.overflow;
        state.previousBodyPaddingRight = body.style.paddingRight;
        state.previousDocumentOverflow = documentElement.style.overflow;

        body.style.overflow = "hidden";
        documentElement.style.overflow = "hidden";

        if (scrollbarWidth > 0) {
            body.style.paddingRight = `${bodyPaddingRight + scrollbarWidth}px`;
        }
    }

    state.scrollLocks.add(lockId);
}

/**
 * Release modal scroll lock
 */
export function releaseScrollLock(lockId: string, delayMs: number): void {
    const state = (window as any).__monoxusScrollLockRuntime ?? (window as any).__monoxusDialogRuntime;
    if (!state || !(state.scrollLocks instanceof Set)) {
        return;
    }

    state.scrollLocks.delete(lockId);
    const restoreToken = ++state.restoreToken;
    if (state.scrollLocks.size > 0) {
        return;
    }

    const restore = () => {
        const currentState = (window as any).__monoxusScrollLockRuntime ?? (window as any).__monoxusDialogRuntime;
        if (!currentState || currentState.restoreToken !== restoreToken) {
            return;
        }

        if (currentState.scrollLocks instanceof Set && currentState.scrollLocks.size > 0) {
            return;
        }

        document.body.style.overflow = currentState.previousBodyOverflow ?? "";
        document.body.style.paddingRight = currentState.previousBodyPaddingRight ?? "";
        document.documentElement.style.overflow = currentState.previousDocumentOverflow ?? "";
    };

    if (delayMs > 0) {
        window.setTimeout(restore, delayMs);
        return;
    }

    restore();
}

/**
 * Scroll element into view nearest
 */
export function scrollElementIntoViewNearest(elementId: string): void {
    const el = document.getElementById(elementId);
    if (!el) return;
    el.scrollIntoView({ block: "nearest", inline: "nearest" });
}

/**
 * Set body user-select
 */
export function setBodyUserSelect(none: boolean): void {
    if (typeof document === "undefined" || !document.body) return;
    document.body.style.userSelect = none ? "none" : "";
    (document.body.style as any).webkitUserSelect = none ? "none" : "";
}

/**
 * Teleport element to host container
 */
export function teleportElementToHost(elementId: string, hostId: string | null): void {
    if (typeof document === "undefined") return;
    const el = document.getElementById(elementId);
    if (!el) return;
    let target = hostId ? document.getElementById(hostId) : null;
    if (!target) {
        target = document.getElementById("portal-root") || document.body;
    }
    if (target && el.parentElement !== target) {
        target.appendChild(el);
    }
}

/**
 * Remove element by ID
 */
export function removeElementById(elementId: string): void {
    if (typeof document === "undefined") return;
    const el = document.getElementById(elementId);
    if (el && el.parentElement) {
        el.remove();
    }
}

/**
 * Measure floating placement coordinates
 */
export function measureFloatingPlacement(
    anchorId: string,
    contentId: string,
    customAnchorId: string | null,
    boundaryId: string | null
): [number, number, number, number, number, number, number, number, number, number] | null {
    if (typeof document === "undefined") return null;
    const trigger = (customAnchorId ? document.getElementById(customAnchorId) : null) || document.getElementById(anchorId);
    const content = document.getElementById(contentId);
    if (!trigger || !content) return null;
    const tr = trigger.getBoundingClientRect();
    const cr = content.getBoundingClientRect();
    let bLeft = 0;
    let bTop = 0;
    let bRight = window.innerWidth;
    let bBottom = window.innerHeight;
    if (boundaryId) {
        const bEl = document.getElementById(boundaryId);
        if (bEl) {
            const br = bEl.getBoundingClientRect();
            bLeft = br.left;
            bTop = br.top;
            bRight = br.right;
            bBottom = br.bottom;
        }
    }
    return [tr.left, tr.top, tr.width, tr.height, cr.width, cr.height, bLeft, bTop, bRight, bBottom];
}

/**
 * Watch form reset events for an element (Watcher)
 * #[watcher]
 */
export function watchFormReset(
    elementId: string,
    formId: string | null,
    emit: (signal: string) => void
): () => void {
    const el = document.getElementById(elementId);
    const form = formId ? document.getElementById(formId) : (el ? el.closest("form") : null);

    const handleReset = () => {
        emit("reset");
    };

    if (form) {
        form.addEventListener("reset", handleReset);
    }

    return () => {
        if (form) {
            form.removeEventListener("reset", handleReset);
        }
    };
}

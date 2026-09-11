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

export interface FormResetEventPayload {
    kind: "reset";
}

/**
 * Watch form reset events for an element (Watcher)
 * #[watcher]
 */
export function watchFormReset(
    elementId: string,
    formId: string | null,
    emit: (event: FormResetEventPayload) => void
): () => void {
    const el = document.getElementById(elementId);
    const form = formId ? document.getElementById(formId) : (el ? el.closest("form") : null);

    const handleReset = () => {
        emit({ kind: "reset" });
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

export type DocumentDismissEventPayload =
    | { kind: "pointer_down"; pathIds: string[] }
    | { kind: "focus_in"; pathIds: string[] }
    | { kind: "escape" };

/**
 * Watch document dismiss events (pointerdown outside, focusin outside, escape keydown)
 * #[watcher]
 */
export function watchDocumentDismiss(
    emit: (event: DocumentDismissEventPayload) => void
): () => void {
    const readPathIds = (event: Event): string[] => {
        if (!event || typeof event.composedPath !== "function") {
            return [];
        }
        return event
            .composedPath()
            .filter((node): node is HTMLElement => node instanceof HTMLElement && typeof node.id === "string" && node.id.length > 0)
            .map((node) => node.id);
    };

    const handlePointerDown = (event: PointerEvent) => {
        emit({ kind: "pointer_down", pathIds: readPathIds(event) });
    };
    const handleFocusIn = (event: FocusEvent) => {
        emit({ kind: "focus_in", pathIds: readPathIds(event) });
    };
    const handleKeyDown = (event: KeyboardEvent) => {
        if (event.defaultPrevented) {
            return;
        }
        if (event.key === "Escape") {
            emit({ kind: "escape" });
        }
    };

    document.addEventListener("pointerdown", handlePointerDown, true);
    document.addEventListener("focusin", handleFocusIn, true);
    document.addEventListener("keydown", handleKeyDown, false);

    return () => {
        document.removeEventListener("pointerdown", handlePointerDown, true);
        document.removeEventListener("focusin", handleFocusIn, true);
        document.removeEventListener("keydown", handleKeyDown, false);
    };
}

export type FloatingAutoUpdatePayload =
    | { kind: "scroll" }
    | { kind: "update" };

/**
 * Watch floating element and anchor element updates (scroll, resize, mutation)
 * #[watcher]
 */
export function watchFloatingAutoUpdate(
    anchorIds: string[],
    contentId: string,
    emit: (event: FloatingAutoUpdatePayload) => void
): () => void {
    const mutationTarget = document.body ?? document.documentElement;
    const visualViewport = window.visualViewport ?? null;
    const ResizeObserverCtor = window.ResizeObserver ?? null;
    const MutationObserverCtor = window.MutationObserver ?? null;
    let stopped = false;
    let framePending = false;
    let scrollTriggered = false;
    let resizeObserver: ResizeObserver | null = null;
    let mutationObserver: MutationObserver | null = null;
    let currentAnchor: HTMLElement | null = null;
    let currentContent: HTMLElement | null = null;

    const readElement = (id: string): HTMLElement | null => {
        const element = document.getElementById(id);
        return element instanceof HTMLElement ? element : null;
    };

    const resolveAnchor = (): HTMLElement | null => {
        for (const id of anchorIds) {
            const anchor = readElement(id);
            if (anchor) {
                return anchor;
            }
        }
        return null;
    };

    const reconnectObservedElements = () => {
        const nextAnchor = resolveAnchor();
        const nextContent = readElement(contentId);

        if (ResizeObserverCtor === null) {
            currentAnchor = nextAnchor;
            currentContent = nextContent;
            return;
        }

        if (resizeObserver === null) {
            resizeObserver = new ResizeObserverCtor(() => queueUpdate());
        }

        if (currentAnchor === nextAnchor && currentContent === nextContent) {
            return;
        }

        resizeObserver.disconnect();
        resizeObserver.observe(document.documentElement);

        currentAnchor = nextAnchor;
        currentContent = nextContent;

        if (currentAnchor) {
            resizeObserver.observe(currentAnchor);
        }

        if (currentContent) {
            resizeObserver.observe(currentContent);
        }
    };

    const queueUpdate = ({ fromScroll = false } = {}) => {
        if (stopped || framePending) {
            return;
        }

        scrollTriggered = scrollTriggered || fromScroll;
        framePending = true;
        window.requestAnimationFrame(() => {
            framePending = false;
            if (stopped) {
                return;
            }

            const event: FloatingAutoUpdatePayload = {
                kind: scrollTriggered ? "scroll" : "update",
            };
            scrollTriggered = false;
            emit(event);
        });
    };

    const handleScroll = () => queueUpdate({ fromScroll: true });
    const handleResize = () => queueUpdate();

    window.addEventListener("scroll", handleScroll, { capture: true, passive: true });
    window.addEventListener("resize", handleResize, { passive: true });

    if (visualViewport) {
        visualViewport.addEventListener("scroll", handleScroll, { passive: true });
        visualViewport.addEventListener("resize", handleResize, { passive: true });
    }

    if (MutationObserverCtor) {
        mutationObserver = new MutationObserverCtor(() => {
            reconnectObservedElements();
            queueUpdate();
        });
    }

    if (mutationTarget && mutationObserver) {
        mutationObserver.observe(mutationTarget, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: ["class", "style", "hidden"],
        });
    }

    reconnectObservedElements();

    return () => {
        stopped = true;
        window.removeEventListener("scroll", handleScroll, true);
        window.removeEventListener("resize", handleResize);

        if (visualViewport) {
            visualViewport.removeEventListener("scroll", handleScroll);
            visualViewport.removeEventListener("resize", handleResize);
        }

        mutationObserver?.disconnect();
        resizeObserver?.disconnect();
    };
}

export type PresenceEventPayload =
    | { kind: "fallback"; cycleId: number; reason: "missing" | "hidden" | "no_animation" }
    | { kind: "animation_end"; cycleId: number; animationName: string }
    | { kind: "animation_cancel"; cycleId: number; animationName: string }
    | { kind: "stopped"; cycleId: number };

/**
 * Watch element presence lifecycle during unmount/close animations
 * #[watcher]
 */
export function watchPresence(
    rootId: string,
    cycleId: number,
    emit: (event: PresenceEventPayload) => void
): () => void {
    const MutationObserverCtor = window.MutationObserver ?? null;
    const root = document.getElementById(rootId);
    let activeAnimationNames: string[] = [];
    let finished = false;
    let mutationObserver: MutationObserver | null = null;

    const parseAnimationNames = (value: string | null): string[] =>
        String(value ?? "")
            .split(",")
            .map((name) => name.trim())
            .filter((name) => name.length > 0 && name !== "none");

    const cleanup = () => {
        mutationObserver?.disconnect();
        if (!(root instanceof HTMLElement)) {
            return;
        }
        root.removeEventListener("animationend", handleAnimationEnd);
        root.removeEventListener("animationcancel", handleAnimationCancel);
    };

    const finish = (event: PresenceEventPayload) => {
        if (finished) {
            return;
        }
        finished = true;
        cleanup();
        emit(event);
    };

    const refreshRootState = () => {
        if (!(root instanceof HTMLElement) || !root.isConnected) {
            finish({ kind: "fallback", cycleId, reason: "missing" });
            return;
        }

        const computed = window.getComputedStyle(root);
        if (root.hidden || computed.display === "none") {
            finish({ kind: "fallback", cycleId, reason: "hidden" });
            return;
        }

        activeAnimationNames = parseAnimationNames(computed.animationName);
        if (activeAnimationNames.length === 0) {
            finish({ kind: "fallback", cycleId, reason: "no_animation" });
        }
    };

    const handleAnimation = (event: AnimationEvent, kind: "animation_end" | "animation_cancel") => {
        if (finished || event.target !== root) {
            return;
        }
        if (
            typeof event.animationName !== "string" ||
            !activeAnimationNames.includes(event.animationName)
        ) {
            return;
        }
        finish({ kind, cycleId, animationName: event.animationName });
    };

    const handleAnimationEnd = (event: AnimationEvent) => handleAnimation(event, "animation_end");
    const handleAnimationCancel = (event: AnimationEvent) => handleAnimation(event, "animation_cancel");

    if (root instanceof HTMLElement) {
        root.addEventListener("animationend", handleAnimationEnd);
        root.addEventListener("animationcancel", handleAnimationCancel);
    }

    if (MutationObserverCtor !== null) {
        mutationObserver = new MutationObserverCtor(() => {
            refreshRootState();
        });
        const target = document.body ?? document.documentElement;
        if (target) {
            mutationObserver.observe(target, {
                childList: true,
                subtree: true,
                attributes: true,
                attributeFilter: ["class", "style", "hidden"],
            });
        }
    }

    window.requestAnimationFrame(() => {
        refreshRootState();
    });

    return () => {
        if (!finished) {
            finish({ kind: "stopped", cycleId });
        }
        cleanup();
    };
}


// monoxus UI Browser Foundation Bridge
// Colocated DOM helpers, scroll lock, focus roving, portals, and overlay event watchers

export type PresenceEventPayload =
    | { kind: "fallback"; cycleId: number; reason: "missing" | "hidden" | "no_animation" }
    | { kind: "animation_end"; cycleId: number; animationName: string }
    | { kind: "animation_cancel"; cycleId: number; animationName: string }
    | { kind: "stopped"; cycleId: number };

export type DocumentDismissEventPayload =
    | { kind: "pointer_down"; pathIds: string[] }
    | { kind: "focus_in"; pathIds: string[] }
    | { kind: "escape" };

export type DocumentDismissPayload = DocumentDismissEventPayload;

export type FloatingAutoUpdatePayload =
    | { kind: "scroll" }
    | { kind: "update" };

export interface FormResetEventPayload {
    kind: "reset";
}

export type FormResetPayload = FormResetEventPayload;

function readPathIds(event: Event): string[] {
    if (!event || typeof event.composedPath !== "function") {
        return [];
    }
    return event
        .composedPath()
        .filter((node): node is HTMLElement => node instanceof HTMLElement && typeof node.id === "string" && node.id.length > 0)
        .map((node) => (node as HTMLElement).id);
}

function isInside(event: Event, boundaryIds: string[]): boolean {
    if (!event || typeof event.composedPath !== "function" || !boundaryIds || boundaryIds.length === 0) {
        return false;
    }
    const path = event.composedPath();
    const idSet = new Set(boundaryIds.filter((id) => typeof id === "string" && id.length > 0));
    if (idSet.size === 0) {
        return false;
    }

    for (const node of path) {
        if (node instanceof HTMLElement) {
            if (node.id && idSet.has(node.id)) {
                return true;
            }
        }
    }
    return false;
}

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

/**
 * Unified document dismiss watcher for overlays (pointerdown outside, focusin outside, Escape key)
 * Suppresses inside clicks with zero IPC emission.
 * #[watcher]
 */
export function watchDocumentDismiss(
    boundaryIds: string[],
    emit: (event: DocumentDismissEventPayload) => void
): () => void {
    const handlePointerDown = (event: PointerEvent) => {
        if (isInside(event, boundaryIds)) {
            return; // Suppress inside clicks
        }
        emit({ kind: "pointer_down", pathIds: readPathIds(event) });
    };

    const handleFocusIn = (event: FocusEvent) => {
        if (isInside(event, boundaryIds)) {
            return; // Suppress inside focus
        }
        emit({ kind: "focus_in", pathIds: readPathIds(event) });
    };

    const handleKeyDown = (event: KeyboardEvent) => {
        if (event.defaultPrevented) {
            return;
        }
        if (event.key === "Escape" || event.key === "Esc") {
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

/**
 * Watch floating element and anchor element updates (scroll, resize, mutation)
 * #[watcher]
 */
export function watchFloatingAutoUpdate(
    anchorIds: string[],
    contentId: string,
    emit: (event: FloatingAutoUpdatePayload) => void
): () => void {
    let stopped = false;
    let scrollTriggered = false;
    let rafId: number | null = null;

    const readElement = (id: string): HTMLElement | null => {
        const el = document.getElementById(id);
        return el instanceof HTMLElement ? el : null;
    };

    const resolveAnchor = (): HTMLElement | null => {
        for (const id of anchorIds) {
            const el = readElement(id);
            if (el) {
                return el;
            }
        }
        return null;
    };

    const queueUpdate = (fromScroll: boolean = false) => {
        if (stopped) {
            return;
        }
        scrollTriggered = scrollTriggered || fromScroll;
        if (rafId === null && typeof window !== "undefined" && window.requestAnimationFrame) {
            rafId = window.requestAnimationFrame(() => {
                rafId = null;
                if (stopped) {
                    return;
                }
                const event: FloatingAutoUpdatePayload = {
                    kind: scrollTriggered ? "scroll" : "update",
                };
                scrollTriggered = false;
                emit(event);
            });
        }
    };

    const getScrollParents = (element: Element): (Element | Window)[] => {
        const parents: (Element | Window)[] = [];
        let current: Node | null = element.parentNode;
        while (current && current instanceof Element && current !== document.body && current !== document.documentElement) {
            const style = window.getComputedStyle(current);
            const overflow = `${style.overflow}${style.overflowY}${style.overflowX}`;
            if (/(auto|scroll|overlay)/.test(overflow)) {
                parents.push(current);
            }
            current = current.parentNode;
        }
        parents.push(window);
        if (window.visualViewport) {
            parents.push(window.visualViewport as any);
        }
        return parents;
    };

    let resizeObserver: ResizeObserver | null = null;
    if (typeof window !== "undefined" && window.ResizeObserver) {
        resizeObserver = new window.ResizeObserver(() => {
            queueUpdate(false);
        });
    }

    let scrollCleanups: (() => void)[] = [];
    let currentAnchor: HTMLElement | null = null;
    let currentContent: HTMLElement | null = null;

    const reconnect = () => {
        if (stopped) {
            return;
        }
        const nextAnchor = resolveAnchor();
        const nextContent = readElement(contentId);

        if (currentAnchor === nextAnchor && currentContent === nextContent) {
            return;
        }

        for (const c of scrollCleanups) {
            c();
        }
        scrollCleanups = [];

        if (resizeObserver) {
            resizeObserver.disconnect();
        }

        currentAnchor = nextAnchor;
        currentContent = nextContent;

        if (resizeObserver) {
            if (document.documentElement) {
                resizeObserver.observe(document.documentElement);
            }
            if (currentAnchor) {
                resizeObserver.observe(currentAnchor);
            }
            if (currentContent) {
                resizeObserver.observe(currentContent);
            }
        }

        const attachScroll = (el: Element) => {
            const parents = getScrollParents(el);
            const handler = () => queueUpdate(true);
            for (const p of parents) {
                p.addEventListener("scroll", handler, { passive: true, capture: true });
                scrollCleanups.push(() => p.removeEventListener("scroll", handler, true));
            }
        };

        if (currentAnchor) {
            attachScroll(currentAnchor);
        }
        if (currentContent) {
            attachScroll(currentContent);
        }
    };

    const handleWindowResize = () => queueUpdate(false);
    window.addEventListener("resize", handleWindowResize, { passive: true });

    let mutationObserver: MutationObserver | null = null;
    if (typeof window !== "undefined" && window.MutationObserver) {
        mutationObserver = new MutationObserver(() => {
            reconnect();
            queueUpdate(false);
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

    reconnect();

    return () => {
        stopped = true;
        if (rafId !== null) {
            window.cancelAnimationFrame(rafId);
            rafId = null;
        }
        window.removeEventListener("resize", handleWindowResize);
        if (mutationObserver) {
            mutationObserver.disconnect();
            mutationObserver = null;
        }
        if (resizeObserver) {
            resizeObserver.disconnect();
            resizeObserver = null;
        }
        for (const c of scrollCleanups) {
            c();
        }
        scrollCleanups = [];
    };
}

/**
 * Watch form reset events on an element's parent form
 * #[watcher]
 */
export function watchFormReset(
    elementId: string,
    formId: string,
    emit: (event: FormResetEventPayload) => void
): () => void {
    const el = document.getElementById(elementId);
    const form = formId ? document.getElementById(formId) : (el && "form" in el ? (el as any).form : null);
    if (!form || !(form instanceof HTMLFormElement)) {
        return () => {};
    }
    const handleReset = () => {
        emit({ kind: "reset" });
    };
    form.addEventListener("reset", handleReset);
    return () => {
        form.removeEventListener("reset", handleReset);
    };
}

// ---------------------------------------------------------------------------
// ScrollLock Subsystem (Body overflow & gutter padding compensation)
// ---------------------------------------------------------------------------

export class ScrollLock {
    private static locks = new Set<string>();
    private static originalOverflow: string = "";
    private static originalPaddingRight: string = "";
    private static cleanupGeneration: number = 0;
    private static pendingCleanupTimeout: any = null;

    private static cancelPendingCleanup(): void {
        if (ScrollLock.pendingCleanupTimeout !== null) {
            clearTimeout(ScrollLock.pendingCleanupTimeout);
            ScrollLock.pendingCleanupTimeout = null;
        }
        ScrollLock.cleanupGeneration++;
    }

    static acquire(lockId: string): void {
        if (typeof document === "undefined") {
            return;
        }

        ScrollLock.cancelPendingCleanup();

        if (ScrollLock.locks.size === 0) {
            ScrollLock.originalOverflow = document.body.style.overflow;
            ScrollLock.originalPaddingRight = document.body.style.paddingRight;

            const htmlStyle = getComputedStyle(document.documentElement);
            const bodyStyle = getComputedStyle(document.body);
            const hasStableGutter =
                (htmlStyle.scrollbarGutter || "").includes("stable") ||
                (bodyStyle.scrollbarGutter || "").includes("stable");

            const scrollbarWidth = window.innerWidth - document.documentElement.clientWidth;
            if (scrollbarWidth > 0 && !hasStableGutter) {
                document.body.style.paddingRight = `${scrollbarWidth}px`;
                document.body.style.setProperty("--scrollbar-width", `${scrollbarWidth}px`);
            }
            document.body.style.overflow = "hidden";
        }
        ScrollLock.locks.add(lockId);
    }

    static release(lockId: string, delayMs: number = 0): void {
        if (typeof document === "undefined") {
            return;
        }

        ScrollLock.locks.delete(lockId);

        if (ScrollLock.locks.size > 0) {
            return;
        }

        ScrollLock.cancelPendingCleanup();
        const currentToken = ScrollLock.cleanupGeneration;

        const doRelease = () => {
            ScrollLock.pendingCleanupTimeout = null;
            if (ScrollLock.cleanupGeneration !== currentToken || ScrollLock.locks.size > 0) {
                return;
            }
            document.body.style.overflow = ScrollLock.originalOverflow;
            document.body.style.paddingRight = ScrollLock.originalPaddingRight;
            document.body.style.removeProperty("--scrollbar-width");
        };

        if (delayMs > 0) {
            ScrollLock.pendingCleanupTimeout = setTimeout(doRelease, delayMs);
        } else {
            doRelease();
        }
    }

    static isLocked(): boolean {
        return ScrollLock.locks.size > 0;
    }

    static lockCount(): number {
        return ScrollLock.locks.size;
    }
}

export function acquireScrollLock(lockId: string): void {
    ScrollLock.acquire(lockId);
}

export function releaseScrollLock(lockId: string, delayMs: number): void {
    ScrollLock.release(lockId, delayMs);
}

// ---------------------------------------------------------------------------
// Focus & Selection & Portal DOM Helpers
// ---------------------------------------------------------------------------

export function focusElementByIdWithOptions(targetId: string, preventScroll: boolean): void {
    if (typeof document === "undefined") {
        return;
    }
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

export function focusFirstFocusable(contentId: string, selector: string): void {
    if (typeof document === "undefined") {
        return;
    }
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

export function scrollElementIntoViewNearest(elementId: string): void {
    if (typeof document === "undefined") {
        return;
    }
    const el = document.getElementById(elementId);
    if (el instanceof HTMLElement) {
        el.scrollIntoView({ block: "nearest", inline: "nearest" });
    }
}

export function setBodyUserSelect(none: boolean): void {
    if (typeof document === "undefined") {
        return;
    }
    if (none) {
        document.body.style.userSelect = "none";
        (document.body.style as any).webkitUserSelect = "none";
    } else {
        document.body.style.userSelect = "";
        (document.body.style as any).webkitUserSelect = "";
    }
}

export function teleportElementToHost(elementId: string, hostId: string): void {
    if (typeof document === "undefined") {
        return;
    }
    const el = document.getElementById(elementId);
    if (!el) {
        return;
    }
    const host = hostId ? document.getElementById(hostId) : document.body;
    if (host && el.parentNode !== host) {
        host.appendChild(el);
    }
}

export function removeElementById(elementId: string): void {
    if (typeof document === "undefined") {
        return;
    }
    const el = document.getElementById(elementId);
    if (el && el.parentNode) {
        el.parentNode.removeChild(el);
    }
}

export function isElementActive(targetId: string): boolean {
    if (typeof document === "undefined") {
        return false;
    }
    const target = document.getElementById(targetId);
    return target instanceof HTMLElement && document.activeElement === target;
}

export function isReferenceHidden(anchorIds: string[]): boolean {
    if (typeof document === "undefined") {
        return false;
    }
    for (const id of anchorIds) {
        const el = document.getElementById(id);
        if (el instanceof HTMLElement) {
            const rect = el.getBoundingClientRect();
            if (rect.width === 0 && rect.height === 0) {
                return true;
            }
            if (rect.bottom < 0 || rect.top > window.innerHeight) {
                return true;
            }
            if (rect.right < 0 || rect.left > window.innerWidth) {
                return true;
            }
        }
    }
    return false;
}

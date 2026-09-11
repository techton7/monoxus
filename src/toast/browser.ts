export interface ToastBrowserOptions {
    hotkey: string;
    viewportId: string;
    gap: number;
    swipeThreshold: number;
    position: string;
    dir: string;
    swipeDirections: string[];
    offsetTop: string;
    offsetRight: string;
    offsetBottom: string;
    offsetLeft: string;
    mobileOffsetTop: string;
    mobileOffsetRight: string;
    mobileOffsetBottom: string;
    mobileOffsetLeft: string;
}

export type ToastEventPayload =
    | { kind: "visibility"; visible: boolean }
    | { kind: "hotkey"; key: string }
    | { kind: "hover"; hovered: boolean }
    | { kind: "focus"; focused: boolean }
    | { kind: "swipe_active"; swiping: boolean }
    | { kind: "swipe_dismiss"; id: number }
    | { kind: "stopped" };

/**
 * Watch toast viewport gestures, stacking geometry, visibility and hotkeys
 * #[watcher]
 */
export function watchToast(
    options: ToastBrowserOptions,
    emit: (event: ToastEventPayload) => void
): () => void {
    const hotkey = options.hotkey || "F8";
    const viewportId = options.viewportId || "monoxus-toast-viewport";
    const gap = typeof options.gap === "number" ? options.gap : 14;
    const swipeThreshold =
        typeof options.swipeThreshold === "number" ? options.swipeThreshold : 45;
    const position = options.position || "bottom-right";
    const dir = options.dir || "auto";
    const swipeDirections =
        Array.isArray(options.swipeDirections) && options.swipeDirections.length > 0
            ? options.swipeDirections
            : position.startsWith("top")
            ? ["top", "right"]
            : ["bottom", "right"];

    const offset = {
        top: options.offsetTop || "32px",
        right: options.offsetRight || "32px",
        bottom: options.offsetBottom || "32px",
        left: options.offsetLeft || "32px",
    };
    const mobileOffset = {
        top: options.mobileOffsetTop || "16px",
        right: options.mobileOffsetRight || "16px",
        bottom: options.mobileOffsetBottom || "16px",
        left: options.mobileOffsetLeft || "16px",
    };

    let stopped = false;

    const getResolvedDir = (): string => {
        if (dir !== "auto") return dir;
        const docDir = document.documentElement.getAttribute("dir");
        if (docDir && docDir !== "auto") return docDir;
        return window.getComputedStyle(document.documentElement).direction || "ltr";
    };

    const applyViewportStyles = (viewport: HTMLElement) => {
        if (!viewport) return;
        const resolvedDir = getResolvedDir();
        if (!viewport.getAttribute("dir")) {
            viewport.setAttribute("dir", resolvedDir);
        }

        const keys: Array<"top" | "right" | "bottom" | "left"> = [
            "top",
            "right",
            "bottom",
            "left",
        ];
        keys.forEach((key) => {
            const offVal = offset[key];
            viewport.style.setProperty(`--offset-${key}`, offVal);
            const mobVal = mobileOffset[key];
            viewport.style.setProperty(`--mobile-offset-${key}`, mobVal);
        });
    };

    const handleVisibilityChange = () => {
        if (stopped) return;
        emit({
            kind: "visibility",
            visible: document.visibilityState === "visible",
        });
    };

    const handleKeyDown = (event: KeyboardEvent) => {
        if (stopped || event.defaultPrevented) return;
        if (event.key === hotkey) {
            event.preventDefault();
            const el = document.getElementById(viewportId);
            if (el) {
                el.focus();
            }
            emit({ kind: "hotkey", key: event.key });
        }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    document.addEventListener("keydown", handleKeyDown);

    // --- Stacking Geometry & Gestures Engine ---
    const observedElements = new Set<Element>();
    const resizeObserver = new ResizeObserver(() => {
        if (!stopped) updateStackGeometry();
    });

    const updateStackGeometry = () => {
        const viewport = document.getElementById(viewportId);
        if (!viewport) return;

        const toasts = Array.from(
            viewport.querySelectorAll<HTMLElement>(
                '[data-sonner-toast], li[role="status"], li[role="alert"]'
            )
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
            const itemOffset = accumulatedH + i * gap;
            const scale = Number(Math.max(0.7, 1.0 - i * 0.05).toFixed(3));

            toast.style.setProperty("--height", `${h}px`);
            toast.style.setProperty("--offset", `${itemOffset}px`);
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
            viewport.querySelectorAll(
                '[data-sonner-toast], li[role="status"], li[role="alert"]'
            )
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
        if (stopped) return;
        if (!isHovered) {
            isHovered = true;
            emit({ kind: "hover", hovered: true });
        }
    };

    const handleMouseMove = () => {
        if (stopped) return;
        if (!isHovered) {
            isHovered = true;
            emit({ kind: "hover", hovered: true });
        }
    };

    const handleMouseLeave = (e: MouseEvent) => {
        if (stopped) return;
        if (
            e &&
            e.relatedTarget &&
            currentBoundViewport &&
            currentBoundViewport.contains(e.relatedTarget as Node)
        ) {
            return;
        }
        if (isHovered) {
            isHovered = false;
            emit({ kind: "hover", hovered: false });
        }
    };

    const handleFocusIn = () => {
        if (!stopped) emit({ kind: "focus", focused: true });
    };

    const handleFocusOut = () => {
        if (!stopped) emit({ kind: "focus", focused: false });
    };

    let dragToast: HTMLElement | null = null;
    let dragStart: { x: number; y: number } | null = null;
    let dragStartTime = 0;
    let swipeDirection: "x" | "y" | null = null;
    let isDragging = false;

    const getDampening = (delta: number): number => {
        const factor = Math.abs(delta) / 20;
        return 1 / (1.5 + factor);
    };

    const handlePointerDown = (e: PointerEvent) => {
        if (stopped || e.button === 2) return;
        const target = e.target as HTMLElement | null;
        if (!target) return;
        const toast = target.closest<HTMLElement>(
            '[data-sonner-toast], li[role="status"], li[role="alert"]'
        );
        if (!toast) return;
        if (target.tagName === "BUTTON" || target.closest("button")) return;

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

    const getEffectiveSwipeDirections = (toast: HTMLElement | null): string[] => {
        if (toast && toast.getAttribute("data-swipe-directions")) {
            return toast
                .getAttribute("data-swipe-directions")!
                .split(",")
                .map((s) => s.trim());
        }
        const toastPos =
            (toast && toast.getAttribute("data-position")) ||
            (currentBoundViewport &&
                currentBoundViewport.getAttribute("data-position")) ||
            position;
        const [y, x] = toastPos.split("-");
        const dirs: string[] = [];
        if (y) dirs.push(y);
        if (x && x !== "center") dirs.push(x);
        return dirs.length > 0 ? dirs : swipeDirections;
    };

    const handlePointerMove = (e: PointerEvent) => {
        if (stopped || !dragToast || !dragStart) return;
        if (!isDragging && (window.getSelection()?.toString().length ?? 0) > 0) return;

        const xDelta = e.clientX - dragStart.x;
        const yDelta = e.clientY - dragStart.y;

        if (!swipeDirection && (Math.abs(xDelta) > 1 || Math.abs(yDelta) > 1)) {
            swipeDirection = Math.abs(xDelta) > Math.abs(yDelta) ? "x" : "y";
        }

        const swipeAmount = { x: 0, y: 0 };
        const activeSwipeDirs = getEffectiveSwipeDirections(dragToast);

        if (swipeDirection === "y") {
            const isAllowed =
                (activeSwipeDirs.includes("top") && yDelta < 0) ||
                (activeSwipeDirs.includes("bottom") && yDelta > 0);
            if (isAllowed) {
                swipeAmount.y = yDelta;
            } else {
                const dampenedDelta = yDelta * getDampening(yDelta);
                swipeAmount.y =
                    Math.abs(dampenedDelta) < Math.abs(yDelta) ? dampenedDelta : yDelta;
            }
        } else if (swipeDirection === "x") {
            const isAllowed =
                (activeSwipeDirs.includes("left") && xDelta < 0) ||
                (activeSwipeDirs.includes("right") && xDelta > 0);
            if (isAllowed) {
                swipeAmount.x = xDelta;
            } else {
                const dampenedDelta = xDelta * getDampening(xDelta);
                swipeAmount.x =
                    Math.abs(dampenedDelta) < Math.abs(xDelta) ? dampenedDelta : xDelta;
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
                emit({ kind: "swipe_active", swiping: true });
            }
            dragToast.style.setProperty("--swipe-amount-x", `${swipeAmount.x}px`);
            dragToast.style.setProperty("--swipe-amount-y", `${swipeAmount.y}px`);
            dragToast.style.setProperty(
                "--drag-offset",
                `${swipeDirection === "x" ? swipeAmount.x : swipeAmount.y}px`
            );
        }
    };

    const handlePointerEnd = (_e: PointerEvent) => {
        if (stopped || !dragToast) return;
        const toast = dragToast;
        const elapsed = Date.now() - dragStartTime;

        const swipeAmountX = Number(
            toast.style.getPropertyValue("--swipe-amount-x").replace("px", "") || 0
        );
        const swipeAmountY = Number(
            toast.style.getPropertyValue("--swipe-amount-y").replace("px", "") || 0
        );

        const activeSwipeDirection = swipeDirection;
        dragToast = null;
        dragStart = null;
        swipeDirection = null;

        toast.removeAttribute("data-swiping");
        toast.removeAttribute("data-swiped");

        if (isDragging) {
            isDragging = false;
            emit({ kind: "swipe_active", swiping: false });

            const activeSwipeDirs = getEffectiveSwipeDirections(toast);
            const swipeAmount =
                activeSwipeDirection === "x" ? swipeAmountX : swipeAmountY;
            const velocity = Math.abs(swipeAmount) / (elapsed || 1);

            const isAllowedDirection =
                activeSwipeDirection === "x"
                    ? activeSwipeDirs.includes(swipeAmountX > 0 ? "right" : "left")
                    : activeSwipeDirs.includes(swipeAmountY > 0 ? "bottom" : "top");

            if (
                isAllowedDirection &&
                (Math.abs(swipeAmount) >= swipeThreshold || velocity > 0.11)
            ) {
                const idStr =
                    toast.getAttribute("data-id") || toast.getAttribute("id") || "";
                const match = idStr.match(/\d+/);
                const toastId = match ? match[0] : "0";

                const swipeOutDir =
                    activeSwipeDirection === "x"
                        ? swipeAmountX > 0
                            ? "right"
                            : "left"
                        : swipeAmountY > 0
                        ? "down"
                        : "up";

                toast.setAttribute("data-swipe-out", "true");
                toast.setAttribute("data-swipe-direction", swipeOutDir);

                emit({ kind: "swipe_dismiss", id: Number(toastId) });
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

    let viewportObserver: MutationObserver | null = null;
    let currentBoundViewport: HTMLElement | null = null;

    const bindViewport = (): boolean => {
        if (stopped) return false;
        const viewport = document.getElementById(viewportId);
        if (!viewport) return false;
        if (currentBoundViewport === viewport) return true;

        if (currentBoundViewport && viewportObserver) {
            viewportObserver.disconnect();
            currentBoundViewport.removeEventListener("mouseenter", handleMouseEnter);
            currentBoundViewport.removeEventListener("mousemove", handleMouseMove);
            currentBoundViewport.removeEventListener("mouseleave", handleMouseLeave);
            currentBoundViewport.removeEventListener("focusin", handleFocusIn);
            currentBoundViewport.removeEventListener("focusout", handleFocusOut);
            currentBoundViewport.removeEventListener("pointerdown", handlePointerDown);
        }

        currentBoundViewport = viewport;
        viewportObserver = new MutationObserver(() => {
            if (!stopped) {
                syncObservedToasts();
                updateStackGeometry();
            }
        });
        viewportObserver.observe(viewport, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: ["data-expanded"],
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

    let bodyObserver: MutationObserver | null = null;
    if (typeof MutationObserver !== "undefined") {
        bodyObserver = new MutationObserver(() => {
            if (!stopped) bindViewport();
        });
        bodyObserver.observe(document.body, { childList: true, subtree: true });
    }

    return () => {
        stopped = true;
        document.removeEventListener("visibilitychange", handleVisibilityChange);
        document.removeEventListener("keydown", handleKeyDown);
        window.removeEventListener("pointermove", handlePointerMove);
        window.removeEventListener("pointerup", handlePointerEnd);
        window.removeEventListener("pointercancel", handlePointerEnd);

        bodyObserver?.disconnect();
        viewportObserver?.disconnect();
        resizeObserver.disconnect();

        if (currentBoundViewport) {
            currentBoundViewport.removeEventListener("mouseenter", handleMouseEnter);
            currentBoundViewport.removeEventListener("mousemove", handleMouseMove);
            currentBoundViewport.removeEventListener("mouseleave", handleMouseLeave);
            currentBoundViewport.removeEventListener("focusin", handleFocusIn);
            currentBoundViewport.removeEventListener("focusout", handleFocusOut);
            currentBoundViewport.removeEventListener("pointerdown", handlePointerDown);
        }
        emit({ kind: "stopped" });
    };
}

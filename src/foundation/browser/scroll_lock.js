export function acquireScrollLock(lockId) {
    const state = window.__monoxusScrollLockRuntime ??= (window.__monoxusDialogRuntime ??= {
        scrollLocks: new Set(),
        previousBodyOverflow: null,
        previousBodyPaddingRight: null,
        previousDocumentOverflow: null,
        restoreToken: 0,
    });
    window.__monoxusDialogRuntime = state;

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

export function releaseScrollLock(lockId, delayMs) {
    const state = window.__monoxusScrollLockRuntime ?? window.__monoxusDialogRuntime;
    if (!state || !(state.scrollLocks instanceof Set)) {
        return;
    }

    state.scrollLocks.delete(lockId);
    const restoreToken = ++state.restoreToken;
    if (state.scrollLocks.size > 0) {
        return;
    }

    const restore = () => {
        const currentState = window.__monoxusScrollLockRuntime ?? window.__monoxusDialogRuntime;
        if (!currentState || currentState.restoreToken !== restoreToken) {
            return;
        }

        if (currentState.scrollLocks instanceof Set && currentState.scrollLocks.size > 0) {
            return;
        }

        document.body.style.overflow = currentState.previousBodyOverflow ?? "";
        document.body.style.paddingRight = currentState.previousBodyPaddingRight ?? "";
        document.documentElement.style.overflow = currentState.previousDocumentOverflow ?? "";
        currentState.previousBodyOverflow = null;
        currentState.previousBodyPaddingRight = null;
        currentState.previousDocumentOverflow = null;
    };

    if (delayMs > 0) {
        window.setTimeout(restore, delayMs);
        return;
    }

    restore();
}

export function scrollElementIntoViewNearest(elementId) {
    const el = document.getElementById(elementId);
    if (el && typeof el.scrollIntoView === "function") {
        el.scrollIntoView({ block: "nearest", inline: "nearest" });
    }
}

export function setBodyUserSelect(prevent) {
    if (typeof document === "undefined" || !document.body) {
        return;
    }
    if (prevent) {
        document.body.style.userSelect = "none";
        document.body.style.webkitUserSelect = "none";
    } else {
        document.body.style.removeProperty("user-select");
        document.body.style.removeProperty("-webkit-user-select");
    }
}

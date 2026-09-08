export function focusElementByIdWithOptions(targetId, preventScroll) {
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

export function focusFirstFocusable(contentId, selector) {
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

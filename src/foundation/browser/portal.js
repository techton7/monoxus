export function teleportElementToHost(elementId, hostId) {
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

export function removeElementById(elementId) {
    if (typeof document === "undefined") return;
    const el = document.getElementById(elementId);
    if (el && el.parentElement) {
        el.remove();
    }
}

// Bridge for live runtime verification of dioxus-js-bindgen in playground

/**
 * Focus element by ID (Command)
 */
export function focusElement(id: string): void {
    const el = document.getElementById(id);
    if (!el) throw new Error("Element not found: " + id);
    el.focus();
}

/**
 * Failing command (Command)
 */
export function failCommand(id: string): void {
    const el = document.getElementById(id);
    if (!el) throw new Error("Intentional command error for id: " + id);
}

export interface BoxRect {
    x: number;
    y: number;
    width: number;
    height: number;
}

/**
 * Get bounding client rect (Query)
 */
export function getBoundingRect(id: string): BoxRect {
    const el = document.getElementById(id);
    if (!el) throw new Error("Query failed: element not found: " + id);
    const r = el.getBoundingClientRect();
    return { x: r.x, y: r.y, width: r.width, height: r.height };
}

export interface WindowMetrics {
    width: number;
    height: number;
}

/**
 * Watch window resize events (Watcher)
 * #[watcher]
 */
export function watchWindowResize(emit: (metrics: WindowMetrics) => void): () => void {
    const handler = () => {
        emit({ width: window.innerWidth, height: window.innerHeight });
    };
    window.addEventListener("resize", handler);
    return () => {
        window.removeEventListener("resize", handler);
    };
}

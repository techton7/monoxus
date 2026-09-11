/**
 * Query the document order of rendered options within a Select content container.
 */
export function getDocumentOptionOrder(contentId: string): string[] {
    if (typeof document === "undefined") return [];
    const root = document.getElementById(contentId);
    if (!root) return [];
    return Array.from(root.querySelectorAll('[role="option"]'))
        .map(el => el.getAttribute("data-value") || "");
}

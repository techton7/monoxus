export function getDocumentOptionOrder(contentId) {
    if (typeof document === "undefined") return [];
    const root = document.getElementById(contentId);
    if (!root) return [];
    return Array.from(root.querySelectorAll('[role="option"]'))
        .map(el => el.getAttribute("data-value") || "");
}

export function measureFloatingPlacement(anchorId, contentId, customAnchorId, boundaryId) {
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

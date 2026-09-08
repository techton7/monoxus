export function getViewportSize() {
    return [window.innerWidth, window.innerHeight];
}

export function isElementActive(targetId) {
    const target = document.getElementById(targetId);
    return target instanceof HTMLElement && document.activeElement === target;
}

export function isReferenceHidden(anchorIds) {
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

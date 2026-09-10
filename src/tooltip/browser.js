const [triggerId, contentId] = await dioxus.recv();
const stopSignal = "stop";
const stoppedSignal = "stopped";
const graceLeaveSignal = "grace:leave";

let finished = false;
let pointerGraceArea = null;
let resolveDone = null;

const done = new Promise((resolve) => {
    resolveDone = resolve;
});

const send = (signal) => {
    dioxus.send(signal);
};

// 1. Geometry helpers directly ported from Radix UI & Bits UI

function getExitSideFromRect(point, rect) {
    const top = Math.abs(rect.top - point.y);
    const bottom = Math.abs(rect.bottom - point.y);
    const right = Math.abs(rect.right - point.x);
    const left = Math.abs(rect.left - point.x);

    switch (Math.min(top, bottom, right, left)) {
        case left:
            return "left";
        case right:
            return "right";
        case top:
            return "top";
        case bottom:
            return "bottom";
        default:
            return "bottom";
    }
}

function getPaddedExitPoints(exitPoint, exitSide, padding = 5) {
    const tipPadding = padding * 1.5;
    switch (exitSide) {
        case "top":
            return [
                { x: exitPoint.x - padding, y: exitPoint.y + padding },
                { x: exitPoint.x, y: exitPoint.y - tipPadding },
                { x: exitPoint.x + padding, y: exitPoint.y + padding },
            ];
        case "bottom":
            return [
                { x: exitPoint.x - padding, y: exitPoint.y - padding },
                { x: exitPoint.x, y: exitPoint.y + tipPadding },
                { x: exitPoint.x + padding, y: exitPoint.y - padding },
            ];
        case "left":
            return [
                { x: exitPoint.x + padding, y: exitPoint.y - padding },
                { x: exitPoint.x - tipPadding, y: exitPoint.y },
                { x: exitPoint.x + padding, y: exitPoint.y + padding },
            ];
        case "right":
            return [
                { x: exitPoint.x - padding, y: exitPoint.y - padding },
                { x: exitPoint.x + tipPadding, y: exitPoint.y },
                { x: exitPoint.x - padding, y: exitPoint.y + padding },
            ];
    }
}

function getPointsFromRect(rect) {
    const { top, right, bottom, left } = rect;
    return [
        { x: left, y: top },
        { x: right, y: top },
        { x: right, y: bottom },
        { x: left, y: bottom },
    ];
}

// Ray-casting point-in-polygon algorithm (substack/point-in-polygon)
function isPointInPolygon(point, polygon) {
    const { x, y } = point;
    let inside = false;
    for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
        const ii = polygon[i];
        const jj = polygon[j];
        const xi = ii.x;
        const yi = ii.y;
        const xj = jj.x;
        const yj = jj.y;

        const intersect = (yi > y !== yj > y) && (x < ((xj - xi) * (y - yi)) / (yj - yi) + xi);
        if (intersect) inside = !inside;
    }
    return inside;
}

// 2D Convex Hull (Andrew's monotone chain algorithm)
function getHull(points) {
    const newPoints = points.slice();
    newPoints.sort((a, b) => {
        if (a.x < b.x) return -1;
        if (a.x > b.x) return 1;
        if (a.y < b.y) return -1;
        if (a.y > b.y) return 1;
        return 0;
    });

    if (newPoints.length <= 1) return newPoints.slice();

    const upperHull = [];
    for (let i = 0; i < newPoints.length; i++) {
        const p = newPoints[i];
        while (upperHull.length >= 2) {
            const q = upperHull[upperHull.length - 1];
            const r = upperHull[upperHull.length - 2];
            if ((q.x - r.x) * (p.y - r.y) >= (q.y - r.y) * (p.x - r.x)) upperHull.pop();
            else break;
        }
        upperHull.push(p);
    }
    upperHull.pop();

    const lowerHull = [];
    for (let i = newPoints.length - 1; i >= 0; i--) {
        const p = newPoints[i];
        while (lowerHull.length >= 2) {
            const q = lowerHull[lowerHull.length - 1];
            const r = lowerHull[lowerHull.length - 2];
            if ((q.x - r.x) * (p.y - r.y) >= (q.y - r.y) * (p.x - r.x)) lowerHull.pop();
            else break;
        }
        lowerHull.push(p);
    }
    lowerHull.pop();

    if (
        upperHull.length === 1 &&
        lowerHull.length === 1 &&
        upperHull[0].x === lowerHull[0].x &&
        upperHull[0].y === lowerHull[0].y
    ) {
        return upperHull;
    }
    return upperHull.concat(lowerHull);
}

// 2. DOM elements and Grace Area Event Manager

let triggerNode = document.getElementById(triggerId);
let contentNode = document.getElementById(contentId);

const getTrigger = () => {
    if (!triggerNode) triggerNode = document.getElementById(triggerId);
    return triggerNode;
};

const getContent = () => {
    if (!contentNode) contentNode = document.getElementById(contentId);
    return contentNode;
};

const removeGraceArea = () => {
    if (pointerGraceArea !== null) {
        pointerGraceArea = null;
        document.removeEventListener("pointermove", handleTrackPointerGrace);
    }
};

const createGraceArea = (e, hoverTarget) => {
    if (finished || !hoverTarget) return;
    const currentTarget = e.currentTarget;
    if (!(currentTarget instanceof HTMLElement)) return;

    const exitPoint = { x: e.clientX, y: e.clientY };
    const exitSide = getExitSideFromRect(exitPoint, currentTarget.getBoundingClientRect());
    const paddedExitPoints = getPaddedExitPoints(exitPoint, exitSide);
    const hoverTargetPoints = getPointsFromRect(hoverTarget.getBoundingClientRect());
    pointerGraceArea = getHull([...paddedExitPoints, ...hoverTargetPoints]);

    document.removeEventListener("pointermove", handleTrackPointerGrace);
    document.addEventListener("pointermove", handleTrackPointerGrace);
};

const handleTrackPointerGrace = (e) => {
    if (finished || !pointerGraceArea) return;

    const target = e.target;
    const pointerPosition = { x: e.clientX, y: e.clientY };

    const trigger = getTrigger();
    const content = getContent();
    const hasEnteredTarget =
        (trigger && trigger.contains(target)) ||
        (content && content.contains(target));

    if (hasEnteredTarget) {
        removeGraceArea();
        return;
    }

    const isPointerOutside = !isPointInPolygon(pointerPosition, pointerGraceArea);
    if (isPointerOutside) {
        removeGraceArea();
        send(graceLeaveSignal);
    }
};

const handleTriggerLeave = (e) => {
    createGraceArea(e, getContent());
};

const handleContentLeave = (e) => {
    createGraceArea(e, getTrigger());
};

const attachListeners = () => {
    const trigger = getTrigger();
    const content = getContent();
    if (trigger instanceof HTMLElement) {
        trigger.removeEventListener("pointerleave", handleTriggerLeave);
        trigger.addEventListener("pointerleave", handleTriggerLeave);
    }
    if (content instanceof HTMLElement) {
        content.removeEventListener("pointerleave", handleContentLeave);
        content.addEventListener("pointerleave", handleContentLeave);
    }
};

attachListeners();
if (!triggerNode || !contentNode) {
    queueMicrotask(attachListeners);
}

const cleanup = () => {
    removeGraceArea();
    const trigger = getTrigger();
    const content = getContent();
    if (trigger instanceof HTMLElement) {
        trigger.removeEventListener("pointerleave", handleTriggerLeave);
    }
    if (content instanceof HTMLElement) {
        content.removeEventListener("pointerleave", handleContentLeave);
    }
};

const finish = (signal) => {
    if (finished) return;
    finished = true;
    cleanup();
    send(signal);
    resolveDone?.();
};

void (async () => {
    try {
        const command = await dioxus.recv();
        if (command === stopSignal) {
            finish(stoppedSignal);
        }
    } catch (_) {
        finish(stoppedSignal);
    }
})();

await done;

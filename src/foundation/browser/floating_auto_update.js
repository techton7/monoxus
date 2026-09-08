const [anchorIds, contentId] = await dioxus.recv();
const stopSignal = "stop";
const stoppedSignal = "stopped";
const updateSignal = "update";
const scrollSignal = "scroll";
const mutationTarget = document.body ?? document.documentElement;
const visualViewport = window.visualViewport ?? null;
const ResizeObserverCtor = window.ResizeObserver ?? null;
const MutationObserverCtor = window.MutationObserver ?? null;
let stopped = false;
let framePending = false;
let scrollTriggered = false;
let resizeObserver = null;
let mutationObserver = null;
let currentAnchor = null;
let currentContent = null;

const readElement = (id) => {
    const element = document.getElementById(id);
    return element instanceof HTMLElement ? element : null;
};

const resolveAnchor = () => {
    for (const id of anchorIds) {
        const anchor = readElement(id);
        if (anchor) {
            return anchor;
        }
    }

    return null;
};

const reconnectObservedElements = () => {
    const nextAnchor = resolveAnchor();
    const nextContent = readElement(contentId);

    if (ResizeObserverCtor === null) {
        currentAnchor = nextAnchor;
        currentContent = nextContent;
        return;
    }

    if (resizeObserver === null) {
        resizeObserver = new ResizeObserverCtor(() => queueUpdate());
    }

    if (currentAnchor === nextAnchor && currentContent === nextContent) {
        return;
    }

    resizeObserver.disconnect();
    resizeObserver.observe(document.documentElement);

    currentAnchor = nextAnchor;
    currentContent = nextContent;

    if (currentAnchor) {
        resizeObserver.observe(currentAnchor);
    }

    if (currentContent) {
        resizeObserver.observe(currentContent);
    }
};

const queueUpdate = ({ fromScroll = false } = {}) => {
    if (stopped || framePending) {
        return;
    }

    scrollTriggered = scrollTriggered || fromScroll;
    framePending = true;
    window.requestAnimationFrame(() => {
        framePending = false;
        if (stopped) {
            return;
        }

        reconnectObservedElements();
        const signal = scrollTriggered ? scrollSignal : updateSignal;
        scrollTriggered = false;
        dioxus.send(signal);
    });
};

const handleScroll = () => queueUpdate({ fromScroll: true });
const handleResize = () => queueUpdate();

window.addEventListener("scroll", handleScroll, { capture: true, passive: true });
window.addEventListener("resize", handleResize, { passive: true });

if (visualViewport) {
    visualViewport.addEventListener("scroll", handleScroll, { passive: true });
    visualViewport.addEventListener("resize", handleResize, { passive: true });
}

if (MutationObserverCtor) {
    mutationObserver = new MutationObserverCtor(() => {
        reconnectObservedElements();
        queueUpdate();
    });
}

if (mutationTarget && mutationObserver) {
    mutationObserver.observe(mutationTarget, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["class", "style", "hidden"],
    });
}

reconnectObservedElements();

const command = await dioxus.recv();
if (command !== stopSignal) {
    console.error(`Unexpected floating auto-update command: ${String(command)}`);
}

stopped = true;
window.removeEventListener("scroll", handleScroll, true);
window.removeEventListener("resize", handleResize);

if (visualViewport) {
    visualViewport.removeEventListener("scroll", handleScroll);
    visualViewport.removeEventListener("resize", handleResize);
}

mutationObserver?.disconnect();
resizeObserver?.disconnect();
dioxus.send(stoppedSignal);

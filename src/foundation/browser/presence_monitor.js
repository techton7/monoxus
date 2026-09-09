const [rootId, cycleId] = await dioxus.recv();
const stopSignal = "stop";
const stoppedSignal = "stopped";
const fallbackSignal = "fallback";
const animationEndSignal = "animationend";
const animationCancelSignal = "animationcancel";
const missingReason = "missing";
const hiddenReason = "hidden";
const noAnimationReason = "no-animation";
const MutationObserverCtor = window.MutationObserver ?? null;
const root = document.getElementById(rootId);
let activeAnimationNames = [];
let finished = false;
let mutationObserver = null;
let resolveDone = null;

const done = new Promise((resolve) => {
    resolveDone = resolve;
});

const parseAnimationNames = (value) =>
    String(value ?? "")
        .split(",")
        .map((name) => name.trim())
        .filter((name) => name.length > 0 && name !== "none");

const send = (signal, detail = "") => {
    dioxus.send([signal, String(cycleId), detail].join("\n"));
};

const cleanup = () => {
    mutationObserver?.disconnect();
    if (!(root instanceof HTMLElement)) {
        return;
    }

    root.removeEventListener("animationend", handleAnimationEnd);
    root.removeEventListener("animationcancel", handleAnimationCancel);
};

const finish = (signal, detail = "") => {
    if (finished) {
        return;
    }

    finished = true;
    cleanup();
    send(signal, detail);
    resolveDone?.();
};

const refreshRootState = () => {
    if (!(root instanceof HTMLElement) || !root.isConnected) {
        finish(fallbackSignal, missingReason);
        return;
    }

    const computed = window.getComputedStyle(root);
    if (root.hidden || computed.display === "none") {
        finish(fallbackSignal, hiddenReason);
        return;
    }

    activeAnimationNames = parseAnimationNames(computed.animationName);
    if (activeAnimationNames.length === 0) {
        finish(fallbackSignal, noAnimationReason);
    }
};

const handleAnimation = (event, signal) => {
    if (finished || event.target !== root) {
        return;
    }

    if (
        typeof event.animationName !== "string" ||
        !activeAnimationNames.includes(event.animationName)
    ) {
        return;
    }

    finish(signal, event.animationName);
};

const handleAnimationEnd = (event) => handleAnimation(event, animationEndSignal);
const handleAnimationCancel = (event) => handleAnimation(event, animationCancelSignal);

if (root instanceof HTMLElement) {
    root.addEventListener("animationend", handleAnimationEnd);
    root.addEventListener("animationcancel", handleAnimationCancel);
}

if (MutationObserverCtor !== null) {
    mutationObserver = new MutationObserverCtor(() => {
        refreshRootState();
    });
    mutationObserver.observe(document.body ?? document.documentElement, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["class", "style", "hidden"],
    });
}

window.requestAnimationFrame(() => {
    refreshRootState();
});

void (async () => {
    const command = await dioxus.recv();
    if (finished) {
        return;
    }

    if (command !== stopSignal) {
        console.error(`Unexpected presence monitor command: ${String(command)}`);
    }

    finish(stoppedSignal);
})();

await done;

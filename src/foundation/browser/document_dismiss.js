const stopSignal = "stop";
const stoppedSignal = "stopped";
const pointerDownSignal = "pointerdown";
const focusInSignal = "focusin";
const escapeSignal = "escape";
const pathSeparator = "\u001f";

const readPathIds = (event) => {
    if (!event || typeof event.composedPath !== "function") {
        return [];
    }

    return event
        .composedPath()
        .filter((node) => node instanceof HTMLElement && typeof node.id === "string" && node.id.length > 0)
        .map((node) => node.id);
};

const sendEvent = (signal, pathIds = []) => {
    dioxus.send([signal, pathIds.join(pathSeparator)].join("\n"));
};

const handlePointerDown = (event) => sendEvent(pointerDownSignal, readPathIds(event));
const handleFocusIn = (event) => sendEvent(focusInSignal, readPathIds(event));
const handleKeyDown = (event) => {
    if (event.defaultPrevented) {
        return;
    }
    if (event.key === "Escape") {
        sendEvent(escapeSignal);
    }
};

document.addEventListener("pointerdown", handlePointerDown, true);
document.addEventListener("focusin", handleFocusIn, true);
document.addEventListener("keydown", handleKeyDown, false);

const command = await dioxus.recv();
if (command !== stopSignal) {
    console.error(`Unexpected document dismiss command: ${String(command)}`);
}

document.removeEventListener("pointerdown", handlePointerDown, true);
document.removeEventListener("focusin", handleFocusIn, true);
document.removeEventListener("keydown", handleKeyDown, false);
dioxus.send([stoppedSignal, ""].join("\n"));

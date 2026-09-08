const [elementId, formId] = await dioxus.recv();
const stopSignal = "stop";
const stoppedSignal = "stopped";
const resetSignal = "reset";

const el = document.getElementById(elementId);
const form = formId ? document.getElementById(formId) : (el ? el.closest("form") : null);

const handleReset = () => {
    dioxus.send(resetSignal);
};

if (form) {
    form.addEventListener("reset", handleReset);
}

const cmd = await dioxus.recv();
if (form) {
    form.removeEventListener("reset", handleReset);
}
dioxus.send(stoppedSignal);

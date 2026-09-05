use dioxus::{document, document::Eval, prelude::spawn};

use crate::foundation::compose::MountedHandle;

const FLOATING_AUTO_UPDATE_SIGNAL_STOP: &str = "stop";
const FLOATING_AUTO_UPDATE_SIGNAL_STOPPED: &str = "stopped";
const FLOATING_AUTO_UPDATE_SIGNAL_SCROLL: &str = "scroll";
const FLOATING_AUTO_UPDATE_SIGNAL_UPDATE: &str = "update";
const DOCUMENT_DISMISS_SIGNAL_STOP: &str = "stop";
const DOCUMENT_DISMISS_SIGNAL_STOPPED: &str = "stopped";
const DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN: &str = "pointerdown";
const DOCUMENT_DISMISS_SIGNAL_FOCUS_IN: &str = "focusin";
const DOCUMENT_DISMISS_SIGNAL_ESCAPE: &str = "escape";
const DOCUMENT_DISMISS_PATH_SEPARATOR: char = '\u{1f}';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FloatingAutoUpdateEvent {
    Scroll,
    Update,
    Stopped,
}

pub(crate) fn start_floating_auto_update_monitor(anchor_ids: &[&str], content_id: &str) -> Eval {
    let anchor_ids = format!("{anchor_ids:?}");

    document::eval(&format!(
        r#"
const anchorIds = {anchor_ids};
const contentId = {content_id:?};
const stopSignal = {FLOATING_AUTO_UPDATE_SIGNAL_STOP:?};
const stoppedSignal = {FLOATING_AUTO_UPDATE_SIGNAL_STOPPED:?};
const updateSignal = {FLOATING_AUTO_UPDATE_SIGNAL_UPDATE:?};
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

const readElement = (id) => {{
    const element = document.getElementById(id);
    return element instanceof HTMLElement ? element : null;
}};

const resolveAnchor = () => {{
    for (const id of anchorIds) {{
        const anchor = readElement(id);
        if (anchor) {{
            return anchor;
        }}
    }}

    return null;
}};

const reconnectObservedElements = () => {{
    const nextAnchor = resolveAnchor();
    const nextContent = readElement(contentId);

    if (ResizeObserverCtor === null) {{
        currentAnchor = nextAnchor;
        currentContent = nextContent;
        return;
    }}

    if (resizeObserver === null) {{
        resizeObserver = new ResizeObserverCtor(() => queueUpdate());
    }}

    if (currentAnchor === nextAnchor && currentContent === nextContent) {{
        return;
    }}

    resizeObserver.disconnect();
    resizeObserver.observe(document.documentElement);

    currentAnchor = nextAnchor;
    currentContent = nextContent;

    if (currentAnchor) {{
        resizeObserver.observe(currentAnchor);
    }}

    if (currentContent) {{
        resizeObserver.observe(currentContent);
    }}
}};

const queueUpdate = ({{ fromScroll = false }} = {{}}) => {{
    if (stopped || framePending) {{
        return;
    }}

    scrollTriggered = scrollTriggered || fromScroll;
    framePending = true;
    window.requestAnimationFrame(() => {{
        framePending = false;
        if (stopped) {{
            return;
        }}

        reconnectObservedElements();
        const signal = scrollTriggered ? {FLOATING_AUTO_UPDATE_SIGNAL_SCROLL:?} : updateSignal;
        scrollTriggered = false;
        dioxus.send(signal);
    }});
}};

const handleScroll = () => queueUpdate({{ fromScroll: true }});
const handleResize = () => queueUpdate();

window.addEventListener("scroll", handleScroll, {{ capture: true, passive: true }});
window.addEventListener("resize", handleResize, {{ passive: true }});

if (visualViewport) {{
    visualViewport.addEventListener("scroll", handleScroll, {{ passive: true }});
    visualViewport.addEventListener("resize", handleResize, {{ passive: true }});
}}

if (MutationObserverCtor) {{
    mutationObserver = new MutationObserverCtor(() => {{
        reconnectObservedElements();
        queueUpdate();
    }});
}}

if (mutationTarget && mutationObserver) {{
    mutationObserver.observe(mutationTarget, {{
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["class", "style", "hidden"],
    }});
}}

reconnectObservedElements();

const command = await dioxus.recv();
if (command !== stopSignal) {{
    console.error(`Unexpected floating auto-update command: ${{String(command)}}`);
}}

stopped = true;
window.removeEventListener("scroll", handleScroll, true);
window.removeEventListener("resize", handleResize);

if (visualViewport) {{
    visualViewport.removeEventListener("scroll", handleScroll);
    visualViewport.removeEventListener("resize", handleResize);
}}

mutationObserver?.disconnect();
resizeObserver?.disconnect();
dioxus.send(stoppedSignal);
"#,
    ))
}

pub(crate) fn stop_floating_auto_update_monitor(monitor: Eval) -> Result<(), String> {
    monitor
        .send(FLOATING_AUTO_UPDATE_SIGNAL_STOP)
        .map_err(|error| format!("floating auto-update stop failed: {error}"))
}

pub(crate) async fn recv_floating_auto_update_event(
    monitor: &mut Eval,
) -> Result<FloatingAutoUpdateEvent, String> {
    let signal: String = monitor
        .recv()
        .await
        .map_err(|error| format!("floating auto-update receive failed: {error}"))?;

    match signal.as_str() {
        FLOATING_AUTO_UPDATE_SIGNAL_SCROLL => Ok(FloatingAutoUpdateEvent::Scroll),
        FLOATING_AUTO_UPDATE_SIGNAL_UPDATE => Ok(FloatingAutoUpdateEvent::Update),
        FLOATING_AUTO_UPDATE_SIGNAL_STOPPED => Ok(FloatingAutoUpdateEvent::Stopped),
        other => Err(format!("unknown floating auto-update signal: {other}")),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DocumentDismissEvent {
    PointerDown { path_ids: Vec<String> },
    FocusIn { path_ids: Vec<String> },
    Escape,
    Stopped,
}

pub(crate) fn start_document_dismiss_monitor() -> Eval {
    document::eval(&format!(
        r#"
const stopSignal = {DOCUMENT_DISMISS_SIGNAL_STOP:?};
const stoppedSignal = {DOCUMENT_DISMISS_SIGNAL_STOPPED:?};
const pointerDownSignal = {DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN:?};
const focusInSignal = {DOCUMENT_DISMISS_SIGNAL_FOCUS_IN:?};
const escapeSignal = {DOCUMENT_DISMISS_SIGNAL_ESCAPE:?};
const pathSeparator = "\u001f";

const readPathIds = (event) => {{
    if (!event || typeof event.composedPath !== "function") {{
        return [];
    }}

    return event
        .composedPath()
        .filter((node) => node instanceof HTMLElement && typeof node.id === "string" && node.id.length > 0)
        .map((node) => node.id);
}};

const sendEvent = (signal, pathIds = []) => {{
    dioxus.send([signal, pathIds.join(pathSeparator)].join("\n"));
}};

const handlePointerDown = (event) => sendEvent(pointerDownSignal, readPathIds(event));
const handleFocusIn = (event) => sendEvent(focusInSignal, readPathIds(event));
const handleKeyDown = (event) => {{
    if (event.key === "Escape") {{
        sendEvent(escapeSignal);
    }}
}};

document.addEventListener("pointerdown", handlePointerDown, true);
document.addEventListener("focusin", handleFocusIn, true);
document.addEventListener("keydown", handleKeyDown, true);

const command = await dioxus.recv();
if (command !== stopSignal) {{
    console.error(`Unexpected document dismiss command: ${{String(command)}}`);
}}

document.removeEventListener("pointerdown", handlePointerDown, true);
document.removeEventListener("focusin", handleFocusIn, true);
document.removeEventListener("keydown", handleKeyDown, true);
dioxus.send([stoppedSignal, ""].join("\n"));
"#,
    ))
}

pub(crate) fn stop_document_dismiss_monitor(monitor: Eval) -> Result<(), String> {
    monitor
        .send(DOCUMENT_DISMISS_SIGNAL_STOP)
        .map_err(|error| format!("document dismiss stop failed: {error}"))
}

pub(crate) async fn recv_document_dismiss_event(
    monitor: &mut Eval,
) -> Result<DocumentDismissEvent, String> {
    let payload: String = monitor
        .recv()
        .await
        .map_err(|error| format!("document dismiss receive failed: {error}"))?;

    parse_document_dismiss_event(&payload)
}

fn parse_document_dismiss_event(payload: &str) -> Result<DocumentDismissEvent, String> {
    let mut fields = payload.splitn(2, '\n');
    let signal = fields.next().unwrap_or_default();
    let path_ids = parse_document_dismiss_path_ids(fields.next());

    match signal {
        DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN => Ok(DocumentDismissEvent::PointerDown { path_ids }),
        DOCUMENT_DISMISS_SIGNAL_FOCUS_IN => Ok(DocumentDismissEvent::FocusIn { path_ids }),
        DOCUMENT_DISMISS_SIGNAL_ESCAPE => Ok(DocumentDismissEvent::Escape),
        DOCUMENT_DISMISS_SIGNAL_STOPPED => Ok(DocumentDismissEvent::Stopped),
        other => Err(format!("unknown document dismiss signal: {other}")),
    }
}

fn parse_document_dismiss_path_ids(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(DOCUMENT_DISMISS_PATH_SEPARATOR)
        .filter(|value| !value.is_empty())
        .map(String::from)
        .collect()
}

pub(crate) const DEFAULT_FOCUSABLE_SELECTOR: &str = "a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex='-1'])";

pub(crate) fn focus_mounted_handle(handle: Option<MountedHandle>) -> bool {
    let Some(handle) = handle else {
        return false;
    };

    spawn(async move {
        let _ = handle.set_focus(true).await;
    });
    true
}

pub(crate) fn focus_element_by_id(target_id: &str) {
    focus_element_by_id_with_options(target_id, false);
}

pub(crate) fn restore_focus_element_by_id(target_id: &str) {
    focus_element_by_id_with_options(target_id, true);
}

fn focus_element_by_id_with_options(target_id: &str, prevent_scroll: bool) {
    document::eval(&format!(
        r#"(function() {{
    const target = document.getElementById({target_id:?});
    if (!(target instanceof HTMLElement)) {{
        return;
    }}

    target.focus({{ preventScroll: {prevent_scroll} }});
}})();"#,
    ));
}

pub(crate) fn focus_first_focusable(content_id: &str, focusable_selector: Option<&str>) {
    let selector = focusable_selector.unwrap_or(DEFAULT_FOCUSABLE_SELECTOR);
    document::eval(&format!(
        r#"(function() {{
    const root = document.getElementById({content_id:?});
    if (!(root instanceof HTMLElement)) {{
        return;
    }}

    const selector = {selector:?};
    const candidate =
        (root.matches(selector) ? root : root.querySelector(selector));

    if (candidate instanceof HTMLElement) {{
        if (candidate === root && !root.hasAttribute("tabindex")) {{
            root.setAttribute("tabindex", "-1");
        }}

        candidate.focus();
        return;
    }}

    if (!root.hasAttribute("tabindex")) {{
        root.setAttribute("tabindex", "-1");
    }}

    root.focus();
}})();"#,
    ));
}

pub(crate) fn acquire_scroll_lock(lock_id: &str) {
    document::eval(&format!(
        r#"(function() {{
    const lockId = {lock_id:?};
    const state = window.__monoxusScrollLockRuntime ??= (window.__monoxusDialogRuntime ??= {{
        scrollLocks: new Set(),
        previousBodyOverflow: null,
        previousBodyPaddingRight: null,
        previousDocumentOverflow: null,
        restoreToken: 0,
    }});
    window.__monoxusDialogRuntime = state;

    state.restoreToken += 1;
    if (state.scrollLocks.has(lockId)) {{
        return;
    }}

    if (state.scrollLocks.size === 0) {{
        const body = document.body;
        const documentElement = document.documentElement;
        const computedBodyStyle = window.getComputedStyle(body);
        const bodyPaddingRight = Number.parseFloat(computedBodyStyle.paddingRight || "0") || 0;
        const scrollbarWidth = Math.max(0, window.innerWidth - documentElement.clientWidth);

        state.previousBodyOverflow = body.style.overflow;
        state.previousBodyPaddingRight = body.style.paddingRight;
        state.previousDocumentOverflow = documentElement.style.overflow;

        body.style.overflow = "hidden";
        documentElement.style.overflow = "hidden";

        if (scrollbarWidth > 0) {{
            body.style.paddingRight = `${{bodyPaddingRight + scrollbarWidth}}px`;
        }}
    }}

    state.scrollLocks.add(lockId);
}})();"#,
    ));
}

pub(crate) fn release_scroll_lock(lock_id: &str, restore_delay: Option<u64>) {
    let delay_ms = restore_delay.unwrap_or_default();

    document::eval(&format!(
        r#"(function() {{
    const lockId = {lock_id:?};
    const delayMs = {delay_ms};
    const state = window.__monoxusScrollLockRuntime ?? window.__monoxusDialogRuntime;
    if (!state || !(state.scrollLocks instanceof Set)) {{
        return;
    }}

    state.scrollLocks.delete(lockId);
    const restoreToken = ++state.restoreToken;
    if (state.scrollLocks.size > 0) {{
        return;
    }}

    const restore = () => {{
        const currentState = window.__monoxusScrollLockRuntime ?? window.__monoxusDialogRuntime;
        if (!currentState || currentState.restoreToken !== restoreToken) {{
            return;
        }}

        if (currentState.scrollLocks instanceof Set && currentState.scrollLocks.size > 0) {{
            return;
        }}

        document.body.style.overflow = currentState.previousBodyOverflow ?? "";
        document.body.style.paddingRight = currentState.previousBodyPaddingRight ?? "";
        document.documentElement.style.overflow = currentState.previousDocumentOverflow ?? "";
        currentState.previousBodyOverflow = null;
        currentState.previousBodyPaddingRight = null;
        currentState.previousDocumentOverflow = null;
    }};

    if (delayMs > 0) {{
        window.setTimeout(restore, delayMs);
        return;
    }}

    restore();
}})();"#,
    ));
}

#[cfg(test)]
mod tests {
    use super::{
        DOCUMENT_DISMISS_SIGNAL_ESCAPE, DOCUMENT_DISMISS_SIGNAL_FOCUS_IN,
        DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN, DocumentDismissEvent, parse_document_dismiss_event,
    };

    #[test]
    fn parses_document_dismiss_events() {
        assert_eq!(
            parse_document_dismiss_event(&format!(
                "{DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN}\ncontent\u{1f}child"
            )),
            Ok(DocumentDismissEvent::PointerDown {
                path_ids: vec![String::from("content"), String::from("child")],
            }),
        );
        assert_eq!(
            parse_document_dismiss_event(&format!("{DOCUMENT_DISMISS_SIGNAL_FOCUS_IN}\ncontent")),
            Ok(DocumentDismissEvent::FocusIn {
                path_ids: vec![String::from("content")],
            }),
        );
        assert_eq!(
            parse_document_dismiss_event(&format!("{DOCUMENT_DISMISS_SIGNAL_ESCAPE}\n")),
            Ok(DocumentDismissEvent::Escape),
        );
    }
}

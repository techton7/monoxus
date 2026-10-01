use dioxus::prelude::*;

use crate::foundation::{overlay::PortalHost, shared::ScopeHandle};

use super::{
    runtime::{DialogRuntime, use_dialog_runtime},
    state::Dialog,
    types::{
        DialogCloseFocusPolicy, DialogMode, DialogOpenFocusPolicy, DialogOutsideInteractionPolicy,
        DialogScrollLockPolicy,
    },
};

#[derive(Clone, Copy)]
pub struct DialogContext {
    pub runtime: Signal<DialogRuntime>,
    pub open: Signal<bool>,
    pub trigger_id: Signal<Option<String>>,
    pub on_open_change: Option<EventHandler<bool>>,
    pub is_alert_dialog: bool,
}

#[component]
pub fn DialogRoot(
    #[props(default)] id: Option<String>,
    #[props(default)] open: Option<Signal<bool>>,
    #[props(default)] default_open: Option<bool>,
    #[props(default)] on_open_change: Option<EventHandler<bool>>,
    #[props(default)] mode: Option<DialogMode>,
    #[props(default)] open_focus: Option<DialogOpenFocusPolicy>,
    #[props(default)] close_focus: Option<DialogCloseFocusPolicy>,
    #[props(default)] scroll_lock: Option<DialogScrollLockPolicy>,
    #[props(default)] outside_interaction: Option<DialogOutsideInteractionPolicy>,
    #[props(default)] portal_host: Option<PortalHost>,
    children: Element,
) -> Element {
    let open_signal = open.unwrap_or_else(|| use_signal(|| default_open.unwrap_or(false)));
    let scope_id = id.clone().unwrap_or_else(|| "dialog".to_string());
    let scope = ScopeHandle::root("dialog").child(scope_id);

    let mut dialog = Dialog::new(scope, open_signal());
    if let Some(m) = mode {
        dialog.lifecycle_mut().set_mode(m);
    }
    if let Some(of) = open_focus {
        dialog.lifecycle_mut().set_open_focus_policy(of);
    }
    if let Some(cf) = close_focus {
        dialog.lifecycle_mut().set_close_focus_policy(cf);
    }
    if let Some(sl) = scroll_lock {
        dialog.lifecycle_mut().set_scroll_lock_policy(sl);
    }
    if let Some(oi) = outside_interaction {
        dialog.lifecycle_mut().set_outside_interaction_policy(oi);
    }
    if let Some(ph) = portal_host {
        dialog = dialog.with_portal_host(ph);
    }

    let runtime = use_dialog_runtime(dialog);
    let mut runtime_sig = use_signal(|| runtime.clone());
    if runtime_sig.peek().is_open() != runtime.is_open() {
        runtime_sig.set(runtime.clone());
    }

    let trigger_id = use_signal(|| None);
    use_context_provider(|| DialogContext {
        runtime: runtime_sig,
        open: open_signal,
        trigger_id,
        on_open_change,
        is_alert_dialog: false,
    });

    let mut was_open_sig = use_signal(|| open_signal());
    use_effect(move || {
        let is_curr = open_signal();
        let was_open = *was_open_sig.peek();
        if was_open && !is_curr {
            let trig = trigger_id
                .peek()
                .clone()
                .unwrap_or_else(|| runtime_sig.peek().relationships().trigger_id().to_string());
            crate::foundation::browser::restore_focus_element_by_id(&trig);
        }
        if was_open != is_curr {
            was_open_sig.set(is_curr);
        }
    });

    let attrs = runtime.root();

    rsx! {
        div {
            id: "{attrs.id()}",
            style: "display: contents;",
            "data-state": attrs.data_state().as_str(),
            {children}
        }
    }
}

#[component]
pub fn DialogTrigger(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let mut ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let trigger = runtime.trigger();
    let trig_id = id.unwrap_or_else(|| trigger.id().to_string());
    runtime.register_trigger_id(&trig_id);
    let mut open = ctx.open;
    let on_change = ctx.on_open_change;

    if ctx.trigger_id.peek().as_deref() != Some(trig_id.as_str()) {
        ctx.trigger_id.set(Some(trig_id.clone()));
    }

    let on_click = move |_| {
        open.set(true);
        if let Some(h) = on_change {
            h.call(true);
        }
    };

    rsx! {
        button {
            id: "{trig_id}",
            r#type: "button",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            aria_haspopup: "dialog",
            aria_expanded: "{trigger.aria_expanded()}",
            aria_controls: "{trigger.aria_controls()}",
            "data-state": trigger.data_state().as_str(),
            onmounted: runtime.mount_trigger(),
            onclick: on_click,
            {children}
        }
    }
}

#[component]
pub fn DialogPortal(
    #[props(default)] host: Option<PortalHost>,
    #[props(default = false)] force_mount: bool,
    children: Element,
) -> Element {
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let should_render = force_mount || runtime.should_render_portal();
    if !should_render {
        return rsx! {};
    }

    let resolved_host = host.unwrap_or_else(|| runtime.portal().host().clone());
    let host_attr = if resolved_host.is_inline() {
        "inline"
    } else {
        resolved_host.id().unwrap_or("default")
    };

    let pid = runtime.relationships().scope().qualify("portal");
    let pid_effect = pid.clone();
    let target_host = resolved_host.clone();
    use_effect(use_reactive((&should_render,), move |(render,)| {
        if !render && !force_mount {
            return;
        }
        if let PortalHost::Named(name) = &target_host {
            crate::foundation::browser::teleport_element_to_host(&pid_effect, Some(name.as_ref()));
        }
    }));

    let pid_cleanup = pid.clone();
    let is_named_host = matches!(resolved_host, PortalHost::Named(_));
    dioxus::core::use_drop(move || {
        if is_named_host {
            crate::foundation::browser::remove_element_by_id(&pid_cleanup);
        }
    });

    rsx! {
        div {
            id: "{pid}",
            style: "display: contents;",
            "data-portal-host": "{host_attr}",
            {children}
        }
    }
}

#[component]
pub fn DialogOverlay(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default = false)] force_mount: bool,
    children: Option<Element>,
) -> Element {
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let should_render = force_mount || runtime.should_render_overlay();
    if !should_render {
        return rsx! {};
    }

    let overlay = runtime.overlay();
    let overlay_id = id.unwrap_or_else(|| overlay.id().to_string());
    runtime.register_overlay_id(&overlay_id);
    let is_open = (ctx.open)();
    let mut open = ctx.open;
    let on_change = ctx.on_open_change;
    let outside_policy = *runtime.lifecycle().outside_interaction_policy();

    let trigger_sig = ctx.trigger_id;
    let fallback_trigger_id = runtime.relationships().trigger_id().to_string();

    let on_click = move |_| {
        if !ctx.is_alert_dialog && outside_policy.pointer_down_outside().dismisses() {
            open.set(false);
            if let Some(h) = on_change {
                h.call(false);
            }
            let trig = trigger_sig
                .peek()
                .clone()
                .unwrap_or_else(|| fallback_trigger_id.clone());
            crate::foundation::browser::restore_focus_element_by_id(&trig);
        }
    };

    let mut resolved_style = style.clone().unwrap_or_default();
    if !is_open || *overlay.data_state() == crate::foundation::state::DataState::Closed {
        if !resolved_style.is_empty() && !resolved_style.ends_with(';') {
            resolved_style.push(';');
        }
        resolved_style.push_str(" pointer-events: none;");
    }

    rsx! {
        div {
            id: "{overlay_id}",
            class: class.as_deref().unwrap_or_default(),
            style: "{resolved_style}",
            "data-state": overlay.data_state().as_str(),
            "data-overlay": "true",
            aria_hidden: "true",
            onmounted: runtime.mount_overlay(),
            onclick: on_click,
            {children}
        }
    }
}

#[component]
pub fn DialogContent(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] aria_labelledby: Option<String>,
    #[props(default)] aria_describedby: Option<String>,
    #[props(default = false)] force_mount: bool,
    children: Element,
) -> Element {
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let should_render = force_mount || runtime.should_render_content();
    if !should_render {
        return rsx! {};
    }

    let content = if ctx.is_alert_dialog {
        runtime.dialog().content_with_role("alertdialog")
    } else {
        runtime.content()
    };
    let content_id = id.unwrap_or_else(|| content.id().to_string());
    runtime.register_content_id(&content_id);
    let labelledby = aria_labelledby.unwrap_or_else(|| content.aria_labelledby().to_string());
    let describedby = aria_describedby.unwrap_or_else(|| content.aria_describedby().to_string());

    let is_open = (ctx.open)();
    let mut open = ctx.open;
    let on_change = ctx.on_open_change;

    let trigger_sig = ctx.trigger_id;
    let fallback_trigger_id = runtime.relationships().trigger_id().to_string();

    let on_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Escape {
            open.set(false);
            if let Some(h) = on_change {
                h.call(false);
            }
            let trig = trigger_sig
                .peek()
                .clone()
                .unwrap_or_else(|| fallback_trigger_id.clone());
            crate::foundation::browser::restore_focus_element_by_id(&trig);
        }
    };

    let effect_content_id = content_id.clone();
    use_effect(use_reactive((&is_open,), move |(open_val,)| {
        if open_val {
            let cid = effect_content_id.clone();
            spawn(async move {
                crate::foundation::browser::focus_first_focusable(&cid, None);
            });
        }
    }));

    let mut resolved_style = style.clone().unwrap_or_default();
    if !is_open || *content.data_state() == crate::foundation::state::DataState::Closed {
        if !resolved_style.is_empty() && !resolved_style.ends_with(';') {
            resolved_style.push(';');
        }
        resolved_style.push_str(" pointer-events: none;");
    }

    rsx! {
        div {
            id: "{content_id}",
            class: class.as_deref().unwrap_or_default(),
            style: "{resolved_style}",
            role: content.role(),
            aria_modal: content.aria_modal(),
            aria_labelledby: "{labelledby}",
            aria_describedby: "{describedby}",
            "data-state": content.data_state().as_str(),
            tabindex: "-1",
            onmounted: runtime.mount_content(),
            onkeydown: on_keydown,
            onclick: move |evt: MouseEvent| evt.stop_propagation(),
            {children}
        }
    }
}

#[component]
pub fn DialogTitle(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<DialogContext>();
    let title_id = id.unwrap_or_else(|| ctx.runtime.read().title().id().to_string());
    rsx! {
        h2 {
            id: "{title_id}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn DialogDescription(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<DialogContext>();
    let desc_id = id.unwrap_or_else(|| ctx.runtime.read().description().id().to_string());
    rsx! {
        p {
            id: "{desc_id}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn DialogClose(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let close = runtime.close();
    let close_id = id.unwrap_or_else(|| close.id().to_string());
    let mut open = ctx.open;
    let on_change = ctx.on_open_change;

    let trigger_sig = ctx.trigger_id;
    let fallback_trigger_id = runtime.relationships().trigger_id().to_string();

    let on_click = move |_| {
        open.set(false);
        if let Some(h) = on_change {
            h.call(false);
        }
        let trig = trigger_sig
            .peek()
            .clone()
            .unwrap_or_else(|| fallback_trigger_id.clone());
        crate::foundation::browser::restore_focus_element_by_id(&trig);
    };

    rsx! {
        button {
            id: "{close_id}",
            r#type: "button",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": close.data_state().as_str(),
            onmounted: runtime.mount_close(),
            onclick: on_click,
            {children}
        }
    }
}

#[component]
pub fn AlertDialogAction(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] on_click: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let default_id = runtime.relationships().scope().qualify("action");
    let action_id = id.unwrap_or(default_id);
    let mut open = ctx.open;
    let on_change = ctx.on_open_change;

    let trigger_sig = ctx.trigger_id;
    let fallback_trigger_id = runtime.relationships().trigger_id().to_string();

    let click_handler = move |evt: MouseEvent| {
        open.set(false);
        if let Some(h) = on_change {
            h.call(false);
        }
        let trig = trigger_sig
            .peek()
            .clone()
            .unwrap_or_else(|| fallback_trigger_id.clone());
        crate::foundation::browser::restore_focus_element_by_id(&trig);
        if let Some(cb) = on_click {
            cb.call(evt);
        }
    };

    rsx! {
        button {
            id: "{action_id}",
            r#type: "button",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": runtime.data_state().as_str(),
            onmounted: runtime.mount_focus_target(action_id.clone()),
            onclick: click_handler,
            {children}
        }
    }
}

#[component]
pub fn AlertDialogCancel(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] on_click: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let default_id = runtime.relationships().scope().qualify("cancel");
    let cancel_id = id.unwrap_or(default_id);
    let mut open = ctx.open;
    let on_change = ctx.on_open_change;

    let trigger_sig = ctx.trigger_id;
    let fallback_trigger_id = runtime.relationships().trigger_id().to_string();

    let click_handler = move |evt: MouseEvent| {
        open.set(false);
        if let Some(h) = on_change {
            h.call(false);
        }
        let trig = trigger_sig
            .peek()
            .clone()
            .unwrap_or_else(|| fallback_trigger_id.clone());
        crate::foundation::browser::restore_focus_element_by_id(&trig);
        if let Some(cb) = on_click {
            cb.call(evt);
        }
    };

    rsx! {
        button {
            id: "{cancel_id}",
            r#type: "button",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": runtime.data_state().as_str(),
            onmounted: runtime.mount_focus_target(cancel_id.clone()),
            onclick: click_handler,
            {children}
        }
    }
}

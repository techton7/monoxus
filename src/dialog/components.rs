use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    keyboard::{is_activation_key, is_ime_composing_key},
    overlay::{OverlayPortalSpec, PortalHost, render_overlay_portal},
    shared::{ScopeHandle, use_stable_id},
};

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
    pub title_id: Signal<Option<String>>,
    pub description_id: Signal<Option<String>>,
    pub on_open_change: Option<EventHandler<bool>>,
    pub is_alert_dialog: bool,
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub open: Option<Signal<bool>>,
    #[props(default)]
    pub default_open: Option<bool>,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub mode: Option<DialogMode>,
    #[props(default)]
    pub open_focus: Option<DialogOpenFocusPolicy>,
    #[props(default)]
    pub close_focus: Option<DialogCloseFocusPolicy>,
    #[props(default)]
    pub scroll_lock: Option<DialogScrollLockPolicy>,
    #[props(default)]
    pub outside_interaction: Option<DialogOutsideInteractionPolicy>,
    #[props(default)]
    pub portal_host: Option<PortalHost>,
    #[props(default)]
    pub prevent_body_pointer_events: Option<bool>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn DialogRoot(props: DialogRootProps) -> Element {
    let DialogRootProps {
        id,
        open,
        default_open,
        on_open_change,
        mode,
        open_focus,
        close_focus,
        scroll_lock,
        outside_interaction,
        portal_host,
        prevent_body_pointer_events,
        attributes,
        children,
    } = props;
    let fallback_open = use_signal(|| default_open.unwrap_or(false));
    let open_signal = open.unwrap_or(fallback_open);
    let scope_id = use_stable_id(id.clone().map(Cow::Owned));
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
    if let Some(pbpe) = prevent_body_pointer_events {
        dialog
            .lifecycle_mut()
            .scroll_lock_policy_mut()
            .set_prevent_body_pointer_events(pbpe);
    }
    if let Some(oi) = outside_interaction {
        dialog.lifecycle_mut().set_outside_interaction_policy(oi);
    }
    if let Some(ph) = portal_host {
        dialog = dialog.with_portal_host(ph);
    }

    let runtime = use_dialog_runtime(dialog);
    let mut runtime_sig = use_signal(|| runtime.clone());
    if runtime_sig.peek().is_open() != runtime.is_open()
        || runtime_sig.peek().root().id() != runtime.root().id()
    {
        runtime_sig.set(runtime.clone());
    }

    let trigger_id = use_signal(|| None);
    let title_id = use_signal(|| None);
    let description_id = use_signal(|| None);
    use_context_provider(|| DialogContext {
        runtime: runtime_sig,
        open: open_signal,
        trigger_id,
        title_id,
        description_id,
        on_open_change,
        is_alert_dialog: false,
    });

    let mut was_open_sig = use_signal(|| open_signal());
    use_effect(move || {
        let is_curr = open_signal();
        let was_open = *was_open_sig.peek();
        if was_open && !is_curr {
            runtime_sig
                .peek()
                .restore_close_focus_with_fallback(trigger_id.peek().as_deref());
        }
        if was_open != is_curr {
            was_open_sig.set(is_curr);
        }
    });

    let attrs = runtime.root();
    let merged = merge_global_attributes(None, Some("display: contents;"), attributes);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": attrs.data_state().as_str(),
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn DialogTrigger(props: DialogTriggerProps) -> Element {
    let DialogTriggerProps {
        id,
        class,
        style,
        disabled,
        attributes,
        children,
    } = props;
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

    let runtime_for_click = runtime.clone();
    let on_click = move |_| {
        if disabled {
            return;
        }
        runtime_for_click.clear_pointer_dismiss();
        open.set(true);
        if let Some(h) = on_change {
            h.call(true);
        }
    };

    let controls = trigger.aria_controls();
    let data_disabled = crate::foundation::shared::data_disabled_attr(disabled);
    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        button {
            id: "{trig_id}",
            r#type: "button",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            disabled: disabled,
            "data-disabled": data_disabled,
            aria_haspopup: "dialog",
            aria_expanded: "{trigger.aria_expanded()}",
            aria_controls: controls,
            "data-state": trigger.data_state().as_str(),
            onmounted: runtime.mount_trigger(),
            onclick: on_click,
            ..merged.rest,
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
    let resolved_host = host.unwrap_or_else(|| runtime.portal().host().clone());
    let pid = runtime.relationships().scope().qualify("portal");

    render_overlay_portal(OverlayPortalSpec {
        portal_id: pid,
        host: resolved_host,
        should_render,
        force_mount,
        disabled: false,
        emit_status_attrs: false,
        children,
    })
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogOverlayProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn DialogOverlay(props: DialogOverlayProps) -> Element {
    let DialogOverlayProps {
        id,
        class,
        style,
        force_mount,
        attributes,
        children,
    } = props;
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
    let runtime_for_ptr = runtime.clone();
    let runtime_for_click = runtime.clone();

    let on_pointer_down = move |_| {
        if !ctx.is_alert_dialog && outside_policy.pointer_down_outside().dismisses() {
            runtime_for_ptr.mark_pointer_dismiss();
        }
    };

    let on_click = move |_| {
        if !*open.peek() {
            return;
        }
        if !ctx.is_alert_dialog && outside_policy.pointer_down_outside().dismisses() {
            runtime_for_click.mark_pointer_dismiss();
            open.set(false);
            if let Some(h) = on_change {
                h.call(false);
            }
            runtime_for_click.restore_close_focus_with_fallback(trigger_sig.peek().as_deref());
        }
    };

    let mut resolved_style = style.clone().unwrap_or_default();
    if !is_open || *overlay.data_state() == crate::foundation::state::DataState::Closed {
        if !resolved_style.is_empty() && !resolved_style.ends_with(';') {
            resolved_style.push(';');
        }
        resolved_style.push_str(" pointer-events: none;");
    } else if runtime.should_force_pointer_events_auto()
        && !resolved_style.contains("pointer-events:")
    {
        if !resolved_style.is_empty() && !resolved_style.ends_with(';') {
            resolved_style.push(';');
        }
        resolved_style.push_str(" pointer-events: auto;");
    }

    let merged = merge_global_attributes(class.as_deref(), Some(resolved_style.as_str()), attributes);

    rsx! {
        div {
            id: "{overlay_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": overlay.data_state().as_str(),
            "data-overlay": "true",
            aria_hidden: "true",
            onmounted: runtime.mount_overlay(),
            onpointerdown: on_pointer_down,
            onclick: on_click,
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub prevent_body_pointer_events: Option<bool>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn DialogContent(props: DialogContentProps) -> Element {
    let DialogContentProps {
        id,
        class,
        style,
        aria_labelledby,
        aria_describedby,
        force_mount,
        prevent_body_pointer_events,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let should_render = force_mount || runtime.should_render_content();

    let content = if ctx.is_alert_dialog {
        runtime.dialog().content_with_role("alertdialog")
    } else {
        runtime.content()
    };
    let content_id = id.unwrap_or_else(|| content.id().to_string());
    let overlay_id = runtime.overlay_element_id_reactive();
    let portal_id = runtime.relationships().scope().qualify("portal");

    let latest_cid = use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(content_id.clone())));
    {
        let mut prev = latest_cid.borrow_mut();
        if *prev != content_id {
            runtime.unregister_content_id(&prev);
            *prev = content_id.clone();
        }
    }

    if should_render {
        runtime.register_content_id(&content_id);
        if prevent_body_pointer_events.is_some() {
            runtime.set_prevent_body_pointer_events_override(prevent_body_pointer_events);
        }
    } else {
        runtime.unregister_content_id(&content_id);
    }

    let parent_branch = try_consume_context::<crate::foundation::overlay::OverlayBranchContext>();
    crate::foundation::overlay::use_register_parent_overlay_branches(
        parent_branch.clone(),
        should_render,
        vec![content_id.clone(), overlay_id, portal_id],
    );

    let dismiss_handler = use_hook(move || {
        let mut open_for_dismiss = ctx.open;
        let on_change_for_dismiss = ctx.on_open_change;
        let trigger_sig_for_dismiss = ctx.trigger_id;
        let runtime_sig = ctx.runtime;
        EventHandler::new(move |was_pointer: bool| {
            let is_currently_open = open_for_dismiss.try_peek().map(|o| *o).unwrap_or(false);
            if !is_currently_open {
                return;
            }
            let rt_close = runtime_sig.peek().clone();
            if was_pointer {
                rt_close.mark_pointer_dismiss();
            } else {
                rt_close.clear_pointer_dismiss();
            }
            if let Ok(mut o) = open_for_dismiss.try_write() {
                *o = false;
            }
            if let Some(h) = on_change_for_dismiss {
                h.call(false);
            }
            let fallback_tid = trigger_sig_for_dismiss
                .try_peek()
                .ok()
                .and_then(|t| t.clone());
            rt_close.restore_close_focus_with_fallback(fallback_tid.as_deref());
        })
    });
    runtime.set_dismiss_callback(ctx.is_alert_dialog, Some(dismiss_handler));

    let cid_for_drop = latest_cid.clone();
    let runtime_for_drop = runtime.clone();
    let had_pbpe_override = prevent_body_pointer_events.is_some();
    use_drop(move || {
        let current_cid = cid_for_drop.borrow().clone();
        runtime_for_drop.set_dismiss_callback(false, None);
        runtime_for_drop.unregister_content_id(&current_cid);
        if had_pbpe_override {
            runtime_for_drop.set_prevent_body_pointer_events_override(None);
        }
    });

    let is_open = (ctx.open)();
    let effect_content_id = content_id.clone();
    let runtime_for_effect = runtime.clone();
    let last_focused_open_cid =
        use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(Option::<String>::None)));
    let focused_cid_effect = last_focused_open_cid.clone();
    use_effect(use_reactive(
        (&should_render, &is_open, &effect_content_id),
        move |(render_val, open_val, cid)| {
            if !render_val || !open_val {
                *focused_cid_effect.borrow_mut() = None;
                return;
            }
            let needs_open_focus = focused_cid_effect.borrow().as_deref() != Some(cid.as_str());
            if needs_open_focus {
                *focused_cid_effect.borrow_mut() = Some(cid.clone());
                runtime_for_effect.apply_open_focus_for_content(&cid);
            }
        },
    ));

    let runtime_for_branch = runtime.clone();
    let parent_for_reg = parent_branch.clone();
    let parent_for_unreg = parent_branch;
    use_context_provider(|| {
        let rt_reg = runtime_for_branch.clone();
        let rt_unreg = runtime_for_branch.clone();
        crate::foundation::overlay::OverlayBranchContext::new(
            move |branch: String| {
                rt_reg.register_branch(branch.clone());
                if let Some(ref pb) = parent_for_reg {
                    pb.register_branch(branch);
                }
            },
            move |branch: String| {
                rt_unreg.unregister_branch(&branch);
                if let Some(ref pb) = parent_for_unreg {
                    pb.unregister_branch(branch);
                }
            },
        )
    });

    if !should_render {
        return rsx! {};
    }

    let labelledby = aria_labelledby.or_else(|| ctx.title_id.read().clone());
    let describedby = aria_describedby.or_else(|| ctx.description_id.read().clone());

    let runtime_for_key = runtime.clone();
    let on_keydown = move |evt: KeyboardEvent| {
        if is_ime_composing_key(&evt) {
            return;
        }
        if runtime_for_key.handle_tab_keydown(&evt) {
            return;
        }
        if runtime_for_key.handle_escape_keydown(&evt) {
            evt.prevent_default();
        }
    };

    let mut resolved_style = style.unwrap_or_default();
    if !is_open || *content.data_state() == crate::foundation::state::DataState::Closed {
        if !resolved_style.is_empty() && !resolved_style.ends_with(';') {
            resolved_style.push(';');
        }
        resolved_style.push_str(" pointer-events: none;");
    } else if runtime.should_force_pointer_events_auto()
        && !resolved_style.contains("pointer-events:")
    {
        if !resolved_style.is_empty() && !resolved_style.ends_with(';') {
            resolved_style.push(';');
        }
        resolved_style.push_str(" pointer-events: auto;");
    }

    let merged = merge_global_attributes(class.as_deref(), Some(resolved_style.as_str()), attributes);

    rsx! {
        div {
            id: "{content_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            role: content.role(),
            aria_modal: content.aria_modal(),
            aria_labelledby: labelledby.as_deref(),
            aria_describedby: describedby.as_deref(),
            "data-state": content.data_state().as_str(),
            tabindex: "-1",
            onmounted: runtime.mount_content(),
            onkeydown: on_keydown,
            onclick: move |evt: MouseEvent| evt.stop_propagation(),
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogTitleProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn DialogTitle(props: DialogTitleProps) -> Element {
    let DialogTitleProps {
        id,
        class,
        style,
        attributes,
        children,
    } = props;
    let mut ctx = use_context::<DialogContext>();
    let title_id = id.unwrap_or_else(|| ctx.runtime.read().title().id().to_string());
    if ctx.title_id.peek().as_deref() != Some(title_id.as_str()) {
        ctx.title_id.set(Some(title_id.clone()));
    }
    let tid_drop = title_id.clone();
    let mut title_sig = ctx.title_id;
    use_drop(move || {
        if let Ok(mut sig) = title_sig.try_write()
            && sig.as_deref() == Some(tid_drop.as_str())
        {
            *sig = None;
        }
    });

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        h2 {
            id: "{title_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogDescriptionProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn DialogDescription(props: DialogDescriptionProps) -> Element {
    let DialogDescriptionProps {
        id,
        class,
        style,
        attributes,
        children,
    } = props;
    let mut ctx = use_context::<DialogContext>();
    let desc_id = id.unwrap_or_else(|| ctx.runtime.read().description().id().to_string());
    if ctx.description_id.peek().as_deref() != Some(desc_id.as_str()) {
        ctx.description_id.set(Some(desc_id.clone()));
    }
    let did_drop = desc_id.clone();
    let mut desc_sig = ctx.description_id;
    use_drop(move || {
        if let Ok(mut sig) = desc_sig.try_write()
            && sig.as_deref() == Some(did_drop.as_str())
        {
            *sig = None;
        }
    });

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        p {
            id: "{desc_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogCloseProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn DialogClose(props: DialogCloseProps) -> Element {
    let DialogCloseProps {
        id,
        class,
        style,
        disabled,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let close = runtime.close();
    let stable_fallback = use_stable_id(None);
    let close_id = id.unwrap_or_else(|| format!("{stable_fallback}-close"));
    let mut open = ctx.open;
    let on_change = ctx.on_open_change;

    let trigger_sig = ctx.trigger_id;
    let runtime_for_ptr = runtime.clone();
    let runtime_for_key = runtime.clone();
    let runtime_for_click = runtime.clone();

    let on_click = move |_| {
        if disabled {
            return;
        }
        open.set(false);
        if let Some(h) = on_change {
            h.call(false);
        }
        runtime_for_click.restore_close_focus_with_fallback(trigger_sig.peek().as_deref());
    };

    let data_disabled = crate::foundation::shared::data_disabled_attr(disabled);
    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        button {
            id: "{close_id}",
            r#type: "button",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            disabled: disabled,
            "data-disabled": data_disabled,
            "data-state": close.data_state().as_str(),
            onmounted: runtime.mount_close(),
            onpointerdown: move |_| {
                if !disabled {
                    runtime_for_ptr.mark_pointer_dismiss();
                }
            },
            onkeydown: move |evt: KeyboardEvent| {
                if !disabled && !is_ime_composing_key(&evt) && is_activation_key(&evt.key()) {
                    runtime_for_key.clear_pointer_dismiss();
                }
            },
            onclick: on_click,
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AlertDialogActionProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    let AlertDialogActionProps {
        id,
        class,
        style,
        on_click,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let default_id = runtime.relationships().scope().qualify("action");
    let action_id = id.unwrap_or(default_id);
    let mut open = ctx.open;
    let on_change = ctx.on_open_change;

    let trigger_sig = ctx.trigger_id;
    let runtime_for_ptr = runtime.clone();
    let runtime_for_key = runtime.clone();
    let runtime_for_click = runtime.clone();

    let click_handler = move |evt: MouseEvent| {
        open.set(false);
        if let Some(h) = on_change {
            h.call(false);
        }
        runtime_for_click.restore_close_focus_with_fallback(trigger_sig.peek().as_deref());
        if let Some(cb) = on_click {
            cb.call(evt);
        }
    };

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        button {
            id: "{action_id}",
            r#type: "button",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": runtime.data_state().as_str(),
            onmounted: runtime.mount_focus_target(action_id.clone()),
            onpointerdown: move |_| runtime_for_ptr.mark_pointer_dismiss(),
            onkeydown: move |evt: KeyboardEvent| {
                if !is_ime_composing_key(&evt) && is_activation_key(&evt.key()) {
                    runtime_for_key.clear_pointer_dismiss();
                }
            },
            onclick: click_handler,
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AlertDialogCancelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    let AlertDialogCancelProps {
        id,
        class,
        style,
        on_click,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read();
    let default_id = runtime.relationships().scope().qualify("cancel");
    let cancel_id = id.unwrap_or(default_id);
    if ctx.is_alert_dialog {
        runtime.set_open_focus_target(&cancel_id);
    }
    let mut open = ctx.open;
    let on_change = ctx.on_open_change;

    let trigger_sig = ctx.trigger_id;
    let runtime_for_ptr = runtime.clone();
    let runtime_for_key = runtime.clone();
    let runtime_for_click = runtime.clone();

    let click_handler = move |evt: MouseEvent| {
        open.set(false);
        if let Some(h) = on_change {
            h.call(false);
        }
        runtime_for_click.restore_close_focus_with_fallback(trigger_sig.peek().as_deref());
        if let Some(cb) = on_click {
            cb.call(evt);
        }
    };

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        button {
            id: "{cancel_id}",
            r#type: "button",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": runtime.data_state().as_str(),
            onmounted: runtime.mount_focus_target(cancel_id.clone()),
            onpointerdown: move |_| runtime_for_ptr.mark_pointer_dismiss(),
            onkeydown: move |evt: KeyboardEvent| {
                if !is_ime_composing_key(&evt) && is_activation_key(&evt.key()) {
                    runtime_for_key.clear_pointer_dismiss();
                }
            },
            onclick: click_handler,
            ..merged.rest,
            {children}
        }
    }
}

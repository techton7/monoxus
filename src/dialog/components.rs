use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{
        collect_user_attrs, merge_props, push_attr_opt, push_bool_data_attr, push_event_handler,
    },
    keyboard::{is_activation_key, is_ime_composing_key},
    overlay::{DismissBehavior, OverlayPortalSpec, PortalHost, render_overlay_portal},
    shared::{ScopeHandle, use_stable_id},
    state::{ControllableSignal, DataState, use_controllable_signal},
};

use super::{
    runtime::{DialogRuntime, use_dialog_runtime},
    state::Dialog,
    types::{
        CloseAutoFocusEvent, DialogCloseFocusPolicy, DialogMode, DialogOpenFocusPolicy,
        DialogOutsideInteractionPolicy, DialogScrollLockPolicy, EscapeKeyDownEvent,
        InteractOutsideEvent, OpenAutoFocusEvent, PointerDownOutsideEvent,
    },
};

#[derive(Clone, Copy)]
pub struct DialogContext {
    pub runtime: Signal<DialogRuntime>,
    pub open: Signal<bool>,
    pub open_ctl: ControllableSignal<bool>,
    pub trigger_id: Signal<Option<String>>,
    pub title_id: Signal<Option<String>>,
    pub description_id: Signal<Option<String>>,
    pub on_open_change: Option<EventHandler<bool>>,
    pub is_alert_dialog: bool,
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default)]
    pub mode: Option<DialogMode>,
    pub children: Element,
}

#[component]
pub fn DialogRoot(props: DialogRootProps) -> Element {
    let DialogRootProps {
        id,
        open,
        default_open,
        on_open_change,
        on_open_change_complete,
        mode,
        children,
    } = props;
    let open_ctl = use_controllable_signal(open, default_open, on_open_change);
    let open_signal = open_ctl.signal();
    let scope_id = use_stable_id(id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("dialog").child(scope_id);

    let mut dialog = Dialog::new(scope, open_signal());
    if let Some(m) = mode {
        dialog.lifecycle_mut().set_mode(m);
    }

    let runtime = use_dialog_runtime(dialog);
    runtime.set_on_open_change(on_open_change);
    runtime.set_on_open_change_complete(on_open_change_complete);
    let mut runtime_sig = use_signal(|| runtime.clone());
    if runtime_sig.peek().dialog() != runtime.dialog() {
        runtime_sig.set(runtime.clone());
    }

    let trigger_id = use_signal(|| None);
    let title_id = use_signal(|| None);
    let description_id = use_signal(|| None);
    use_context_provider(|| DialogContext {
        runtime: runtime_sig,
        open: open_signal,
        open_ctl,
        trigger_id,
        title_id,
        description_id,
        on_open_change,
        is_alert_dialog: false,
    });

    rsx! {
        {children}
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogTriggerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn DialogTrigger(props: DialogTriggerProps) -> Element {
    let DialogTriggerProps {
        id,
        class,
        style,
        disabled,
        on_click,
        on_key_down,
        on_mounted,
        child,
        attributes,
        children,
    } = props;
    let mut ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read().clone();
    let trigger = runtime.trigger();
    let trig_id = id.unwrap_or_else(|| trigger.id().to_string());
    runtime.register_trigger_id(&trig_id);
    let open_ctl = ctx.open_ctl;

    if ctx.trigger_id.peek().as_deref() != Some(trig_id.as_str()) {
        ctx.trigger_id.set(Some(trig_id.clone()));
    }

    let controls = trigger.aria_controls().map(str::to_owned);
    let aria_expanded = if trigger.aria_expanded() {
        "true"
    } else {
        "false"
    };
    let data_state = trigger.data_state().as_str();

    let runtime_for_click = runtime.clone();
    let mut mount_cb = runtime.mount_trigger();

    let mut base = Vec::with_capacity(11);
    base.push(Attribute::new("id", trig_id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    if disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_bool_data_attr(&mut base, "data-disabled", disabled);
    base.push(Attribute::new("aria-haspopup", "dialog", None, false));
    base.push(Attribute::new("aria-expanded", aria_expanded, None, false));
    push_attr_opt(&mut base, "aria-controls", controls);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            if disabled {
                evt.prevent_default();
                return;
            }
            if evt.default_action_enabled() {
                runtime_for_click.clear_pointer_dismiss();
                open_ctl.set(true);
            }
        })),
    );

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);
    if !disabled {
        push_event_handler(&mut user, "onclick", on_click);
    }
    push_event_handler(&mut user, "onkeydown", on_key_down);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogPortalProps {
    #[props(default)]
    pub host: Option<PortalHost>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub force_mount: bool,
    pub children: Element,
}

#[component]
pub fn DialogPortal(props: DialogPortalProps) -> Element {
    let DialogPortalProps {
        host,
        disabled,
        force_mount,
        children,
    } = props;
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
        disabled,
        emit_status_attrs: false,
        children,
    })
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogOverlayProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub keep_mounted: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
        keep_mounted,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read().clone();
    runtime.set_overlay_keep_mounted(keep_mounted);

    let should_render = force_mount || runtime.should_render_overlay();
    if !should_render {
        return rsx! {};
    }

    let overlay = runtime.overlay();
    let overlay_id = id.unwrap_or_else(|| overlay.id().to_string());
    runtime.register_overlay_id(&overlay_id);
    let is_open = (ctx.open)();
    let open = ctx.open;
    let open_ctl = ctx.open_ctl;
    let is_alert = ctx.is_alert_dialog;
    let data_state = overlay.data_state().as_str();
    let is_closed_state = *overlay.data_state() == DataState::Closed;

    let runtime_for_ptr = runtime.clone();
    let runtime_for_click = runtime.clone();
    let mut mount_cb = runtime.mount_overlay();

    let base_pointer_style: Option<&'static str> = if !is_open || is_closed_state {
        Some("pointer-events: none;")
    } else if runtime.should_force_pointer_events_auto()
        && !style
            .as_deref()
            .is_some_and(|s| s.contains("pointer-events:"))
    {
        Some("pointer-events: auto;")
    } else {
        None
    };

    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("id", overlay_id, None, false));
    push_attr_opt(&mut base, "style", base_pointer_style);
    base.push(Attribute::new("data-state", data_state, None, false));
    base.push(Attribute::new("data-overlay", "true", None, false));
    base.push(Attribute::new("aria-hidden", "true", None, false));
    push_attr_opt(
        &mut base,
        "data-starting-style",
        runtime.overlay_data_starting_style(),
    );
    push_attr_opt(
        &mut base,
        "data-ending-style",
        runtime.overlay_data_ending_style(),
    );
    if !force_mount && runtime.is_overlay_hidden() {
        base.push(Attribute::new("hidden", true, None, false));
    }
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new(move |_evt: PointerEvent| {
            if runtime_for_ptr.should_dismiss_on_outside_pointer(is_alert) {
                runtime_for_ptr.mark_pointer_dismiss();
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            if !evt.default_action_enabled() || !*open.peek() {
                return;
            }
            if runtime_for_click.should_dismiss_on_outside_pointer(is_alert) {
                runtime_for_click.mark_pointer_dismiss();
                open_ctl.set(false);
            }
        })),
    );

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogContentProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub keep_mounted: bool,
    #[props(default)]
    pub open_focus: Option<DialogOpenFocusPolicy>,
    #[props(default)]
    pub close_focus: Option<DialogCloseFocusPolicy>,
    #[props(default)]
    pub scroll_lock: Option<DialogScrollLockPolicy>,
    #[props(default)]
    pub outside_interaction: Option<DialogOutsideInteractionPolicy>,
    #[props(default)]
    pub prevent_body_pointer_events: Option<bool>,
    #[props(default)]
    pub escape_key_down_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub interact_outside_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
        keep_mounted,
        open_focus,
        close_focus,
        scroll_lock,
        outside_interaction,
        prevent_body_pointer_events,
        escape_key_down_behavior,
        interact_outside_behavior,
        on_open_auto_focus,
        on_close_auto_focus,
        on_escape_key_down,
        on_pointer_down_outside,
        on_interact_outside,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read().clone();

    runtime.set_content_keep_mounted(keep_mounted);
    runtime.set_content_policy_overrides(
        open_focus,
        close_focus,
        scroll_lock,
        outside_interaction,
        prevent_body_pointer_events,
        escape_key_down_behavior,
        interact_outside_behavior,
    );
    runtime.set_content_event_handlers(
        on_open_auto_focus,
        on_close_auto_focus,
        on_escape_key_down,
        on_pointer_down_outside,
        on_interact_outside,
    );

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
    } else {
        runtime.unregister_content_id(&content_id);
    }

    let is_open = (ctx.open)();
    let parent_branch = try_consume_context::<crate::foundation::overlay::OverlayBranchContext>();
    crate::foundation::overlay::use_register_parent_overlay_branches(
        parent_branch.clone(),
        is_open,
        vec![content_id.clone(), overlay_id, portal_id],
    );

    let dismiss_handler = use_hook(move || {
        let open_for_dismiss = ctx.open;
        let open_ctl_for_dismiss = ctx.open_ctl;
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
            open_ctl_for_dismiss.set(false);
        })
    });
    runtime.set_dismiss_callback(ctx.is_alert_dialog, Some(dismiss_handler));

    let cid_for_drop = latest_cid.clone();
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        let current_cid = cid_for_drop.borrow().clone();
        runtime_for_drop.set_dismiss_callback(false, None);
        runtime_for_drop.unregister_content_id(&current_cid);
        runtime_for_drop.set_content_keep_mounted(false);
        runtime_for_drop.set_content_policy_overrides(None, None, None, None, None, None, None);
        runtime_for_drop.set_content_event_handlers(None, None, None, None, None);
    });

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
    let role = content.role();
    let aria_modal = if content.aria_modal() {
        "true"
    } else {
        "false"
    };
    let data_state = content.data_state().as_str();
    let is_closed_state = *content.data_state() == DataState::Closed;

    let base_pointer_style: Option<&'static str> = if !is_open || is_closed_state {
        Some("pointer-events: none;")
    } else if runtime.should_force_pointer_events_auto()
        && !style
            .as_deref()
            .is_some_and(|s| s.contains("pointer-events:"))
    {
        Some("pointer-events: auto;")
    } else {
        None
    };

    let runtime_for_key = runtime.clone();
    let mut mount_cb = runtime.mount_content();

    let mut base = Vec::with_capacity(15);
    base.push(Attribute::new("id", content_id, None, false));
    push_attr_opt(&mut base, "style", base_pointer_style);
    base.push(Attribute::new("role", role, None, false));
    base.push(Attribute::new("aria-modal", aria_modal, None, false));
    push_attr_opt(&mut base, "aria-labelledby", labelledby);
    push_attr_opt(&mut base, "aria-describedby", describedby);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_attr_opt(
        &mut base,
        "data-starting-style",
        runtime.content_data_starting_style(),
    );
    push_attr_opt(
        &mut base,
        "data-ending-style",
        runtime.content_data_ending_style(),
    );
    if !force_mount && runtime.is_content_hidden() {
        base.push(Attribute::new("hidden", true, None, false));
    }
    base.push(Attribute::new("tabindex", "-1", None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onkeydown",
        Some(EventHandler::new(move |evt: KeyboardEvent| {
            if is_ime_composing_key(&evt) {
                return;
            }
            if runtime_for_key.handle_tab_keydown(&evt) {
                return;
            }
            if runtime_for_key.handle_escape_keydown(&evt) {
                evt.prevent_default();
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            evt.stop_propagation();
        })),
    );

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogTitleProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn DialogTitle(props: DialogTitleProps) -> Element {
    let DialogTitleProps {
        id,
        class,
        style,
        child,
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

    let base = vec![Attribute::new("id", title_id, None, false)];
    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        h2 {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogDescriptionProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn DialogDescription(props: DialogDescriptionProps) -> Element {
    let DialogDescriptionProps {
        id,
        class,
        style,
        child,
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

    let base = vec![Attribute::new("id", desc_id, None, false)];
    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        p {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct DialogCloseProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn DialogClose(props: DialogCloseProps) -> Element {
    let DialogCloseProps {
        id,
        class,
        style,
        disabled,
        on_click,
        on_key_down,
        on_mounted,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read().clone();
    let close = runtime.close();
    let stable_fallback = use_stable_id(None);
    let close_id = id.unwrap_or_else(|| format!("{stable_fallback}-close"));
    let open_ctl = ctx.open_ctl;
    let data_state = close.data_state().as_str();

    let runtime_for_ptr = runtime.clone();
    let runtime_for_key = runtime.clone();
    let mut mount_cb = runtime.mount_close();

    let mut base = Vec::with_capacity(10);
    base.push(Attribute::new("id", close_id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    if disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_bool_data_attr(&mut base, "data-disabled", disabled);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new(move |_evt: PointerEvent| {
            if !disabled {
                runtime_for_ptr.mark_pointer_dismiss();
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onkeydown",
        Some(EventHandler::new(move |evt: KeyboardEvent| {
            if !disabled && !is_ime_composing_key(&evt) && is_activation_key(&evt.key()) {
                runtime_for_key.clear_pointer_dismiss();
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            if disabled {
                evt.prevent_default();
                return;
            }
            if evt.default_action_enabled() {
                open_ctl.set(false);
            }
        })),
    );

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);
    if !disabled {
        push_event_handler(&mut user, "onclick", on_click);
    }
    push_event_handler(&mut user, "onkeydown", on_key_down);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AlertDialogActionProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    let AlertDialogActionProps {
        id,
        class,
        style,
        on_click,
        on_key_down,
        on_mounted,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read().clone();
    let default_id = runtime.relationships().scope().qualify("action");
    let action_id = id.unwrap_or(default_id);
    let open_ctl = ctx.open_ctl;
    let data_state = runtime.data_state().as_str().to_owned();

    let runtime_for_ptr = runtime.clone();
    let runtime_for_key = runtime.clone();
    let mut mount_cb = runtime.mount_focus_target(action_id.clone());

    let mut base = Vec::with_capacity(8);
    base.push(Attribute::new("id", action_id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    base.push(Attribute::new("data-state", data_state, None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new(move |_evt: PointerEvent| {
            runtime_for_ptr.mark_pointer_dismiss();
        })),
    );
    push_event_handler(
        &mut base,
        "onkeydown",
        Some(EventHandler::new(move |evt: KeyboardEvent| {
            if !is_ime_composing_key(&evt) && is_activation_key(&evt.key()) {
                runtime_for_key.clear_pointer_dismiss();
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            if evt.default_action_enabled() {
                open_ctl.set(false);
            }
        })),
    );

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);
    push_event_handler(&mut user, "onclick", on_click);
    push_event_handler(&mut user, "onkeydown", on_key_down);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AlertDialogCancelProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    let AlertDialogCancelProps {
        id,
        class,
        style,
        on_click,
        on_key_down,
        on_mounted,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read().clone();
    let default_id = runtime.relationships().scope().qualify("cancel");
    let cancel_id = id.unwrap_or(default_id);
    if ctx.is_alert_dialog {
        runtime.set_open_focus_target(&cancel_id);
    }
    let open_ctl = ctx.open_ctl;
    let data_state = runtime.data_state().as_str().to_owned();

    let runtime_for_ptr = runtime.clone();
    let runtime_for_key = runtime.clone();
    let mut mount_cb = runtime.mount_focus_target(cancel_id.clone());

    let mut base = Vec::with_capacity(8);
    base.push(Attribute::new("id", cancel_id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    base.push(Attribute::new("data-state", data_state, None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new(move |_evt: PointerEvent| {
            runtime_for_ptr.mark_pointer_dismiss();
        })),
    );
    push_event_handler(
        &mut base,
        "onkeydown",
        Some(EventHandler::new(move |evt: KeyboardEvent| {
            if !is_ime_composing_key(&evt) && is_activation_key(&evt.key()) {
                runtime_for_key.clear_pointer_dismiss();
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            if evt.default_action_enabled() {
                open_ctl.set(false);
            }
        })),
    );

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);
    push_event_handler(&mut user, "onclick", on_click);
    push_event_handler(&mut user, "onkeydown", on_key_down);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}

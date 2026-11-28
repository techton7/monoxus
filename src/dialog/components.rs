use std::{
    borrow::Cow,
    collections::HashMap,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::{core::AttributeValue, prelude::*};
use ui_sdk::{
    agent::{ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, SnapshotValue},
    kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource},
};

use crate::{
    agent::{
        ActionDispatcher, DispatchExecResult, SnapshotReader, SurfaceMetaDefaults,
        current_change_context, use_agent_surface,
    },
    foundation::{
        browser::click_element_by_id,
        compose::{
            collect_user_attrs, merge_props, push_attr_opt, push_bool_data_attr, push_event_handler,
        },
        config::use_portal_host_context,
        keyboard::{is_activation_key, is_ime_composing_key},
        overlay::{
            AnyDialogHandle, DialogHandleProp, DismissBehavior, DismissEvent, OverlayPayload,
            OverlayPortalSpec, PortalHost, render_overlay_portal, use_overlay_handle_version,
        },
        shared::{ScopeHandle, use_stable_id},
        state::{
            ControllableConfig, ControllableSignal, DataState, sync_signal,
            use_controllable_signal_config,
        },
    },
};

use super::{
    runtime::{DialogRuntime, use_dialog_runtime},
    state::Dialog,
    types::{
        CloseAutoFocusEvent, DialogCloseFocusPolicy, DialogMode, DialogNestedState,
        DialogOpenFocusPolicy, DialogOutsideInteractionPolicy, DialogScrollLockPolicy,
        EscapeKeyDownEvent, InteractOutsideEvent, OpenAutoFocusEvent, PointerDownOutsideEvent,
    },
};

static NEXT_DIALOG_NESTING_ID: AtomicU64 = AtomicU64::new(1);

/// Context tracking nested open dialog counts across `DialogRoot` and `AlertDialogRoot`
/// hierarchies (`S2-LD-08`).
#[derive(Clone, Copy)]
pub struct DialogNestingContext {
    pub dialog_id: u64,
    pub child_counts: Signal<HashMap<u64, usize>>,
}

pub(crate) fn use_dialog_nesting(is_open: bool) -> (bool, usize) {
    // Capture parent BEFORE providing our own context so re-renders never read self.
    let parent_nesting = use_hook(|| try_consume_context::<DialogNestingContext>());
    let own_dialog_id = use_hook(|| NEXT_DIALOG_NESTING_ID.fetch_add(1, Ordering::Relaxed));
    let own_child_counts = use_signal(HashMap::<u64, usize>::new);
    use_context_provider(|| DialogNestingContext {
        dialog_id: own_dialog_id,
        child_counts: own_child_counts,
    });

    let is_nested = parent_nesting.is_some();
    let own_nested_open_count: usize = own_child_counts.read().values().copied().sum();
    let mut reported_open = use_hook(|| CopyValue::new(false));

    if let Some(mut parent) = parent_nesting {
        if is_open {
            let report_count = 1usize.saturating_add(own_nested_open_count);
            let needs_update = parent
                .child_counts
                .try_peek()
                .is_ok_and(|map| map.get(&own_dialog_id).copied() != Some(report_count));
            if needs_update
                && let Ok(mut map) = parent.child_counts.try_write()
            {
                map.insert(own_dialog_id, report_count);
            }
            if reported_open.try_read().is_ok_and(|flag| !*flag) {
                reported_open.set(true);
            }
        } else if reported_open.try_read().is_ok_and(|flag| *flag) {
            let has_entry = parent
                .child_counts
                .try_peek()
                .is_ok_and(|map| map.contains_key(&own_dialog_id));
            if has_entry
                && let Ok(mut map) = parent.child_counts.try_write()
            {
                map.remove(&own_dialog_id);
            }
            reported_open.set(false);
        }
    }

    use_drop(move || {
        let was_reported = reported_open
            .try_write()
            .map(|mut flag| {
                let prev = *flag;
                *flag = false;
                prev
            })
            .unwrap_or(false);
        if was_reported
            && let Some(mut parent) = parent_nesting
            && let Ok(mut map) = parent.child_counts.try_write()
        {
            map.remove(&own_dialog_id);
        }
    });

    (is_nested, own_nested_open_count)
}

pub(crate) fn use_dialog_handle_attachment(
    handle: Option<AnyDialogHandle>,
    root_id: String,
    is_open: bool,
    open_signal: Signal<bool>,
    open_ctl: ControllableSignal<bool>,
    runtime: &DialogRuntime,
    runtime_sig: Signal<DialogRuntime>,
) {
    let _handle_version = use_overlay_handle_version(handle.as_ref());
    let mut attached: CopyValue<Option<(AnyDialogHandle, String)>> =
        use_hook(|| CopyValue::new(None));

    let needs_rebind = {
        let current = attached.try_read().ok().and_then(|g| g.clone());
        match (&current, &handle) {
            (Some((prev_h, prev_id)), Some(next_h)) => prev_h != next_h || prev_id != &root_id,
            (Some(_), None) | (None, Some(_)) => true,
            (None, None) => false,
        }
    };

    if needs_rebind {
        if let Some((prev_h, prev_id)) = attached.try_write().ok().and_then(|mut g| g.take()) {
            prev_h.detach_root(&prev_id);
        }
        if let Some(ref next_h) = handle {
            let open_sig_cb = open_signal;
            let open_ctl_cb = open_ctl;
            let runtime_sig_cb = runtime_sig;
            next_h.attach_root(
                root_id.clone(),
                Rc::new(move || open_sig_cb.try_peek().map(|v| *v).unwrap_or(false)),
                Rc::new(move |next: bool| {
                    if let Ok(rt) = runtime_sig_cb.try_peek() {
                        rt.clear_pointer_dismiss();
                    }
                    let reason = if next {
                        ChangeReason::Open
                    } else {
                        ChangeReason::Close
                    };
                    let ctx = ChangeContext::new(reason, ChangeSource::Programmatic);
                    open_ctl_cb.set_with_context(next, ctx);
                }),
            );
            attached.set(Some((next_h.clone(), root_id.clone())));
        }
    }

    if let Some(ref h) = handle {
        h.sync_open_state_for_root(&root_id, is_open);
        if let Some(active_tid) = h.active_trigger_id() {
            runtime.register_trigger_id(&active_tid);
            runtime.set_restore_focus_target(Some(active_tid));
        } else if h.opened_without_trigger() {
            runtime.set_restore_focus_target(None);
        }
    }

    use_drop(move || {
        if let Some((prev_h, prev_id)) = attached.try_write().ok().and_then(|mut g| g.take()) {
            prev_h.detach_root(&prev_id);
        }
    });
}

pub(crate) fn use_dialog_agent_surface(
    kind: &'static str,
    default_label: String,
    agent: Option<&AgentHints>,
    open_signal: Signal<bool>,
    open_ctl: ControllableSignal<bool>,
) {
    let is_open = open_signal.try_peek().map(|v| *v).unwrap_or(false);
    use_agent_surface(
        agent,
        SurfaceMetaDefaults::default(),
        || {
            AgentSurface::new(kind, default_label)
                .with_snapshot(
                    ComponentSnapshot::new().with_field("open", SnapshotValue::Bool(is_open)),
                )
                .with_action(ActionDescriptor::new("open", "Open dialog", false))
                .with_action(ActionDescriptor::new("close", "Close dialog", false))
        },
        || {
            let dispatcher: ActionDispatcher = Rc::new(move |dispatch| match dispatch.action.as_str()
            {
                "open" => {
                    let ctx = current_change_context()
                        .unwrap_or_else(|| ChangeContext::agent(ChangeReason::Open));
                    let outcome = open_ctl.set_with_context(true, ctx);
                    DispatchExecResult::ok()
                        .with_canceled(outcome.is_canceled())
                        .with_expected("open", SnapshotValue::Bool(true))
                }
                "close" => {
                    let ctx = current_change_context()
                        .unwrap_or_else(|| ChangeContext::agent(ChangeReason::Close));
                    let outcome = open_ctl.set_with_context(false, ctx);
                    DispatchExecResult::ok()
                        .with_canceled(outcome.is_canceled())
                        .with_expected("open", SnapshotValue::Bool(false))
                }
                other => {
                    DispatchExecResult::ok().with_host_error(format!("unsupported action `{other}`"))
                }
            });
            dispatcher
        },
        || {
            let reader: SnapshotReader = Rc::new(move || {
                let open_now = open_signal.try_peek().map(|v| *v).unwrap_or(false);
                ComponentSnapshot::new().with_field("open", SnapshotValue::Bool(open_now))
            });
            reader
        },
    );
}

fn extract_aria_label(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if attr.name == "aria-label" {
            match &attr.value {
                AttributeValue::Text(s) if !s.trim().is_empty() => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        }
    })
}

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
    pub handle: Signal<Option<AnyDialogHandle>>,
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
    pub on_open_change_details: Option<EventHandler<ChangeEvent<bool>>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default)]
    pub mode: Option<DialogMode>,
    #[props(default, into)]
    pub handle: DialogHandleProp,
    #[props(default)]
    pub agent: Option<AgentHints>,
    pub children: Element,
}

#[component]
pub fn DialogRoot(props: DialogRootProps) -> Element {
    let DialogRootProps {
        id,
        open,
        default_open,
        on_open_change,
        on_open_change_details,
        on_open_change_complete,
        mode,
        handle,
        agent,
        children,
    } = props;
    let open_ctl = use_controllable_signal_config(
        ControllableConfig::new(open, default_open)
            .with_explicit_default(default_open)
            .with_on_change(on_open_change)
            .with_on_change_details(on_open_change_details)
            .with_component("Dialog")
            .with_prop("open"),
    );
    let open_signal = open_ctl.signal();
    let is_open = open_signal();
    let (is_nested, nested_open_count) = use_dialog_nesting(is_open);

    let scope_id = use_stable_id(id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("dialog").child(scope_id);

    let mut dialog = Dialog::new(scope, is_open);
    if let Some(m) = mode {
        dialog.lifecycle_mut().set_mode(m);
    }
    let mut nested_state = DialogNestedState::new(is_nested);
    nested_state.nested_open_dialog_count = nested_open_count;
    dialog.set_nested_state(nested_state);

    let any_handle = handle.as_any().cloned();
    if let Some(ref h) = any_handle {
        for tid in h.registered_trigger_ids() {
            dialog
                .lifecycle_mut()
                .dismiss_layer_mut()
                .register_branch(tid);
        }
    }

    let root_id = dialog.relationships().root_id().to_owned();
    let runtime = use_dialog_runtime(dialog);
    runtime.set_on_open_change(on_open_change);
    runtime.set_on_open_change_complete(on_open_change_complete);
    let mut runtime_sig = use_signal(|| runtime.clone());
    if runtime_sig.peek().dialog() != runtime.dialog() {
        runtime_sig.set(runtime.clone());
    }

    use_dialog_handle_attachment(
        any_handle.clone(),
        root_id,
        is_open,
        open_signal,
        open_ctl,
        &runtime,
        runtime_sig,
    );

    use_dialog_agent_surface(
        "dialog",
        id.unwrap_or_else(|| String::from("dialog")),
        agent.as_ref(),
        open_signal,
        open_ctl,
    );

    let trigger_id = use_signal(|| None);
    let title_id = use_signal(|| None);
    let description_id = use_signal(|| None);
    let mut handle_sig = use_signal(|| any_handle.clone());
    sync_signal(&mut handle_sig, any_handle);

    use_context_provider(|| DialogContext {
        runtime: runtime_sig,
        open: open_signal,
        open_ctl,
        trigger_id,
        title_id,
        description_id,
        on_open_change,
        is_alert_dialog: false,
        handle: handle_sig,
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
    #[props(default, into)]
    pub handle: DialogHandleProp,
    #[props(default)]
    pub payload: Option<OverlayPayload>,
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
        handle,
        payload,
        on_click,
        on_key_down,
        on_mounted,
        child,
        attributes,
        children,
    } = props;
    let ctx = try_consume_context::<DialogContext>();
    let stable_fallback = use_stable_id(id.clone().map(Cow::Owned));

    let effective_handle = handle
        .as_any()
        .cloned()
        .or_else(|| ctx.and_then(|c| c.handle.read().clone()));
    let _handle_version = use_overlay_handle_version(effective_handle.as_ref());

    if ctx.is_none() && effective_handle.is_none() {
        panic!("DialogTrigger must be rendered inside a DialogRoot/AlertDialogRoot or provided a `handle`");
    }

    let runtime = ctx.map(|c| c.runtime.read().clone());
    let trigger_attrs = runtime.as_ref().map(|rt| rt.trigger());
    let trig_id = id.unwrap_or_else(|| {
        trigger_attrs
            .as_ref()
            .map(|t| t.id().to_string())
            .unwrap_or_else(|| format!("{stable_fallback}-trigger"))
    });

    if let Some(ref rt) = runtime
        && effective_handle.is_none()
    {
        rt.register_trigger_id(&trig_id);
    }
    if let Some(mut c) = ctx
        && effective_handle.is_none()
        && c.trigger_id.peek().as_deref() != Some(trig_id.as_str())
    {
        c.trigger_id.set(Some(trig_id.clone()));
    }

    let mounted_slot: CopyValue<Option<Rc<MountedData>>> = use_hook(|| CopyValue::new(None));
    let mut registered_handle: CopyValue<Option<(AnyDialogHandle, String)>> =
        use_hook(|| CopyValue::new(None));

    {
        let needs_rebind = {
            let current = registered_handle.try_read().ok().and_then(|g| g.clone());
            match (&current, &effective_handle) {
                (Some((prev_h, prev_id)), Some(next_h)) => prev_h != next_h || prev_id != &trig_id,
                (Some(_), None) | (None, Some(_)) => true,
                (None, None) => false,
            }
        };
        if needs_rebind {
            if let Some((prev_h, prev_id)) =
                registered_handle.try_write().ok().and_then(|mut g| g.take())
            {
                prev_h.unregister_trigger(&prev_id);
            }
            if let Some(ref next_h) = effective_handle {
                let mounted = mounted_slot.try_read().ok().and_then(|g| g.clone());
                next_h.register_trigger(&trig_id, mounted, payload.clone());
                registered_handle.set(Some((next_h.clone(), trig_id.clone())));
            }
        } else if let Some(ref next_h) = effective_handle {
            next_h.update_trigger_payload(&trig_id, payload.clone());
        }
    }

    use_drop(move || {
        if let Some((prev_h, prev_id)) =
            registered_handle.try_write().ok().and_then(|mut g| g.take())
        {
            prev_h.unregister_trigger(&prev_id);
        }
    });

    let (aria_expanded, data_state, controls) = if let Some(ref h) = effective_handle {
        let expanded = h.is_opened_by_trigger(&trig_id);
        let controls = if expanded {
            runtime.as_ref().map(|rt| rt.content_element_id())
        } else {
            None
        };
        (
            if expanded { "true" } else { "false" },
            if expanded { "open" } else { "closed" },
            controls,
        )
    } else if let Some(ref trigger) = trigger_attrs {
        (
            if trigger.aria_expanded() {
                "true"
            } else {
                "false"
            },
            trigger.data_state().as_str(),
            trigger.aria_controls().map(str::to_owned),
        )
    } else {
        ("false", "closed", None)
    };

    let runtime_for_mount = runtime.clone();
    let handle_for_mount = effective_handle.clone();
    let trig_id_for_mount = trig_id.clone();
    let mut mounted_slot_for_mount = mounted_slot;

    let runtime_for_click = runtime;
    let handle_for_click = effective_handle;
    let trig_id_for_click = trig_id.clone();
    let open_ctl_opt = ctx.map(|c| c.open_ctl);

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
        Some(EventHandler::new(move |evt: MountedEvent| {
            let data = evt.data();
            if let Ok(mut slot) = mounted_slot_for_mount.try_write() {
                *slot = Some(data.clone());
            }
            if let Some(ref h) = handle_for_mount {
                h.update_trigger_mounted(&trig_id_for_mount, Some(data.clone()));
            }
            if let Some(ref rt) = runtime_for_mount {
                rt.capture_trigger(data);
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
                if let Some(ref rt) = runtime_for_click {
                    rt.clear_pointer_dismiss();
                }
                if let Some(ref h) = handle_for_click {
                    h.activate_trigger(&trig_id_for_click, true);
                } else if let Some(open_ctl) = open_ctl_opt {
                    open_ctl.set_with_context(
                        true,
                        ChangeContext::new(ChangeReason::Open, ChangeSource::Mouse),
                    );
                }
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
    let resolved_host = use_portal_host_context(host, runtime.portal().host().clone());
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
    #[props(default = false)]
    pub force_render: bool,
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
        force_render,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<DialogContext>();
    let runtime = ctx.runtime.read().clone();
    runtime.set_overlay_keep_mounted(keep_mounted);

    let is_nested = runtime.is_nested_reactive();
    let should_render =
        (force_render || !is_nested) && (force_mount || runtime.should_render_overlay());
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
            if runtime_for_ptr.is_topmost_dialog()
                && runtime_for_ptr.should_dismiss_on_outside_pointer(is_alert)
            {
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
            if runtime_for_click.is_topmost_dialog()
                && runtime_for_click.should_dismiss_on_outside_pointer(is_alert)
            {
                runtime_for_click.mark_pointer_dismiss();
                let outcome = open_ctl.set_with_context(
                    false,
                    ChangeContext::new(ChangeReason::OutsidePress, ChangeSource::Mouse),
                );
                if outcome.is_canceled() {
                    runtime_for_click.clear_pointer_dismiss();
                }
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

    let dismiss_details_handler = use_hook(move || {
        let open_for_dismiss = ctx.open;
        let open_ctl_for_dismiss = ctx.open_ctl;
        let runtime_sig = ctx.runtime;
        EventHandler::new(move |(evt, change_evt): (DismissEvent, ChangeEvent<bool>)| {
            let is_currently_open = open_for_dismiss.try_peek().map(|o| *o).unwrap_or(false);
            if !is_currently_open {
                return;
            }
            let was_pointer = matches!(evt, DismissEvent::PointerDownOutside);
            let rt_close = runtime_sig.peek().clone();
            if was_pointer {
                rt_close.mark_pointer_dismiss();
            } else {
                rt_close.clear_pointer_dismiss();
            }
            let outcome = open_ctl_for_dismiss.set_with_event(false, &change_evt);
            if outcome.is_canceled() {
                rt_close.clear_pointer_dismiss();
            }
        })
    });
    runtime.set_dismiss_details_callback(ctx.is_alert_dialog, Some(dismiss_details_handler));

    let cid_for_drop = latest_cid.clone();
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        let current_cid = cid_for_drop.borrow().clone();
        runtime_for_drop.set_dismiss_callback(false, None);
        runtime_for_drop.set_dismiss_details_callback(false, None);
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

    let is_nested = runtime.is_nested_reactive();
    let nested_open_count = runtime.nested_open_count_reactive();
    let base_style = match base_pointer_style {
        Some(ptr) => format!("{ptr} --nested-dialogs: {nested_open_count};"),
        None => format!("--nested-dialogs: {nested_open_count};"),
    };

    let runtime_for_key = runtime.clone();
    let mut mount_cb = runtime.mount_content();

    let mut base = Vec::with_capacity(18);
    base.push(Attribute::new("id", content_id, None, false));
    base.push(Attribute::new("style", base_style, None, false));
    base.push(Attribute::new("role", role, None, false));
    base.push(Attribute::new("aria-modal", aria_modal, None, false));
    push_attr_opt(&mut base, "aria-labelledby", labelledby);
    push_attr_opt(&mut base, "aria-describedby", describedby);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_bool_data_attr(&mut base, "data-nested", is_nested);
    push_bool_data_attr(&mut base, "data-nested-dialog-open", nested_open_count > 0);
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
            let _ = runtime_for_key.handle_escape_keydown(&evt);
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
                open_ctl.set_with_context(
                    false,
                    ChangeContext::new(ChangeReason::Close, ChangeSource::Mouse),
                );
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
    pub agent: Option<AgentHints>,
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
        agent,
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

    let mut action_id_slot = use_hook(|| CopyValue::new(action_id.clone()));
    action_id_slot.set(action_id.clone());
    let aria_label = extract_aria_label(&attributes);
    use_agent_surface(
        agent.as_ref(),
        SurfaceMetaDefaults::default(),
        || {
            let label = aria_label.unwrap_or_else(|| action_id.clone());
            AgentSurface::new("button", label)
                .with_snapshot(ComponentSnapshot::new())
                .with_action(ActionDescriptor::new(
                    "press",
                    "Confirm alert dialog action",
                    false,
                ))
        },
        || {
            let dispatcher: ActionDispatcher = Rc::new(move |dispatch| {
                if dispatch.action != "press" {
                    return DispatchExecResult::ok()
                        .with_host_error(format!("unsupported action `{}`", dispatch.action));
                }
                let dom_id = action_id_slot
                    .try_read()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                if !dom_id.is_empty() && click_element_by_id(&dom_id) {
                    return DispatchExecResult::ok();
                }
                let change_ctx = current_change_context()
                    .unwrap_or_else(|| ChangeContext::agent(ChangeReason::Close));
                let outcome = open_ctl.set_with_context(false, change_ctx);
                DispatchExecResult::ok().with_canceled(outcome.is_canceled())
            });
            dispatcher
        },
        || Rc::new(ComponentSnapshot::new) as SnapshotReader,
    );

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
                open_ctl.set_with_context(
                    false,
                    ChangeContext::new(ChangeReason::Close, ChangeSource::Mouse),
                );
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
    pub agent: Option<AgentHints>,
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
        agent,
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

    let mut cancel_id_slot = use_hook(|| CopyValue::new(cancel_id.clone()));
    cancel_id_slot.set(cancel_id.clone());
    let aria_label = extract_aria_label(&attributes);
    use_agent_surface(
        agent.as_ref(),
        SurfaceMetaDefaults::default(),
        || {
            let label = aria_label.unwrap_or_else(|| cancel_id.clone());
            AgentSurface::new("button", label)
                .with_snapshot(ComponentSnapshot::new())
                .with_action(ActionDescriptor::new(
                    "press",
                    "Cancel alert dialog",
                    false,
                ))
        },
        || {
            let dispatcher: ActionDispatcher = Rc::new(move |dispatch| {
                if dispatch.action != "press" {
                    return DispatchExecResult::ok()
                        .with_host_error(format!("unsupported action `{}`", dispatch.action));
                }
                let dom_id = cancel_id_slot
                    .try_read()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                if !dom_id.is_empty() && click_element_by_id(&dom_id) {
                    return DispatchExecResult::ok();
                }
                let change_ctx = current_change_context()
                    .unwrap_or_else(|| ChangeContext::agent(ChangeReason::Close));
                let outcome = open_ctl.set_with_context(false, change_ctx);
                DispatchExecResult::ok().with_canceled(outcome.is_canceled())
            });
            dispatcher
        },
        || Rc::new(ComponentSnapshot::new) as SnapshotReader,
    );

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
                open_ctl.set_with_context(
                    false,
                    ChangeContext::new(ChangeReason::Close, ChangeSource::Mouse),
                );
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

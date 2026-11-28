use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::agent::{
    ActionDescriptor, ActionDispatch, AgentHints, AgentSurface, ComponentSnapshot, SnapshotValue,
};
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::agent::{
    DispatchExecResult, SurfaceMetaDefaults, cancel_current_agent_action, use_agent_surface,
};
use crate::dialog::{
    DialogContent, DialogDescription, DialogOpenFocusPolicy, DialogOverlay, DialogPortal,
    DialogRoot, DialogTitle,
};
use crate::foundation::browser::{FocusOptions, focus_element_sync_or_defer};
use crate::foundation::overlay::PortalHost;
use crate::foundation::state::{ControllableConfig, use_controllable_signal_config};
use crate::visually_hidden::VisuallyHidden;

use super::super::{
    runtime::{CommandContext, CommandFilterCallback},
    score::compute_command_score,
    types::CommandStateSnapshot,
};
use super::root::CommandRoot;

const DEFAULT_POSITIONER_STYLE: &str = "position: fixed; inset: 0; z-index: 60; display: flex; align-items: center; justify-content: center; pointer-events: none;";

#[derive(Clone, PartialEq, Props)]
pub struct CommandDialogProps {
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
    #[props(default = "Command Palette".to_string(), into)]
    pub title: String,
    #[props(default = "Search for a command to run...".to_string(), into)]
    pub description: String,
    #[props(default)]
    pub portal_host: Option<PortalHost>,
    #[props(default, into)]
    pub overlay_class: Option<String>,
    #[props(default, into)]
    pub overlay_style: Option<String>,
    #[props(default, into)]
    pub positioner_class: Option<String>,
    #[props(default, into)]
    pub positioner_style: Option<String>,
    #[props(default, into)]
    pub content_id: Option<String>,
    #[props(default, into)]
    pub content_class: Option<String>,
    #[props(default, into)]
    pub content_style: Option<String>,
    #[props(default = true)]
    pub auto_focus_input: bool,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<String>>>,
    #[props(default)]
    pub on_state_change: Option<EventHandler<CommandStateSnapshot>>,
    #[props(default, into)]
    pub label: Option<String>,
    #[props(default = true)]
    pub should_filter: bool,
    #[props(default)]
    pub filter: Option<CommandFilterCallback>,
    #[props(default = false)]
    pub loop_focus: bool,
    #[props(default = true)]
    pub vim_bindings: bool,
    #[props(default = false)]
    pub disable_pointer_selection: bool,
    #[props(default = false)]
    pub disable_initial_scroll: bool,
    #[props(default)]
    pub columns: Option<usize>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
fn CommandDialogAutoFocus(
    enabled: bool,
    ctx_slot: CopyValue<Option<CommandContext>>,
) -> Element {
    let ctx = use_context::<CommandContext>();
    let mut slot = ctx_slot;
    let _ = slot.try_write().map(|mut w| *w = Some(ctx));
    use_drop(move || {
        let mut s = ctx_slot;
        let _ = s.try_write().map(|mut w| *w = None);
    });
    let mut did_focus = use_signal(|| false);
    use_effect(move || {
        if enabled && !*did_focus.peek() {
            let input_id = ctx.input_id.read().clone();
            if !input_id.is_empty() {
                did_focus.set(true);
                focus_element_sync_or_defer(&input_id, FocusOptions::new(false));
            }
        }
    });
    rsx! {}
}

#[component]
fn CommandDialogFilterSync(revision: u64) -> Element {
    let mut ctx = use_context::<CommandContext>();
    let mut last_applied = use_hook(|| CopyValue::new(revision));
    if last_applied.try_read().is_ok_and(|prev| *prev != revision) {
        last_applied.set(revision);
        ctx.recompute_filtered(false);
    }
    rsx! {}
}

/// Headless modal command palette dialog (`4B D-3`) composing `monoxus::dialog`
/// with a full-viewport flex-centering positioner, library-owned `CommandInput` open focus,
/// and screen-reader-only `DialogTitle`/`DialogDescription` around `CommandRoot`.
#[component]
pub fn CommandDialog(props: CommandDialogProps) -> Element {
    let open_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.open, props.default_open)
            .with_explicit_default(props.default_open)
            .with_on_change(props.on_open_change)
            .with_on_change_details(props.on_open_change_details)
            .with_component("CommandDialog")
            .with_prop("open"),
    );
    let open_sig = open_ctl.signal();
    let cmd_ctx_slot: CopyValue<Option<CommandContext>> = use_hook(|| CopyValue::new(None));

    let mut on_value_change_slot = use_hook(|| CopyValue::new(props.on_value_change));
    on_value_change_slot.set(props.on_value_change);
    let mut on_value_change_details_slot =
        use_hook(|| CopyValue::new(props.on_value_change_details));
    on_value_change_details_slot.set(props.on_value_change_details);
    let mut on_state_change_slot = use_hook(|| CopyValue::new(props.on_state_change));
    on_state_change_slot.set(props.on_state_change);

    let mut filter_slot = use_hook(|| CopyValue::new(props.filter));
    filter_slot.set(props.filter);
    let mut filter_rev = use_hook(|| CopyValue::new(0_u64));
    let has_filter = props.filter.is_some();
    let mut filter_initialized = use_hook(|| CopyValue::new(false));
    let mut prev_had_filter = use_hook(|| CopyValue::new(has_filter));
    // Dioxus 0.8 `Props::memoize` updates `Some(cb1) -> Some(cb2)` in place via
    // `Callback::__point_to`, preserving `GenerationalBox` pointer equality. Bump
    // `filter_rev` on post-mount re-renders whenever a custom filter is (or was)
    // present; `CommandDialog` only passes the plain `u64` to `CommandDialogFilterSync`
    // (no root signal subscription), so there is zero render loop.
    if filter_initialized.try_read().is_ok_and(|init| *init) {
        let had_filter = prev_had_filter.try_read().is_ok_and(|had| *had);
        prev_had_filter.set(has_filter);
        if has_filter || had_filter {
            let next = filter_rev.try_read().map(|r| r.wrapping_add(1)).unwrap_or(1);
            filter_rev.set(next);
        }
    } else {
        filter_initialized.set(true);
    }
    let current_filter_rev = filter_rev.try_read().map(|r| *r).unwrap_or(0);

    let stable_on_open_change = use_hook(|| {
        EventHandler::new(move |next: bool| {
            let _ = open_ctl.set_with_context(
                next,
                ChangeContext::new(ChangeReason::Activation, ChangeSource::Mouse),
            );
        })
    });
    let stable_on_value_change = use_hook(|| {
        EventHandler::new(move |next: String| {
            if let Some(handler) = on_value_change_slot.try_read().ok().and_then(|s| *s) {
                handler.call(next);
            }
        })
    });
    let stable_on_value_change_details = use_hook(|| {
        EventHandler::new(move |next: ChangeEvent<String>| {
            if let Some(handler) = on_value_change_details_slot
                .try_read()
                .ok()
                .and_then(|s| *s)
            {
                handler.call(next);
            }
        })
    });
    let stable_on_state_change = use_hook(|| {
        EventHandler::new(move |next: CommandStateSnapshot| {
            if let Some(handler) = on_state_change_slot.try_read().ok().and_then(|s| *s) {
                handler.call(next);
            }
        })
    });
    let stable_filter: CommandFilterCallback = use_hook(|| {
        Callback::new(move |(val, query, keywords): (String, String, Vec<String>)| {
            if let Some(cb) = filter_slot.try_read().ok().and_then(|s| *s) {
                cb.call((val, query, keywords))
            } else {
                compute_command_score(&val, &query, &keywords)
            }
        })
    });

    let command_label = props.label.unwrap_or_else(|| props.title.clone());
    let resolved_positioner_style = match props.positioner_style.as_deref().map(str::trim) {
        Some(extra) if !extra.is_empty() => format!("{DEFAULT_POSITIONER_STYLE} {extra}"),
        _ => DEFAULT_POSITIONER_STYLE.to_string(),
    };
    let resolved_content_style = match props.content_style.as_deref().map(str::trim) {
        Some(extra) if !extra.is_empty() => {
            if extra.contains("pointer-events:") {
                Some(extra.to_string())
            } else {
                Some(format!("pointer-events: auto; {extra}"))
            }
        }
        _ => Some("pointer-events: auto;".to_string()),
    };
    let open_focus = if props.auto_focus_input {
        None
    } else {
        Some(DialogOpenFocusPolicy::Suppress)
    };

    let build_dialog_snapshot = move || {
        let is_open = open_sig.try_peek().map(|g| *g).unwrap_or(false);
        let (val, q, items) = cmd_ctx_slot
            .try_read()
            .ok()
            .and_then(|g| *g)
            .map(|ctx| {
                (
                    ctx.value.try_peek().map(|g| g.clone()).unwrap_or_default(),
                    ctx.search.try_peek().map(|g| g.clone()).unwrap_or_default(),
                    ctx.valid_items(),
                )
            })
            .unwrap_or_default();
        ComponentSnapshot::new()
            .with_field("open", SnapshotValue::Bool(is_open))
            .with_field("value", SnapshotValue::Text(val))
            .with_field("search", SnapshotValue::Text(q))
            .with_items(items)
    };
    let agent_label = command_label.clone();
    let reader_snap = build_dialog_snapshot;
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new(),
        move || {
            AgentSurface::new("command", agent_label)
                .with_snapshot(build_dialog_snapshot())
                .with_actions(vec![
                    ActionDescriptor::new("open", "Open command dialog", false),
                    ActionDescriptor::new("close", "Close command dialog", false),
                    ActionDescriptor::new("input", "Update command search query", true),
                    ActionDescriptor::new("select", "Select and activate a command item", true),
                ])
        },
        move || {
            Rc::new(move |dispatch: &ActionDispatch| match dispatch.action.as_str() {
                "open" => {
                    let outcome = open_ctl.set_with_context(true, dispatch.ctx);
                    if outcome.is_canceled() {
                        cancel_current_agent_action();
                        DispatchExecResult::ok().with_canceled(true)
                    } else {
                        DispatchExecResult::ok()
                    }
                }
                "close" => {
                    let outcome = open_ctl.set_with_context(false, dispatch.ctx);
                    if outcome.is_canceled() {
                        cancel_current_agent_action();
                        DispatchExecResult::ok().with_canceled(true)
                    } else {
                        DispatchExecResult::ok()
                    }
                }
                "input" => {
                    let Some(SnapshotValue::Text(q)) = dispatch.args.get("value") else {
                        return DispatchExecResult::ok()
                            .with_host_error("missing or invalid `value` argument");
                    };
                    if let Some(mut ctx) = cmd_ctx_slot.try_read().ok().and_then(|g| *g) {
                        ctx.set_search(q.clone());
                        DispatchExecResult::ok()
                    } else {
                        DispatchExecResult::ok().with_host_error("command dialog is not open")
                    }
                }
                "select" => {
                    let Some(SnapshotValue::Text(v)) = dispatch.args.get("value") else {
                        return DispatchExecResult::ok()
                            .with_host_error("missing or invalid `value` argument");
                    };
                    if let Some(mut ctx) = cmd_ctx_slot.try_read().ok().and_then(|g| *g) {
                        let outcome = ctx.set_value_with_context(v.clone(), false, dispatch.ctx);
                        if outcome.is_canceled() {
                            cancel_current_agent_action();
                            DispatchExecResult::ok().with_canceled(true)
                        } else {
                            let _ = ctx.activate_selected();
                            DispatchExecResult::ok()
                        }
                    } else {
                        DispatchExecResult::ok().with_host_error("command dialog is not open")
                    }
                }
                other => DispatchExecResult::ok()
                    .with_host_error(format!("unsupported command action: {other}")),
            })
        },
        move || Rc::new(reader_snap),
    );

    let current_open = open_sig();

    rsx! {
        DialogRoot {
            id: props.id.clone(),
            open: Some(current_open),
            on_open_change: Some(stable_on_open_change),
            DialogPortal {
                host: props.portal_host,
                DialogOverlay {
                    class: props.overlay_class,
                    style: props.overlay_style,
                }
                div {
                    "data-command-dialog-positioner": "",
                    class: props.positioner_class,
                    style: "{resolved_positioner_style}",
                    DialogContent {
                        id: props.content_id,
                        class: props.content_class,
                        style: resolved_content_style,
                        open_focus,
                        "data-command-dialog": "",
                        VisuallyHidden {
                            DialogTitle { "{props.title}" }
                            DialogDescription { "{props.description}" }
                        }
                        CommandRoot {
                            value: props.value,
                            default_value: props.default_value,
                            on_value_change: Some(stable_on_value_change),
                            on_value_change_details: Some(stable_on_value_change_details),
                            on_state_change: Some(stable_on_state_change),
                            label: command_label,
                            should_filter: props.should_filter,
                            filter: Some(stable_filter),
                            loop_focus: props.loop_focus,
                            vim_bindings: props.vim_bindings,
                            disable_pointer_selection: props.disable_pointer_selection,
                            disable_initial_scroll: props.disable_initial_scroll,
                            columns: props.columns,
                            class: props.class,
                            style: props.style,
                            agent: Some(AgentHints::new().with_hidden(true)),
                            attributes: props.attributes,
                            CommandDialogAutoFocus { enabled: props.auto_focus_input, ctx_slot: cmd_ctx_slot }
                            CommandDialogFilterSync { revision: current_filter_rev }
                            {props.children}
                        }
                    }
                }
            }
        }
    }
}

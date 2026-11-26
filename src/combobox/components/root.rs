use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    form::use_form_control_sync_with_epoch,
    overlay::{
        DismissCallbacks, DismissConfig, DismissEvent, FloatingHandle, FloatingMonitorSpec,
        FloatingScrollPolicy, PlacementAlign, PlacementSide, PortalHost, PresenceState,
        use_dismissable_layer, use_floating_layer, use_presence,
    },
    shared::{ScopeHandle, use_stable_id},
    state::sync_signal,
};

use super::hidden_input::ComboboxHiddenInput;
use crate::combobox::{
    relationships::ComboboxRelationships,
    runtime::{ComboboxRuntime, ComboboxRuntimeState},
    state::ComboboxOptionRegistry,
    types::{ComboboxContext, ComboboxItemData, ComboboxMode, ComboboxScrollAlignment},
};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub mode: ComboboxMode,
    #[props(default)]
    pub value: Option<Signal<Option<String>>>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<String>>>,
    #[props(default)]
    pub values: Option<Signal<Vec<String>>>,
    #[props(default)]
    pub default_values: Option<Vec<String>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default)]
    pub input_value: Option<Signal<String>>,
    #[props(default)]
    pub default_input_value: Option<String>,
    #[props(default)]
    pub on_input_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub open: Option<Signal<bool>>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default = false)]
    pub loop_focus: bool,
    #[props(default)]
    pub scroll_alignment: ComboboxScrollAlignment,
    #[props(default)]
    pub items: Vec<ComboboxItemData>,
    #[props(default = true)]
    pub allow_deselect: bool,
    #[props(default = false)]
    pub clear_on_deselect: bool,
    #[props(default)]
    pub portal_host: PortalHost,
    pub children: Element,
}

#[component]
pub fn ComboboxRoot(props: ComboboxRootProps) -> Element {
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = use_hook(|| ScopeHandle::root(stable_id.clone()));
    let relationships = use_hook(|| ComboboxRelationships::new(scope.clone()));

    let initial_input_value = {
        if let Some(ref iv) = props.default_input_value {
            iv.clone()
        } else if let Some(ref dv) = props.default_value {
            props
                .items
                .iter()
                .find(|i| &i.value == dv)
                .map(|i| i.label.clone())
                .unwrap_or_default()
        } else {
            String::new()
        }
    };

    // 4B D-4: Reactive config signals synced on every render (zero mount-time snapshot).
    let mut mode_sig = use_signal(|| props.mode);
    sync_signal(&mut mode_sig, props.mode);
    let mut disabled_sig = use_signal(|| props.disabled);
    sync_signal(&mut disabled_sig, props.disabled);
    let mut required_sig = use_signal(|| props.required);
    sync_signal(&mut required_sig, props.required);
    let mut loop_focus_sig = use_signal(|| props.loop_focus);
    sync_signal(&mut loop_focus_sig, props.loop_focus);
    let mut scroll_alignment_sig = use_signal(|| props.scroll_alignment);
    sync_signal(&mut scroll_alignment_sig, props.scroll_alignment);
    let mut allow_deselect_sig = use_signal(|| props.allow_deselect);
    sync_signal(&mut allow_deselect_sig, props.allow_deselect);
    let mut root_clear_on_deselect_sig = use_signal(|| props.clear_on_deselect);
    sync_signal(&mut root_clear_on_deselect_sig, props.clear_on_deselect);
    let input_clear_on_deselect_sig = use_signal(|| None::<bool>);
    let mut name_sig = use_signal(|| props.name.clone());
    sync_signal(&mut name_sig, props.name.clone());
    let mut form_sig = use_signal(|| props.form.clone());
    sync_signal(&mut form_sig, props.form.clone());
    let mut portal_host_sig = use_signal(|| props.portal_host.clone());
    sync_signal(&mut portal_host_sig, props.portal_host.clone());

    // Non-reactive callback slots (`Dioxus #5894`) synced on every render.
    let mut on_open_change_slot = use_hook(|| CopyValue::new(props.on_open_change));
    on_open_change_slot.set(props.on_open_change);
    let mut on_open_change_complete_slot =
        use_hook(|| CopyValue::new(props.on_open_change_complete));
    on_open_change_complete_slot.set(props.on_open_change_complete);
    let mut on_value_change_slot = use_hook(|| CopyValue::new(props.on_value_change));
    on_value_change_slot.set(props.on_value_change);
    let mut on_values_change_slot = use_hook(|| CopyValue::new(props.on_values_change));
    on_values_change_slot.set(props.on_values_change);
    let mut on_input_value_change_slot = use_hook(|| CopyValue::new(props.on_input_value_change));
    on_input_value_change_slot.set(props.on_input_value_change);
    let on_interact_outside_slot = use_hook(|| {
        CopyValue::new(None::<EventHandler<crate::combobox::monitors::ComboboxInteractOutsideEvent>>)
    });
    let on_escape_key_down_slot = use_hook(|| {
        CopyValue::new(None::<EventHandler<crate::foundation::shared::EscapeKeyDownEvent>>)
    });

    let open_sig = use_signal(|| props.default_open);
    let mut controlled_open_sig = use_signal(|| props.open);
    sync_signal(&mut controlled_open_sig, props.open);

    let value_sig = use_signal(|| props.default_value.clone());
    let mut controlled_value_sig = use_signal(|| props.value);
    sync_signal(&mut controlled_value_sig, props.value);

    let values_sig = use_signal(|| props.default_values.clone().unwrap_or_default());
    let mut controlled_values_sig = use_signal(|| props.values);
    sync_signal(&mut controlled_values_sig, props.values);

    let input_value_sig = use_signal(|| initial_input_value.clone());
    let mut controlled_input_value_sig = use_signal(|| props.input_value);
    sync_signal(&mut controlled_input_value_sig, props.input_value);

    let highlighted_value = use_signal(|| None::<String>);
    let pending_open_highlight = use_signal(|| None);
    let pending_input_reconcile = use_signal(|| 0_u64);
    let registry = use_signal(|| {
        let mut reg = ComboboxOptionRegistry::new();
        reg.seed_items(&props.items);
        reg
    });
    let registry_version = use_signal(|| 0_u64);
    let pressed_item = use_signal(|| None);
    let suppress_next_click = use_signal(|| false);
    let is_composing = use_signal(|| false);
    let user_has_scrolled = use_signal(|| false);
    let content_mounted = use_signal(|| false);
    let is_static_content = use_signal(|| false);
    let default_value_initial = use_signal(|| props.default_value.clone());
    let default_values_initial = use_signal(|| props.default_values.clone().unwrap_or_default());
    let default_input_value_initial = use_signal(|| initial_input_value);
    let mut reset_epoch_sig = use_signal(|| 0_u64);

    let input_id_sig = use_signal(|| relationships.input_id().to_owned());
    let trigger_id_sig = use_signal(|| relationships.trigger_id().to_owned());
    let content_id_sig = use_signal(|| relationships.content_id().to_owned());
    let viewport_id_sig = use_signal(|| relationships.viewport_id().to_owned());
    let arrow_id_sig = use_signal(|| None::<String>);

    let preferred_side = use_signal(|| PlacementSide::Bottom);
    let preferred_align = use_signal(|| PlacementAlign::Start);
    let side_offset = use_signal(|| 4.0_f32);
    let align_offset = use_signal(|| 0.0_f32);
    let avoid_collisions = use_signal(|| true);
    let collision_boundary = use_signal(|| None::<String>);
    let collision_padding = use_signal(|| 0.0_f32);
    let arrow_padding = use_signal(|| 0.0_f32);
    let hide_when_detached = use_signal(|| false);
    let sticky = use_signal(|| None::<String>);
    let custom_anchor = use_signal(|| None::<String>);
    let force_mount = use_signal(|| false);

    let can_scroll_up = use_signal(|| false);
    let can_scroll_down = use_signal(|| false);

    let mut rt_slot: CopyValue<Option<ComboboxRuntime>> = use_hook(|| CopyValue::new(None));
    let mut floating_slot: CopyValue<Option<FloatingHandle>> = use_hook(|| CopyValue::new(None));

    let is_open = if let Some(controlled) = *controlled_open_sig.read() {
        *controlled.read()
    } else {
        *open_sig.read()
    };
    let content_id = content_id_sig.read().clone();
    let input_id = input_id_sig.read().clone();
    let trigger_id = trigger_id_sig.read().clone();
    let wrapper_id = format!("{content_id}-floating-wrapper");
    let portal_id = format!("{content_id}-portal-root");

    // 1. Presence (`use_presence`, `B10`, `FM-OVL-6`):
    // Defer `floating.clear()` & `on_open_change_complete(false)` until exit completion.
    let presence = use_presence(
        is_open,
        &content_id,
        true,
        Some(EventHandler::new(move |open_now: bool| {
            if !open_now
                && let Some(f) = floating_slot.try_read().ok().and_then(|g| *g)
            {
                f.clear();
            }
            if let Some(cb) = on_open_change_complete_slot
                .try_read()
                .ok()
                .and_then(|s| *s)
            {
                cb.call(open_now);
            }
        })),
    );

    // 2. Floating layer (`use_floating_layer`, fixing `arrow_id: None` bug at `combobox/monitors.rs:223`).
    let is_static = *is_static_content.read();
    let effective_anchor = custom_anchor
        .read()
        .clone()
        .unwrap_or_else(|| input_id.clone());
    let mut floating_spec = FloatingMonitorSpec::new(effective_anchor, content_id.clone());
    floating_spec.arrow_id = arrow_id_sig.read().clone();
    floating_spec.boundary_id = collision_boundary.read().clone();
    floating_spec.side = *preferred_side.read();
    floating_spec.align = *preferred_align.read();
    floating_spec.side_offset = *side_offset.read();
    floating_spec.align_offset = *align_offset.read();
    floating_spec.avoid_collisions = *avoid_collisions.read();
    floating_spec.collision_padding = *collision_padding.read();
    floating_spec.arrow_padding = *arrow_padding.read();
    floating_spec.hide_when_detached = *hide_when_detached.read();
    floating_spec.sticky = sticky.read().as_deref() == Some("always");
    floating_spec.namespace = Some("combobox".to_owned());
    floating_spec.scroll_policy = FloatingScrollPolicy::Reposition;

    let floating = use_floating_layer(is_open && !is_static, floating_spec, None);
    floating_slot.set(Some(floating));

    let presence_state = presence.state();
    use_effect(use_reactive(
        (&is_open, &presence_state),
        move |(open_now, p_state)| {
            if !open_now && p_state == PresenceState::Unmounted {
                floating.clear();
            }
        },
    ));

    // 3. Dismissable layer (`use_dismissable_layer`) with IME-gated Escape & single-invocation dedup.
    let composing_now = *is_composing.read();
    let dismiss_cfg = DismissConfig::new(content_id.clone())
        .with_boundaries(vec![
            input_id.clone(),
            trigger_id,
            content_id,
            wrapper_id,
            portal_id,
        ])
        .with_modal(false)
        .with_listen_escape(!composing_now)
        .with_disable_outside_pointer_events(false);

    let dismiss_callbacks = DismissCallbacks {
        on_escape_key_down: Some(EventHandler::new(move |evt| {
            if let Some(cb) = on_escape_key_down_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
        on_pointer_down_outside: None,
        on_focus_outside: None,
        on_interact_outside: Some(EventHandler::new(move |evt| {
            if let Some(cb) = on_interact_outside_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
        on_dismiss: Some(EventHandler::new(move |_evt: DismissEvent| {
            if let Some(rt) = rt_slot.try_read().ok().and_then(|g| g.clone()) {
                rt.close_menu();
            }
        })),
    };
    let dismiss = use_dismissable_layer(is_open, dismiss_cfg, dismiss_callbacks);

    let runtime = ComboboxRuntime {
        relationships: relationships.clone(),
        state: ComboboxRuntimeState {
            mode: mode_sig,
            disabled: disabled_sig,
            required: required_sig,
            loop_focus: loop_focus_sig,
            scroll_alignment: scroll_alignment_sig,
            allow_deselect: allow_deselect_sig,
            root_clear_on_deselect: root_clear_on_deselect_sig,
            input_clear_on_deselect: input_clear_on_deselect_sig,
            name: name_sig,
            form: form_sig,
            portal_host: portal_host_sig,
            on_open_change: on_open_change_slot,
            on_open_change_complete: on_open_change_complete_slot,
            on_value_change: on_value_change_slot,
            on_values_change: on_values_change_slot,
            on_input_value_change: on_input_value_change_slot,
            open: open_sig,
            controlled_open: controlled_open_sig,
            value: value_sig,
            controlled_value: controlled_value_sig,
            values: values_sig,
            controlled_values: controlled_values_sig,
            input_value: input_value_sig,
            controlled_input_value: controlled_input_value_sig,
            highlighted_value,
            pending_open_highlight,
            pending_input_reconcile,
            registry,
            registry_version,
            pressed_item,
            suppress_next_click,
            is_composing,
            user_has_scrolled,
            content_mounted,
            is_static_content,
            default_value_initial,
            default_values_initial,
            default_input_value_initial,
            reset_epoch: reset_epoch_sig,
            input_id: input_id_sig,
            trigger_id: trigger_id_sig,
            content_id: content_id_sig,
            viewport_id: viewport_id_sig,
            arrow_id: arrow_id_sig,
            preferred_side,
            preferred_align,
            side_offset,
            align_offset,
            avoid_collisions,
            collision_boundary,
            collision_padding,
            arrow_padding,
            hide_when_detached,
            sticky,
            custom_anchor,
            force_mount,
            can_scroll_up,
            can_scroll_down,
            on_interact_outside: on_interact_outside_slot,
            on_escape_key_down: on_escape_key_down_slot,
            presence,
            floating,
            dismiss,
        },
    };
    rt_slot.set(Some(runtime.clone()));

    runtime.seed_items(&props.items);

    // CBX-3 / DEF-CBX-09: Watch selection non-empty -> empty transitions so programmatic
    // deselection also clears `input_value` when `clear_on_deselect` is enabled.
    let has_selection = match runtime.mode() {
        ComboboxMode::Single => runtime.value().is_some(),
        ComboboxMode::Multiple => !runtime.values().is_empty(),
    };
    let mut prev_has_selection = use_signal(|| has_selection);
    {
        let rt = runtime.clone();
        use_effect(use_reactive((&has_selection,), move |(now_has_sel,)| {
            let was_has_sel = *prev_has_selection.peek();
            if was_has_sel != now_has_sel {
                prev_has_selection.set(now_has_sel);
                if was_has_sel
                    && !now_has_sel
                    && rt.clear_on_deselect()
                    && !rt.input_value_untracked().is_empty()
                {
                    rt.set_input_value(String::new(), true);
                }
            }
        }));
    }

    let reset_rt = runtime.clone();
    let (_has_form, reset_epoch) = use_form_control_sync_with_epoch(
        &input_id,
        props.name.as_deref(),
        props.form.as_deref(),
        move || {
            reset_rt.handle_form_reset();
        },
    );
    sync_signal(&mut reset_epoch_sig, reset_epoch);

    use_context_provider(|| ComboboxContext {
        runtime: runtime.clone(),
    });

    let has_name = props.name.as_deref().is_some_and(|n| !n.is_empty());

    rsx! {
        {props.children}
        if has_name {
            ComboboxHiddenInput {}
        }
    }
}

pub use ComboboxRoot as Combobox;

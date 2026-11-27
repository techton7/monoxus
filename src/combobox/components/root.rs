use std::borrow::Cow;

use dioxus::prelude::*;

use crate::field::try_use_field_context;
use crate::foundation::{
    browser::focus::use_composite_focus_out_tracked,
    form::use_form_control_sync_with_epoch,
    overlay::{
        DismissBehavior, DismissCallbacks, DismissConfig, DismissEvent, FloatingHandle,
        FloatingMonitorSpec, FloatingScrollPolicy, OverlayLayerSpec, PlacementAlign, PlacementSide,
        PresenceState, StickyStrategy, sync_overlay_layer_spec, use_dismissable_layer,
        use_floating_layer, use_presence_with_keep_mounted,
    },
    shared::{ScopeHandle, use_stable_id},
    state::{sync_signal, use_controllable_signal},
};

use super::hidden_input::ComboboxHiddenInput;
use crate::combobox::{
    registry::ComboboxOptionRegistry,
    relationships::ComboboxRelationships,
    runtime::{ComboboxRuntime, ComboboxRuntimeState},
    types::{ComboboxContext, ComboboxItemData, ComboboxMode, ComboboxScrollAlignment},
};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub mode: ComboboxMode,
    #[props(default)]
    pub value: Option<Option<String>>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<String>>>,
    #[props(default)]
    pub values: Option<Vec<String>>,
    #[props(default)]
    pub default_values: Option<Vec<String>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default, into)]
    pub input_value: Option<String>,
    #[props(default, into)]
    pub default_input_value: Option<String>,
    #[props(default)]
    pub on_input_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default, into)]
    pub name: Option<String>,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default = false)]
    pub loop_focus: bool,
    #[props(default)]
    pub scroll_alignment: ComboboxScrollAlignment,
    #[props(default)]
    pub items: Vec<ComboboxItemData>,
    #[props(default = true)]
    pub allow_deselect: bool,
    pub children: Element,
}

#[component]
pub fn ComboboxRoot(props: ComboboxRootProps) -> Element {
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = use_hook(|| ScopeHandle::root(stable_id.clone()));
    let relationships = use_hook(|| ComboboxRelationships::new(scope.clone()));

    let field_ctx = try_use_field_context();
    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_required = props.required || field_ctx.is_some_and(|f| f.is_required());
    let effective_name = props
        .name
        .clone()
        .or_else(|| field_ctx.and_then(|f| f.name()));
    let effective_invalid = props
        .invalid
        .or_else(|| field_ctx.map(|f| f.is_invalid()))
        .unwrap_or(false);

    // Freeze mount-time reset baselines (`use_hook`) from controlled-or-default.
    let (baseline_value, baseline_values, baseline_input_value) = use_hook(|| {
        let init_val = props.value.clone().unwrap_or_else(|| props.default_value.clone());
        let init_vals = props
            .values
            .clone()
            .unwrap_or_else(|| props.default_values.clone().unwrap_or_default());
        let init_input = if let Some(ref iv) = props.input_value {
            iv.clone()
        } else if let Some(ref iv) = props.default_input_value {
            iv.clone()
        } else if let Some(ref dv) = init_val {
            props
                .items
                .iter()
                .find(|i| &i.value == dv)
                .map(|i| i.label.clone())
                .unwrap_or_default()
        } else {
            String::new()
        };
        (init_val, init_vals, init_input)
    });

    let open_ctl = use_controllable_signal(props.open, props.default_open, props.on_open_change);
    let open_sig = open_ctl.signal();

    let value_ctl =
        use_controllable_signal(props.value.clone(), baseline_value.clone(), props.on_value_change);
    let value_sig = value_ctl.signal();

    let values_ctl =
        use_controllable_signal(props.values, baseline_values.clone(), props.on_values_change);
    let values_sig = values_ctl.signal();

    let input_value_ctl = use_controllable_signal(
        props.input_value,
        baseline_input_value.clone(),
        props.on_input_value_change,
    );
    let input_value_sig = input_value_ctl.signal();

    // Sync uncontrolled `input_value` when controlled `props.value` updates on re-render.
    let mut prev_controlled_val = use_hook(|| CopyValue::new(props.value.clone().flatten()));
    if let Some(ref ext_val) = props.value {
        let changed = prev_controlled_val
            .try_read()
            .is_ok_and(|prev| prev.as_ref() != ext_val.as_ref());
        if changed {
            prev_controlled_val.set(ext_val.clone());
            if !input_value_ctl.is_controlled() {
                if let Some(v) = ext_val {
                    if let Some(item) = props.items.iter().find(|i| &i.value == v) {
                        input_value_ctl.set(item.label.clone());
                    }
                } else {
                    input_value_ctl.set(String::new());
                }
            }
        }
    }

    // 4B D-4: Reactive config signals synced on every render (zero mount-time snapshot).
    let mut mode_sig = use_signal(|| props.mode);
    sync_signal(&mut mode_sig, props.mode);
    let mut disabled_sig = use_signal(|| effective_disabled);
    sync_signal(&mut disabled_sig, effective_disabled);
    let mut required_sig = use_signal(|| effective_required);
    sync_signal(&mut required_sig, effective_required);
    let mut invalid_sig = use_signal(|| effective_invalid);
    sync_signal(&mut invalid_sig, effective_invalid);
    let mut root_invalid_override_sig = use_signal(|| props.invalid);
    sync_signal(&mut root_invalid_override_sig, props.invalid);
    let mut root_aria_describedby_sig = use_signal(|| props.aria_describedby.clone());
    sync_signal(&mut root_aria_describedby_sig, props.aria_describedby.clone());
    let mut loop_focus_sig = use_signal(|| props.loop_focus);
    sync_signal(&mut loop_focus_sig, props.loop_focus);
    let mut scroll_alignment_sig = use_signal(|| props.scroll_alignment);
    sync_signal(&mut scroll_alignment_sig, props.scroll_alignment);
    let mut allow_deselect_sig = use_signal(|| props.allow_deselect);
    sync_signal(&mut allow_deselect_sig, props.allow_deselect);
    let clear_on_deselect_sig = use_signal(|| false);
    let mut name_sig = use_signal(|| effective_name.clone());
    sync_signal(&mut name_sig, effective_name.clone());
    let mut form_sig = use_signal(|| props.form.clone());
    sync_signal(&mut form_sig, props.form.clone());

    // Non-reactive callback slots (`Dioxus #5894`) synced on every render.
    let mut on_open_change_complete_slot =
        use_hook(|| CopyValue::new(props.on_open_change_complete));
    on_open_change_complete_slot.set(props.on_open_change_complete);
    let on_interact_outside_slot = use_hook(|| {
        CopyValue::new(None::<EventHandler<crate::combobox::monitors::ComboboxInteractOutsideEvent>>)
    });
    let on_escape_key_down_slot = use_hook(|| {
        CopyValue::new(None::<EventHandler<crate::foundation::shared::EscapeKeyDownEvent>>)
    });

    let mut highlighted_value = use_signal(|| None::<String>);
    let mut pending_open_highlight = use_signal(|| None);
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
    let mut user_has_scrolled = use_signal(|| false);
    let content_mounted = use_signal(|| false);
    let is_static_content = use_signal(|| false);
    let default_value_initial = use_signal(|| baseline_value);
    let default_values_initial = use_signal(|| baseline_values);
    let default_input_value_initial = use_signal(|| baseline_input_value);
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
    let sticky = use_signal(|| None::<StickyStrategy>);
    let custom_anchor = use_signal(|| None::<String>);
    let force_mount = use_signal(|| false);
    let keep_mounted = use_signal(|| false);
    let escape_key_down_behavior = use_signal(|| None::<DismissBehavior>);
    let interact_outside_behavior = use_signal(|| None::<DismissBehavior>);

    let can_scroll_up = use_signal(|| false);
    let can_scroll_down = use_signal(|| false);

    let mut rt_slot: CopyValue<Option<ComboboxRuntime>> = use_hook(|| CopyValue::new(None));
    let mut floating_slot: CopyValue<Option<FloatingHandle>> = use_hook(|| CopyValue::new(None));

    let is_open = *open_sig.read();
    let content_id = content_id_sig.read().clone();
    let input_id = input_id_sig.read().clone();
    let trigger_id = trigger_id_sig.read().clone();
    let wrapper_id = format!("{content_id}-floating-wrapper");
    let portal_id = format!("{content_id}-portal-root");

    let (handle_focus_in, handle_focus_out, handle_region_dismiss) =
        use_composite_focus_out_tracked(
            vec![input_id.clone(), trigger_id.clone(), content_id.clone()],
            props.on_focus_out,
        );
    let mut on_composite_focus_in_slot = use_hook(|| CopyValue::new(Some(handle_focus_in)));
    on_composite_focus_in_slot.set(Some(handle_focus_in));
    let mut on_composite_focus_out_slot = use_hook(|| CopyValue::new(Some(handle_focus_out)));
    on_composite_focus_out_slot.set(Some(handle_focus_out));
    let mut prev_open_for_focus = use_hook(|| CopyValue::new(is_open));
    if prev_open_for_focus.try_read().is_ok_and(|was| *was != is_open) {
        let was_open = prev_open_for_focus.try_read().map(|w| *w).unwrap_or(false);
        prev_open_for_focus.set(is_open);
        if was_open && !is_open {
            let _ = pending_open_highlight.try_write().map(|mut w| *w = None);
            sync_signal(&mut user_has_scrolled, false);
            let _ = highlighted_value.try_write().map(|mut w| *w = None);
            handle_region_dismiss.call(());
        } else if !was_open && is_open {
            sync_signal(&mut user_has_scrolled, false);
        }
    }

    // 1. Presence (`use_presence_with_keep_mounted`, `B10`, `FM-OVL-6`):
    // Defer `floating.clear()` & `on_open_change_complete(false)` until exit completion.
    let presence = use_presence_with_keep_mounted(
        is_open,
        &content_id,
        true,
        *keep_mounted.read(),
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
    floating_spec.sticky = *sticky.read() == Some(StickyStrategy::Always);
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

    // 3. Dismissable layer (`use_dismissable_layer`) with IME-gated Escape, 4-way DismissBehavior,
    // & single-invocation dedup.
    let composing_now = *is_composing.read();
    let escape_behavior = escape_key_down_behavior
        .read()
        .unwrap_or(DismissBehavior::Close);
    let outside_behavior = interact_outside_behavior
        .read()
        .unwrap_or(DismissBehavior::Close);

    let dismiss_cfg = DismissConfig::new(content_id.clone())
        .with_boundaries(vec![
            input_id.clone(),
            trigger_id,
            content_id.clone(),
            wrapper_id,
            portal_id,
        ])
        .with_modal(false)
        .with_listen_escape(!composing_now && escape_behavior.dismisses())
        .with_listen_pointer_down_outside(outside_behavior.dismisses())
        .with_listen_focus_outside(outside_behavior.dismisses())
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
            handle_region_dismiss.call(());
        })),
    };
    let dismiss = use_dismissable_layer(is_open, dismiss_cfg, dismiss_callbacks);

    if is_open && !content_id.is_empty() {
        sync_overlay_layer_spec(
            OverlayLayerSpec::new(content_id)
                .with_modal(false)
                .with_escape_behavior(escape_behavior)
                .with_outside_press_behavior(outside_behavior),
        );
    }

    let runtime = ComboboxRuntime {
        relationships: relationships.clone(),
        state: ComboboxRuntimeState {
            mode: mode_sig,
            disabled: disabled_sig,
            required: required_sig,
            invalid: invalid_sig,
            root_invalid_override: root_invalid_override_sig,
            root_aria_describedby: root_aria_describedby_sig,
            loop_focus: loop_focus_sig,
            scroll_alignment: scroll_alignment_sig,
            allow_deselect: allow_deselect_sig,
            clear_on_deselect: clear_on_deselect_sig,
            name: name_sig,
            form: form_sig,
            on_open_change_complete: on_open_change_complete_slot,
            on_composite_focus_in: on_composite_focus_in_slot,
            on_composite_focus_out: on_composite_focus_out_slot,
            open: open_sig,
            open_ctl,
            value: value_sig,
            value_ctl,
            values: values_sig,
            values_ctl,
            input_value: input_value_sig,
            input_value_ctl,
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
            keep_mounted,
            escape_key_down_behavior,
            interact_outside_behavior,
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
        effective_name.as_deref(),
        props.form.as_deref(),
        move || {
            reset_rt.handle_form_reset();
        },
    );
    sync_signal(&mut reset_epoch_sig, reset_epoch);

    use_context_provider(|| ComboboxContext {
        runtime: runtime.clone(),
    });

    let has_name = effective_name.as_deref().is_some_and(|n| !n.is_empty());

    rsx! {
        {props.children}
        if has_name {
            ComboboxHiddenInput {}
        }
    }
}

pub use ComboboxRoot as Combobox;

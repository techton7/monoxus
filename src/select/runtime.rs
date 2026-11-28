use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::foundation::typeahead::{TypeaheadBuffer, TypeaheadConfig};
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

pub use crate::foundation::overlay::serialize_css_custom_properties;
pub use crate::foundation::shared::{EscapeKeyDownEvent, PointerDownOutsideEvent};

use crate::foundation::{
    browser::{FocusOptions, restore_focus_element_by_id, restore_focus_element_by_id_with_options},
    compose::MountedHandle,
    form::use_form_control_sync_with_epoch,
    overlay::{
        DismissBehavior, DismissCallbacks, DismissConfig, DismissEvent, DismissLayerHandle,
        FloatingHandle, FloatingMonitorSpec, FloatingPlacement, FloatingReadiness,
        FloatingScrollPolicy, FocusScopeConfig, FocusScopeHandle, GeometryVars, ModalLayerConfig,
        ModalLayerHandle, ModalShieldPolicy, PlacementAlign, PlacementSide, PresenceController,
        PresenceHandle, PresenceState, StickyStrategy, use_dismissable_layer_with_details,
        use_floating_layer, use_focus_scope, use_modal_layer, use_presence_with_keep_mounted,
    },
    shared::{Direction, PreventableEvent},
    state::{ChangeOutcome, ControllableSignal, DataState, sync_signal},
};

use super::{
    attrs::{
        SelectContentAttributes, SelectItemAttributes, SelectRootAttributes,
        SelectTriggerAttributes,
    },
    registry::{MountedOptionRegistry, SelectItemRegistration},
    relationships::SelectRelationships,
    state::Select,
    types::{SelectItemData, SelectMode},
};

pub type SelectValueChangeHandler = Rc<dyn Fn(Option<String>)>;
pub type SelectValuesChangeHandler = Rc<dyn Fn(Vec<String>)>;
pub type SelectOpenChangeHandler = Rc<dyn Fn(bool)>;
pub type SelectOpenChangeCompleteHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Copy, PartialEq)]
pub struct SelectRuntimeState {
    pub value: Signal<Option<String>>,
    pub value_ctl: Option<ControllableSignal<Option<String>>>,
    pub default_value: Signal<Option<String>>,
    pub values: Signal<Vec<String>>,
    pub values_ctl: Option<ControllableSignal<Vec<String>>>,
    pub default_values: Signal<Vec<String>>,
    pub open: Signal<bool>,
    pub open_ctl: Option<ControllableSignal<bool>>,
    pub disabled: Signal<bool>,
    pub highlighted_value: Signal<Option<String>>,
    pub initial_highlight_pending: Signal<bool>,
    pub items: Signal<Vec<SelectItemData>>,
    pub registry: Signal<MountedOptionRegistry>,
    pub typeahead: Signal<TypeaheadBuffer>,
    pub last_key_timestamp_ms: Signal<f64>,
    pub preferred_side: Signal<PlacementSide>,
    pub preferred_align: Signal<PlacementAlign>,
    pub side_offset: Signal<f32>,
    pub align_offset: Signal<f32>,
    pub avoid_collisions: Signal<bool>,
    pub hide_when_detached: Signal<bool>,
    pub custom_anchor: Signal<Option<String>>,
    pub prevent_scroll: Signal<bool>,
    pub prevent_overflow_text_selection: Signal<bool>,
    pub force_mount: Signal<bool>,
    pub keep_mounted: Signal<bool>,
    pub collision_boundary: Signal<Option<String>>,
    pub collision_padding: Signal<f32>,
    pub arrow_padding: Signal<f32>,
    pub arrow_id: Signal<Option<String>>,
    pub sticky: Signal<Option<StickyStrategy>>,
    pub escape_key_down_behavior: Signal<Option<DismissBehavior>>,
    pub interact_outside_behavior: Signal<Option<DismissBehavior>>,
    pub dir: Signal<Direction>,
    pub reference_hidden: Signal<bool>,
    pub last_dismiss_was_pointer: Signal<bool>,
    pub pending_restore_focus: CopyValue<bool>,
    pub is_static_content: Signal<bool>,
    pub reset_epoch: Signal<u64>,
    pub trigger_id: Signal<String>,
    pub content_id: Signal<String>,
    pub viewport_id: Signal<String>,
    pub can_scroll_up: Signal<bool>,
    pub can_scroll_down: Signal<bool>,
    pub invalid: Signal<bool>,
    pub root_invalid_override: Signal<Option<bool>>,
    pub on_composite_focus_in: CopyValue<Option<EventHandler<FocusEvent>>>,
    pub on_composite_focus_out: CopyValue<Option<EventHandler<FocusEvent>>>,
    pub on_escape_key_down: CopyValue<Option<EventHandler<EscapeKeyDownEvent>>>,
    pub on_pointer_down_outside: CopyValue<Option<EventHandler<PointerDownOutsideEvent>>>,
    pub on_close_auto_focus: CopyValue<Option<EventHandler<()>>>,
    pub presence: Option<PresenceHandle>,
    pub presence_fallback: Signal<PresenceController>,
    pub floating: Option<FloatingHandle>,
    pub dismiss: Option<DismissLayerHandle>,
    pub modal_layer: Option<ModalLayerHandle>,
    pub focus_scope: Option<FocusScopeHandle>,
    pub fallback_readiness: Signal<FloatingReadiness>,
    pub selected_labels: Signal<std::collections::BTreeMap<String, String>>,
    pub pressed_item: Signal<Option<super::types::PressedSelectItem>>,
    pub suppress_next_click: CopyValue<bool>,
}

#[derive(Clone)]
pub struct SelectRuntime {
    pub(crate) relationships: SelectRelationships,
    pub(crate) select: Signal<Select>,
    pub(crate) on_value_change: CopyValue<Option<SelectValueChangeHandler>>,
    pub(crate) on_values_change: CopyValue<Option<SelectValuesChangeHandler>>,
    pub(crate) on_open_change: CopyValue<Option<SelectOpenChangeHandler>>,
    pub(crate) on_open_change_complete: CopyValue<Option<SelectOpenChangeCompleteHandler>>,
    pub(crate) state: SelectRuntimeState,
}

impl PartialEq for SelectRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.relationships == other.relationships
            && self.select == other.select
            && self.state == other.state
    }
}

pub fn use_select_runtime<F1, F2>(
    select: Select,
    on_value_change: Option<F1>,
    on_open_change: Option<F2>,
) -> SelectRuntime
where
    F1: Fn(Option<String>) + 'static,
    F2: Fn(bool) + 'static,
{
    use_select_runtime_full(
        select,
        on_value_change,
        None::<fn(Vec<String>)>,
        on_open_change,
        None::<fn(bool)>,
    )
}

pub(crate) fn use_select_runtime_controllable(
    select: Select,
    value_ctl: ControllableSignal<Option<String>>,
    values_ctl: ControllableSignal<Vec<String>>,
    open_ctl: ControllableSignal<bool>,
    on_open_change_complete: Option<EventHandler<bool>>,
    dir: Direction,
) -> SelectRuntime {
    let initial_def_val = use_hook(|| {
        if value_ctl.is_controlled() {
            value_ctl.peek()
        } else {
            select
                .default_value()
                .or_else(|| select.value())
                .map(str::to_owned)
        }
    });
    let initial_def_vals = use_hook(|| {
        if values_ctl.is_controlled() {
            values_ctl.peek()
        } else if !select.default_values().is_empty() {
            select.default_values().to_vec()
        } else {
            select.values().to_vec()
        }
    });
    let default_value_sig = use_signal(|| initial_def_val);
    let default_values_sig = use_signal(|| initial_def_vals);

    let on_complete_handler: Option<SelectOpenChangeCompleteHandler> =
        on_open_change_complete.map(|h| Rc::new(move |o: bool| h.call(o)) as _);

    use_select_runtime_inner(
        select,
        value_ctl.signal(),
        Some(value_ctl),
        default_value_sig,
        values_ctl.signal(),
        Some(values_ctl),
        default_values_sig,
        open_ctl.signal(),
        Some(open_ctl),
        None,
        None,
        None,
        on_complete_handler,
        dir,
    )
}

pub fn use_select_runtime_full<F1, F2, F3, F4>(
    select: Select,
    on_value_change: Option<F1>,
    on_values_change: Option<F2>,
    on_open_change: Option<F3>,
    on_open_change_complete: Option<F4>,
) -> SelectRuntime
where
    F1: Fn(Option<String>) + 'static,
    F2: Fn(Vec<String>) + 'static,
    F3: Fn(bool) + 'static,
    F4: Fn(bool) + 'static,
{
    let synced_val_change: Option<SelectValueChangeHandler> =
        on_value_change.map(|f| Rc::new(f) as SelectValueChangeHandler);
    let synced_vals_change: Option<SelectValuesChangeHandler> =
        on_values_change.map(|f| Rc::new(f) as SelectValuesChangeHandler);
    let synced_open_change: Option<SelectOpenChangeHandler> =
        on_open_change.map(|f| Rc::new(f) as SelectOpenChangeHandler);
    let synced_open_change_complete: Option<SelectOpenChangeCompleteHandler> =
        on_open_change_complete.map(|f| Rc::new(f) as SelectOpenChangeCompleteHandler);

    let initial_val = select.value().map(str::to_owned);
    let initial_def_val = use_hook(|| {
        if select.is_value_controlled() {
            select.value().map(str::to_owned)
        } else {
            select
                .default_value()
                .or_else(|| select.value())
                .map(str::to_owned)
        }
    });
    let initial_vals = select.values().to_vec();
    let initial_def_vals = use_hook(|| {
        if select.is_values_controlled() {
            select.values().to_vec()
        } else if !select.default_values().is_empty() {
            select.default_values().to_vec()
        } else {
            select.values().to_vec()
        }
    });
    let initial_open = select.is_open();

    let mut value_sig = use_signal(|| initial_val);
    let default_value_sig = use_signal(|| initial_def_val);
    let mut values_sig = use_signal(|| initial_vals);
    let default_values_sig = use_signal(|| initial_def_vals);
    let mut open_sig = use_signal(|| initial_open);

    // Synchronous render-time value/values sync (Pattern C controlled mode).
    let mut prev_prop_val = use_hook(|| CopyValue::new(select.value().map(str::to_owned)));
    let mut prev_prop_vals = use_hook(|| CopyValue::new(select.values().to_vec()));

    let next_val = select.value().map(str::to_owned);
    let next_vals = select.values().to_vec();
    if select.is_value_controlled() {
        prev_prop_val.set(next_val.clone());
        sync_signal(&mut value_sig, next_val.clone());
        if select.is_values_controlled() {
            prev_prop_vals.set(next_vals.clone());
            sync_signal(&mut values_sig, next_vals);
        } else {
            let derived_vals = next_val.map(|v| vec![v]).unwrap_or_default();
            prev_prop_vals.set(derived_vals.clone());
            sync_signal(&mut values_sig, derived_vals);
        }
    } else if select.is_values_controlled() {
        prev_prop_vals.set(next_vals.clone());
        sync_signal(&mut values_sig, next_vals.clone());
        let first_val = next_vals.first().cloned();
        prev_prop_val.set(first_val.clone());
        sync_signal(&mut value_sig, first_val);
    } else {
        if prev_prop_val
            .try_read()
            .map(|cur| *cur != next_val)
            .unwrap_or(true)
        {
            prev_prop_val.set(next_val.clone());
            sync_signal(&mut value_sig, next_val);
        }
        if prev_prop_vals
            .try_read()
            .map(|cur| *cur != next_vals)
            .unwrap_or(true)
        {
            prev_prop_vals.set(next_vals.clone());
            sync_signal(&mut values_sig, next_vals);
        }
    }

    let mut prev_prop_open = use_hook(|| CopyValue::new(select.is_open()));
    let prop_open = select.is_open();
    use_effect(use_reactive((&prop_open,), move |(next_open,)| {
        if prev_prop_open
            .try_read()
            .map(|cur| *cur != next_open)
            .unwrap_or(true)
        {
            prev_prop_open.set(next_open);
            sync_signal(&mut open_sig, next_open);
        }
    }));

    use_select_runtime_inner(
        select,
        value_sig,
        None,
        default_value_sig,
        values_sig,
        None,
        default_values_sig,
        open_sig,
        None,
        synced_val_change,
        synced_vals_change,
        synced_open_change,
        synced_open_change_complete,
        Direction::Ltr,
    )
}

#[allow(clippy::too_many_arguments)]
fn use_select_runtime_inner(
    select: Select,
    value_sig: Signal<Option<String>>,
    value_ctl: Option<ControllableSignal<Option<String>>>,
    default_value_sig: Signal<Option<String>>,
    values_sig: Signal<Vec<String>>,
    values_ctl: Option<ControllableSignal<Vec<String>>>,
    default_values_sig: Signal<Vec<String>>,
    open_sig: Signal<bool>,
    open_ctl: Option<ControllableSignal<bool>>,
    synced_val_change: Option<SelectValueChangeHandler>,
    synced_vals_change: Option<SelectValuesChangeHandler>,
    synced_open_change: Option<SelectOpenChangeHandler>,
    synced_open_change_complete: Option<SelectOpenChangeCompleteHandler>,
    dir: Direction,
) -> SelectRuntime {
    let initial_registry = select.registry();
    let mut initial_labels = std::collections::BTreeMap::new();
    for item in select.items() {
        initial_labels.insert(item.value.clone(), item.text.clone());
    }

    let relationships = select.relationships().clone();
    let mut select_sig = use_signal(|| select.clone());
    sync_signal(&mut select_sig, select.clone());

    let mut disabled_sig = use_signal(|| select.is_disabled());
    sync_signal(&mut disabled_sig, select.is_disabled());
    let highlighted_value_sig = use_signal(|| None);
    let initial_highlight_pending_sig = use_signal(|| false);
    let items_sig = use_signal(|| select.items().to_vec());
    let mut registry_sig = use_signal(|| initial_registry);
    let typeahead_sig = use_signal(|| TypeaheadBuffer::new(TypeaheadConfig::SELECT));
    let last_key_timestamp_ms_sig = use_signal(|| 0.0);
    let preferred_side_sig = use_signal(|| PlacementSide::Bottom);
    let preferred_align_sig = use_signal(|| PlacementAlign::Start);
    let side_offset_sig = use_signal(|| 4.0);
    let align_offset_sig = use_signal(|| 0.0);
    let avoid_collisions_sig = use_signal(|| true);
    let hide_when_detached_sig = use_signal(|| false);
    let custom_anchor_sig = use_signal(|| None::<String>);
    let prevent_scroll_sig = use_signal(|| false);
    let prevent_overflow_text_selection_sig = use_signal(|| false);
    let force_mount_sig = use_signal(|| false);
    let keep_mounted_sig = use_signal(|| false);
    let collision_boundary_sig = use_signal(|| None::<String>);
    let collision_padding_sig = use_signal(|| 0.0);
    let arrow_padding_sig = use_signal(|| 0.0);
    let arrow_id_sig = use_signal(|| None::<String>);
    let sticky_sig = use_signal(|| None::<StickyStrategy>);
    let escape_behavior_sig = use_signal(|| None::<DismissBehavior>);
    let interact_outside_behavior_sig = use_signal(|| None::<DismissBehavior>);
    let mut dir_sig = use_signal(|| dir);
    sync_signal(&mut dir_sig, dir);
    let reference_hidden_sig = use_signal(|| false);
    let last_dismiss_was_pointer_sig = use_signal(|| false);
    let pending_restore_focus_slot = use_hook(|| CopyValue::new(true));
    let is_static_content_sig = use_signal(|| false);
    let mut reset_epoch_sig = use_signal(|| 0_u64);
    let next_default_trigger_id = select.relationships().trigger_id().to_owned();
    let next_default_content_id = select.relationships().content_id().to_owned();
    let next_default_viewport_id = select.relationships().viewport_id();
    let mut prev_default_trigger_id = use_hook(|| CopyValue::new(next_default_trigger_id.clone()));
    let mut prev_default_content_id = use_hook(|| CopyValue::new(next_default_content_id.clone()));
    let mut prev_default_viewport_id =
        use_hook(|| CopyValue::new(next_default_viewport_id.clone()));
    let mut trigger_id_sig = use_signal(|| next_default_trigger_id.clone());
    let mut content_id_sig = use_signal(|| next_default_content_id.clone());
    let mut viewport_id_sig = use_signal(|| next_default_viewport_id.clone());
    if prev_default_trigger_id
        .try_read()
        .is_ok_and(|prev| *prev != next_default_trigger_id)
    {
        let prev = prev_default_trigger_id.read().clone();
        prev_default_trigger_id.set(next_default_trigger_id.clone());
        if trigger_id_sig.try_peek().is_ok_and(|cur| *cur == prev) {
            sync_signal(&mut trigger_id_sig, next_default_trigger_id);
        }
    }
    if prev_default_content_id
        .try_read()
        .is_ok_and(|prev| *prev != next_default_content_id)
    {
        let prev = prev_default_content_id.read().clone();
        prev_default_content_id.set(next_default_content_id.clone());
        if content_id_sig.try_peek().is_ok_and(|cur| *cur == prev) {
            sync_signal(&mut content_id_sig, next_default_content_id);
        }
    }
    if prev_default_viewport_id
        .try_read()
        .is_ok_and(|prev| *prev != next_default_viewport_id)
    {
        let prev = prev_default_viewport_id.read().clone();
        prev_default_viewport_id.set(next_default_viewport_id.clone());
        if viewport_id_sig.try_peek().is_ok_and(|cur| *cur == prev) {
            sync_signal(&mut viewport_id_sig, next_default_viewport_id);
        }
    }
    let can_scroll_up_sig = use_signal(|| false);
    let can_scroll_down_sig = use_signal(|| false);
    let invalid_sig = use_signal(|| false);
    let root_invalid_override_sig = use_signal(|| None::<bool>);
    let fallback_readiness_sig = use_signal(FloatingReadiness::default);
    let selected_labels_sig = use_signal(|| initial_labels);
    let pressed_item_sig = use_signal(|| None);
    let presence_fallback_sig = use_signal(|| {
        PresenceController::new(select.presence().desired_present())
            .with_retained_mount(select.presence().retain_mount())
    });

    let on_composite_focus_in_slot =
        use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));
    let on_composite_focus_out_slot =
        use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));
    let on_escape_key_down_slot =
        use_hook(|| CopyValue::new(None::<EventHandler<EscapeKeyDownEvent>>));
    let on_pointer_down_outside_slot =
        use_hook(|| CopyValue::new(None::<EventHandler<PointerDownOutsideEvent>>));
    let on_close_auto_focus_slot = use_hook(|| CopyValue::new(None::<EventHandler<()>>));
    let suppress_next_click_slot = use_hook(|| CopyValue::new(false));
    let mut on_val_change_slot = use_hook(|| CopyValue::new(synced_val_change.clone()));
    on_val_change_slot.set(synced_val_change);
    let mut on_vals_change_slot = use_hook(|| CopyValue::new(synced_vals_change.clone()));
    on_vals_change_slot.set(synced_vals_change);
    let mut on_open_change_slot = use_hook(|| CopyValue::new(synced_open_change.clone()));
    on_open_change_slot.set(synced_open_change);
    let mut on_complete_cb_slot =
        use_hook(|| CopyValue::new(synced_open_change_complete.clone()));
    on_complete_cb_slot.set(synced_open_change_complete);

    if !select.items().is_empty() && registry_sig.peek().is_empty() {
        registry_sig.set(select.registry());
    }

    let mut rt_slot: CopyValue<Option<SelectRuntime>> = use_hook(|| CopyValue::new(None));

    let content_id = content_id_sig.read().clone();
    let default_content_id = select.relationships().content_id().to_owned();
    let trigger_id = trigger_id_sig.read().clone();
    let wrapper_id = format!("{content_id}-wrapper");
    let portal_id = format!("{default_content_id}-portal-root");
    let is_open = *open_sig.read();
    let keep_mounted = *keep_mounted_sig.read();

    // 1. Presence (`use_presence_with_keep_mounted`)
    let presence = use_presence_with_keep_mounted(
        is_open,
        &content_id,
        select.presence().retain_mount(),
        keep_mounted,
        Some(EventHandler::new(move |open: bool| {
            if let Some(cb) = on_complete_cb_slot.try_read().ok().and_then(|g| g.clone()) {
                cb(open);
            }
        })),
    );

    // 2. Floating layer (`use_floating_layer` with `arrow_id` registration)
    let should_render = presence.should_render() || *force_mount_sig.read();
    let is_static = *is_static_content_sig.read();
    let effective_anchor = custom_anchor_sig
        .read()
        .clone()
        .unwrap_or_else(|| trigger_id.clone());
    let mut floating_spec = FloatingMonitorSpec::new(effective_anchor, content_id.clone());
    floating_spec.arrow_id = arrow_id_sig.read().clone();
    floating_spec.boundary_id = collision_boundary_sig.read().clone();
    floating_spec.side = *preferred_side_sig.read();
    floating_spec.align = *preferred_align_sig.read();
    floating_spec.direction = *dir_sig.read();
    floating_spec.side_offset = *side_offset_sig.read();
    floating_spec.align_offset = *align_offset_sig.read();
    floating_spec.avoid_collisions = *avoid_collisions_sig.read();
    floating_spec.collision_padding = *collision_padding_sig.read();
    floating_spec.arrow_padding = *arrow_padding_sig.read();
    floating_spec.hide_when_detached = *hide_when_detached_sig.read();
    floating_spec.sticky = *sticky_sig.read() == Some(StickyStrategy::Always);
    floating_spec.namespace = Some("select".to_owned());
    floating_spec.scroll_policy = FloatingScrollPolicy::Reposition;

    let floating = use_floating_layer(is_open && should_render && !is_static, floating_spec, None);
    let presence_state = presence.state();
    use_effect(use_reactive(
        (&is_open, &presence_state),
        move |(open_now, p_state)| {
            if !open_now && p_state == PresenceState::Unmounted {
                floating.clear();
            }
        },
    ));

    // 3. Modal scroll lock (`use_modal_layer`)
    let prevent_scroll = *prevent_scroll_sig.read();
    let modal_cfg = ModalLayerConfig {
        lock_id: format!("select-scroll-lock-{content_id}"),
        keep_ids: vec![content_id.clone(), wrapper_id.clone(), portal_id.clone()],
        shield_policy: ModalShieldPolicy::None,
        prevent_scroll,
        prevent_body_pointer_events: prevent_scroll,
        restore_delay: None,
    };
    let modal_layer = use_modal_layer(is_open && prevent_scroll, modal_cfg);

    // 4. Focus scope (`use_focus_scope`)
    let mut focus_cfg = FocusScopeConfig::new(content_id.clone());
    focus_cfg.restore_target_id = Some(trigger_id.clone());
    focus_cfg.auto_focus_on_open = false;
    focus_cfg.restore_focus_on_close = true;
    let focus_scope = use_focus_scope(
        is_open,
        focus_cfg,
        None,
        Some(EventHandler::new(move |_evt| {
            if let Some(cb) = on_close_auto_focus_slot.try_read().ok().and_then(|s| *s) {
                cb.call(());
            }
        })),
    );

    // 5. Dismissable layer (`use_dismissable_layer`)
    let escape_behavior = escape_behavior_sig
        .read()
        .unwrap_or(DismissBehavior::Close);
    let outside_behavior = interact_outside_behavior_sig
        .read()
        .unwrap_or(DismissBehavior::Close);
    let dismiss_cfg = DismissConfig::new(content_id.clone())
        .with_boundaries(vec![
            trigger_id.clone(),
            content_id.clone(),
            wrapper_id,
            portal_id,
        ])
        .with_modal(false)
        .with_listen_escape(escape_behavior.dismisses())
        .with_listen_pointer_down_outside(outside_behavior.dismisses())
        .with_listen_focus_outside(false)
        .with_disable_outside_pointer_events(false);
    let dismiss_callbacks = DismissCallbacks {
        on_escape_key_down: Some(EventHandler::new(move |evt| {
            if let Some(cb) = on_escape_key_down_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
        on_pointer_down_outside: Some(EventHandler::new(move |evt| {
            if let Some(cb) = on_pointer_down_outside_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
        on_focus_outside: None,
        on_interact_outside: None,
        on_dismiss: None,
    };
    let on_dismiss_details = Some(EventHandler::new(
        move |(evt, change_evt): (DismissEvent, ChangeEvent<bool>)| {
            let Some(rt) = rt_slot.try_read().ok().and_then(|g| g.clone()) else {
                return;
            };
            match evt {
                DismissEvent::Escape => {
                    rt.clear_pointer_dismiss();
                    let _ = rt.close_dropdown_with_event(true, &change_evt);
                }
                DismissEvent::PointerDownOutside => {
                    rt.mark_pointer_dismiss();
                    let _ = rt.close_dropdown_with_event(true, &change_evt);
                }
                DismissEvent::FocusOutside => {
                    rt.clear_pointer_dismiss();
                    let _ = rt.close_dropdown_with_event(false, &change_evt);
                }
            }
        },
    ));
    let dismiss = use_dismissable_layer_with_details(
        is_open,
        dismiss_cfg,
        dismiss_callbacks,
        on_dismiss_details,
    );

    // 6. Controlled open transition side-effects (fires when parent updates controlled `open` prop).
    let mut prev_is_open = use_hook(|| CopyValue::new(is_open));
    use_effect(use_reactive((&is_open,), move |(open_now,)| {
        let was_open = prev_is_open.try_read().map(|g| *g).unwrap_or(open_now);
        if was_open == open_now {
            return;
        }
        prev_is_open.set(open_now);
        if open_ctl.is_some_and(|c| c.is_controlled())
            && let Some(rt) = rt_slot.try_read().ok().and_then(|g| g.clone())
        {
            if open_now {
                rt.apply_open_side_effects();
            } else {
                let restore = rt
                    .state
                    .pending_restore_focus
                    .try_read()
                    .map(|g| *g)
                    .unwrap_or(true);
                rt.apply_close_side_effects(restore);
            }
        }
    }));

    // 7. Form reset synchronization (`use_form_control_sync_with_epoch`, `B12`)
    let (_has_form, reset_epoch) = use_form_control_sync_with_epoch(
        &trigger_id,
        select.name(),
        select.form(),
        move || {
            if let Some(rt) = rt_slot.try_read().ok().and_then(|g| g.clone()) {
                rt.handle_form_reset();
            }
        },
    );
    sync_signal(&mut reset_epoch_sig, reset_epoch);

    let state = SelectRuntimeState {
        value: value_sig,
        value_ctl,
        default_value: default_value_sig,
        values: values_sig,
        values_ctl,
        default_values: default_values_sig,
        open: open_sig,
        open_ctl,
        disabled: disabled_sig,
        highlighted_value: highlighted_value_sig,
        initial_highlight_pending: initial_highlight_pending_sig,
        items: items_sig,
        registry: registry_sig,
        typeahead: typeahead_sig,
        last_key_timestamp_ms: last_key_timestamp_ms_sig,
        preferred_side: preferred_side_sig,
        preferred_align: preferred_align_sig,
        side_offset: side_offset_sig,
        align_offset: align_offset_sig,
        avoid_collisions: avoid_collisions_sig,
        hide_when_detached: hide_when_detached_sig,
        custom_anchor: custom_anchor_sig,
        prevent_scroll: prevent_scroll_sig,
        prevent_overflow_text_selection: prevent_overflow_text_selection_sig,
        force_mount: force_mount_sig,
        keep_mounted: keep_mounted_sig,
        collision_boundary: collision_boundary_sig,
        collision_padding: collision_padding_sig,
        arrow_padding: arrow_padding_sig,
        arrow_id: arrow_id_sig,
        sticky: sticky_sig,
        escape_key_down_behavior: escape_behavior_sig,
        interact_outside_behavior: interact_outside_behavior_sig,
        dir: dir_sig,
        reference_hidden: reference_hidden_sig,
        last_dismiss_was_pointer: last_dismiss_was_pointer_sig,
        pending_restore_focus: pending_restore_focus_slot,
        is_static_content: is_static_content_sig,
        reset_epoch: reset_epoch_sig,
        trigger_id: trigger_id_sig,
        content_id: content_id_sig,
        viewport_id: viewport_id_sig,
        can_scroll_up: can_scroll_up_sig,
        can_scroll_down: can_scroll_down_sig,
        invalid: invalid_sig,
        root_invalid_override: root_invalid_override_sig,
        on_composite_focus_in: on_composite_focus_in_slot,
        on_composite_focus_out: on_composite_focus_out_slot,
        on_escape_key_down: on_escape_key_down_slot,
        on_pointer_down_outside: on_pointer_down_outside_slot,
        on_close_auto_focus: on_close_auto_focus_slot,
        presence: Some(presence),
        presence_fallback: presence_fallback_sig,
        floating: Some(floating),
        dismiss: Some(dismiss),
        modal_layer: Some(modal_layer),
        focus_scope: Some(focus_scope),
        fallback_readiness: fallback_readiness_sig,
        selected_labels: selected_labels_sig,
        pressed_item: pressed_item_sig,
        suppress_next_click: suppress_next_click_slot,
    };

    let runtime = SelectRuntime {
        relationships,
        select: select_sig,
        on_value_change: on_val_change_slot,
        on_values_change: on_vals_change_slot,
        on_open_change: on_open_change_slot,
        on_open_change_complete: on_complete_cb_slot,
        state,
    };
    rt_slot.set(Some(runtime.clone()));

    runtime
}

impl SelectRuntime {
    pub fn new(select: Select) -> Self {
        let initial_val = select.value().map(str::to_owned);
        let initial_def_val = select.default_value().map(str::to_owned);
        let initial_vals = select.values().to_vec();
        let initial_def_vals = select.default_values().to_vec();
        let initial_open = select.is_open();
        let initial_registry = select.registry();
        let mut initial_labels = std::collections::BTreeMap::new();
        for item in select.items() {
            initial_labels.insert(item.value.clone(), item.text.clone());
        }

        let state = SelectRuntimeState {
            value: Signal::new(initial_val),
            value_ctl: None,
            default_value: Signal::new(initial_def_val),
            values: Signal::new(initial_vals),
            values_ctl: None,
            default_values: Signal::new(initial_def_vals),
            open: Signal::new(initial_open),
            open_ctl: None,
            disabled: Signal::new(select.is_disabled()),
            highlighted_value: Signal::new(None),
            initial_highlight_pending: Signal::new(false),
            items: Signal::new(select.items().to_vec()),
            registry: Signal::new(initial_registry),
            typeahead: Signal::new(TypeaheadBuffer::new(TypeaheadConfig::SELECT)),
            last_key_timestamp_ms: Signal::new(0.0),
            preferred_side: Signal::new(PlacementSide::Bottom),
            preferred_align: Signal::new(PlacementAlign::Start),
            side_offset: Signal::new(4.0),
            align_offset: Signal::new(0.0),
            avoid_collisions: Signal::new(true),
            hide_when_detached: Signal::new(false),
            custom_anchor: Signal::new(None),
            prevent_scroll: Signal::new(false),
            prevent_overflow_text_selection: Signal::new(false),
            force_mount: Signal::new(false),
            keep_mounted: Signal::new(false),
            collision_boundary: Signal::new(None),
            collision_padding: Signal::new(0.0),
            arrow_padding: Signal::new(0.0),
            arrow_id: Signal::new(None),
            sticky: Signal::new(None),
            escape_key_down_behavior: Signal::new(None),
            interact_outside_behavior: Signal::new(None),
            dir: Signal::new(Direction::Ltr),
            reference_hidden: Signal::new(false),
            last_dismiss_was_pointer: Signal::new(false),
            pending_restore_focus: CopyValue::new(true),
            is_static_content: Signal::new(false),
            reset_epoch: Signal::new(0),
            trigger_id: Signal::new(select.relationships().trigger_id().to_owned()),
            content_id: Signal::new(select.relationships().content_id().to_owned()),
            viewport_id: Signal::new(select.relationships().viewport_id()),
            can_scroll_up: Signal::new(false),
            can_scroll_down: Signal::new(false),
            invalid: Signal::new(false),
            root_invalid_override: Signal::new(None),
            on_composite_focus_in: CopyValue::new(None),
            on_composite_focus_out: CopyValue::new(None),
            on_escape_key_down: CopyValue::new(None),
            on_pointer_down_outside: CopyValue::new(None),
            on_close_auto_focus: CopyValue::new(None),
            presence: None,
            presence_fallback: Signal::new(
                PresenceController::new(select.presence().desired_present())
                    .with_retained_mount(select.presence().retain_mount()),
            ),
            floating: None,
            dismiss: None,
            modal_layer: None,
            focus_scope: None,
            fallback_readiness: Signal::new(FloatingReadiness::default()),
            selected_labels: Signal::new(initial_labels),
            pressed_item: Signal::new(None),
            suppress_next_click: CopyValue::new(false),
        };

        let relationships = select.relationships().clone();
        Self {
            relationships,
            select: Signal::new(select),
            on_value_change: CopyValue::new(None),
            on_values_change: CopyValue::new(None),
            on_open_change: CopyValue::new(None),
            on_open_change_complete: CopyValue::new(None),
            state,
        }
    }

    pub fn select(&self) -> Select {
        self.select.read().clone()
    }

    pub fn relationships(&self) -> &SelectRelationships {
        &self.relationships
    }

    pub fn default_trigger_id(&self) -> String {
        self.select.read().relationships().trigger_id().to_owned()
    }

    pub fn default_content_id(&self) -> String {
        self.select.read().relationships().content_id().to_owned()
    }

    pub fn default_viewport_id(&self) -> String {
        self.select.read().relationships().viewport_id()
    }

    pub fn current_trigger_id(&self) -> String {
        self.state.trigger_id.read().clone()
    }

    pub fn set_trigger_id(&self, id: String) {
        let normalized = if id.trim().is_empty() {
            self.default_trigger_id()
        } else {
            id
        };
        let mut sig = self.state.trigger_id;
        sync_signal(&mut sig, normalized);
    }

    pub fn current_content_id(&self) -> String {
        self.state.content_id.read().clone()
    }

    pub fn set_content_id(&self, id: String) {
        let normalized = if id.trim().is_empty() {
            self.default_content_id()
        } else {
            id
        };
        let mut sig = self.state.content_id;
        sync_signal(&mut sig, normalized);
    }

    pub fn current_viewport_id(&self) -> String {
        self.state.viewport_id.read().clone()
    }

    pub fn set_viewport_id(&self, id: String) {
        let normalized = if id.trim().is_empty() {
            self.default_viewport_id()
        } else {
            id
        };
        let mut sig = self.state.viewport_id;
        sync_signal(&mut sig, normalized);
    }

    pub fn can_scroll_up(&self) -> bool {
        *self.state.can_scroll_up.read()
    }

    pub fn can_scroll_down(&self) -> bool {
        *self.state.can_scroll_down.read()
    }

    pub fn sync_viewport_scroll_state(&self) {
        let vp_id = self.state.viewport_id.peek().clone();
        let (up, down) =
            if let Some(metrics) = crate::foundation::browser::get_element_scroll_metrics(&vp_id) {
                let can_up = metrics.scroll_top > 0.5;
                let can_down =
                    metrics.scroll_top + metrics.client_height < metrics.scroll_height - 0.5;
                (can_up, can_down)
            } else {
                (false, false)
            };
        let mut up_sig = self.state.can_scroll_up;
        let mut down_sig = self.state.can_scroll_down;
        sync_signal(&mut up_sig, up);
        sync_signal(&mut down_sig, down);
    }

    pub fn scroll_viewport_by(&self, delta_y: f64) {
        let vp_id = self.state.viewport_id.peek().clone();
        if let Some(metrics) = crate::foundation::browser::get_element_scroll_metrics(&vp_id) {
            let max_scroll = (metrics.scroll_height - metrics.client_height).max(0.0);
            let next_y = (metrics.scroll_top + delta_y).clamp(0.0, max_scroll);
            crate::foundation::browser::set_element_scroll_position(
                &vp_id,
                metrics.scroll_left,
                next_y,
            );
            self.sync_viewport_scroll_state();
        }
    }

    pub fn is_invalid(&self) -> bool {
        *self.state.invalid.read()
    }

    pub fn set_invalid(&self, invalid: bool) {
        let mut sig = self.state.invalid;
        sync_signal(&mut sig, invalid);
    }

    pub fn root_invalid_override(&self) -> Option<bool> {
        *self.state.root_invalid_override.read()
    }

    pub fn set_root_invalid_override(&self, invalid: Option<bool>) {
        let mut sig = self.state.root_invalid_override;
        sync_signal(&mut sig, invalid);
    }

    pub(crate) fn set_composite_focus_handlers(
        &self,
        on_focus_in: Option<EventHandler<FocusEvent>>,
        on_focus_out: Option<EventHandler<FocusEvent>>,
    ) {
        let mut in_slot = self.state.on_composite_focus_in;
        in_slot.set(on_focus_in);
        let mut out_slot = self.state.on_composite_focus_out;
        out_slot.set(on_focus_out);
    }

    pub(crate) fn notify_composite_focus_in(&self, evt: FocusEvent) {
        if let Some(cb) = self
            .state
            .on_composite_focus_in
            .try_read()
            .ok()
            .and_then(|s| *s)
        {
            cb.call(evt);
        }
    }

    pub(crate) fn notify_composite_focus_out(&self, evt: FocusEvent) {
        if let Some(cb) = self
            .state
            .on_composite_focus_out
            .try_read()
            .ok()
            .and_then(|s| *s)
        {
            cb.call(evt);
        }
    }

    pub fn should_render_portal(&self) -> bool {
        self.should_render_content()
    }

    pub fn should_render_content(&self) -> bool {
        if let Some(p) = self.state.presence {
            p.should_render()
        } else {
            self.state.presence_fallback.read().should_render()
        }
    }

    pub fn presence_state(&self) -> PresenceState {
        if let Some(p) = self.state.presence {
            p.state()
        } else {
            self.state.presence_fallback.read().state()
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.state.presence.is_some_and(|p| p.is_hidden())
    }

    pub fn data_starting_style(&self) -> Option<&'static str> {
        self.state.presence.and_then(|p| p.data_starting_style())
    }

    pub fn data_ending_style(&self) -> Option<&'static str> {
        self.state.presence.and_then(|p| p.data_ending_style())
    }

    pub fn keep_mounted(&self) -> bool {
        *self.state.keep_mounted.read()
    }

    pub fn set_keep_mounted(&self, keep: bool) {
        let mut sig = self.state.keep_mounted;
        sync_signal(&mut sig, keep);
    }

    pub fn escape_key_down_behavior(&self) -> Option<DismissBehavior> {
        *self.state.escape_key_down_behavior.read()
    }

    pub fn set_escape_key_down_behavior(&self, behavior: Option<DismissBehavior>) {
        let mut sig = self.state.escape_key_down_behavior;
        sync_signal(&mut sig, behavior);
    }

    pub fn interact_outside_behavior(&self) -> Option<DismissBehavior> {
        *self.state.interact_outside_behavior.read()
    }

    pub fn set_interact_outside_behavior(&self, behavior: Option<DismissBehavior>) {
        let mut sig = self.state.interact_outside_behavior;
        sync_signal(&mut sig, behavior);
    }

    pub fn reset_epoch(&self) -> u64 {
        *self.state.reset_epoch.read()
    }

    pub fn value(&self) -> Option<String> {
        self.state.value.read().clone()
    }

    pub fn values(&self) -> Vec<String> {
        self.state.values.read().clone()
    }

    pub fn default_value(&self) -> Option<String> {
        self.state.default_value.read().clone()
    }

    pub fn default_values(&self) -> Vec<String> {
        self.state.default_values.read().clone()
    }

    pub fn is_selected(&self, val: &str) -> bool {
        match self.select.read().mode() {
            SelectMode::Single => self.value().as_deref() == Some(val) && !val.is_empty(),
            SelectMode::Multiple => {
                self.state.values.read().iter().any(|v| v == val) && !val.is_empty()
            }
        }
    }

    pub fn is_open(&self) -> bool {
        *self.state.open.read()
    }

    pub fn is_disabled(&self) -> bool {
        *self.state.disabled.read()
    }

    pub fn highlighted_value(&self) -> Option<String> {
        self.state.highlighted_value.read().clone()
    }

    pub(crate) fn highlighted_value_untracked(&self) -> Option<String> {
        self.state.highlighted_value.peek().clone()
    }

    pub fn set_highlighted(&self, val: Option<String>) {
        self.set_highlighted_with_scroll(val, true);
    }

    pub fn set_highlighted_with_scroll(&self, val: Option<String>, should_scroll: bool) {
        let mut hl_sig = self.state.highlighted_value;
        if *hl_sig.peek() != val {
            hl_sig.set(val.clone());
            if let Some(ref v) = val
                && should_scroll
                && self.is_open()
                && self.content_readiness() == FloatingReadiness::Ready
            {
                let item_id = self.relationships().item_id(v);
                let vp_id = self.current_viewport_id();
                let cid = self.current_content_id();
                super::viewport::scroll_item_within_container(&vp_id, &cid, &item_id);
            }
        }
    }

    pub(crate) fn clear_initial_highlight_pending(&self) {
        let mut pending_sig = self.state.initial_highlight_pending;
        if *pending_sig.peek() {
            pending_sig.set(false);
        }
    }

    fn resolve_pending_initial_highlight(&self) {
        if !self.is_open()
            || !*self.state.initial_highlight_pending.peek()
            || self.state.registry.read().ordered_item_ids.is_empty()
        {
            return;
        }

        let current_val = match self.select.read().mode() {
            SelectMode::Single => self.value(),
            SelectMode::Multiple => self.values().first().cloned(),
        };
        let resolved = self.resolve_initial_highlight();
        self.set_highlighted(resolved.clone());
        if current_val.is_none() || resolved == current_val {
            self.clear_initial_highlight_pending();
        }
    }

    pub(crate) fn resolve_initial_highlight(&self) -> Option<String> {
        let current_val = match self.select.read().mode() {
            SelectMode::Single => self.value(),
            SelectMode::Multiple => self.values().first().cloned(),
        };
        let reg = self.state.registry.read();

        if !reg.ordered_item_ids.is_empty() {
            return current_val
                .as_ref()
                .and_then(|value| {
                    reg.active_items()
                        .find(|item| item.value == value.as_str() && !item.disabled)
                        .map(|item| item.value.clone())
                })
                .or_else(|| {
                    reg.active_items()
                        .find(|item| !item.disabled)
                        .map(|item| item.value.clone())
                });
        }

        let items = self.state.items.read();
        current_val
            .as_ref()
            .and_then(|value| {
                items
                    .iter()
                    .find(|item| item.value == value.as_str() && !item.disabled)
                    .map(|item| item.value.clone())
            })
            .or_else(|| {
                items
                    .iter()
                    .find(|item| !item.disabled)
                    .map(|item| item.value.clone())
            })
    }

    pub fn preferred_side(&self) -> PlacementSide {
        *self.state.preferred_side.read()
    }

    pub fn side(&self) -> PlacementSide {
        self.placement()
            .map(|p| p.side())
            .unwrap_or_else(|| self.preferred_side())
    }

    pub fn set_side(&self, side: PlacementSide) {
        let mut pref_sig = self.state.preferred_side;
        sync_signal(&mut pref_sig, side);
    }

    pub fn preferred_align(&self) -> PlacementAlign {
        *self.state.preferred_align.read()
    }

    pub fn align(&self) -> PlacementAlign {
        self.placement()
            .map(|p| p.align())
            .unwrap_or_else(|| self.preferred_align())
    }

    pub fn set_align(&self, align: PlacementAlign) {
        let mut pref_sig = self.state.preferred_align;
        sync_signal(&mut pref_sig, align);
    }

    pub fn side_offset(&self) -> f32 {
        *self.state.side_offset.read()
    }

    pub fn set_side_offset(&self, offset: f32) {
        let mut sig = self.state.side_offset;
        sync_signal(&mut sig, offset);
    }

    pub fn align_offset(&self) -> f32 {
        *self.state.align_offset.read()
    }

    pub fn set_align_offset(&self, offset: f32) {
        let mut sig = self.state.align_offset;
        sync_signal(&mut sig, offset);
    }

    pub fn avoid_collisions(&self) -> bool {
        *self.state.avoid_collisions.read()
    }

    pub fn set_avoid_collisions(&self, avoid: bool) {
        let mut sig = self.state.avoid_collisions;
        sync_signal(&mut sig, avoid);
    }

    pub fn hide_when_detached(&self) -> bool {
        *self.state.hide_when_detached.read()
    }

    pub fn set_hide_when_detached(&self, hide: bool) {
        let mut sig = self.state.hide_when_detached;
        sync_signal(&mut sig, hide);
    }

    pub fn collision_boundary(&self) -> Option<String> {
        self.state.collision_boundary.read().clone()
    }

    pub fn set_collision_boundary(&self, boundary: Option<String>) {
        let mut sig = self.state.collision_boundary;
        sync_signal(&mut sig, boundary);
    }

    pub fn collision_padding(&self) -> f32 {
        *self.state.collision_padding.read()
    }

    pub fn set_collision_padding(&self, padding: f32) {
        let mut sig = self.state.collision_padding;
        sync_signal(&mut sig, padding);
    }

    pub fn arrow_padding(&self) -> f32 {
        *self.state.arrow_padding.read()
    }

    pub fn set_arrow_padding(&self, padding: f32) {
        let mut sig = self.state.arrow_padding;
        sync_signal(&mut sig, padding);
    }

    pub fn arrow_id(&self) -> Option<String> {
        if let Some(f) = self.state.floating
            && let Some(id) = f.arrow_id()
        {
            return Some(id);
        }
        self.state.arrow_id.read().clone()
    }

    pub fn register_arrow_id(&self, id: &str) {
        let mut sig = self.state.arrow_id;
        sync_signal(&mut sig, Some(id.to_owned()));
        if let Some(f) = self.state.floating {
            f.register_arrow_id(id);
        }
    }

    pub fn unregister_arrow_id(&self, id: &str) {
        let mut sig = self.state.arrow_id;
        if sig.try_peek().is_ok_and(|cur| cur.as_deref() == Some(id))
            && let Ok(mut w) = sig.try_write()
        {
            *w = None;
        }
        if let Some(f) = self.state.floating {
            f.unregister_arrow_id(id);
        }
    }

    pub fn capture_trigger_mount(&self, handle: MountedHandle) {
        if let Some(f) = self.state.floating {
            f.capture_anchor(handle);
        }
    }

    pub fn capture_content_mount(&self, handle: MountedHandle) {
        if let Some(f) = self.state.floating {
            f.capture_content(handle);
        }
    }

    pub fn sticky(&self) -> Option<StickyStrategy> {
        *self.state.sticky.read()
    }

    pub fn set_sticky(&self, sticky: Option<StickyStrategy>) {
        let mut sig = self.state.sticky;
        sync_signal(&mut sig, sticky);
    }

    pub fn is_reference_hidden(&self) -> bool {
        self.placement()
            .map(|p| p.reference_hidden())
            .unwrap_or_else(|| *self.state.reference_hidden.read())
    }

    pub fn set_reference_hidden(&self, hidden: bool) {
        let mut sig = self.state.reference_hidden;
        sync_signal(&mut sig, hidden);
    }

    pub fn set_on_escape_key_down(&self, cb: Option<EventHandler<EscapeKeyDownEvent>>) {
        let mut slot = self.state.on_escape_key_down;
        slot.set(cb);
    }

    pub fn custom_anchor(&self) -> Option<String> {
        self.state.custom_anchor.read().clone()
    }

    pub fn set_custom_anchor(&self, anchor: Option<String>) {
        let mut sig = self.state.custom_anchor;
        sync_signal(&mut sig, anchor);
    }

    pub fn prevent_scroll(&self) -> bool {
        *self.state.prevent_scroll.read()
    }

    pub fn set_prevent_scroll(&self, prevent: bool) {
        let mut sig = self.state.prevent_scroll;
        sync_signal(&mut sig, prevent);
    }

    pub fn prevent_overflow_text_selection(&self) -> bool {
        *self.state.prevent_overflow_text_selection.read()
    }

    pub fn set_prevent_overflow_text_selection(&self, prevent: bool) {
        let mut sig = self.state.prevent_overflow_text_selection;
        sync_signal(&mut sig, prevent);
    }

    pub fn force_mount(&self) -> bool {
        *self.state.force_mount.read()
    }

    pub fn set_force_mount(&self, force: bool) {
        let mut sig = self.state.force_mount;
        sync_signal(&mut sig, force);
    }

    pub fn set_is_static_content(&self, is_static: bool) {
        let mut sig = self.state.is_static_content;
        sync_signal(&mut sig, is_static);
    }

    pub fn set_on_pointer_down_outside(&self, cb: Option<EventHandler<PointerDownOutsideEvent>>) {
        let mut slot = self.state.on_pointer_down_outside;
        slot.set(cb);
    }

    pub fn set_on_close_auto_focus(&self, cb: Option<EventHandler<()>>) {
        let mut slot = self.state.on_close_auto_focus;
        slot.set(cb);
    }

    pub fn trigger_pointer_down_outside(&self) -> bool {
        let evt: PointerDownOutsideEvent = PreventableEvent::new(None);
        if let Some(cb) = self.state.on_pointer_down_outside.try_read().ok().and_then(|s| *s) {
            cb.call(evt.clone());
        }
        !evt.is_default_prevented()
    }

    pub fn mark_pointer_dismiss(&self) {
        let mut sig = self.state.last_dismiss_was_pointer;
        sig.set(true);
    }

    pub fn clear_pointer_dismiss(&self) {
        let mut sig = self.state.last_dismiss_was_pointer;
        sig.set(false);
    }

    pub fn last_dismiss_was_pointer(&self) -> bool {
        *self.state.last_dismiss_was_pointer.peek()
    }

    pub fn trigger_close_auto_focus(&self) {
        let was_pointer = *self.state.last_dismiss_was_pointer.peek();
        let mut sig = self.state.last_dismiss_was_pointer;
        sig.set(false);
        if let Some(fs) = self.state.focus_scope {
            fs.apply_close_focus(was_pointer);
            return;
        }
        if let Some(cb) = self.state.on_close_auto_focus.try_read().ok().and_then(|s| *s) {
            cb.call(());
        }
        let trigger_id = self.current_trigger_id();
        if was_pointer {
            restore_focus_element_by_id_with_options(
                &trigger_id,
                FocusOptions::with_focus_visible(false, Some(false)),
            );
        } else {
            restore_focus_element_by_id(&trigger_id);
        }
    }

    fn apply_open_side_effects(&self) {
        self.clear_pointer_dismiss();
        let content_id = self.current_content_id();
        crate::foundation::overlay::ensure_active_overlay_layer(&content_id);

        if let Some(p) = self.state.presence {
            p.sync(true, &content_id);
        } else {
            let mut fallback = self.state.presence_fallback;
            fallback.write().sync(true);
        }

        let mut ta_sig = self.state.typeahead;
        ta_sig.write().reset();
        let mut time_sig = self.state.last_key_timestamp_ms;
        time_sig.set(0.0);

        let init_highlight = self.resolve_initial_highlight();
        self.set_highlighted(init_highlight);
        let mut pending_sig = self.state.initial_highlight_pending;
        pending_sig.set(self.state.registry.read().ordered_item_ids.is_empty());

        self.start_dismiss_monitor();
    }

    fn apply_close_side_effects(&self, restore_focus: bool) {
        let content_id = self.current_content_id();
        crate::foundation::overlay::unregister_active_overlay_layer(&content_id);

        if let Some(p) = self.state.presence {
            p.sync(false, &content_id);
        } else {
            let mut fallback = self.state.presence_fallback;
            let update = fallback.write().sync(false);
            if let Some(cycle) = update.started_close_cycle() {
                fallback.write().complete_close_cycle(cycle);
            }
        }

        self.set_highlighted(None);
        self.stop_dismiss_monitor();

        let mut ta_sig = self.state.typeahead;
        ta_sig.write().reset();
        let mut time_sig = self.state.last_key_timestamp_ms;
        time_sig.set(0.0);

        if self.presence_state() == PresenceState::Unmounted {
            self.stop_position_monitor();
        }
        let mut fb_rd = self.state.fallback_readiness;
        sync_signal(&mut fb_rd, FloatingReadiness::Measuring);

        self.clear_pressed_item();
        self.set_suppress_next_click(false);

        if restore_focus {
            self.trigger_close_auto_focus();
        } else if let Some(cb) = self.state.on_close_auto_focus.try_read().ok().and_then(|s| *s) {
            cb.call(());
        }
    }

    pub fn open_dropdown(&self) {
        let _ = self.open_dropdown_with_context(ChangeContext::new(
            ChangeReason::Activation,
            ChangeSource::Mouse,
        ));
    }

    pub(crate) fn open_dropdown_with_context(&self, ctx: ChangeContext) -> ChangeOutcome {
        if self.is_disabled() || self.is_open() {
            return ChangeOutcome::Unchanged;
        }

        if let Some(open_ctl) = self.state.open_ctl {
            let outcome = open_ctl.set_with_context(true, ctx);
            if outcome.is_committed() {
                self.apply_open_side_effects();
            }
            return outcome;
        }

        let mut open_sig = self.state.open;
        open_sig.set(true);
        self.apply_open_side_effects();

        if let Some(cb) = self.on_open_change.try_read().ok().and_then(|g| g.clone()) {
            cb(true);
        }
        if self.state.presence.is_none()
            && let Some(cb) = self
                .on_open_change_complete
                .try_read()
                .ok()
                .and_then(|g| g.clone())
        {
            cb(true);
        }
        ChangeOutcome::Committed(ctx)
    }

    pub fn registry(&self) -> MountedOptionRegistry {
        self.state.registry.read().clone()
    }

    pub fn register_item(&self, item: SelectItemRegistration) {
        let mut labels_sig = self.state.selected_labels;
        if let Ok(mut labels) = labels_sig.try_write() {
            labels.insert(item.value.clone(), item.label.clone());
        }

        let mut reg_sig = self.state.registry;
        let (active, has_order): (Vec<SelectItemData>, bool) = {
            let mut reg = reg_sig.write();
            if reg.items_by_id.get(&item.id) != Some(&item) {
                reg.register(item);
            }
            let active = reg
                .active_items()
                .map(|i| SelectItemData {
                    value: i.value.clone(),
                    text: i.label.clone(),
                    disabled: i.disabled,
                })
                .collect();
            (active, !reg.ordered_item_ids.is_empty())
        };

        if has_order || !active.is_empty() {
            let mut items_sig = self.state.items;
            items_sig.set(active);
        }

        if self.is_open() && *self.state.initial_highlight_pending.peek() {
            self.sync_dom_order(&self.current_content_id());
            self.resolve_pending_initial_highlight();
        }
    }

    pub fn register_item_data(&self, val: &str, text: &str, disabled: bool) {
        let mut labels_sig = self.state.selected_labels;
        if let Ok(mut labels) = labels_sig.try_write() {
            labels.insert(val.to_string(), text.to_string());
        }
        let id = self.relationships().item_id(val);
        self.register_item(SelectItemRegistration::new(id, val, text, disabled));
    }

    pub fn unregister_item(&self, id: &str) {
        let mut reg_sig = self.state.registry;
        let Ok(mut r) = reg_sig.try_write() else {
            return;
        };
        let removed = {
            let item = r.items_by_id.get(id).cloned();
            r.unregister(id);
            item
        };

        if let Some(item) = removed {
            let active: Vec<SelectItemData> = r
                .active_items()
                .map(|i| SelectItemData {
                    value: i.value.clone(),
                    text: i.label.clone(),
                    disabled: i.disabled,
                })
                .collect();
            let next_cand = if self.highlighted_value().as_deref() == Some(&item.value) {
                r.active_items()
                    .find(|i| !i.disabled)
                    .map(|i| i.value.clone())
            } else {
                None
            };
            drop(r);

            let mut items_sig = self.state.items;
            if let Ok(mut items) = items_sig.try_write() {
                *items = active;
            }

            if next_cand.is_some() {
                self.set_highlighted(next_cand);
            }
        }
    }

    pub fn set_ordered_items(&self, ordered_ids: Vec<String>) {
        let mut reg_sig = self.state.registry;
        let active: Vec<SelectItemData> = {
            let mut reg = reg_sig.write();
            reg.set_tree_order(ordered_ids);
            reg.active_items()
                .map(|i| SelectItemData {
                    value: i.value.clone(),
                    text: i.label.clone(),
                    disabled: i.disabled,
                })
                .collect()
        };
        let mut items_sig = self.state.items;
        items_sig.set(active);

        self.resolve_pending_initial_highlight();
    }

    pub fn sync_dom_order(&self, content_id: &str) {
        let mut candidate_keys: Vec<String> = self
            .state
            .registry
            .read()
            .items_by_id
            .keys()
            .cloned()
            .collect();
        let current_val = match self.select.read().mode() {
            SelectMode::Single => self.value(),
            SelectMode::Multiple => self.values().first().cloned(),
        };
        let current_id = current_val
            .as_deref()
            .map(|value| self.relationships().item_id(value));
        if let Some(ref id) = current_id
            && !candidate_keys.contains(id)
        {
            candidate_keys.push(id.clone());
        }
        let candidate_refs: Vec<&str> = candidate_keys.iter().map(|s| s.as_str()).collect();

        if let Ok(new_order) =
            crate::foundation::browser::query_descendant_order(content_id, &candidate_refs)
        {
            let dom_first_value = if current_val.is_none() {
                oxidase::dom::document()
                    .element_by_id(content_id)
                    .and_then(|content| {
                        content.query_selector("[role='option']:not([aria-disabled='true'])")
                    })
                    .and_then(|item| item.get_attribute("data-value"))
            } else {
                None
            };
            let selected_is_in_dom_order = current_id
                .as_ref()
                .is_some_and(|id| new_order.contains(id));
            let selected_is_enabled = current_id.as_ref().is_some_and(|id| {
                oxidase::dom::document()
                    .element_by_id(id)
                    .and_then(|element| element.get_attribute("aria-disabled"))
                    .as_deref()
                    != Some("true")
            });
            let enabled_selected_value = if selected_is_in_dom_order && selected_is_enabled {
                current_val.clone()
            } else {
                None
            };
            if self.is_open()
                && *self.state.initial_highlight_pending.peek()
                && let Some(value) = enabled_selected_value.or(dom_first_value)
            {
                self.set_highlighted(Some(value));
                self.clear_initial_highlight_pending();
            }

            let registered_order: Vec<String> = {
                let reg = self.state.registry.read();
                new_order
                    .into_iter()
                    .filter(|id| reg.items_by_id.contains_key(id))
                    .collect()
            };
            if self.state.registry.read().ordered_item_ids != registered_order {
                self.set_ordered_items(registered_order);
            }
        }
    }

    pub fn close_dropdown(&self) {
        self.close_dropdown_with_options(true);
    }

    pub fn close_dropdown_without_restore(&self) {
        self.close_dropdown_with_options(false);
    }

    pub fn close_dropdown_with_options(&self, restore_focus: bool) {
        let _ = self.close_dropdown_with_context(
            restore_focus,
            ChangeContext::new(ChangeReason::Activation, ChangeSource::Mouse),
        );
    }

    pub(crate) fn close_dropdown_with_context(
        &self,
        restore_focus: bool,
        ctx: ChangeContext,
    ) -> ChangeOutcome {
        let event = ChangeEvent::new(false, ctx);
        self.close_dropdown_with_event(restore_focus, &event)
    }

    pub(crate) fn close_dropdown_with_event(
        &self,
        restore_focus: bool,
        event: &ChangeEvent<bool>,
    ) -> ChangeOutcome {
        if !self.is_open() {
            return ChangeOutcome::Unchanged;
        }

        if let Some(open_ctl) = self.state.open_ctl {
            if open_ctl.is_controlled() {
                let mut pending = self.state.pending_restore_focus;
                let _ = pending.try_write().map(|mut w| *w = restore_focus);
            }
            let outcome = open_ctl.set_with_event(false, event);
            if outcome.is_committed() {
                self.clear_initial_highlight_pending();
                self.apply_close_side_effects(restore_focus);
            }
            return outcome;
        }

        self.clear_initial_highlight_pending();
        let mut open_sig = self.state.open;
        open_sig.set(false);
        self.apply_close_side_effects(restore_focus);

        if let Some(cb) = self.on_open_change.try_read().ok().and_then(|g| g.clone()) {
            cb(false);
        }
        if self.state.presence.is_none()
            && let Some(cb) = self
                .on_open_change_complete
                .try_read()
                .ok()
                .and_then(|g| g.clone())
        {
            cb(false);
        }
        ChangeOutcome::Committed(event.to_context())
    }

    pub fn toggle_open(&self) {
        if self.is_disabled() {
            return;
        }
        if self.is_open() {
            self.close_dropdown();
        } else {
            self.open_dropdown();
        }
    }

    pub fn root_attributes(&self) -> SelectRootAttributes {
        SelectRootAttributes {
            id: self.relationships().root_id().to_owned(),
            data_state: if self.is_open() {
                DataState::Open
            } else {
                DataState::Closed
            },
            disabled: self.is_disabled(),
            invalid: self.is_invalid(),
        }
    }

    pub fn trigger_attributes(&self) -> SelectTriggerAttributes {
        let select_def = self.select.read();
        let is_placeholder = match select_def.mode() {
            SelectMode::Single => {
                self.value().is_none() || self.value().as_deref() == Some("")
            }
            SelectMode::Multiple => self.values().is_empty(),
        };
        let is_required = select_def.is_required();
        drop(select_def);
        let should_emit = self.is_open() || self.should_render_content();
        let is_invalid = self.is_invalid();
        let content_id = self.current_content_id();

        SelectTriggerAttributes {
            id: self.current_trigger_id(),
            role: "combobox",
            aria_haspopup: "listbox",
            aria_expanded: if self.is_open() { "true" } else { "false" },
            aria_controls: if should_emit {
                Some(content_id.clone())
            } else {
                None
            },
            content_id,
            aria_disabled: if self.is_disabled() {
                Some("true")
            } else {
                None
            },
            aria_required: if is_required {
                Some("true")
            } else {
                None
            },
            aria_invalid: is_invalid.then_some("true"),
            aria_describedby: None,
            aria_labelledby: None,
            data_state: if self.is_open() {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_placeholder: is_placeholder,
            disabled: self.is_disabled(),
            invalid: is_invalid,
            data_invalid: ui_sdk::foundation::shared::data_invalid_attr(is_invalid),
            tabindex: if self.is_disabled() { -1 } else { 0 },
        }
    }

    pub fn record_pressed_item(&self, value: &str, client_x: f64, client_y: f64) {
        let mut sig = self.state.pressed_item;
        if let Ok(mut w) = sig.try_write() {
            *w = Some(super::types::PressedSelectItem {
                value: value.to_owned(),
                client_x,
                client_y,
            });
        }
    }

    pub fn take_pressed_item(&self) -> Option<super::types::PressedSelectItem> {
        let mut sig = self.state.pressed_item;
        if let Ok(mut w) = sig.try_write() {
            w.take()
        } else {
            None
        }
    }

    pub fn clear_pressed_item(&self) {
        let mut sig = self.state.pressed_item;
        if let Ok(mut w) = sig.try_write() {
            *w = None;
        }
    }

    pub fn set_suppress_next_click(&self, suppress: bool) {
        let mut slot = self.state.suppress_next_click;
        if let Ok(mut w) = slot.try_write() {
            *w = suppress;
        }
    }

    pub fn take_suppress_next_click(&self) -> bool {
        let mut slot = self.state.suppress_next_click;
        slot.try_write()
            .map(|mut w| {
                let prev = *w;
                *w = false;
                prev
            })
            .unwrap_or(false)
    }

    pub fn content_attributes(&self, activedescendant: Option<String>) -> SelectContentAttributes {
        self.content_attributes_with_side_and_align(activedescendant, self.side(), self.align())
    }

    pub fn content_attributes_with_side_and_align(
        &self,
        activedescendant: Option<String>,
        side: PlacementSide,
        align: PlacementAlign,
    ) -> SelectContentAttributes {
        SelectContentAttributes {
            id: self.current_content_id(),
            role: "listbox",
            tabindex: -1,
            aria_activedescendant: activedescendant,
            data_state: if self.is_open() {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_side: side,
            data_align: align,
            reference_hidden: self.is_reference_hidden(),
        }
    }

    pub fn item_attributes(
        &self,
        item_val: &str,
        is_highlighted: bool,
        disabled: bool,
    ) -> SelectItemAttributes {
        let is_selected = self.is_selected(item_val);
        let eff_disabled = self.is_disabled() || disabled;

        SelectItemAttributes {
            id: self.relationships().item_id(item_val),
            role: "option",
            aria_selected: if is_selected { "true" } else { "false" },
            aria_disabled: if eff_disabled { Some("true") } else { None },
            data_state: if is_selected { "checked" } else { "unchecked" },
            is_highlighted,
            disabled: eff_disabled,
            value: item_val.to_owned(),
            label: item_val.to_owned(),
        }
    }

    pub fn placement(&self) -> Option<FloatingPlacement> {
        self.state.floating.and_then(|f| f.placement())
    }

    pub fn content_readiness(&self) -> FloatingReadiness {
        if *self.state.is_static_content.read() {
            return *self.state.fallback_readiness.read();
        }
        self.state
            .floating
            .map(|f| f.readiness())
            .unwrap_or_else(|| *self.state.fallback_readiness.read())
    }

    pub fn content_z_index(&self) -> Option<String> {
        self.state.floating.and_then(|f| f.content_z_index())
    }

    pub fn sync_content_z_index(&self) {
        if let Some(f) = self.state.floating {
            f.sync_z_index();
        }
    }

    pub fn clear_content_z_index(&self) {
        if let Some(f) = self.state.floating {
            f.clear_z_index();
        }
    }

    pub fn content_positioning_state(&self) -> &'static str {
        self.content_readiness().positioning_state()
    }

    pub fn content_css_variables(&self) -> Vec<(String, String)> {
        self.placement()
            .map(|placement| select_content_css_variables(placement.geometry()))
            .unwrap_or_default()
    }

    pub fn content_css_custom_properties(&self) -> String {
        serialize_css_custom_properties(&self.content_css_variables())
    }
}

pub const SELECT_RADIX_COMPATIBILITY_PREFIX: &str = "radix-select";
pub const SELECT_RADIX_ANCHOR_LABEL: &str = "trigger";

pub fn select_content_css_variables(geometry: &GeometryVars) -> Vec<(String, String)> {
    let mut vars: Vec<(String, String)> = geometry
        .css_iter()
        .chain(geometry.compatibility_alias_iter(
            SELECT_RADIX_COMPATIBILITY_PREFIX,
            SELECT_RADIX_ANCHOR_LABEL,
        ))
        .collect();

    let avail_w = geometry
        .css_get("--monoxus-select-available-width")
        .unwrap_or_else(|| format!("{}px", geometry.available_width()));
    let avail_h = geometry
        .css_get("--monoxus-select-available-height")
        .unwrap_or_else(|| format!("{}px", geometry.available_height()));

    // Standardized --monoxus-select-* content variables
    vars.push((
        "--monoxus-select-content-transform-origin".to_string(),
        geometry.transform_origin_css_value(),
    ));
    vars.push((
        "--monoxus-select-content-available-width".to_string(),
        avail_w,
    ));
    vars.push((
        "--monoxus-select-content-available-height".to_string(),
        avail_h,
    ));

    // Legacy --bits-select-* compatibility aliases
    vars.push((
        "--bits-select-anchor-width".to_string(),
        format!("{}px", geometry.anchor_width()),
    ));
    vars.push((
        "--bits-select-anchor-height".to_string(),
        format!("{}px", geometry.anchor_height()),
    ));
    vars.push((
        "--bits-select-content-available-width".to_string(),
        format!("{}px", geometry.available_width()),
    ));
    vars.push((
        "--bits-select-content-available-height".to_string(),
        format!("{}px", geometry.available_height()),
    ));
    vars.push((
        "--bits-select-content-transform-origin".to_string(),
        geometry.transform_origin_css_value(),
    ));

    vars
}

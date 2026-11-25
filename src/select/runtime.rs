use std::rc::Rc;

use dioxus::prelude::*;

#[allow(unused_imports)]
use crate::foundation::{
    browser::{
        DocumentDismissEvent, FloatingAutoUpdateEvent, FocusOptions, PresenceMonitorEvent,
        WatcherGuard, restore_focus_element_by_id, restore_focus_element_by_id_with_options,
        scroll_element_into_view_nearest, start_document_dismiss_monitor,
        start_floating_auto_update_monitor, start_form_reset_monitor, start_presence_monitor,
    },
    overlay::{
        FloatingLayer, FloatingPlacement, FloatingReadiness, GeometryVars, PlacementAlign,
        PlacementSide, Presence, PresenceCloseCycleId, PresenceController,
        PresenceControllerUpdate, PresenceState, Rect, Size,
    },
    state::DataState,
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

pub use super::monitors::PointerDownOutsideEvent;
pub(crate) use super::presence::{
    RetainedRootPresenceMonitorState, SelectContentPresenceLane, stop_select_presence_monitor,
    sync_select_presence,
};

pub type SelectValueChangeHandler = Rc<dyn Fn(Option<String>)>;
pub type SelectValuesChangeHandler = Rc<dyn Fn(Vec<String>)>;
pub type SelectOpenChangeHandler = Rc<dyn Fn(bool)>;
pub type SelectOpenChangeCompleteHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Copy, PartialEq)]
pub struct SelectRuntimeState {
    pub value: Signal<Option<String>>,
    pub default_value: Signal<Option<String>>,
    pub values: Signal<Vec<String>>,
    pub default_values: Signal<Vec<String>>,
    pub open: Signal<bool>,
    pub disabled: Signal<bool>,
    pub highlighted_value: Signal<Option<String>>,
    pub initial_highlight_pending: Signal<bool>,
    pub items: Signal<Vec<SelectItemData>>,
    pub registry: Signal<MountedOptionRegistry>,
    pub typeahead_buffer: Signal<String>,
    pub last_key_timestamp_ms: Signal<f64>,
    pub preferred_side: Signal<PlacementSide>,
    pub preferred_align: Signal<PlacementAlign>,
    pub side: Signal<PlacementSide>,
    pub align: Signal<PlacementAlign>,
    pub side_offset: Signal<f32>,
    pub align_offset: Signal<f32>,
    pub avoid_collisions: Signal<bool>,
    pub hide_when_detached: Signal<bool>,
    pub custom_anchor: Signal<Option<String>>,
    pub prevent_scroll: Signal<bool>,
    pub prevent_overflow_text_selection: Signal<bool>,
    pub force_mount: Signal<bool>,
    pub collision_boundary: Signal<Option<String>>,
    pub collision_padding: Signal<f32>,
    pub arrow_padding: Signal<f32>,
    pub sticky: Signal<Option<String>>,
    pub reference_hidden: Signal<bool>,
    pub escape_prevented: Signal<bool>,
    pub last_dismiss_was_pointer: Signal<bool>,
    pub on_escape_keydown: Signal<Option<EventHandler<KeyboardEvent>>>,
    pub on_pointer_down_outside: Signal<Option<EventHandler<PointerDownOutsideEvent>>>,
    pub on_close_auto_focus: Signal<Option<EventHandler<()>>>,
    pub dismiss_monitor: Signal<Option<WatcherGuard>>,
    pub dismiss_loop_token: Signal<u64>,
    pub position_monitor: Signal<Option<WatcherGuard>>,
    pub position_loop_token: Signal<u64>,
    pub form_reset_monitor: Signal<Option<WatcherGuard>>,
    pub presence_lane: Signal<SelectContentPresenceLane>,
    pub presence_monitor: RetainedRootPresenceMonitorState,
    pub placement: Signal<Option<FloatingPlacement>>,
    pub content_readiness: Signal<FloatingReadiness>,
    pub trigger_handle: Signal<Option<Rc<MountedData>>>,
    pub content_handle: Signal<Option<Rc<MountedData>>>,
    pub selected_labels: Signal<std::collections::BTreeMap<String, String>>,
    pub pressed_item: Signal<Option<super::types::PressedSelectItem>>,
}

#[derive(Clone)]
pub struct SelectRuntime {
    pub(crate) select: Select,
    pub(crate) on_value_change: Option<SelectValueChangeHandler>,
    pub(crate) on_values_change: Option<SelectValuesChangeHandler>,
    pub(crate) on_open_change: Option<SelectOpenChangeHandler>,
    pub(crate) on_open_change_complete: Option<SelectOpenChangeCompleteHandler>,
    pub(crate) state: SelectRuntimeState,
}

impl PartialEq for SelectRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.select == other.select && self.state == other.state
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
        value: use_signal(|| initial_val),
        default_value: use_signal(|| initial_def_val),
        values: use_signal(|| initial_vals),
        default_values: use_signal(|| initial_def_vals),
        open: use_signal(|| initial_open),
        disabled: use_signal(|| select.is_disabled()),
        highlighted_value: use_signal(|| None),
        initial_highlight_pending: use_signal(|| false),
        items: use_signal(|| select.items().to_vec()),
        registry: use_signal(|| initial_registry),
        typeahead_buffer: use_signal(String::new),
        last_key_timestamp_ms: use_signal(|| 0.0),
        preferred_side: use_signal(|| PlacementSide::Bottom),
        preferred_align: use_signal(|| PlacementAlign::Start),
        side: use_signal(|| PlacementSide::Bottom),
        align: use_signal(|| PlacementAlign::Start),
        side_offset: use_signal(|| 4.0),
        align_offset: use_signal(|| 0.0),
        avoid_collisions: use_signal(|| true),
        hide_when_detached: use_signal(|| false),
        custom_anchor: use_signal(|| None),
        prevent_scroll: use_signal(|| false),
        prevent_overflow_text_selection: use_signal(|| false),
        force_mount: use_signal(|| false),
        collision_boundary: use_signal(|| None),
        collision_padding: use_signal(|| 0.0),
        arrow_padding: use_signal(|| 0.0),
        sticky: use_signal(|| None),
        reference_hidden: use_signal(|| false),
        escape_prevented: use_signal(|| false),
        last_dismiss_was_pointer: use_signal(|| false),
        on_escape_keydown: use_signal(|| None),
        on_pointer_down_outside: use_signal(|| None),
        on_close_auto_focus: use_signal(|| None),
        dismiss_monitor: use_signal(|| None),
        dismiss_loop_token: use_signal(|| 0),
        position_monitor: use_signal(|| None),
        position_loop_token: use_signal(|| 0),
        form_reset_monitor: use_signal(|| None),
        presence_lane: use_signal(|| SelectContentPresenceLane::new(select.presence())),
        presence_monitor: RetainedRootPresenceMonitorState {
            monitor: use_signal(|| None),
            cycle_id: use_signal(|| None),
        },
        placement: use_signal(|| None),
        content_readiness: use_signal(FloatingReadiness::default),
        trigger_handle: use_signal(|| None),
        content_handle: use_signal(|| None),
        selected_labels: use_signal(|| initial_labels),
        pressed_item: use_signal(|| None),
    };

    let cleanup_state = state;
    let cleanup_content_id = select.relationships().content_id().to_owned();
    dioxus::core::use_drop(move || {
        crate::foundation::overlay::unregister_active_overlay_layer(cleanup_content_id.as_str());
        stop_select_presence_monitor(cleanup_state, cleanup_content_id.as_str());
        let mut dm_sig = cleanup_state.dismiss_monitor;
        if let Ok(mut dm) = dm_sig.try_write() {
            *dm = None;
        }
        let mut pm_sig = cleanup_state.position_monitor;
        if let Ok(mut pm) = pm_sig.try_write() {
            *pm = None;
        }
        let mut frm_sig = cleanup_state.form_reset_monitor;
        if let Ok(mut frm) = frm_sig.try_write() {
            *frm = None;
        }
    });

    let presence_sync_state = state;
    let presence_content_id = select.relationships().content_id().to_owned();
    let mut layer_registered = use_signal(|| false);
    use_effect(use_reactive((&state.open,), move |(open,)| {
        let is_open = *open.read();
        sync_select_presence(&presence_content_id, presence_sync_state, is_open);
        let registered = *layer_registered.peek();
        if is_open && !registered {
            crate::foundation::overlay::register_active_overlay_layer(&presence_content_id);
            layer_registered.set(true);
        } else if !is_open && registered {
            crate::foundation::overlay::unregister_active_overlay_layer(&presence_content_id);
            layer_registered.set(false);
        }
    }));

    let effect_state = state;
    use_effect(use_reactive((&select,), move |(select,)| {
        let mut val_sig = effect_state.value;
        let incoming_val = select.value().map(str::to_owned);
        if *val_sig.peek() != incoming_val {
            val_sig.set(incoming_val);
        }
        let mut vals_sig = effect_state.values;
        let incoming_vals = select.values().to_vec();
        if *vals_sig.peek() != incoming_vals {
            vals_sig.set(incoming_vals);
        }
        let mut open_sig = effect_state.open;
        if *open_sig.peek() != select.is_open() {
            open_sig.set(select.is_open());
        }
        let mut disabled_sig = effect_state.disabled;
        if *disabled_sig.peek() != select.is_disabled() {
            disabled_sig.set(select.is_disabled());
        }
        let current_items = select.items();
        if !current_items.is_empty() && effect_state.registry.peek().is_empty() {
            let mut reg_sig = effect_state.registry;
            reg_sig.set(select.registry());
        }
    }));

    let runtime = SelectRuntime {
        select,
        on_value_change: synced_val_change,
        on_values_change: synced_vals_change,
        on_open_change: synced_open_change,
        on_open_change_complete: synced_open_change_complete,
        state,
    };

    let has_form = runtime.select().name().is_some() || runtime.select().form().is_some();
    let rt = runtime.clone();
    use_effect(move || {
        if has_form {
            rt.start_form_reset_monitor();
        }
    });

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
            default_value: Signal::new(initial_def_val),
            values: Signal::new(initial_vals),
            default_values: Signal::new(initial_def_vals),
            open: Signal::new(initial_open),
            disabled: Signal::new(select.is_disabled()),
            highlighted_value: Signal::new(None),
            initial_highlight_pending: Signal::new(false),
            items: Signal::new(select.items().to_vec()),
            registry: Signal::new(initial_registry),
            typeahead_buffer: Signal::new(String::new()),
            last_key_timestamp_ms: Signal::new(0.0),
            preferred_side: Signal::new(PlacementSide::Bottom),
            preferred_align: Signal::new(PlacementAlign::Start),
            side: Signal::new(PlacementSide::Bottom),
            align: Signal::new(PlacementAlign::Start),
            side_offset: Signal::new(4.0),
            align_offset: Signal::new(0.0),
            avoid_collisions: Signal::new(true),
            hide_when_detached: Signal::new(false),
            custom_anchor: Signal::new(None),
            prevent_scroll: Signal::new(false),
            prevent_overflow_text_selection: Signal::new(false),
            force_mount: Signal::new(false),
            collision_boundary: Signal::new(None),
            collision_padding: Signal::new(0.0),
            arrow_padding: Signal::new(0.0),
            sticky: Signal::new(None),
            reference_hidden: Signal::new(false),
            escape_prevented: Signal::new(false),
            last_dismiss_was_pointer: Signal::new(false),
            on_escape_keydown: Signal::new(None),
            on_pointer_down_outside: Signal::new(None),
            on_close_auto_focus: Signal::new(None),
            dismiss_monitor: Signal::new(None),
            dismiss_loop_token: Signal::new(0),
            position_monitor: Signal::new(None),
            position_loop_token: Signal::new(0),
            form_reset_monitor: Signal::new(None),
            presence_lane: Signal::new(SelectContentPresenceLane::new(select.presence())),
            presence_monitor: RetainedRootPresenceMonitorState {
                monitor: Signal::new(None),
                cycle_id: Signal::new(None),
            },
            placement: Signal::new(None),
            content_readiness: Signal::new(FloatingReadiness::default()),
            trigger_handle: Signal::new(None),
            content_handle: Signal::new(None),
            selected_labels: Signal::new(initial_labels),
            pressed_item: Signal::new(None),
        };

        Self {
            select,
            on_value_change: None,
            on_values_change: None,
            on_open_change: None,
            on_open_change_complete: None,
            state,
        }
    }

    pub fn select(&self) -> Select {
        self.select.clone()
    }

    pub fn relationships(&self) -> &SelectRelationships {
        self.select.relationships()
    }

    pub fn should_render_portal(&self) -> bool {
        self.state.presence_lane.read().should_render_portal()
    }

    pub fn should_render_content(&self) -> bool {
        self.state.presence_lane.read().should_render_content()
    }

    pub fn presence_state(&self) -> PresenceState {
        self.state.presence_lane.read().content.state()
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
        match self.select.mode() {
            SelectMode::Single { .. } => self.value().as_deref() == Some(val) && !val.is_empty(),
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
                let vp_id = self.relationships().viewport_id();
                super::viewport::scroll_item_within_container(
                    &vp_id,
                    self.relationships().content_id(),
                    &item_id,
                );
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

        let current_val = match self.select.mode() {
            SelectMode::Single { .. } => self.value(),
            SelectMode::Multiple => self.values().first().cloned(),
        };
        let resolved = self.resolve_initial_highlight();
        self.set_highlighted(resolved.clone());
        if current_val.is_none() || resolved == current_val {
            self.clear_initial_highlight_pending();
        }
    }

    pub(crate) fn resolve_initial_highlight(&self) -> Option<String> {
        let current_val = match self.select.mode() {
            SelectMode::Single { .. } => self.value(),
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
        *self.state.side.read()
    }

    pub fn set_side(&self, side: PlacementSide) {
        let mut pref_sig = self.state.preferred_side;
        if *pref_sig.peek() != side {
            pref_sig.set(side);
            let mut side_sig = self.state.side;
            if *side_sig.peek() != side {
                side_sig.set(side);
            }
        }
    }

    pub fn preferred_align(&self) -> PlacementAlign {
        *self.state.preferred_align.read()
    }

    pub fn align(&self) -> PlacementAlign {
        *self.state.align.read()
    }

    pub fn set_align(&self, align: PlacementAlign) {
        let mut pref_sig = self.state.preferred_align;
        if *pref_sig.peek() != align {
            pref_sig.set(align);
            let mut align_sig = self.state.align;
            if *align_sig.peek() != align {
                align_sig.set(align);
            }
        }
    }

    pub fn side_offset(&self) -> f32 {
        *self.state.side_offset.read()
    }

    pub fn set_side_offset(&self, offset: f32) {
        let mut sig = self.state.side_offset;
        if *sig.peek() != offset {
            sig.set(offset);
        }
    }

    pub fn align_offset(&self) -> f32 {
        *self.state.align_offset.read()
    }

    pub fn set_align_offset(&self, offset: f32) {
        let mut sig = self.state.align_offset;
        if *sig.peek() != offset {
            sig.set(offset);
        }
    }

    pub fn avoid_collisions(&self) -> bool {
        *self.state.avoid_collisions.read()
    }

    pub fn set_avoid_collisions(&self, avoid: bool) {
        let mut sig = self.state.avoid_collisions;
        if *sig.peek() != avoid {
            sig.set(avoid);
        }
    }

    pub fn hide_when_detached(&self) -> bool {
        *self.state.hide_when_detached.read()
    }

    pub fn set_hide_when_detached(&self, hide: bool) {
        let mut sig = self.state.hide_when_detached;
        if *sig.peek() != hide {
            sig.set(hide);
        }
    }

    pub fn collision_boundary(&self) -> Option<String> {
        self.state.collision_boundary.read().clone()
    }

    pub fn set_collision_boundary(&self, boundary: Option<String>) {
        let mut sig = self.state.collision_boundary;
        if *sig.peek() != boundary {
            sig.set(boundary);
        }
    }

    pub fn collision_padding(&self) -> f32 {
        *self.state.collision_padding.read()
    }

    pub fn set_collision_padding(&self, padding: f32) {
        let mut sig = self.state.collision_padding;
        if *sig.peek() != padding {
            sig.set(padding);
        }
    }

    pub fn arrow_padding(&self) -> f32 {
        *self.state.arrow_padding.read()
    }

    pub fn set_arrow_padding(&self, padding: f32) {
        let mut sig = self.state.arrow_padding;
        if *sig.peek() != padding {
            sig.set(padding);
        }
    }

    pub fn sticky(&self) -> Option<String> {
        self.state.sticky.read().clone()
    }

    pub fn set_sticky(&self, sticky: Option<String>) {
        let mut sig = self.state.sticky;
        if *sig.peek() != sticky {
            sig.set(sticky);
        }
    }

    pub fn is_reference_hidden(&self) -> bool {
        *self.state.reference_hidden.read()
    }

    pub fn set_reference_hidden(&self, hidden: bool) {
        if let Ok(current) = self.state.reference_hidden.try_peek() {
            if *current != hidden {
                drop(current);
                let mut sig = self.state.reference_hidden;
                if let Ok(mut w) = sig.try_write() {
                    *w = hidden;
                }
            }
        }
    }

    pub fn is_escape_prevented(&self) -> bool {
        *self.state.escape_prevented.read()
    }

    pub fn set_escape_prevented(&self, val: bool) {
        let mut sig = self.state.escape_prevented;
        sig.set(val);
    }

    pub fn set_on_escape_keydown(&self, cb: Option<EventHandler<KeyboardEvent>>) {
        let mut sig = self.state.on_escape_keydown;
        sig.set(cb);
    }

    pub fn custom_anchor(&self) -> Option<String> {
        self.state.custom_anchor.read().clone()
    }

    pub fn set_custom_anchor(&self, anchor: Option<String>) {
        let mut sig = self.state.custom_anchor;
        if *sig.peek() != anchor {
            sig.set(anchor);
        }
    }

    pub fn prevent_scroll(&self) -> bool {
        *self.state.prevent_scroll.read()
    }

    pub fn set_prevent_scroll(&self, prevent: bool) {
        let mut sig = self.state.prevent_scroll;
        if *sig.peek() != prevent {
            sig.set(prevent);
        }
    }

    pub fn prevent_overflow_text_selection(&self) -> bool {
        *self.state.prevent_overflow_text_selection.read()
    }

    pub fn set_prevent_overflow_text_selection(&self, prevent: bool) {
        let mut sig = self.state.prevent_overflow_text_selection;
        if *sig.peek() != prevent {
            sig.set(prevent);
        }
    }

    pub fn force_mount(&self) -> bool {
        *self.state.force_mount.read()
    }

    pub fn set_force_mount(&self, force: bool) {
        let mut sig = self.state.force_mount;
        if *sig.peek() != force {
            sig.set(force);
        }
    }

    pub fn set_on_pointer_down_outside(&self, cb: Option<EventHandler<PointerDownOutsideEvent>>) {
        let mut sig = self.state.on_pointer_down_outside;
        sig.set(cb);
    }

    pub fn set_on_close_auto_focus(&self, cb: Option<EventHandler<()>>) {
        let mut sig = self.state.on_close_auto_focus;
        sig.set(cb);
    }

    pub fn trigger_pointer_down_outside(&self) -> bool {
        let evt = PointerDownOutsideEvent::new();
        if let Some(cb) = self.state.on_pointer_down_outside.read().clone() {
            cb.call(evt.clone());
        }
        evt.default_action_enabled()
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
        if let Some(cb) = self.state.on_close_auto_focus.read().clone() {
            cb.call(());
        }
        let was_pointer = *self.state.last_dismiss_was_pointer.peek();
        let mut sig = self.state.last_dismiss_was_pointer;
        sig.set(false);
        if was_pointer {
            restore_focus_element_by_id_with_options(
                self.relationships().trigger_id(),
                FocusOptions::with_focus_visible(false, Some(false)),
            );
        } else {
            restore_focus_element_by_id(self.relationships().trigger_id());
        }
    }

    pub fn open_dropdown(&self) {
        if self.is_disabled() || self.is_open() {
            return;
        }

        self.clear_pointer_dismiss();
        let mut open_sig = self.state.open;
        open_sig.set(true);
        crate::foundation::overlay::register_active_overlay_layer(
            self.relationships().content_id(),
        );
        sync_select_presence(self.relationships().content_id(), self.state, true);

        let mut tb_sig = self.state.typeahead_buffer;
        tb_sig.set(String::new());
        let mut time_sig = self.state.last_key_timestamp_ms;
        time_sig.set(0.0);

        if let Some(ref cb) = self.on_open_change {
            cb(true);
        }
        if let Some(ref cb) = self.on_open_change_complete {
            cb(true);
        }

        let init_highlight = self.resolve_initial_highlight();
        self.set_highlighted(init_highlight);
        let mut pending_sig = self.state.initial_highlight_pending;
        pending_sig.set(self.state.registry.read().ordered_item_ids.is_empty());

        self.start_dismiss_monitor();
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
            self.sync_dom_order(self.relationships().content_id());
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
            // Keep items synchronized in pure Rust
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
        let current_val = match self.select.mode() {
            SelectMode::Single { .. } => self.value(),
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
            if self.is_open() && *self.state.initial_highlight_pending.peek() {
                if let Some(value) = enabled_selected_value.or(dom_first_value) {
                    self.set_highlighted(Some(value));
                    self.clear_initial_highlight_pending();
                }
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
        self.clear_initial_highlight_pending();
        if !self.is_open() {
            return;
        }

        let mut open_sig = self.state.open;
        open_sig.set(false);
        sync_select_presence(self.relationships().content_id(), self.state, false);

        if let Some(ref cb) = self.on_open_change {
            cb(false);
        }
        if let Some(ref cb) = self.on_open_change_complete {
            cb(false);
        }

        self.set_highlighted(None);
        self.stop_dismiss_monitor();
        self.stop_position_monitor();

        let mut tb_sig = self.state.typeahead_buffer;
        tb_sig.set(String::new());
        let mut time_sig = self.state.last_key_timestamp_ms;
        time_sig.set(0.0);

        let mut placement_sig = self.state.placement;
        placement_sig.set(None);
        let mut readiness_sig = self.state.content_readiness;
        readiness_sig.set(FloatingReadiness::Measuring);
        let mut ch_sig = self.state.content_handle;
        ch_sig.set(None);

        self.clear_pressed_item();

        if restore_focus {
            self.trigger_close_auto_focus();
        } else if let Some(cb) = self.state.on_close_auto_focus.read().clone() {
            cb.call(());
        }
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
        }
    }

    pub fn trigger_attributes(&self) -> SelectTriggerAttributes {
        let is_placeholder = match self.select.mode() {
            SelectMode::Single { .. } => {
                self.value().is_none() || self.value().as_deref() == Some("")
            }
            SelectMode::Multiple => self.values().is_empty(),
        };
        let should_emit = self.is_open() || self.should_render_content();

        SelectTriggerAttributes {
            id: self.relationships().trigger_id().to_owned(),
            role: "combobox",
            aria_haspopup: "listbox",
            aria_expanded: if self.is_open() { "true" } else { "false" },
            aria_controls: if should_emit {
                Some(self.relationships().content_id().to_owned())
            } else {
                None
            },
            content_id: self.relationships().content_id().to_owned(),
            aria_disabled: if self.is_disabled() {
                Some("true")
            } else {
                None
            },
            aria_required: if self.select.is_required() {
                Some("true")
            } else {
                None
            },
            data_state: if self.is_open() {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_placeholder: is_placeholder,
            disabled: self.is_disabled(),
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
            id: self.relationships().content_id().to_owned(),
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
        self.state.placement.read().clone()
    }

    pub fn content_readiness(&self) -> FloatingReadiness {
        *self.state.content_readiness.read()
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
    let mut vars: Vec<(String, String)> =
        geometry
            .css_iter()
            .chain(geometry.compatibility_alias_iter(
                SELECT_RADIX_COMPATIBILITY_PREFIX,
                SELECT_RADIX_ANCHOR_LABEL,
            ))
            .collect();

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

pub fn serialize_css_custom_properties(entries: &[(String, String)]) -> String {
    let mut style = String::new();
    for (name, value) in entries {
        style.push(' ');
        style.push_str(name);
        style.push_str(": ");
        style.push_str(value);
        style.push(';');
    }
    style
}

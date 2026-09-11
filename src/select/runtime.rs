use std::rc::Rc;

use dioxus::prelude::*;

#[allow(unused_imports)]
use crate::foundation::{
    browser::{
        DocumentDismissEvent, FloatingAutoUpdateEvent, PresenceMonitorEvent,
        WatcherGuard, restore_focus_element_by_id, scroll_element_into_view_nearest,
        start_document_dismiss_monitor, start_floating_auto_update_monitor,
        start_form_reset_monitor, start_presence_monitor,
    },
    overlay::{
        FloatingLayer, PlacementAlign, PlacementSide, Presence, PresenceCloseCycleId,
        PresenceController, PresenceControllerUpdate, PresenceState, Rect, Size,
    },
    state::DataState,
};

use super::{
    attrs::{
        SelectContentAttributes, SelectItemAttributes, SelectRootAttributes,
        SelectTriggerAttributes,
    },
    relationships::SelectRelationships,
    state::Select,
    types::{SelectItemData, SelectMode},
};

#[derive(Clone, Copy, Debug)]
struct SyntheticEscapeKey;

impl dioxus::html::ModifiersInteraction for SyntheticEscapeKey {
    fn modifiers(&self) -> keyboard_types::Modifiers {
        keyboard_types::Modifiers::empty()
    }
}

impl dioxus::events::HasKeyboardData for SyntheticEscapeKey {
    fn key(&self) -> Key {
        Key::Escape
    }
    fn code(&self) -> keyboard_types::Code {
        keyboard_types::Code::Escape
    }
    fn location(&self) -> keyboard_types::Location {
        keyboard_types::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerDownOutsideEvent {
    prevented: std::rc::Rc<std::cell::Cell<bool>>,
}

impl PointerDownOutsideEvent {
    pub fn new() -> Self {
        Self {
            prevented: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }

    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }

    pub fn default_action_enabled(&self) -> bool {
        !self.prevented.get()
    }
}

impl Default for PointerDownOutsideEvent {
    fn default() -> Self {
        Self::new()
    }
}

pub type SelectValueChangeHandler = Rc<dyn Fn(Option<String>)>;
pub type SelectValuesChangeHandler = Rc<dyn Fn(Vec<String>)>;
pub type SelectOpenChangeHandler = Rc<dyn Fn(bool)>;
pub type SelectOpenChangeCompleteHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectContentPresenceLane {
    pub(crate) content: PresenceController,
}

impl SelectContentPresenceLane {
    pub fn new(presence: &Presence) -> Self {
        Self {
            content: PresenceController::new(presence.desired_present())
                .with_retained_mount(presence.retain_mount()),
        }
    }

    pub fn sync(&mut self, desired_present: bool) -> PresenceControllerUpdate {
        self.content.sync(desired_present)
    }

    pub const fn should_render_portal(&self) -> bool {
        self.should_render_content()
    }

    pub const fn should_render_content(&self) -> bool {
        self.content.should_render()
    }

    pub fn complete_close_cycle(&mut self, cycle_id: PresenceCloseCycleId) -> bool {
        self.content.complete_close_cycle(cycle_id)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct RetainedRootPresenceMonitorState {
    pub monitor: Signal<Option<WatcherGuard>>,
    pub cycle_id: Signal<Option<PresenceCloseCycleId>>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct SelectRuntimeState {
    pub value: Signal<Option<String>>,
    pub default_value: Signal<Option<String>>,
    pub values: Signal<Vec<String>>,
    pub default_values: Signal<Vec<String>>,
    pub open: Signal<bool>,
    pub highlighted_value: Signal<Option<String>>,
    pub items: Signal<Vec<SelectItemData>>,
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

    let state = SelectRuntimeState {
        value: use_signal(|| initial_val),
        default_value: use_signal(|| initial_def_val),
        values: use_signal(|| initial_vals),
        default_values: use_signal(|| initial_def_vals),
        open: use_signal(|| initial_open),
        highlighted_value: use_signal(|| None),
        items: use_signal(|| select.items().to_vec()),
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
    };

    let cleanup_state = state;
    let cleanup_content_id = select.relationships().content_id().to_owned();
    dioxus::core::use_drop(move || {
        stop_select_presence_monitor(cleanup_state, cleanup_content_id.as_str());
        let mut dm_sig = cleanup_state.dismiss_monitor;
        dm_sig.set(None);
        let mut pm_sig = cleanup_state.position_monitor;
        pm_sig.set(None);
        let mut frm_sig = cleanup_state.form_reset_monitor;
        frm_sig.set(None);
    });

    let presence_sync_state = state;
    let presence_content_id = select.relationships().content_id().to_owned();
    use_effect(use_reactive((&state.open,), move |(open,)| {
        sync_select_presence(&presence_content_id, presence_sync_state, *open.read());
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
    if has_form {
        let rt = runtime.clone();
        use_effect(move || {
            rt.start_form_reset_monitor();
        });
    }

    runtime
}

impl SelectRuntime {
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
        self.select.is_disabled()
    }

    pub fn highlighted_value(&self) -> Option<String> {
        self.state.highlighted_value.read().clone()
    }

    pub fn set_highlighted(&self, val: Option<String>) {
        let mut hl_sig = self.state.highlighted_value;
        if *hl_sig.peek() != val {
            hl_sig.set(val.clone());
            if let Some(ref v) = val {
                let item_id = self.relationships().item_id(v);
                scroll_element_into_view_nearest(&item_id);
            }
        }
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
        let mut sig = self.state.reference_hidden;
        if *sig.peek() != hidden {
            sig.set(hidden);
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

    pub fn trigger_close_auto_focus(&self) {
        if let Some(cb) = self.state.on_close_auto_focus.read().clone() {
            cb.call(());
        }
        restore_focus_element_by_id(self.relationships().trigger_id());
    }

    pub fn open_dropdown(&self) {
        if self.is_disabled() || self.is_open() {
            return;
        }

        let mut open_sig = self.state.open;
        open_sig.set(true);

        if let Some(ref cb) = self.on_open_change {
            cb(true);
        }
        if let Some(ref cb) = self.on_open_change_complete {
            cb(true);
        }

        let items = self.state.items.read();
        let current_val = match self.select.mode() {
            SelectMode::Single { .. } => self.value(),
            SelectMode::Multiple => self.values().first().cloned(),
        };
        let init_highlight = current_val
            .as_ref()
            .and_then(|v| {
                items
                    .iter()
                    .find(|i| &i.value == v && !i.disabled)
                    .map(|i| i.value.clone())
            })
            .or_else(|| items.iter().find(|i| !i.disabled).map(|i| i.value.clone()));

        self.set_highlighted(init_highlight);

        self.start_dismiss_monitor();
    }

    pub fn close_dropdown(&self) {
        self.close_dropdown_with_options(true);
    }

    pub fn close_dropdown_without_restore(&self) {
        self.close_dropdown_with_options(false);
    }

    pub fn close_dropdown_with_options(&self, restore_focus: bool) {
        if !self.is_open() {
            return;
        }

        let mut open_sig = self.state.open;
        open_sig.set(false);

        if let Some(ref cb) = self.on_open_change {
            cb(false);
        }
        if let Some(ref cb) = self.on_open_change_complete {
            cb(false);
        }

        self.set_highlighted(None);
        self.stop_dismiss_monitor();
        self.stop_position_monitor();

        if restore_focus {
            self.trigger_close_auto_focus();
        } else if let Some(cb) = self.state.on_close_auto_focus.read().clone() {
            cb.call(());
        }
    }

    pub fn toggle_open(&self) {
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

        SelectTriggerAttributes {
            id: self.relationships().trigger_id().to_owned(),
            role: "combobox",
            aria_haspopup: "listbox",
            aria_expanded: if self.is_open() { "true" } else { "false" },
            aria_controls: self.relationships().content_id().to_owned(),
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

    pub fn start_dismiss_monitor(&self) {
        self.stop_dismiss_monitor();

        let runtime = self.clone();
        let trigger_id = self.relationships().trigger_id().to_owned();
        let content_id = self.relationships().content_id().to_owned();

        let watcher = start_document_dismiss_monitor(move |event| {
            match event {
                DocumentDismissEvent::Escape => {
                    if let Some(cb) = runtime.state.on_escape_keydown.read().clone() {
                        let synth = SyntheticEscapeKey;
                        let kb_data = dioxus::html::KeyboardData::new(synth);
                        let evt = dioxus::core::Event::new(std::rc::Rc::new(kb_data), true);
                        cb.call(evt.clone());
                        if !evt.default_action_enabled() {
                            return;
                        }
                    }
                    if runtime.is_escape_prevented() {
                        runtime.set_escape_prevented(false);
                        return;
                    }
                    runtime.close_dropdown();
                }
                DocumentDismissEvent::PointerDown { path_ids } => {
                    let is_inside = path_ids
                        .iter()
                        .any(|id| id == &trigger_id || id == &content_id);
                    if !is_inside {
                        let should_close = runtime.trigger_pointer_down_outside();
                        if !should_close {
                            return;
                        }
                        runtime.close_dropdown_without_restore();
                    }
                }
                DocumentDismissEvent::FocusIn { path_ids } => {
                    let is_inside = path_ids
                        .iter()
                        .any(|id| id == &trigger_id || id == &content_id);
                    if !is_inside {
                        runtime.close_dropdown_without_restore();
                    }
                }
            }
        });

        let mut dm_sig = self.state.dismiss_monitor;
        dm_sig.set(Some(watcher));
    }

    pub fn stop_dismiss_monitor(&self) {
        let mut dm_sig = self.state.dismiss_monitor;
        dm_sig.set(None);
    }

    pub fn start_position_monitor(&self) {
        self.stop_position_monitor();

        let trigger_id = self.relationships().trigger_id().to_owned();
        let content_id = self.relationships().content_id().to_owned();

        let runtime = self.clone();
        let watcher = start_floating_auto_update_monitor(
            &[&trigger_id],
            &content_id,
            move |_event| {
                let runtime = runtime.clone();
                spawn(async move {
                    runtime.recalculate_floating_position().await;
                });
            },
        );

        let mut monitor_sig = self.state.position_monitor;
        monitor_sig.set(Some(watcher));

        let runtime = self.clone();
        spawn(async move {
            runtime.recalculate_floating_position().await;
        });
    }

    pub fn stop_position_monitor(&self) {
        let mut pm_sig = self.state.position_monitor;
        pm_sig.set(None);
    }

    pub async fn recalculate_floating_position(&self) {
        let trigger_id = self.relationships().trigger_id();
        let content_id = self.relationships().content_id();
        let boundary_id = self.collision_boundary();
        let custom_anchor_id = self.custom_anchor();

        if let Some(arr) = crate::foundation::browser::measure_floating_placement(
            trigger_id,
            content_id,
            custom_anchor_id.as_deref(),
            boundary_id.as_deref(),
        )
        .await
        {
            let t_x = arr[0] as f32;
            let t_y = arr[1] as f32;
            let t_w = arr[2] as f32;
            let t_h = arr[3] as f32;
            let c_w = arr[4] as f32;
            let c_h = arr[5] as f32;
            let padding = self.collision_padding();
            let b_left = arr[6] as f32 + padding;
            let b_top = arr[7] as f32 + padding;
            let b_right = (arr[8] as f32 - padding).max(b_left);
            let b_bottom = (arr[9] as f32 - padding).max(b_top);

            let anchor_rect = Rect::new(t_x - b_left, t_y - b_top, t_w, t_h);
            let content_size = Size::new(c_w, c_h);
            let available_size = Size::new(b_right - b_left, b_bottom - b_top);

            let preferred_side = self.preferred_side();
            let preferred_align = self.preferred_align();
            let avoid_collisions = self.avoid_collisions();
            let hide_when_detached = self.hide_when_detached();
            let side_offset = self.side_offset();
            let align_offset = self.align_offset();

            let layer = FloatingLayer::new(preferred_side)
                .with_align(preferred_align)
                .with_side_offset(side_offset)
                .with_align_offset(align_offset)
                .with_hide_when_detached(hide_when_detached);

            let computed =
                layer.position_with_available_size(anchor_rect, content_size, available_size);

            if avoid_collisions {
                let mut side_sig = self.state.side;
                if *side_sig.peek() != computed.side() {
                    side_sig.set(computed.side());
                }
                let mut align_sig = self.state.align;
                if *align_sig.peek() != computed.align() {
                    align_sig.set(computed.align());
                }
            } else {
                let mut side_sig = self.state.side;
                if *side_sig.peek() != preferred_side {
                    side_sig.set(preferred_side);
                }
                let mut align_sig = self.state.align;
                if *align_sig.peek() != preferred_align {
                    align_sig.set(preferred_align);
                }
            }
            let is_sticky_always = self.sticky().as_deref() == Some("always");
            let ref_hidden = if is_sticky_always {
                false
            } else {
                computed.reference_hidden()
            };
            self.set_reference_hidden(ref_hidden);
        }
    }

    pub fn start_form_reset_monitor(&self) {
        self.stop_form_reset_monitor();

        let trigger_id = self.relationships().trigger_id().to_owned();
        let form_id = self.select.form().map(str::to_owned);

        let runtime = self.clone();
        let watcher = start_form_reset_monitor(&trigger_id, form_id.as_deref(), move || {
            runtime.handle_form_reset();
        });
        let mut frm_sig = self.state.form_reset_monitor;
        frm_sig.set(Some(watcher));
    }

    pub fn stop_form_reset_monitor(&self) {
        let mut frm_sig = self.state.form_reset_monitor;
        frm_sig.set(None);
    }

    pub fn handle_form_reset(&self) {
        match self.select.mode() {
            SelectMode::Single { .. } => {
                let default_val = self.default_value();
                let mut val_sig = self.state.value;
                val_sig.set(default_val.clone());
                if let Some(ref cb) = self.on_value_change {
                    cb(default_val);
                }
            }
            SelectMode::Multiple => {
                let default_vals = self.default_values();
                let mut vals_sig = self.state.values;
                vals_sig.set(default_vals.clone());
                if let Some(ref cb) = self.on_values_change {
                    cb(default_vals);
                }
            }
        }
    }
}

fn sync_select_presence(content_id: &str, mut state: SelectRuntimeState, open: bool) {
    let update = state
        .presence_lane
        .with_mut(|lane| lane.sync(open));

    if update.invalidated_close_cycle().is_some() || !update.should_render() {
        stop_select_presence_monitor(state, content_id);
    }

    if let Some(cycle_id) = update.started_close_cycle() {
        start_select_presence_monitor(content_id, state, cycle_id);
    }
}

fn start_select_presence_monitor(
    content_id: &str,
    state: SelectRuntimeState,
    cycle_id: PresenceCloseCycleId,
) {
    stop_select_presence_monitor(state, content_id);

    let watcher = start_presence_monitor(content_id, cycle_id, move |event| {
        match event {
            PresenceMonitorEvent::Fallback {
                cycle_id: event_cycle,
                ..
            }
            | PresenceMonitorEvent::AnimationEnd {
                cycle_id: event_cycle,
                ..
            }
            | PresenceMonitorEvent::AnimationCancel {
                cycle_id: event_cycle,
                ..
            } => {
                complete_select_presence_close_cycle(state, event_cycle);
            }
            PresenceMonitorEvent::Stopped {
                cycle_id: event_cycle,
            } => {
                if state
                    .presence_monitor
                    .cycle_id
                    .with_peek(|current| *current == Some(event_cycle))
                {
                    clear_select_presence_monitor_state(state);
                }
            }
        }
    });
    let mut active_monitor = state.presence_monitor.monitor;
    active_monitor.set(Some(watcher));
    let mut active_cycle_id = state.presence_monitor.cycle_id;
    active_cycle_id.set(Some(cycle_id));
}

fn stop_select_presence_monitor(state: SelectRuntimeState, _content_id: &str) {
    clear_select_presence_monitor_state(state);
}

fn clear_select_presence_monitor_state(state: SelectRuntimeState) {
    let mut active_monitor = state.presence_monitor.monitor;
    active_monitor.set(None);
    let mut active_cycle_id = state.presence_monitor.cycle_id;
    active_cycle_id.set(None);
}

fn complete_select_presence_close_cycle(
    mut state: SelectRuntimeState,
    cycle_id: PresenceCloseCycleId,
) {
    clear_select_presence_monitor_state(state);
    let _ = state
        .presence_lane
        .with_mut(|lane| lane.complete_close_cycle(cycle_id));
}

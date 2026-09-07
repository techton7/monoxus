use std::rc::Rc;

use dioxus::document::Eval;
use dioxus::prelude::*;

#[allow(unused_imports)]
use crate::foundation::{
    browser::{
        recv_document_dismiss_event, recv_floating_auto_update_event, recv_form_reset_event,
        restore_focus_element_by_id, start_document_dismiss_monitor,
        start_floating_auto_update_monitor, start_form_reset_monitor,
        stop_document_dismiss_monitor, stop_floating_auto_update_monitor,
        stop_form_reset_monitor, DocumentDismissEvent, FloatingAutoUpdateEvent, FormResetEvent,
    },
    overlay::{FloatingLayer, PlacementAlign, PlacementSide, Rect, Size},
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
    pub highlighted_value: Signal<Option<String>>,
    pub items: Signal<Vec<SelectItemData>>,
    pub typeahead_buffer: Signal<String>,
    pub last_key_timestamp_ms: Signal<f64>,
    pub side: Signal<PlacementSide>,
    pub align: Signal<PlacementAlign>,
    pub side_offset: Signal<f32>,
    pub align_offset: Signal<f32>,
    pub avoid_collisions: Signal<bool>,
    pub hide_when_detached: Signal<bool>,
    pub collision_padding: Signal<f32>,
    pub reference_hidden: Signal<bool>,
    pub escape_prevented: Signal<bool>,
    pub on_pointer_down_outside: Signal<Option<EventHandler<()>>>,
    pub on_close_auto_focus: Signal<Option<EventHandler<()>>>,
    pub dismiss_monitor: Signal<Option<Eval>>,
    pub dismiss_loop_token: Signal<u64>,
    pub position_monitor: Signal<Option<Eval>>,
    pub position_loop_token: Signal<u64>,
    pub form_reset_monitor: Signal<Option<Eval>>,
    pub form_reset_loop_token: Signal<u64>,
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
        side: use_signal(|| PlacementSide::Bottom),
        align: use_signal(|| PlacementAlign::Start),
        side_offset: use_signal(|| 4.0),
        align_offset: use_signal(|| 0.0),
        avoid_collisions: use_signal(|| true),
        hide_when_detached: use_signal(|| false),
        collision_padding: use_signal(|| 0.0),
        reference_hidden: use_signal(|| false),
        escape_prevented: use_signal(|| false),
        on_pointer_down_outside: use_signal(|| None),
        on_close_auto_focus: use_signal(|| None),
        dismiss_monitor: use_signal(|| None),
        dismiss_loop_token: use_signal(|| 0),
        position_monitor: use_signal(|| None),
        position_loop_token: use_signal(|| 0),
        form_reset_monitor: use_signal(|| None),
        form_reset_loop_token: use_signal(|| 0),
    };

    let cleanup_state = state;
    dioxus::core::use_drop(move || {
        if let Some(monitor) = cleanup_state.dismiss_monitor.peek().clone() {
            let _ = stop_document_dismiss_monitor(monitor);
        }
        if let Some(monitor) = cleanup_state.position_monitor.peek().clone() {
            let _ = stop_floating_auto_update_monitor(monitor);
        }
        if let Some(monitor) = cleanup_state.form_reset_monitor.peek().clone() {
            let _ = stop_form_reset_monitor(monitor);
        }
    });

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
            hl_sig.set(val);
        }
    }

    pub fn side(&self) -> PlacementSide {
        *self.state.side.read()
    }

    pub fn set_side(&self, side: PlacementSide) {
        let mut side_sig = self.state.side;
        if *side_sig.peek() != side {
            side_sig.set(side);
        }
    }

    pub fn align(&self) -> PlacementAlign {
        *self.state.align.read()
    }

    pub fn set_align(&self, align: PlacementAlign) {
        let mut align_sig = self.state.align;
        if *align_sig.peek() != align {
            align_sig.set(align);
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

    pub fn collision_padding(&self) -> f32 {
        *self.state.collision_padding.read()
    }

    pub fn set_collision_padding(&self, padding: f32) {
        let mut sig = self.state.collision_padding;
        if *sig.peek() != padding {
            sig.set(padding);
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

    pub fn set_on_pointer_down_outside(&self, cb: Option<EventHandler<()>>) {
        let mut sig = self.state.on_pointer_down_outside;
        sig.set(cb);
    }

    pub fn set_on_close_auto_focus(&self, cb: Option<EventHandler<()>>) {
        let mut sig = self.state.on_close_auto_focus;
        sig.set(cb);
    }

    pub fn trigger_pointer_down_outside(&self) {
        if let Some(cb) = self.state.on_pointer_down_outside.read().clone() {
            cb.call(());
        }
    }

    pub fn trigger_close_auto_focus(&self) {
        if let Some(cb) = self.state.on_close_auto_focus.read().clone() {
            cb.call(());
        } else {
            restore_focus_element_by_id(self.relationships().trigger_id());
        }
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
            .or_else(|| {
                items
                    .iter()
                    .find(|i| !i.disabled)
                    .map(|i| i.value.clone())
            });

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
        self.content_attributes_with_side_and_align(
            activedescendant,
            self.side(),
            self.align(),
        )
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

        let mut token_sig = self.state.dismiss_loop_token;
        let next_token = token_sig.peek().saturating_add(1);
        token_sig.set(next_token);

        let monitor = start_document_dismiss_monitor();
        let mut dm_sig = self.state.dismiss_monitor;
        dm_sig.set(Some(monitor.clone()));

        let runtime = self.clone();
        let trigger_id = self.relationships().trigger_id().to_owned();
        let content_id = self.relationships().content_id().to_owned();

        spawn(async move {
            let mut monitor = monitor;
            loop {
                if *runtime.state.dismiss_loop_token.peek() != next_token {
                    break;
                }

                match recv_document_dismiss_event(&mut monitor).await {
                    Ok(DocumentDismissEvent::Stopped) => break,
                    Ok(DocumentDismissEvent::Escape) => {
                        if runtime.is_escape_prevented() {
                            runtime.set_escape_prevented(false);
                            continue;
                        }
                        runtime.close_dropdown();
                        break;
                    }
                    Ok(DocumentDismissEvent::PointerDown { path_ids }) => {
                        let is_inside =
                            path_ids.iter().any(|id| id == &trigger_id || id == &content_id);
                        if !is_inside {
                            runtime.trigger_pointer_down_outside();
                            runtime.close_dropdown_without_restore();
                            break;
                        }
                    }
                    Ok(DocumentDismissEvent::FocusIn { path_ids }) => {
                        let is_inside =
                            path_ids.iter().any(|id| id == &trigger_id || id == &content_id);
                        if !is_inside {
                            runtime.close_dropdown_without_restore();
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    pub fn stop_dismiss_monitor(&self) {
        let current_monitor = self.state.dismiss_monitor.peek().clone();
        if let Some(monitor) = current_monitor {
            let mut dm_sig = self.state.dismiss_monitor;
            dm_sig.set(None);
            let _ = stop_document_dismiss_monitor(monitor);
        }
    }

    pub fn start_position_monitor(&self) {
        self.stop_position_monitor();

        let trigger_id = self.relationships().trigger_id().to_owned();
        let content_id = self.relationships().content_id().to_owned();
        let monitor = start_floating_auto_update_monitor(&[&trigger_id], &content_id);

        let mut monitor_sig = self.state.position_monitor;
        monitor_sig.set(Some(monitor.clone()));

        let mut token_sig = self.state.position_loop_token;
        let next_token = token_sig.peek().saturating_add(1);
        token_sig.set(next_token);

        let runtime = self.clone();
        spawn(async move {
            let mut monitor = monitor;
            loop {
                if *runtime.state.position_loop_token.peek() != next_token {
                    break;
                }
                match recv_floating_auto_update_event(&mut monitor).await {
                    Ok(FloatingAutoUpdateEvent::Scroll) | Ok(FloatingAutoUpdateEvent::Update) => {
                        runtime.recalculate_floating_position().await;
                    }
                    Ok(FloatingAutoUpdateEvent::Stopped) | Err(_) => break,
                }
            }
        });
    }

    pub fn stop_position_monitor(&self) {
        let current_monitor = self.state.position_monitor.peek().clone();
        if let Some(monitor) = current_monitor {
            let mut pm_sig = self.state.position_monitor;
            pm_sig.set(None);
            let _ = stop_floating_auto_update_monitor(monitor);
        }
    }

    pub async fn recalculate_floating_position(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let trigger_id = self.relationships().trigger_id().to_owned();
            let content_id = self.relationships().content_id().to_owned();
            let script = format!(
                r#"(function() {{
                    const trigger = document.getElementById({trigger_id:?});
                    const content = document.getElementById({content_id:?});
                    if (!trigger || !content) return null;
                    const tr = trigger.getBoundingClientRect();
                    const cr = content.getBoundingClientRect();
                    return [tr.left, tr.top, tr.width, tr.height, cr.width, cr.height, window.innerWidth, window.innerHeight];
                }})()"#
            );

            if let Ok(val) = js_sys::eval(&script) {
                if !val.is_null() && !val.is_undefined() {
                    let arr = js_sys::Array::from(&val);
                    if arr.length() == 8 {
                        let t_x = arr.get(0).as_f64().unwrap_or(0.0) as f32;
                        let t_y = arr.get(1).as_f64().unwrap_or(0.0) as f32;
                        let t_w = arr.get(2).as_f64().unwrap_or(0.0) as f32;
                        let t_h = arr.get(3).as_f64().unwrap_or(0.0) as f32;
                        let c_w = arr.get(4).as_f64().unwrap_or(0.0) as f32;
                        let c_h = arr.get(5).as_f64().unwrap_or(0.0) as f32;
                        let win_w = arr.get(6).as_f64().unwrap_or(1024.0) as f32;
                        let win_h = arr.get(7).as_f64().unwrap_or(768.0) as f32;

                        let anchor_rect = Rect::new(t_x, t_y, t_w, t_h);
                        let content_size = Size::new(c_w, c_h);
                        let viewport_size = Size::new(win_w, win_h);

                        let preferred_side = self.side();
                        let avoid_collisions = self.avoid_collisions();
                        let hide_when_detached = self.hide_when_detached();
                        let side_offset = self.side_offset();
                        let align_offset = self.align_offset();
                        let target_align = self.align();

                        let layer = FloatingLayer::new(preferred_side)
                            .with_align(target_align)
                            .with_side_offset(side_offset)
                            .with_align_offset(align_offset)
                            .with_hide_when_detached(hide_when_detached);

                        let computed = layer.position_with_available_size(
                            anchor_rect,
                            content_size,
                            viewport_size,
                        );

                        if avoid_collisions {
                            self.set_side(computed.side());
                            self.set_align(computed.align());
                        }
                        self.set_reference_hidden(computed.reference_hidden());
                    }
                }
            }
        }
    }

    pub fn start_form_reset_monitor(&self) {
        self.stop_form_reset_monitor();

        let trigger_id = self.relationships().trigger_id().to_owned();
        let form_id = self.select.form().map(str::to_owned);

        let mut token_sig = self.state.form_reset_loop_token;
        let next_token = token_sig.peek().saturating_add(1);
        token_sig.set(next_token);

        let monitor = start_form_reset_monitor(&trigger_id, form_id.as_deref());
        let mut frm_sig = self.state.form_reset_monitor;
        frm_sig.set(Some(monitor.clone()));

        let runtime = self.clone();
        spawn(async move {
            let mut monitor = monitor;
            loop {
                if *runtime.state.form_reset_loop_token.peek() != next_token {
                    break;
                }
                match recv_form_reset_event(&mut monitor).await {
                    Ok(FormResetEvent::Reset) => {
                        runtime.handle_form_reset();
                    }
                    Ok(FormResetEvent::Stopped) | Err(_) => break,
                }
            }
        });
    }

    pub fn stop_form_reset_monitor(&self) {
        let current_monitor = self.state.form_reset_monitor.peek().clone();
        if let Some(monitor) = current_monitor {
            let mut frm_sig = self.state.form_reset_monitor;
            frm_sig.set(None);
            let _ = stop_form_reset_monitor(monitor);
        }
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

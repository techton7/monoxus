use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        start_document_dismiss_monitor_with_boundaries,
        start_floating_auto_update_monitor, start_form_reset_monitor, DocumentDismissEvent,
    },
    overlay::{FloatingLayer, FloatingReadiness, Rect, Size},
};

use super::{runtime::SelectRuntime, types::SelectMode};

struct SyntheticEscapeKey;

impl dioxus::html::ModifiersInteraction for SyntheticEscapeKey {
    fn modifiers(&self) -> dioxus::html::input_data::keyboard_types::Modifiers {
        dioxus::html::input_data::keyboard_types::Modifiers::empty()
    }
}

impl dioxus::events::HasKeyboardData for SyntheticEscapeKey {
    fn key(&self) -> dioxus::html::input_data::keyboard_types::Key {
        dioxus::html::input_data::keyboard_types::Key::Escape
    }

    fn code(&self) -> dioxus::html::input_data::keyboard_types::Code {
        dioxus::html::input_data::keyboard_types::Code::Escape
    }

    fn location(&self) -> dioxus::html::input_data::keyboard_types::Location {
        dioxus::html::input_data::keyboard_types::Location::Standard
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

impl SelectRuntime {
    pub fn start_dismiss_monitor(&self) {
        self.stop_dismiss_monitor();

        let runtime = self.clone();
        let trigger_id = self.relationships().trigger_id().to_owned();
        let content_id = self.relationships().content_id().to_owned();

        let boundaries = [trigger_id.as_str(), content_id.as_str()];
        let trigger_id_clone = trigger_id.clone();
        let content_id_clone = content_id.clone();
        let watcher = start_document_dismiss_monitor_with_boundaries(&boundaries, move |event| {
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
                        .any(|id| id == &trigger_id_clone || id == &content_id_clone);
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
                        .any(|id| id == &trigger_id_clone || id == &content_id_clone);
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

            let mut placement_sig = self.state.placement;
            placement_sig.set(Some(computed));
            let mut readiness_sig = self.state.content_readiness;
            readiness_sig.set(FloatingReadiness::Ready);
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

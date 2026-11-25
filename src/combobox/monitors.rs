use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::keyboard_types::{Code, Key, Location, Modifiers, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        DocumentDismissEvent, get_element_client_rect, get_element_scroll_metrics,
        set_element_scroll_position, spawn_root_task,
        start_document_dismiss_monitor_with_boundaries, start_floating_auto_update_monitor,
    },
    overlay::{
        FloatingLayer, FloatingReadiness, OverlayPositionConfig, Rect, Size,
        compute_overlay_position, is_topmost_overlay_layer,
    },
};

use super::{
    runtime::ComboboxRuntime,
    state::compute_viewport_scroll_target,
    types::ComboboxScrollAlignment,
};

struct SyntheticEscapeKey;

impl dioxus::html::ModifiersInteraction for SyntheticEscapeKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}

impl dioxus::events::HasKeyboardData for SyntheticEscapeKey {
    fn key(&self) -> Key {
        Key::Named(NamedKey::Escape)
    }

    fn code(&self) -> Code {
        Code::Escape
    }

    fn location(&self) -> Location {
        Location::Standard
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
pub struct ComboboxInteractOutsideEvent {
    prevented: Rc<Cell<bool>>,
}

impl ComboboxInteractOutsideEvent {
    pub fn new() -> Self {
        Self {
            prevented: Rc::new(Cell::new(false)),
        }
    }

    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }

    pub fn default_action_enabled(&self) -> bool {
        !self.prevented.get()
    }
}

impl Default for ComboboxInteractOutsideEvent {
    fn default() -> Self {
        Self::new()
    }
}

impl ComboboxRuntime {
    pub fn start_dismiss_monitor(&self) {
        self.stop_dismiss_monitor();

        let runtime = self.clone();
        let input_id = self.current_input_id_untracked();
        let trigger_id = self.current_trigger_id_untracked();
        let content_id = self.current_content_id_untracked();

        let boundaries = [input_id.as_str(), trigger_id.as_str(), content_id.as_str()];
        let input_id_clone = input_id.clone();
        let trigger_id_clone = trigger_id.clone();
        let content_id_clone = content_id.clone();

        let watcher =
            start_document_dismiss_monitor_with_boundaries(&boundaries, move |event| match event {
                DocumentDismissEvent::Escape => {
                    if !is_topmost_overlay_layer(&content_id_clone) {
                        return;
                    }
                    if let Ok(Some(cb)) =
                        runtime.state.on_escape_key_down.try_peek().map(|g| g.clone())
                    {
                        let synth = SyntheticEscapeKey;
                        let kb_data = dioxus::html::KeyboardData::new(synth);
                        let evt = dioxus::core::Event::new(Rc::new(kb_data), true);
                        cb.call(evt.clone());
                        if !evt.default_action_enabled() {
                            return;
                        }
                    }
                    // Escape closes without reverting `input_value`.
                    runtime.close_menu();
                }
                DocumentDismissEvent::PointerDown { path_ids }
                | DocumentDismissEvent::Click { path_ids } => {
                    let is_inside = path_ids.iter().any(|id| {
                        id == &input_id_clone
                            || id == &trigger_id_clone
                            || id == &content_id_clone
                    });
                    if !is_inside {
                        if let Ok(Some(cb)) =
                            runtime.state.on_interact_outside.try_peek().map(|g| g.clone())
                        {
                            let evt = ComboboxInteractOutsideEvent::new();
                            cb.call(evt.clone());
                            if !evt.default_action_enabled() {
                                return;
                            }
                        }
                        runtime.close_menu();
                    }
                }
                DocumentDismissEvent::FocusIn { path_ids } => {
                    let is_inside = path_ids.iter().any(|id| {
                        id == &input_id_clone
                            || id == &trigger_id_clone
                            || id == &content_id_clone
                    });
                    if !is_inside {
                        runtime.close_menu();
                    }
                }
            });

        let mut dm_sig = self.state.dismiss_monitor;
        let _ = dm_sig.try_write().map(|mut w| *w = Some(watcher));
    }

    pub fn stop_dismiss_monitor(&self) {
        let mut dm_sig = self.state.dismiss_monitor;
        if let Ok(mut dm) = dm_sig.try_write() {
            *dm = None;
        }
    }

    pub fn start_position_monitor(&self) {
        self.stop_position_monitor();

        let anchor_id = self
            .custom_anchor()
            .unwrap_or_else(|| self.current_input_id_untracked());
        let content_id = self.current_content_id_untracked();

        let runtime = self.clone();
        let watcher =
            start_floating_auto_update_monitor(&[&anchor_id], &content_id, move |_event| {
                if !runtime.recalculate_floating_position_sync() {
                    let runtime = runtime.clone();
                    spawn_root_task(async move {
                        runtime.recalculate_floating_position().await;
                    });
                }
            });

        let mut pm_sig = self.state.position_monitor;
        let _ = pm_sig.try_write().map(|mut w| *w = Some(watcher));

        self.recalculate_floating_position_sync();
    }

    pub fn stop_position_monitor(&self) {
        let mut pm_sig = self.state.position_monitor;
        if let Ok(mut pm) = pm_sig.try_write() {
            *pm = None;
        }
    }

    pub fn recalculate_floating_position_sync(&self) -> bool {
        let default_anchor = self.current_input_id_untracked();
        let custom_anchor_id = self.custom_anchor();
        let anchor_id = custom_anchor_id.as_deref().unwrap_or(&default_anchor);
        let content_id = self.current_content_id_untracked();
        let boundary_id = self.collision_boundary();

        let config = OverlayPositionConfig {
            anchor_id,
            content_id: &content_id,
            arrow_id: None,
            boundary_id: boundary_id.as_deref(),
            side: self.preferred_side(),
            align: self.preferred_align(),
            side_offset: self.side_offset(),
            align_offset: self.align_offset(),
            avoid_collisions: self.avoid_collisions(),
            collision_padding: self.collision_padding(),
            arrow_padding: self.arrow_padding(),
            hide_when_detached: self.hide_when_detached(),
            sticky: self.sticky().as_deref() == Some("always"),
            namespace: Some("combobox"),
        };

        if let Some(computed) = compute_overlay_position(&config) {
            let mut side_sig = self.state.side;
            if side_sig.try_peek().is_ok_and(|g| *g != computed.side()) {
                let _ = side_sig.try_write().map(|mut w| *w = computed.side());
            }
            let mut align_sig = self.state.align;
            if align_sig.try_peek().is_ok_and(|g| *g != computed.align()) {
                let _ = align_sig.try_write().map(|mut w| *w = computed.align());
            }

            let geom = computed.geometry();
            let next_readiness = if geom.content_width() > 0.0 && geom.content_height() > 0.0 {
                FloatingReadiness::Ready
            } else {
                FloatingReadiness::Measuring
            };
            let is_ready = next_readiness == FloatingReadiness::Ready;
            let mut readiness_sig = self.state.content_readiness;
            if readiness_sig.try_peek().is_ok_and(|g| *g != next_readiness) {
                let _ = readiness_sig.try_write().map(|mut w| *w = next_readiness);
            }

            let mut placement_sig = self.state.placement;
            if placement_sig
                .try_peek()
                .is_ok_and(|g| g.as_ref() != Some(&computed))
            {
                let _ = placement_sig.try_write().map(|mut w| *w = Some(computed));
            }
            is_ready
        } else {
            false
        }
    }

    pub async fn recalculate_floating_position(&self) {
        if self.recalculate_floating_position_sync() {
            return;
        }

        let input_handle_opt = self
            .state
            .input_handle
            .try_peek()
            .ok()
            .and_then(|h| h.clone());
        let content_handle_opt = self
            .state
            .content_handle
            .try_peek()
            .ok()
            .and_then(|h| h.clone());
        if let (Some(ih), Some(ch)) = (input_handle_opt, content_handle_opt) {
            if let (Ok(in_rect), Ok(co_rect)) =
                (ih.get_client_rect().await, ch.get_client_rect().await)
            {
                let anchor = Rect::new(
                    in_rect.origin.x as f32,
                    in_rect.origin.y as f32,
                    in_rect.width() as f32,
                    in_rect.height() as f32,
                );
                let content_size = Size::new(co_rect.width() as f32, co_rect.height() as f32);
                let avail_size = crate::foundation::browser::get_viewport_size()
                    .await
                    .ok()
                    .map(|vp| Size::new(vp[0] as f32, vp[1] as f32));
                let placement = match avail_size {
                    Some(avail) => FloatingLayer::new(self.preferred_side())
                        .with_align(self.preferred_align())
                        .with_side_offset(self.side_offset())
                        .with_align_offset(self.align_offset())
                        .with_available_space(avail)
                        .position_with_available_size(anchor, content_size, avail),
                    None => FloatingLayer::new(self.preferred_side())
                        .with_align(self.preferred_align())
                        .with_side_offset(self.side_offset())
                        .with_align_offset(self.align_offset())
                        .position(anchor, content_size),
                };

                let mut side_sig = self.state.side;
                if side_sig.try_peek().is_ok_and(|g| *g != placement.side()) {
                    let _ = side_sig.try_write().map(|mut w| *w = placement.side());
                }
                let mut align_sig = self.state.align;
                if align_sig.try_peek().is_ok_and(|g| *g != placement.align()) {
                    let _ = align_sig.try_write().map(|mut w| *w = placement.align());
                }

                let geom = placement.geometry();
                let next_readiness = if geom.content_width() > 0.0 && geom.content_height() > 0.0 {
                    FloatingReadiness::Ready
                } else {
                    FloatingReadiness::Measuring
                };
                let mut readiness_sig = self.state.content_readiness;
                if readiness_sig.try_peek().is_ok_and(|g| *g != next_readiness) {
                    let _ = readiness_sig.try_write().map(|mut w| *w = next_readiness);
                }

                let mut placement_sig = self.state.placement;
                if placement_sig
                    .try_peek()
                    .is_ok_and(|g| g.as_ref() != Some(&placement))
                {
                    let _ = placement_sig.try_write().map(|mut w| *w = Some(placement));
                }
            }
        }
    }
}

pub(crate) fn scroll_item_in_viewport(
    viewport_id: &str,
    fallback_content_id: &str,
    item_id: &str,
    alignment: ComboboxScrollAlignment,
) {
    let viewport_id = viewport_id.to_owned();
    let fallback_content_id = fallback_content_id.to_owned();
    let item_id = item_id.to_owned();

    spawn_root_task(async move {
        for attempt in 0..3 {
            if try_scroll_item_in_viewport(
                &viewport_id,
                &fallback_content_id,
                &item_id,
                alignment,
            ) {
                break;
            }
            if attempt < 2 {
                let _ = oxidase::frame::next_frame().await;
            }
        }
    });
}

fn try_scroll_item_in_viewport(
    viewport_id: &str,
    fallback_content_id: &str,
    item_id: &str,
    alignment: ComboboxScrollAlignment,
) -> bool {
    let container_id = if get_element_client_rect(viewport_id).is_some() {
        viewport_id
    } else {
        fallback_content_id
    };

    let Some(container_rect) = get_element_client_rect(container_id) else {
        return false;
    };
    let Some(item_rect) = get_element_client_rect(item_id) else {
        return false;
    };
    let metrics = get_element_scroll_metrics(container_id).unwrap_or_default();

    if let Some(target_top) = compute_viewport_scroll_target(
        container_rect.top(),
        container_rect.bottom(),
        container_rect.height,
        metrics.scroll_top,
        item_rect.top(),
        item_rect.bottom(),
        item_rect.height,
        alignment,
    ) {
        set_element_scroll_position(container_id, metrics.scroll_left, target_top);
    }
    true
}

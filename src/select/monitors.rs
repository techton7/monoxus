use dioxus::prelude::*;

use crate::foundation::overlay::{
    FloatingReadiness, OverlayPositionConfig, compute_overlay_position,
};

use super::{runtime::SelectRuntime, types::SelectMode};

impl SelectRuntime {
    pub fn start_dismiss_monitor(&self) {
        if let Some(d) = self.state.dismiss {
            d.reset_interaction();
        }
    }

    pub fn stop_dismiss_monitor(&self) {
        if let Some(d) = self.state.dismiss {
            d.stop();
        }
    }

    pub fn start_position_monitor(&self) {
        if let Some(f) = self.state.floating {
            f.refresh();
        } else {
            let _ = self.recalculate_floating_position_sync();
        }
    }

    pub fn stop_position_monitor(&self) {
        if let Some(f) = self.state.floating {
            f.clear();
        }
    }

    pub fn recalculate_floating_position_sync(&self) -> bool {
        if let Some(f) = self.state.floating {
            return f.recalculate_sync();
        }

        let trigger_id = self.relationships().trigger_id();
        let content_id = self.relationships().content_id();
        let boundary_id = self.collision_boundary();
        let custom_anchor_id = self.custom_anchor();
        let anchor_id = custom_anchor_id.as_deref().unwrap_or(trigger_id);
        let arrow_id = self.arrow_id();

        let config = OverlayPositionConfig {
            anchor_id,
            content_id,
            arrow_id: arrow_id.as_deref(),
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
            namespace: Some("select"),
        };

        if let Some(computed) = compute_overlay_position(&config) {
            let geom = computed.geometry();
            let next_readiness = if geom.content_width() > 0.0 && geom.content_height() > 0.0 {
                FloatingReadiness::Ready
            } else {
                FloatingReadiness::Measuring
            };
            let is_ready = next_readiness == FloatingReadiness::Ready;
            let mut readiness_sig = self.state.fallback_readiness;
            if readiness_sig.try_peek().is_ok_and(|cur| *cur != next_readiness) {
                let _ = readiness_sig.try_write().map(|mut w| *w = next_readiness);
            }
            self.set_reference_hidden(computed.reference_hidden());
            is_ready
        } else {
            false
        }
    }

    pub async fn recalculate_floating_position(&self) {
        if let Some(f) = self.state.floating {
            f.recalculate_with_async_fallback().await;
        } else {
            let _ = self.recalculate_floating_position_sync();
        }
    }

    pub fn handle_form_reset(&self) {
        let mode = self.select.read().mode();
        match mode {
            SelectMode::Single { .. } => {
                let default_val = self.default_value();
                let mut val_sig = self.state.value;
                val_sig.set(default_val.clone());
                if let Some(cb) = self.on_value_change.try_read().ok().and_then(|g| g.clone()) {
                    cb(default_val);
                }
            }
            SelectMode::Multiple => {
                let default_vals = self.default_values();
                let mut vals_sig = self.state.values;
                vals_sig.set(default_vals.clone());
                if let Some(cb) = self.on_values_change.try_read().ok().and_then(|g| g.clone()) {
                    cb(default_vals);
                }
            }
        }
    }
}

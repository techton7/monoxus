use dioxus::prelude::*;

use crate::foundation::{
    browser::{restore_body_user_select, set_body_user_select_none},
    overlay::Rect2D,
    shared::use_direction_context,
    state::{ControllableSignal, sync_signal, use_controllable_signal_by},
};

use super::{
    attrs::{
        SliderRangeAttributes, SliderRootAttributes, SliderThumbAttributes,
        SliderThumbLabelAttributes, SliderTickAttributes, SliderTickLabelAttributes,
        SliderTrackAttributes, resolve_thumb_aria_label,
    },
    relationships::SliderRelationships,
    state::{
        SliderDefinition, get_contained_percent, get_next_custom_step_value, get_next_step_value,
        percent_to_value, resolve_keyboard_step_direction, update_thumb_value, value_to_percent,
    },
    types::{Direction, Orientation, SliderLabelPosition, SliderValue},
};

fn f64_vec_bits_eq(a: &Vec<f64>, b: &Vec<f64>) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|(lhs, rhs)| lhs.to_bits() == rhs.to_bits())
}

fn sanitize_slider_values(values: Vec<f64>, fallback: f64) -> Vec<f64> {
    let safe_fallback = if fallback.is_finite() { fallback } else { 0.0 };
    if values.is_empty() {
        return vec![safe_fallback];
    }
    values
        .into_iter()
        .map(|v| if v.is_finite() { v } else { safe_fallback })
        .collect()
}

/// Finds the index of the thumb closest in domain value to `target_value`.
pub fn find_closest_thumb(values: &[f64], target_value: f64) -> usize {
    if values.len() <= 1 {
        return 0;
    }
    let mut closest_idx = 0;
    let mut min_diff = (values[0] - target_value).abs();
    for (i, &v) in values.iter().enumerate().skip(1) {
        let diff = (v - target_value).abs();
        if diff < min_diff {
            min_diff = diff;
            closest_idx = i;
        }
    }
    closest_idx
}

/// Reactive runtime engine coordinating state, user interaction, and attribute publishing for a Slider.
#[derive(Clone)]
pub struct SliderRuntime {
    pub def: SliderDefinition,
    pub rels: SliderRelationships,
    pub values: Signal<Vec<f64>>,
    pub values_ctl: Option<ControllableSignal<Vec<f64>>>,
    pub is_single: bool,
    pub active_thumb: Signal<Option<usize>>,
    pub is_dragging: Signal<bool>,
    pub drag_start_values: Signal<Option<Vec<f64>>>,
    pub focused_thumb: Signal<Option<usize>>,
    pub on_value_change: CopyValue<Option<EventHandler<SliderValue>>>,
    pub on_value_commit: CopyValue<Option<EventHandler<SliderValue>>>,
    pub track_size: Signal<Option<f64>>,
    pub thumb_sizes: Signal<Vec<Option<f64>>>,
    pub track_id: Signal<String>,
    pub thumb_ids: Signal<Vec<Option<String>>>,
    pub thumb_drag_claimed: Signal<bool>,
}

impl SliderRuntime {
    fn export_value(&self, vals: Vec<f64>) -> SliderValue {
        if self.is_single {
            SliderValue::Single(vals.first().copied().unwrap_or(self.def.min))
        } else {
            SliderValue::Multiple(vals)
        }
    }

    fn emit_change(&self, vals: Vec<f64>) {
        if let Some(handler) = self.on_value_change.try_read().ok().and_then(|s| *s) {
            handler.call(self.export_value(vals));
        }
    }

    fn emit_commit(&self, vals: Vec<f64>) {
        if let Some(handler) = self.on_value_commit.try_read().ok().and_then(|s| *s) {
            handler.call(self.export_value(vals));
        }
    }

    /// Proposes `new_vals` through [`ControllableSignal`] (skipping internal signal mutation in
    /// controlled mode so rejected updates produce zero 1-frame flash, `FM-COL-1`).
    pub(crate) fn propose_values(&mut self, new_vals: Vec<f64>) -> bool {
        if let Some(ctl) = self.values_ctl {
            ctl.set_by(new_vals, f64_vec_bits_eq)
        } else {
            let changed = self
                .values
                .try_peek()
                .is_ok_and(|cur| !f64_vec_bits_eq(&cur, &new_vals));
            if changed {
                self.values.set(new_vals);
            }
            changed
        }
    }

    /// Resets slider values on enclosing `<form>` reset, respecting controlled ownership.
    pub(crate) fn reset_values(&mut self, initial_values: Vec<f64>) {
        if self.propose_values(initial_values.clone()) {
            self.emit_change(initial_values);
        }
    }

    /// Computes the normalized `[0.0, 1.0]` pointer fraction along the slider track given orientation,
    /// reading direction, and inversion.
    pub fn compute_pointer_fraction(&self, client_x: f64, client_y: f64, rect: &Rect2D) -> f64 {
        let fraction = match self.def.orientation {
            Orientation::Vertical => {
                let is_sliding_from_bottom = !self.def.inverted;
                let h = rect.height.max(1.0);
                let f = (rect.bottom() - client_y) / h;
                if is_sliding_from_bottom { f } else { 1.0 - f }
            }
            _ => {
                let is_sliding_from_left = (self.def.direction == Direction::Ltr
                    && !self.def.inverted)
                    || (self.def.direction == Direction::Rtl && self.def.inverted);
                let w = rect.width.max(1.0);
                let f = (client_x - rect.left()) / w;
                if is_sliding_from_left { f } else { 1.0 - f }
            }
        };
        fraction.clamp(0.0, 1.0)
    }

    fn tick_geometry(&self, value: f64) -> (f64, bool, bool, f64) {
        let values = self.current_values();
        let raw_pct = value_to_percent(value, self.def.min, self.def.max);

        let (range_min, range_max) = if values.len() <= 1 {
            let val = values.first().copied().unwrap_or(self.def.min);
            (self.def.min, val)
        } else {
            let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            (min_val, max_val)
        };

        let is_bounded = value >= range_min && value <= range_max;
        let is_selected = values.iter().any(|&v| (v - value).abs() < 1e-9);

        let t_size = *self.track_size.read();
        let th_size = self.thumb_sizes.read().first().copied().flatten();
        let effective_pct = get_contained_percent(
            raw_pct,
            self.def.thumb_alignment,
            self.def.track_padding,
            t_size,
            th_size,
        );

        (raw_pct, is_bounded, is_selected, effective_pct)
    }

    /// Returns the active track element id registered or resolved from relationships.
    pub fn resolved_track_id(&self) -> String {
        self.track_id.peek().clone()
    }

    /// Registers the DOM element id for the slider track.
    pub fn register_track_id(&mut self, id: String) {
        sync_signal(&mut self.track_id, id);
    }

    /// Returns the resolved thumb element id for the thumb at `idx`.
    pub fn resolved_thumb_id(&self, idx: usize) -> String {
        self.thumb_ids
            .peek()
            .get(idx)
            .and_then(|o| o.clone())
            .unwrap_or_else(|| self.rels.thumb_id(idx))
    }

    /// Registers a custom DOM element id for the thumb at `idx`.
    pub fn register_thumb_id(&mut self, idx: usize, id: String) {
        let mut ids = self.thumb_ids.peek().clone();
        if ids.len() <= idx {
            ids.resize(idx + 1, None);
        }
        if ids[idx].as_deref() != Some(&id) {
            ids[idx] = Some(id);
            self.thumb_ids.set(ids);
        }
    }

    /// Returns the current domain values as a slice.
    pub fn current_values(&self) -> Vec<f64> {
        (self.values)()
    }

    /// Derives attributes for [`SliderRoot`].
    pub fn root_attrs(&self) -> SliderRootAttributes {
        SliderRootAttributes::new(
            self.rels.root_id().to_string(),
            self.def.orientation,
            self.def.direction,
            self.def.disabled,
        )
    }

    /// Derives attributes for [`SliderTrack`].
    pub fn track_attrs(&self) -> SliderTrackAttributes {
        SliderTrackAttributes::new(
            self.rels.track_id().to_string(),
            self.def.orientation,
            self.def.disabled,
        )
    }

    /// Derives attributes for [`SliderRange`].
    ///
    /// In `Contain` mode, single-thumb ranges remain empty `(0.0, 0.0)` at `min`, fill `(0.0, 100.0)`
    /// at `max`, and align intermediate endpoints to the contained thumb center; multi-thumb ranges
    /// align both endpoints to their respective contained thumb centers so the fill stays centered
    /// beneath the thumbs across all values.
    pub fn range_attrs(&self) -> SliderRangeAttributes {
        let values = self.current_values();
        let t_size = *self.track_size.read();
        let thumb_sizes = self.thumb_sizes.read();

        let (start_pct, end_pct) = if values.len() <= 1 {
            let val = values.first().copied().unwrap_or(self.def.min);
            if val <= self.def.min {
                (0.0, 0.0)
            } else if val >= self.def.max {
                (0.0, 100.0)
            } else {
                let raw_pct = value_to_percent(val, self.def.min, self.def.max);
                let th_size = thumb_sizes.first().copied().flatten();
                let end = get_contained_percent(
                    raw_pct,
                    self.def.thumb_alignment,
                    self.def.track_padding,
                    t_size,
                    th_size,
                );
                (0.0, end)
            }
        } else {
            let (min_idx, min_val) = values
                .iter()
                .copied()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                .unwrap();
            let (max_idx, max_val) = values
                .iter()
                .copied()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .unwrap();
            let raw_start = value_to_percent(min_val, self.def.min, self.def.max);
            let raw_end = value_to_percent(max_val, self.def.min, self.def.max);
            let th_min = thumb_sizes.get(min_idx).copied().flatten();
            let th_max = thumb_sizes.get(max_idx).copied().flatten();
            let start = get_contained_percent(
                raw_start,
                self.def.thumb_alignment,
                self.def.track_padding,
                t_size,
                th_min,
            );
            let end = get_contained_percent(
                raw_end,
                self.def.thumb_alignment,
                self.def.track_padding,
                t_size,
                th_max,
            );
            (start, end)
        };

        SliderRangeAttributes::new(
            self.rels.range_id().to_string(),
            self.def.orientation,
            self.def.direction,
            self.def.inverted,
            self.def.disabled,
            start_pct,
            end_pct,
        )
    }

    /// Derives attributes for [`SliderThumb`] at the given index.
    pub fn thumb_attrs(
        &self,
        index: usize,
        user_aria_label: Option<String>,
        aria_labelledby: Option<String>,
        aria_valuetext: Option<String>,
    ) -> SliderThumbAttributes {
        let values = self.current_values();
        let val = values.get(index).copied().unwrap_or(self.def.min);
        let raw_pct = value_to_percent(val, self.def.min, self.def.max);

        let t_size = *self.track_size.read();
        let th_size = self.thumb_sizes.read().get(index).copied().flatten();
        let effective_pct = get_contained_percent(
            raw_pct,
            self.def.thumb_alignment,
            self.def.track_padding,
            t_size,
            th_size,
        );

        let is_active = *self.active_thumb.read() == Some(index);
        let aria_label = resolve_thumb_aria_label(user_aria_label.as_deref(), index, values.len());

        SliderThumbAttributes::new(
            self.rels.thumb_id(index),
            val,
            self.def.min,
            self.def.max,
            self.def.orientation,
            self.def.direction,
            self.def.inverted,
            self.def.disabled,
            is_active,
            effective_pct,
            aria_label,
            aria_labelledby,
            aria_valuetext,
        )
    }

    /// Derives attributes for [`SliderTick`].
    pub fn tick_attrs(&self, value: f64) -> SliderTickAttributes {
        let (raw_pct, is_bounded, is_selected, effective_pct) = self.tick_geometry(value);

        SliderTickAttributes::new(
            self.rels.tick_id((raw_pct * 100.0) as usize),
            value,
            self.def.orientation,
            self.def.direction,
            self.def.inverted,
            self.def.disabled,
            is_bounded,
            is_selected,
            effective_pct,
        )
    }

    /// Derives attributes for [`SliderTickLabel`].
    pub fn tick_label_attrs(
        &self,
        value: f64,
        position: Option<SliderLabelPosition>,
    ) -> SliderTickLabelAttributes {
        let (raw_pct, is_bounded, is_selected, effective_pct) = self.tick_geometry(value);
        let pos = position.unwrap_or(match self.def.orientation {
            Orientation::Vertical => SliderLabelPosition::Right,
            _ => SliderLabelPosition::Bottom,
        });

        SliderTickLabelAttributes::new(
            self.rels.tick_label_id((raw_pct * 100.0) as usize),
            value,
            pos,
            self.def.orientation,
            self.def.direction,
            self.def.inverted,
            self.def.disabled,
            is_bounded,
            is_selected,
            effective_pct,
        )
    }

    /// Derives attributes for [`SliderThumbLabel`].
    pub fn thumb_label_attrs(
        &self,
        index: usize,
        position: Option<SliderLabelPosition>,
    ) -> SliderThumbLabelAttributes {
        let values = self.current_values();
        let val = values.get(index).copied().unwrap_or(self.def.min);
        let raw_pct = value_to_percent(val, self.def.min, self.def.max);

        let is_active = *self.active_thumb.read() == Some(index);
        let pos = position.unwrap_or(match self.def.orientation {
            Orientation::Vertical => SliderLabelPosition::Left,
            _ => SliderLabelPosition::Top,
        });

        let t_size = *self.track_size.read();
        let th_size = self.thumb_sizes.read().get(index).copied().flatten();
        let effective_pct = get_contained_percent(
            raw_pct,
            self.def.thumb_alignment,
            self.def.track_padding,
            t_size,
            th_size,
        );

        SliderThumbLabelAttributes::new(
            self.rels.thumb_label_id(index),
            val,
            pos,
            self.def.orientation,
            self.def.direction,
            self.def.inverted,
            self.def.disabled,
            is_active,
            effective_pct,
        )
    }

    /// Starts a drag interaction for the thumb at `thumb_idx`.
    pub fn start_drag(&mut self, thumb_idx: usize) {
        if self.def.disabled {
            return;
        }
        self.is_dragging.set(true);
        self.active_thumb.set(Some(thumb_idx));
        self.drag_start_values.set(Some(self.values.peek().clone()));
        set_body_user_select_none();
        if dioxus::core::Runtime::try_current().is_some() {
            crate::foundation::browser::focus_element_by_id(&self.resolved_thumb_id(thumb_idx));
        }
    }

    /// Updates the active thumb's position from pointer coordinates against the measured rect.
    pub fn update_pointer_coord(&mut self, client_x: f64, client_y: f64, rect: &Rect2D) {
        if self.def.disabled || !*self.is_dragging.peek() {
            return;
        }
        let Some(active_idx) = *self.active_thumb.peek() else {
            return;
        };

        let percent = self.compute_pointer_fraction(client_x, client_y, rect) * 100.0;
        let raw_val = percent_to_value(percent, self.def.min, self.def.max);

        let current_vals = self.values.peek().clone();
        let (new_vals, new_active_idx) = update_thumb_value(
            &current_vals,
            active_idx,
            raw_val,
            self.def.min,
            self.def.max,
            self.def.step,
            self.def.steps.as_deref(),
            self.def.min_steps_between_thumbs,
            self.def.order_policy,
        );

        if self.propose_values(new_vals.clone()) {
            self.active_thumb.set(Some(new_active_idx));
            if new_active_idx != active_idx && dioxus::core::Runtime::try_current().is_some() {
                crate::foundation::browser::focus_element_by_id(
                    &self.resolved_thumb_id(new_active_idx),
                );
            }
            self.emit_change(new_vals);
        }
    }

    /// Ends an active drag interaction, restoring body selection and committing changed values.
    pub fn end_drag(&mut self) {
        if !*self.is_dragging.peek() {
            return;
        }
        restore_body_user_select();
        self.is_dragging.set(false);

        let start_vals = self.drag_start_values.take();
        let current_vals = self.values.peek().clone();

        if start_vals.as_ref() != Some(&current_vals) {
            self.emit_commit(current_vals);
        }

        self.active_thumb.set(None);
    }

    /// Dispatches keyboard step adjustments for the specified thumb.
    pub fn handle_keydown(&mut self, thumb_idx: usize, key: &str, shift_key: bool) {
        if self.def.disabled {
            return;
        }

        let values = self.values.peek().clone();
        if thumb_idx >= values.len() {
            return;
        }

        let current_val = values[thumb_idx];
        let target_val = if key == "Home" {
            self.def.min
        } else if key == "End" {
            self.def.max
        } else if let Some((direction, multiplier)) = resolve_keyboard_step_direction(
            key,
            self.def.orientation,
            self.def.direction,
            self.def.inverted,
        ) {
            let mult = if shift_key { 10 } else { multiplier };
            if let Some(custom_steps) = self.def.steps.as_deref() {
                get_next_custom_step_value(current_val, custom_steps, direction, mult)
            } else {
                get_next_step_value(
                    current_val,
                    self.def.min,
                    self.def.max,
                    self.def.step,
                    direction,
                    mult,
                )
            }
        } else {
            return;
        };

        let (new_vals, new_active_idx) = update_thumb_value(
            &values,
            thumb_idx,
            target_val,
            self.def.min,
            self.def.max,
            self.def.step,
            self.def.steps.as_deref(),
            self.def.min_steps_between_thumbs,
            self.def.order_policy,
        );

        if self.propose_values(new_vals.clone()) {
            self.active_thumb.set(Some(new_active_idx));
            if new_active_idx != thumb_idx && dioxus::core::Runtime::try_current().is_some() {
                crate::foundation::browser::focus_element_by_id(
                    &self.resolved_thumb_id(new_active_idx),
                );
            }
            self.emit_change(new_vals.clone());
            self.emit_commit(new_vals);
        }
    }
}

/// Context struct holding [`SliderRuntime`] for child components.
#[derive(Clone)]
pub struct SliderContext(pub SliderRuntime);

/// Hook accessing [`SliderContext`] within slider subtree.
pub fn use_slider_context() -> SliderContext {
    use_context::<SliderContext>()
}

/// Initializes and binds the reactive slider runtime hook.
pub fn use_slider_runtime(
    mut def: SliderDefinition,
    rels: SliderRelationships,
    value: Option<SliderValue>,
    default_value: Option<SliderValue>,
    dir: Option<Direction>,
    on_value_change: Option<EventHandler<SliderValue>>,
    on_value_commit: Option<EventHandler<SliderValue>>,
) -> SliderRuntime {
    let resolved_direction = use_direction_context(dir);
    def = def.with_direction(resolved_direction).normalize();

    let is_single = if let Some(ref v) = value {
        v.is_single()
    } else if let Some(ref dv) = default_value {
        dv.is_single()
    } else {
        true
    };

    let min_fallback = def.min;
    let default_values = default_value
        .as_ref()
        .map(SliderValue::to_vec)
        .unwrap_or_else(|| vec![min_fallback]);

    let values_ctl = use_controllable_signal_by(
        value.as_ref().map(SliderValue::to_vec),
        default_values,
        None,
        move |vals| sanitize_slider_values(vals, min_fallback),
        f64_vec_bits_eq,
    );
    let initial_len = values_ctl.peek().len().max(1);

    let mut on_change_slot = use_hook(|| CopyValue::new(on_value_change));
    on_change_slot.set(on_value_change);
    let mut on_commit_slot = use_hook(|| CopyValue::new(on_value_commit));
    on_commit_slot.set(on_value_commit);

    let active_thumb = use_signal(|| None);
    let is_dragging = use_signal(|| false);
    let drag_start_values = use_signal(|| None);
    let focused_thumb = use_signal(|| None);
    let track_size = use_signal(|| None);
    let thumb_sizes = use_signal(|| vec![None; initial_len]);
    let track_id = use_signal(|| rels.track_id().to_string());
    let thumb_ids = use_signal(|| vec![None; initial_len]);
    let thumb_drag_claimed = use_signal(|| false);

    use_drop({
        let is_dragging = is_dragging;
        move || {
            if is_dragging.try_peek().is_ok_and(|d| *d) {
                restore_body_user_select();
            }
        }
    });

    SliderRuntime {
        def,
        rels,
        values: values_ctl.signal(),
        values_ctl: Some(values_ctl),
        is_single,
        active_thumb,
        is_dragging,
        drag_start_values,
        focused_thumb,
        on_value_change: on_change_slot,
        on_value_commit: on_commit_slot,
        track_size,
        thumb_sizes,
        track_id,
        thumb_ids,
        thumb_drag_claimed,
    }
}

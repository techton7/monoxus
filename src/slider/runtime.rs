use dioxus::prelude::*;

use crate::foundation::{
    browser::{restore_body_user_select, set_body_user_select_none},
    overlay::Rect2D,
    shared::use_direction_context,
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
    pub is_single: bool,
    pub active_thumb: Signal<Option<usize>>,
    pub is_dragging: Signal<bool>,
    pub drag_start_values: Signal<Option<Vec<f64>>>,
    pub focused_thumb: Signal<Option<usize>>,
    pub on_value_change: Option<EventHandler<SliderValue>>,
    pub on_value_commit: Option<EventHandler<SliderValue>>,
    pub track_size: Signal<Option<f64>>,
    pub thumb_sizes: Signal<Vec<Option<f64>>>,
    pub track_id: Signal<String>,
    pub thumb_ids: Signal<Vec<Option<String>>>,
    pub thumb_drag_claimed: Signal<bool>,
}

impl SliderRuntime {
    /// Returns the active track element id registered or resolved from relationships.
    pub fn resolved_track_id(&self) -> String {
        self.track_id.peek().clone()
    }

    /// Registers the DOM element id for the slider track.
    pub fn register_track_id(&mut self, id: String) {
        if *self.track_id.peek() != id {
            self.track_id.set(id);
        }
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
        let values = self.current_values();
        let raw_pct = value_to_percent(value, self.def.min, self.def.max);

        let (range_min, range_max) = if values.len() <= 1 {
            let val = values.first().copied().unwrap_or(self.def.min);
            (self.def.min, val)
        } else {
            let min_val = values.iter().copied().fold(f64::INFINITY, |a, b| a.min(b));
            let max_val = values
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, |a, b| a.max(b));
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
        let values = self.current_values();
        let raw_pct = value_to_percent(value, self.def.min, self.def.max);

        let (range_min, range_max) = if values.len() <= 1 {
            let val = values.first().copied().unwrap_or(self.def.min);
            (self.def.min, val)
        } else {
            let min_val = values.iter().copied().fold(f64::INFINITY, |a, b| a.min(b));
            let max_val = values
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, |a, b| a.max(b));
            (min_val, max_val)
        };

        let is_bounded = value >= range_min && value <= range_max;
        let is_selected = values.iter().any(|&v| (v - value).abs() < 1e-9);

        let pos = position.unwrap_or(match self.def.orientation {
            Orientation::Vertical => SliderLabelPosition::Right,
            _ => SliderLabelPosition::Bottom,
        });

        let t_size = *self.track_size.read();
        let th_size = self.thumb_sizes.read().first().copied().flatten();
        let effective_pct = get_contained_percent(
            raw_pct,
            self.def.thumb_alignment,
            self.def.track_padding,
            t_size,
            th_size,
        );

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

        let percent = fraction.clamp(0.0, 1.0) * 100.0;
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

        if new_vals != current_vals {
            self.values.set(new_vals.clone());
            self.active_thumb.set(Some(new_active_idx));
            if new_active_idx != active_idx && dioxus::core::Runtime::try_current().is_some() {
                crate::foundation::browser::focus_element_by_id(
                    &self.resolved_thumb_id(new_active_idx),
                );
            }
            if let Some(ref handler) = self.on_value_change {
                let exported = if self.is_single {
                    SliderValue::Single(new_vals[0])
                } else {
                    SliderValue::Multiple(new_vals)
                };
                handler.call(exported);
            }
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
            if let Some(ref handler) = self.on_value_commit {
                let exported = if self.is_single {
                    SliderValue::Single(current_vals[0])
                } else {
                    SliderValue::Multiple(current_vals)
                };
                handler.call(exported);
            }
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

        if new_vals != values {
            self.values.set(new_vals.clone());
            self.active_thumb.set(Some(new_active_idx));
            if new_active_idx != thumb_idx && dioxus::core::Runtime::try_current().is_some() {
                crate::foundation::browser::focus_element_by_id(
                    &self.resolved_thumb_id(new_active_idx),
                );
            }
            let exported = if self.is_single {
                SliderValue::Single(new_vals[0])
            } else {
                SliderValue::Multiple(new_vals)
            };
            if let Some(ref handler) = self.on_value_change {
                handler.call(exported.clone());
            }
            if let Some(ref handler) = self.on_value_commit {
                handler.call(exported);
            }
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

    let initial_values = if let Some(ref v) = value {
        v.to_vec()
    } else if let Some(ref dv) = default_value {
        dv.to_vec()
    } else {
        vec![def.min]
    };

    let mut values_signal = use_signal(|| initial_values.clone());
    let active_thumb = use_signal(|| None);
    let is_dragging = use_signal(|| false);
    let drag_start_values = use_signal(|| None);
    let focused_thumb = use_signal(|| None);
    let track_size = use_signal(|| None);
    let thumb_sizes = use_signal(|| vec![None; initial_values.len()]);
    let track_id = use_signal(|| rels.track_id().to_string());
    let thumb_ids = use_signal(|| vec![None; initial_values.len()]);
    let thumb_drag_claimed = use_signal(|| false);

    use_effect(use_reactive((&value,), move |(prop_val,)| {
        if let Some(v) = prop_val {
            let vec = v.to_vec();
            if *values_signal.peek() != vec {
                values_signal.set(vec);
            }
        }
    }));

    use_drop({
        let is_dragging = is_dragging;
        move || {
            if *is_dragging.peek() {
                restore_body_user_select();
            }
        }
    });

    SliderRuntime {
        def,
        rels,
        values: values_signal,
        is_single,
        active_thumb,
        is_dragging,
        drag_start_values,
        focused_thumb,
        on_value_change,
        on_value_commit,
        track_size,
        thumb_sizes,
        track_id,
        thumb_ids,
        thumb_drag_claimed,
    }
}

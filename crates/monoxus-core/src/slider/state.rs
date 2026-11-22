use crate::foundation::shared::ScopeHandle;

use super::types::{
    Direction, Orientation, SliderStepSpec, SliderThumbAlignment, SliderThumbOrderPolicy,
};

/// Computes the number of decimal fractional places in a numeric value,
/// correctly parsing scientific notation (e.g. `1e-5`, `1.23e-4`, `1e-7`)
/// to prevent floating-point representation drift.
pub fn get_decimal_count(val: f64) -> usize {
    if !val.is_finite() {
        return 0;
    }
    let s = val.to_string();
    if let Some(pos) = s.find(|c| c == 'e' || c == 'E') {
        let (base, exp_str) = s.split_at(pos);
        let exp: i32 = exp_str[1..].parse().unwrap_or(0);
        let base_decimals = if let Some(dot_pos) = base.find('.') {
            base.len() - dot_pos - 1
        } else {
            0
        };
        if exp < 0 {
            base_decimals + (-exp as usize)
        } else {
            base_decimals.saturating_sub(exp as usize)
        }
    } else if let Some(dot_pos) = s.find('.') {
        s.len() - dot_pos - 1
    } else {
        0
    }
}

/// Rounds a floating point value to the specified decimal precision.
/// Normalizes `-0.0` to `0.0`.
pub fn round_to_precision(val: f64, decimal_count: usize) -> f64 {
    if !val.is_finite() {
        return val;
    }
    if decimal_count == 0 {
        let rounded = val.round();
        if rounded == 0.0 { 0.0 } else { rounded }
    } else {
        let factor = 10f64.powi(decimal_count as i32);
        let rounded = (val * factor).round() / factor;
        if rounded == 0.0 { 0.0 } else { rounded }
    }
}

/// Validates and normalizes `min`, `max`, and `step` values against non-finite or invalid bounds.
pub fn validate_and_normalize_bounds(min: f64, max: f64, step: f64) -> (f64, f64, f64) {
    let valid_min = if min.is_finite() { min } else { 0.0 };
    let valid_max = if max.is_finite() && max > valid_min {
        max
    } else {
        valid_min + 100.0
    };
    let valid_step = if step.is_finite() && step > 0.0 {
        step
    } else {
        1.0
    };
    (valid_min, valid_max, valid_step)
}

/// Filters, sorts, and deduplicates custom discrete step points within `[min, max]`.
pub fn normalize_custom_steps(steps: &[f64], min: f64, max: f64) -> Vec<f64> {
    let mut filtered: Vec<f64> = steps
        .iter()
        .copied()
        .filter(|s| s.is_finite() && *s >= min && *s <= max)
        .collect();
    filtered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    filtered.dedup();
    filtered
}

/// Snaps a value to the nearest discrete step point in `steps`.
pub fn snap_to_steps(value: f64, steps: &[f64]) -> f64 {
    if steps.is_empty() {
        return value;
    }
    let mut closest = steps[0];
    let mut min_diff = (value - closest).abs();
    for &s in &steps[1..] {
        let diff = (value - s).abs();
        if diff < min_diff {
            min_diff = diff;
            closest = s;
        }
    }
    closest
}

/// Implements off-grid step recovery and keyboard stepping math matching Radix UI.
///
/// When the current value is aligned with the grid, stepping increments/decrements by `step * multiplier`.
/// When the current value is off-grid, stepping snaps to the nearest aligned boundary in the specified
/// direction without skipping a step.
pub fn get_next_step_value(
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    direction: i32,
    multiplier: usize,
) -> f64 {
    if max <= min || step <= 0.0 {
        return min;
    }
    let decimal_count = get_decimal_count(step).max(get_decimal_count(min));
    let step_difference = value - min;
    let nearest_steps = (step_difference / step).round();
    let tolerance = (step.abs() * 1e-6).max(1e-12);
    let is_aligned = (value - (nearest_steps * step + min)).abs() <= tolerance;

    let next_steps = if is_aligned {
        nearest_steps + (multiplier as f64) * (direction as f64)
    } else if direction > 0 {
        (step_difference / step).ceil()
            + ((multiplier.saturating_sub(1)) as f64) * (direction as f64)
    } else {
        (step_difference / step).floor()
            + ((multiplier.saturating_sub(1)) as f64) * (direction as f64)
    };

    let next_value = round_to_precision(next_steps * step + min, decimal_count);
    next_value.clamp(min, max)
}

/// Steps to the adjacent step value in a discrete custom steps array.
pub fn get_next_custom_step_value(
    value: f64,
    steps: &[f64],
    direction: i32,
    multiplier: usize,
) -> f64 {
    if steps.is_empty() {
        return value;
    }

    let mut current_idx = 0;
    let mut found_exact = false;
    for (i, &s) in steps.iter().enumerate() {
        if (s - value).abs() < 1e-9 {
            current_idx = i;
            found_exact = true;
            break;
        }
    }

    if !found_exact {
        if direction > 0 {
            if let Some(pos) = steps.iter().position(|&s| s > value) {
                current_idx = pos;
            } else {
                return *steps.last().unwrap();
            }
        } else if let Some(pos) = steps.iter().rposition(|&s| s < value) {
            current_idx = pos;
        } else {
            return steps[0];
        }
        let extra = multiplier.saturating_sub(1);
        if direction > 0 {
            current_idx = (current_idx + extra).min(steps.len() - 1);
        } else {
            current_idx = current_idx.saturating_sub(extra);
        }
    } else if direction > 0 {
        current_idx = (current_idx + multiplier).min(steps.len() - 1);
    } else {
        current_idx = current_idx.saturating_sub(multiplier);
    }

    steps[current_idx]
}

/// Converts a domain value within `[min, max]` to a percentage in `[0.0, 100.0]`.
pub fn value_to_percent(value: f64, min: f64, max: f64) -> f64 {
    if max <= min {
        return 0.0;
    }
    let clamped = value.clamp(min, max);
    ((clamped - min) / (max - min)) * 100.0
}

/// Converts a percentage in `[0.0, 100.0]` to a domain value within `[min, max]`.
pub fn percent_to_value(percent: f64, min: f64, max: f64) -> f64 {
    if max <= min {
        return min;
    }
    let clamped_pct = percent.clamp(0.0, 100.0);
    min + (clamped_pct / 100.0) * (max - min)
}

/// Computes the effective percentage offset for a thumb under the given alignment policy.
///
/// In `Exact` mode, returns the raw percentage.
/// In `Contain` mode, compresses the scale into `[padding, 100.0 - padding]` so the thumb edges
/// remain fully within the track bounds.
pub fn get_contained_percent(
    percent: f64,
    alignment: SliderThumbAlignment,
    track_padding: Option<f64>,
    track_size: Option<f64>,
    thumb_size: Option<f64>,
) -> f64 {
    match alignment {
        SliderThumbAlignment::Exact => percent,
        SliderThumbAlignment::Contain => {
            let padding = if let Some(p) = track_padding {
                p
            } else if let (Some(t_size), Some(th_size)) = (track_size, thumb_size) {
                if t_size > 0.0 && th_size > 0.0 {
                    (th_size / 2.0 / t_size) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            };
            if padding <= 0.0 || padding >= 50.0 {
                percent
            } else {
                padding + (percent / 100.0) * (100.0 - 2.0 * padding)
            }
        }
    }
}

/// Updates a thumb's value, enforcing either `Preserve` (blocking crossover and respecting
/// `min_steps_between_thumbs`) or `Swap` (reordering thumbs and returning the new active index).
pub fn update_thumb_value(
    values: &[f64],
    thumb_index: usize,
    mut new_val: f64,
    min: f64,
    max: f64,
    step: f64,
    steps: Option<&[f64]>,
    min_steps_between_thumbs: usize,
    policy: SliderThumbOrderPolicy,
) -> (Vec<f64>, usize) {
    if values.is_empty() || thumb_index >= values.len() {
        return (values.to_vec(), thumb_index);
    }

    new_val = if let Some(custom_steps) = steps {
        snap_to_steps(new_val, custom_steps)
    } else {
        let decimal_count = get_decimal_count(step).max(get_decimal_count(min));
        let steps_count = ((new_val - min) / step).round();
        round_to_precision(steps_count * step + min, decimal_count)
    };

    let min_distance = min_steps_between_thumbs as f64 * step;
    let mut next_values = values.to_vec();

    match policy {
        SliderThumbOrderPolicy::Preserve => {
            let lower = if thumb_index > 0 {
                next_values[thumb_index - 1] + min_distance
            } else {
                min
            };
            let upper = if thumb_index + 1 < next_values.len() {
                next_values[thumb_index + 1] - min_distance
            } else {
                max
            };

            let effective_lower = lower.min(upper);
            let effective_upper = upper.max(lower);
            let clamped = new_val.clamp(effective_lower, effective_upper);
            next_values[thumb_index] = clamped;
            (next_values, thumb_index)
        }
        SliderThumbOrderPolicy::Swap => {
            new_val = new_val.clamp(min, max);
            next_values[thumb_index] = new_val;

            let mut indexed: Vec<(usize, f64)> = next_values.into_iter().enumerate().collect();
            indexed.sort_by(|(idx_a, val_a), (idx_b, val_b)| {
                val_a
                    .partial_cmp(val_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| idx_a.cmp(idx_b))
            });

            let new_active_index = indexed
                .iter()
                .position(|(orig_idx, _)| *orig_idx == thumb_index)
                .unwrap_or(thumb_index);

            let sorted_vals: Vec<f64> = indexed.into_iter().map(|(_, val)| val).collect();
            (sorted_vals, new_active_index)
        }
    }
}

/// Resolves the step direction (+1 or -1) and multiplier (1 or 10) from keyboard events.
pub fn resolve_keyboard_step_direction(
    key: &str,
    orientation: Orientation,
    direction: Direction,
    inverted: bool,
) -> Option<(i32, usize)> {
    match key {
        "ArrowRight" => {
            if orientation == Orientation::Vertical {
                Some((1, 1))
            } else {
                let is_sliding_from_left = (direction == Direction::Ltr && !inverted)
                    || (direction == Direction::Rtl && inverted);
                if is_sliding_from_left {
                    Some((1, 1))
                } else {
                    Some((-1, 1))
                }
            }
        }
        "ArrowLeft" => {
            if orientation == Orientation::Vertical {
                Some((-1, 1))
            } else {
                let is_sliding_from_left = (direction == Direction::Ltr && !inverted)
                    || (direction == Direction::Rtl && inverted);
                if is_sliding_from_left {
                    Some((-1, 1))
                } else {
                    Some((1, 1))
                }
            }
        }
        "ArrowUp" => {
            if orientation == Orientation::Vertical {
                let is_sliding_from_bottom = !inverted;
                if is_sliding_from_bottom {
                    Some((1, 1))
                } else {
                    Some((-1, 1))
                }
            } else {
                Some((1, 1))
            }
        }
        "ArrowDown" => {
            if orientation == Orientation::Vertical {
                let is_sliding_from_bottom = !inverted;
                if is_sliding_from_bottom {
                    Some((-1, 1))
                } else {
                    Some((1, 1))
                }
            } else {
                Some((-1, 1))
            }
        }
        "PageUp" => {
            if orientation == Orientation::Vertical {
                let is_sliding_from_bottom = !inverted;
                if is_sliding_from_bottom {
                    Some((1, 10))
                } else {
                    Some((-1, 10))
                }
            } else {
                let is_sliding_from_left = (direction == Direction::Ltr && !inverted)
                    || (direction == Direction::Rtl && inverted);
                if is_sliding_from_left {
                    Some((1, 10))
                } else {
                    Some((-1, 10))
                }
            }
        }
        "PageDown" => {
            if orientation == Orientation::Vertical {
                let is_sliding_from_bottom = !inverted;
                if is_sliding_from_bottom {
                    Some((-1, 10))
                } else {
                    Some((1, 10))
                }
            } else {
                let is_sliding_from_left = (direction == Direction::Ltr && !inverted)
                    || (direction == Direction::Rtl && inverted);
                if is_sliding_from_left {
                    Some((-1, 10))
                } else {
                    Some((1, 10))
                }
            }
        }
        _ => None,
    }
}

/// Immutable definition struct representing pure configuration of a Slider widget.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderDefinition {
    pub scope: ScopeHandle,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub steps: Option<Vec<f64>>,
    pub min_steps_between_thumbs: usize,
    pub order_policy: SliderThumbOrderPolicy,
    pub thumb_alignment: SliderThumbAlignment,
    pub track_padding: Option<f64>,
    pub orientation: Orientation,
    pub direction: Direction,
    pub inverted: bool,
    pub disabled: bool,
}

impl SliderDefinition {
    /// Creates a new SliderDefinition with sensible defaults.
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            scope,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            steps: None,
            min_steps_between_thumbs: 0,
            order_policy: SliderThumbOrderPolicy::Swap,
            thumb_alignment: SliderThumbAlignment::Contain,
            track_padding: None,
            orientation: Orientation::Horizontal,
            direction: Direction::Ltr,
            inverted: false,
            disabled: false,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    pub fn with_step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn with_steps(mut self, steps: Option<Vec<f64>>) -> Self {
        self.steps = steps;
        self
    }

    pub fn with_step_spec(mut self, spec: SliderStepSpec) -> Self {
        self.step = spec.step;
        self.steps = spec.steps;
        self
    }

    pub fn with_min_steps_between_thumbs(mut self, min_steps: usize) -> Self {
        self.min_steps_between_thumbs = min_steps;
        self
    }

    pub fn with_order_policy(mut self, policy: SliderThumbOrderPolicy) -> Self {
        self.order_policy = policy;
        self
    }

    pub fn with_thumb_alignment(mut self, alignment: SliderThumbAlignment) -> Self {
        self.thumb_alignment = alignment;
        self
    }

    pub fn with_track_padding(mut self, padding: Option<f64>) -> Self {
        self.track_padding = padding;
        self
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Normalizes bounds and custom steps within this definition.
    pub fn normalize(mut self) -> Self {
        let (min, max, step) = validate_and_normalize_bounds(self.min, self.max, self.step);
        self.min = min;
        self.max = max;
        self.step = step;
        if let Some(steps) = self.steps.as_ref() {
            self.steps = Some(normalize_custom_steps(steps, min, max));
        }
        self
    }
}

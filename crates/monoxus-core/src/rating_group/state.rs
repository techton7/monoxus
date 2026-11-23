use crate::foundation::shared::ScopeHandle;

use super::types::{
    Direction, Orientation, RatingGroupItemData, RatingGroupItemState, RatingGroupKey,
};

/// Normalizes `-0.0` to `0.0` and keeps finite values clean for display and ARIA output.
#[inline]
pub fn normalize_rating_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

/// Formats a rating numeric value consistently (`-0.0` -> `"0"`, `2.5` -> `"2.5"`, `3.0` -> `"3"`).
pub fn format_rating_value(value: f64) -> String {
    normalize_rating_zero(value).to_string()
}

/// Formats the default `aria-valuetext` string (`"{value} out of {max}"`).
pub fn format_default_aria_valuetext(value: f64, max: usize) -> String {
    format!("{} out of {max}", format_rating_value(value))
}

/// Clamps `value` safely into `[min, max]`, guarding against non-finite inputs and `min > max`.
pub fn clamp_rating_value(value: f64, min: f64, max: usize) -> f64 {
    let valid_min = if min.is_finite() {
        normalize_rating_zero(min)
    } else {
        0.0
    };
    let upper = (max as f64).max(valid_min);
    if !value.is_finite() {
        return valid_min;
    }
    normalize_rating_zero(value.clamp(valid_min, upper))
}

fn round_step_precision(value: f64, min: f64) -> f64 {
    let min_decimals = if min.is_finite() {
        let s = min.to_string();
        s.find('.').map_or(0, |dot| s.len() - dot - 1)
    } else {
        0
    };
    let decimals = min_decimals.max(1).min(6);
    let factor = 10f64.powi(decimals as i32);
    normalize_rating_zero((value * factor).round() / factor)
}

/// Derives the visual fill state for item `item_index` (0-based) given `value` and `allow_half`.
pub fn derive_item_state(
    value: f64,
    item_index: usize,
    allow_half: bool,
) -> RatingGroupItemState {
    let item_value = (item_index + 1) as f64;
    let half_value = item_value - 0.5;
    if value >= item_value {
        RatingGroupItemState::Active
    } else if allow_half && value >= half_value {
        RatingGroupItemState::Partial
    } else {
        RatingGroupItemState::Inactive
    }
}

/// Builds the full item snapshot list for indices `0..max`.
pub fn derive_items(value: f64, max: usize, allow_half: bool) -> Vec<RatingGroupItemData> {
    (0..max)
        .map(|index| RatingGroupItemData {
            index,
            value: (index + 1) as f64,
            state: derive_item_state(value, index, allow_half),
        })
        .collect()
}

/// Computes the rating value from pointer client coordinates against an item's bounding rect.
///
/// Per Decision D-4, RTL `1.0 - position` inversion applies only on [`Orientation::Horizontal`];
/// [`Orientation::Vertical`] remains top-to-bottom regardless of text direction.
#[allow(clippy::too_many_arguments)]
pub fn calculate_rating_from_pointer(
    item_index: usize,
    allow_half: bool,
    orientation: Orientation,
    direction: Direction,
    client_x: f64,
    client_y: f64,
    rect_left: f64,
    rect_top: f64,
    rect_width: f64,
    rect_height: f64,
) -> f64 {
    let rating_value = (item_index + 1) as f64;
    if !allow_half {
        return rating_value;
    }

    let normalized_position = match orientation {
        Orientation::Vertical => {
            let h = rect_height.max(1.0);
            (client_y - rect_top) / h
        }
        Orientation::Horizontal | Orientation::Both => {
            let w = rect_width.max(1.0);
            let pos = (client_x - rect_left) / w;
            if direction == Direction::Rtl {
                1.0 - pos
            } else {
                pos
            }
        }
    };

    if normalized_position < 0.5 {
        rating_value - 0.5
    } else {
        rating_value
    }
}

/// Computes the next committed value when an item is clicked, supporting first-item click-to-clear.
pub fn resolve_item_click_value(
    item_index: usize,
    current_value: f64,
    min: f64,
    max: usize,
    calculated_value: f64,
) -> f64 {
    let valid_min = if min.is_finite() {
        normalize_rating_zero(min)
    } else {
        0.0
    };
    if item_index == 0
        && valid_min == 0.0
        && current_value > 0.0
        && calculated_value == current_value
    {
        0.0
    } else {
        clamp_rating_value(calculated_value, valid_min, max)
    }
}

/// Decimal typing buffer state machine (e.g. `'2'` -> `'.'` -> `'5'` -> `2.5`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RatingGroupKeySequence {
    #[default]
    Idle,
    Digit(u8),
    DigitDot(u8),
}

/// Pure outcome produced by reducing one keyboard intent against [`RatingGroupDefinition`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RatingGroupKeyOutcome {
    pub next_value: Option<f64>,
    pub clear_hover: bool,
    pub prevent_default: bool,
    pub next_sequence: RatingGroupKeySequence,
    pub start_decimal_timer: bool,
    pub clear_decimal_timer: bool,
}

/// Immutable configuration definition for a `RatingGroup`.
#[derive(Clone, Debug, PartialEq)]
pub struct RatingGroupDefinition {
    pub scope: ScopeHandle,
    pub min: f64,
    pub max: usize,
    pub allow_half: bool,
    pub hover_preview: bool,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub orientation: Orientation,
    pub direction: Direction,
}

impl RatingGroupDefinition {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            scope,
            min: 0.0,
            max: 5,
            allow_half: false,
            hover_preview: true,
            disabled: false,
            read_only: false,
            required: false,
            orientation: Orientation::Horizontal,
            direction: Direction::Ltr,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min = if min.is_finite() {
            normalize_rating_zero(min)
        } else {
            0.0
        };
        self
    }

    pub fn with_max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }

    pub fn with_allow_half(mut self, allow_half: bool) -> Self {
        self.allow_half = allow_half;
        self
    }

    pub fn with_hover_preview(mut self, hover_preview: bool) -> Self {
        self.hover_preview = hover_preview;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
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
}

/// Pure keyboard reducer for `RatingGroupRoot`.
pub fn reduce_rating_group_key(
    def: &RatingGroupDefinition,
    current_value: f64,
    sequence: RatingGroupKeySequence,
    key: RatingGroupKey,
) -> RatingGroupKeyOutcome {
    if def.disabled || def.read_only {
        return RatingGroupKeyOutcome {
            next_value: None,
            clear_hover: false,
            prevent_default: false,
            next_sequence: sequence,
            start_decimal_timer: false,
            clear_decimal_timer: false,
        };
    }

    let step = if def.allow_half { 0.5 } else { 1.0 };
    let max_f = def.max as f64;

    let step_outcome = |target: f64| RatingGroupKeyOutcome {
        next_value: Some(clamp_rating_value(target, def.min, def.max)),
        clear_hover: true,
        prevent_default: true,
        next_sequence: RatingGroupKeySequence::Idle,
        start_decimal_timer: false,
        clear_decimal_timer: true,
    };

    match key {
        RatingGroupKey::ArrowUp => {
            step_outcome(round_step_precision(current_value + step, def.min))
        }
        RatingGroupKey::ArrowDown => {
            step_outcome(round_step_precision(current_value - step, def.min))
        }
        RatingGroupKey::ArrowRight => {
            let delta = if def.direction == Direction::Rtl {
                -step
            } else {
                step
            };
            step_outcome(round_step_precision(current_value + delta, def.min))
        }
        RatingGroupKey::ArrowLeft => {
            let delta = if def.direction == Direction::Rtl {
                step
            } else {
                -step
            };
            step_outcome(round_step_precision(current_value + delta, def.min))
        }
        RatingGroupKey::Home => step_outcome(def.min),
        RatingGroupKey::End => step_outcome(max_f),
        RatingGroupKey::PageUp => {
            step_outcome(round_step_precision(current_value + 1.0, def.min))
        }
        RatingGroupKey::PageDown => {
            step_outcome(round_step_precision(current_value - 1.0, def.min))
        }
        RatingGroupKey::DecimalPoint => {
            if def.allow_half {
                let next_sequence = match sequence {
                    RatingGroupKeySequence::Digit(d) => RatingGroupKeySequence::DigitDot(d),
                    _ => RatingGroupKeySequence::Idle,
                };
                RatingGroupKeyOutcome {
                    next_value: None,
                    clear_hover: false,
                    prevent_default: true,
                    next_sequence,
                    start_decimal_timer: false,
                    clear_decimal_timer: false,
                }
            } else {
                RatingGroupKeyOutcome {
                    next_value: None,
                    clear_hover: false,
                    prevent_default: false,
                    next_sequence: RatingGroupKeySequence::Idle,
                    start_decimal_timer: false,
                    clear_decimal_timer: true,
                }
            }
        }
        RatingGroupKey::Digit(5)
            if def.allow_half && matches!(sequence, RatingGroupKeySequence::DigitDot(_)) =>
        {
            let RatingGroupKeySequence::DigitDot(base) = sequence else {
                unreachable!();
            };
            let target = (base as f64) + 0.5;
            let next_value = if target >= def.min && target <= max_f {
                Some(clamp_rating_value(target, def.min, def.max))
            } else {
                None
            };
            RatingGroupKeyOutcome {
                next_value,
                clear_hover: false,
                prevent_default: true,
                next_sequence: RatingGroupKeySequence::Idle,
                start_decimal_timer: false,
                clear_decimal_timer: true,
            }
        }
        RatingGroupKey::Digit(d) => {
            let num = d as f64;
            if num >= def.min && num <= max_f {
                let (next_sequence, start_decimal_timer, clear_decimal_timer) = if def.allow_half {
                    (RatingGroupKeySequence::Digit(d), true, false)
                } else {
                    (RatingGroupKeySequence::Idle, false, true)
                };
                RatingGroupKeyOutcome {
                    next_value: Some(clamp_rating_value(num, def.min, def.max)),
                    clear_hover: false,
                    prevent_default: true,
                    next_sequence,
                    start_decimal_timer,
                    clear_decimal_timer,
                }
            } else {
                RatingGroupKeyOutcome {
                    next_value: None,
                    clear_hover: false,
                    prevent_default: true,
                    next_sequence: RatingGroupKeySequence::Idle,
                    start_decimal_timer: false,
                    clear_decimal_timer: true,
                }
            }
        }
        RatingGroupKey::Other => RatingGroupKeyOutcome {
            next_value: None,
            clear_hover: false,
            prevent_default: false,
            next_sequence: RatingGroupKeySequence::Idle,
            start_decimal_timer: false,
            clear_decimal_timer: true,
        },
    }
}

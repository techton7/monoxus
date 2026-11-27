use crate::foundation::shared::{data_disabled_attr, data_invalid_attr, data_required_attr};

use super::types::{Direction, Orientation, SliderLabelPosition};

/// Pure attribute publisher for `SliderRoot`.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderRootAttributes {
    pub id: String,
    pub orientation: Orientation,
    pub direction: Direction,
    pub disabled: bool,
    pub required: bool,
    pub invalid: bool,
    pub aria_disabled: Option<&'static str>,
    pub aria_describedby: Option<String>,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub style: &'static str,
}

impl SliderRootAttributes {
    pub fn new(id: String, orientation: Orientation, direction: Direction, disabled: bool) -> Self {
        let data_orientation = match orientation {
            Orientation::Vertical => "vertical",
            _ => "horizontal",
        };
        let aria_disabled = if disabled { Some("true") } else { None };
        let data_disabled = data_disabled_attr(disabled);
        let style = "position: relative; touch-action: none;";

        Self {
            id,
            orientation,
            direction,
            disabled,
            required: false,
            invalid: false,
            aria_disabled,
            aria_describedby: None,
            data_orientation,
            data_disabled,
            data_required: None,
            data_invalid: None,
            style,
        }
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self.data_required = data_required_attr(required);
        self
    }

    pub fn with_invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self.data_invalid = data_invalid_attr(invalid);
        self
    }

    pub fn with_aria_describedby(mut self, aria_describedby: Option<String>) -> Self {
        self.aria_describedby = aria_describedby;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_required(&self) -> Option<&'static str> {
        self.data_required
    }

    pub fn data_invalid(&self) -> Option<&'static str> {
        self.data_invalid
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn aria_describedby(&self) -> Option<&str> {
        self.aria_describedby.as_deref()
    }

    pub fn style(&self) -> &'static str {
        self.style
    }
}

/// Pure attribute publisher for `SliderTrack`.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderTrackAttributes {
    pub id: String,
    pub orientation: Orientation,
    pub disabled: bool,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub style: &'static str,
}

impl SliderTrackAttributes {
    pub fn new(id: String, orientation: Orientation, disabled: bool) -> Self {
        let data_orientation = match orientation {
            Orientation::Vertical => "vertical",
            _ => "horizontal",
        };
        let data_disabled = data_disabled_attr(disabled);
        let style = "position: relative;";

        Self {
            id,
            orientation,
            disabled,
            data_orientation,
            data_disabled,
            style,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn style(&self) -> &'static str {
        self.style
    }
}

/// Pure attribute publisher for `SliderRange`.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderRangeAttributes {
    pub id: String,
    pub orientation: Orientation,
    pub disabled: bool,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub style: String,
}

impl SliderRangeAttributes {
    pub fn new(
        id: String,
        orientation: Orientation,
        direction: Direction,
        inverted: bool,
        disabled: bool,
        start_percent: f64,
        end_percent: f64,
    ) -> Self {
        let data_orientation = match orientation {
            Orientation::Vertical => "vertical",
            _ => "horizontal",
        };
        let data_disabled = data_disabled_attr(disabled);

        let min_pct = start_percent.min(end_percent).clamp(0.0, 100.0);
        let max_pct = start_percent.max(end_percent).clamp(0.0, 100.0);
        let span = (max_pct - min_pct).max(0.0);

        let style = match orientation {
            Orientation::Vertical => {
                let is_sliding_from_bottom = !inverted;
                if is_sliding_from_bottom {
                    format!("position: absolute; bottom: {min_pct}%; height: {span}%;")
                } else {
                    format!("position: absolute; top: {min_pct}%; height: {span}%;")
                }
            }
            _ => {
                let is_sliding_from_left = (direction == Direction::Ltr && !inverted)
                    || (direction == Direction::Rtl && inverted);
                if is_sliding_from_left {
                    format!("position: absolute; left: {min_pct}%; width: {span}%;")
                } else {
                    format!("position: absolute; right: {min_pct}%; width: {span}%;")
                }
            }
        };

        Self {
            id,
            orientation,
            disabled,
            data_orientation,
            data_disabled,
            style,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn style(&self) -> &str {
        &self.style
    }
}

/// Resolves default ARIA label for thumbs when not explicitly supplied.
pub fn resolve_thumb_aria_label(
    user_label: Option<&str>,
    thumb_index: usize,
    thumb_count: usize,
) -> Option<String> {
    if let Some(label) = user_label {
        return Some(label.to_string());
    }
    if thumb_count == 2 {
        if thumb_index == 0 {
            Some("Minimum".to_string())
        } else {
            Some("Maximum".to_string())
        }
    } else if thumb_count > 2 {
        Some(format!("Value {} of {}", thumb_index + 1, thumb_count))
    } else {
        None
    }
}

/// Pure attribute publisher for `SliderThumb`.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderThumbAttributes {
    pub id: String,
    pub role: &'static str,
    pub tabindex: i32,
    pub aria_valuenow: f64,
    pub aria_valuemin: f64,
    pub aria_valuemax: f64,
    pub aria_valuetext: Option<String>,
    pub aria_orientation: &'static str,
    pub aria_disabled: Option<&'static str>,
    pub aria_invalid: Option<&'static str>,
    pub aria_describedby: Option<String>,
    pub aria_label: Option<String>,
    pub aria_labelledby: Option<String>,
    pub required: bool,
    pub invalid: bool,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub data_value: String,
    pub data_active: Option<&'static str>,
    pub style: String,
}

impl SliderThumbAttributes {
    pub fn new(
        id: String,
        value: f64,
        min: f64,
        max: f64,
        orientation: Orientation,
        direction: Direction,
        inverted: bool,
        disabled: bool,
        is_active: bool,
        percent: f64,
        aria_label: Option<String>,
        aria_labelledby: Option<String>,
        aria_valuetext: Option<String>,
    ) -> Self {
        let data_orientation = match orientation {
            Orientation::Vertical => "vertical",
            _ => "horizontal",
        };
        let tabindex = if disabled { -1 } else { 0 };
        let aria_disabled = if disabled { Some("true") } else { None };
        let data_disabled = data_disabled_attr(disabled);
        let data_active = if is_active { Some("") } else { None };
        let data_value = value.to_string();

        let style = match orientation {
            Orientation::Vertical => {
                let is_sliding_from_bottom = !inverted;
                if is_sliding_from_bottom {
                    format!(
                        "position: absolute; bottom: {percent}%; left: 50%; transform: translate(-50%, 50%);"
                    )
                } else {
                    format!(
                        "position: absolute; top: {percent}%; left: 50%; transform: translate(-50%, -50%);"
                    )
                }
            }
            _ => {
                let is_sliding_from_left = (direction == Direction::Ltr && !inverted)
                    || (direction == Direction::Rtl && inverted);
                if is_sliding_from_left {
                    format!(
                        "position: absolute; left: {percent}%; top: 50%; transform: translate(-50%, -50%);"
                    )
                } else {
                    format!(
                        "position: absolute; right: {percent}%; top: 50%; transform: translate(50%, -50%);"
                    )
                }
            }
        };

        Self {
            id,
            role: "slider",
            tabindex,
            aria_valuenow: value,
            aria_valuemin: min,
            aria_valuemax: max,
            aria_valuetext,
            aria_orientation: data_orientation,
            aria_disabled,
            aria_invalid: None,
            aria_describedby: None,
            aria_label,
            aria_labelledby,
            required: false,
            invalid: false,
            data_orientation,
            data_disabled,
            data_required: None,
            data_invalid: None,
            data_value,
            data_active,
            style,
        }
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self.data_required = data_required_attr(required);
        self
    }

    pub fn with_invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self.aria_invalid = invalid.then_some("true");
        self.data_invalid = data_invalid_attr(invalid);
        self
    }

    pub fn with_aria_describedby(mut self, aria_describedby: Option<String>) -> Self {
        self.aria_describedby = aria_describedby;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn aria_invalid(&self) -> Option<&'static str> {
        self.aria_invalid
    }

    pub fn aria_describedby(&self) -> Option<&str> {
        self.aria_describedby.as_deref()
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_required(&self) -> Option<&'static str> {
        self.data_required
    }

    pub fn data_invalid(&self) -> Option<&'static str> {
        self.data_invalid
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn data_active(&self) -> Option<&'static str> {
        self.data_active
    }

    pub fn style(&self) -> &str {
        &self.style
    }
}

/// Pure attribute publisher for `SliderTick`.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderTickAttributes {
    pub id: String,
    pub value: f64,
    pub orientation: Orientation,
    pub disabled: bool,
    pub is_bounded: bool,
    pub is_selected: bool,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_value: String,
    pub data_bounded: Option<&'static str>,
    pub data_selected: Option<&'static str>,
    pub style: String,
}

impl SliderTickAttributes {
    pub fn new(
        id: String,
        value: f64,
        orientation: Orientation,
        direction: Direction,
        inverted: bool,
        disabled: bool,
        is_bounded: bool,
        is_selected: bool,
        percent: f64,
    ) -> Self {
        let data_orientation = match orientation {
            Orientation::Vertical => "vertical",
            _ => "horizontal",
        };
        let data_disabled = data_disabled_attr(disabled);
        let data_bounded = if is_bounded { Some("") } else { None };
        let data_selected = if is_selected { Some("") } else { None };
        let data_value = value.to_string();

        let style = match orientation {
            Orientation::Vertical => {
                let is_sliding_from_bottom = !inverted;
                if is_sliding_from_bottom {
                    format!(
                        "position: absolute; bottom: {percent}%; left: 50%; transform: translateY(50%);"
                    )
                } else {
                    format!(
                        "position: absolute; top: {percent}%; left: 50%; transform: translateY(-50%);"
                    )
                }
            }
            _ => {
                let is_sliding_from_left = (direction == Direction::Ltr && !inverted)
                    || (direction == Direction::Rtl && inverted);
                if is_sliding_from_left {
                    format!(
                        "position: absolute; left: {percent}%; top: 50%; transform: translateX(-50%);"
                    )
                } else {
                    format!(
                        "position: absolute; right: {percent}%; top: 50%; transform: translateX(50%);"
                    )
                }
            }
        };

        Self {
            id,
            value,
            orientation,
            disabled,
            is_bounded,
            is_selected,
            data_orientation,
            data_disabled,
            data_value,
            data_bounded,
            data_selected,
            style,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn data_bounded(&self) -> Option<&'static str> {
        self.data_bounded
    }

    pub fn data_selected(&self) -> Option<&'static str> {
        self.data_selected
    }

    pub fn style(&self) -> &str {
        &self.style
    }
}

/// Pure attribute publisher for `SliderTickLabel`.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderTickLabelAttributes {
    pub id: String,
    pub value: f64,
    pub position: SliderLabelPosition,
    pub orientation: Orientation,
    pub disabled: bool,
    pub is_bounded: bool,
    pub is_selected: bool,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_value: String,
    pub data_bounded: Option<&'static str>,
    pub data_selected: Option<&'static str>,
    pub data_position: &'static str,
    pub style: String,
}

impl SliderTickLabelAttributes {
    pub fn new(
        id: String,
        value: f64,
        position: SliderLabelPosition,
        orientation: Orientation,
        direction: Direction,
        inverted: bool,
        disabled: bool,
        is_bounded: bool,
        is_selected: bool,
        percent: f64,
    ) -> Self {
        let data_orientation = match orientation {
            Orientation::Vertical => "vertical",
            _ => "horizontal",
        };
        let data_position = match position {
            SliderLabelPosition::Top => "top",
            SliderLabelPosition::Bottom => "bottom",
            SliderLabelPosition::Left => "left",
            SliderLabelPosition::Right => "right",
        };
        let data_disabled = data_disabled_attr(disabled);
        let data_bounded = if is_bounded { Some("") } else { None };
        let data_selected = if is_selected { Some("") } else { None };
        let data_value = value.to_string();

        let style = match orientation {
            Orientation::Vertical => {
                let is_sliding_from_bottom = !inverted;
                if is_sliding_from_bottom {
                    format!("position: absolute; bottom: {percent}%; transform: translateY(50%);")
                } else {
                    format!("position: absolute; top: {percent}%; transform: translateY(-50%);")
                }
            }
            _ => {
                let is_sliding_from_left = (direction == Direction::Ltr && !inverted)
                    || (direction == Direction::Rtl && inverted);
                if is_sliding_from_left {
                    format!("position: absolute; left: {percent}%; transform: translateX(-50%);")
                } else {
                    format!("position: absolute; right: {percent}%; transform: translateX(50%);")
                }
            }
        };

        Self {
            id,
            value,
            position,
            orientation,
            disabled,
            is_bounded,
            is_selected,
            data_orientation,
            data_disabled,
            data_value,
            data_bounded,
            data_selected,
            data_position,
            style,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn data_bounded(&self) -> Option<&'static str> {
        self.data_bounded
    }

    pub fn data_selected(&self) -> Option<&'static str> {
        self.data_selected
    }

    pub fn data_position(&self) -> &'static str {
        self.data_position
    }

    pub fn style(&self) -> &str {
        &self.style
    }
}

/// Pure attribute publisher for `SliderThumbLabel`.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderThumbLabelAttributes {
    pub id: String,
    pub value: f64,
    pub position: SliderLabelPosition,
    pub orientation: Orientation,
    pub disabled: bool,
    pub is_active: bool,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_value: String,
    pub data_active: Option<&'static str>,
    pub data_position: &'static str,
    pub style: String,
}

impl SliderThumbLabelAttributes {
    pub fn new(
        id: String,
        value: f64,
        position: SliderLabelPosition,
        orientation: Orientation,
        direction: Direction,
        inverted: bool,
        disabled: bool,
        is_active: bool,
        percent: f64,
    ) -> Self {
        let data_orientation = match orientation {
            Orientation::Vertical => "vertical",
            _ => "horizontal",
        };
        let data_position = match position {
            SliderLabelPosition::Top => "top",
            SliderLabelPosition::Bottom => "bottom",
            SliderLabelPosition::Left => "left",
            SliderLabelPosition::Right => "right",
        };
        let data_disabled = data_disabled_attr(disabled);
        let data_active = if is_active { Some("") } else { None };
        let data_value = value.to_string();

        let style = match orientation {
            Orientation::Vertical => {
                let is_sliding_from_bottom = !inverted;
                if is_sliding_from_bottom {
                    format!("position: absolute; bottom: {percent}%; transform: translateY(50%);")
                } else {
                    format!("position: absolute; top: {percent}%; transform: translateY(-50%);")
                }
            }
            _ => {
                let is_sliding_from_left = (direction == Direction::Ltr && !inverted)
                    || (direction == Direction::Rtl && inverted);
                if is_sliding_from_left {
                    format!("position: absolute; left: {percent}%; transform: translateX(-50%);")
                } else {
                    format!("position: absolute; right: {percent}%; transform: translateX(50%);")
                }
            }
        };

        Self {
            id,
            value,
            position,
            orientation,
            disabled,
            is_active,
            data_orientation,
            data_disabled,
            data_value,
            data_active,
            data_position,
            style,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn data_active(&self) -> Option<&'static str> {
        self.data_active
    }

    pub fn data_position(&self) -> &'static str {
        self.data_position
    }

    pub fn style(&self) -> &str {
        &self.style
    }
}

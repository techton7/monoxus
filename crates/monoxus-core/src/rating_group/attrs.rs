use crate::foundation::{
    compose::{Slottable, project_as_child},
    shared::data_disabled_attr,
};

use super::{
    state::{format_default_aria_valuetext, normalize_rating_zero},
    types::{Direction, Orientation, RatingGroupItemState},
};

fn orientation_str(orientation: Orientation) -> &'static str {
    match orientation {
        Orientation::Vertical => "vertical",
        Orientation::Horizontal | Orientation::Both => "horizontal",
    }
}

/// Pure attribute publisher for `RatingGroupRoot`.
#[derive(Clone, Debug, PartialEq)]
pub struct RatingGroupRootAttributes {
    pub id: String,
    pub role: &'static str,
    pub dir: &'static str,
    pub tabindex: i32,
    pub aria_valuenow: f64,
    pub aria_valuemin: f64,
    pub aria_valuemax: usize,
    pub aria_valuetext: String,
    pub aria_orientation: &'static str,
    pub aria_required: &'static str,
    pub aria_disabled: Option<&'static str>,
    pub aria_label: Option<String>,
    pub aria_labelledby: Option<String>,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_rating_group_root: &'static str,
}

impl RatingGroupRootAttributes {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        value: f64,
        min: f64,
        max: usize,
        orientation: Orientation,
        direction: Direction,
        disabled: bool,
        read_only: bool,
        required: bool,
        aria_label: Option<String>,
        aria_labelledby: Option<String>,
        aria_valuetext: Option<String>,
    ) -> Self {
        let data_orientation = orientation_str(orientation);
        let normalized_value = normalize_rating_zero(value);
        let normalized_min = normalize_rating_zero(min);
        let resolved_aria_valuetext =
            aria_valuetext.unwrap_or_else(|| format_default_aria_valuetext(normalized_value, max));
        let resolved_aria_label = match (aria_label, aria_labelledby.as_ref()) {
            (Some(label), _) => Some(label),
            (None, Some(_)) => None,
            (None, None) => Some(String::from("Rating")),
        };

        Self {
            id,
            role: "slider",
            dir: direction.as_str(),
            tabindex: if disabled { -1 } else { 0 },
            aria_valuenow: normalized_value,
            aria_valuemin: normalized_min,
            aria_valuemax: max,
            aria_valuetext: resolved_aria_valuetext,
            aria_orientation: data_orientation,
            aria_required: if required { "true" } else { "false" },
            aria_disabled: if disabled { Some("true") } else { None },
            aria_label: resolved_aria_label,
            aria_labelledby,
            data_orientation,
            data_disabled: data_disabled_attr(disabled),
            data_readonly: data_disabled_attr(read_only),
            data_rating_group_root: "",
        }
    }
}

/// Pure attribute publisher for `RatingGroupItem`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RatingGroupItemAttributes {
    pub id: String,
    pub role: &'static str,
    pub data_value: usize,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_state: RatingGroupItemState,
    pub data_rating_group_item: &'static str,
}

impl RatingGroupItemAttributes {
    pub fn new(
        id: String,
        index: usize,
        orientation: Orientation,
        disabled: bool,
        read_only: bool,
        state: RatingGroupItemState,
    ) -> Self {
        Self {
            id,
            role: "presentation",
            data_value: index + 1,
            data_orientation: orientation_str(orientation),
            data_disabled: data_disabled_attr(disabled),
            data_readonly: data_disabled_attr(read_only),
            data_state: state,
            data_rating_group_item: "",
        }
    }
}

/// Projects `RatingGroupRoot` attributes onto a slottable child target without adding a wrapper.
pub fn project_rating_group_root_as_child<T, C>(
    attrs: &RatingGroupRootAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, RatingGroupRootAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

/// Projects `RatingGroupItem` attributes onto a slottable child target without adding a wrapper.
pub fn project_rating_group_item_as_child<T, C>(
    attrs: &RatingGroupItemAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, RatingGroupItemAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

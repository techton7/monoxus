use super::types::ProgressState;

fn number_attr(value: f64) -> String {
    value.to_string()
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressRootAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_valuemin: String,
    pub aria_valuemax: String,
    pub aria_valuenow: Option<String>,
    pub aria_valuetext: Option<String>,
    pub data_state: &'static str,
    pub data_value: Option<String>,
    pub data_max: String,
    pub data_min: &'static str,
    pub data_indeterminate: Option<&'static str>,
    pub data_progress_root: &'static str,
}

impl ProgressRootAttributes {
    pub fn new(
        id: String,
        state: ProgressState,
        value: Option<f64>,
        max: f64,
        value_label: Option<String>,
        loaded_state_alias: bool,
    ) -> Self {
        Self {
            id,
            role: "progressbar",
            aria_valuemin: "0".to_string(),
            aria_valuemax: number_attr(max),
            aria_valuenow: value.map(number_attr),
            aria_valuetext: value.and(value_label),
            data_state: state.compatibility_str(loaded_state_alias),
            data_value: value.map(number_attr),
            data_max: number_attr(max),
            data_min: "0",
            data_indeterminate: state.is_indeterminate().then_some(""),
            data_progress_root: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressIndicatorAttributes {
    pub id: String,
    pub data_state: &'static str,
    pub data_value: Option<String>,
    pub data_max: String,
    pub data_min: &'static str,
    pub data_indeterminate: Option<&'static str>,
    pub data_progress_indicator: &'static str,
}

impl ProgressIndicatorAttributes {
    pub fn new(
        id: String,
        state: ProgressState,
        value: Option<f64>,
        max: f64,
        loaded_state_alias: bool,
    ) -> Self {
        Self {
            id,
            data_state: state.compatibility_str(loaded_state_alias),
            data_value: value.map(number_attr),
            data_max: number_attr(max),
            data_min: "0",
            data_indeterminate: state.is_indeterminate().then_some(""),
            data_progress_indicator: "",
        }
    }
}

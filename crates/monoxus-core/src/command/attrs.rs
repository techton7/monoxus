use crate::foundation::{
    compose::{Slottable, project_as_child},
    shared::data_disabled_attr,
};

#[inline]
pub const fn bool_to_empty_attr(flag: bool) -> Option<&'static str> {
    if flag { Some("") } else { None }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandRootAttrs {
    pub id: String,
    pub role: &'static str,
    pub tabindex: &'static str,
    pub aria_label: Option<String>,
    pub data_command_root: &'static str,
}

impl CommandRootAttrs {
    pub fn new(id: impl Into<String>, label: Option<&str>) -> Self {
        let aria_label = label.filter(|s| !s.is_empty()).map(str::to_string);
        Self {
            id: id.into(),
            role: "application",
            tabindex: "-1",
            aria_label,
            data_command_root: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandInputLabelAttrs {
    pub id: String,
    pub r#for: String,
    pub data_command_input_label: &'static str,
}

impl CommandInputLabelAttrs {
    pub fn new(id: impl Into<String>, input_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            r#for: input_id.into(),
            data_command_input_label: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandInputAttrs {
    pub id: String,
    pub r#type: &'static str,
    pub role: &'static str,
    pub aria_autocomplete: &'static str,
    pub aria_expanded: &'static str,
    pub aria_controls: Option<String>,
    pub aria_labelledby: Option<String>,
    pub aria_activedescendant: Option<String>,
    pub autocomplete: &'static str,
    pub autocorrect: &'static str,
    pub spellcheck: &'static str,
    pub data_command_input: &'static str,
}

impl CommandInputAttrs {
    pub fn new(
        id: impl Into<String>,
        viewport_id: Option<String>,
        label_id: Option<String>,
        active_descendant: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            r#type: "text",
            role: "combobox",
            aria_autocomplete: "list",
            aria_expanded: "true",
            aria_controls: viewport_id,
            aria_labelledby: label_id,
            aria_activedescendant: active_descendant,
            autocomplete: "off",
            autocorrect: "off",
            spellcheck: "false",
            data_command_input: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandListAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_label: String,
    pub data_command_list: &'static str,
}

impl CommandListAttrs {
    pub fn new(id: impl Into<String>, aria_label: Option<&str>) -> Self {
        Self {
            id: id.into(),
            role: "listbox",
            aria_label: aria_label.unwrap_or("Suggestions...").to_string(),
            data_command_list: "",
        }
    }

    pub fn format_height_style(height_px: Option<f64>, extra_style: Option<&str>) -> String {
        let mut style = String::new();
        if let Some(h) = height_px {
            style.push_str(&format!(
                "--monoxus-command-list-height: {h}px; --bits-command-list-height: {h}px;"
            ));
        }
        if let Some(extra) = extra_style {
            let trimmed = extra.trim();
            if !trimmed.is_empty() {
                if !style.is_empty() {
                    style.push(' ');
                }
                style.push_str(trimmed);
            }
        }
        style
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandViewportAttrs {
    pub id: String,
    pub data_command_viewport: &'static str,
}

impl CommandViewportAttrs {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            data_command_viewport: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandEmptyAttrs {
    pub id: String,
    pub role: &'static str,
    pub data_command_empty: &'static str,
}

impl CommandEmptyAttrs {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role: "presentation",
            data_command_empty: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandLoadingAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_valuenow: f64,
    pub aria_valuemin: &'static str,
    pub aria_valuemax: &'static str,
    pub aria_label: String,
    pub data_command_loading: &'static str,
}

impl CommandLoadingAttrs {
    pub fn new(id: impl Into<String>, progress: f64, aria_label: Option<&str>) -> Self {
        let clamped = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 100.0)
        };
        Self {
            id: id.into(),
            role: "progressbar",
            aria_valuenow: clamped,
            aria_valuemin: "0",
            aria_valuemax: "100",
            aria_label: aria_label.unwrap_or("Loading...").to_string(),
            data_command_loading: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandGroupAttrs {
    pub id: String,
    pub role: &'static str,
    pub hidden: bool,
    pub data_command_group: &'static str,
    pub data_value: String,
}

impl CommandGroupAttrs {
    pub fn new(id: impl Into<String>, value: impl Into<String>, should_render: bool) -> Self {
        Self {
            id: id.into(),
            role: "presentation",
            hidden: !should_render,
            data_command_group: "",
            data_value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandGroupHeadingAttrs {
    pub id: String,
    pub data_command_group_heading: &'static str,
}

impl CommandGroupHeadingAttrs {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            data_command_group_heading: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandGroupItemsAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_labelledby: Option<String>,
    pub data_command_group_items: &'static str,
}

impl CommandGroupItemsAttrs {
    pub fn new(id: impl Into<String>, heading_id: Option<String>) -> Self {
        Self {
            id: id.into(),
            role: "group",
            aria_labelledby: heading_id,
            data_command_group_items: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandItemAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_disabled: &'static str,
    pub aria_selected: &'static str,
    pub data_command_item: &'static str,
    pub data_value: String,
    pub data_group: Option<String>,
    pub data_disabled: Option<&'static str>,
    pub data_selected: Option<&'static str>,
}

impl CommandItemAttrs {
    pub fn new(
        id: impl Into<String>,
        value: impl Into<String>,
        group_value: Option<String>,
        disabled: bool,
        selected: bool,
    ) -> Self {
        Self {
            id: id.into(),
            role: "option",
            aria_disabled: if disabled { "true" } else { "false" },
            aria_selected: if selected { "true" } else { "false" },
            data_command_item: "",
            data_value: value.into(),
            data_group: group_value,
            data_disabled: data_disabled_attr(disabled),
            data_selected: bool_to_empty_attr(selected),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandSeparatorAttrs {
    pub id: String,
    pub aria_hidden: &'static str,
    pub data_command_separator: &'static str,
}

impl CommandSeparatorAttrs {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            aria_hidden: "true",
            data_command_separator: "",
        }
    }
}

pub fn merge_flex_column_style(extra_style: Option<&str>, is_grid: bool) -> Option<String> {
    let trimmed = extra_style.map(str::trim).filter(|s| !s.is_empty());
    if is_grid {
        return trimmed.map(str::to_string);
    }
    match trimmed {
        Some(extra) => Some(format!("display: flex; flex-direction: column; {extra}")),
        None => Some("display: flex; flex-direction: column;".to_string()),
    }
}

pub fn merge_ranked_item_style(
    should_render: bool,
    visual_order: Option<usize>,
    extra_style: Option<&str>,
) -> Option<String> {
    if !should_render {
        return Some("display: none;".to_string());
    }
    let trimmed = extra_style.map(str::trim).filter(|s| !s.is_empty());
    match (trimmed, visual_order) {
        (Some(extra), Some(rank)) => {
            let sep = if extra.ends_with(';') { " " } else { "; " };
            Some(format!("{extra}{sep}order: {rank};"))
        }
        (None, Some(rank)) => Some(format!("order: {rank};")),
        (Some(extra), None) => Some(extra.to_string()),
        (None, None) => None,
    }
}

pub fn project_command_part_as_child<A, T, C>(
    attrs: &A,
    target: T,
    content: Slottable<C>,
) -> (T, A, C)
where
    A: Clone,
{
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

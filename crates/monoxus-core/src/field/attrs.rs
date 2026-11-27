use crate::foundation::{
    compose::{Slottable, project_attrs_as_child},
    shared::{
        data_disabled_attr, data_invalid_attr, data_readonly_attr, data_required_attr,
    },
};

use super::{
    state::FieldDefinition,
    types::{FieldErrorLiveMode, FieldLegendVariant},
};

#[inline]
const fn aria_bool_attr(flag: bool) -> Option<&'static str> {
    if flag { Some("true") } else { None }
}

/// Computed attributes for `FieldRoot` (`<div role="group">`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldRootAttrs {
    pub id: String,
    pub role: Option<&'static str>,
    pub aria_labelledby: Option<String>,
    pub aria_describedby: Option<String>,
    pub data_orientation: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_monoxus_field_root: &'static str,
}

impl FieldRootAttrs {
    pub fn from_definition(
        def: &FieldDefinition,
        role: Option<&'static str>,
        aria_labelledby: Option<String>,
        aria_describedby: Option<String>,
    ) -> Self {
        Self {
            id: def.relationships.root_id().to_owned(),
            role,
            aria_labelledby,
            aria_describedby,
            data_orientation: def.orientation.as_str(),
            data_invalid: data_invalid_attr(def.invalid),
            data_disabled: data_disabled_attr(def.disabled),
            data_required: data_required_attr(def.required),
            data_readonly: data_readonly_attr(def.read_only, def.disabled),
            data_monoxus_field_root: "",
        }
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        project_attrs_as_child(self, target, content)
    }
}

/// Computed attributes for `FieldSet` (`<fieldset>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldSetAttrs {
    pub id: String,
    pub disabled: bool,
    pub aria_invalid: Option<&'static str>,
    pub aria_describedby: Option<String>,
    pub data_orientation: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_monoxus_field_set: &'static str,
}

impl FieldSetAttrs {
    pub fn from_definition(def: &FieldDefinition, aria_describedby: Option<String>) -> Self {
        Self {
            id: def.relationships.fieldset_id().to_owned(),
            disabled: def.disabled,
            aria_invalid: aria_bool_attr(def.invalid),
            aria_describedby,
            data_orientation: def.orientation.as_str(),
            data_invalid: data_invalid_attr(def.invalid),
            data_disabled: data_disabled_attr(def.disabled),
            data_required: data_required_attr(def.required),
            data_readonly: data_readonly_attr(def.read_only, def.disabled),
            data_monoxus_field_set: "",
        }
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        project_attrs_as_child(self, target, content)
    }
}

/// Computed attributes for `FieldLegend` (`<legend>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldLegendAttrs {
    pub id: String,
    pub data_variant: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_monoxus_field_legend: &'static str,
}

impl FieldLegendAttrs {
    pub fn new(
        id: impl Into<String>,
        variant: FieldLegendVariant,
        invalid: bool,
        disabled: bool,
        required: bool,
        read_only: bool,
    ) -> Self {
        Self {
            id: id.into(),
            data_variant: variant.as_str(),
            data_invalid: data_invalid_attr(invalid),
            data_disabled: data_disabled_attr(disabled),
            data_required: data_required_attr(required),
            data_readonly: data_readonly_attr(read_only, disabled),
            data_monoxus_field_legend: "",
        }
    }

    pub fn from_definition(def: &FieldDefinition, variant: FieldLegendVariant) -> Self {
        Self::new(
            def.relationships.legend_id(),
            variant,
            def.invalid,
            def.disabled,
            def.required,
            def.read_only,
        )
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        project_attrs_as_child(self, target, content)
    }
}

/// Computed attributes for `FieldLabel` (`<label>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldLabelAttrs {
    pub id: String,
    pub r#for: Option<String>,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_monoxus_field_label: &'static str,
}

impl FieldLabelAttrs {
    pub fn new(
        id: impl Into<String>,
        html_for: Option<String>,
        invalid: bool,
        disabled: bool,
        required: bool,
        read_only: bool,
    ) -> Self {
        Self {
            id: id.into(),
            r#for: html_for.filter(|v| !v.trim().is_empty()),
            data_invalid: data_invalid_attr(invalid),
            data_disabled: data_disabled_attr(disabled),
            data_required: data_required_attr(required),
            data_readonly: data_readonly_attr(read_only, disabled),
            data_monoxus_field_label: "",
        }
    }

    pub fn from_definition(def: &FieldDefinition, html_for: Option<String>) -> Self {
        Self::new(
            def.relationships.label_id(),
            html_for,
            def.invalid,
            def.disabled,
            def.required,
            def.read_only,
        )
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        project_attrs_as_child(self, target, content)
    }
}

/// Computed attributes for a field control slot (`FieldControl` / `use_field_control`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldControlAttrs {
    pub id: String,
    pub name: Option<String>,
    pub aria_labelledby: Option<String>,
    pub aria_describedby: Option<String>,
    pub aria_invalid: Option<&'static str>,
    pub aria_required: Option<&'static str>,
    pub aria_disabled: Option<&'static str>,
    pub aria_readonly: Option<&'static str>,
    pub disabled: bool,
    pub required: bool,
    pub read_only: bool,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_monoxus_field_control: &'static str,
}

impl FieldControlAttrs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        name: Option<String>,
        aria_labelledby: Option<String>,
        aria_describedby: Option<String>,
        invalid: bool,
        disabled: bool,
        required: bool,
        read_only: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.filter(|n| !n.is_empty()),
            aria_labelledby: aria_labelledby.filter(|v| !v.trim().is_empty()),
            aria_describedby: aria_describedby.filter(|v| !v.trim().is_empty()),
            aria_invalid: aria_bool_attr(invalid),
            aria_required: aria_bool_attr(required),
            aria_disabled: aria_bool_attr(disabled),
            aria_readonly: aria_bool_attr(read_only && !disabled),
            disabled,
            required,
            read_only,
            data_invalid: data_invalid_attr(invalid),
            data_disabled: data_disabled_attr(disabled),
            data_required: data_required_attr(required),
            data_readonly: data_readonly_attr(read_only, disabled),
            data_monoxus_field_control: "",
        }
    }

    pub fn from_definition(
        def: &FieldDefinition,
        control_id: Option<String>,
        aria_labelledby: Option<String>,
        aria_describedby: Option<String>,
    ) -> Self {
        let id = control_id
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| def.relationships.control_id().to_owned());
        Self::new(
            id,
            def.name.clone(),
            aria_labelledby,
            aria_describedby,
            def.invalid,
            def.disabled,
            def.required,
            def.read_only,
        )
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        project_attrs_as_child(self, target, content)
    }
}

/// Computed attributes for `FieldDescription` (`<p>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldDescriptionAttrs {
    pub id: String,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_monoxus_field_description: &'static str,
}

impl FieldDescriptionAttrs {
    pub fn new(id: impl Into<String>, invalid: bool, disabled: bool) -> Self {
        Self {
            id: id.into(),
            data_invalid: data_invalid_attr(invalid),
            data_disabled: data_disabled_attr(disabled),
            data_monoxus_field_description: "",
        }
    }

    pub fn from_definition(def: &FieldDefinition) -> Self {
        Self::new(def.relationships.description_id(), def.invalid, def.disabled)
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        project_attrs_as_child(self, target, content)
    }
}

/// Computed attributes for `FieldError` (`<div role="alert">`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldErrorAttrs {
    pub id: String,
    pub role: Option<&'static str>,
    pub aria_live: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_monoxus_field_error: &'static str,
}

impl FieldErrorAttrs {
    pub fn new(
        id: impl Into<String>,
        active: bool,
        live_mode: FieldErrorLiveMode,
        invalid: bool,
        disabled: bool,
    ) -> Self {
        Self {
            id: id.into(),
            role: active.then_some("alert"),
            aria_live: if active {
                live_mode.as_aria_live()
            } else {
                None
            },
            data_invalid: data_invalid_attr(invalid || active),
            data_disabled: data_disabled_attr(disabled),
            data_monoxus_field_error: "",
        }
    }

    pub fn from_definition(
        def: &FieldDefinition,
        active: bool,
        live_mode: FieldErrorLiveMode,
    ) -> Self {
        Self::new(
            def.relationships.error_id(),
            active,
            live_mode,
            def.invalid,
            def.disabled,
        )
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        project_attrs_as_child(self, target, content)
    }
}

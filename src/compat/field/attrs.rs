// [KEEP-MONOXUS-BRAND]
use ui_sdk::foundation::compose::{Slottable, project_attrs_as_child};

use super::{
    state::FieldDefinition,
    types::{FieldErrorLiveMode, FieldLegendVariant},
};

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

impl From<ui_sdk::field::attrs::FieldRootAttrs> for FieldRootAttrs {
    fn from(base: ui_sdk::field::attrs::FieldRootAttrs) -> Self {
        Self {
            id: base.id,
            role: base.role,
            aria_labelledby: base.aria_labelledby,
            aria_describedby: base.aria_describedby,
            data_orientation: base.data_orientation,
            data_invalid: base.data_invalid,
            data_disabled: base.data_disabled,
            data_required: base.data_required,
            data_readonly: base.data_readonly,
            data_monoxus_field_root: base.data_field_root,
        }
    }
}

impl FieldRootAttrs {
    pub fn from_definition(
        def: &FieldDefinition,
        role: Option<&'static str>,
        aria_labelledby: Option<String>,
        aria_describedby: Option<String>,
    ) -> Self {
        ui_sdk::field::attrs::FieldRootAttrs::from_definition(
            def,
            role,
            aria_labelledby,
            aria_describedby,
        )
        .into()
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

impl From<ui_sdk::field::attrs::FieldSetAttrs> for FieldSetAttrs {
    fn from(base: ui_sdk::field::attrs::FieldSetAttrs) -> Self {
        Self {
            id: base.id,
            disabled: base.disabled,
            aria_invalid: base.aria_invalid,
            aria_describedby: base.aria_describedby,
            data_orientation: base.data_orientation,
            data_invalid: base.data_invalid,
            data_disabled: base.data_disabled,
            data_required: base.data_required,
            data_readonly: base.data_readonly,
            data_monoxus_field_set: base.data_field_set,
        }
    }
}

impl FieldSetAttrs {
    pub fn from_definition(def: &FieldDefinition, aria_describedby: Option<String>) -> Self {
        ui_sdk::field::attrs::FieldSetAttrs::from_definition(def, aria_describedby).into()
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

impl From<ui_sdk::field::attrs::FieldLegendAttrs> for FieldLegendAttrs {
    fn from(base: ui_sdk::field::attrs::FieldLegendAttrs) -> Self {
        Self {
            id: base.id,
            data_variant: base.data_variant,
            data_invalid: base.data_invalid,
            data_disabled: base.data_disabled,
            data_required: base.data_required,
            data_readonly: base.data_readonly,
            data_monoxus_field_legend: base.data_field_legend,
        }
    }
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
        ui_sdk::field::attrs::FieldLegendAttrs::new(
            id, variant, invalid, disabled, required, read_only,
        )
        .into()
    }

    pub fn from_definition(def: &FieldDefinition, variant: FieldLegendVariant) -> Self {
        ui_sdk::field::attrs::FieldLegendAttrs::from_definition(def, variant).into()
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

impl From<ui_sdk::field::attrs::FieldLabelAttrs> for FieldLabelAttrs {
    fn from(base: ui_sdk::field::attrs::FieldLabelAttrs) -> Self {
        Self {
            id: base.id,
            r#for: base.r#for,
            data_invalid: base.data_invalid,
            data_disabled: base.data_disabled,
            data_required: base.data_required,
            data_readonly: base.data_readonly,
            data_monoxus_field_label: base.data_field_label,
        }
    }
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
        ui_sdk::field::attrs::FieldLabelAttrs::new(
            id, html_for, invalid, disabled, required, read_only,
        )
        .into()
    }

    pub fn from_definition(def: &FieldDefinition, html_for: Option<String>) -> Self {
        ui_sdk::field::attrs::FieldLabelAttrs::from_definition(def, html_for).into()
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

impl From<ui_sdk::field::attrs::FieldControlAttrs> for FieldControlAttrs {
    fn from(base: ui_sdk::field::attrs::FieldControlAttrs) -> Self {
        Self {
            id: base.id,
            name: base.name,
            aria_labelledby: base.aria_labelledby,
            aria_describedby: base.aria_describedby,
            aria_invalid: base.aria_invalid,
            aria_required: base.aria_required,
            aria_disabled: base.aria_disabled,
            aria_readonly: base.aria_readonly,
            disabled: base.disabled,
            required: base.required,
            read_only: base.read_only,
            data_invalid: base.data_invalid,
            data_disabled: base.data_disabled,
            data_required: base.data_required,
            data_readonly: base.data_readonly,
            data_monoxus_field_control: base.data_field_control,
        }
    }
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
        ui_sdk::field::attrs::FieldControlAttrs::new(
            id,
            name,
            aria_labelledby,
            aria_describedby,
            invalid,
            disabled,
            required,
            read_only,
        )
        .into()
    }

    pub fn from_definition(
        def: &FieldDefinition,
        control_id: Option<String>,
        aria_labelledby: Option<String>,
        aria_describedby: Option<String>,
    ) -> Self {
        ui_sdk::field::attrs::FieldControlAttrs::from_definition(
            def,
            control_id,
            aria_labelledby,
            aria_describedby,
        )
        .into()
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

impl From<ui_sdk::field::attrs::FieldDescriptionAttrs> for FieldDescriptionAttrs {
    fn from(base: ui_sdk::field::attrs::FieldDescriptionAttrs) -> Self {
        Self {
            id: base.id,
            data_invalid: base.data_invalid,
            data_disabled: base.data_disabled,
            data_monoxus_field_description: base.data_field_description,
        }
    }
}

impl FieldDescriptionAttrs {
    pub fn new(id: impl Into<String>, invalid: bool, disabled: bool) -> Self {
        ui_sdk::field::attrs::FieldDescriptionAttrs::new(id, invalid, disabled).into()
    }

    pub fn from_definition(def: &FieldDefinition) -> Self {
        ui_sdk::field::attrs::FieldDescriptionAttrs::from_definition(def).into()
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

impl From<ui_sdk::field::attrs::FieldErrorAttrs> for FieldErrorAttrs {
    fn from(base: ui_sdk::field::attrs::FieldErrorAttrs) -> Self {
        Self {
            id: base.id,
            role: base.role,
            aria_live: base.aria_live,
            data_invalid: base.data_invalid,
            data_disabled: base.data_disabled,
            data_monoxus_field_error: base.data_field_error,
        }
    }
}

impl FieldErrorAttrs {
    pub fn new(
        id: impl Into<String>,
        active: bool,
        live_mode: FieldErrorLiveMode,
        invalid: bool,
        disabled: bool,
    ) -> Self {
        ui_sdk::field::attrs::FieldErrorAttrs::new(id, active, live_mode, invalid, disabled).into()
    }

    pub fn from_definition(
        def: &FieldDefinition,
        active: bool,
        live_mode: FieldErrorLiveMode,
    ) -> Self {
        ui_sdk::field::attrs::FieldErrorAttrs::from_definition(def, active, live_mode).into()
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        project_attrs_as_child(self, target, content)
    }
}

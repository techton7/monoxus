// [KEEP-MONOXUS-BRAND]
use ui_sdk::foundation::shared::data_invalid_attr;

/// Attribute publisher for `PinInputRoot`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Serialize)]
pub struct PinInputRootAttributes {
    pub id: String,
    pub style: String,
    pub aria_disabled: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub data_monoxus_pin_input_root: &'static str,
}

impl From<ui_sdk::pin_input::attrs::PinInputRootAttributes> for PinInputRootAttributes {
    fn from(base: ui_sdk::pin_input::attrs::PinInputRootAttributes) -> Self {
        Self {
            id: base.id,
            style: base.style,
            aria_disabled: base.aria_disabled,
            data_disabled: base.data_disabled,
            data_invalid: base.data_invalid,
            data_monoxus_pin_input_root: base.data_pin_input_root,
        }
    }
}

impl PinInputRootAttributes {
    pub fn new(id: String, disabled: bool) -> Self {
        ui_sdk::pin_input::attrs::PinInputRootAttributes::new(id, disabled).into()
    }

    pub fn with_invalid(mut self, invalid: bool) -> Self {
        self.data_invalid = data_invalid_attr(invalid);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn style(&self) -> &str {
        &self.style
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_invalid(&self) -> Option<&'static str> {
        self.data_invalid
    }

    pub fn is_invalid(&self) -> bool {
        self.data_invalid.is_some()
    }

    pub fn data_monoxus_pin_input_root(&self) -> &'static str {
        self.data_monoxus_pin_input_root
    }
}

/// Attribute publisher for the overlaid `<input>` owned by `PinInputRoot`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Serialize)]
pub struct PinInputInputAttributes {
    pub id: String,
    pub r#type: &'static str,
    pub autocomplete: &'static str,
    pub inputmode: &'static str,
    pub pattern: Option<&'static str>,
    pub maxlength: usize,
    pub value: String,
    pub placeholder: Option<String>,
    pub disabled: bool,
    pub readonly: bool,
    pub required: bool,
    pub autofocus: bool,
    pub name: Option<String>,
    pub form: Option<String>,
    pub aria_invalid: Option<&'static str>,
    pub aria_describedby: Option<String>,
    pub aria_labelledby: Option<String>,
    pub aria_label: Option<String>,
    pub data_invalid: Option<&'static str>,
    pub style: String,
    pub data_monoxus_pin_input_input: &'static str,
    pub data_pin_input_input_mss: Option<String>,
    pub data_pin_input_input_mse: Option<String>,
}

/// Alias matching compound naming conventions when referenced as hidden input attributes.
pub type PinInputHiddenAttributes = PinInputInputAttributes;

impl From<ui_sdk::pin_input::attrs::PinInputInputAttributes> for PinInputInputAttributes {
    fn from(base: ui_sdk::pin_input::attrs::PinInputInputAttributes) -> Self {
        Self {
            id: base.id,
            r#type: base.r#type,
            autocomplete: base.autocomplete,
            inputmode: base.inputmode,
            pattern: base.pattern,
            maxlength: base.maxlength,
            value: base.value,
            placeholder: base.placeholder,
            disabled: base.disabled,
            readonly: base.readonly,
            required: base.required,
            autofocus: base.autofocus,
            name: base.name,
            form: base.form,
            aria_invalid: base.aria_invalid,
            aria_describedby: base.aria_describedby,
            aria_labelledby: base.aria_labelledby,
            aria_label: base.aria_label,
            data_invalid: base.data_invalid,
            style: base.style,
            data_monoxus_pin_input_input: base.data_pin_input_input,
            data_pin_input_input_mss: base.data_pin_input_input_mss,
            data_pin_input_input_mse: base.data_pin_input_input_mse,
        }
    }
}

impl PinInputInputAttributes {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        mask: bool,
        autocomplete: &'static str,
        inputmode: &'static str,
        pattern: Option<&'static str>,
        maxlength: usize,
        value: String,
        placeholder: Option<String>,
        disabled: bool,
        readonly: bool,
        required: bool,
        autofocus: bool,
        name: Option<String>,
        form: Option<String>,
        text_align: &'static str,
        selection_start: Option<usize>,
        selection_end: Option<usize>,
    ) -> Self {
        ui_sdk::pin_input::attrs::PinInputInputAttributes::new(
            id,
            mask,
            autocomplete,
            inputmode,
            pattern,
            maxlength,
            value,
            placeholder,
            disabled,
            readonly,
            required,
            autofocus,
            name,
            form,
            text_align,
            selection_start,
            selection_end,
        )
        .into()
    }

    pub fn with_invalid(mut self, invalid: bool) -> Self {
        self.aria_invalid = invalid.then_some("true");
        self.data_invalid = data_invalid_attr(invalid);
        self
    }

    pub fn with_aria_describedby(mut self, aria_describedby: Option<String>) -> Self {
        self.aria_describedby = aria_describedby;
        self
    }

    pub fn with_aria_labelledby(mut self, aria_labelledby: Option<String>) -> Self {
        self.aria_labelledby = aria_labelledby;
        self
    }

    pub fn with_aria_label(mut self, aria_label: Option<String>) -> Self {
        self.aria_label = aria_label;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn input_type(&self) -> &'static str {
        self.r#type
    }

    pub fn autocomplete(&self) -> &'static str {
        self.autocomplete
    }

    pub fn inputmode(&self) -> &'static str {
        self.inputmode
    }

    pub fn pattern(&self) -> Option<&'static str> {
        self.pattern
    }

    pub fn maxlength(&self) -> usize {
        self.maxlength
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn aria_invalid(&self) -> Option<&'static str> {
        self.aria_invalid
    }

    pub fn aria_describedby(&self) -> Option<&str> {
        self.aria_describedby.as_deref()
    }

    pub fn aria_labelledby(&self) -> Option<&str> {
        self.aria_labelledby.as_deref()
    }

    pub fn aria_label(&self) -> Option<&str> {
        self.aria_label.as_deref()
    }

    pub fn data_invalid(&self) -> Option<&'static str> {
        self.data_invalid
    }

    pub fn style(&self) -> &str {
        &self.style
    }

    pub fn data_monoxus_pin_input_input(&self) -> &'static str {
        self.data_monoxus_pin_input_input
    }
}

/// Attribute publisher for `PinInputCell`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Serialize)]
pub struct PinInputCellAttributes {
    pub id: String,
    pub index: usize,
    pub data_active: Option<&'static str>,
    pub data_inactive: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub data_monoxus_pin_input_cell: &'static str,
}

impl From<ui_sdk::pin_input::attrs::PinInputCellAttributes> for PinInputCellAttributes {
    fn from(base: ui_sdk::pin_input::attrs::PinInputCellAttributes) -> Self {
        Self {
            id: base.id,
            index: base.index,
            data_active: base.data_active,
            data_inactive: base.data_inactive,
            data_disabled: base.data_disabled,
            data_invalid: base.data_invalid,
            data_monoxus_pin_input_cell: base.data_pin_input_cell,
        }
    }
}

impl PinInputCellAttributes {
    pub fn new(id: String, index: usize, is_active: bool, disabled: bool) -> Self {
        ui_sdk::pin_input::attrs::PinInputCellAttributes::new(id, index, is_active, disabled).into()
    }

    pub fn with_invalid(mut self, invalid: bool) -> Self {
        self.data_invalid = data_invalid_attr(invalid);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn is_active(&self) -> bool {
        self.data_active.is_some()
    }

    pub fn is_inactive(&self) -> bool {
        self.data_inactive.is_some()
    }

    pub fn is_disabled(&self) -> bool {
        self.data_disabled.is_some()
    }

    pub fn is_invalid(&self) -> bool {
        self.data_invalid.is_some()
    }

    pub fn data_active(&self) -> Option<&'static str> {
        self.data_active
    }

    pub fn data_inactive(&self) -> Option<&'static str> {
        self.data_inactive
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_invalid(&self) -> Option<&'static str> {
        self.data_invalid
    }

    pub fn data_monoxus_pin_input_cell(&self) -> &'static str {
        self.data_monoxus_pin_input_cell
    }
}

use crate::foundation::shared::{data_disabled_attr, data_invalid_attr};

/// Attribute publisher for `PinInputRoot`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinInputRootAttributes {
    pub id: String,
    pub style: String,
    pub aria_disabled: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub data_monoxus_pin_input_root: &'static str,
}

impl PinInputRootAttributes {
    pub fn new(id: String, disabled: bool) -> Self {
        let cursor = if disabled { "default" } else { "text" };
        Self {
            id,
            style: format!(
                "position: relative; user-select: none; -webkit-user-select: none; cursor: {cursor};"
            ),
            aria_disabled: disabled.then_some("true"),
            data_disabled: data_disabled_attr(disabled),
            data_invalid: None,
            data_monoxus_pin_input_root: "",
        }
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
        let cursor = if disabled { "default" } else { "text" };
        // `opacity: 0` suppresses Blitz's unconditional `SELECTION_COLOR` highlight
        // and plaintext rendering while keeping hit-testing and focus intact.
        let style = format!(
            "position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; border: 0; margin: 0; padding: 0; background: transparent; cursor: {cursor}; text-align: {text_align};"
        );
        Self {
            id,
            r#type: if mask { "password" } else { "text" },
            autocomplete,
            inputmode,
            pattern,
            maxlength: maxlength.max(1),
            value,
            placeholder,
            disabled,
            readonly,
            required,
            autofocus,
            name: name.filter(|s| !s.is_empty()),
            form: form.filter(|s| !s.is_empty()),
            aria_invalid: None,
            aria_describedby: None,
            aria_labelledby: None,
            aria_label: None,
            data_invalid: None,
            style,
            data_monoxus_pin_input_input: "",
            data_pin_input_input_mss: selection_start.map(|s| s.to_string()),
            data_pin_input_input_mse: selection_end.map(|e| e.to_string()),
        }
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
pub struct PinInputCellAttributes {
    pub id: String,
    pub index: usize,
    pub data_active: Option<&'static str>,
    pub data_inactive: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub data_monoxus_pin_input_cell: &'static str,
}

impl PinInputCellAttributes {
    pub fn new(id: String, index: usize, is_active: bool, disabled: bool) -> Self {
        let active = is_active && !disabled;
        Self {
            id,
            index,
            data_active: data_disabled_attr(active),
            data_inactive: data_disabled_attr(!active),
            data_disabled: data_disabled_attr(disabled),
            data_invalid: None,
            data_monoxus_pin_input_cell: "",
        }
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


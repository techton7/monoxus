//! Pure form-associated hidden `<input>` configuration and attribute contracts.
//!
//! Host-agnostic counterpart to `monoxus::foundation::form`, containing zero Dioxus or
//! browser dependencies.

/// Out-of-flow 1x1px visual clamp style overlaid adjacent to the headless trigger.
///
/// Prevents Flexbox/Grid item or `gap` distortion across Web and Blitz Native Taffy layout
/// while keeping native browser constraint validation popups anchored near the trigger.
pub const HIDDEN_INPUT_VISUAL_CLAMP_STYLE: &str = "position: absolute; pointer-events: none; opacity: 0; margin: 0; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; padding: 0; transform: translateX(-100%);";

/// HTML `<input type="...">` variants supported by the shared hidden form helper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HiddenInputKind {
    #[default]
    Checkbox,
    Radio,
    Hidden,
    Text,
}

impl HiddenInputKind {
    /// Returns the standard lowercase HTML `type` attribute token.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Hidden => "hidden",
            Self::Text => "text",
        }
    }

    /// Returns `true` when the input type participates in HTML `checked` state (`checkbox` or `radio`).
    #[inline]
    pub const fn uses_checked_state(&self) -> bool {
        matches!(self, Self::Checkbox | Self::Radio)
    }
}

/// Host-neutral configuration for deriving [`HiddenInputAttributes`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HiddenInputConfig {
    pub id: Option<String>,
    pub kind: HiddenInputKind,
    pub name: Option<String>,
    pub value: String,
    pub checked: Option<bool>,
    pub disabled: bool,
    pub required: bool,
    pub read_only: bool,
    pub form: Option<String>,
    pub autocomplete: Option<String>,
}

impl Default for HiddenInputConfig {
    fn default() -> Self {
        Self {
            id: None,
            kind: HiddenInputKind::default(),
            name: None,
            value: String::from("on"),
            checked: None,
            disabled: false,
            required: false,
            read_only: false,
            form: None,
            autocomplete: Some(String::from("off")),
        }
    }
}

/// Pure attribute publisher for hidden form-associated `<input>` elements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HiddenInputAttributes {
    pub id: Option<String>,
    pub r#type: &'static str,
    pub name: Option<String>,
    pub value: String,
    pub checked: Option<bool>,
    pub disabled: bool,
    pub required: bool,
    pub read_only: bool,
    pub form: Option<String>,
    pub autocomplete: Option<String>,
    pub aria_hidden: &'static str,
    pub tabindex: i32,
    pub style: &'static str,
}

impl HiddenInputAttributes {
    /// Derives normalized HTML attributes from [`HiddenInputConfig`].
    pub fn from_config(config: &HiddenInputConfig) -> Self {
        let checked = if config.kind.uses_checked_state() {
            Some(config.checked.unwrap_or(false))
        } else {
            None
        };
        Self {
            id: config.id.clone().filter(|id| !id.is_empty()),
            r#type: config.kind.as_str(),
            name: config.name.clone().filter(|name| !name.is_empty()),
            value: config.value.clone(),
            checked,
            disabled: config.disabled,
            required: config.required,
            read_only: config.read_only,
            form: config.form.clone().filter(|form| !form.is_empty()),
            autocomplete: config
                .autocomplete
                .clone()
                .filter(|autocomplete| !autocomplete.is_empty()),
            aria_hidden: "true",
            tabindex: -1,
            style: HIDDEN_INPUT_VISUAL_CLAMP_STYLE,
        }
    }
}

/// Pure helper determining whether a form-associated hidden input should mount immediately
/// prior to or alongside live DOM `<form>` ancestor detection.
#[inline]
pub fn should_render_hidden_input(
    name: Option<&str>,
    form: Option<&str>,
    has_ancestor_form: bool,
) -> bool {
    name.is_some_and(|n| !n.is_empty())
        || form.is_some_and(|f| !f.is_empty())
        || has_ancestor_form
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_input_kind_tokens_and_checked_semantics() {
        assert_eq!(HiddenInputKind::Checkbox.as_str(), "checkbox");
        assert!(HiddenInputKind::Checkbox.uses_checked_state());

        assert_eq!(HiddenInputKind::Radio.as_str(), "radio");
        assert!(HiddenInputKind::Radio.uses_checked_state());

        assert_eq!(HiddenInputKind::Hidden.as_str(), "hidden");
        assert!(!HiddenInputKind::Hidden.uses_checked_state());

        assert_eq!(HiddenInputKind::Text.as_str(), "text");
        assert!(!HiddenInputKind::Text.uses_checked_state());
    }

    #[test]
    fn hidden_input_attributes_normalize_empty_optional_fields_and_checked() {
        let checkbox_cfg = HiddenInputConfig {
            id: Some(String::new()),
            kind: HiddenInputKind::Checkbox,
            name: Some(String::from("terms")),
            value: String::from("yes"),
            checked: None,
            disabled: false,
            required: true,
            read_only: false,
            form: Some(String::new()),
            autocomplete: Some(String::new()),
        };
        let attrs = HiddenInputAttributes::from_config(&checkbox_cfg);
        assert_eq!(attrs.id, None);
        assert_eq!(attrs.r#type, "checkbox");
        assert_eq!(attrs.name.as_deref(), Some("terms"));
        assert_eq!(attrs.value, "yes");
        assert_eq!(attrs.checked, Some(false));
        assert!(attrs.required);
        assert_eq!(attrs.form, None);
        assert_eq!(attrs.autocomplete, None);
        assert_eq!(attrs.aria_hidden, "true");
        assert_eq!(attrs.tabindex, -1);
        assert_eq!(attrs.style, HIDDEN_INPUT_VISUAL_CLAMP_STYLE);

        let hidden_cfg = HiddenInputConfig {
            kind: HiddenInputKind::Hidden,
            checked: Some(true),
            form: Some(String::from("profile-form")),
            ..Default::default()
        };
        let hidden_attrs = HiddenInputAttributes::from_config(&hidden_cfg);
        assert_eq!(hidden_attrs.r#type, "hidden");
        assert_eq!(hidden_attrs.checked, None);
        assert_eq!(hidden_attrs.value, "on");
        assert_eq!(hidden_attrs.form.as_deref(), Some("profile-form"));
        assert_eq!(hidden_attrs.autocomplete.as_deref(), Some("off"));
    }

    #[test]
    fn should_render_hidden_input_checks_non_empty_name_form_or_ancestor() {
        assert!(!should_render_hidden_input(None, None, false));
        assert!(!should_render_hidden_input(Some(""), Some(""), false));
        assert!(should_render_hidden_input(Some("field"), None, false));
        assert!(should_render_hidden_input(None, Some("form-1"), false));
        assert!(should_render_hidden_input(None, None, true));
    }
}

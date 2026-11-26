use dioxus::prelude::*;

use crate::foundation::form::{HiddenFormInput, HiddenFormInputs, HiddenInputKind};

#[component]
pub fn SelectHiddenInput(
    name: String,
    value: Option<String>,
    #[props(default)] values: Option<Vec<String>>,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] required: bool,
    #[props(default)] form: Option<String>,
    #[props(default)] autocomplete: Option<String>,
    #[props(default)] focus_target_id: Option<String>,
    #[props(default = 0)] reset_epoch: u64,
) -> Element {
    if let Some(list) = values {
        return rsx! {
            HiddenFormInputs {
                name: Some(name),
                values: list,
                emit_empty_placeholder: true,
                disabled: disabled,
                required: required,
                form: form,
                autocomplete: autocomplete,
                focus_target_id: focus_target_id,
                reset_epoch: reset_epoch,
            }
        };
    }

    rsx! {
        HiddenFormInput {
            kind: HiddenInputKind::Hidden,
            name: Some(name),
            value: value.unwrap_or_default(),
            disabled: disabled,
            required: required,
            form: form,
            autocomplete: autocomplete,
            focus_target_id: focus_target_id,
            reset_epoch: reset_epoch,
        }
    }
}

use dioxus::prelude::*;

use crate::foundation::form::{HiddenFormInput, HiddenFormInputs, HiddenInputKind};

use crate::combobox::{runtime::use_combobox_context, types::ComboboxMode};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxHiddenInputProps {
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default)]
    pub disabled: Option<bool>,
    #[props(default)]
    pub required: Option<bool>,
}

#[component]
pub fn ComboboxHiddenInput(props: ComboboxHiddenInputProps) -> Element {
    let ctx = use_combobox_context();
    let name = props.name.or_else(|| ctx.runtime.name());

    let Some(name_str) = name.filter(|n| !n.is_empty()) else {
        return rsx! {};
    };

    let disabled = props.disabled.unwrap_or_else(|| ctx.runtime.is_disabled());
    let required = props.required.unwrap_or_else(|| ctx.runtime.is_required());
    let form = props.form.or_else(|| ctx.runtime.form());
    let focus_target_id = Some(ctx.runtime.current_input_id());
    let reset_epoch = *ctx.runtime.state.reset_epoch.read();

    match ctx.runtime.mode() {
        ComboboxMode::Single => {
            let val = ctx.runtime.value().unwrap_or_default();
            rsx! {
                HiddenFormInput {
                    kind: HiddenInputKind::Hidden,
                    name: Some(name_str),
                    value: val,
                    disabled: disabled,
                    required: required,
                    form: form,
                    focus_target_id: focus_target_id,
                    reset_epoch: reset_epoch,
                }
            }
        }
        ComboboxMode::Multiple => {
            let vals = ctx.runtime.values();
            rsx! {
                HiddenFormInputs {
                    name: Some(name_str),
                    values: vals,
                    disabled: disabled,
                    required: required,
                    form: form,
                    focus_target_id: focus_target_id,
                    reset_epoch: reset_epoch,
                }
            }
        }
    }
}

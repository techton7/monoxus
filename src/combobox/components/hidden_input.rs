use dioxus::prelude::*;

use crate::foundation::{
    browser::{FocusOptions, focus_element_sync_or_defer},
    compose::merge_global_attributes,
    form::{HiddenFormInput, HiddenFormInputs, HiddenInputAttributes, HiddenInputConfig, HiddenInputKind},
};

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
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
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

    if props.attributes.is_empty() {
        return match ctx.runtime.mode() {
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
        };
    }

    let focus_target = focus_target_id.filter(|id| !id.is_empty());
    let on_focus = move |_evt: FocusEvent| {
        if let Some(ref target_id) = focus_target {
            focus_element_sync_or_defer(target_id, FocusOptions::new(false));
        }
    };

    match ctx.runtime.mode() {
        ComboboxMode::Single => {
            let val = ctx.runtime.value().unwrap_or_default();
            let attrs = HiddenInputAttributes::from_config(&HiddenInputConfig {
                id: None,
                kind: HiddenInputKind::Hidden,
                name: Some(name_str),
                value: val,
                checked: None,
                disabled,
                required,
                read_only: false,
                form,
                autocomplete: Some(String::from("off")),
            });
            let merged = merge_global_attributes(None, Some(attrs.style), props.attributes);
            rsx! {
                for epoch in [reset_epoch] {
                    input {
                        key: "{epoch}",
                        r#type: attrs.r#type,
                        name: attrs.name.as_deref(),
                        value: "{attrs.value}",
                        disabled: attrs.disabled,
                        required: attrs.required,
                        readonly: attrs.read_only,
                        form: attrs.form.as_deref(),
                        autocomplete: attrs.autocomplete.as_deref(),
                        "aria-hidden": attrs.aria_hidden,
                        tabindex: attrs.tabindex,
                        class: merged.class.as_deref(),
                        style: merged.style.as_deref(),
                        onfocus: on_focus.clone(),
                        ..merged.rest.clone(),
                    }
                }
            }
        }
        ComboboxMode::Multiple => {
            let vals = ctx.runtime.values();
            if vals.is_empty() {
                return rsx! {};
            }
            rsx! {
                for (idx, val) in vals.into_iter().enumerate() {
                    {
                        let attrs = HiddenInputAttributes::from_config(&HiddenInputConfig {
                            id: None,
                            kind: HiddenInputKind::Hidden,
                            name: Some(name_str.clone()),
                            value: val.clone(),
                            checked: None,
                            disabled,
                            required,
                            read_only: false,
                            form: form.clone(),
                            autocomplete: Some(String::from("off")),
                        });
                        let merged = merge_global_attributes(None, Some(attrs.style), props.attributes.clone());
                        let on_f = on_focus.clone();
                        rsx! {
                            input {
                                key: "{reset_epoch}-{idx}-{val}",
                                r#type: attrs.r#type,
                                name: attrs.name.as_deref(),
                                value: "{attrs.value}",
                                disabled: attrs.disabled,
                                required: attrs.required,
                                readonly: attrs.read_only,
                                form: attrs.form.as_deref(),
                                autocomplete: attrs.autocomplete.as_deref(),
                                "aria-hidden": attrs.aria_hidden,
                                tabindex: attrs.tabindex,
                                class: merged.class.as_deref(),
                                style: merged.style.as_deref(),
                                onfocus: on_f,
                                ..merged.rest,
                            }
                        }
                    }
                }
            }
        }
    }
}

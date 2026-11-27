use dioxus::prelude::*;

use crate::foundation::{
    browser::{FocusOptions, focus_element_sync_or_defer},
    compose::{merge_props, push_attr_opt, push_event_handler},
    form::{
        HiddenFormInput, HiddenFormInputs, HiddenInputAttributes, HiddenInputConfig,
        HiddenInputKind,
    },
};

use crate::combobox::{runtime::use_combobox_context, types::ComboboxMode};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxHiddenInputProps {
    #[props(default, into)]
    pub name: Option<String>,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default)]
    pub disabled: Option<bool>,
    #[props(default)]
    pub required: Option<bool>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

fn build_hidden_input_attrs(
    attrs: HiddenInputAttributes,
    on_focus: EventHandler<FocusEvent>,
    user_attrs: Vec<Attribute>,
) -> Vec<Attribute> {
    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("type", attrs.r#type, None, false));
    push_attr_opt(&mut base, "name", attrs.name.clone());
    base.push(Attribute::new("value", attrs.value.clone(), None, false));
    if attrs.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    if attrs.required {
        base.push(Attribute::new("required", true, None, false));
    }
    if attrs.read_only {
        base.push(Attribute::new("readonly", true, None, false));
    }
    push_attr_opt(&mut base, "form", attrs.form.clone());
    push_attr_opt(&mut base, "autocomplete", attrs.autocomplete.clone());
    base.push(Attribute::new("aria-hidden", attrs.aria_hidden, None, false));
    base.push(Attribute::new("tabindex", attrs.tabindex, None, false));
    if !attrs.style.is_empty() {
        base.push(Attribute::new("style", attrs.style, None, false));
    }
    push_event_handler(&mut base, "onfocus", Some(on_focus));
    merge_props(base, user_attrs)
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

    if props.attributes.is_empty() && props.child.is_none() {
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
    let on_focus = EventHandler::new(move |_evt: FocusEvent| {
        if let Some(ref target_id) = focus_target {
            focus_element_sync_or_defer(target_id, FocusOptions::new(false));
        }
    });

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
            let merged = build_hidden_input_attrs(attrs, on_focus, props.attributes);
            if let Some(child) = props.child {
                return child.call(merged);
            }
            rsx! {
                for epoch in [reset_epoch] {
                    input {
                        key: "{epoch}",
                        ..merged.clone(),
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
                        let merged =
                            build_hidden_input_attrs(attrs, on_focus, props.attributes.clone());
                        if let Some(child) = props.child {
                            child.call(merged)
                        } else {
                            rsx! {
                                input {
                                    key: "{reset_epoch}-{idx}-{val}",
                                    ..merged,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

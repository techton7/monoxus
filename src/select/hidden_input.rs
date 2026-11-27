use dioxus::prelude::*;

use crate::foundation::{
    browser::{FocusOptions, focus_element_sync_or_defer},
    compose::{merge_props, push_attr_opt, push_event_handler},
    form::{HiddenInputAttributes, HiddenInputConfig, HiddenInputKind},
};

#[derive(Clone, PartialEq, Props)]
pub struct SelectHiddenInputProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(into)]
    pub name: String,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default)]
    pub values: Option<Vec<String>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default, into)]
    pub autocomplete: Option<String>,
    #[props(default, into)]
    pub focus_target_id: Option<String>,
    #[props(default = 0)]
    pub reset_epoch: u64,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

fn build_hidden_input_attrs(
    config: &HiddenInputConfig,
    focus_target_id: Option<String>,
    user_attrs: Vec<Attribute>,
) -> Vec<Attribute> {
    let attrs = HiddenInputAttributes::from_config(config);
    let mut base = Vec::with_capacity(13);
    push_attr_opt(&mut base, "id", attrs.id.clone());
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
    base.push(Attribute::new(
        "tabindex",
        attrs.tabindex.to_string(),
        None,
        false,
    ));
    base.push(Attribute::new("style", attrs.style, None, false));
    let focus_target = focus_target_id.filter(|id| !id.is_empty());
    push_event_handler(
        &mut base,
        "onfocus",
        Some(EventHandler::new(move |_evt: FocusEvent| {
            if let Some(ref target_id) = focus_target {
                focus_element_sync_or_defer(target_id, FocusOptions::new(false));
            }
        })),
    );
    merge_props(base, user_attrs)
}

#[component]
pub fn SelectHiddenInput(props: SelectHiddenInputProps) -> Element {
    let SelectHiddenInputProps {
        id,
        name,
        value,
        values,
        disabled,
        required,
        read_only,
        form,
        autocomplete,
        focus_target_id,
        reset_epoch,
        attributes,
    } = props;

    if let Some(list) = values {
        if list.is_empty() {
            let merged = build_hidden_input_attrs(
                &HiddenInputConfig {
                    id,
                    kind: HiddenInputKind::Hidden,
                    name: Some(name),
                    value: String::new(),
                    checked: None,
                    disabled,
                    required,
                    read_only,
                    form,
                    autocomplete,
                },
                focus_target_id,
                attributes,
            );
            return rsx! {
                for epoch in [reset_epoch] {
                    input {
                        key: "{epoch}-empty",
                        ..merged.clone(),
                    }
                }
            };
        }

        return rsx! {
            for (idx, val) in list.into_iter().enumerate() {
                {
                    let item_id = if idx == 0 { id.clone() } else { None };
                    let merged = build_hidden_input_attrs(
                        &HiddenInputConfig {
                            id: item_id,
                            kind: HiddenInputKind::Hidden,
                            name: Some(name.clone()),
                            value: val.clone(),
                            checked: None,
                            disabled,
                            required,
                            read_only,
                            form: form.clone(),
                            autocomplete: autocomplete.clone(),
                        },
                        focus_target_id.clone(),
                        attributes.clone(),
                    );
                    rsx! {
                        input {
                            key: "{reset_epoch}-{idx}-{val}",
                            ..merged,
                        }
                    }
                }
            }
        };
    }

    let merged = build_hidden_input_attrs(
        &HiddenInputConfig {
            id,
            kind: HiddenInputKind::Hidden,
            name: Some(name),
            value: value.unwrap_or_default(),
            checked: None,
            disabled,
            required,
            read_only,
            form,
            autocomplete,
        },
        focus_target_id,
        attributes,
    );

    rsx! {
        for epoch in [reset_epoch] {
            input {
                key: "{epoch}",
                ..merged.clone(),
            }
        }
    }
}

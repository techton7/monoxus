use dioxus::prelude::*;

#[component]
pub fn SelectHiddenInput(
    name: String,
    value: Option<String>,
    #[props(default)] values: Option<Vec<String>>,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] required: bool,
    #[props(default)] form: Option<String>,
    #[props(default)] autocomplete: Option<String>,
) -> Element {
    let form_attr = form.as_deref();
    let auto_attr = autocomplete.as_deref();

    if let Some(list) = values {
        if !list.is_empty() {
            return rsx! {
                for v in list {
                    input {
                        key: "{v}",
                        r#type: "hidden",
                        name: "{name}",
                        value: "{v}",
                        disabled: disabled,
                        required: required,
                        form: form_attr,
                        autocomplete: auto_attr,
                        aria_hidden: "true",
                    }
                }
            };
        }
    }

    let val_str = value.unwrap_or_default();
    rsx! {
        input {
            r#type: "hidden",
            name: "{name}",
            value: "{val_str}",
            disabled: disabled,
            required: required,
            form: form_attr,
            autocomplete: auto_attr,
            aria_hidden: "true",
        }
    }
}

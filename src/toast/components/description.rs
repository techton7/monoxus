use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToastDescriptionProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub children: Element,
}

/// Secondary text container for toast description details.
#[component]
pub fn ToastDescription(props: ToastDescriptionProps) -> Element {
    rsx! {
        div {
            class: props.class.as_deref().unwrap_or_default(),
            {props.children}
        }
    }
}

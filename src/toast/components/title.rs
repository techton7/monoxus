use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToastTitleProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub children: Element,
}

/// Semantic heading container for primary toast message text.
#[component]
pub fn ToastTitle(props: ToastTitleProps) -> Element {
    rsx! {
        div {
            class: props.class.as_deref().unwrap_or_default(),
            {props.children}
        }
    }
}

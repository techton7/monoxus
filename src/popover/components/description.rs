use dioxus::prelude::*;

use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverDescriptionProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub children: Element,
}

#[component]
pub fn PopoverDescription(props: PopoverDescriptionProps) -> Element {
    let mut ctx = use_context::<PopoverContext>();
    let desc_id = props
        .id
        .unwrap_or_else(|| ctx.relationships.description_id().to_string());

    if ctx.description_id.peek().as_deref() != Some(desc_id.as_str()) {
        ctx.description_id.set(Some(desc_id.clone()));
    }

    let cleanup_id = desc_id.clone();
    let mut cleanup_ctx = ctx.clone();
    dioxus::core::use_drop(move || {
        if cleanup_ctx.description_id.peek().as_deref() == Some(cleanup_id.as_str()) {
            cleanup_ctx.description_id.set(None);
        }
    });

    rsx! {
        p {
            id: "{desc_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            {props.children}
        }
    }
}

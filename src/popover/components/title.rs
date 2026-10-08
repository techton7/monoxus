use dioxus::prelude::*;

use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverTitleProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub children: Element,
}

#[component]
pub fn PopoverTitle(props: PopoverTitleProps) -> Element {
    let mut ctx = use_context::<PopoverContext>();
    let title_id = props
        .id
        .unwrap_or_else(|| ctx.relationships.title_id().to_string());

    if ctx.title_id.peek().as_deref() != Some(title_id.as_str()) {
        ctx.title_id.set(Some(title_id.clone()));
    }

    let cleanup_id = title_id.clone();
    let mut cleanup_ctx = ctx.clone();
    dioxus::core::use_drop(move || {
        if cleanup_ctx.title_id.peek().as_deref() == Some(cleanup_id.as_str()) {
            cleanup_ctx.title_id.set(None);
        }
    });

    rsx! {
        h2 {
            id: "{title_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            {props.children}
        }
    }
}

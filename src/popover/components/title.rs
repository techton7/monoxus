use dioxus::prelude::*;

use crate::foundation::compose::merge_global_attributes;
use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverTitleProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
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
        if cleanup_ctx
            .title_id
            .try_peek()
            .map(|id| id.as_deref() == Some(cleanup_id.as_str()))
            .unwrap_or(false)
        {
            let _ = cleanup_ctx.title_id.try_write().map(|mut w| *w = None);
        }
    });

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        h2 {
            id: "{title_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            ..merged.rest,
            {props.children}
        }
    }
}

use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props};
use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverDescriptionProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn PopoverDescription(props: PopoverDescriptionProps) -> Element {
    let PopoverDescriptionProps {
        id,
        class,
        style,
        child,
        attributes,
        children,
    } = props;
    let mut ctx = use_context::<PopoverContext>();
    let desc_id = id.unwrap_or_else(|| ctx.relationships.description_id().to_string());

    if ctx.description_id.peek().as_deref() != Some(desc_id.as_str()) {
        ctx.description_id.set(Some(desc_id.clone()));
    }

    let cleanup_id = desc_id.clone();
    let mut cleanup_ctx = ctx.clone();
    dioxus::core::use_drop(move || {
        if cleanup_ctx
            .description_id
            .try_peek()
            .map(|id| id.as_deref() == Some(cleanup_id.as_str()))
            .unwrap_or(false)
        {
            let _ = cleanup_ctx.description_id.try_write().map(|mut w| *w = None);
        }
    });

    let base = vec![Attribute::new("id", desc_id, None, false)];
    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        p {
            ..merged,
            {children}
        }
    }
}

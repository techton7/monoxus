use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props, push_event_handler};
use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverAnchorProps {
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
pub fn PopoverAnchor(props: PopoverAnchorProps) -> Element {
    let PopoverAnchorProps {
        id,
        class,
        style,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();
    let anchor = runtime.anchor();
    let anchor_id = id.unwrap_or_else(|| anchor.id().to_string());
    let mut mount_cb = runtime.mount_anchor();

    let mut base = Vec::with_capacity(2);
    base.push(Attribute::new("id", anchor_id, None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

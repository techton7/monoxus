use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};

pub use ui_sdk::visually_hidden::*;

/// Properties for the headless `VisuallyHidden` primitive.
#[derive(Clone, PartialEq, Props)]
pub struct VisuallyHiddenProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Screen-reader-only container hiding content visually while keeping it exposed to assistive technology.
#[component]
pub fn VisuallyHidden(props: VisuallyHiddenProps) -> Element {
    let attrs = VisuallyHiddenState::new().attributes();

    let mut base = Vec::with_capacity(3);
    push_attr_opt(&mut base, "id", props.id);
    base.push(Attribute::new("style", attrs.style, None, false));
    base.push(Attribute::new(
        "data-visually-hidden-root",
        attrs.data_visually_hidden_root,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onclick", props.on_click);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);
    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        span {
            ..merged,
            {props.children}
        }
    }
}

pub use VisuallyHidden as VisuallyHiddenRoot;

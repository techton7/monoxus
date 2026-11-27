use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};

pub use ui_sdk::separator::*;

/// Properties for the headless `Separator` component.
#[derive(Clone, PartialEq, Props)]
pub struct SeparatorProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub orientation: SeparatorOrientation,
    #[props(default = false)]
    pub decorative: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Headless separator component rendering semantic or decorative dividers.
#[component]
pub fn Separator(props: SeparatorProps) -> Element {
    let state = Separator::new()
        .with_orientation(props.orientation)
        .with_decorative(props.decorative);
    let attrs = state.attributes();

    let mut base = Vec::with_capacity(5);
    push_attr_opt(&mut base, "id", props.id);
    base.push(Attribute::new("role", attrs.role, None, false));
    push_attr_opt(&mut base, "aria-orientation", attrs.aria_orientation);
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation,
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-separator-root",
        attrs.data_separator_root,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);
    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
        }
    }
}

pub use Separator as SeparatorRoot;

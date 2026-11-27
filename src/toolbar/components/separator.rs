use dioxus::prelude::*;

use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};
use crate::toolbar::components::ToolbarRootContext;

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarSeparatorProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub decorative: bool,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ToolbarSeparator(props: ToolbarSeparatorProps) -> Element {
    let ctx = use_context::<ToolbarRootContext>();
    let toolbar_state = ctx.runtime.state();
    let attrs = toolbar_state.separator_attributes(props.id.clone(), props.decorative);

    let mut base = Vec::with_capacity(6);
    push_attr_opt(&mut base, "id", attrs.id().map(str::to_owned));
    base.push(Attribute::new("role", attrs.role(), None, false));
    push_attr_opt(&mut base, "aria-orientation", attrs.aria_orientation());
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-separator-root",
        attrs.data_separator_root(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-toolbar-separator",
        attrs.data_toolbar_separator(),
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

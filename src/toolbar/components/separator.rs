use dioxus::prelude::*;

use crate::foundation::compose::merge_global_attributes;
use crate::toolbar::components::ToolbarRootContext;

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarSeparatorProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub decorative: bool,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ToolbarSeparator(props: ToolbarSeparatorProps) -> Element {
    let ctx = use_context::<ToolbarRootContext>();
    let toolbar_state = ctx.runtime.state();
    let attrs = toolbar_state.separator_attributes(props.id.clone(), props.decorative);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: attrs.id(),
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: attrs.role(),
            aria_orientation: attrs.aria_orientation(),
            "data-orientation": attrs.data_orientation(),
            "data-separator-root": attrs.data_separator_root(),
            "data-toolbar-separator": attrs.data_toolbar_separator(),
            onmounted: move |evt| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(evt);
                }
            },
            ..merged.rest,
        }
    }
}

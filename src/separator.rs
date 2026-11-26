use dioxus::prelude::*;

use crate::foundation::compose::merge_global_attributes;

pub use monoxus_core::separator::*;

/// Properties for the headless `Separator` component.
#[derive(Clone, PartialEq, Props)]
pub struct SeparatorProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub orientation: SeparatorOrientation,
    #[props(default = false)]
    pub decorative: bool,
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

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: props.id,
            role: attrs.role,
            aria_orientation: attrs.aria_orientation,
            class: merged.class,
            style: merged.style,
            "data-orientation": attrs.data_orientation,
            "data-separator-root": attrs.data_separator_root,
            onmounted: move |evt| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(evt);
                }
            },
            ..merged.rest,
        }
    }
}

pub use Separator as SeparatorRoot;

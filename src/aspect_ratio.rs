use dioxus::prelude::*;

pub use monoxus_core::aspect_ratio::*;

/// Values supplied to the component-level `render_child` callback.
#[derive(Clone)]
pub struct AspectRatioRenderProps {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: String,
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub data_aspect_ratio_root: &'static str,
}

impl PartialEq for AspectRatioRenderProps {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.class == other.class
            && self.style == other.style
            && self.data_aspect_ratio_root == other.data_aspect_ratio_root
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AspectRatioProps {
    #[props(default = 1.0)]
    pub ratio: f64,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<AspectRatioRenderProps, Element>>,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Renders a padding wrapper around an absolutely inset Root.
#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    let state = AspectRatio::new(props.ratio);
    let attributes = state.attributes();
    let style = root_style(props.style.clone());

    let root = if props.as_child {
        if let Some(render_child) = props.render_child {
            render_child.call(AspectRatioRenderProps {
                id: props.id,
                class: props.class,
                style,
                on_click: props.on_click,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
                data_aspect_ratio_root: attributes.data_aspect_ratio_root,
            })
        } else {
            rsx! {
                {props.children}
            }
        }
    } else {
        rsx! {
            div {
                id: props.id,
                class: props.class,
                style,
                "data-aspect-ratio-root": attributes.data_aspect_ratio_root,
                onclick: move |event| {
                    if let Some(handler) = props.on_click.as_ref() {
                        handler.call(event);
                    }
                },
                onmounted: move |event| {
                    if let Some(handler) = props.on_mounted.as_ref() {
                        handler.call(event);
                    }
                },
                ..props.attributes,
                {props.children}
            }
        }
    };

    rsx! {
        div {
            style: attributes.wrapper_style,
            "data-radix-aspect-ratio-wrapper": attributes.data_radix_aspect_ratio_wrapper,
            "data-aspect-ratio-wrapper": attributes.data_aspect_ratio_wrapper,
            {root}
        }
    }
}

pub use AspectRatio as AspectRatioRoot;

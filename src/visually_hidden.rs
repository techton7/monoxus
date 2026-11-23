use dioxus::prelude::*;

pub use monoxus_core::visually_hidden::*;

/// Props passed to an explicit `render_child` callback for `VisuallyHidden`.
#[derive(Clone, PartialEq, Props)]
pub struct VisuallyHiddenRenderProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    pub style: String,
    pub data_visually_hidden_root: &'static str,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Properties for the headless `VisuallyHidden` primitive.
#[derive(Clone, PartialEq, Props)]
pub struct VisuallyHiddenProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<VisuallyHiddenRenderProps, Element>>,
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
    let attrs = VisuallyHiddenState::new().attributes_with_style(props.style.as_deref());

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(VisuallyHiddenRenderProps {
                id: props.id,
                class: props.class,
                style: attrs.style,
                data_visually_hidden_root: attrs.data_visually_hidden_root,
                on_click: props.on_click,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::VisuallyHidden: as_child requires render_child; rendering the default span"
        );
    }

    rsx! {
        span {
            id: props.id,
            class: props.class,
            style: attrs.style,
            "data-visually-hidden-root": attrs.data_visually_hidden_root,
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
}

pub use VisuallyHidden as VisuallyHiddenRoot;

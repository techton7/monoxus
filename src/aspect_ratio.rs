use dioxus::prelude::*;

use crate::foundation::compose::{Slottable, project_as_child};

pub const ASPECT_RATIO_WRAPPER_STYLE: &str = "position: relative; width: 100%;";
pub const ASPECT_RATIO_ROOT_STYLE: &str =
    "position: absolute; top: 0; right: 0; bottom: 0; left: 0;";

/// Pure aspect-ratio state and style projector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectRatio {
    ratio: f64,
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl AspectRatio {
    pub const fn new(ratio: f64) -> Self {
        Self { ratio }
    }

    pub const fn ratio(&self) -> f64 {
        self.ratio
    }

    /// Returns the wrapper padding percentage. Invalid host layout inputs collapse safely.
    pub fn padding_bottom_percent(&self) -> f64 {
        normalize_aspect_ratio(self.ratio)
    }

    pub fn attributes(&self) -> AspectRatioAttributes {
        AspectRatioAttributes {
            wrapper_style: format!(
                "{ASPECT_RATIO_WRAPPER_STYLE} padding-bottom: {}%;",
                self.padding_bottom_percent()
            ),
            root_style: ASPECT_RATIO_ROOT_STYLE.to_string(),
            data_radix_aspect_ratio_wrapper: "",
            data_aspect_ratio_wrapper: "",
            data_aspect_ratio_root: "",
        }
    }
}

/// Host-safe padding normalization approved for AspectRatio.
pub fn normalize_aspect_ratio(ratio: f64) -> f64 {
    if ratio.is_finite() && ratio > 0.0 {
        100.0 / ratio
    } else {
        0.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AspectRatioAttributes {
    pub wrapper_style: String,
    pub root_style: String,
    pub data_radix_aspect_ratio_wrapper: &'static str,
    pub data_aspect_ratio_wrapper: &'static str,
    pub data_aspect_ratio_root: &'static str,
}

/// Projects AspectRatio's state onto one custom target and its slotted content.
pub fn project_aspect_ratio_as_child<T, C>(
    state: &AspectRatio,
    target: T,
    content: Slottable<C>,
) -> (T, AspectRatioAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.attributes(), content)
}

/// Generic renderer adapter for hosts that supply their own element target.
pub fn render_aspect_ratio_child<T, C, R>(
    state: &AspectRatio,
    target: T,
    content: Slottable<C>,
    render_child: impl FnOnce(T, AspectRatioAttributes, C) -> R,
) -> R {
    let (target, attributes, content) = project_aspect_ratio_as_child(state, target, content);
    render_child(target, attributes, content)
}

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

fn root_style(consumer_style: Option<String>) -> String {
    match consumer_style {
        Some(style) if !style.trim().is_empty() => {
            format!("{style}; {ASPECT_RATIO_ROOT_STYLE}")
        }
        _ => ASPECT_RATIO_ROOT_STYLE.to_string(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_padding_normalization_follow_d2() {
        let default = AspectRatio::default();
        assert_eq!(default.ratio(), 1.0);
        assert_eq!(default.padding_bottom_percent(), 100.0);
        assert_eq!(normalize_aspect_ratio(16.0 / 9.0), 56.25);
        assert_eq!(normalize_aspect_ratio(4.0), 25.0);

        for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(normalize_aspect_ratio(invalid), 0.0);
        }
    }

    #[test]
    fn projects_exact_wrapper_root_styles_and_data_hooks() {
        let attributes = AspectRatio::new(4.0).attributes();
        assert_eq!(
            attributes.wrapper_style,
            "position: relative; width: 100%; padding-bottom: 25%;"
        );
        assert_eq!(
            attributes.root_style,
            "position: absolute; top: 0; right: 0; bottom: 0; left: 0;"
        );
        assert_eq!(attributes.data_radix_aspect_ratio_wrapper, "");
        assert_eq!(attributes.data_aspect_ratio_wrapper, "");
        assert_eq!(attributes.data_aspect_ratio_root, "");
    }

    #[test]
    fn projects_and_renders_exactly_one_as_child_target() {
        let state = AspectRatio::new(2.0);
        let (target, attributes, content) = project_aspect_ratio_as_child(
            &state,
            "picture-target",
            Slottable::new("picture-content"),
        );
        assert_eq!(target, "picture-target");
        assert_eq!(content, "picture-content");
        assert_eq!(attributes.wrapper_style.ends_with("50%;"), true);

        let rendered = render_aspect_ratio_child(
            &state,
            "article-target",
            Slottable::new(7_u8),
            |target, attributes, content| (target, attributes.root_style, content),
        );
        assert_eq!(rendered.0, "article-target");
        assert_eq!(rendered.1, ASPECT_RATIO_ROOT_STYLE);
        assert_eq!(rendered.2, 7);
    }

    #[test]
    fn consumer_style_precedes_required_root_inset() {
        assert_eq!(
            root_style(Some("display: flex".into())),
            "display: flex; position: absolute; top: 0; right: 0; bottom: 0; left: 0;"
        );
    }
}

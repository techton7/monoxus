use dioxus::prelude::*;

use crate::visually_hidden::VisuallyHidden;

pub use monoxus_core::accessible_icon::*;

/// Properties for the headless `AccessibleIcon` primitive.
#[derive(Clone, PartialEq, Props)]
pub struct AccessibleIconProps {
    pub label: String,
    #[props(default)]
    pub render_child: Option<Callback<AccessibleIconChildProps, Element>>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Decorative icon accessibility wrapper that hides the icon from assistive technology
/// and exposes a sibling `VisuallyHidden` accessible label.
#[component]
pub fn AccessibleIcon(props: AccessibleIconProps) -> Element {
    let child_props = AccessibleIconChildProps::new();

    let icon_node = if let Some(render_child) = props.render_child.as_ref() {
        Some(render_child.call(child_props))
    } else if let Some(children) = props.children {
        Some(rsx! {
            span {
                "aria-hidden": child_props.aria_hidden,
                style: ACCESSIBLE_ICON_FALLBACK_WRAPPER_STYLE,
                {children}
            }
        })
    } else {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::AccessibleIcon: expected `render_child` or `children`; rendering only the visually hidden label"
        );
        None
    };

    rsx! {
        if let Some(icon) = icon_node {
            {icon}
        }
        VisuallyHidden { "{props.label}" }
    }
}

pub use AccessibleIcon as AccessibleIconRoot;

use dioxus::prelude::*;

use crate::foundation::compose::merge_props;
use crate::visually_hidden::VisuallyHidden;

pub use ui_sdk::accessible_icon::*;

/// Properties for the headless `AccessibleIcon` primitive.
#[derive(Clone, PartialEq, Props)]
pub struct AccessibleIconProps {
    #[props(into)]
    pub label: String,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Decorative icon accessibility wrapper that hides the icon from assistive technology
/// and exposes a sibling `VisuallyHidden` accessible label.
#[component]
pub fn AccessibleIcon(props: AccessibleIconProps) -> Element {
    let child_props = AccessibleIconChildProps::new();

    let icon_node = if let Some(child) = props.child {
        let base = vec![
            Attribute::new("aria-hidden", child_props.aria_hidden, None, false),
            Attribute::new("focusable", child_props.focusable, None, false),
        ];
        let merged = merge_props(base, props.attributes);
        Some(child.call(merged))
    } else if let Some(children) = props.children {
        let base = vec![
            Attribute::new("aria-hidden", child_props.aria_hidden, None, false),
            Attribute::new(
                "style",
                ACCESSIBLE_ICON_FALLBACK_WRAPPER_STYLE,
                None,
                false,
            ),
        ];
        let merged = merge_props(base, props.attributes);
        Some(rsx! {
            span {
                ..merged,
                {children}
            }
        })
    } else {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::AccessibleIcon: expected `child` or `children`; rendering only the visually hidden label"
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

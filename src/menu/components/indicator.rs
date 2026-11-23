use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;
use crate::menu::{
    CheckedState, MenuIndicatorContext, MenuItemIndicatorAttributes, MenuPart, MenuRootContext,
    merge_attributes_with_part,
};

/// Render props passed to `render_child` when `ItemIndicator` uses `as_child: true`.
#[derive(Clone, PartialEq)]
pub struct MenuItemIndicatorRenderProps {
    pub attrs: MenuItemIndicatorAttributes,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Option<Element>,
}

#[derive(Props, Clone, PartialEq)]
pub struct MenuItemIndicatorProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MenuItemIndicatorRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Option<Element>,
}

/// Indicator slot rendered inside `CheckboxItem` or `RadioItem` when checked/indeterminate or `force_mount: true` (`M-10`).
#[component]
pub fn MenuItemIndicator(props: MenuItemIndicatorProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let ctx = try_use_context::<MenuIndicatorContext>().unwrap_or(MenuIndicatorContext {
        checked: CheckedState::Unchecked,
        disabled: false,
    });
    let id = use_stable_id(props.id.map(Into::into)).into_owned();

    let attrs = MenuItemIndicatorAttributes::from_parts(
        id,
        ctx.checked,
        ctx.disabled,
        props.force_mount,
        root.variant,
    );

    if !attrs.visible {
        return rsx! {};
    }

    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::ItemIndicator.static_data_attr_name(root.variant),
    );

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(MenuItemIndicatorRenderProps {
            attrs,
            class: props.class,
            style: props.style,
            attributes: merged_attrs,
            children: props.children,
        });
    }

    rsx! {
        span {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            "data-state": "{attrs.data_state.as_str()}",
            "data-disabled": attrs.data_disabled,
            ..merged_attrs,
            if let Some(children) = props.children {
                {children}
            }
        }
    }
}

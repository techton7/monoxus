use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    shared::use_stable_id,
};
use crate::menu::{
    CheckedState, MenuIndicatorContext, MenuItemIndicatorAttributes, MenuPart, MenuRootContext,
};

#[derive(Props, Clone, PartialEq)]
pub struct MenuItemIndicatorProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Option<Element>,
}

/// Indicator slot rendered inside `CheckboxItem` or `RadioItem` when checked/indeterminate or `force_mount: true` (`M-10`).
#[component]
pub fn MenuItemIndicator(props: MenuItemIndicatorProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let (checked, disabled) = try_use_context::<MenuIndicatorContext>()
        .map(|ctx| (*ctx.checked.read(), *ctx.disabled.read()))
        .unwrap_or((CheckedState::Unchecked, false));
    let id = use_stable_id(props.id.map(Into::into)).into_owned();

    let attrs = MenuItemIndicatorAttributes::from_parts(
        id,
        checked,
        disabled,
        props.force_mount,
        root.variant,
    );

    if !attrs.visible {
        return rsx! {};
    }

    let part_attr = MenuPart::ItemIndicator.static_data_attr_name(root.variant);
    let data_state_str = attrs.data_state.as_str();

    let mut base = Vec::with_capacity(4);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("data-state", data_state_str, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    base.push(Attribute::new(part_attr, "", None, false));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        span {
            ..merged,
            if let Some(children) = props.children {
                {children}
            }
        }
    }
}

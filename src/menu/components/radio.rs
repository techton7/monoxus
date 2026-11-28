use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::kernel::ChangeEvent;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    shared::use_stable_id,
    state::{ControllableConfig, sync_signal, use_controllable_signal_config},
};
use crate::menu::{
    CheckedState, MenuGroupAttributes, MenuIndicatorContext, MenuItemKind, MenuPart,
    MenuRadioGroupContext, MenuRadioItemAttributes, MenuRootContext, MenuSelectEvent,
    MenuSurfaceContext,
};

use super::item::{build_leaf_item_handlers, push_leaf_item_handlers, use_menu_item_registration};

#[derive(Props, Clone, PartialEq)]
pub struct MenuRadioGroupProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<String>>>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Exclusive single-selection radio group inside a menu (`M-9`).
#[component]
pub fn MenuRadioGroup(props: MenuRadioGroupProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let id = use_stable_id(props.id.map(Into::into)).into_owned();
    let has_explicit_default = props.default_value.is_some();
    let value_state = use_controllable_signal_config(
        ControllableConfig::new(props.value, props.default_value.unwrap_or_default())
            .with_explicit_default(has_explicit_default)
            .with_on_change(props.on_value_change)
            .with_on_change_details(props.on_value_change_details)
            .with_component("MenuRadioGroup")
            .with_prop("value"),
    );

    use_context_provider(|| MenuRadioGroupContext {
        value: value_state.signal(),
        on_select_value: Rc::new(move |next_val: &str| {
            value_state.set(next_val.to_owned());
        }),
    });

    let attrs = MenuGroupAttributes::new(id, props.aria_labelledby, root.variant, MenuPart::RadioGroup);
    let part_attr = MenuPart::RadioGroup.static_data_attr_name(root.variant);

    let mut base = Vec::with_capacity(4);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    push_attr_opt(&mut base, "aria-labelledby", attrs.aria_labelledby);
    base.push(Attribute::new(part_attr, "", None, false));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenuRadioItemProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(into)]
    pub value: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub text_value: Option<String>,
    #[props(default = true)]
    pub close_on_select: bool,
    #[props(default)]
    pub on_select: Option<EventHandler<MenuSelectEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Exclusive selection radio item inside [`MenuRadioGroup`] (`role="menuitemradio"`, `M-9`).
#[component]
pub fn MenuRadioItem(props: MenuRadioItemProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let surface_ctx = use_context::<MenuSurfaceContext>();
    let surface = surface_ctx.surface;
    let radio_group = use_context::<MenuRadioGroupContext>();
    let id = use_stable_id(props.id.map(Into::into)).into_owned();

    let checked = *radio_group.value.read() == props.value;

    use_menu_item_registration(
        &surface,
        &id,
        props.text_value.clone(),
        &props.children,
        props.disabled,
        MenuItemKind::RadioItem,
    );

    let checked_state = CheckedState::from(checked);
    let mut indicator_checked = use_signal(|| checked_state);
    let mut indicator_disabled = use_signal(|| props.disabled);
    sync_signal(&mut indicator_checked, checked_state);
    sync_signal(&mut indicator_disabled, props.disabled);
    use_context_provider(|| MenuIndicatorContext {
        checked: indicator_checked,
        disabled: indicator_disabled,
    });

    let highlighted =
        !props.disabled && surface.current_highlighted_id().as_deref() == Some(id.as_str());
    let attrs = MenuRadioItemAttributes::from_parts(
        id.clone(),
        checked,
        props.disabled,
        highlighted,
        root.variant,
    );
    let part_attr = MenuPart::RadioItem.static_data_attr_name(root.variant);
    let data_state_str = attrs.data_state.as_str();

    let on_select = props.on_select;
    let item_value = props.value.clone();
    let rg_select = radio_group.on_select_value.clone();
    let handlers = build_leaf_item_handlers(
        id,
        props.disabled,
        props.close_on_select,
        root,
        surface,
        Rc::new(move |select_evt| {
            if let Some(cb) = on_select.as_ref() {
                cb.call(select_evt.clone());
            }
            (rg_select)(&item_value);
        }),
    );

    let mut base = Vec::with_capacity(17);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("tabindex", attrs.tabindex, None, false));
    base.push(Attribute::new(
        "aria-checked",
        attrs.aria_checked,
        None,
        false,
    ));
    push_attr_opt(&mut base, "aria-disabled", attrs.aria_disabled);
    base.push(Attribute::new("data-state", data_state_str, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-highlighted", attrs.data_highlighted);
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation,
        None,
        false,
    ));
    base.push(Attribute::new("data-value", props.value, None, false));
    base.push(Attribute::new(part_attr, "", None, false));
    push_leaf_item_handlers(&mut base, handlers);

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

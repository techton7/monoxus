use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::kernel::ChangeEvent;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    shared::use_stable_id,
    state::{ControllableConfig, sync_signal, use_controllable_signal_config},
};
use crate::menu::{
    CheckedState, MenuCheckboxGroupContext, MenuCheckboxItemAttributes, MenuGroupAttributes,
    MenuIndicatorContext, MenuItemKind, MenuPart, MenuRootContext, MenuSelectEvent,
    MenuSurfaceContext, toggle_checkbox_group_value,
};

use super::item::{build_leaf_item_handlers, push_leaf_item_handlers, use_menu_item_registration};

#[derive(Props, Clone, PartialEq)]
pub struct MenuCheckboxGroupProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<Vec<String>>,
    #[props(default)]
    pub default_value: Option<Vec<String>>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Vec<String>>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<Vec<String>>>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Multi-value checkbox group inside a menu (`3B D-4`).
#[component]
pub fn MenuCheckboxGroup(props: MenuCheckboxGroupProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let id = use_stable_id(props.id.map(Into::into)).into_owned();
    let has_explicit_default = props.default_value.is_some();
    let value_state = use_controllable_signal_config(
        ControllableConfig::new(props.value, props.default_value.unwrap_or_default())
            .with_explicit_default(has_explicit_default)
            .with_on_change(props.on_value_change)
            .with_on_change_details(props.on_value_change_details)
            .with_component("MenuCheckboxGroup")
            .with_prop("value"),
    );
    let value_sig = value_state.signal();
    let mut disabled_sig = use_signal(|| props.disabled);
    sync_signal(&mut disabled_sig, props.disabled);

    let toggle_disabled = disabled_sig;
    use_context_provider(|| MenuCheckboxGroupContext {
        value: value_sig,
        disabled: disabled_sig,
        on_toggle: Rc::new(move |item_val: &str| {
            if toggle_disabled.try_read().map(|g| *g).unwrap_or(false) {
                return;
            }
            let current = value_state.try_peek_cloned().unwrap_or_default();
            let next = toggle_checkbox_group_value(&current, item_val);
            value_state.set(next);
        }),
    });

    let attrs = MenuGroupAttributes::new(
        id,
        props.aria_labelledby,
        root.variant,
        MenuPart::CheckboxGroup,
    );
    let part_attr = MenuPart::CheckboxGroup.static_data_attr_name(root.variant);

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
pub struct MenuCheckboxItemProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    /// Optional value key when participating in an enclosing `CheckboxGroup` (`3B D-4`).
    #[props(default, into)]
    pub value: Option<String>,
    /// Standalone controlled `CheckedState` (`Unchecked | Checked | Indeterminate`).
    #[props(default)]
    pub checked: Option<CheckedState>,
    #[props(default = CheckedState::Unchecked)]
    pub default_checked: CheckedState,
    #[props(default)]
    pub on_checked_change: Option<EventHandler<CheckedState>>,
    #[props(default)]
    pub on_checked_change_details: Option<EventHandler<ChangeEvent<CheckedState>>>,
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

/// Checkbox menu item supporting standalone `CheckedState` (with `Indeterminate`) or `CheckboxGroup` multi-select (`M-8`, `3B D-4`).
#[component]
pub fn MenuCheckboxItem(props: MenuCheckboxItemProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let surface_ctx = use_context::<MenuSurfaceContext>();
    let surface = surface_ctx.surface;
    let group_ctx = try_use_context::<MenuCheckboxGroupContext>();
    let id = use_stable_id(props.id.map(Into::into)).into_owned();

    let is_group_member = group_ctx.is_some() && props.value.is_some();
    let local_checked = use_controllable_signal_config(
        ControllableConfig::new(props.checked, props.default_checked)
            .with_explicit_default(props.default_checked != CheckedState::Unchecked)
            .with_on_change(props.on_checked_change)
            .with_on_change_details(props.on_checked_change_details)
            .with_component("MenuCheckboxItem")
            .with_prop("checked"),
    );

    let current_checked = if is_group_member {
        let group = group_ctx.as_ref().unwrap();
        let val = props.value.as_deref().unwrap_or_default();
        if group.value.read().iter().any(|entry| entry == val) {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        }
    } else {
        local_checked.get()
    };

    let effective_disabled = props.disabled
        || group_ctx
            .as_ref()
            .is_some_and(|g| *g.disabled.read());

    use_menu_item_registration(
        &surface,
        &id,
        props.text_value.clone(),
        &props.children,
        effective_disabled,
        MenuItemKind::CheckboxItem,
    );

    let mut indicator_checked = use_signal(|| current_checked);
    let mut indicator_disabled = use_signal(|| effective_disabled);
    sync_signal(&mut indicator_checked, current_checked);
    sync_signal(&mut indicator_disabled, effective_disabled);
    use_context_provider(|| MenuIndicatorContext {
        checked: indicator_checked,
        disabled: indicator_disabled,
    });

    let highlighted =
        !effective_disabled && surface.current_highlighted_id().as_deref() == Some(id.as_str());
    let attrs = MenuCheckboxItemAttributes::from_parts(
        id.clone(),
        current_checked,
        effective_disabled,
        highlighted,
        root.variant,
    );
    let part_attr = MenuPart::CheckboxItem.static_data_attr_name(root.variant);
    let data_state_str = attrs.data_state.as_str();

    let on_select = props.on_select;
    let group_for_activate = group_ctx.clone();
    let item_value = props.value.clone();
    let handlers = build_leaf_item_handlers(
        id,
        effective_disabled,
        props.close_on_select,
        root,
        surface,
        Rc::new(move |select_evt| {
            if let Some(cb) = on_select.as_ref() {
                cb.call(select_evt.clone());
            }
            if is_group_member {
                if let (Some(g), Some(v)) = (group_for_activate.as_ref(), item_value.as_deref()) {
                    (g.on_toggle)(v);
                }
            } else {
                let next = local_checked.try_peek_cloned().unwrap_or_default().toggle();
                local_checked.set(next);
            }
        }),
    );

    let mut base = Vec::with_capacity(16);
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

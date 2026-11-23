use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;
use crate::menu::{
    CheckedState, MenuCheckboxGroupContext, MenuCheckboxItemAttributes, MenuGroupAttributes,
    MenuIndicatorContext, MenuItemKind, MenuPart, MenuRootContext, MenuSelectEvent,
    MenuSurfaceContext, merge_attributes_with_part, toggle_checkbox_group_value,
};

use super::item::{build_leaf_item_handlers, use_menu_item_registration};

#[derive(Props, Clone, PartialEq)]
pub struct MenuCheckboxGroupProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<Vec<String>>,
    #[props(default)]
    pub default_value: Option<Vec<String>>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Vec<String>>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Multi-value checkbox group inside a menu (`3B D-4`).
#[component]
pub fn MenuCheckboxGroup(props: MenuCheckboxGroupProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let id = use_stable_id(props.id.map(Into::into)).into_owned();
    let is_controlled = props.value.is_some();
    let value_sig = use_signal(|| {
        props
            .value
            .clone()
            .or_else(|| props.default_value.clone())
            .unwrap_or_default()
    });
    let disabled_sig = use_signal(|| props.disabled);

    if let Some(controlled) = props.value.as_ref() {
        let mut s = value_sig;
        if s.try_peek().is_ok_and(|cur| *cur != *controlled) {
            let _ = s.try_write().map(|mut w| *w = controlled.clone());
        }
    }
    {
        let mut ds = disabled_sig;
        if ds.try_peek().is_ok_and(|cur| *cur != props.disabled) {
            let _ = ds.try_write().map(|mut w| *w = props.disabled);
        }
    }

    let on_value_change = props.on_value_change;
    let toggle_sig = value_sig;
    let toggle_disabled = disabled_sig;
    use_context_provider(|| MenuCheckboxGroupContext {
        value: value_sig,
        disabled: disabled_sig,
        on_toggle: Rc::new(move |item_val: &str| {
            if toggle_disabled.try_read().map(|g| *g).unwrap_or(false) {
                return;
            }
            let current = toggle_sig.try_read().map(|g| g.clone()).unwrap_or_default();
            let next = toggle_checkbox_group_value(&current, item_val);
            if !is_controlled {
                let mut w = toggle_sig;
                let _ = w.try_write().map(|mut dst| *dst = next.clone());
            }
            if let Some(cb) = on_value_change.as_ref() {
                cb.call(next);
            }
        }),
    });

    let attrs = MenuGroupAttributes::new(
        id,
        props.aria_labelledby,
        root.variant,
        MenuPart::CheckboxGroup,
    );
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::CheckboxGroup.static_data_attr_name(root.variant),
    );

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: "{attrs.role}",
            "aria-labelledby": attrs.aria_labelledby.as_deref(),
            ..merged_attrs,
            {props.children}
        }
    }
}

/// Render props passed to `render_child` when `CheckboxItem` uses `as_child: true`.
#[derive(Clone, PartialEq)]
pub struct MenuCheckboxItemRenderProps {
    pub attrs: MenuCheckboxItemAttributes,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_pointer_up: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_pointer_move: EventHandler<PointerEvent>,
    pub on_pointer_leave: EventHandler<PointerEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct MenuCheckboxItemProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    /// Optional value key when participating in an enclosing `CheckboxGroup` (`3B D-4`).
    #[props(default)]
    pub value: Option<String>,
    /// Standalone controlled `CheckedState` (`Unchecked | Checked | Indeterminate`).
    #[props(default)]
    pub checked: Option<CheckedState>,
    #[props(default = CheckedState::Unchecked)]
    pub default_checked: CheckedState,
    #[props(default)]
    pub on_checked_change: Option<EventHandler<CheckedState>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub text_value: Option<String>,
    #[props(default = true)]
    pub close_on_select: bool,
    #[props(default)]
    pub on_select: Option<EventHandler<MenuSelectEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MenuCheckboxItemRenderProps, Element>>,
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
    let is_controlled = props.checked.is_some();
    let local_checked = use_signal(|| props.checked.unwrap_or(props.default_checked));
    if let Some(c) = props.checked {
        let mut s = local_checked;
        if s.try_peek().is_ok_and(|cur| *cur != c) {
            let _ = s.try_write().map(|mut w| *w = c);
        }
    }

    let current_checked = if is_group_member {
        let group = group_ctx.as_ref().unwrap();
        let val = props.value.as_deref().unwrap_or_default();
        if group.value.read().iter().any(|entry| entry == val) {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        }
    } else {
        *local_checked.read()
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

    use_context_provider(|| MenuIndicatorContext {
        checked: current_checked,
        disabled: effective_disabled,
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
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::CheckboxItem.static_data_attr_name(root.variant),
    );

    let on_checked_change = props.on_checked_change;
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
                let next = local_checked.try_read().map(|g| *g).unwrap_or_default().toggle();
                if !is_controlled {
                    let mut w = local_checked;
                    let _ = w.try_write().map(|mut dst| *dst = next);
                }
                if let Some(cb) = on_checked_change.as_ref() {
                    cb.call(next);
                }
            }
        }),
    );

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(MenuCheckboxItemRenderProps {
            attrs,
            class: props.class,
            style: props.style,
            on_pointer_down: handlers.on_pointer_down,
            on_pointer_up: handlers.on_pointer_up,
            on_click: handlers.on_click,
            on_pointer_move: handlers.on_pointer_move,
            on_pointer_leave: handlers.on_pointer_leave,
            on_key_down: handlers.on_key_down,
            attributes: merged_attrs,
            children: props.children,
        });
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: "{attrs.role}",
            tabindex: "{attrs.tabindex}",
            "aria-checked": "{attrs.aria_checked}",
            "aria-disabled": attrs.aria_disabled,
            "data-state": "{attrs.data_state.as_str()}",
            "data-disabled": attrs.data_disabled,
            "data-highlighted": attrs.data_highlighted,
            "data-orientation": "{attrs.data_orientation}",
            onpointerdown: move |evt| handlers.on_pointer_down.call(evt),
            onpointerup: move |evt| handlers.on_pointer_up.call(evt),
            onclick: move |evt| handlers.on_click.call(evt),
            onpointermove: move |evt| handlers.on_pointer_move.call(evt),
            onpointerleave: move |evt| handlers.on_pointer_leave.call(evt),
            onkeydown: move |evt| handlers.on_key_down.call(evt),
            ..merged_attrs,
            {props.children}
        }
    }
}

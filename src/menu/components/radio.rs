use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{shared::use_stable_id, state::sync_signal};
use crate::menu::{
    CheckedState, MenuGroupAttributes, MenuIndicatorContext, MenuItemKind, MenuPart,
    MenuRadioGroupContext, MenuRadioItemAttributes, MenuRootContext, MenuSelectEvent,
    MenuSurfaceContext, merge_attributes_with_part,
};

use super::item::{build_leaf_item_handlers, use_menu_item_registration};

#[derive(Props, Clone, PartialEq)]
pub struct MenuRadioGroupProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Exclusive single-selection radio group inside a menu (`M-9`).
#[component]
pub fn MenuRadioGroup(props: MenuRadioGroupProps) -> Element {
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

    if let Some(controlled) = props.value.as_ref() {
        let mut s = value_sig;
        if s.try_peek().is_ok_and(|cur| *cur != *controlled) {
            let _ = s.try_write().map(|mut w| *w = controlled.clone());
        }
    }

    let mut is_controlled_slot = use_hook(|| CopyValue::new(is_controlled));
    is_controlled_slot.set(is_controlled);
    let mut on_value_change_slot = use_hook(|| CopyValue::new(props.on_value_change));
    on_value_change_slot.set(props.on_value_change);
    let select_sig = value_sig;
    use_context_provider(|| MenuRadioGroupContext {
        value: value_sig,
        on_select_value: Rc::new(move |next_val: &str| {
            let current = select_sig.try_read().map(|g| g.clone()).unwrap_or_default();
            if current == next_val {
                return;
            }
            let controlled = is_controlled_slot.try_read().map(|g| *g).unwrap_or(false);
            if !controlled {
                let mut w = select_sig;
                let _ = w.try_write().map(|mut dst| *dst = next_val.to_owned());
            }
            if let Some(cb) = on_value_change_slot.try_read().ok().and_then(|s| *s) {
                cb.call(next_val.to_owned());
            }
        }),
    });

    let attrs = MenuGroupAttributes::new(id, props.aria_labelledby, root.variant, MenuPart::RadioGroup);
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::RadioGroup.static_data_attr_name(root.variant),
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

/// Render props passed to `render_child` when `RadioItem` uses `as_child: true`.
#[derive(Clone, PartialEq)]
pub struct MenuRadioItemRenderProps {
    pub attrs: MenuRadioItemAttributes,
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
pub struct MenuRadioItemProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub value: String,
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
    pub render_child: Option<Callback<MenuRadioItemRenderProps, Element>>,
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
    let attrs =
        MenuRadioItemAttributes::from_parts(id.clone(), checked, props.disabled, highlighted, root.variant);
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::RadioItem.static_data_attr_name(root.variant),
    );

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

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(MenuRadioItemRenderProps {
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
            "data-value": "{props.value}",
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

use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    browser::is_element_focused_by_id,
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    keyboard::is_ime_composing_key,
    shared::use_stable_id,
};
use crate::menu::{
    MENU_POINTER_SCROLL_GUARD_PX, MenuItemAttributes, MenuItemEntry, MenuItemKind, MenuKey,
    MenuPart, MenuRootContext, MenuSelectEvent, MenuSurfaceContext, MenuSurfaceRuntime,
    PressedItemSnapshot, extract_vnode_text, translate_menu_key,
};

/// Shared DOM registration hook keeping `MenuSurfaceRuntime.items` synchronized.
pub fn use_menu_item_registration(
    surface: &MenuSurfaceRuntime,
    id: &str,
    text_value: Option<String>,
    children: &Element,
    disabled: bool,
    kind: MenuItemKind,
) {
    let resolved_text = text_value.unwrap_or_else(|| extract_vnode_text(children));
    let current_order = surface
        .items
        .try_peek()
        .ok()
        .and_then(|items| items.iter().find(|i| i.id == id).map(|i| i.dom_order))
        .unwrap_or(usize::MAX);
    surface.register_item(MenuItemEntry::new(
        id.to_owned(),
        resolved_text,
        disabled,
        kind,
        current_order,
    ));

    let cleanup_surface = surface.clone();
    let cleanup_id = id.to_owned();
    dioxus::core::use_drop(move || {
        cleanup_surface.unregister_item(&cleanup_id);
    });
}

/// Returns true if an interactive descendant inside `item_id` currently owns focus (`#3232`).
pub fn is_descendant_owning_focus(item_id: &str) -> bool {
    let Some(doc) = oxidase::dom::Document::current() else {
        return false;
    };
    doc.is_focus_within(item_id) && !is_element_focused_by_id(item_id)
}

/// Shared pointer/keyboard callbacks for leaf menu items (`Item`, `CheckboxItem`, `RadioItem`).
#[derive(Clone)]
pub struct LeafItemHandlers {
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_pointer_up: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_pointer_move: EventHandler<PointerEvent>,
    pub on_pointer_leave: EventHandler<PointerEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
}

pub(crate) fn push_leaf_item_handlers(attrs: &mut Vec<Attribute>, handlers: LeafItemHandlers) {
    push_event_handler::<PointerData>(attrs, "onpointerdown", Some(handlers.on_pointer_down));
    push_event_handler::<PointerData>(attrs, "onpointerup", Some(handlers.on_pointer_up));
    push_event_handler::<MouseData>(attrs, "onclick", Some(handlers.on_click));
    push_event_handler::<PointerData>(attrs, "onpointermove", Some(handlers.on_pointer_move));
    push_event_handler::<PointerData>(attrs, "onpointerleave", Some(handlers.on_pointer_leave));
    push_event_handler::<KeyboardData>(attrs, "onkeydown", Some(handlers.on_key_down));
}

pub fn build_leaf_item_handlers(
    id: String,
    disabled: bool,
    close_on_select: bool,
    root: MenuRootContext,
    surface: MenuSurfaceRuntime,
    on_activate: Rc<dyn Fn(MenuSelectEvent)>,
) -> LeafItemHandlers {
    let activate = Rc::new(move || {
        if disabled {
            return;
        }
        let select_evt = MenuSelectEvent::empty();
        on_activate(select_evt.clone());
        if close_on_select && !select_evt.is_default_prevented() {
            if let Some(bridge) = root.menubar_bridge.as_ref() {
                (bridge.close_menubar)();
            } else {
                root.close_menu_tree(true);
            }
        }
    });

    let pd_id = id.clone();
    let pd_surface = surface.clone();
    let on_pointer_down = EventHandler::new(move |evt: PointerEvent| {
        if disabled {
            return;
        }
        let coords = evt.data().client_coordinates();
        let mut pressed = pd_surface.pressed_item;
        let _ = pressed
            .try_write()
            .map(|mut w| *w = Some(PressedItemSnapshot::new(pd_id.clone(), coords.x, coords.y)));
    });

    let pu_id = id.clone();
    let pu_surface = surface.clone();
    let pu_activate = activate.clone();
    let on_pointer_up = EventHandler::new(move |evt: PointerEvent| {
        if disabled {
            return;
        }
        let coords = evt.data().client_coordinates();
        let valid_release = {
            let mut pressed_sig = pu_surface.pressed_item;
            let snap = pressed_sig.try_write().ok().and_then(|mut w| w.take());
            snap.is_some_and(|s| {
                s.item_id == pu_id
                    && !s.moved_beyond(coords.x, coords.y, MENU_POINTER_SCROLL_GUARD_PX)
            })
        };
        if valid_release {
            let mut consumed = pu_surface.pointer_up_consumed_click;
            let _ = consumed.try_write().map(|mut w| *w = Some(pu_id.clone()));
            pu_activate();
        }
    });

    let clk_id = id.clone();
    let clk_surface = surface.clone();
    let clk_activate = activate.clone();
    let on_click = EventHandler::new(move |_evt: MouseEvent| {
        if disabled {
            return;
        }
        let already_consumed = {
            let mut consumed = clk_surface.pointer_up_consumed_click;
            let val = consumed.try_peek().ok().and_then(|g| g.clone());
            if val.as_deref() == Some(clk_id.as_str()) {
                let _ = consumed.try_write().map(|mut w| *w = None);
                true
            } else {
                false
            }
        };
        if !already_consumed {
            clk_activate();
        }
    });

    let pm_id = id.clone();
    let pm_surface = surface.clone();
    let on_pointer_move = EventHandler::new(move |evt: PointerEvent| {
        if disabled || evt.pointer_type() == "touch" {
            return;
        }
        let coords = evt.data().client_coordinates();
        pm_surface.record_pointer(coords.x, coords.y);
        if pm_surface.is_submenu_grace_blocking(Some(&pm_id)) {
            return;
        }
        pm_surface.close_other_child_submenus(None);
        if pm_surface.current_highlighted_id().as_deref() != Some(pm_id.as_str()) {
            pm_surface.highlight_item(Some(pm_id.clone()), true, false);
        }
    });

    let pl_id = id.clone();
    let pl_surface = surface.clone();
    let on_pointer_leave = EventHandler::new(move |evt: PointerEvent| {
        if disabled || evt.pointer_type() == "touch" {
            return;
        }
        if pl_surface.current_highlighted_id().as_deref() == Some(pl_id.as_str()) {
            pl_surface.highlight_item(None, true, false);
        }
    });

    let kd_id = id;
    let kd_surface = surface;
    let kd_activate = activate;
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if disabled || is_ime_composing_key(&evt) || is_descendant_owning_focus(&kd_id) {
            return;
        }
        let Some(key) = translate_menu_key(&evt.key()) else {
            return;
        };
        let is_select = matches!(key, MenuKey::Enter)
            || (matches!(key, MenuKey::Space) && !kd_surface.is_typeahead_active());
        if is_select {
            evt.prevent_default();
            evt.stop_propagation();
            kd_activate();
        }
    });

    LeafItemHandlers {
        on_pointer_down,
        on_pointer_up,
        on_click,
        on_pointer_move,
        on_pointer_leave,
        on_key_down,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenuItemProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
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

/// Actionable item inside a menu (`role="menuitem"`, scroll-under-pointer guard, typeahead registration, `on_select`).
#[component]
pub fn MenuItem(props: MenuItemProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let surface_ctx = use_context::<MenuSurfaceContext>();
    let surface = surface_ctx.surface;
    let id = use_stable_id(props.id.map(Into::into)).into_owned();

    use_menu_item_registration(
        &surface,
        &id,
        props.text_value.clone(),
        &props.children,
        props.disabled,
        MenuItemKind::Item,
    );

    let highlighted =
        !props.disabled && surface.current_highlighted_id().as_deref() == Some(id.as_str());
    let attrs = MenuItemAttributes::from_parts(id.clone(), props.disabled, highlighted, root.variant);
    let part_attr = MenuPart::Item.static_data_attr_name(root.variant);

    let on_select = props.on_select;
    let handlers = build_leaf_item_handlers(
        id,
        props.disabled,
        props.close_on_select,
        root,
        surface,
        Rc::new(move |evt| {
            if let Some(cb) = on_select.as_ref() {
                cb.call(evt);
            }
        }),
    );

    let mut base = Vec::with_capacity(14);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("tabindex", attrs.tabindex, None, false));
    push_attr_opt(&mut base, "aria-disabled", attrs.aria_disabled);
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

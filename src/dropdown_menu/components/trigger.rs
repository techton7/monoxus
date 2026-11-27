use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    keyboard::is_ime_composing_key,
    state::sync_signal,
};
use crate::menu::{
    MenuKey, MenuOpenFocusTarget, MenuPart, MenuRootContext, MenuTriggerAttributes, MenuVariant,
    translate_menu_key,
};

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuTriggerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Interactive button toggling an enclosing `DropdownMenu` and owning return focus (`DM-1`, `DM-2`, `DM-5`).
#[component]
pub fn DropdownMenuTrigger(props: DropdownMenuTriggerProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let id = props
        .id
        .unwrap_or_else(|| root.relationships.trigger_id().to_owned());

    {
        let mut tid = root.trigger_id;
        sync_signal(&mut tid, id.clone());
    }

    let is_open = root.is_open();
    let content_id = root.root_surface.current_content_id();
    let attrs = MenuTriggerAttributes::from_parts(
        id,
        &content_id,
        is_open,
        props.disabled,
        MenuVariant::DropdownMenu,
    );

    let toggled_on_pointer_down = use_signal(|| false);

    let pd_root = root.clone();
    let mut pd_flag = toggled_on_pointer_down;
    let on_pointer_down = EventHandler::new(move |evt: PointerEvent| {
        if props.disabled {
            return;
        }
        if evt.pointer_type() == "mouse"
            && evt
                .trigger_button()
                .is_some_and(|b| b != MouseButton::Primary)
        {
            return;
        }
        pd_flag.set(true);
        let mut tgt = pd_root.root_surface.open_focus_target;
        let _ = tgt
            .try_write()
            .map(|mut w| *w = MenuOpenFocusTarget::ContentContainer);
        pd_root.request_open_change(!pd_root.is_open());
    });

    let clk_root = root.clone();
    let mut clk_flag = toggled_on_pointer_down;
    let on_click = EventHandler::new(move |_evt: MouseEvent| {
        if props.disabled {
            return;
        }
        if *clk_flag.peek() {
            clk_flag.set(false);
            return;
        }
        let next = !clk_root.is_open();
        let target = if next {
            MenuOpenFocusTarget::FirstItem
        } else {
            MenuOpenFocusTarget::ContentContainer
        };
        let mut tgt = clk_root.root_surface.open_focus_target;
        let _ = tgt.try_write().map(|mut w| *w = target);
        clk_root.request_open_change(next);
    });

    let kd_root = root;
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if props.disabled || is_ime_composing_key(&evt) {
            return;
        }
        let Some(key) = translate_menu_key(&evt.key()) else {
            return;
        };
        if matches!(key, MenuKey::Enter | MenuKey::Space | MenuKey::ArrowDown) {
            evt.prevent_default();
            evt.stop_propagation();
            let mut tgt = kd_root.root_surface.open_focus_target;
            let _ = tgt
                .try_write()
                .map(|mut w| *w = MenuOpenFocusTarget::FirstItem);
            kd_root.request_open_change(true);
        }
    });

    let data_state_str = attrs.data_state.as_str();
    let part_attr = MenuPart::Trigger.static_data_attr_name(MenuVariant::DropdownMenu);

    let mut base = Vec::with_capacity(14);
    base.push(Attribute::new("type", "button", None, false));
    base.push(Attribute::new("id", attrs.id, None, false));
    if attrs.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new(
        "aria-haspopup",
        attrs.aria_haspopup,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-expanded",
        if attrs.aria_expanded { "true" } else { "false" },
        None,
        false,
    ));
    push_attr_opt(&mut base, "aria-controls", attrs.aria_controls);
    base.push(Attribute::new("data-state", data_state_str, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    base.push(Attribute::new(part_attr, "", None, false));
    push_event_handler::<PointerData>(&mut base, "onpointerdown", Some(on_pointer_down));
    push_event_handler::<MouseData>(&mut base, "onclick", Some(on_click));
    push_event_handler::<KeyboardData>(&mut base, "onkeydown", Some(on_key_down));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler::<MouseData>(&mut user, "onclick", props.on_click);
    push_event_handler::<KeyboardData>(&mut user, "onkeydown", props.on_key_down);
    push_event_handler::<MountedData>(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {props.children}
        }
    }
}

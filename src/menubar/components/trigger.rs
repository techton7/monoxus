use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    state::sync_signal,
};
use crate::menu::{MenuOpenFocusTarget, MenuPart, MenuVariant};
use crate::menubar::MenubarRuntimeContext;

use super::menu::MenubarSingleMenuContext;

#[derive(Props, Clone, PartialEq)]
pub struct MenubarTriggerProps {
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

/// Interactive button inside [`MenubarMenu`](super::menu::MenubarMenu) participating in horizontal roving focus and hover swap (`MB-2`, `MB-3`).
#[component]
pub fn MenubarTrigger(props: MenubarTriggerProps) -> Element {
    let menubar = use_context::<MenubarRuntimeContext>();
    let single = use_context::<MenubarSingleMenuContext>();
    let id = props
        .id
        .unwrap_or_else(|| single.trigger_id.read().clone());

    {
        let mut tid = single.trigger_id;
        sync_signal(&mut tid, id.clone());
        let mut dis = single.disabled;
        sync_signal(&mut dis, props.disabled);
    }

    let content_id = single.content_id.read().clone();
    let attrs = menubar
        .state
        .read()
        .trigger_attrs(&single.value, id, &content_id, props.disabled);

    let toggled_on_pointer_down = use_signal(|| false);

    let pd_mb = menubar.clone();
    let pd_val = single.value.clone();
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
        if pd_mb.is_menu_open(&pd_val) {
            pd_mb.close_menu(&pd_val);
        } else {
            pd_mb.open_menu(&pd_val, MenuOpenFocusTarget::ContentContainer);
        }
    });

    let clk_mb = menubar.clone();
    let clk_val = single.value.clone();
    let mut clk_flag = toggled_on_pointer_down;
    let on_click = EventHandler::new(move |_evt: MouseEvent| {
        if props.disabled {
            return;
        }
        if *clk_flag.peek() {
            clk_flag.set(false);
            return;
        }
        if clk_mb.is_menu_open(&clk_val) {
            clk_mb.close_menu(&clk_val);
        } else {
            clk_mb.open_menu(&clk_val, MenuOpenFocusTarget::FirstItem);
        }
    });

    let pe_mb = menubar.clone();
    let pe_val = single.value.clone();
    let on_pointer_enter = EventHandler::new(move |evt: PointerEvent| {
        if props.disabled || evt.pointer_type() == "touch" {
            return;
        }
        pe_mb.on_trigger_pointer_enter(&pe_val);
    });

    let f_mb = menubar.clone();
    let f_val = single.value.clone();
    let on_focus = EventHandler::new(move |_evt: FocusEvent| {
        if props.disabled {
            return;
        }
        let mut st = f_mb.state;
        let _ = st.try_write().map(|mut w| {
            w.set_current_tabstop_value(Some(f_val.clone()));
            w.set_highlighted_value(Some(f_val.clone()));
        });
    });

    let b_mb = menubar.clone();
    let b_val = single.value.clone();
    let on_blur = EventHandler::new(move |_evt: FocusEvent| {
        let mut st = b_mb.state;
        let _ = st.try_write().map(|mut w| {
            if w.highlighted_value() == Some(b_val.as_str()) {
                w.set_highlighted_value(None);
            }
        });
    });

    let kd_mb = menubar;
    let kd_val = single.value;
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if props.disabled {
            return;
        }
        kd_mb.on_trigger_key(&kd_val, &evt);
    });

    let data_state_str = attrs.data_state.as_str();
    let part_attr = MenuPart::Trigger.static_data_attr_name(MenuVariant::Menubar);

    let mut base = Vec::with_capacity(22);
    base.push(Attribute::new("type", "button", None, false));
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("tabindex", attrs.tabindex, None, false));
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
    push_attr_opt(&mut base, "aria-disabled", attrs.aria_disabled);
    base.push(Attribute::new("data-state", data_state_str, None, false));
    push_attr_opt(&mut base, "data-highlighted", attrs.data_highlighted);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    base.push(Attribute::new(part_attr, "", None, false));
    push_event_handler::<PointerData>(&mut base, "onpointerdown", Some(on_pointer_down));
    push_event_handler::<MouseData>(&mut base, "onclick", Some(on_click));
    push_event_handler::<PointerData>(&mut base, "onpointerenter", Some(on_pointer_enter));
    push_event_handler::<PointerData>(&mut base, "onpointermove", Some(on_pointer_enter));
    push_event_handler::<FocusData>(&mut base, "onfocus", Some(on_focus));
    push_event_handler::<FocusData>(&mut base, "onblur", Some(on_blur));
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

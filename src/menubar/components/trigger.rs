use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::menu::{MenuOpenFocusTarget, MenuPart, MenuVariant, merge_attributes_with_part};
use crate::menubar::{MenubarRuntimeContext, MenubarTriggerAttributes};

use super::menu::MenubarSingleMenuContext;

/// Render props passed to `render_child` when `MenubarTrigger` uses `as_child: true`.
#[derive(Clone, PartialEq)]
pub struct MenubarTriggerRenderProps {
    pub attrs: MenubarTriggerAttributes,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_pointer_enter: EventHandler<PointerEvent>,
    pub on_focus: EventHandler<FocusEvent>,
    pub on_blur: EventHandler<FocusEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MenubarTriggerRenderProps, Element>>,
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
        if tid.try_peek().is_ok_and(|g| *g != id) {
            let _ = tid.try_write().map(|mut w| *w = id.clone());
        }
        let mut dis = single.disabled;
        if dis.try_peek().is_ok_and(|g| *g != props.disabled) {
            let _ = dis.try_write().map(|mut w| *w = props.disabled);
        }
    }

    let content_id = single.content_id.read().clone();
    let attrs = menubar
        .state
        .read()
        .trigger_attrs(&single.value, id, &content_id, props.disabled);
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::Trigger.static_data_attr_name(MenuVariant::Menubar),
    );

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

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(MenubarTriggerRenderProps {
            attrs,
            class: props.class,
            style: props.style,
            on_pointer_down,
            on_click,
            on_pointer_enter,
            on_focus,
            on_blur,
            on_key_down,
            attributes: merged_attrs,
            children: props.children,
        });
    }

    rsx! {
        button {
            r#type: "button",
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: "{attrs.role}",
            tabindex: "{attrs.tabindex}",
            disabled: attrs.disabled,
            "aria-haspopup": "{attrs.aria_haspopup}",
            "aria-expanded": "{attrs.aria_expanded}",
            "aria-controls": attrs.aria_controls.as_deref(),
            "aria-disabled": attrs.aria_disabled,
            "data-state": "{attrs.data_state.as_str()}",
            "data-highlighted": attrs.data_highlighted,
            "data-disabled": attrs.data_disabled,
            onpointerdown: move |evt| on_pointer_down.call(evt),
            onclick: move |evt| on_click.call(evt),
            onpointerenter: move |evt| on_pointer_enter.call(evt),
            onpointermove: move |evt| on_pointer_enter.call(evt),
            onfocus: move |evt| on_focus.call(evt),
            onblur: move |evt| on_blur.call(evt),
            onkeydown: move |evt| on_key_down.call(evt),
            ..merged_attrs,
            {props.children}
        }
    }
}

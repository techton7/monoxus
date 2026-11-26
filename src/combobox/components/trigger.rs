use dioxus::html::input_data::MouseButton;
use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::combobox::{attrs::ComboboxTriggerAttrs, runtime::use_combobox_context};
use crate::foundation::{
    compose::merge_global_attributes, keyboard::is_ime_composing_key,
};

#[derive(Clone, PartialEq)]
pub struct ComboboxTriggerRenderProps {
    pub attrs: ComboboxTriggerAttrs,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_mouse_down: EventHandler<MouseEvent>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_pointer_up: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxTriggerRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn ComboboxTrigger(props: ComboboxTriggerProps) -> Element {
    let ctx = use_combobox_context();
    let id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().trigger_id().to_owned());
    ctx.runtime.set_trigger_id(id.clone());

    let attrs = ctx.runtime.trigger_attributes(id, props.disabled);
    let is_disabled = attrs.is_disabled();
    let toggled_on_pointer = use_signal(|| false);

    // Prevent mousedown focus transfer so `<input>` never blurs on Web or Blitz Native.
    let on_mouse_down = EventHandler::new(move |evt: MouseEvent| {
        if !is_disabled {
            evt.prevent_default();
        }
    });

    let pd_rt = ctx.runtime.clone();
    let mut pd_flag = toggled_on_pointer;
    let on_pointer_down = EventHandler::new(move |evt: PointerEvent| {
        if is_disabled {
            return;
        }
        if evt.pointer_type() == "touch" {
            return;
        }
        if evt.pointer_type() == "mouse"
            && evt
                .trigger_button()
                .is_some_and(|b| b != MouseButton::Primary)
        {
            return;
        }
        evt.prevent_default();
        pd_flag.set(true);
        pd_rt.focus_input();
        pd_rt.toggle_menu();
    });

    let pu_rt = ctx.runtime.clone();
    let mut pu_flag = toggled_on_pointer;
    let on_pointer_up = EventHandler::new(move |evt: PointerEvent| {
        if is_disabled {
            return;
        }
        if evt.pointer_type() == "touch" {
            pu_flag.set(true);
            pu_rt.focus_input();
            pu_rt.toggle_menu();
        }
    });

    let clk_rt = ctx.runtime.clone();
    let mut clk_flag = toggled_on_pointer;
    let on_click = EventHandler::new(move |_evt: MouseEvent| {
        if is_disabled {
            return;
        }
        if *clk_flag.peek() {
            clk_flag.set(false);
            return;
        }
        clk_rt.focus_input();
        clk_rt.toggle_menu();
    });

    let key_rt = ctx.runtime.clone();
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if is_disabled || is_ime_composing_key(&evt) {
            return;
        }
        match evt.key() {
            Key::Named(NamedKey::Enter) => {
                evt.prevent_default();
                key_rt.focus_input();
                key_rt.toggle_menu();
            }
            Key::Character(ref s) if s == " " => {
                evt.prevent_default();
                key_rt.focus_input();
                key_rt.toggle_menu();
            }
            _ => {}
        }
    });

    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            return render_child.call(ComboboxTriggerRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                on_mouse_down,
                on_pointer_down,
                on_pointer_up,
                on_click,
                on_key_down,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        button {
            id: "{attrs.id()}",
            r#type: "button",
            "aria-haspopup": "{attrs.aria_haspopup()}",
            "aria-expanded": "{attrs.aria_expanded()}",
            "aria-controls": attrs.aria_controls(),
            disabled: is_disabled,
            "data-combobox-trigger": "",
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": attrs.data_disabled(),
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            onmousedown: move |evt| on_mouse_down.call(evt),
            onpointerdown: move |evt| on_pointer_down.call(evt),
            onpointerup: move |evt| on_pointer_up.call(evt),
            onclick: move |evt| on_click.call(evt),
            onkeydown: move |evt| on_key_down.call(evt),
            ..merged.rest,
            {props.children}
        }
    }
}

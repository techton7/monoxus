use dioxus::html::input_data::MouseButton;
use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::combobox::runtime::use_combobox_context;
use crate::field::try_use_field_context;
use crate::foundation::{
    compose::{
        collect_user_attrs, extract_aria_invalid_and_describedby, merge_aria_describedby,
        merge_props, push_attr_opt, push_event_handler, resolve_invalid_and_describedby,
    },
    keyboard::is_ime_composing_key,
};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxTriggerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn ComboboxTrigger(props: ComboboxTriggerProps) -> Element {
    let ctx = use_combobox_context();
    let field_ctx = try_use_field_context();
    let id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().trigger_id().to_owned());
    ctx.runtime.set_trigger_id(id.clone());

    let extracted = extract_aria_invalid_and_describedby(props.attributes);
    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let root_describedby = ctx.runtime.root_aria_describedby();
    let combined_context_describedby = merge_aria_describedby([
        field_describedby.as_deref(),
        root_describedby.as_deref(),
    ]);
    let fallback_invalid = ctx
        .runtime
        .root_invalid_override()
        .or_else(|| field_ctx.map(|f| f.is_invalid()));
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        None,
        extracted.aria_invalid,
        fallback_invalid,
        false,
        None,
        extracted.aria_describedby.as_deref(),
        combined_context_describedby.as_deref(),
    );

    let attrs = ctx
        .runtime
        .trigger_attributes(id, props.disabled)
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby);
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

    let focus_in_rt = ctx.runtime.clone();
    let on_focus_in = EventHandler::new(move |evt: FocusEvent| {
        focus_in_rt.notify_composite_focus_in(evt);
    });
    let focus_out_rt = ctx.runtime.clone();
    let on_focus_out = EventHandler::new(move |evt: FocusEvent| {
        focus_out_rt.notify_composite_focus_out(evt);
    });

    let id_str = attrs.id().to_owned();
    let aria_haspopup = attrs.aria_haspopup();
    let aria_expanded = attrs.aria_expanded();
    let aria_controls = attrs.aria_controls().map(str::to_owned);
    let aria_invalid = attrs.aria_invalid();
    let aria_describedby = attrs.aria_describedby().map(str::to_owned);
    let aria_required = attrs.aria_required();
    let data_state = attrs.data_state_str();
    let data_disabled = attrs.data_disabled();
    let data_invalid = attrs.data_invalid();

    let mut base = Vec::with_capacity(20);
    base.push(Attribute::new("id", id_str, None, false));
    base.push(Attribute::new("type", "button", None, false));
    base.push(Attribute::new("aria-haspopup", aria_haspopup, None, false));
    base.push(Attribute::new("aria-expanded", aria_expanded, None, false));
    push_attr_opt(&mut base, "aria-controls", aria_controls);
    push_attr_opt(&mut base, "aria-invalid", aria_invalid);
    push_attr_opt(&mut base, "aria-describedby", aria_describedby);
    push_attr_opt(&mut base, "aria-required", aria_required);
    if is_disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("data-combobox-trigger", "", None, false));
    base.push(Attribute::new("data-state", data_state, None, false));
    push_attr_opt(&mut base, "data-disabled", data_disabled);
    push_attr_opt(&mut base, "data-invalid", data_invalid);
    push_event_handler(&mut base, "onmousedown", Some(on_mouse_down));
    push_event_handler(&mut base, "onpointerdown", Some(on_pointer_down));
    push_event_handler(&mut base, "onpointerup", Some(on_pointer_up));
    push_event_handler(&mut base, "onclick", Some(on_click));
    push_event_handler(&mut base, "onkeydown", Some(on_key_down));
    push_event_handler(&mut base, "onfocusin", Some(on_focus_in));
    push_event_handler(&mut base, "onfocusout", Some(on_focus_out));

    let user = collect_user_attrs(props.class, props.style, extracted.rest);
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

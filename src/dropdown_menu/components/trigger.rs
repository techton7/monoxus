use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::foundation::keyboard::is_ime_composing_key;
use crate::menu::{
    MenuKey, MenuOpenFocusTarget, MenuPart, MenuRootContext, MenuTriggerAttributes, MenuVariant,
    merge_attributes_with_part, translate_menu_key,
};

/// Render props passed to `render_child` when `DropdownMenuTrigger` uses `as_child: true`.
#[derive(Clone, PartialEq)]
pub struct DropdownMenuTriggerRenderProps {
    pub attrs: MenuTriggerAttributes,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuTriggerProps {
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
    pub render_child: Option<Callback<DropdownMenuTriggerRenderProps, Element>>,
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
        if tid.try_peek().is_ok_and(|g| *g != id) {
            let _ = tid.try_write().map(|mut w| *w = id.clone());
        }
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
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::Trigger.static_data_attr_name(MenuVariant::DropdownMenu),
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

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(DropdownMenuTriggerRenderProps {
            attrs,
            class: props.class,
            style: props.style,
            on_pointer_down,
            on_click,
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
            disabled: attrs.disabled,
            "aria-haspopup": "{attrs.aria_haspopup}",
            "aria-expanded": "{attrs.aria_expanded}",
            "aria-controls": attrs.aria_controls.as_deref(),
            "data-state": "{attrs.data_state.as_str()}",
            "data-disabled": attrs.data_disabled,
            onpointerdown: move |evt| on_pointer_down.call(evt),
            onclick: move |evt| on_click.call(evt),
            onkeydown: move |evt| on_key_down.call(evt),
            ..merged_attrs,
            {props.children}
        }
    }
}

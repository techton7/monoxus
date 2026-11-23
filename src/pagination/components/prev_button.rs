use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::roving_focus::{RovingItemMeta, use_roving_focus_item};

use super::super::{
    attrs::PaginationPrevButtonAttributes,
    relationships::PREV_ROVING_KEY,
    runtime::use_pagination_context,
};

#[derive(Clone)]
pub struct PaginationPrevButtonRenderChildProps {
    pub attrs: PaginationPrevButtonAttributes,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct PaginationPrevButtonProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub as_child: Option<bool>,
    #[props(default)]
    pub render_child: Option<Callback<PaginationPrevButtonRenderChildProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn PaginationPrevButton(props: PaginationPrevButtonProps) -> Element {
    let ctx = use_pagination_context();
    let runtime = ctx.runtime;

    let prev_attrs = runtime.prev_button_attributes(props.id.clone(), props.disabled);
    let effective_disabled = prev_attrs.is_disabled();

    use_roving_focus_item(
        runtime.roving,
        PREV_ROVING_KEY,
        RovingItemMeta::new(prev_attrs.id.clone(), !effective_disabled, false),
    );

    let as_child = props.as_child.unwrap_or(false);
    if as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PaginationPrevButton: as_child requires render_child; rendering the default button"
        );
    }
    if let Some(render_child) = props.render_child.as_ref().filter(|_| as_child) {
        return render_child.call(PaginationPrevButtonRenderChildProps {
            attrs: prev_attrs,
            children: props.children,
        });
    }

    let on_click = {
        let runtime = runtime.clone();
        let consumer_on_click = props.on_click;
        move |evt: MouseEvent| {
            if evt
                .trigger_button()
                .is_some_and(|b| b != dioxus::html::input_data::MouseButton::Primary)
            {
                return;
            }
            if effective_disabled {
                evt.prevent_default();
                return;
            }
            if let Some(ref handler) = consumer_on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() {
                runtime.prev_page();
            }
        }
    };

    let on_key_down = {
        let runtime = runtime.clone();
        let consumer_on_keydown = props.on_key_down;
        move |evt: KeyboardEvent| {
            let key = evt.key();
            if key == Key::Named(NamedKey::Enter) || key == Key::Character(String::from(" ")) {
                if effective_disabled {
                    evt.prevent_default();
                    return;
                }
                if let Some(ref handler) = consumer_on_keydown {
                    handler.call(evt.clone());
                }
                if evt.default_action_enabled() {
                    evt.prevent_default();
                    runtime.prev_page();
                }
                return;
            }

            if let Some(ref handler) = consumer_on_keydown {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() {
                runtime
                    .roving
                    .handle_keydown(Some(PREV_ROVING_KEY), &evt, None::<fn(&str)>);
            }
        }
    };

    let on_focus = {
        let runtime = runtime.clone();
        move |_| {
            runtime
                .roving
                .handle_item_focus(PREV_ROVING_KEY, !effective_disabled);
        }
    };

    rsx! {
        button {
            id: "{prev_attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            r#type: "{prev_attrs.r#type}",
            aria_label: props.aria_label.as_deref(),
            aria_disabled: prev_attrs.aria_disabled,
            disabled: prev_attrs.disabled,
            "data-disabled": prev_attrs.data_disabled,
            "data-monoxus-pagination-prev": "{prev_attrs.data_monoxus_pagination_prev}",
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

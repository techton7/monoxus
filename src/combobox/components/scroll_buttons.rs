use dioxus::prelude::*;

use crate::foundation::browser::{get_element_scroll_metrics, set_element_scroll_position};

use crate::combobox::{attrs::ComboboxScrollButtonAttrs, runtime::use_combobox_context};

const SCROLL_BUTTON_STEP_PX: f64 = 24.0;

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxScrollButtonProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn ComboboxScrollUpButton(props: ComboboxScrollButtonProps) -> Element {
    let ctx = use_combobox_context();
    let can_scroll = ctx.runtime.can_scroll_up();
    if !can_scroll && !props.force_mount {
        return rsx! {};
    }

    let attrs = ComboboxScrollButtonAttrs::new();
    let rt = ctx.runtime.clone();
    let step_up = move || {
        let vp_id = rt.current_viewport_id_untracked();
        if let Some(m) = get_element_scroll_metrics(&vp_id) {
            let next_y = (m.scroll_top - SCROLL_BUTTON_STEP_PX).max(0.0);
            set_element_scroll_position(&vp_id, m.scroll_left, next_y);
            rt.refresh_scroll_buttons();
        }
    };

    rsx! {
        div {
            "aria-hidden": "{attrs.aria_hidden()}",
            "data-combobox-scroll-up-button": "",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            onmousedown: move |evt| {
                evt.prevent_default();
                step_up();
            },
            ..props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn ComboboxScrollDownButton(props: ComboboxScrollButtonProps) -> Element {
    let ctx = use_combobox_context();
    let can_scroll = ctx.runtime.can_scroll_down();
    if !can_scroll && !props.force_mount {
        return rsx! {};
    }

    let attrs = ComboboxScrollButtonAttrs::new();
    let rt = ctx.runtime.clone();
    let step_down = move || {
        let vp_id = rt.current_viewport_id_untracked();
        if let Some(m) = get_element_scroll_metrics(&vp_id) {
            let max_y = (m.scroll_height - m.client_height).max(0.0);
            let next_y = (m.scroll_top + SCROLL_BUTTON_STEP_PX).min(max_y);
            set_element_scroll_position(&vp_id, m.scroll_left, next_y);
            rt.refresh_scroll_buttons();
        }
    };

    rsx! {
        div {
            "aria-hidden": "{attrs.aria_hidden()}",
            "data-combobox-scroll-down-button": "",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            onmousedown: move |evt| {
                evt.prevent_default();
                step_down();
            },
            ..props.attributes,
            {props.children}
        }
    }
}

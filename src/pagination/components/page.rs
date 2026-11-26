use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    keyboard::is_activation_key,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
};

use super::super::{
    attrs::PaginationPageAttributes,
    relationships::page_roving_key,
    runtime::use_pagination_context,
    types::PageData,
};

#[derive(Clone)]
pub struct PaginationPageRenderChildProps {
    pub attrs: PaginationPageAttributes,
    pub page: PageData,
    pub is_selected: bool,
    pub children: Option<Element>,
}

#[derive(Clone, PartialEq, Props)]
pub struct PaginationPageProps {
    #[props(default)]
    pub id: Option<String>,
    pub page: PageData,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<PaginationPageRenderChildProps, Element>>,
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
    pub children: Option<Element>,
}

#[component]
pub fn PaginationPage(props: PaginationPageProps) -> Element {
    let ctx = use_pagination_context();
    let runtime = ctx.runtime;

    let mut page_attrs =
        runtime.page_attributes(props.page.value, props.id.clone(), props.disabled);
    if let Some(ref custom_label) = props.aria_label {
        page_attrs.aria_label = custom_label.clone();
    }

    let is_selected = page_attrs.is_selected();
    let effective_disabled = page_attrs.is_disabled();
    let page_value = props.page.value;
    let roving_key = page_roving_key(page_value);

    use_roving_focus_item(
        runtime.roving,
        &roving_key,
        RovingItemMeta::new(page_attrs.id.clone(), !effective_disabled, is_selected),
    );

    if props.as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PaginationPage: as_child requires render_child; rendering the default button"
        );
    }
    if let Some(render_child) = props.render_child.as_ref().filter(|_| props.as_child) {
        return render_child.call(PaginationPageRenderChildProps {
            attrs: page_attrs,
            page: props.page,
            is_selected,
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
                runtime.set_page(page_value);
            }
        }
    };

    let on_key_down = {
        let runtime = runtime.clone();
        let roving_key = roving_key.clone();
        let consumer_on_keydown = props.on_key_down;
        move |evt: KeyboardEvent| {
            if is_activation_key(&evt.key()) {
                if effective_disabled {
                    evt.prevent_default();
                    return;
                }
                if let Some(ref handler) = consumer_on_keydown {
                    handler.call(evt.clone());
                }
                if evt.default_action_enabled() {
                    evt.prevent_default();
                    runtime.set_page(page_value);
                }
                return;
            }

            if let Some(ref handler) = consumer_on_keydown {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() {
                runtime
                    .roving
                    .handle_keydown(Some(&roving_key), &evt, None::<fn(&str)>);
            }
        }
    };

    let on_focus = {
        let runtime = runtime.clone();
        let roving_key = roving_key.clone();
        move |_| {
            runtime
                .roving
                .handle_item_focus(&roving_key, !effective_disabled);
        }
    };

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        button {
            id: "{page_attrs.id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            r#type: "{page_attrs.r#type}",
            aria_label: "{page_attrs.aria_label}",
            "aria-current": page_attrs.aria_current,
            aria_disabled: page_attrs.aria_disabled,
            disabled: page_attrs.disabled,
            "data-value": "{page_attrs.data_value}",
            "data-selected": page_attrs.data_selected,
            "data-disabled": page_attrs.data_disabled,
            "data-monoxus-pagination-page": "{page_attrs.data_monoxus_pagination_page}",
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            if let Some(children) = props.children {
                {children}
            } else {
                "{page_attrs.value}"
            }
        }
    }
}

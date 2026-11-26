use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    keyboard::is_activation_key,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
};

use super::super::{
    attrs::PaginationPrevButtonAttributes,
    relationships::{NEXT_ROVING_KEY, PREV_ROVING_KEY},
    runtime::use_pagination_context,
};
use super::next_button::PaginationNextButtonRenderChildProps;

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
    pub as_child: bool,
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

pub(crate) enum PaginationStepKind {
    Prev {
        render_child: Option<Callback<PaginationPrevButtonRenderChildProps, Element>>,
    },
    Next {
        render_child: Option<Callback<PaginationNextButtonRenderChildProps, Element>>,
    },
}

pub(crate) struct PaginationStepButtonSpec {
    pub id: Option<String>,
    pub disabled: bool,
    pub aria_label: Option<String>,
    pub as_child: bool,
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub kind: PaginationStepKind,
}

/// Plain-function surface unifying `PaginationPrevButton` and `PaginationNextButton` without an
/// internal `#[component]` wrapper (`Dioxus #5894`, `FM-API-4`).
pub(crate) fn render_pagination_step_button(spec: PaginationStepButtonSpec) -> Element {
    let ctx = use_pagination_context();
    let runtime = ctx.runtime;
    let is_prev = matches!(spec.kind, PaginationStepKind::Prev { .. });
    let roving_key = if is_prev { PREV_ROVING_KEY } else { NEXT_ROVING_KEY };

    let mut on_click_slot = use_hook(|| CopyValue::new(spec.on_click));
    on_click_slot.set(spec.on_click);
    let mut on_key_down_slot = use_hook(|| CopyValue::new(spec.on_key_down));
    on_key_down_slot.set(spec.on_key_down);
    let mut on_mounted_slot = use_hook(|| CopyValue::new(spec.on_mounted));
    on_mounted_slot.set(spec.on_mounted);

    let (
        btn_id,
        btn_type,
        aria_disabled,
        disabled,
        data_disabled,
        prev_marker,
        next_marker,
        early_child,
    ) = match spec.kind {
        PaginationStepKind::Prev { render_child } => {
            let attrs = runtime.prev_button_attributes(spec.id, spec.disabled);
            let effective_disabled = attrs.is_disabled();
            use_roving_focus_item(
                runtime.roving,
                PREV_ROVING_KEY,
                RovingItemMeta::new(attrs.id.clone(), !effective_disabled, false),
            );
            if spec.as_child && render_child.is_none() {
                #[cfg(debug_assertions)]
                eprintln!(
                    "monoxus::PaginationPrevButton: as_child requires render_child; rendering the default button"
                );
            }
            if let Some(cb) = render_child.as_ref().filter(|_| spec.as_child) {
                return cb.call(PaginationPrevButtonRenderChildProps {
                    attrs,
                    children: spec.children,
                });
            }
            (
                attrs.id,
                attrs.r#type,
                attrs.aria_disabled,
                attrs.disabled,
                attrs.data_disabled,
                Some(attrs.data_monoxus_pagination_prev),
                None,
                spec.children,
            )
        }
        PaginationStepKind::Next { render_child } => {
            let attrs = runtime.next_button_attributes(spec.id, spec.disabled);
            let effective_disabled = attrs.is_disabled();
            use_roving_focus_item(
                runtime.roving,
                NEXT_ROVING_KEY,
                RovingItemMeta::new(attrs.id.clone(), !effective_disabled, false),
            );
            if spec.as_child && render_child.is_none() {
                #[cfg(debug_assertions)]
                eprintln!(
                    "monoxus::PaginationNextButton: as_child requires render_child; rendering the default button"
                );
            }
            if let Some(cb) = render_child.as_ref().filter(|_| spec.as_child) {
                return cb.call(PaginationNextButtonRenderChildProps {
                    attrs,
                    children: spec.children,
                });
            }
            (
                attrs.id,
                attrs.r#type,
                attrs.aria_disabled,
                attrs.disabled,
                attrs.data_disabled,
                None,
                Some(attrs.data_monoxus_pagination_next),
                spec.children,
            )
        }
    };

    let effective_disabled = disabled;

    let on_click = {
        let runtime = runtime.clone();
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
            if let Some(handler) = on_click_slot.try_read().ok().and_then(|s| *s) {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() {
                if is_prev {
                    runtime.prev_page();
                } else {
                    runtime.next_page();
                }
            }
        }
    };

    let on_key_down = {
        let runtime = runtime.clone();
        move |evt: KeyboardEvent| {
            if is_activation_key(&evt.key()) {
                if effective_disabled {
                    evt.prevent_default();
                    return;
                }
                if let Some(handler) = on_key_down_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt.clone());
                }
                if evt.default_action_enabled() {
                    evt.prevent_default();
                    if is_prev {
                        runtime.prev_page();
                    } else {
                        runtime.next_page();
                    }
                }
                return;
            }

            if let Some(handler) = on_key_down_slot.try_read().ok().and_then(|s| *s) {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() {
                runtime
                    .roving
                    .handle_keydown(Some(roving_key), &evt, None::<fn(&str)>);
            }
        }
    };

    let on_focus = {
        let runtime = runtime.clone();
        move |_| {
            runtime
                .roving
                .handle_item_focus(roving_key, !effective_disabled);
        }
    };

    let merged = merge_global_attributes(
        spec.class.as_deref(),
        spec.style.as_deref(),
        spec.attributes,
    );

    rsx! {
        button {
            id: "{btn_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            r#type: "{btn_type}",
            aria_label: spec.aria_label.as_deref(),
            aria_disabled: aria_disabled,
            disabled: disabled,
            "data-disabled": data_disabled,
            "data-monoxus-pagination-prev": prev_marker,
            "data-monoxus-pagination-next": next_marker,
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onmounted: move |evt| {
                if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {early_child}
        }
    }
}

#[component]
pub fn PaginationPrevButton(props: PaginationPrevButtonProps) -> Element {
    render_pagination_step_button(PaginationStepButtonSpec {
        id: props.id,
        disabled: props.disabled,
        aria_label: props.aria_label,
        as_child: props.as_child,
        on_click: props.on_click,
        on_key_down: props.on_key_down,
        on_mounted: props.on_mounted,
        class: props.class,
        style: props.style,
        attributes: props.attributes,
        children: props.children,
        kind: PaginationStepKind::Prev {
            render_child: props.render_child,
        },
    })
}

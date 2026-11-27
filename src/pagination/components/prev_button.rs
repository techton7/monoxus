use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    keyboard::is_activation_key,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
};

use super::super::{
    relationships::{NEXT_ROVING_KEY, PREV_ROVING_KEY},
    runtime::use_pagination_context,
};

#[derive(Clone, PartialEq, Props)]
pub struct PaginationPrevButtonProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
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

pub(crate) enum PaginationStepKind {
    Prev,
    Next,
}

pub(crate) struct PaginationStepButtonSpec {
    pub id: Option<String>,
    pub disabled: bool,
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub kind: PaginationStepKind,
}

/// Plain-function surface unifying `PaginationPrevButton` and `PaginationNextButton` without an
/// internal `#[component]` wrapper (`Dioxus #5894`, `FM-API-4`).
pub(crate) fn render_pagination_step_button(spec: PaginationStepButtonSpec) -> Element {
    let ctx = use_pagination_context();
    let runtime = ctx.runtime;
    let is_prev = matches!(spec.kind, PaginationStepKind::Prev);
    let roving_key = if is_prev {
        PREV_ROVING_KEY
    } else {
        NEXT_ROVING_KEY
    };

    let (
        btn_id,
        btn_type,
        aria_disabled,
        disabled,
        data_disabled,
        prev_marker,
        next_marker,
    ) = match spec.kind {
        PaginationStepKind::Prev => {
            let attrs = runtime.prev_button_attributes(spec.id, spec.disabled);
            let effective_disabled = attrs.is_disabled();
            use_roving_focus_item(
                runtime.roving,
                PREV_ROVING_KEY,
                RovingItemMeta::new(attrs.id.clone(), !effective_disabled, false),
            );
            (
                attrs.id,
                attrs.r#type,
                attrs.aria_disabled,
                attrs.disabled,
                attrs.data_disabled,
                Some(attrs.data_monoxus_pagination_prev),
                None,
            )
        }
        PaginationStepKind::Next => {
            let attrs = runtime.next_button_attributes(spec.id, spec.disabled);
            let effective_disabled = attrs.is_disabled();
            use_roving_focus_item(
                runtime.roving,
                NEXT_ROVING_KEY,
                RovingItemMeta::new(attrs.id.clone(), !effective_disabled, false),
            );
            (
                attrs.id,
                attrs.r#type,
                attrs.aria_disabled,
                attrs.disabled,
                attrs.data_disabled,
                None,
                Some(attrs.data_monoxus_pagination_next),
            )
        }
    };

    let effective_disabled = disabled;
    let runtime_click = runtime.clone();
    let runtime_key = runtime.clone();
    let runtime_focus = runtime;

    let mut base = Vec::with_capacity(11);
    base.push(Attribute::new("id", btn_id, None, false));
    base.push(Attribute::new("type", btn_type, None, false));
    push_attr_opt(&mut base, "aria-disabled", aria_disabled);
    if disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_attr_opt(&mut base, "data-disabled", data_disabled);
    push_attr_opt(&mut base, "data-monoxus-pagination-prev", prev_marker);
    push_attr_opt(&mut base, "data-monoxus-pagination-next", next_marker);
    base.push(Attribute::new(
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                let mouse_evt: MouseEvent = evt.map(|d| MouseData::from(d));
                if mouse_evt
                    .trigger_button()
                    .is_some_and(|b| b != dioxus::html::input_data::MouseButton::Primary)
                {
                    return;
                }
                if effective_disabled {
                    mouse_evt.prevent_default();
                    return;
                }
                if mouse_evt.default_action_enabled() {
                    if is_prev {
                        runtime_click.prev_page();
                    } else {
                        runtime_click.next_page();
                    }
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onkeydown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                let kb_evt: KeyboardEvent = evt.map(|d| KeyboardData::from(d));
                if is_activation_key(&kb_evt.key()) {
                    if effective_disabled {
                        kb_evt.prevent_default();
                        return;
                    }
                    if kb_evt.default_action_enabled() {
                        kb_evt.prevent_default();
                        if is_prev {
                            runtime_key.prev_page();
                        } else {
                            runtime_key.next_page();
                        }
                    }
                    return;
                }
                if kb_evt.default_action_enabled() {
                    runtime_key.roving.handle_keydown(
                        Some(roving_key),
                        &kb_evt,
                        None::<fn(&str)>,
                    );
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onfocus",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                runtime_focus
                    .roving
                    .handle_item_focus(roving_key, !effective_disabled);
            })
            .erase(),
        ),
        None,
        false,
    ));

    let mut user = collect_user_attrs(spec.class, spec.style, spec.attributes);
    if !effective_disabled {
        push_event_handler(&mut user, "onclick", spec.on_click);
    }
    push_event_handler(&mut user, "onkeydown", spec.on_key_down);
    push_event_handler(&mut user, "onmounted", spec.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = spec.child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {spec.children}
        }
    }
}

#[component]
pub fn PaginationPrevButton(props: PaginationPrevButtonProps) -> Element {
    render_pagination_step_button(PaginationStepButtonSpec {
        id: props.id,
        disabled: props.disabled,
        on_click: props.on_click,
        on_key_down: props.on_key_down,
        on_mounted: props.on_mounted,
        class: props.class,
        style: props.style,
        child: props.child,
        attributes: props.attributes,
        children: props.children,
        kind: PaginationStepKind::Prev,
    })
}

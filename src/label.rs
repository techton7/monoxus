use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::compose::merge_global_attributes;

pub use ui_sdk::label::*;

/// Properties for the headless `Label` component.
#[derive(Clone, PartialEq, Props)]
pub struct LabelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub r#for: Option<String>,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default)]
    pub disabled: Option<bool>,
    #[props(default)]
    pub required: Option<bool>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mouse_down: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Headless label component providing accessible control association and double-click
/// text-selection suppression matching radix-ui/bits-ui contracts.
#[component]
pub fn Label(props: LabelProps) -> Element {
    let field_ctx = try_use_field_context();
    let control_ctx = try_use_field_control_context();

    let resolved_for = props
        .r#for
        .clone()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| control_ctx.map(|c| c.control_id()))
        .or_else(|| {
            field_ctx.and_then(|c| (!c.is_fieldset()).then(|| c.resolved_control_id()))
        });

    let effective_invalid = props
        .invalid
        .unwrap_or_else(|| field_ctx.is_some_and(|f| f.is_invalid()));
    let effective_disabled = props
        .disabled
        .unwrap_or_else(|| field_ctx.is_some_and(|f| f.is_disabled()));
    let effective_required = props
        .required
        .unwrap_or_else(|| field_ctx.is_some_and(|f| f.is_required()));

    let state = Label {
        id: props.id.clone(),
        html_for: resolved_for,
    };
    let attrs = state
        .attributes()
        .with_invalid(effective_invalid)
        .with_disabled(effective_disabled)
        .with_required(effective_required);

    let mut on_click_slot = use_hook(|| CopyValue::new(props.on_click));
    on_click_slot.set(props.on_click);
    let mut on_mouse_down_slot = use_hook(|| CopyValue::new(props.on_mouse_down));
    on_mouse_down_slot.set(props.on_mouse_down);
    let mut on_mounted_slot = use_hook(|| CopyValue::new(props.on_mounted));
    on_mounted_slot.set(props.on_mounted);

    let handle_mouse_down = move |evt: MouseEvent| {
        // Only prevent text selection if clicking inside the label itself,
        // preserving interaction/selection on nested interactive elements.
        if let Some(target) = oxidase::dom::mouse_event_target(&evt.data()) {
            if target.closest(LABEL_INTERACTIVE_TARGET_SELECTOR).is_some() {
                return;
            }
        }

        if let Some(handler) = on_mouse_down_slot.try_read().ok().and_then(|s| *s) {
            handler.call(evt.clone());
        }

        // Prevent text selection when double-clicking label text.
        if evt.default_action_enabled() && oxidase::dom::mouse_event_detail(&evt.data()) > 1 {
            evt.prevent_default();
        }
    };

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        label {
            id: attrs.id,
            r#for: attrs.r#for,
            class: merged.class,
            style: merged.style,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            "data-label-root": attrs.data_label_root,
            onclick: move |evt| {
                if let Some(handler) = on_click_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            onmousedown: handle_mouse_down,
            onmounted: move |evt| {
                if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

pub use Label as LabelRoot;

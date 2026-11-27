use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::compose::{
    collect_user_attrs, merge_props, push_attr_opt, push_event_handler,
};

pub use ui_sdk::label::*;

/// Properties for the headless `Label` component.
#[derive(Clone, PartialEq, Props)]
pub struct LabelProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub r#for: Option<String>,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default)]
    pub disabled: Option<bool>,
    #[props(default)]
    pub required: Option<bool>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mouse_down: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
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

    let on_mouse_down = props.on_mouse_down;
    let handle_mouse_down = EventHandler::new(move |evt: MouseEvent| {
        // Only prevent text selection if clicking inside the label itself,
        // preserving interaction/selection on nested interactive elements.
        if let Some(target) = oxidase::dom::mouse_event_target(&evt.data()) {
            if target.closest(LABEL_INTERACTIVE_TARGET_SELECTOR).is_some() {
                return;
            }
        }

        if let Some(handler) = on_mouse_down {
            handler.call(evt.clone());
        }

        // Prevent text selection when double-clicking label text.
        if evt.default_action_enabled() && oxidase::dom::mouse_event_detail(&evt.data()) > 1 {
            evt.prevent_default();
        }
    });

    let mut base = Vec::with_capacity(7);
    push_attr_opt(&mut base, "id", attrs.id);
    push_attr_opt(&mut base, "for", attrs.r#for);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    base.push(Attribute::new(
        "data-label-root",
        attrs.data_label_root,
        None,
        false,
    ));
    push_event_handler(&mut base, "onmousedown", Some(handle_mouse_down));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onclick", props.on_click);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);
    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        label {
            ..merged,
            {props.children}
        }
    }
}

pub use Label as LabelRoot;

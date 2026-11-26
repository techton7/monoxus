use dioxus::prelude::*;

use crate::foundation::compose::merge_global_attributes;

pub use monoxus_core::label::*;

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
    let state = Label {
        id: props.id.clone(),
        html_for: props.r#for.clone(),
    };
    let attrs = state.attributes();

    let handle_mouse_down = move |evt: MouseEvent| {
        // Only prevent text selection if clicking inside the label itself,
        // preserving interaction/selection on nested interactive elements.
        if let Some(target) = oxidase::dom::mouse_event_target(&evt.data()) {
            if target.closest(LABEL_INTERACTIVE_TARGET_SELECTOR).is_some() {
                return;
            }
        }

        if let Some(handler) = props.on_mouse_down.as_ref() {
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
            "data-label-root": attrs.data_label_root,
            onclick: move |evt| {
                if let Some(handler) = props.on_click.as_ref() {
                    handler.call(evt);
                }
            },
            onmousedown: handle_mouse_down,
            onmounted: move |evt| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

pub use Label as LabelRoot;

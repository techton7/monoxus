use dioxus::prelude::*;
use crate::toast::attrs::ToastViewportAttrs;
use crate::toast::components::{ToastAction, ToastClose, ToastDescription, ToastRoot, ToastTitle};
use crate::toast::runtime::{use_toast_runtime, TOAST_STORE};
use crate::toast::types::ToastPosition;

#[derive(Props, Clone, PartialEq)]
pub struct ToastViewportProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub dir: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub position: Option<ToastPosition>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Landmark container (<ol role="region">) mounting and structuring active notifications.
#[component]
pub fn ToastViewport(props: ToastViewportProps) -> Element {
    let _runtime = use_toast_runtime();
    let store = TOAST_STORE.read();
    let is_expanded = store.expanded;
    let dir = props
        .dir
        .as_deref()
        .unwrap_or_else(|| store.config.dir.as_str());
    let effective_position = props.position.unwrap_or(store.config.position);
    let attrs = ToastViewportAttrs::with_position(&store.config, is_expanded, dir, effective_position);
    let viewport_id = props
        .id
        .clone()
        .unwrap_or_else(|| "monoxus-toast-viewport".to_string());

    let combined_style = match (&props.style, attrs.style.is_empty()) {
        (Some(s), false) => format!("{} {}", attrs.style, s),
        (Some(s), true) => s.clone(),
        (None, false) => attrs.style.clone(),
        (None, true) => String::new(),
    };

    let content = if let Some(children) = props.children {
        children
    } else {
        rsx! {
            for (idx, (item, _visible)) in store.visible_items().into_iter().enumerate() {
                ToastRoot {
                    key: "{item.id.0}",
                    id: item.id,
                    toast_type: item.toast_type,
                    phase: item.phase,
                    index: idx,
                    visible_limit: store.config.visible_toasts,
                    expanded: is_expanded,
                    position: item.options.position.unwrap_or(effective_position),
                    test_id: item.options.test_id.clone(),
                    if let Some(ref custom_render) = item.options.custom_renderer {
                        {custom_render(item.id)}
                    } else {
                        ToastTitle {
                            "{item.title}"
                        }
                        if let Some(ref desc) = item.description {
                            ToastDescription {
                                "{desc}"
                            }
                        }
                        if let Some(ref action) = item.options.action {
                            ToastAction {
                                alt_text: action.alt_text.clone(),
                                toast_id: Some(item.id),
                                on_click: action.on_click,
                                "{action.label}"
                            }
                        }
                        if item.options.dismissible != Some(false) {
                            ToastClose {
                                toast_id: Some(item.id),
                                aria_label: Some(store.config.close_button_aria_label.clone()),
                            }
                        }
                    }
                }
            }
        }
    };

    rsx! {
        ol {
            role: "{attrs.role}",
            aria_label: "{attrs.aria_label}",
            tabindex: "{attrs.tabindex}",
            dir: "{attrs.dir}",
            "data-expanded": "{attrs.data_expanded}",
            "data-position": "{attrs.data_position}",
            "data-x-position": "{attrs.data_x_position}",
            "data-y-position": "{attrs.data_y_position}",
            class: props.class.as_deref().unwrap_or_default(),
            id: "{viewport_id}",
            style: if combined_style.is_empty() { None } else { Some(combined_style.as_str()) },
            {content}
        }
    }
}

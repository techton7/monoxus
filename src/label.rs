use dioxus::prelude::*;

use crate::foundation::compose::{Slottable, project_as_child};

/// CSS selector matching interactive controls that should not have their text selection
/// or native activation suppressed when nested inside a `<label>`.
pub const LABEL_INTERACTIVE_TARGET_SELECTOR: &str = "button, input, select, textarea";

/// Pure state definition and attribute projector for `Label`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Label {
    pub id: Option<String>,
    pub html_for: Option<String>,
}

impl Label {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_html_for(mut self, html_for: impl Into<String>) -> Self {
        self.html_for = Some(html_for.into());
        self
    }

    /// Evaluates HTML attributes for `<label>`.
    pub fn attributes(&self) -> LabelAttributes {
        LabelAttributes {
            id: self.id.clone(),
            r#for: self.html_for.clone(),
            data_label_root: "",
        }
    }
}

/// Computed HTML and data attributes published by `Label`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelAttributes {
    pub id: Option<String>,
    pub r#for: Option<String>,
    pub data_label_root: &'static str,
}

/// Projects headless label attributes onto a slottable child without adding wrapper elements.
pub fn project_label_as_child<T, C>(
    state: &Label,
    target: T,
    content: Slottable<C>,
) -> (T, LabelAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.attributes(), content)
}

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

    rsx! {
        label {
            id: attrs.id,
            r#for: attrs.r#for,
            class: props.class,
            style: props.style,
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
            ..props.attributes,
            {props.children}
        }
    }
}

pub use Label as LabelRoot;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_attributes_projection() {
        let l1 = Label::new()
            .with_id("lbl-1")
            .with_html_for("target-input");
        let a1 = l1.attributes();
        assert_eq!(a1.id.as_deref(), Some("lbl-1"));
        assert_eq!(a1.r#for.as_deref(), Some("target-input"));
        assert_eq!(a1.data_label_root, "");

        let l2 = Label::new();
        let a2 = l2.attributes();
        assert_eq!(a2.id, None);
        assert_eq!(a2.r#for, None);
        assert_eq!(a2.data_label_root, "");
    }

    #[test]
    fn label_as_child_slot_projection() {
        let state = Label::new().with_id("slot-lbl").with_html_for("slot-input");
        let (target, attrs, content) =
            project_label_as_child(&state, "span-wrapper", Slottable::new("First Name"));
        assert_eq!(target, "span-wrapper");
        assert_eq!(attrs.id.as_deref(), Some("slot-lbl"));
        assert_eq!(attrs.r#for.as_deref(), Some("slot-input"));
        assert_eq!(attrs.data_label_root, "");
        assert_eq!(content, "First Name");
    }

    #[test]
    fn label_interactive_selector_constant() {
        assert_eq!(
            LABEL_INTERACTIVE_TARGET_SELECTOR,
            "button, input, select, textarea"
        );
    }
}

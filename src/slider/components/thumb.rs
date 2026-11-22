use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{ResizeObserverBox, use_element_size_with_mount},
    compose::{Slottable, project_as_child},
};

use crate::slider::{
    attrs::SliderThumbAttributes, runtime::use_slider_context, types::Orientation,
};

/// Projects child targets through [`AsChildSlot`] for `SliderThumb`.
pub fn project_slider_thumb_as_child<T, C>(
    attrs: &SliderThumbAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, SliderThumbAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

/// Props for [`SliderThumb`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderThumbProps {
    #[props(default = 0)]
    pub index: usize,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub as_child: Option<bool>,
    pub children: Option<Element>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub aria_label: Option<String>,
    pub aria_labelledby: Option<String>,
    pub aria_valuetext: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Headless interactive thumb handle supporting keyboard navigation, pointer dragging, and ARIA attributes.
#[component]
pub fn SliderThumb(props: SliderThumbProps) -> Element {
    let ctx = use_slider_context();
    let attrs = ctx.0.thumb_attrs(
        props.index,
        props.aria_label.clone(),
        props.aria_labelledby.clone(),
        props.aria_valuetext.clone(),
    );
    let thumb_id = props.id.unwrap_or_else(|| attrs.id().to_string());

    use_effect(use_reactive((&props.index, &thumb_id), {
        let mut runtime = ctx.0.clone();
        move |(idx, tid)| {
            runtime.register_thumb_id(idx, tid);
        }
    }));

    let mut mounted_rev = use_signal(|| 0u64);
    let thumb_size_sig =
        use_element_size_with_mount(&thumb_id, ResizeObserverBox::BorderBox, mounted_rev);
    use_effect({
        let mut runtime = ctx.0.clone();
        let index = props.index;
        let orientation = runtime.def.orientation;
        move || {
            if let Some(size) = *thumb_size_sig.read() {
                let s = match orientation {
                    Orientation::Vertical => size.height,
                    _ => size.width,
                };
                let mut sizes = runtime.thumb_sizes.peek().clone();
                if sizes.len() <= index {
                    sizes.resize(index + 1, None);
                }
                if sizes[index] != Some(s) {
                    sizes[index] = Some(s);
                    runtime.thumb_sizes.set(sizes);
                }
            }
        }
    });

    let on_pointer_down = {
        let mut runtime = ctx.0.clone();
        let index = props.index;
        move |evt: PointerEvent| {
            if runtime.def.disabled {
                return;
            }
            if evt
                .trigger_button()
                .is_some_and(|b| b != dioxus::html::input_data::MouseButton::Primary)
            {
                return;
            }
            evt.prevent_default();
            runtime.thumb_drag_claimed.set(true);
            runtime.start_drag(index);
        }
    };

    let on_key_down = {
        let mut runtime = ctx.0.clone();
        let index = props.index;
        move |evt: KeyboardEvent| {
            if runtime.def.disabled {
                return;
            }
            let key_name = match evt.key() {
                Key::Named(NamedKey::ArrowRight) => "ArrowRight",
                Key::Named(NamedKey::ArrowLeft) => "ArrowLeft",
                Key::Named(NamedKey::ArrowUp) => "ArrowUp",
                Key::Named(NamedKey::ArrowDown) => "ArrowDown",
                Key::Named(NamedKey::PageUp) => "PageUp",
                Key::Named(NamedKey::PageDown) => "PageDown",
                Key::Named(NamedKey::Home) => "Home",
                Key::Named(NamedKey::End) => "End",
                _ => return,
            };
            evt.prevent_default();
            let shift = evt.modifiers().contains(Modifiers::SHIFT);
            runtime.handle_keydown(index, key_name, shift);
        }
    };

    let on_focus = {
        let mut runtime = ctx.0.clone();
        let index = props.index;
        move |_| {
            runtime.focused_thumb.set(Some(index));
        }
    };

    let on_blur = {
        let mut runtime = ctx.0.clone();
        move |_| {
            runtime.focused_thumb.set(None);
        }
    };

    let combined_style = if let Some(ref s) = props.style {
        format!("{} {}", attrs.style(), s)
    } else {
        attrs.style().to_string()
    };

    rsx! {
        span {
            id: "{thumb_id}",
            role: "{attrs.role()}",
            tabindex: attrs.tabindex(),
            aria_valuenow: attrs.aria_valuenow,
            aria_valuemin: attrs.aria_valuemin,
            aria_valuemax: attrs.aria_valuemax,
            aria_valuetext: attrs.aria_valuetext.as_deref(),
            aria_orientation: attrs.aria_orientation,
            aria_disabled: attrs.aria_disabled,
            aria_label: attrs.aria_label.as_deref(),
            aria_labelledby: attrs.aria_labelledby.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: "{combined_style}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-value": "{attrs.data_value()}",
            "data-active": attrs.data_active(),
            "data-monoxus-slider-thumb": "",
            onpointerdown: on_pointer_down,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onblur: on_blur,
            onmounted: move |_| mounted_rev += 1,
            ..props.attributes,
            {props.children}
        }
    }
}

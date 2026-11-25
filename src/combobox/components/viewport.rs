use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::browser::{WatcherGuard, start_element_scroll_monitor};

use crate::combobox::{attrs::ComboboxViewportAttrs, runtime::use_combobox_context};

#[derive(Clone, PartialEq)]
pub struct ComboboxViewportRenderProps {
    pub attrs: ComboboxViewportAttrs,
    pub class: Option<String>,
    pub style: String,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxViewportProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxViewportRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ComboboxViewport(props: ComboboxViewportProps) -> Element {
    let ctx = use_combobox_context();
    let vp_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().viewport_id().to_owned());
    ctx.runtime.set_viewport_id(vp_id.clone());

    let is_open = ctx.runtime.is_open();
    let input_value = ctx.runtime.input_value();
    let scroll_guard: Rc<RefCell<Option<WatcherGuard>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    {
        let rt = ctx.runtime.clone();
        let vid = vp_id.clone();
        let guard_slot = Rc::clone(&scroll_guard);
        use_effect(use_reactive((&is_open, &input_value), move |(open, _)| {
            if let Some(prev) = guard_slot.borrow_mut().take() {
                prev.stop();
            }
            if !open {
                return;
            }
            rt.refresh_scroll_buttons();
            let scroll_rt = rt.clone();
            let guard = start_element_scroll_monitor(&vid, move || {
                scroll_rt.refresh_scroll_buttons();
            });
            *guard_slot.borrow_mut() = Some(guard);
        }));
    }

    {
        let guard_slot = Rc::clone(&scroll_guard);
        use_drop(move || {
            if let Some(prev) = guard_slot.borrow_mut().take() {
                prev.stop();
            }
        });
    }

    let attrs = ctx.runtime.viewport_attributes(vp_id);
    let default_style =
        "position: relative; flex: 1 1 auto; min-height: 0; overflow-x: hidden; overflow-y: auto;";
    let combined_style = match props.style {
        Some(ref s) if !s.is_empty() => format!("{default_style} {s}"),
        _ => default_style.to_string(),
    };

    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            return render_child.call(ComboboxViewportRenderProps {
                attrs,
                class: props.class,
                style: combined_style,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            "data-combobox-viewport": "",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{combined_style}",
            ..props.attributes,
            {props.children}
        }
    }
}

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{compose::merge_global_attributes, shared::use_stable_id};

use crate::combobox::{
    attrs::{ComboboxGroupAttrs, ComboboxGroupHeadingAttrs},
    runtime::use_combobox_context,
    types::ComboboxGroupContext,
};

#[derive(Clone, PartialEq)]
pub struct ComboboxGroupRenderProps {
    pub attrs: ComboboxGroupAttrs,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxGroupProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub group_key: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxGroupRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ComboboxGroup(props: ComboboxGroupProps) -> Element {
    let ctx = use_combobox_context();
    let fallback_key = use_stable_id(props.group_key.clone().map(Cow::Owned));
    let group_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().group_id(fallback_key.as_ref()));
    let heading_id = use_signal(|| None::<String>);

    use_context_provider(|| ComboboxGroupContext {
        group_id: group_id.clone(),
        heading_id,
    });

    let attrs = ComboboxGroupAttrs::new(group_id, heading_id.read().clone());

    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            return render_child.call(ComboboxGroupRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            "aria-labelledby": attrs.aria_labelledby(),
            "data-combobox-group": "",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            ..merged.rest,
            {props.children}
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ComboboxGroupHeadingRenderProps {
    pub attrs: ComboboxGroupHeadingAttrs,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxGroupHeadingProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxGroupHeadingRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ComboboxGroupHeading(props: ComboboxGroupHeadingProps) -> Element {
    let group_ctx = try_consume_context::<ComboboxGroupContext>();
    let stable_heading = use_stable_id(props.id.clone().map(Cow::Owned));
    let heading_id = props.id.clone().unwrap_or_else(|| {
        if let Some(ref g) = group_ctx {
            format!("{}-heading", g.group_id)
        } else {
            stable_heading.into_owned()
        }
    });

    // DEF-CBX-12: Track latest dynamic `heading_id` for `use_drop`.
    let tracked_heading_id: Rc<RefCell<String>> =
        use_hook(|| Rc::new(RefCell::new(heading_id.clone())));
    if *tracked_heading_id.borrow() != heading_id {
        *tracked_heading_id.borrow_mut() = heading_id.clone();
    }

    if let Some(mut g) = group_ctx.clone() {
        if g.heading_id.try_peek().ok().as_deref() != Some(&Some(heading_id.clone())) {
            let _ = g
                .heading_id
                .try_write()
                .map(|mut w| *w = Some(heading_id.clone()));
        }
    }

    {
        let g = group_ctx;
        let tracked = Rc::clone(&tracked_heading_id);
        use_drop(move || {
            let hid = tracked.borrow().clone();
            if let Some(mut group) = g {
                if let Ok(mut w) = group.heading_id.try_write() {
                    if w.as_deref() == Some(&hid) {
                        *w = None;
                    }
                }
            }
        });
    }

    let attrs = ComboboxGroupHeadingAttrs::new(heading_id);

    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            return render_child.call(ComboboxGroupHeadingRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{attrs.id()}",
            "data-combobox-group-label": "",
            "data-combobox-group-heading": "",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            ..merged.rest,
            {props.children}
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ComboboxSeparatorRenderProps {
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxSeparatorProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxSeparatorRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ComboboxSeparator(props: ComboboxSeparatorProps) -> Element {
    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            return render_child.call(ComboboxSeparatorRenderProps {
                class: props.class,
                style: props.style,
                attributes: props.attributes,
            });
        }
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            role: "presentation",
            "aria-hidden": "true",
            "data-combobox-separator": "",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            ..merged.rest,
        }
    }
}

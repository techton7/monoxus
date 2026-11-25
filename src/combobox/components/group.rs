use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;

use crate::combobox::{
    attrs::{ComboboxGroupAttrs, ComboboxGroupHeadingAttrs},
    runtime::use_combobox_context,
    types::ComboboxGroupContext,
};

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

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            "aria-labelledby": attrs.aria_labelledby(),
            "data-combobox-group": "",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            ..props.attributes,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxGroupHeadingProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
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
        let hid = heading_id.clone();
        use_drop(move || {
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

    rsx! {
        div {
            id: "{attrs.id()}",
            "data-combobox-group-label": "",
            "data-combobox-group-heading": "",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            ..props.attributes,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxSeparatorProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ComboboxSeparator(props: ComboboxSeparatorProps) -> Element {
    rsx! {
        div {
            role: "presentation",
            "aria-hidden": "true",
            "data-combobox-separator": "",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            ..props.attributes,
        }
    }
}

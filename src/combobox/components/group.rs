use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    shared::use_stable_id,
};

use crate::combobox::{
    attrs::{ComboboxGroupAttrs, ComboboxGroupHeadingAttrs},
    runtime::use_combobox_context,
    types::ComboboxGroupContext,
};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxGroupProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub group_key: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
    let id_str = attrs.id().to_owned();
    let role_str = attrs.role();
    let aria_labelledby = attrs.aria_labelledby().map(str::to_owned);

    let mut base = Vec::with_capacity(4);
    base.push(Attribute::new("id", id_str, None, false));
    base.push(Attribute::new("role", role_str, None, false));
    push_attr_opt(&mut base, "aria-labelledby", aria_labelledby);
    base.push(Attribute::new("data-combobox-group", "", None, false));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxGroupHeadingProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
    let id_str = attrs.id().to_owned();

    let mut base = Vec::with_capacity(3);
    base.push(Attribute::new("id", id_str, None, false));
    base.push(Attribute::new("data-combobox-group-label", "", None, false));
    base.push(Attribute::new(
        "data-combobox-group-heading",
        "",
        None,
        false,
    ));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxSeparatorProps {
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ComboboxSeparator(props: ComboboxSeparatorProps) -> Element {
    let mut base = Vec::with_capacity(3);
    base.push(Attribute::new("role", "presentation", None, false));
    base.push(Attribute::new("aria-hidden", "true", None, false));
    base.push(Attribute::new("data-combobox-separator", "", None, false));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
        }
    }
}

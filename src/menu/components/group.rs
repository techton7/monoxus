use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    shared::use_stable_id,
};
use crate::menu::{
    MenuGroupAttributes, MenuGroupContext, MenuHeadingAttributes, MenuPart, MenuRootContext,
};

#[derive(Props, Clone, PartialEq)]
pub struct MenuGroupProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Semantic `role="group"` container automatically linked to any nested `GroupHeading` via `aria-labelledby` (`3B D-6`).
#[component]
pub fn MenuGroup(props: MenuGroupProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let base_id = use_stable_id(None);
    let group_id = props.id.unwrap_or_else(|| format!("{base_id}-group"));
    let default_heading_id = format!("{base_id}-heading");
    let heading_id = use_signal(|| None);

    use_context_provider(|| MenuGroupContext {
        group_id: group_id.clone(),
        heading_id,
        default_heading_id,
    });

    let effective_labelledby = props
        .aria_labelledby
        .or_else(|| heading_id.read().clone());

    let attrs = MenuGroupAttributes::new(
        group_id,
        effective_labelledby,
        root.variant,
        MenuPart::Group,
    );
    let part_attr = MenuPart::Group.static_data_attr_name(root.variant);

    let mut base = Vec::with_capacity(4);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    push_attr_opt(&mut base, "aria-labelledby", attrs.aria_labelledby);
    base.push(Attribute::new(part_attr, "", None, false));

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
pub struct MenuGroupHeadingProps {
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

/// Heading inside [`MenuGroup`] registering its ID with the enclosing group's `aria-labelledby` (`3B D-6`).
#[component]
pub fn MenuGroupHeading(props: MenuGroupHeadingProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let group = try_use_context::<MenuGroupContext>();
    let fallback_id = use_stable_id(None);

    let resolved_id = props.id.unwrap_or_else(|| {
        group
            .as_ref()
            .map(|g| g.default_heading_id.clone())
            .unwrap_or_else(|| fallback_id.into_owned())
    });

    if let Some(group_ctx) = group.as_ref() {
        let mut hid = group_ctx.heading_id;
        if hid
            .try_peek()
            .is_ok_and(|g| g.as_deref() != Some(resolved_id.as_str()))
        {
            let _ = hid.try_write().map(|mut w| *w = Some(resolved_id.clone()));
        }
    }

    let cleanup_group = group.clone();
    let cleanup_id = resolved_id.clone();
    dioxus::core::use_drop(move || {
        if let Some(g) = cleanup_group {
            let mut hid = g.heading_id;
            if hid
                .try_peek()
                .is_ok_and(|cur| cur.as_deref() == Some(cleanup_id.as_str()))
            {
                let _ = hid.try_write().map(|mut w| *w = None);
            }
        }
    });

    let attrs = MenuHeadingAttributes::new(resolved_id, root.variant, MenuPart::GroupHeading);
    let part_attr = MenuPart::GroupHeading.static_data_attr_name(root.variant);

    let base = vec![
        Attribute::new("id", attrs.id, None, false),
        Attribute::new(part_attr, "", None, false),
    ];

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

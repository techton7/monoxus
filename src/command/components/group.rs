use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    shared::use_stable_id,
    state::sync_signal,
};

use super::super::{
    attrs::{
        CommandGroupAttrs, CommandGroupHeadingAttrs, CommandGroupItemsAttrs,
        merge_flex_column_style, merge_ranked_item_style,
    },
    runtime::{CommandContext, CommandGroupContext},
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandGroupProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandGroup(props: CommandGroupProps) -> Element {
    let mut ctx = use_context::<CommandContext>();
    let fallback_key = use_stable_id(None).into_owned();
    let group_id = props.id.clone().unwrap_or_else(|| {
        let key = props.value.as_deref().unwrap_or(&fallback_key);
        ctx.relationships.peek().group_id(key)
    });
    let resolved_value = props
        .value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| group_id.clone());

    let mut group_value_sig = use_signal(|| resolved_value.clone());
    let prev_value = group_value_sig.peek().clone();
    if prev_value != resolved_value {
        ctx.unregister_group(&prev_value);
        group_value_sig.set(resolved_value.clone());
    }
    let mut group_id_sig = use_signal(|| group_id.clone());
    sync_signal(&mut group_id_sig, group_id.clone());
    let heading_id_sig = use_signal(|| None::<String>);
    let mut force_mount_sig = use_signal(|| props.force_mount);
    sync_signal(&mut force_mount_sig, props.force_mount);

    use_context_provider(|| CommandGroupContext {
        value: group_value_sig,
        group_id: group_id_sig,
        heading_id: heading_id_sig,
        force_mount: force_mount_sig,
    });

    ctx.register_group(
        resolved_value.clone(),
        group_id.clone(),
        heading_id_sig.peek().clone(),
        props.force_mount,
    );

    let mut ctx_for_drop = ctx;
    use_drop(move || {
        if let Ok(val) = group_value_sig.try_peek() {
            ctx_for_drop.unregister_group(&val);
        }
    });

    let should_render = ctx.should_render_group_value(&resolved_value, props.force_mount);
    let visual_order = ctx.group_visual_order(&resolved_value);
    let base_style = merge_ranked_item_style(should_render, visual_order, None);
    let attrs = CommandGroupAttrs::new(group_id, resolved_value, should_render);

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    if attrs.hidden {
        base.push(Attribute::new("hidden", true, None, false));
    }
    base.push(Attribute::new(
        "data-command-group",
        attrs.data_command_group,
        None,
        false,
    ));
    base.push(Attribute::new("data-value", attrs.data_value, None, false));
    push_attr_opt(&mut base, "style", base_style);

    let mut user = if should_render {
        collect_user_attrs(props.class, props.style, props.attributes)
    } else {
        let mut filtered = collect_user_attrs(props.class, None, props.attributes);
        filtered.retain(|a| !(a.namespace.is_none() && a.name == "style"));
        filtered
    };
    push_event_handler(&mut user, "onmounted", props.on_mounted);
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

#[derive(Clone, PartialEq, Props)]
pub struct CommandGroupHeadingProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandGroupHeading(props: CommandGroupHeadingProps) -> Element {
    let mut ctx = use_context::<CommandContext>();
    let mut group_ctx = use_context::<CommandGroupContext>();
    let fallback_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let heading_id = props.id.clone().unwrap_or_else(|| {
        let gv = group_ctx.value.peek().clone();
        if gv.is_empty() {
            fallback_id
        } else {
            ctx.relationships.peek().group_heading_id(&gv)
        }
    });

    let mut tracked_heading_id = use_signal(|| heading_id.clone());
    if *tracked_heading_id.peek() != heading_id {
        tracked_heading_id.set(heading_id.clone());
    }
    if group_ctx.heading_id.peek().as_deref() != Some(heading_id.as_str()) {
        group_ctx.heading_id.set(Some(heading_id.clone()));
    }
    let gv = group_ctx.value.peek().clone();
    ctx.set_group_heading(&gv, Some(heading_id.clone()));

    let mut heading_sig = group_ctx.heading_id;
    let group_val_sig = group_ctx.value;
    let mut ctx_drop = ctx;
    use_drop(move || {
        let latest_id = tracked_heading_id.try_peek().ok().map(|s| s.clone());
        if let Ok(mut sig) = heading_sig.try_write() {
            if latest_id.is_none() || sig.as_deref() == latest_id.as_deref() {
                *sig = None;
            }
        }
        if let Ok(gv) = group_val_sig.try_peek() {
            ctx_drop.set_group_heading(&gv, None);
        }
    });

    let attrs = CommandGroupHeadingAttrs::new(heading_id);

    let mut base = Vec::with_capacity(2);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new(
        "data-command-group-heading",
        attrs.data_command_group_heading,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);
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

#[derive(Clone, PartialEq, Props)]
pub struct CommandGroupItemsProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandGroupItems(props: CommandGroupItemsProps) -> Element {
    let ctx = use_context::<CommandContext>();
    let group_ctx = use_context::<CommandGroupContext>();
    let items_id = props.id.clone().unwrap_or_else(|| {
        let gv = group_ctx.value.peek().clone();
        ctx.relationships.peek().group_items_id(&gv)
    });
    let heading_id = group_ctx.heading_id.read().clone();
    let is_grid = ctx.columns.read().is_some();
    let base_style = merge_flex_column_style(None, is_grid);
    let attrs = CommandGroupItemsAttrs::new(items_id, heading_id);

    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    push_attr_opt(&mut base, "aria-labelledby", attrs.aria_labelledby);
    base.push(Attribute::new(
        "data-command-group-items",
        attrs.data_command_group_items,
        None,
        false,
    ));
    push_attr_opt(&mut base, "style", base_style);

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);
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

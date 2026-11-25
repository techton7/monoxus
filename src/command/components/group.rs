use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;

use super::super::{
    attrs::{
        CommandGroupAttrs, CommandGroupHeadingAttrs, CommandGroupItemsAttrs,
        merge_flex_column_style, merge_ranked_item_style,
    },
    runtime::{CommandContext, CommandGroupContext},
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandGroupRenderProps {
    pub id: String,
    pub role: &'static str,
    pub hidden: bool,
    pub data_command_group: &'static str,
    pub data_value: String,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandGroupProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandGroupRenderProps, Element>>,
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
    if *group_id_sig.peek() != group_id {
        group_id_sig.set(group_id.clone());
    }
    let heading_id_sig = use_signal(|| None::<String>);
    let mut force_mount_sig = use_signal(|| props.force_mount);
    if *force_mount_sig.peek() != props.force_mount {
        force_mount_sig.set(props.force_mount);
    }

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
    let rendered_style = merge_ranked_item_style(should_render, visual_order, props.style.as_deref());
    let attrs = CommandGroupAttrs::new(group_id, resolved_value, should_render);

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandGroupRenderProps {
                id: attrs.id,
                role: attrs.role,
                hidden: attrs.hidden,
                data_command_group: attrs.data_command_group,
                data_value: attrs.data_value,
                class: props.class,
                style: rendered_style,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id}",
            role: "{attrs.role}",
            hidden: attrs.hidden,
            "data-command-group": attrs.data_command_group,
            "data-value": "{attrs.data_value}",
            class: props.class,
            style: rendered_style,
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

#[derive(Clone, PartialEq, Props)]
pub struct CommandGroupHeadingRenderProps {
    pub id: String,
    pub data_command_group_heading: &'static str,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandGroupHeadingProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandGroupHeadingRenderProps, Element>>,
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

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandGroupHeadingRenderProps {
                id: attrs.id,
                data_command_group_heading: attrs.data_command_group_heading,
                class: props.class,
                style: props.style,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id}",
            "data-command-group-heading": attrs.data_command_group_heading,
            class: props.class,
            style: props.style,
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

#[derive(Clone, PartialEq, Props)]
pub struct CommandGroupItemsRenderProps {
    pub id: String,
    pub role: &'static str,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    pub data_command_group_items: &'static str,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandGroupItemsProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandGroupItemsRenderProps, Element>>,
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
    let merged_style = merge_flex_column_style(props.style.as_deref(), is_grid);
    let attrs = CommandGroupItemsAttrs::new(items_id, heading_id);

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandGroupItemsRenderProps {
                id: attrs.id,
                role: attrs.role,
                aria_labelledby: attrs.aria_labelledby,
                data_command_group_items: attrs.data_command_group_items,
                class: props.class,
                style: merged_style,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id}",
            role: "{attrs.role}",
            aria_labelledby: attrs.aria_labelledby.as_deref(),
            "data-command-group-items": attrs.data_command_group_items,
            class: props.class,
            style: merged_style,
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

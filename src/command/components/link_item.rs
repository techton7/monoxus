use dioxus::prelude::*;

use crate::foundation::browser::is_element_focused_by_id;
use crate::foundation::shared::use_stable_id;

use super::super::{
    attrs::{CommandItemAttrs, merge_ranked_item_style},
    runtime::{CommandContext, CommandGroupContext},
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandLinkItemRenderProps {
    pub id: String,
    pub role: &'static str,
    pub aria_disabled: &'static str,
    pub aria_selected: &'static str,
    pub data_command_item: &'static str,
    pub data_value: String,
    #[props(default)]
    pub data_group: Option<String>,
    #[props(default)]
    pub data_disabled: Option<&'static str>,
    #[props(default)]
    pub data_selected: Option<&'static str>,
    #[props(default)]
    pub href: Option<String>,
    #[props(default)]
    pub target: Option<String>,
    #[props(default)]
    pub rel: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_mousedown: EventHandler<MouseEvent>,
    pub on_pointermove: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandLinkItemProps {
    pub value: String,
    #[props(default)]
    pub href: Option<String>,
    #[props(default)]
    pub target: Option<String>,
    #[props(default)]
    pub rel: Option<String>,
    #[props(default)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub keywords: Vec<String>,
    #[props(default)]
    pub on_select: Option<EventHandler<()>>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandLinkItemRenderProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_pointermove: Option<EventHandler<PointerEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandLinkItem(props: CommandLinkItemProps) -> Element {
    let mut ctx = use_context::<CommandContext>();
    let group_ctx = try_consume_context::<CommandGroupContext>();
    let normalized_value = props.value.trim().to_string();

    let fallback_key = use_stable_id(None).into_owned();
    let item_id = props.id.clone().unwrap_or_else(|| {
        let key = if normalized_value.is_empty() {
            fallback_key.as_str()
        } else {
            normalized_value.as_str()
        };
        ctx.relationships.peek().item_id(key)
    });

    let group_value = group_ctx.map(|g| g.value.read().clone());
    let group_force_mount = group_ctx.is_some_and(|g| *g.force_mount.read());

    let mut tracked_value = use_signal(|| normalized_value.clone());
    let prev_val = tracked_value.peek().clone();
    if prev_val != normalized_value {
        ctx.unregister_item(&prev_val);
        tracked_value.set(normalized_value.clone());
    }

    ctx.register_item(
        normalized_value.clone(),
        item_id.clone(),
        group_value.clone(),
        props.disabled,
        props.force_mount,
        props.keywords.clone(),
        props.on_select,
    );
    ctx.set_link_item(&normalized_value, true);

    let mut ctx_for_drop = ctx;
    use_drop(move || {
        if let Ok(val) = tracked_value.try_peek() {
            ctx_for_drop.unregister_item(&val);
        }
    });

    let should_render = ctx.should_render_item_value(
        &normalized_value,
        props.force_mount,
        group_force_mount,
    );
    let visual_order = ctx.item_visual_order(&normalized_value);
    let rendered_style =
        merge_ranked_item_style(should_render, visual_order, props.style.as_deref());

    let is_selected =
        should_render && !normalized_value.is_empty() && *ctx.value.read() == normalized_value;
    let attrs = CommandItemAttrs::new(
        item_id,
        normalized_value.clone(),
        group_value,
        props.disabled,
        is_selected,
    );

    let disabled = props.disabled;
    let on_select = props.on_select;
    let consumer_click = props.on_click;
    let consumer_pointermove = props.on_pointermove;

    let input_id_sig = ctx.input_id;
    let handle_mousedown = EventHandler::new(move |evt: MouseEvent| {
        // 4B D-7: Keep DOM focus on CommandInput only when CommandInput currently holds focus;
        // when focus is outside CommandInput, allow default press focus so CommandRoot/<a> receives focus.
        let input_id = input_id_sig.peek().clone();
        if is_element_focused_by_id(&input_id) {
            evt.prevent_default();
        }
    });

    let mut ctx_for_move = ctx;
    let val_for_move = normalized_value.clone();
    let handle_pointermove = EventHandler::new(move |evt: PointerEvent| {
        if let Some(handler) = consumer_pointermove {
            handler.call(evt);
        }
        if disabled || *ctx_for_move.disable_pointer_selection.peek() {
            return;
        }
        ctx_for_move.set_value(val_for_move.clone(), true);
    });

    let mut ctx_for_click = ctx;
    let val_for_click = normalized_value;
    let handle_click = EventHandler::new(move |evt: MouseEvent| {
        if disabled {
            evt.prevent_default();
            return;
        }
        ctx_for_click.set_value(val_for_click.clone(), true);
        if let Some(cb) = on_select {
            cb.call(());
        }
        if let Some(handler) = consumer_click {
            handler.call(evt);
        }
    });

    if props.as_child {
        if !should_render {
            return rsx! {};
        }
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandLinkItemRenderProps {
                id: attrs.id,
                role: attrs.role,
                aria_disabled: attrs.aria_disabled,
                aria_selected: attrs.aria_selected,
                data_command_item: attrs.data_command_item,
                data_value: attrs.data_value,
                data_group: attrs.data_group,
                data_disabled: attrs.data_disabled,
                data_selected: attrs.data_selected,
                href: props.href,
                target: props.target,
                rel: props.rel,
                class: props.class,
                style: rendered_style,
                on_mousedown: handle_mousedown,
                on_pointermove: handle_pointermove,
                on_click: handle_click,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        a {
            id: should_render.then_some(attrs.id),
            role: should_render.then_some(attrs.role),
            hidden: !should_render,
            aria_hidden: (!should_render).then_some("true"),
            aria_disabled: should_render.then_some(attrs.aria_disabled),
            aria_selected: should_render.then_some(attrs.aria_selected),
            "data-command-item": should_render.then_some(attrs.data_command_item),
            "data-value": should_render.then_some(attrs.data_value),
            "data-group": if should_render { attrs.data_group.as_deref() } else { None },
            "data-disabled": if should_render { attrs.data_disabled } else { None },
            "data-selected": if should_render { attrs.data_selected } else { None },
            href: props.href.as_deref(),
            target: props.target.as_deref(),
            rel: props.rel.as_deref(),
            class: props.class,
            style: rendered_style,
            onmousedown: move |evt| handle_mousedown.call(evt),
            onpointermove: move |evt| handle_pointermove.call(evt),
            onclick: move |evt| handle_click.call(evt),
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

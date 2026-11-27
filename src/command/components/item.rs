use dioxus::prelude::*;

use crate::foundation::{
    browser::is_element_focused_by_id,
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    shared::use_stable_id,
};

use super::super::{
    attrs::{CommandItemAttrs, merge_ranked_item_style},
    runtime::{CommandContext, CommandGroupContext},
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandItemProps {
    #[props(into)]
    pub value: String,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub keywords: Vec<String>,
    #[props(default)]
    pub on_select: Option<EventHandler<()>>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_pointer_move: Option<EventHandler<PointerEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

pub(crate) enum CommandItemKind {
    Div,
    Link {
        href: Option<String>,
        target: Option<String>,
        rel: Option<String>,
    },
}

pub(crate) struct CommandItemSurfaceSpec {
    pub value: String,
    pub id: Option<String>,
    pub disabled: bool,
    pub force_mount: bool,
    pub keywords: Vec<String>,
    pub on_select: Option<EventHandler<()>>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub on_pointer_move: Option<EventHandler<PointerEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub kind: CommandItemKind,
}

/// Plain-function surface unifying `CommandItem` and `CommandLinkItem` without an internal
/// `#[component]` wrapper (`Dioxus #5894`, `FM-API-4`).
pub(crate) fn render_command_item_surface(spec: CommandItemSurfaceSpec) -> Element {
    let mut ctx = use_context::<CommandContext>();
    let group_ctx = try_consume_context::<CommandGroupContext>();
    let normalized_value = spec.value.trim().to_string();
    let is_link = matches!(spec.kind, CommandItemKind::Link { .. });

    let fallback_key = use_stable_id(None).into_owned();
    let item_id = spec.id.clone().unwrap_or_else(|| {
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
        spec.disabled,
        spec.force_mount,
        spec.keywords,
        spec.on_select,
    );
    if is_link {
        ctx.set_link_item(&normalized_value, true);
    }

    let mut ctx_for_drop = ctx;
    use_drop(move || {
        if let Ok(val) = tracked_value.try_peek() {
            ctx_for_drop.unregister_item(&val);
        }
    });

    let should_render = ctx.should_render_item_value(
        &normalized_value,
        spec.force_mount,
        group_force_mount,
    );
    let visual_order = ctx.item_visual_order(&normalized_value);
    let base_style = merge_ranked_item_style(should_render, visual_order, None);

    let is_selected =
        should_render && !normalized_value.is_empty() && *ctx.value.read() == normalized_value;
    let attrs = CommandItemAttrs::new(
        item_id,
        normalized_value.clone(),
        group_value,
        spec.disabled,
        is_selected,
    );

    let mut disabled_slot = use_hook(|| CopyValue::new(spec.disabled));
    disabled_slot.set(spec.disabled);
    let mut on_select_slot = use_hook(|| CopyValue::new(spec.on_select));
    on_select_slot.set(spec.on_select);
    let mut on_click_slot = use_hook(|| CopyValue::new(spec.on_click));
    on_click_slot.set(spec.on_click);
    let mut val_slot = use_hook(|| CopyValue::new(normalized_value.clone()));
    if val_slot
        .try_read()
        .is_ok_and(|cur| *cur != normalized_value)
    {
        val_slot.set(normalized_value);
    }

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
    let handle_pointer_move = EventHandler::new(move |_evt: PointerEvent| {
        let disabled = disabled_slot.try_read().map(|d| *d).unwrap_or(false);
        if disabled || *ctx_for_move.disable_pointer_selection.peek() {
            return;
        }
        if let Ok(val) = val_slot.try_read() {
            ctx_for_move.set_value(val.clone(), true);
        }
    });

    let mut ctx_for_click = ctx;
    let handle_click = EventHandler::new(move |evt: MouseEvent| {
        let disabled = disabled_slot.try_read().map(|d| *d).unwrap_or(false);
        if disabled {
            if is_link {
                evt.prevent_default();
            }
            return;
        }
        if let Ok(val) = val_slot.try_read() {
            ctx_for_click.set_value(val.clone(), true);
        }
        if let Some(cb) = on_select_slot.try_read().ok().and_then(|s| *s) {
            cb.call(());
        }
        if let Some(handler) = on_click_slot.try_read().ok().and_then(|s| *s) {
            handler.call(evt);
        }
    });

    let mut base = Vec::with_capacity(18);
    if should_render {
        base.push(Attribute::new("id", attrs.id, None, false));
        base.push(Attribute::new("role", attrs.role, None, false));
        base.push(Attribute::new(
            "aria-disabled",
            attrs.aria_disabled,
            None,
            false,
        ));
        base.push(Attribute::new(
            "aria-selected",
            attrs.aria_selected,
            None,
            false,
        ));
        base.push(Attribute::new(
            "data-command-item",
            attrs.data_command_item,
            None,
            false,
        ));
        base.push(Attribute::new("data-value", attrs.data_value, None, false));
        push_attr_opt(&mut base, "data-group", attrs.data_group);
        push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
        push_attr_opt(&mut base, "data-selected", attrs.data_selected);
    } else {
        base.push(Attribute::new("hidden", true, None, false));
        base.push(Attribute::new("aria-hidden", "true", None, false));
    }
    push_attr_opt(&mut base, "style", base_style);
    if let CommandItemKind::Link {
        ref href,
        ref target,
        ref rel,
    } = spec.kind
    {
        push_attr_opt(&mut base, "href", href.clone());
        push_attr_opt(&mut base, "target", target.clone());
        push_attr_opt(&mut base, "rel", rel.clone());
    }
    push_event_handler(&mut base, "onmousedown", Some(handle_mousedown));
    push_event_handler(&mut base, "onpointermove", Some(handle_pointer_move));
    push_event_handler(&mut base, "onclick", Some(handle_click));

    let mut user = collect_user_attrs(spec.class, spec.style, spec.attributes);
    push_event_handler(&mut user, "onpointermove", spec.on_pointer_move);
    push_event_handler(&mut user, "onmounted", spec.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = spec.child {
        if !should_render {
            return rsx! {};
        }
        return child.call(merged);
    }

    match spec.kind {
        CommandItemKind::Div => rsx! {
            div {
                ..merged,
                {spec.children}
            }
        },
        CommandItemKind::Link { .. } => rsx! {
            a {
                ..merged,
                {spec.children}
            }
        },
    }
}

#[component]
pub fn CommandItem(props: CommandItemProps) -> Element {
    render_command_item_surface(CommandItemSurfaceSpec {
        value: props.value,
        id: props.id,
        disabled: props.disabled,
        force_mount: props.force_mount,
        keywords: props.keywords,
        on_select: props.on_select,
        class: props.class,
        style: props.style,
        child: props.child,
        on_click: props.on_click,
        on_pointer_move: props.on_pointer_move,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
        kind: CommandItemKind::Div,
    })
}

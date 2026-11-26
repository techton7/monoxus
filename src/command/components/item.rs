use dioxus::prelude::*;

use crate::foundation::{
    browser::is_element_focused_by_id,
    compose::merge_global_attributes,
    shared::use_stable_id,
};

use super::super::{
    attrs::{CommandItemAttrs, merge_ranked_item_style},
    runtime::{CommandContext, CommandGroupContext},
};
use super::link_item::CommandLinkItemRenderProps;

#[derive(Clone, PartialEq, Props)]
pub struct CommandItemRenderProps {
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
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_mousedown: EventHandler<MouseEvent>,
    pub on_pointer_move: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandItemProps {
    pub value: String,
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
    pub render_child: Option<Callback<CommandItemRenderProps, Element>>,
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
    Div {
        render_child: Option<Callback<CommandItemRenderProps, Element>>,
    },
    Link {
        href: Option<String>,
        target: Option<String>,
        rel: Option<String>,
        render_child: Option<Callback<CommandLinkItemRenderProps, Element>>,
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
    pub as_child: bool,
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
    let rendered_style =
        merge_ranked_item_style(should_render, visual_order, spec.style.as_deref());

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
    let mut on_pointer_move_slot = use_hook(|| CopyValue::new(spec.on_pointer_move));
    on_pointer_move_slot.set(spec.on_pointer_move);
    let mut on_mounted_slot = use_hook(|| CopyValue::new(spec.on_mounted));
    on_mounted_slot.set(spec.on_mounted);
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
    let handle_pointer_move = EventHandler::new(move |evt: PointerEvent| {
        if let Some(handler) = on_pointer_move_slot.try_read().ok().and_then(|s| *s) {
            handler.call(evt);
        }
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

    match spec.kind {
        CommandItemKind::Div { render_child } => {
            if spec.as_child {
                if !should_render {
                    return rsx! {};
                }
                if let Some(render_child) = render_child.as_ref() {
                    return render_child.call(CommandItemRenderProps {
                        id: attrs.id,
                        role: attrs.role,
                        aria_disabled: attrs.aria_disabled,
                        aria_selected: attrs.aria_selected,
                        data_command_item: attrs.data_command_item,
                        data_value: attrs.data_value,
                        data_group: attrs.data_group,
                        data_disabled: attrs.data_disabled,
                        data_selected: attrs.data_selected,
                        class: spec.class,
                        style: rendered_style,
                        on_mousedown: handle_mousedown,
                        on_pointer_move: handle_pointer_move,
                        on_click: handle_click,
                        on_mounted: spec.on_mounted,
                        attributes: spec.attributes,
                        children: spec.children,
                    });
                }
            }

            let merged = merge_global_attributes(
                spec.class.as_deref(),
                rendered_style.as_deref(),
                spec.attributes,
            );

            rsx! {
                div {
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
                    class: merged.class,
                    style: merged.style,
                    onmousedown: move |evt| handle_mousedown.call(evt),
                    onpointermove: move |evt| handle_pointer_move.call(evt),
                    onclick: move |evt| handle_click.call(evt),
                    onmounted: move |evt| {
                        if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                            handler.call(evt);
                        }
                    },
                    ..merged.rest,
                    {spec.children}
                }
            }
        }
        CommandItemKind::Link {
            href,
            target,
            rel,
            render_child,
        } => {
            if spec.as_child {
                if !should_render {
                    return rsx! {};
                }
                if let Some(render_child) = render_child.as_ref() {
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
                        href,
                        target,
                        rel,
                        class: spec.class,
                        style: rendered_style,
                        on_mousedown: handle_mousedown,
                        on_pointer_move: handle_pointer_move,
                        on_click: handle_click,
                        on_mounted: spec.on_mounted,
                        attributes: spec.attributes,
                        children: spec.children,
                    });
                }
            }

            let merged = merge_global_attributes(
                spec.class.as_deref(),
                rendered_style.as_deref(),
                spec.attributes,
            );

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
                    href: href.as_deref(),
                    target: target.as_deref(),
                    rel: rel.as_deref(),
                    class: merged.class,
                    style: merged.style,
                    onmousedown: move |evt| handle_mousedown.call(evt),
                    onpointermove: move |evt| handle_pointer_move.call(evt),
                    onclick: move |evt| handle_click.call(evt),
                    onmounted: move |evt| {
                        if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                            handler.call(evt);
                        }
                    },
                    ..merged.rest,
                    {spec.children}
                }
            }
        }
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
        as_child: props.as_child,
        on_click: props.on_click,
        on_pointer_move: props.on_pointer_move,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
        kind: CommandItemKind::Div {
            render_child: props.render_child,
        },
    })
}

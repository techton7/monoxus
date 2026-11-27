use dioxus::prelude::*;

use crate::foundation::{
    browser::spawn_root_task,
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    overlay::{use_presence, use_watcher_slot},
};

use super::{
    super::{
        runtime::{
            NavigationMenuContext, measure_viewport_offset_size, start_offset_resize_observer,
        },
        types::NavigationMenuViewportSize,
    },
    content::{NavigationMenuContentImplSpec, render_navigation_menu_content_impl},
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuViewportProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
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
pub fn NavigationMenuViewport(props: NavigationMenuViewportProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    let viewport_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.viewport_id().to_owned());

    ctx.register_viewport(viewport_id.clone());
    let drop_ctx = ctx.clone();
    let drop_vp_id = viewport_id.clone();
    use_drop(move || {
        drop_ctx.unregister_viewport(&drop_vp_id);
    });

    let current_val = ctx.current_value();
    let previous_val = ctx.previous_value_str();
    let is_open = !current_val.is_empty();
    let desired_present = props.force_mount || is_open;

    let mut size = use_signal(|| None::<NavigationMenuViewportSize>);
    let on_presence_complete = use_hook(move || {
        EventHandler::new(move |open: bool| {
            if !open
                && size.try_peek().is_ok_and(|s| s.is_some())
                && let Ok(mut w) = size.try_write()
            {
                *w = None;
            }
        })
    });
    let presence = use_presence(
        desired_present,
        &viewport_id,
        true,
        Some(on_presence_complete),
    );
    let resize_guard = use_watcher_slot();

    let should_render = desired_present || presence.should_render();
    let active_content_value = if is_open {
        current_val
    } else {
        previous_val
    };

    let entries = ctx.viewport_contents.read().clone();
    let active_content_id = if !active_content_value.is_empty() {
        entries
            .iter()
            .find(|(v, _)| v == &active_content_value)
            .map(|(_, data)| data.content_id.clone())
    } else {
        None
    };

    let ctx_alive = ctx.clone();
    use_effect(use_reactive(
        (&should_render, &active_content_id),
        move |(render, cid_opt)| {
            resize_guard.clear();
            if !render {
                if size.try_peek().is_ok_and(|s| s.is_some())
                    && let Ok(mut w) = size.try_write()
                {
                    *w = None;
                }
                return;
            }
            let Some(cid) = cid_opt else {
                return;
            };
            let mut measure_now = {
                let cid = cid.clone();
                move || {
                    if let Some(measured) = measure_viewport_offset_size(&cid)
                        && (measured.width > 0.0 || measured.height > 0.0)
                        && size.try_peek().is_ok_and(|cur| *cur != Some(measured))
                        && let Ok(mut w) = size.try_write()
                    {
                        *w = Some(measured);
                    }
                }
            };
            measure_now();

            let mut measure_next = measure_now.clone();
            let alive_ref = ctx_alive.clone();
            spawn_root_task(async move {
                let _ = oxidase::frame::next_frame().await;
                if alive_ref.is_alive() {
                    measure_next();
                }
            });

            resize_guard.replace(start_offset_resize_observer(&cid, measure_now));
        },
    ));

    if !should_render {
        return rsx! {};
    }

    let vp_attrs = ctx.viewport_attributes(is_open, *size.read(), Some(viewport_id), None);

    let on_pointer_enter = {
        let ctx = ctx.clone();
        EventHandler::new(move |_: PointerEvent| {
            ctx.on_content_enter();
        })
    };

    let on_pointer_leave = {
        let ctx = ctx.clone();
        EventHandler::new(move |evt: PointerEvent| {
            if evt.pointer_type() != "mouse" {
                return;
            }
            if ctx.active_item_open_on_hover() {
                ctx.on_content_leave(ctx.active_item_open_on_hover_opt());
            }
        })
    };

    let base_style = {
        let s = vp_attrs.style();
        (!s.trim().is_empty()).then(|| s.to_owned())
    };

    let mut base = Vec::with_capacity(10);
    base.push(Attribute::new("id", vp_attrs.id().to_owned(), None, false));
    push_attr_opt(&mut base, "style", base_style);
    base.push(Attribute::new(
        "data-state",
        vp_attrs.data_state().as_str(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        vp_attrs.data_orientation(),
        None,
        false,
    ));
    push_attr_opt(
        &mut base,
        "data-starting-style",
        presence.data_starting_style(),
    );
    push_attr_opt(&mut base, "data-ending-style", presence.data_ending_style());
    base.push(Attribute::new(
        "data-monoxus-navigation-menu-viewport",
        vp_attrs.data_monoxus_navigation_menu_viewport(),
        None,
        false,
    ));
    push_event_handler::<PointerData>(&mut base, "onpointerenter", Some(on_pointer_enter));
    push_event_handler::<PointerData>(&mut base, "onpointerleave", Some(on_pointer_leave));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler::<MountedData>(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    let combined_children = rsx! {
        {props.children}
        for (val, _) in entries.into_iter() {
            NavigationMenuViewportItem {
                key: "{val}",
                value: val.clone(),
                is_item_open: is_open && active_content_value == val,
                is_active: active_content_value == val,
            }
        }
    };

    rsx! {
        div {
            ..merged,
            {combined_children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct NavigationMenuViewportItemProps {
    value: String,
    is_active: bool,
    is_item_open: bool,
}

#[component]
fn NavigationMenuViewportItem(props: NavigationMenuViewportItemProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    let entry_opt = ctx
        .viewport_contents
        .read()
        .iter()
        .find(|(v, _)| v == &props.value)
        .map(|(_, data)| data.clone());

    let content_id = entry_opt
        .as_ref()
        .map(|e| e.content_id.clone())
        .unwrap_or_else(|| ctx.relationships.content_id(&props.value));
    let desired_present = entry_opt
        .as_ref()
        .is_some_and(|e| e.force_mount || props.is_active);
    let presence = use_presence(desired_present, &content_id, true, None);

    let Some(entry) = entry_opt else {
        return rsx! {};
    };
    let active = desired_present || presence.should_render();

    render_navigation_menu_content_impl(
        &ctx,
        NavigationMenuContentImplSpec {
            active,
            value: entry.value,
            content_id: entry.content_id,
            trigger_id: entry.trigger_id,
            focus_proxy_id: entry.focus_proxy_id,
            item_state: entry.item_state,
            is_open: props.is_item_open,
            inline_mode: false,
            data_starting_style: presence.data_starting_style(),
            data_ending_style: presence.data_ending_style(),
            class: entry.class,
            style: entry.style,
            on_escape_key_down: entry.on_escape_key_down,
            on_pointer_down_outside: entry.on_pointer_down_outside,
            on_focus_outside: entry.on_focus_outside,
            on_interact_outside: entry.on_interact_outside,
            escape_keydown_behavior: entry.escape_keydown_behavior,
            interact_outside_behavior: entry.interact_outside_behavior,
            child: entry.child,
            on_mounted: entry.on_mounted,
            attributes: entry.attributes,
            children: entry.children,
        },
    )
}

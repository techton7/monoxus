use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    browser::{WatcherGuard, spawn_root_task},
    overlay::PresenceController,
};

use super::{
    super::{
        runtime::{
            NavigationMenuContext, NavigationMenuViewportRenderProps, ViewportContentData,
            measure_viewport_offset_size, start_offset_resize_observer, sync_presence_lifecycle,
        },
        types::NavigationMenuViewportSize,
    },
    content::NavigationMenuContentImpl,
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuViewportProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<NavigationMenuViewportRenderProps, Element>>,
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

    let presence =
        use_signal(|| PresenceController::new(desired_present).with_retained_mount(true));
    let presence_guard = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));
    let resize_guard = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));
    let mut size = use_signal(|| None::<NavigationMenuViewportSize>);

    let vp_id_for_presence = viewport_id.clone();
    let pg_for_effect = Rc::clone(&presence_guard);
    use_effect(use_reactive((&desired_present,), move |(present,)| {
        sync_presence_lifecycle(&vp_id_for_presence, present, presence, &pg_for_effect, || {
            if size.try_peek().is_ok_and(|s| s.is_some())
                && let Ok(mut w) = size.try_write()
            {
                *w = None;
            }
        });
    }));

    let should_render = desired_present || presence.read().should_render();
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

    let rg_for_effect = Rc::clone(&resize_guard);
    let ctx_alive = ctx.clone();
    use_effect(use_reactive(
        (&should_render, &active_content_id),
        move |(render, cid_opt)| {
            *rg_for_effect.borrow_mut() = None;
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

            *rg_for_effect.borrow_mut() = Some(start_offset_resize_observer(&cid, measure_now));
        },
    ));

    use_drop({
        let pg = Rc::clone(&presence_guard);
        let rg = Rc::clone(&resize_guard);
        move || {
            *pg.borrow_mut() = None;
            *rg.borrow_mut() = None;
        }
    });

    if !should_render {
        return rsx! {};
    }

    let vp_attrs = ctx.viewport_attributes(
        is_open,
        *size.read(),
        Some(viewport_id),
        props.style.as_deref(),
    );

    let on_pointer_enter = {
        let ctx = ctx.clone();
        move |_: Event<PointerData>| {
            ctx.on_content_enter();
        }
    };

    let on_pointer_leave = {
        let ctx = ctx.clone();
        move |evt: Event<PointerData>| {
            if evt.pointer_type() != "mouse" {
                return;
            }
            if ctx.active_item_open_on_hover() {
                ctx.on_content_leave(ctx.active_item_open_on_hover_opt());
            }
        }
    };

    let combined_children = rsx! {
        {props.children}
        for (val, entry) in entries.into_iter() {
            NavigationMenuViewportItem {
                key: "{val}",
                is_item_open: is_open && active_content_value == val,
                is_active: active_content_value == val,
                entry,
            }
        }
    };

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(NavigationMenuViewportRenderProps {
                attrs: vp_attrs,
                is_open,
                class: props.class,
                style: props.style,
                attributes: props.attributes,
                children: combined_children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::NavigationMenuViewport: as_child requires render_child; rendering the default div"
        );
    }

    rsx! {
        div {
            id: "{vp_attrs.id()}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{vp_attrs.style()}",
            "data-state": vp_attrs.data_state().as_str(),
            "data-orientation": "{vp_attrs.data_orientation()}",
            "data-monoxus-navigation-menu-viewport": "{vp_attrs.data_monoxus_navigation_menu_viewport()}",
            onpointerenter: on_pointer_enter,
            onpointerleave: on_pointer_leave,
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {combined_children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct NavigationMenuViewportItemProps {
    entry: ViewportContentData,
    is_active: bool,
    is_item_open: bool,
}

#[component]
fn NavigationMenuViewportItem(props: NavigationMenuViewportItemProps) -> Element {
    let entry = props.entry;
    let desired_present = entry.force_mount || props.is_active;
    let presence =
        use_signal(|| PresenceController::new(desired_present).with_retained_mount(true));
    let presence_guard = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));

    let cid = entry.content_id.clone();
    let pg = Rc::clone(&presence_guard);
    use_effect(use_reactive((&desired_present,), move |(present,)| {
        sync_presence_lifecycle(&cid, present, presence, &pg, || {});
    }));
    use_drop({
        let pg = Rc::clone(&presence_guard);
        move || {
            *pg.borrow_mut() = None;
        }
    });

    if !desired_present && !presence.read().should_render() {
        return rsx! {};
    }

    rsx! {
        NavigationMenuContentImpl {
            value: entry.value,
            content_id: entry.content_id,
            trigger_id: entry.trigger_id,
            focus_proxy_id: entry.focus_proxy_id,
            item_state: entry.item_state,
            is_open: props.is_item_open,
            inline_mode: false,
            class: entry.class,
            style: entry.style,
            on_escape_key_down: entry.on_escape_key_down,
            on_pointer_down_outside: entry.on_pointer_down_outside,
            on_focus_outside: entry.on_focus_outside,
            on_interact_outside: entry.on_interact_outside,
            escape_keydown_behavior: entry.escape_keydown_behavior,
            interact_outside_behavior: entry.interact_outside_behavior,
            as_child: entry.as_child,
            render_child: entry.render_child,
            on_mounted: entry.on_mounted,
            attributes: entry.attributes,
            children: entry.children,
        }
    }
}

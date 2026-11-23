use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    browser::{WatcherGuard, spawn_root_task},
    overlay::PresenceController,
};

use super::super::{
    runtime::{
        IndicatorMountData, NavigationMenuContext, NavigationMenuIndicatorRenderProps,
        measure_indicator_offset_position, start_offset_resize_observer, sync_presence_lifecycle,
    },
    types::NavigationMenuIndicatorPosition,
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuIndicatorProps {
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
    pub render_child: Option<Callback<NavigationMenuIndicatorRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn NavigationMenuIndicator(props: NavigationMenuIndicatorProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();

    ctx.register_indicator(IndicatorMountData {
        id: props.id,
        class: props.class,
        style: props.style,
        force_mount: props.force_mount,
        as_child: props.as_child,
        render_child: props.render_child,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
    });

    let cleanup_ctx = ctx.clone();
    use_drop(move || {
        cleanup_ctx.unregister_indicator();
    });

    rsx! {}
}

#[component]
pub fn NavigationMenuIndicatorSlot() -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    let data_opt = ctx.indicator_config.read().clone();

    let current_val = ctx.current_value();
    let is_visible = !current_val.is_empty();
    let desired_present = data_opt
        .as_ref()
        .is_some_and(|data| data.force_mount || is_visible);

    let presence =
        use_signal(|| PresenceController::new(desired_present).with_retained_mount(true));
    let presence_guard = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));
    let trigger_resize_guard = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));
    let track_resize_guard = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));

    let mut position = use_signal(|| None::<NavigationMenuIndicatorPosition>);
    let mut last_trigger_id = use_signal(|| None::<String>);

    let active_trigger_id = if is_visible {
        ctx.trigger_id_for(&current_val)
    } else {
        None
    };
    if let Some(ref tid) = active_trigger_id
        && last_trigger_id.peek().as_deref() != Some(tid.as_str())
    {
        last_trigger_id.set(Some(tid.clone()));
    }

    let track_id = ctx.track_id.read().clone();
    let orientation = ctx.current_orientation();
    let indicator_id = data_opt
        .as_ref()
        .and_then(|d| d.id.clone())
        .unwrap_or_else(|| ctx.relationships.indicator_id().to_owned());

    let presence_id = indicator_id.clone();
    let pg_for_effect = Rc::clone(&presence_guard);
    use_effect(use_reactive((&desired_present,), move |(present,)| {
        sync_presence_lifecycle(&presence_id, present, presence, &pg_for_effect, || {
            if position.try_peek().is_ok_and(|p| p.is_some())
                && let Ok(mut w) = position.try_write()
            {
                *w = None;
            }
        });
    }));

    let should_render = data_opt.is_some() && (desired_present || presence.read().should_render());
    let tracked_tid = active_trigger_id.or_else(|| last_trigger_id.read().clone());

    let trg_cell = Rc::clone(&trigger_resize_guard);
    let trk_cell = Rc::clone(&track_resize_guard);
    let ctx_alive = ctx.clone();
    use_effect(use_reactive(
        (&should_render, &tracked_tid, &track_id, &orientation),
        move |(render, tid_opt, track_opt, ori)| {
            *trg_cell.borrow_mut() = None;
            *trk_cell.borrow_mut() = None;
            if !render {
                return;
            }
            let Some(tid) = tid_opt else {
                return;
            };

            let mut measure_now = {
                let tid = tid.clone();
                move || {
                    if let Some(measured) = measure_indicator_offset_position(&tid, ori)
                        && position.try_peek().is_ok_and(|cur| *cur != Some(measured))
                        && let Ok(mut w) = position.try_write()
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

            let m1 = measure_now.clone();
            *trg_cell.borrow_mut() = Some(start_offset_resize_observer(&tid, m1));
            if let Some(tr_id) = track_opt {
                *trk_cell.borrow_mut() = Some(start_offset_resize_observer(&tr_id, measure_now));
            }
        },
    ));

    use_drop({
        let pg = Rc::clone(&presence_guard);
        let trg = Rc::clone(&trigger_resize_guard);
        let trk = Rc::clone(&track_resize_guard);
        move || {
            *pg.borrow_mut() = None;
            *trg.borrow_mut() = None;
            *trk.borrow_mut() = None;
        }
    });

    let Some(data) = data_opt else {
        return rsx! {};
    };

    if !should_render {
        return rsx! {};
    }

    let Some(pos) = *position.read() else {
        return rsx! {};
    };

    let attrs =
        ctx.indicator_attributes(is_visible, pos, Some(indicator_id), data.style.as_deref());

    if data.as_child {
        if let Some(render_child) = data.render_child.as_ref() {
            return render_child.call(NavigationMenuIndicatorRenderProps {
                attrs,
                is_visible,
                class: data.class,
                style: data.style,
                attributes: data.attributes,
                children: data.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::NavigationMenuIndicator: as_child requires render_child; rendering the default div"
        );
    }

    rsx! {
        div {
            id: "{attrs.id()}",
            class: data.class.as_deref().unwrap_or_default(),
            style: "{attrs.style()}",
            aria_hidden: "{attrs.aria_hidden()}",
            "data-state": "{attrs.data_state()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-monoxus-navigation-menu-indicator": "{attrs.data_monoxus_navigation_menu_indicator()}",
            onmounted: move |evt| {
                if let Some(ref handler) = data.on_mounted {
                    handler.call(evt);
                }
            },
            ..data.attributes,
            {data.children}
        }
    }
}

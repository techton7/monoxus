use dioxus::prelude::*;

use crate::foundation::{
    browser::spawn_root_task,
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    overlay::{use_presence, use_watcher_slot},
};

use super::super::{
    runtime::{
        IndicatorMountData, NavigationMenuContext, measure_indicator_offset_position,
        start_offset_resize_observer,
    },
    types::NavigationMenuIndicatorPosition,
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuIndicatorProps {
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
pub fn NavigationMenuIndicator(props: NavigationMenuIndicatorProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();

    ctx.register_indicator(IndicatorMountData {
        id: props.id,
        class: props.class,
        style: props.style,
        force_mount: props.force_mount,
        child: props.child,
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
pub(crate) fn NavigationMenuIndicatorSlot() -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    let data_opt = ctx.indicator_config.read().clone();

    let current_val = ctx.current_value();
    let is_visible = !current_val.is_empty();
    let desired_present = data_opt
        .as_ref()
        .is_some_and(|data| data.force_mount || is_visible);

    let trigger_resize_guard = use_watcher_slot();
    let track_resize_guard = use_watcher_slot();

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

    let on_presence_complete = use_hook(move || {
        EventHandler::new(move |open: bool| {
            if !open
                && position.try_peek().is_ok_and(|p| p.is_some())
                && let Ok(mut w) = position.try_write()
            {
                *w = None;
            }
        })
    });
    let presence = use_presence(
        desired_present,
        &indicator_id,
        true,
        Some(on_presence_complete),
    );

    let should_render = data_opt.is_some() && (desired_present || presence.should_render());
    let tracked_tid = active_trigger_id.or_else(|| last_trigger_id.read().clone());

    let ctx_alive = ctx.clone();
    use_effect(use_reactive(
        (&should_render, &tracked_tid, &track_id, &orientation),
        move |(render, tid_opt, track_opt, ori)| {
            trigger_resize_guard.clear();
            track_resize_guard.clear();
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
            trigger_resize_guard.replace(start_offset_resize_observer(&tid, m1));
            if let Some(tr_id) = track_opt {
                track_resize_guard.replace(start_offset_resize_observer(&tr_id, measure_now));
            }
        },
    ));

    let Some(data) = data_opt else {
        return rsx! {};
    };

    if !should_render {
        return rsx! {};
    }

    let Some(pos) = *position.read() else {
        return rsx! {};
    };

    let attrs = ctx.indicator_attributes(is_visible, pos, Some(indicator_id), None);
    let base_style = {
        let s = attrs.style();
        (!s.trim().is_empty()).then(|| s.to_owned())
    };

    let mut base = Vec::with_capacity(9);
    base.push(Attribute::new("id", attrs.id().to_owned(), None, false));
    push_attr_opt(&mut base, "style", base_style);
    base.push(Attribute::new("aria-hidden", attrs.aria_hidden(), None, false));
    base.push(Attribute::new("data-state", attrs.data_state(), None, false));
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
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
        "data-monoxus-navigation-menu-indicator",
        attrs.data_monoxus_navigation_menu_indicator(),
        None,
        false,
    ));

    let mut user = collect_user_attrs(data.class, data.style, data.attributes);
    push_event_handler::<MountedData>(&mut user, "onmounted", data.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = data.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {data.children}
        }
    }
}

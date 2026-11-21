use std::cell::RefCell;
use std::rc::Rc;

use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        ElementSize, PresenceMonitorEvent, ResizeObserverBox, WatcherGuard,
        get_element_client_rect, get_element_scroll_metrics, restore_body_user_select,
        set_body_user_select_none, set_element_scroll_position, start_presence_monitor,
        start_resize_monitor,
    },
    overlay::PresenceController,
    shared::{Direction, Orientation},
};

use super::{
    super::{
        attrs::ScrollAreaScrollbarAttributes,
        runtime::{
            forward_wheel_to_viewport, try_release_pointer_capture, try_set_pointer_capture,
        },
        state::{
            get_scroll_position_from_pointer, get_thumb_size, is_overflowing, is_scrollbar_visible,
            transition_scroll_state,
        },
        types::{ScrollAreaSizes, ScrollAreaStateEvent, ScrollAreaType, ScrollbarPadding},
    },
    ScrollAreaContext, ScrollbarContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct ScrollAreaScrollbarProps {
    #[props(default)]
    pub id: Option<String>,
    pub orientation: Orientation,
    #[props(default = false)]
    pub force_mount: bool,
    pub as_child: Option<bool>,
    pub children: Element,
    pub class: Option<String>,
    pub style: Option<String>,
}

#[component]
pub fn ScrollAreaScrollbar(props: ScrollAreaScrollbarProps) -> Element {
    let ctx = use_context::<ScrollAreaContext>();
    let orientation = props.orientation;

    let scrollbar_id = props.id.clone().unwrap_or_else(|| {
        ctx.relationships
            .read()
            .scrollbar_id(orientation)
            .to_string()
    });
    let thumb_id_resolved = ctx.relationships.read().thumb_id(orientation).to_string();
    let viewport_id = ctx.relationships.read().viewport_id().to_string();

    use_effect(use_reactive((&scrollbar_id,), {
        let mut rels = ctx.relationships;
        move |(sid,)| {
            let cur = rels.peek().clone();
            let needs_update = match orientation {
                Orientation::Horizontal => cur.scrollbar_x_id() != sid,
                Orientation::Vertical | Orientation::Both => cur.scrollbar_y_id() != sid,
            };
            if needs_update {
                let updated = match orientation {
                    Orientation::Horizontal => cur.with_scrollbar_x_id(sid),
                    Orientation::Vertical | Orientation::Both => cur.with_scrollbar_y_id(sid),
                };
                rels.set(updated);
            }
        }
    }));

    let mounted_el = use_hook(|| Rc::new(RefCell::new(None::<oxidase::Element>)));
    let captured_pid = use_hook(|| Rc::new(RefCell::new(None::<i32>)));
    let is_dragging = use_signal(|| false);
    let pointer_offset = use_signal(|| 0.0);
    let thumb_pointer_active = use_signal(|| false);

    let initial_sizes = {
        let metrics = ctx.runtime.viewport_metrics();
        let (content, viewport) = if let Some(m) = metrics.as_ref() {
            match orientation {
                Orientation::Horizontal => (m.scroll_width, m.client_width),
                Orientation::Vertical | Orientation::Both => (m.scroll_height, m.client_height),
            }
        } else {
            (0.0, 0.0)
        };
        ScrollAreaSizes::new(content, viewport, 0.0, ScrollbarPadding::default())
    };
    let sizes = use_signal(|| initial_sizes);

    use_effect({
        let mut sizes = sizes;
        let metrics_signal = ctx.runtime.viewport_metrics;
        move || {
            if let Some(metrics) = metrics_signal() {
                let (content, viewport) = match orientation {
                    Orientation::Horizontal => (metrics.scroll_width, metrics.client_width),
                    Orientation::Vertical | Orientation::Both => {
                        (metrics.scroll_height, metrics.client_height)
                    }
                };
                let current = *sizes.peek();
                if current.content != content || current.viewport != viewport {
                    sizes.set(current.with_content(content).with_viewport(viewport));
                }
            }
        }
    });

    use_effect({
        let mut runtime = ctx.runtime;
        move || match orientation {
            Orientation::Horizontal => runtime.scrollbar_x_enabled.set(true),
            Orientation::Vertical => runtime.scrollbar_y_enabled.set(true),
            Orientation::Both => {
                runtime.scrollbar_x_enabled.set(true);
                runtime.scrollbar_y_enabled.set(true);
            }
        }
    });

    let current_sizes = sizes();
    let overflowing = is_overflowing(current_sizes);
    let scroll_state = ctx.runtime.scroll_state();
    let hover_visible = ctx.runtime.hover_visible();

    let is_visible = is_scrollbar_visible(ctx.r#type, scroll_state, hover_visible, overflowing);

    let presence = use_hook(|| {
        Rc::new(RefCell::new(
            PresenceController::new(is_visible).with_retained_mount(true),
        ))
    });
    let mut presence_epoch = use_signal(|| 0u64);
    let presence_monitor = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));
    let _ = presence_epoch();

    let needs_sync = presence.borrow().desired_present() != is_visible;
    if needs_sync {
        let update = presence.borrow_mut().sync(is_visible);
        if let Some(cycle_id) = update.started_close_cycle() {
            let presence_cb = Rc::clone(&presence);
            let watcher =
                start_presence_monitor(scrollbar_id.as_str(), cycle_id, move |event| match event {
                    PresenceMonitorEvent::TransitionEnd {
                        cycle_id: event_cycle,
                        ..
                    }
                    | PresenceMonitorEvent::TransitionCancel {
                        cycle_id: event_cycle,
                        ..
                    }
                    | PresenceMonitorEvent::AnimationEnd {
                        cycle_id: event_cycle,
                        ..
                    }
                    | PresenceMonitorEvent::AnimationCancel {
                        cycle_id: event_cycle,
                        ..
                    }
                    | PresenceMonitorEvent::Fallback {
                        cycle_id: event_cycle,
                        ..
                    } => {
                        if presence_cb.borrow_mut().complete_close_cycle(event_cycle) {
                            if let Ok(mut epoch) = presence_epoch.try_write() {
                                *epoch += 1;
                            }
                        }
                    }
                    _ => {}
                });
            *presence_monitor.borrow_mut() = Some(watcher);
        } else if !update.should_render() || update.invalidated_close_cycle().is_some() {
            *presence_monitor.borrow_mut() = None;
        }
    }

    let should_render = props.force_mount || presence.borrow().should_render();

    let resize_guard_cell = use_hook(|| Rc::new(RefCell::new(WatcherGuard::noop())));

    use_effect(use_reactive(
        (&should_render, &scrollbar_id, &viewport_id),
        {
            let resize_guard_cell = Rc::clone(&resize_guard_cell);
            let mut sizes = sizes;
            let mut runtime = ctx.runtime;
            move |(rendered, sid, vid)| {
                if rendered {
                    if let Some(rect) = get_element_client_rect(&sid) {
                        let track_size = match orientation {
                            Orientation::Horizontal => rect.width,
                            Orientation::Vertical | Orientation::Both => rect.height,
                        };
                        if track_size > 0.0 && sizes.peek().scrollbar_size != track_size {
                            sizes.write().scrollbar_size = track_size;
                        }
                        match orientation {
                            Orientation::Horizontal => {
                                if rect.height > 0.0 && *runtime.corner_height.peek() != rect.height
                                {
                                    runtime.corner_height.set(rect.height);
                                }
                            }
                            Orientation::Vertical => {
                                if rect.width > 0.0 && *runtime.corner_width.peek() != rect.width {
                                    runtime.corner_width.set(rect.width);
                                }
                            }
                            Orientation::Both => {
                                if rect.height > 0.0 && *runtime.corner_height.peek() != rect.height
                                {
                                    runtime.corner_height.set(rect.height);
                                }
                                if rect.width > 0.0 && *runtime.corner_width.peek() != rect.width {
                                    runtime.corner_width.set(rect.width);
                                }
                            }
                        }
                    }

                    {
                        let sid_shot = sid.clone();
                        let vid_shot = vid.clone();
                        let mut s = sizes;
                        let mut rt = runtime;
                        spawn(async move {
                            oxidase::frame::next_frame().await;
                            if let Some(rect) = get_element_client_rect(&sid_shot) {
                                let track_size = match orientation {
                                    Orientation::Horizontal => rect.width,
                                    Orientation::Vertical | Orientation::Both => rect.height,
                                };
                                if track_size > 0.0 && s.peek().scrollbar_size != track_size {
                                    s.write().scrollbar_size = track_size;
                                }
                                match orientation {
                                    Orientation::Horizontal => {
                                        if rect.height > 0.0
                                            && *rt.corner_height.peek() != rect.height
                                        {
                                            rt.corner_height.set(rect.height);
                                        }
                                    }
                                    Orientation::Vertical => {
                                        if rect.width > 0.0 && *rt.corner_width.peek() != rect.width
                                        {
                                            rt.corner_width.set(rect.width);
                                        }
                                    }
                                    Orientation::Both => {
                                        if rect.height > 0.0
                                            && *rt.corner_height.peek() != rect.height
                                        {
                                            rt.corner_height.set(rect.height);
                                        }
                                        if rect.width > 0.0 && *rt.corner_width.peek() != rect.width
                                        {
                                            rt.corner_width.set(rect.width);
                                        }
                                    }
                                }
                            }
                            if let Some(m) = get_element_scroll_metrics(&vid_shot) {
                                let (content, viewport) = match orientation {
                                    Orientation::Horizontal => (m.scroll_width, m.client_width),
                                    Orientation::Vertical | Orientation::Both => {
                                        (m.scroll_height, m.client_height)
                                    }
                                };
                                let cur = *s.peek();
                                if cur.content != content || cur.viewport != viewport {
                                    s.set(cur.with_content(content).with_viewport(viewport));
                                }
                            }
                        });
                    }

                    let mut s = sizes;
                    let mut rt = runtime;
                    let next_guard = start_resize_monitor(
                        &sid,
                        ResizeObserverBox::BorderBox,
                        move |size: ElementSize| {
                            let track_size = match orientation {
                                Orientation::Horizontal => size.width,
                                Orientation::Vertical | Orientation::Both => size.height,
                            };
                            if s.peek().scrollbar_size != track_size {
                                s.write().scrollbar_size = track_size;
                            }

                            match orientation {
                                Orientation::Horizontal => {
                                    if *rt.corner_height.peek() != size.height {
                                        rt.corner_height.set(size.height);
                                    }
                                }
                                Orientation::Vertical => {
                                    if *rt.corner_width.peek() != size.width {
                                        rt.corner_width.set(size.width);
                                    }
                                }
                                Orientation::Both => {
                                    if *rt.corner_height.peek() != size.height {
                                        rt.corner_height.set(size.height);
                                    }
                                    if *rt.corner_width.peek() != size.width {
                                        rt.corner_width.set(size.width);
                                    }
                                }
                            }
                        },
                    );
                    let prev = std::mem::replace(&mut *resize_guard_cell.borrow_mut(), next_guard);
                    prev.stop();
                } else {
                    std::mem::replace(&mut *resize_guard_cell.borrow_mut(), WatcherGuard::noop())
                        .stop();
                    match orientation {
                        Orientation::Horizontal => {
                            if *runtime.corner_height.peek() != 0.0 {
                                runtime.corner_height.set(0.0);
                            }
                        }
                        Orientation::Vertical => {
                            if *runtime.corner_width.peek() != 0.0 {
                                runtime.corner_width.set(0.0);
                            }
                        }
                        Orientation::Both => {
                            if *runtime.corner_height.peek() != 0.0 {
                                runtime.corner_height.set(0.0);
                            }
                            if *runtime.corner_width.peek() != 0.0 {
                                runtime.corner_width.set(0.0);
                            }
                        }
                    }
                }
            }
        },
    ));

    use_drop({
        let resize_guard = Rc::clone(&resize_guard_cell);
        let presence_monitor = Rc::clone(&presence_monitor);
        let is_dragging = is_dragging;
        let mut runtime = ctx.runtime;
        move || {
            std::mem::replace(&mut *resize_guard.borrow_mut(), WatcherGuard::noop()).stop();
            *presence_monitor.borrow_mut() = None;
            if *is_dragging.peek() {
                restore_body_user_select();
            }
            match orientation {
                Orientation::Horizontal => {
                    runtime.scrollbar_x_enabled.set(false);
                    if *runtime.corner_height.peek() != 0.0 {
                        runtime.corner_height.set(0.0);
                    }
                }
                Orientation::Vertical => {
                    runtime.scrollbar_y_enabled.set(false);
                    if *runtime.corner_width.peek() != 0.0 {
                        runtime.corner_width.set(0.0);
                    }
                }
                Orientation::Both => {
                    runtime.scrollbar_x_enabled.set(false);
                    runtime.scrollbar_y_enabled.set(false);
                    if *runtime.corner_height.peek() != 0.0 {
                        runtime.corner_height.set(0.0);
                    }
                    if *runtime.corner_width.peek() != 0.0 {
                        runtime.corner_width.set(0.0);
                    }
                }
            }
        }
    });

    let scrollbar_ctx = ScrollbarContext {
        orientation,
        scrollbar_id: use_signal(|| scrollbar_id.clone()),
        thumb_id: use_signal(|| thumb_id_resolved.clone()),
        sizes,
        pointer_offset,
        is_dragging,
        thumb_pointer_active,
        force_mount: props.force_mount,
    };
    use_context_provider(|| scrollbar_ctx);

    use_effect(use_reactive((&scrollbar_id,), {
        let mut sid_sig = scrollbar_ctx.scrollbar_id;
        move |(sid,)| {
            if sid_sig.peek().as_str() != sid {
                sid_sig.set(sid);
            }
        }
    }));
    use_effect(use_reactive((&thumb_id_resolved,), {
        let mut tid_sig = scrollbar_ctx.thumb_id;
        move |(tid,)| {
            if tid_sig.peek().as_str() != tid {
                tid_sig.set(tid);
            }
        }
    }));

    if !should_render {
        return rsx! {};
    }

    let thumb_size = get_thumb_size(current_sizes);
    let mut scrollbar_style = match orientation {
        Orientation::Vertical | Orientation::Both => format!(
            "position: absolute; top: 0; bottom: var(--monoxus-scroll-area-corner-height, var(--radix-scroll-area-corner-height, 0px)); right: {}; left: {}; pointer-events: auto; --monoxus-scroll-area-thumb-height: {thumb_size}px; --radix-scroll-area-thumb-height: {thumb_size}px; ",
            if ctx.dir == Direction::Ltr {
                "0"
            } else {
                "auto"
            },
            if ctx.dir == Direction::Rtl {
                "0"
            } else {
                "auto"
            },
        ),
        Orientation::Horizontal => format!(
            "position: absolute; bottom: 0; left: {}; right: {}; pointer-events: auto; --monoxus-scroll-area-thumb-width: {thumb_size}px; --radix-scroll-area-thumb-width: {thumb_size}px; ",
            if ctx.dir == Direction::Rtl {
                "var(--monoxus-scroll-area-corner-width, var(--radix-scroll-area-corner-width, 0px))"
            } else {
                "0"
            },
            if ctx.dir == Direction::Ltr {
                "var(--monoxus-scroll-area-corner-width, var(--radix-scroll-area-corner-width, 0px))"
            } else {
                "0"
            },
        ),
    };
    if let Some(user_style) = &props.style {
        scrollbar_style.push_str(user_style);
    }

    let scrollbar_attrs =
        ScrollAreaScrollbarAttributes::new(&scrollbar_id, orientation, &viewport_id)
            .with_visible(is_visible);

    let onpointerenter = {
        let mut runtime = ctx.runtime;
        let r#type = ctx.r#type;
        move |_| {
            if r#type == ScrollAreaType::Scroll {
                let next = transition_scroll_state(
                    runtime.scroll_state(),
                    ScrollAreaStateEvent::PointerEnter,
                );
                runtime.scroll_state.set(next);
            }
        }
    };

    let onpointerleave = {
        let mut runtime = ctx.runtime;
        let r#type = ctx.r#type;
        move |_| {
            if r#type == ScrollAreaType::Scroll {
                let next = transition_scroll_state(
                    runtime.scroll_state(),
                    ScrollAreaStateEvent::PointerLeave,
                );
                runtime.scroll_state.set(next);
            }
        }
    };

    let onpointerdown = {
        let mounted_el = Rc::clone(&mounted_el);
        let captured_pid = Rc::clone(&captured_pid);
        let mut is_dragging = is_dragging;
        let mut pointer_offset = pointer_offset;
        let mut thumb_pointer_active = thumb_pointer_active;
        let mut runtime = ctx.runtime;
        let sid = scrollbar_id.clone();
        let rels = ctx.relationships;
        let dir = ctx.dir;

        move |evt: PointerEvent| {
            if evt.trigger_button() == Some(dioxus::html::input_data::MouseButton::Secondary) {
                return;
            }
            let pid = try_set_pointer_capture(&evt.data(), mounted_el.borrow().as_ref());
            *captured_pid.borrow_mut() = pid;
            set_body_user_select_none();
            is_dragging.set(true);

            // If the thumb itself was clicked, do not jump track
            if *thumb_pointer_active.peek() {
                thumb_pointer_active.set(false);
                return;
            }

            pointer_offset.set(0.0);
            let vid = rels.peek().viewport_id().to_string();

            // Track jump click: when clicking on track outside thumb, jump thumb to pointer position
            if let Some(rect) = get_element_client_rect(&sid) {
                let coords = evt.client_coordinates();
                let pointer_pos = match orientation {
                    Orientation::Horizontal => coords.x - rect.x,
                    Orientation::Vertical | Orientation::Both => coords.y - rect.y,
                };
                let cur_sizes = *sizes.peek();
                let target_scroll =
                    get_scroll_position_from_pointer(pointer_pos, 0.0, cur_sizes, dir);
                match orientation {
                    Orientation::Horizontal => {
                        let cur_top = runtime
                            .viewport_metrics()
                            .map(|m| m.scroll_top)
                            .unwrap_or(0.0);
                        set_element_scroll_position(&vid, target_scroll, cur_top);
                    }
                    Orientation::Vertical | Orientation::Both => {
                        let cur_left = runtime
                            .viewport_metrics()
                            .map(|m| m.scroll_left)
                            .unwrap_or(0.0);
                        set_element_scroll_position(&vid, cur_left, target_scroll);
                    }
                }
                if let Some(m) = get_element_scroll_metrics(&vid) {
                    runtime.viewport_metrics.set(Some(m));
                }
            }
        }
    };

    let onpointermove = {
        let sid = scrollbar_id.clone();
        let rels = ctx.relationships;
        let mut runtime = ctx.runtime;
        let dir = ctx.dir;

        move |evt: PointerEvent| {
            if !*is_dragging.peek() {
                return;
            }
            let vid = rels.peek().viewport_id().to_string();
            if let Some(rect) = get_element_client_rect(&sid) {
                let coords = evt.client_coordinates();
                let pointer_pos = match orientation {
                    Orientation::Horizontal => coords.x - rect.x,
                    Orientation::Vertical | Orientation::Both => coords.y - rect.y,
                };
                let cur_sizes = *sizes.peek();
                let offset = *pointer_offset.peek();
                let target_scroll =
                    get_scroll_position_from_pointer(pointer_pos, offset, cur_sizes, dir);
                match orientation {
                    Orientation::Horizontal => {
                        let cur_top = runtime
                            .viewport_metrics()
                            .map(|m| m.scroll_top)
                            .unwrap_or(0.0);
                        set_element_scroll_position(&vid, target_scroll, cur_top);
                    }
                    Orientation::Vertical | Orientation::Both => {
                        let cur_left = runtime
                            .viewport_metrics()
                            .map(|m| m.scroll_left)
                            .unwrap_or(0.0);
                        set_element_scroll_position(&vid, cur_left, target_scroll);
                    }
                }
                if let Some(m) = get_element_scroll_metrics(&vid) {
                    runtime.viewport_metrics.set(Some(m));
                }
            }
        }
    };

    let onpointerup = {
        let mounted_el = Rc::clone(&mounted_el);
        let captured_pid = Rc::clone(&captured_pid);
        let mut is_dragging = is_dragging;
        let mut thumb_pointer_active = thumb_pointer_active;

        move |evt: PointerEvent| {
            try_release_pointer_capture(
                Some(&evt),
                mounted_el.borrow().as_ref(),
                *captured_pid.borrow(),
            );
            restore_body_user_select();
            is_dragging.set(false);
            thumb_pointer_active.set(false);
            *captured_pid.borrow_mut() = None;
        }
    };

    let onlostpointercapture = {
        let mounted_el = Rc::clone(&mounted_el);
        let captured_pid = Rc::clone(&captured_pid);
        let mut is_dragging = is_dragging;
        let mut thumb_pointer_active = thumb_pointer_active;

        move |evt: PointerEvent| {
            try_release_pointer_capture(
                Some(&evt),
                mounted_el.borrow().as_ref(),
                *captured_pid.borrow(),
            );
            restore_body_user_select();
            is_dragging.set(false);
            thumb_pointer_active.set(false);
            *captured_pid.borrow_mut() = None;
        }
    };

    let onwheel = {
        let rels = ctx.relationships;
        let mut runtime = ctx.runtime;
        move |evt: WheelEvent| {
            let vid = rels.peek().viewport_id().to_string();
            let delta = evt.delta();
            let (dx, dy) = match delta {
                WheelDelta::Pixels(p) => (p.x, p.y),
                WheelDelta::Lines(l) => (l.x * 16.0, l.y * 16.0),
                WheelDelta::Pages(pg) => (pg.x * 100.0, pg.y * 100.0),
            };
            let within_bounds = forward_wheel_to_viewport(&vid, dx, dy, orientation);
            if let Some(m) = get_element_scroll_metrics(&vid) {
                runtime.viewport_metrics.set(Some(m));
            }
            if within_bounds {
                evt.prevent_default();
            }
        }
    };

    let onmounted = {
        let mounted_el = Rc::clone(&mounted_el);
        move |evt: MountedEvent| {
            *mounted_el.borrow_mut() = Some(oxidase::Element::from_mounted(evt.data()));
        }
    };

    let ontransitionend = {
        let presence = Rc::clone(&presence);
        let mut presence_epoch = presence_epoch;
        move |_evt: Event<TransitionData>| {
            let cycle = presence.borrow().active_close_cycle();
            if let Some(cycle) = cycle {
                if presence.borrow_mut().complete_close_cycle(cycle) {
                    if let Ok(mut epoch) = presence_epoch.try_write() {
                        *epoch += 1;
                    }
                }
            }
        }
    };

    let onanimationend = {
        let presence = Rc::clone(&presence);
        let mut presence_epoch = presence_epoch;
        move |_evt: Event<AnimationData>| {
            let cycle = presence.borrow().active_close_cycle();
            if let Some(cycle) = cycle {
                if presence.borrow_mut().complete_close_cycle(cycle) {
                    if let Ok(mut epoch) = presence_epoch.try_write() {
                        *epoch += 1;
                    }
                }
            }
        }
    };

    rsx! {
        div {
            id: "{scrollbar_attrs.id()}",
            class: props.class.as_deref(),
            style: "{scrollbar_style}",
            role: "{scrollbar_attrs.role()}",
            "aria-orientation": "{scrollbar_attrs.aria_orientation()}",
            "aria-controls": "{scrollbar_attrs.aria_controls()}",
            "data-orientation": "{scrollbar_attrs.data_orientation()}",
            "data-state": "{scrollbar_attrs.data_state()}",
            "data-monoxus-scroll-area-scrollbar": "",
            onpointerenter: onpointerenter,
            onpointerleave: onpointerleave,
            onpointerdown: onpointerdown,
            onpointermove: onpointermove,
            onpointerup: onpointerup,
            onlostpointercapture: onlostpointercapture,
            onwheel: onwheel,
            onmounted: onmounted,
            ontransitionend: ontransitionend,
            onanimationend: onanimationend,
            {props.children}
        }
    }
}

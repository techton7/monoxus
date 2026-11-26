use std::cell::RefCell;
use std::rc::Rc;

use dioxus::html::geometry::{ClientPoint, WheelDelta};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        ElementSize, ResizeObserverBox, WatcherGuard, get_element_client_rect,
        get_element_scroll_metrics, restore_body_user_select, set_body_user_select_none,
        set_element_scroll_position, start_resize_monitor,
    },
    compose::{join_styles, merge_global_attributes},
    overlay::use_presence,
    shared::{Direction, Orientation},
    state::sync_signal,
};

use super::{
    super::{
        attrs::ScrollAreaScrollbarAttributes,
        runtime::{
            ScrollAreaRuntime, forward_wheel_to_viewport, try_release_pointer_capture,
            try_set_pointer_capture,
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
    pub children: Element,
    pub class: Option<String>,
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

fn apply_pointer_scroll(
    viewport_id: &str,
    scrollbar_id: &str,
    coords: ClientPoint,
    offset: f64,
    orientation: Orientation,
    dir: Direction,
    sizes: Signal<ScrollAreaSizes>,
    mut runtime: ScrollAreaRuntime,
) {
    if let Some(rect) = get_element_client_rect(scrollbar_id) {
        let pointer_pos = match orientation {
            Orientation::Horizontal => coords.x - rect.x,
            Orientation::Vertical | Orientation::Both => coords.y - rect.y,
        };
        let cur_sizes = *sizes.peek();
        let target_scroll =
            get_scroll_position_from_pointer(pointer_pos, offset, cur_sizes, dir);
        match orientation {
            Orientation::Horizontal => {
                let cur_top = runtime
                    .viewport_metrics()
                    .map(|m| m.scroll_top)
                    .unwrap_or(0.0);
                set_element_scroll_position(viewport_id, target_scroll, cur_top);
            }
            Orientation::Vertical | Orientation::Both => {
                let cur_left = runtime
                    .viewport_metrics()
                    .map(|m| m.scroll_left)
                    .unwrap_or(0.0);
                set_element_scroll_position(viewport_id, cur_left, target_scroll);
            }
        }
        if let Some(m) = get_element_scroll_metrics(viewport_id) {
            runtime.viewport_metrics.set(Some(m));
        }
    }
}

fn finish_pointer_drag(
    evt: &PointerEvent,
    mounted_el: &Rc<RefCell<Option<oxidase::Element>>>,
    captured_pid: &Rc<RefCell<Option<i32>>>,
    mut is_dragging: Signal<bool>,
    mut thumb_pointer_active: Signal<bool>,
) {
    try_release_pointer_capture(Some(evt), mounted_el.borrow().as_ref(), *captured_pid.borrow());
    restore_body_user_select();
    is_dragging.set(false);
    thumb_pointer_active.set(false);
    *captured_pid.borrow_mut() = None;
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

    let scroll_type = ctx.scroll_type();
    let dir = ctx.dir();
    let is_visible = is_scrollbar_visible(scroll_type, scroll_state, hover_visible, overflowing);
    let presence = use_presence(is_visible, &scrollbar_id, true, None);
    let should_render = props.force_mount || presence.should_render();
    let keep_corner_active = props.force_mount || is_visible;

    let mut should_render_sig = use_signal(|| should_render);
    sync_signal(&mut should_render_sig, should_render);
    let mut keep_corner_active_sig = use_signal(|| keep_corner_active);
    sync_signal(&mut keep_corner_active_sig, keep_corner_active);
    let alive = use_hook(|| CopyValue::new(true));

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
                        runtime.sync_corner_size(orientation, rect.width, rect.height);
                    }

                    {
                        let sid_shot = sid.clone();
                        let vid_shot = vid.clone();
                        let mut s = sizes;
                        let mut rt = runtime;
                        spawn(async move {
                            oxidase::frame::next_frame().await;
                            if !alive.try_read().is_ok_and(|a| *a)
                                || !should_render_sig.try_peek().is_ok_and(|r| *r)
                                || !keep_corner_active_sig.try_peek().is_ok_and(|a| *a)
                            {
                                return;
                            }
                            if let Some(rect) = get_element_client_rect(&sid_shot) {
                                let track_size = match orientation {
                                    Orientation::Horizontal => rect.width,
                                    Orientation::Vertical | Orientation::Both => rect.height,
                                };
                                if track_size > 0.0
                                    && s.try_peek()
                                        .is_ok_and(|cur| cur.scrollbar_size != track_size)
                                    && let Ok(mut guard) = s.try_write()
                                {
                                    guard.scrollbar_size = track_size;
                                }
                                rt.sync_corner_size(orientation, rect.width, rect.height);
                            }
                            if let Some(m) = get_element_scroll_metrics(&vid_shot) {
                                let (content, viewport) = match orientation {
                                    Orientation::Horizontal => (m.scroll_width, m.client_width),
                                    Orientation::Vertical | Orientation::Both => {
                                        (m.scroll_height, m.client_height)
                                    }
                                };
                                if let Ok(cur) = s.try_peek().map(|g| *g)
                                    && (cur.content != content || cur.viewport != viewport)
                                    && let Ok(mut guard) = s.try_write()
                                {
                                    *guard = cur.with_content(content).with_viewport(viewport);
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
                            if !alive.try_read().is_ok_and(|a| *a)
                                || !should_render_sig.try_peek().is_ok_and(|r| *r)
                                || !keep_corner_active_sig.try_peek().is_ok_and(|a| *a)
                            {
                                return;
                            }
                            let track_size = match orientation {
                                Orientation::Horizontal => size.width,
                                Orientation::Vertical | Orientation::Both => size.height,
                            };
                            if s.try_peek()
                                .is_ok_and(|cur| cur.scrollbar_size != track_size)
                                && let Ok(mut guard) = s.try_write()
                            {
                                guard.scrollbar_size = track_size;
                            }
                            rt.sync_corner_size(orientation, size.width, size.height);
                        },
                    );
                    let prev = std::mem::replace(&mut *resize_guard_cell.borrow_mut(), next_guard);
                    prev.stop();
                } else {
                    std::mem::replace(&mut *resize_guard_cell.borrow_mut(), WatcherGuard::noop())
                        .stop();
                    runtime.sync_corner_size(orientation, 0.0, 0.0);
                }
            }
        },
    ));

    use_drop({
        let resize_guard = Rc::clone(&resize_guard_cell);
        let is_dragging = is_dragging;
        let mut runtime = ctx.runtime;
        let mut alive = alive;
        move || {
            if let Ok(mut flag) = alive.try_write() {
                *flag = false;
            }
            std::mem::replace(&mut *resize_guard.borrow_mut(), WatcherGuard::noop()).stop();
            if is_dragging.try_peek().is_ok_and(|d| *d) {
                restore_body_user_select();
            }
            match orientation {
                Orientation::Horizontal => {
                    sync_signal(&mut runtime.scrollbar_x_enabled, false);
                }
                Orientation::Vertical => {
                    sync_signal(&mut runtime.scrollbar_y_enabled, false);
                }
                Orientation::Both => {
                    sync_signal(&mut runtime.scrollbar_x_enabled, false);
                    sync_signal(&mut runtime.scrollbar_y_enabled, false);
                }
            }
            runtime.sync_corner_size(orientation, 0.0, 0.0);
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
            sync_signal(&mut sid_sig, sid);
        }
    }));
    use_effect(use_reactive((&thumb_id_resolved,), {
        let mut tid_sig = scrollbar_ctx.thumb_id;
        move |(tid,)| {
            sync_signal(&mut tid_sig, tid);
        }
    }));

    if !should_render {
        return rsx! {};
    }

    let thumb_size = get_thumb_size(current_sizes);
    let base_style = match orientation {
        Orientation::Vertical | Orientation::Both => format!(
            "position: absolute; top: 0; bottom: var(--monoxus-scroll-area-corner-height, var(--radix-scroll-area-corner-height, 0px)); right: {}; left: {}; pointer-events: auto; --monoxus-scroll-area-thumb-height: {thumb_size}px; --radix-scroll-area-thumb-height: {thumb_size}px;",
            if dir == Direction::Ltr { "0" } else { "auto" },
            if dir == Direction::Rtl { "0" } else { "auto" },
        ),
        Orientation::Horizontal => format!(
            "position: absolute; bottom: 0; left: {}; right: {}; pointer-events: auto; --monoxus-scroll-area-thumb-width: {thumb_size}px; --radix-scroll-area-thumb-width: {thumb_size}px;",
            if dir == Direction::Rtl {
                "var(--monoxus-scroll-area-corner-width, var(--radix-scroll-area-corner-width, 0px))"
            } else {
                "0"
            },
            if dir == Direction::Ltr {
                "var(--monoxus-scroll-area-corner-width, var(--radix-scroll-area-corner-width, 0px))"
            } else {
                "0"
            },
        ),
    };
    let combined_style = join_styles([base_style.as_str(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

    let scrollbar_attrs =
        ScrollAreaScrollbarAttributes::new(&scrollbar_id, orientation, &viewport_id)
            .with_visible(is_visible);

    let onpointerenter = {
        let mut runtime = ctx.runtime;
        let type_sig = ctx.r#type;
        move |_| {
            if type_sig
                .try_peek()
                .is_ok_and(|t| *t == ScrollAreaType::Scroll)
            {
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
        let type_sig = ctx.r#type;
        move |_| {
            if type_sig
                .try_peek()
                .is_ok_and(|t| *t == ScrollAreaType::Scroll)
            {
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
        let runtime = ctx.runtime;
        let sid = scrollbar_id.clone();
        let rels = ctx.relationships;
        let dir_sig = ctx.dir;

        move |evt: PointerEvent| {
            if evt.pointer_type() == "mouse"
                && evt
                    .trigger_button()
                    .is_some_and(|b| b != dioxus::html::input_data::MouseButton::Primary)
            {
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
            let dir = dir_sig.try_peek().map(|d| *d).unwrap_or(Direction::Ltr);
            apply_pointer_scroll(
                &vid,
                &sid,
                evt.client_coordinates(),
                0.0,
                orientation,
                dir,
                sizes,
                runtime,
            );
        }
    };

    let onpointermove = {
        let sid = scrollbar_id.clone();
        let rels = ctx.relationships;
        let runtime = ctx.runtime;
        let dir_sig = ctx.dir;

        move |evt: PointerEvent| {
            if !*is_dragging.peek() {
                return;
            }
            let vid = rels.peek().viewport_id().to_string();
            let offset = *pointer_offset.peek();
            let dir = dir_sig.try_peek().map(|d| *d).unwrap_or(Direction::Ltr);
            apply_pointer_scroll(
                &vid,
                &sid,
                evt.client_coordinates(),
                offset,
                orientation,
                dir,
                sizes,
                runtime,
            );
        }
    };

    let onpointerup = {
        let mounted_el = Rc::clone(&mounted_el);
        let captured_pid = Rc::clone(&captured_pid);
        move |evt: PointerEvent| {
            finish_pointer_drag(&evt, &mounted_el, &captured_pid, is_dragging, thumb_pointer_active);
        }
    };

    let onlostpointercapture = {
        let mounted_el = Rc::clone(&mounted_el);
        let captured_pid = Rc::clone(&captured_pid);
        move |evt: PointerEvent| {
            finish_pointer_drag(&evt, &mounted_el, &captured_pid, is_dragging, thumb_pointer_active);
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

    rsx! {
        div {
            id: "{scrollbar_attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
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
            ontransitionend: move |_| {
                presence.finish_active_exit();
            },
            onanimationend: move |_| {
                presence.finish_active_exit();
            },
            ..merged.rest,
            {props.children}
        }
    }
}

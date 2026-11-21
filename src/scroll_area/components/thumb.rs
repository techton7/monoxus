use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        PresenceMonitorEvent, WatcherGuard, get_element_client_rect, start_presence_monitor,
    },
    overlay::PresenceController,
    shared::Orientation,
};

use super::{
    super::{
        attrs::ScrollAreaThumbAttributes,
        runtime::start_unlinked_scroll_sync,
        state::{get_thumb_offset_from_scroll, get_thumb_size, has_thumb},
    },
    ScrollAreaContext, ScrollbarContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct ScrollAreaThumbProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    pub as_child: Option<bool>,
    pub children: Option<Element>,
    pub class: Option<String>,
    pub style: Option<String>,
}

#[component]
pub fn ScrollAreaThumb(props: ScrollAreaThumbProps) -> Element {
    let ctx = use_context::<ScrollAreaContext>();
    let scrollbar_ctx = use_context::<ScrollbarContext>();

    let thumb_id = props
        .id
        .clone()
        .unwrap_or_else(|| scrollbar_ctx.thumb_id.read().clone());

    use_effect(use_reactive((&thumb_id,), {
        let mut tid_sig = scrollbar_ctx.thumb_id;
        let mut rels = ctx.relationships;
        let orientation = scrollbar_ctx.orientation;
        move |(tid,)| {
            if tid_sig.peek().as_str() != tid {
                tid_sig.set(tid.clone());
            }
            let cur = rels.peek().clone();
            let needs_update = match orientation {
                Orientation::Horizontal => cur.thumb_x_id() != tid,
                Orientation::Vertical | Orientation::Both => cur.thumb_y_id() != tid,
            };
            if needs_update {
                let updated = match orientation {
                    Orientation::Horizontal => cur.with_thumb_x_id(tid),
                    Orientation::Vertical | Orientation::Both => cur.with_thumb_y_id(tid),
                };
                rels.set(updated);
            }
        }
    }));

    let current_sizes = scrollbar_ctx.sizes();
    let has_thumb_val = has_thumb(current_sizes);

    let presence = use_hook(|| {
        Rc::new(RefCell::new(
            PresenceController::new(has_thumb_val).with_retained_mount(true),
        ))
    });
    let mut presence_epoch = use_signal(|| 0u64);
    let presence_monitor = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));
    let _ = presence_epoch();

    let needs_sync = presence.borrow().desired_present() != has_thumb_val;
    if needs_sync {
        let update = presence.borrow_mut().sync(has_thumb_val);
        if let Some(cycle_id) = update.started_close_cycle() {
            let presence_cb = Rc::clone(&presence);
            let watcher =
                start_presence_monitor(thumb_id.as_str(), cycle_id, move |event| match event {
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

    let mut mounted_rev = use_signal(|| 0u64);
    let scroll_sync_guard_cell = use_hook(|| Rc::new(RefCell::new(WatcherGuard::noop())));
    let viewport_id = ctx.relationships.read().viewport_id().to_string();

    use_effect(use_reactive(
        (&should_render, &thumb_id, &viewport_id, &mounted_rev()),
        {
            let scroll_sync_guard_cell = Rc::clone(&scroll_sync_guard_cell);
            let rels = ctx.relationships;
            let tid_sig = scrollbar_ctx.thumb_id;
            let orientation = scrollbar_ctx.orientation;
            let dir = ctx.dir;
            let sizes = scrollbar_ctx.sizes;
            move |(rendered, _tid, _vid, _rev)| {
                let next_guard = if rendered {
                    start_unlinked_scroll_sync(rels, tid_sig, orientation, dir, sizes)
                } else {
                    WatcherGuard::noop()
                };
                let prev = std::mem::replace(&mut *scroll_sync_guard_cell.borrow_mut(), next_guard);
                prev.stop();
            }
        },
    ));

    use_drop({
        let scroll_sync_guard_cell = Rc::clone(&scroll_sync_guard_cell);
        let presence_monitor = Rc::clone(&presence_monitor);
        move || {
            std::mem::replace(
                &mut *scroll_sync_guard_cell.borrow_mut(),
                WatcherGuard::noop(),
            )
            .stop();
            *presence_monitor.borrow_mut() = None;
        }
    });

    if !should_render {
        return rsx! {};
    }

    let scroll_pos = match scrollbar_ctx.orientation {
        Orientation::Horizontal => ctx
            .runtime
            .viewport_metrics()
            .map(|m| m.scroll_left)
            .unwrap_or(0.0),
        Orientation::Vertical | Orientation::Both => ctx
            .runtime
            .viewport_metrics()
            .map(|m| m.scroll_top)
            .unwrap_or(0.0),
    };

    let offset = get_thumb_offset_from_scroll(scroll_pos, current_sizes, ctx.dir);
    let thumb_size = get_thumb_size(current_sizes);

    let mut thumb_style = match scrollbar_ctx.orientation {
        Orientation::Vertical | Orientation::Both => format!(
            "width: 100%; height: var(--monoxus-scroll-area-thumb-height, var(--radix-scroll-area-thumb-height, {thumb_size}px)); transform: translate3d(0, {offset}px, 0); "
        ),
        Orientation::Horizontal => format!(
            "height: 100%; width: var(--monoxus-scroll-area-thumb-width, var(--radix-scroll-area-thumb-width, {thumb_size}px)); transform: translate3d({offset}px, 0, 0); "
        ),
    };
    if let Some(user_style) = &props.style {
        thumb_style.push_str(user_style);
    }

    let thumb_attrs = ScrollAreaThumbAttributes::new(&thumb_id).with_visible(has_thumb_val);

    let onpointerdown = {
        let mut pointer_offset = scrollbar_ctx.pointer_offset;
        let mut thumb_pointer_active = scrollbar_ctx.thumb_pointer_active;
        let orientation = scrollbar_ctx.orientation;
        let tid = thumb_id.clone();
        move |evt: PointerEvent| {
            if let Some(rect) = get_element_client_rect(&tid) {
                let coords = evt.client_coordinates();
                let offset = match orientation {
                    Orientation::Horizontal => coords.x - rect.x,
                    Orientation::Vertical | Orientation::Both => coords.y - rect.y,
                };
                pointer_offset.set(offset);
                thumb_pointer_active.set(true);
            }
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
            id: "{thumb_attrs.id()}",
            class: props.class.as_deref(),
            style: "{thumb_style}",
            "data-state": "{thumb_attrs.data_state()}",
            "data-monoxus-scroll-area-thumb": "",
            onmounted: move |_| mounted_rev += 1,
            onpointerdown: onpointerdown,
            ontransitionend: ontransitionend,
            onanimationend: onanimationend,
            {props.children}
        }
    }
}

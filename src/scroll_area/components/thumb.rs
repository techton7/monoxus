use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    browser::{WatcherGuard, get_element_client_rect},
    compose::{join_styles, merge_global_attributes},
    overlay::use_presence,
    shared::Orientation,
    state::sync_signal,
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
    pub children: Option<Element>,
    pub class: Option<String>,
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
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
            sync_signal(&mut tid_sig, tid.clone());
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
    let presence = use_presence(has_thumb_val, &thumb_id, true, None);
    let should_render = props.force_mount || presence.should_render();

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
        move || {
            std::mem::replace(
                &mut *scroll_sync_guard_cell.borrow_mut(),
                WatcherGuard::noop(),
            )
            .stop();
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

    let base_style = match scrollbar_ctx.orientation {
        Orientation::Vertical | Orientation::Both => format!(
            "width: 100%; height: var(--monoxus-scroll-area-thumb-height, var(--radix-scroll-area-thumb-height, {thumb_size}px)); transform: translate3d(0, {offset}px, 0);"
        ),
        Orientation::Horizontal => format!(
            "height: 100%; width: var(--monoxus-scroll-area-thumb-width, var(--radix-scroll-area-thumb-width, {thumb_size}px)); transform: translate3d({offset}px, 0, 0);"
        ),
    };
    let combined_style = join_styles([base_style.as_str(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

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

    rsx! {
        div {
            id: "{thumb_attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": "{thumb_attrs.data_state()}",
            "data-monoxus-scroll-area-thumb": "",
            onmounted: move |_| mounted_rev += 1,
            onpointerdown: onpointerdown,
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

use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        ElementSize, ResizeObserverBox, WatcherGuard, get_element_scroll_metrics,
        start_element_scroll_monitor, start_resize_monitor,
    },
    compose::{join_styles, merge_global_attributes},
    state::sync_signal,
};

use super::{
    super::{
        attrs::{ScrollAreaContentAttributes, ScrollAreaViewportAttributes},
        state::transition_scroll_state,
        types::{ScrollAreaStateEvent, ScrollAreaType},
    },
    ScrollAreaContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct ScrollAreaViewportProps {
    #[props(default)]
    pub id: Option<String>,
    pub nonce: Option<String>,
    pub disable_implicit_content_element: Option<bool>,
    pub children: Element,
    pub class: Option<String>,
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ScrollAreaViewport(props: ScrollAreaViewportProps) -> Element {
    let ctx = use_context::<ScrollAreaContext>();
    let viewport_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.read().viewport_id().to_string());

    let scroll_end_timer = use_hook(|| Rc::new(Cell::new(0_u64)));
    let idle_hide_timer = use_hook(|| Rc::new(Cell::new(0_u64)));

    let scrollbar_x_enabled = ctx.runtime.scrollbar_x_enabled();
    let scrollbar_y_enabled = ctx.runtime.scrollbar_y_enabled();

    let base_style = format!(
        "scrollbar-width: none; -ms-overflow-style: none; -webkit-overflow-scrolling: touch; display: flex; flex-direction: column; align-items: stretch; overflow-x: {}; overflow-y: {};",
        if scrollbar_x_enabled { "scroll" } else { "hidden" },
        if scrollbar_y_enabled { "scroll" } else { "hidden" },
    );
    let combined_style = join_styles([base_style.as_str(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

    let viewport_attrs = ScrollAreaViewportAttributes::new(&viewport_id);

    use_effect(use_reactive((&viewport_id,), {
        let mut rels = ctx.relationships;
        move |(vid,)| {
            if rels.peek().viewport_id() != vid {
                let updated = rels.peek().clone().with_viewport_id(vid);
                rels.set(updated);
            }
        }
    }));

    let resize_guard_cell = use_hook(|| Rc::new(std::cell::RefCell::new(WatcherGuard::noop())));
    let scroll_guard_cell = use_hook(|| Rc::new(std::cell::RefCell::new(WatcherGuard::noop())));
    let mut mounted_rev = use_signal(|| 0u64);

    use_effect(use_reactive((&viewport_id, &mounted_rev()), {
        let resize_guard_cell = Rc::clone(&resize_guard_cell);
        let mut runtime = ctx.runtime;
        move |(vid, _rev)| {
            runtime.sync_viewport_metrics(&vid, None);

            {
                let vid_shot = vid.clone();
                let mut rt = runtime;
                spawn(async move {
                    oxidase::frame::next_frame().await;
                    rt.sync_viewport_metrics(&vid_shot, None);
                });
            }

            let vid_for_cb = vid.clone();
            let mut rt = runtime;
            let next_guard = start_resize_monitor(
                &vid,
                ResizeObserverBox::BorderBox,
                move |size: ElementSize| {
                    rt.sync_viewport_metrics(&vid_for_cb, Some(size));
                },
            );
            let prev = std::mem::replace(&mut *resize_guard_cell.borrow_mut(), next_guard);
            prev.stop();
        }
    }));

    use_effect(use_reactive((&viewport_id, &mounted_rev()), {
        let scroll_guard_cell = Rc::clone(&scroll_guard_cell);
        let scroll_end_timer = Rc::clone(&scroll_end_timer);
        let idle_hide_timer = Rc::clone(&idle_hide_timer);
        let mut runtime = ctx.runtime;
        let r#type = ctx.r#type;
        let scroll_hide_delay = ctx.scroll_hide_delay;

        move |(vid, _rev)| {
            let scroll_end_timer = Rc::clone(&scroll_end_timer);
            let idle_hide_timer = Rc::clone(&idle_hide_timer);
            let vid_scroll = vid.clone();

            let next_guard = start_element_scroll_monitor(&vid, move || {
                if let Some(metrics) = get_element_scroll_metrics(&vid_scroll) {
                    sync_signal(&mut runtime.viewport_metrics, Some(metrics));
                }

                if r#type == ScrollAreaType::Scroll {
                    let next_state = transition_scroll_state(
                        runtime.scroll_state(),
                        ScrollAreaStateEvent::Scroll,
                    );
                    runtime.scroll_state.set(next_state);

                    idle_hide_timer.set(idle_hide_timer.get().wrapping_add(1));

                    let next_end_gen = scroll_end_timer.get().wrapping_add(1);
                    scroll_end_timer.set(next_end_gen);
                    let end_timer_cell = Rc::clone(&scroll_end_timer);
                    let idle_timer_cell = Rc::clone(&idle_hide_timer);

                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_millis(100)).await;
                        if end_timer_cell.get() == next_end_gen {
                            let idle_state = transition_scroll_state(
                                runtime.scroll_state(),
                                ScrollAreaStateEvent::ScrollEnd,
                            );
                            runtime.scroll_state.set(idle_state);

                            let next_idle_gen = idle_timer_cell.get().wrapping_add(1);
                            idle_timer_cell.set(next_idle_gen);
                            let hide_cell = Rc::clone(&idle_timer_cell);

                            spawn(async move {
                                futures_timer::Delay::new(std::time::Duration::from_millis(
                                    scroll_hide_delay,
                                ))
                                .await;
                                if hide_cell.get() == next_idle_gen {
                                    let hidden_state = transition_scroll_state(
                                        runtime.scroll_state(),
                                        ScrollAreaStateEvent::Hide,
                                    );
                                    runtime.scroll_state.set(hidden_state);
                                }
                            });
                        }
                    });
                }
            });
            let prev = std::mem::replace(&mut *scroll_guard_cell.borrow_mut(), next_guard);
            prev.stop();
        }
    }));

    use_drop({
        let resize_guard = Rc::clone(&resize_guard_cell);
        let scroll_guard = Rc::clone(&scroll_guard_cell);
        let end_timer = Rc::clone(&scroll_end_timer);
        let hide_timer = Rc::clone(&idle_hide_timer);
        move || {
            std::mem::replace(&mut *resize_guard.borrow_mut(), WatcherGuard::noop()).stop();
            std::mem::replace(&mut *scroll_guard.borrow_mut(), WatcherGuard::noop()).stop();
            end_timer.set(end_timer.get().wrapping_add(1));
            hide_timer.set(hide_timer.get().wrapping_add(1));
        }
    });

    let css_rules = "[data-monoxus-scroll-area-viewport]{scrollbar-width:none;-ms-overflow-style:none;-webkit-overflow-scrolling:touch;}[data-monoxus-scroll-area-viewport]::-webkit-scrollbar{display:none;}[data-radix-scroll-area-viewport]{scrollbar-width:none;-ms-overflow-style:none;-webkit-overflow-scrolling:touch;}[data-radix-scroll-area-viewport]::-webkit-scrollbar{display:none;}";

    let children = if props.disable_implicit_content_element.unwrap_or(false) {
        rsx! { {props.children} }
    } else {
        rsx! {
            ScrollAreaContent { {props.children} }
        }
    };

    rsx! {
        style {
            nonce: props.nonce.as_deref(),
            "{css_rules}"
        }
        div {
            id: "{viewport_attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            tabindex: "{viewport_attrs.tabindex()}",
            "data-monoxus-scroll-area-viewport": "",
            "data-radix-scroll-area-viewport": "",
            onmounted: move |_| mounted_rev += 1,
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct ScrollAreaContentProps {
    #[props(default)]
    pub id: Option<String>,
    pub children: Element,
    pub class: Option<String>,
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ScrollAreaContent(props: ScrollAreaContentProps) -> Element {
    let ctx = use_context::<ScrollAreaContext>();
    let content_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.read().content_id().to_string());
    let vid = ctx.relationships.read().viewport_id().to_string();

    use_effect(use_reactive((&content_id,), {
        let mut rels = ctx.relationships;
        move |(cid,)| {
            if rels.peek().content_id() != cid {
                let updated = rels.peek().clone().with_content_id(cid);
                rels.set(updated);
            }
        }
    }));

    let base_style = if ctx.runtime.scrollbar_x_enabled() {
        "display: flex; flex-direction: column; flex-grow: 1; min-width: fit-content;"
    } else {
        "display: flex; flex-direction: column; flex-grow: 1;"
    };
    let combined_style = join_styles([base_style, props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

    let content_attrs = ScrollAreaContentAttributes::new(&content_id);
    let resize_guard_cell = use_hook(|| Rc::new(std::cell::RefCell::new(WatcherGuard::noop())));

    use_effect(use_reactive((&content_id, &vid), {
        let resize_guard_cell = Rc::clone(&resize_guard_cell);
        let runtime = ctx.runtime;
        move |(cid, vid_target)| {
            let vid_for_cb = vid_target.clone();
            let mut rt = runtime;
            let next_guard = start_resize_monitor(
                &cid,
                ResizeObserverBox::BorderBox,
                move |_content_size: ElementSize| {
                    rt.sync_viewport_metrics(&vid_for_cb, None);
                },
            );
            let prev = std::mem::replace(&mut *resize_guard_cell.borrow_mut(), next_guard);
            prev.stop();
        }
    }));

    // Deferred content resize check for Auto mode on Blitz Native
    use_effect({
        let rels = ctx.relationships;
        let runtime = ctx.runtime;
        move || {
            let vid_c = rels.peek().viewport_id().to_string();
            let mut rt = runtime;
            spawn(async move {
                oxidase::frame::next_frame().await;
                rt.sync_viewport_metrics(&vid_c, None);
            });
        }
    });

    use_drop({
        let resize_guard = Rc::clone(&resize_guard_cell);
        move || {
            std::mem::replace(&mut *resize_guard.borrow_mut(), WatcherGuard::noop()).stop();
        }
    });

    rsx! {
        div {
            id: "{content_attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: "{content_attrs.role()}",
            "data-monoxus-scroll-area-content": "",
            ..merged.rest,
            {props.children}
        }
    }
}

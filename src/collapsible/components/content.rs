use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::collapsible::attrs::{format_collapsible_css_vars, merge_disclosure_styles};
use crate::collapsible::components::CollapsibleContext;
use crate::foundation::{
    browser::{
        ElementSize, PresenceMonitorEvent, ResizeObserverBox, WatcherGuard,
        get_element_client_rect, is_element_animating, measure_element_without_animation,
        start_presence_monitor, start_resize_monitor,
    },
    overlay::PresenceController,
    shared::{data_disabled_attr, use_mount_animation_guard},
};

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub hidden_until_found: bool,
    pub children: Element,
}

#[component]
pub fn CollapsibleContent(props: CollapsibleContentProps) -> Element {
    let ctx = use_context::<CollapsibleContext>();
    let runtime = ctx.runtime.read();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = props.id.clone().unwrap_or_else(|| content.id().to_string());

    use_effect(use_reactive((&props.id,), {
        let mut cid_sig = ctx.content_id;
        move |(custom_id,)| {
            if let Some(id) = custom_id {
                if cid_sig.peek().as_ref() != Some(&id) {
                    cid_sig.set(Some(id));
                }
            }
        }
    }));

    let guard_style = use_mount_animation_guard(is_open);

    let presence = use_hook(|| {
        Rc::new(RefCell::new(
            PresenceController::new(is_open).with_retained_mount(true),
        ))
    });
    let mut presence_epoch = use_signal(|| 0u64);
    let presence_monitor = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));
    let _ = presence_epoch();

    let resize_guard = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));
    let mut content_size = use_signal(|| None::<ElementSize>);
    let mut mounted_rev = use_signal(|| 0u64);

    let needs_sync = presence.borrow().desired_present() != is_open;
    if needs_sync {
        let update = presence.borrow_mut().sync(is_open);
        if let Some(cycle_id) = update.started_close_cycle() {
            let presence_cb = Rc::clone(&presence);
            let watcher =
                start_presence_monitor(content_id.as_str(), cycle_id, move |event| match event {
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

    let is_presence_mounted = presence.borrow().should_render();
    let should_render = is_open || is_presence_mounted || props.force_mount;

    {
        let mut mounted_sig = ctx.content_mounted;
        if *mounted_sig.peek() != should_render {
            mounted_sig.set(should_render);
        }
    }

    use_drop({
        let mut mounted_sig = ctx.content_mounted;
        let presence_monitor = presence_monitor.clone();
        let resize_guard = resize_guard.clone();
        move || {
            if let Ok(mut m) = mounted_sig.try_write() {
                *m = false;
            }
            *presence_monitor.borrow_mut() = None;
            *resize_guard.borrow_mut() = None;
        }
    });

    use_effect(use_reactive((&is_open, &mounted_rev()), {
        let content_id = content_id.clone();
        let resize_guard = resize_guard.clone();
        move |(open, _rev)| {
            if open {
                if content_size.peek().is_none() {
                    if let Some(size) = measure_element_without_animation(&content_id)
                        && (size.width > 0.0 || size.height > 0.0)
                    {
                        content_size.set(Some(size));
                    }
                }

                let cid = content_id.clone();
                let cid_check = content_id.clone();
                let watcher =
                    start_resize_monitor(&cid, ResizeObserverBox::BorderBox, move |new_size| {
                        if is_element_animating(&cid_check) {
                            return;
                        }
                        if new_size.width <= 0.0 && new_size.height <= 0.0 {
                            return;
                        }
                        if content_size.peek().as_ref() != Some(&new_size) {
                            content_size.set(Some(new_size));
                        }
                    });
                *resize_guard.borrow_mut() = Some(watcher);
            } else {
                *resize_guard.borrow_mut() = None;
            }
        }
    }));

    if !should_render {
        return rsx! {};
    }

    let hidden_attr: Option<&'static str> = if is_open || is_presence_mounted {
        None
    } else if props.hidden_until_found {
        Some("until-found")
    } else {
        Some("true")
    };

    let css_vars = (*content_size.read()).map(format_collapsible_css_vars);
    let effective_style =
        merge_disclosure_styles(props.style.as_deref(), guard_style, css_vars.as_deref());

    let presence_trans = Rc::clone(&presence);
    let presence_anim = Rc::clone(&presence);
    let cid_mount = content_id.clone();
    let cid_trans = content_id.clone();
    let cid_anim = content_id.clone();

    rsx! {
        div {
            id: "{content_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: effective_style.as_deref().unwrap_or_default(),
            hidden: hidden_attr,
            "data-state": content.data_state_str(),
            "data-disabled": data_disabled_attr(content.is_disabled()),
            onmounted: move |_| {
                mounted_rev += 1;
                if is_open
                    && let Some(size) = measure_element_without_animation(&cid_mount)
                    && (size.width > 0.0 || size.height > 0.0)
                {
                    let existing = *content_size.peek();
                    let animating_zero_height = size.height <= 0.0
                        && existing.is_some_and(|prev| prev.height > 0.0)
                        && is_element_animating(&cid_mount);
                    if !animating_zero_height
                        && existing != Some(size)
                        && let Ok(mut cs) = content_size.try_write()
                    {
                        *cs = Some(size);
                    }
                }
            },
            ontransitionend: move |_evt: Event<TransitionData>| {
                let cycle = presence_trans.borrow().active_close_cycle();
                if let Some(cycle) = cycle {
                    if !is_element_animating(&cid_trans)
                        && presence_trans.borrow_mut().complete_close_cycle(cycle)
                    {
                        if let Ok(mut epoch) = presence_epoch.try_write() {
                            *epoch += 1;
                        }
                    }
                } else if is_open
                    && !is_element_animating(&cid_trans)
                    && let Some(rect) = get_element_client_rect(&cid_trans)
                    && (rect.width > 0.0 || rect.height > 0.0)
                {
                    let size = ElementSize {
                        width: rect.width,
                        height: rect.height,
                    };
                    if content_size.peek().as_ref() != Some(&size)
                        && let Ok(mut cs) = content_size.try_write()
                    {
                        *cs = Some(size);
                    }
                }
            },
            onanimationend: move |_evt: Event<AnimationData>| {
                let cycle = presence_anim.borrow().active_close_cycle();
                if let Some(cycle) = cycle {
                    if !is_element_animating(&cid_anim)
                        && presence_anim.borrow_mut().complete_close_cycle(cycle)
                    {
                        if let Ok(mut epoch) = presence_epoch.try_write() {
                            *epoch += 1;
                        }
                    }
                } else if is_open
                    && !is_element_animating(&cid_anim)
                    && let Some(rect) = get_element_client_rect(&cid_anim)
                    && (rect.width > 0.0 || rect.height > 0.0)
                {
                    let size = ElementSize {
                        width: rect.width,
                        height: rect.height,
                    };
                    if content_size.peek().as_ref() != Some(&size)
                        && let Ok(mut cs) = content_size.try_write()
                    {
                        *cs = Some(size);
                    }
                }
            },
            {props.children}
        }
    }
}

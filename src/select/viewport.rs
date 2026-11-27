use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::keyboard_types::Key;
use dioxus::prelude::*;
use oxidase::WatcherGuard;
use oxidase::dom::ResizeObserverBox;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    keyboard::{NavKey, is_ime_composing_key, translate_nav_key},
};

use super::{runtime::SelectRuntime, types::SelectItemData};

pub fn apply_document_order(items: &mut Vec<SelectItemData>, order: &[String]) {
    if !order.is_empty() {
        items.sort_by_key(|item| {
            order
                .iter()
                .position(|v| v == &item.value)
                .unwrap_or(usize::MAX)
        });
    }
}

impl SelectRuntime {
    pub fn item_label(&self, val: &str) -> Option<String> {
        if let Ok(labels) = self.state.selected_labels.try_read()
            && let Some(lbl) = labels.get(val)
        {
            return Some(lbl.clone());
        }
        let reg = self.state.registry.read();
        if let Some(item) = reg.active_items().find(|i| i.value == val) {
            return Some(item.label.clone());
        }
        self.state
            .items
            .read()
            .iter()
            .find(|i| i.value == val)
            .map(|i| i.text.clone())
    }

    pub fn highlight_next(&self) {
        self.clear_initial_highlight_pending();
        let reg = self.state.registry.read();
        let enabled: Vec<String> = if !reg.ordered_item_ids.is_empty() {
            reg.active_items()
                .filter(|i| !i.disabled)
                .map(|i| i.value.clone())
                .collect()
        } else {
            self.state
                .items
                .read()
                .iter()
                .filter(|i| !i.disabled)
                .map(|i| i.value.clone())
                .collect()
        };
        if enabled.is_empty() {
            return;
        }

        let loop_selection = self.select.read().loop_selection();
        let current = self.highlighted_value();
        let curr_idx = current
            .as_ref()
            .and_then(|v| enabled.iter().position(|i| i == v));

        let next_idx = match curr_idx {
            Some(idx) => {
                if idx + 1 < enabled.len() {
                    idx + 1
                } else if loop_selection {
                    0
                } else {
                    idx
                }
            }
            None => 0,
        };

        self.set_highlighted(Some(enabled[next_idx].clone()));
    }

    pub fn highlight_prev(&self) {
        self.clear_initial_highlight_pending();
        let reg = self.state.registry.read();
        let enabled: Vec<String> = if !reg.ordered_item_ids.is_empty() {
            reg.active_items()
                .filter(|i| !i.disabled)
                .map(|i| i.value.clone())
                .collect()
        } else {
            self.state
                .items
                .read()
                .iter()
                .filter(|i| !i.disabled)
                .map(|i| i.value.clone())
                .collect()
        };
        if enabled.is_empty() {
            return;
        }

        let loop_selection = self.select.read().loop_selection();
        let current = self.highlighted_value();
        let curr_idx = current
            .as_ref()
            .and_then(|v| enabled.iter().position(|i| i == v));

        let prev_idx = match curr_idx {
            Some(0) => {
                if loop_selection {
                    enabled.len() - 1
                } else {
                    0
                }
            }
            Some(idx) => idx - 1,
            None => enabled.len() - 1,
        };

        self.set_highlighted(Some(enabled[prev_idx].clone()));
    }

    pub fn highlight_first(&self) {
        self.clear_initial_highlight_pending();
        let reg = self.state.registry.read();
        if let Some(first) = reg.active_items().find(|i| !i.disabled) {
            self.set_highlighted(Some(first.value.clone()));
            return;
        }
        let items = self.state.items.read();
        if let Some(first) = items.iter().find(|i| !i.disabled) {
            self.set_highlighted(Some(first.value.clone()));
        }
    }

    pub fn highlight_last(&self) {
        self.clear_initial_highlight_pending();
        let reg = self.state.registry.read();
        let active: Vec<_> = reg.active_items().filter(|i| !i.disabled).collect();
        if let Some(last) = active.last() {
            self.set_highlighted(Some(last.value.clone()));
            return;
        }
        let items = self.state.items.read();
        if let Some(last) = items.iter().rev().find(|i| !i.disabled) {
            self.set_highlighted(Some(last.value.clone()));
        }
    }

    pub fn handle_content_keydown(&self, event: &KeyboardEvent, now_ms: f64) {
        if !self.is_open() || is_ime_composing_key(event) {
            return;
        }
        let key = event.key();
        match translate_nav_key(&key) {
            Some(NavKey::ArrowDown) => {
                event.prevent_default();
                self.highlight_next();
            }
            Some(NavKey::ArrowUp) => {
                event.prevent_default();
                self.highlight_prev();
            }
            Some(NavKey::Home) => {
                event.prevent_default();
                self.highlight_first();
            }
            Some(NavKey::End) => {
                event.prevent_default();
                self.highlight_last();
            }
            Some(NavKey::Enter) => {
                event.prevent_default();
                if let Some(cand) = self.highlighted_value() {
                    self.select_item(&cand);
                }
            }
            Some(NavKey::Space) => {
                event.prevent_default();
                if self.has_active_typeahead(now_ms) {
                    self.handle_typeahead(' ', now_ms);
                } else if let Some(cand) = self.highlighted_value() {
                    self.select_item(&cand);
                }
            }
            Some(NavKey::Escape) => {
                event.prevent_default();
                self.close_dropdown();
            }
            Some(NavKey::Tab) => {
                // Do NOT prevent_default(), let browser naturally advance tab focus!
                self.close_dropdown_without_restore();
            }
            _ => {
                if let Key::Character(ref s) = key {
                    let mut chars = s.chars();
                    if let (Some(ch), None) = (chars.next(), chars.next())
                        && (ch.is_alphanumeric() || ch.is_whitespace())
                    {
                        self.handle_typeahead(ch, now_ms);
                    }
                }
            }
        }
    }
}

pub fn scroll_item_within_viewport(viewport_id: &str, item_id: &str) {
    scroll_item_within_container(viewport_id, viewport_id, item_id);
}

async fn try_scroll_item_within_container(
    viewport_id: &str,
    fallback_content_id: &str,
    item_id: &str,
) -> bool {
    let doc = oxidase::dom::document();

    let has_viewport_geometry =
        crate::foundation::browser::get_element_client_rect(viewport_id).is_some();
    let container_id = if has_viewport_geometry {
        viewport_id
    } else {
        fallback_content_id
    };
    let Some(container_rect) = crate::foundation::browser::get_element_client_rect(container_id)
    else {
        return false;
    };
    let Some(item_rect) = crate::foundation::browser::get_element_client_rect(item_id) else {
        return false;
    };
    if item_rect.height <= 0.0 || container_rect.height <= 0.0 {
        return false;
    }

    const EDGE_INSET: f64 = 1.0;
    let visible_bottom = container_rect.bottom() - EDGE_INSET;
    let delta_y = if item_rect.top() < container_rect.top() {
        item_rect.top() - container_rect.top()
    } else if item_rect.bottom() > visible_bottom {
        item_rect.bottom() - visible_bottom
    } else {
        return true;
    };

    let Some(container_el) = doc.element_by_id(container_id) else {
        return false;
    };
    container_el.scroll_by(0.0, delta_y).await.is_ok()
}

pub(crate) fn scroll_item_within_container(
    viewport_id: &str,
    fallback_content_id: &str,
    item_id: &str,
) {
    let viewport_id = viewport_id.to_owned();
    let fallback_content_id = fallback_content_id.to_owned();
    let item_id = item_id.to_owned();
    crate::foundation::browser::spawn_root_task(async move {
        for attempt in 0..3 {
            if try_scroll_item_within_container(&viewport_id, &fallback_content_id, &item_id).await
            {
                break;
            }
            if attempt < 2 {
                let _ = oxidase::frame::next_frame().await;
            }
        }
    });
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectViewportProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn SelectViewport(props: SelectViewportProps) -> Element {
    let SelectViewportProps {
        id,
        class,
        style,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<super::types::SelectContext>();
    let vp_id = id.unwrap_or_else(|| ctx.runtime.relationships().viewport_id());
    ctx.runtime.set_viewport_id(vp_id.clone());

    let scroll_guard = use_hook(|| Rc::new(RefCell::new(WatcherGuard::noop())));
    let resize_guard = use_hook(|| Rc::new(RefCell::new(WatcherGuard::noop())));
    let is_open = ctx.runtime.is_open();
    let reg_sig = ctx.runtime.state.registry;

    {
        let rt = ctx.runtime.clone();
        let sg = Rc::clone(&scroll_guard);
        let rg = Rc::clone(&resize_guard);
        use_effect(use_reactive((&vp_id, &is_open, &reg_sig), move |(id, open, _)| {
            if !open {
                let prev_s = std::mem::replace(&mut *sg.borrow_mut(), WatcherGuard::noop());
                prev_s.stop();
                let prev_r = std::mem::replace(&mut *rg.borrow_mut(), WatcherGuard::noop());
                prev_r.stop();
                return;
            }
            rt.sync_viewport_scroll_state();
            let rt_scroll = rt.clone();
            let next_s = crate::foundation::browser::start_element_scroll_monitor(&id, move || {
                rt_scroll.sync_viewport_scroll_state();
            });
            let prev_s = std::mem::replace(&mut *sg.borrow_mut(), next_s);
            prev_s.stop();

            let rt_resize = rt.clone();
            let next_r = crate::foundation::browser::start_resize_monitor(
                &id,
                ResizeObserverBox::BorderBox,
                move |_| {
                    rt_resize.sync_viewport_scroll_state();
                },
            );
            let prev_r = std::mem::replace(&mut *rg.borrow_mut(), next_r);
            prev_r.stop();
        }));
    }

    {
        let drop_rt = ctx.runtime.clone();
        let drop_vp_id = vp_id.clone();
        let sg = Rc::clone(&scroll_guard);
        let rg = Rc::clone(&resize_guard);
        use_drop(move || {
            let prev_s = std::mem::replace(&mut *sg.borrow_mut(), WatcherGuard::noop());
            prev_s.stop();
            let prev_r = std::mem::replace(&mut *rg.borrow_mut(), WatcherGuard::noop());
            prev_r.stop();
            if drop_rt.current_viewport_id() == drop_vp_id {
                drop_rt.set_viewport_id(drop_rt.relationships().viewport_id());
            }
        });
    }

    let default_style =
        "position: relative; flex: 1 1 auto; min-height: 0; overflow-x: hidden; overflow-y: auto;";

    let mut base = Vec::with_capacity(4);
    base.push(Attribute::new("id", vp_id, None, false));
    base.push(Attribute::new("style", default_style, None, false));
    push_event_handler(
        &mut base,
        "onscroll",
        Some(EventHandler::new({
            let rt = ctx.runtime.clone();
            move |_evt: ScrollEvent| {
                rt.sync_viewport_scroll_state();
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new({
            let rt = ctx.runtime.clone();
            move |_evt: MountedEvent| {
                rt.sync_viewport_scroll_state();
            }
        })),
    );

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

fn start_auto_scroll_loop(
    runtime: SelectRuntime,
    mut gen_slot: CopyValue<u64>,
    step_y: f64,
    is_up: bool,
) {
    let next_gen = gen_slot
        .try_read()
        .map(|g| g.wrapping_add(1))
        .unwrap_or(1);
    let _ = gen_slot.try_write().map(|mut g| *g = next_gen);

    crate::foundation::browser::spawn_root_task(async move {
        loop {
            let still_active = gen_slot.try_read().is_ok_and(|g| *g == next_gen);
            if !still_active {
                break;
            }
            let can_continue = if is_up {
                runtime.can_scroll_up()
            } else {
                runtime.can_scroll_down()
            };
            if !can_continue {
                break;
            }
            runtime.scroll_viewport_by(step_y);
            let _ = oxidase::frame::next_frame().await;
        }
    });
}

fn stop_auto_scroll_loop(mut gen_slot: CopyValue<u64>) {
    let next_gen = gen_slot
        .try_read()
        .map(|g| g.wrapping_add(1))
        .unwrap_or(1);
    let _ = gen_slot.try_write().map(|mut g| *g = next_gen);
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectScrollUpButtonProps {
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
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn SelectScrollUpButton(props: SelectScrollUpButtonProps) -> Element {
    let SelectScrollUpButtonProps {
        id,
        class,
        style,
        force_mount,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<super::types::SelectContext>();
    let gen_slot = use_hook(|| CopyValue::new(0_u64));
    use_drop(move || {
        stop_auto_scroll_loop(gen_slot);
    });

    let can_scroll = ctx.runtime.can_scroll_up();
    if !can_scroll && !force_mount {
        return rsx! {};
    }

    let mut base = Vec::with_capacity(8);
    push_attr_opt(&mut base, "id", id);
    base.push(Attribute::new("aria-hidden", "true", None, false));
    base.push(Attribute::new(
        "data-monoxus-select-scroll-up-button",
        "",
        None,
        false,
    ));
    push_event_handler(
        &mut base,
        "onpointerenter",
        Some(EventHandler::new({
            let rt = ctx.runtime.clone();
            move |evt: PointerEvent| {
                if evt.pointer_type() != "touch" {
                    start_auto_scroll_loop(rt.clone(), gen_slot, -5.0, true);
                }
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new({
            let rt = ctx.runtime.clone();
            move |_evt: PointerEvent| {
                start_auto_scroll_loop(rt.clone(), gen_slot, -5.0, true);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onpointerleave",
        Some(EventHandler::new(move |_evt: PointerEvent| {
            stop_auto_scroll_loop(gen_slot);
        })),
    );
    push_event_handler(
        &mut base,
        "onpointerup",
        Some(EventHandler::new(move |_evt: PointerEvent| {
            stop_auto_scroll_loop(gen_slot);
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new({
            let rt = ctx.runtime.clone();
            move |evt: MouseEvent| {
                evt.stop_propagation();
                rt.scroll_viewport_by(-40.0);
            }
        })),
    );

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            if let Some(c) = children {
                {c}
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectScrollDownButtonProps {
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
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn SelectScrollDownButton(props: SelectScrollDownButtonProps) -> Element {
    let SelectScrollDownButtonProps {
        id,
        class,
        style,
        force_mount,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<super::types::SelectContext>();
    let gen_slot = use_hook(|| CopyValue::new(0_u64));
    use_drop(move || {
        stop_auto_scroll_loop(gen_slot);
    });

    let can_scroll = ctx.runtime.can_scroll_down();
    if !can_scroll && !force_mount {
        return rsx! {};
    }

    let mut base = Vec::with_capacity(8);
    push_attr_opt(&mut base, "id", id);
    base.push(Attribute::new("aria-hidden", "true", None, false));
    base.push(Attribute::new(
        "data-monoxus-select-scroll-down-button",
        "",
        None,
        false,
    ));
    push_event_handler(
        &mut base,
        "onpointerenter",
        Some(EventHandler::new({
            let rt = ctx.runtime.clone();
            move |evt: PointerEvent| {
                if evt.pointer_type() != "touch" {
                    start_auto_scroll_loop(rt.clone(), gen_slot, 5.0, false);
                }
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new({
            let rt = ctx.runtime.clone();
            move |_evt: PointerEvent| {
                start_auto_scroll_loop(rt.clone(), gen_slot, 5.0, false);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onpointerleave",
        Some(EventHandler::new(move |_evt: PointerEvent| {
            stop_auto_scroll_loop(gen_slot);
        })),
    );
    push_event_handler(
        &mut base,
        "onpointerup",
        Some(EventHandler::new(move |_evt: PointerEvent| {
            stop_auto_scroll_loop(gen_slot);
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new({
            let rt = ctx.runtime.clone();
            move |evt: MouseEvent| {
                evt.stop_propagation();
                rt.scroll_viewport_by(40.0);
            }
        })),
    );

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            if let Some(c) = children {
                {c}
            }
        }
    }
}

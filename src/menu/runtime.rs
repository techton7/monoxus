use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::foundation::{
    browser::{
        DocumentDismissEvent, DocumentPointerStreamGuard, FocusOptions, ModalShieldGuard,
        PresenceMonitorEvent, WatcherGuard, acquire_scroll_lock_with_options,
        focus_element_by_id_with_options, focus_element_sync_or_defer, get_element_client_rect,
        get_element_scroll_metrics, is_element_focused_by_id, is_element_tabbable_by_id,
        query_descendant_order, release_scroll_lock, restore_focus_element_by_id_with_options,
        scroll_element_into_view_nearest, spawn_root_task,
        start_document_dismiss_monitor_with_boundaries, start_document_pointer_stream,
        start_floating_auto_update_monitor, start_presence_monitor, start_window_blur_monitor,
        suppress_external_dom, use_is_using_keyboard,
    },
    overlay::{
        FloatingPlacement, FloatingReadiness, OverlayAnchor, PlacementAlign, PlacementSide,
        Point2D, PresenceCloseCycleId, PresenceController, PresenceState, StickyStrategy,
        VirtualOverlayPositionConfig, compute_virtual_overlay_position,
    },
    shared::Direction,
};

use super::{
    relationships::MenuRelationships,
    state::{
        SubmenuIntentMachine, TypeaheadBuffer, first_enabled_item_id, last_enabled_item_id,
        menu_css_vars, resolve_menu_navigation,
    },
    types::{
        CheckedState, MENU_DEFAULT_SUBMENU_OPEN_DELAY_MS, MENU_TYPEAHEAD_TIMEOUT_MS,
        MenuItemEntry, MenuKey, MenuNavigationStep, MenuOpenFocusTarget, MenuOutsideEvent,
        MenuPreventableEvent, MenuVariant, PressedItemSnapshot, SubmenuPointerEvaluation,
        translate_menu_key,
    },
};

const TABBABLE_SELECTOR: &str = "a[href], area[href], button:not([disabled]), input:not([disabled]):not([type=\"hidden\"]), select:not([disabled]), textarea:not([disabled]), summary, iframe, [tabindex]:not([tabindex=\"-1\"]), [contenteditable]:not([contenteditable=\"false\"])";

/// Hook provided by `Menubar` when a menu instance belongs to a horizontal `Menubar` (`MB-4`, `MB-6`).
#[derive(Clone)]
pub struct MenubarMenuBridge {
    pub menubar_root_id: String,
    pub menu_value: String,
    pub all_trigger_ids: Rc<dyn Fn() -> Vec<String>>,
    pub on_cross_menu_arrow: Rc<dyn Fn(MenuKey, bool, bool) -> bool>,
    pub close_menubar: Rc<dyn Fn()>,
}

/// Active child submenu currently registered for hover grace corridor tracking (`M-11`).
#[derive(Clone)]
pub struct ActiveCorridorSubmenu {
    pub sub_id: String,
    pub trigger_id: String,
    pub content_id: String,
    pub side: Signal<PlacementSide>,
    pub close_sub: Rc<dyn Fn()>,
}

/// Shared root context provided by `DropdownMenuRoot`, `ContextMenuRoot`, `MenubarMenu`, or internal `MenuRoot`.
#[derive(Clone)]
pub struct MenuRootContext {
    pub variant: MenuVariant,
    pub relationships: MenuRelationships,
    pub open: Signal<bool>,
    pub is_controlled: bool,
    pub on_open_change: Option<EventHandler<bool>>,
    pub on_open_change_complete: Option<EventHandler<bool>>,
    pub modal: Signal<bool>,
    pub dir: Signal<Direction>,
    pub loop_focus: Signal<bool>,
    pub trigger_id: Signal<String>,
    pub anchor_id: Signal<Option<String>>,
    pub virtual_anchor: Signal<Option<(f64, f64)>>,
    pub open_focus_target: Signal<MenuOpenFocusTarget>,
    pub previously_focused_id: Signal<Option<String>>,
    pub ignore_close_auto_focus: Signal<bool>,
    pub skip_exit_animation: Signal<bool>,
    pub using_keyboard: Signal<bool>,
    pub last_pointer_type: Signal<Option<String>>,
    pub submenu_open_delay_ms: Signal<u64>,
    pub root_surface: MenuSurfaceRuntime,
    pub menubar_bridge: Option<MenubarMenuBridge>,
    pub child_close_handlers: Signal<Vec<(String, Rc<dyn Fn()>)>>,
}

impl MenuRootContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        variant: MenuVariant,
        relationships: MenuRelationships,
        open: Signal<bool>,
        is_controlled: bool,
        on_open_change: Option<EventHandler<bool>>,
        on_open_change_complete: Option<EventHandler<bool>>,
        modal: bool,
        dir: Direction,
        loop_focus: bool,
        submenu_open_delay_ms: Option<u64>,
        menubar_bridge: Option<MenubarMenuBridge>,
    ) -> Self {
        let using_keyboard = use_is_using_keyboard();
        let root_surface = MenuSurfaceRuntime::new(
            relationships.root_id().to_owned(),
            false,
            open,
            relationships.content_id().to_owned(),
            relationships.wrapper_id().to_owned(),
            relationships.trigger_id().to_owned(),
            PlacementSide::Bottom,
            PlacementAlign::Start,
            4.0,
            0.0,
            loop_focus,
        );

        Self {
            variant,
            relationships: relationships.clone(),
            open,
            is_controlled,
            on_open_change,
            on_open_change_complete,
            modal: use_signal(|| modal),
            dir: use_signal(|| dir),
            loop_focus: use_signal(|| loop_focus),
            trigger_id: use_signal(|| relationships.trigger_id().to_owned()),
            anchor_id: use_signal(|| None),
            virtual_anchor: use_signal(|| None),
            open_focus_target: use_signal(|| MenuOpenFocusTarget::ContentContainer),
            previously_focused_id: use_signal(|| None),
            ignore_close_auto_focus: use_signal(|| false),
            skip_exit_animation: use_signal(|| false),
            using_keyboard,
            last_pointer_type: use_signal(|| None),
            submenu_open_delay_ms: use_signal(|| {
                submenu_open_delay_ms.unwrap_or(MENU_DEFAULT_SUBMENU_OPEN_DELAY_MS)
            }),
            root_surface,
            menubar_bridge,
            child_close_handlers: use_signal(Vec::new),
        }
    }

    pub fn is_open(&self) -> bool {
        self.open.try_read().map(|g| *g).unwrap_or(false)
    }

    pub fn is_modal(&self) -> bool {
        self.modal.try_read().map(|g| *g).unwrap_or(true)
    }

    pub fn direction(&self) -> Direction {
        self.dir.try_read().map(|g| *g).unwrap_or(Direction::Ltr)
    }

    pub fn resolved_anchor_id(&self) -> String {
        self.anchor_id
            .try_read()
            .ok()
            .and_then(|a| a.clone())
            .unwrap_or_else(|| {
                self.trigger_id
                    .try_read()
                    .map(|g| g.clone())
                    .unwrap_or_else(|_| self.relationships.trigger_id().to_owned())
            })
    }

    pub fn request_open_change(&self, next: bool) {
        if self.is_open() == next {
            return;
        }
        if !self.is_controlled {
            let mut open_sig = self.open;
            if let Ok(mut w) = open_sig.try_write() {
                *w = next;
            }
        }
        if let Some(cb) = self.on_open_change.as_ref() {
            cb.call(next);
        }
    }

    pub fn close_submenus(&self) {
        if let Ok(handlers) = self.child_close_handlers.try_read() {
            for (_, close_fn) in handlers.iter() {
                close_fn();
            }
        }
    }

    pub fn close_menu_tree(&self, focus_trigger: bool) {
        let mut ignore = self.ignore_close_auto_focus;
        if let Ok(mut w) = ignore.try_write() {
            *w = !focus_trigger;
        }
        self.close_submenus();
        self.request_open_change(false);
    }

    pub fn register_sub_close_handler(&self, id: String, close_fn: Rc<dyn Fn()>) {
        let mut sig = self.child_close_handlers;
        if let Ok(mut list) = sig.try_write() {
            list.retain(|(existing, _)| existing != &id);
            list.push((id, close_fn));
        }
    }

    pub fn unregister_sub_close_handler(&self, id: &str) {
        let mut sig = self.child_close_handlers;
        if let Ok(mut list) = sig.try_write() {
            list.retain(|(existing, _)| existing != id);
        }
    }

    pub fn focus_root_trigger(&self) {
        let target = self
            .trigger_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.trigger_id().to_owned());
        let focus_visible = self.using_keyboard.try_read().map(|g| *g).unwrap_or(false);
        let opts = FocusOptions::with_focus_visible(false, Some(focus_visible));
        restore_focus_element_by_id_with_options(&target, opts);
    }

    pub fn css_vars(&self, placement: &FloatingPlacement) -> Vec<(String, String)> {
        menu_css_vars(placement.geometry(), self.variant)
    }
}

/// Shared surface runtime owning item roving registry, typeahead, positioning, dismiss, and grace corridors.
#[derive(Clone)]
pub struct MenuSurfaceRuntime {
    pub surface_key: String,
    pub is_submenu: bool,
    pub open: Signal<bool>,
    pub content_id: Signal<String>,
    pub wrapper_id: Signal<String>,
    pub portal_id: Signal<Option<String>>,
    pub arrow_id: Signal<Option<String>>,
    pub anchor_id: Signal<String>,
    pub items: Signal<Vec<MenuItemEntry>>,
    pub highlighted_id: Signal<Option<String>>,
    pub pressed_item: Signal<Option<PressedItemSnapshot>>,
    pub pointer_up_consumed_click: Signal<Option<String>>,
    pub typeahead: Signal<TypeaheadBuffer>,
    pub typeahead_gen: Rc<Cell<u64>>,
    pub typeahead_task: Rc<RefCell<Option<dioxus::core::Task>>>,
    pub active_submenu_id: Signal<Option<String>>,
    pub active_submenu_close: Signal<Option<(String, Rc<dyn Fn()>)>>,
    pub active_corridor_sub: Signal<Option<ActiveCorridorSubmenu>>,
    pub child_subcontents: Signal<Vec<String>>,
    pub branches: Signal<Vec<String>>,
    pub registered_sub_triggers: Signal<Vec<(String, Rc<dyn Fn()>, Rc<dyn Fn()>)>>,
    pub on_close_submenu: Signal<Option<Rc<dyn Fn()>>>,
    pub submenu_intent: Signal<SubmenuIntentMachine>,
    pub corridor_gen: Rc<Cell<u64>>,
    pub corridor_task: Rc<RefCell<Option<dioxus::core::Task>>>,
    pub pointer_stream: Signal<Option<DocumentPointerStreamGuard>>,
    pub presence_lane: Signal<PresenceController>,
    pub presence_monitor: Signal<Option<WatcherGuard>>,
    pub active_close_cycle: Signal<Option<PresenceCloseCycleId>>,
    pub placement: Signal<Option<FloatingPlacement>>,
    pub readiness: Signal<FloatingReadiness>,
    pub content_z_index: Signal<Option<String>>,
    pub position_monitor: Signal<Option<WatcherGuard>>,
    pub dismiss_monitor: Signal<Option<WatcherGuard>>,
    pub blur_monitor: Signal<Option<WatcherGuard>>,
    pub modal_shield: Signal<Option<ModalShieldGuard>>,
    pub scroll_lock_held: Signal<bool>,
    pub open_focus_target: Signal<MenuOpenFocusTarget>,
    pub open_focus_gen: Rc<Cell<u64>>,
    pub side: Signal<PlacementSide>,
    pub align: Signal<PlacementAlign>,
    pub side_offset: Signal<f32>,
    pub align_offset: Signal<f32>,
    pub avoid_collisions: Signal<bool>,
    pub collision_padding: Signal<f32>,
    pub arrow_padding: Signal<f32>,
    pub sticky: Signal<StickyStrategy>,
    pub hide_when_detached: Signal<bool>,
    pub force_mount: Signal<bool>,
    pub prevent_scroll: Signal<bool>,
    pub loop_focus: Signal<bool>,
    pub focus_on_open_armed: Signal<bool>,
    pub on_escape_keydown: Signal<Option<EventHandler<MenuPreventableEvent>>>,
    pub on_pointer_down_outside: Signal<Option<EventHandler<MenuOutsideEvent>>>,
    pub on_focus_outside: Signal<Option<EventHandler<MenuOutsideEvent>>>,
    pub on_interact_outside: Signal<Option<EventHandler<MenuOutsideEvent>>>,
    pub on_open_auto_focus: Signal<Option<EventHandler<MenuPreventableEvent>>>,
    pub on_close_auto_focus: Signal<Option<EventHandler<MenuPreventableEvent>>>,
}

impl MenuSurfaceRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        surface_key: String,
        is_submenu: bool,
        open: Signal<bool>,
        content_id: String,
        wrapper_id: String,
        anchor_id: String,
        default_side: PlacementSide,
        default_align: PlacementAlign,
        default_side_offset: f32,
        default_align_offset: f32,
        loop_focus: bool,
    ) -> Self {
        let initial_open = open.try_read().map(|g| *g).unwrap_or(false);
        Self {
            surface_key,
            is_submenu,
            open,
            content_id: use_signal(|| content_id),
            wrapper_id: use_signal(|| wrapper_id),
            portal_id: use_signal(|| None),
            arrow_id: use_signal(|| None),
            anchor_id: use_signal(|| anchor_id),
            items: use_signal(Vec::new),
            highlighted_id: use_signal(|| None),
            pressed_item: use_signal(|| None),
            pointer_up_consumed_click: use_signal(|| None),
            typeahead: use_signal(TypeaheadBuffer::default),
            typeahead_gen: use_hook(|| Rc::new(Cell::new(0))),
            typeahead_task: use_hook(|| Rc::new(RefCell::new(None))),
            active_submenu_id: use_signal(|| None),
            active_submenu_close: use_signal(|| None),
            active_corridor_sub: use_signal(|| None),
            child_subcontents: use_signal(Vec::new),
            branches: use_signal(Vec::new),
            registered_sub_triggers: use_signal(Vec::new),
            on_close_submenu: use_signal(|| None),
            submenu_intent: use_signal(SubmenuIntentMachine::default),
            corridor_gen: use_hook(|| Rc::new(Cell::new(0))),
            corridor_task: use_hook(|| Rc::new(RefCell::new(None))),
            pointer_stream: use_signal(|| None),
            presence_lane: use_signal(|| {
                PresenceController::new(initial_open).with_retained_mount(true)
            }),
            presence_monitor: use_signal(|| None),
            active_close_cycle: use_signal(|| None),
            placement: use_signal(|| None),
            readiness: use_signal(FloatingReadiness::default),
            content_z_index: use_signal(|| None),
            position_monitor: use_signal(|| None),
            dismiss_monitor: use_signal(|| None),
            blur_monitor: use_signal(|| None),
            modal_shield: use_signal(|| None),
            scroll_lock_held: use_signal(|| false),
            open_focus_target: use_signal(|| MenuOpenFocusTarget::ContentContainer),
            open_focus_gen: use_hook(|| Rc::new(Cell::new(0))),
            side: use_signal(|| default_side),
            align: use_signal(|| default_align),
            side_offset: use_signal(|| default_side_offset),
            align_offset: use_signal(|| default_align_offset),
            avoid_collisions: use_signal(|| true),
            collision_padding: use_signal(|| 0.0),
            arrow_padding: use_signal(|| 0.0),
            sticky: use_signal(|| StickyStrategy::Partial),
            hide_when_detached: use_signal(|| false),
            force_mount: use_signal(|| false),
            prevent_scroll: use_signal(|| false),
            loop_focus: use_signal(|| loop_focus),
            focus_on_open_armed: use_signal(|| false),
            on_escape_keydown: use_signal(|| None),
            on_pointer_down_outside: use_signal(|| None),
            on_focus_outside: use_signal(|| None),
            on_interact_outside: use_signal(|| None),
            on_open_auto_focus: use_signal(|| None),
            on_close_auto_focus: use_signal(|| None),
        }
    }

    pub fn should_render_content(&self) -> bool {
        let force = self.force_mount.try_read().map(|g| *g).unwrap_or(false);
        let is_open = self.open.try_read().map(|g| *g).unwrap_or(false);
        force
            || is_open
            || self
                .presence_lane
                .try_read()
                .map(|p| p.should_render())
                .unwrap_or(false)
    }

    pub fn current_content_id(&self) -> String {
        self.content_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn current_wrapper_id(&self) -> String {
        self.wrapper_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn current_highlighted_id(&self) -> Option<String> {
        self.highlighted_id.try_read().ok().and_then(|g| g.clone())
    }

    pub fn register_item(&self, entry: MenuItemEntry) {
        let already_exact = self
            .items
            .try_peek()
            .is_ok_and(|items| items.iter().any(|e| *e == entry));
        if already_exact {
            return;
        }
        let mut items_sig = self.items;
        if let Ok(mut items) = items_sig.try_write() {
            if let Some(existing) = items.iter_mut().find(|e| e.id == entry.id) {
                *existing = entry;
            } else {
                items.push(entry);
            }
        }
        self.sync_items_dom_order();
    }

    pub fn unregister_item(&self, id: &str) {
        let present = self
            .items
            .try_peek()
            .is_ok_and(|items| items.iter().any(|e| e.id == id));
        if present {
            let mut items_sig = self.items;
            if let Ok(mut items) = items_sig.try_write() {
                items.retain(|e| e.id != id);
            }
        }
        if self.current_highlighted_id().as_deref() == Some(id) {
            let mut hl = self.highlighted_id;
            if let Ok(mut w) = hl.try_write() {
                *w = None;
            }
        }
    }

    pub fn sync_items_dom_order(&self) {
        let content_id = self
            .content_id
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_default();
        if content_id.is_empty() {
            return;
        }
        let ids: Vec<String> = self
            .items
            .try_peek()
            .map(|items| items.iter().map(|i| i.id.clone()).collect())
            .unwrap_or_default();
        if ids.len() <= 1 {
            return;
        }
        let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        if let Ok(ordered) = query_descendant_order(&content_id, &id_refs) {
            let order_changed = self.items.try_peek().is_ok_and(|items| {
                items.iter().enumerate().any(|(idx, item)| {
                    let next_pos = ordered
                        .iter()
                        .position(|id| id == &item.id)
                        .unwrap_or(item.dom_order);
                    next_pos != item.dom_order
                        || (idx > 0 && items[idx - 1].dom_order > next_pos)
                })
            });
            if !order_changed {
                return;
            }
            let mut items_sig = self.items;
            if let Ok(mut items) = items_sig.try_write() {
                for item in items.iter_mut() {
                    if let Some(pos) = ordered.iter().position(|id| id == &item.id) {
                        item.dom_order = pos;
                    }
                }
                items.sort_by_key(|item| item.dom_order);
            }
        }
    }

    pub fn register_child_subcontent(&self, id: String) {
        let mut sig = self.child_subcontents;
        if let Ok(mut list) = sig.try_write() {
            if !list.contains(&id) {
                list.push(id);
            }
        }
    }

    pub fn unregister_child_subcontent(&self, id: &str) {
        let mut sig = self.child_subcontents;
        if let Ok(mut list) = sig.try_write() {
            list.retain(|entry| entry != id);
        }
    }

    pub fn register_branch(&self, id: impl Into<String>) {
        let id = id.into();
        let mut sig = self.branches;
        let already = sig.try_peek().is_ok_and(|list| list.contains(&id));
        if !already {
            if let Ok(mut list) = sig.try_write() {
                if !list.contains(&id) {
                    list.push(id);
                }
            }
        }
    }

    pub fn unregister_branch(&self, id: &str) {
        let mut sig = self.branches;
        let present = sig
            .try_peek()
            .is_ok_and(|list| list.iter().any(|entry| entry == id));
        if present {
            if let Ok(mut list) = sig.try_write() {
                list.retain(|entry| entry != id);
            }
        }
    }

    pub fn branches(&self) -> Vec<String> {
        self.branches
            .try_read()
            .map(|list| list.clone())
            .unwrap_or_default()
    }

    pub fn register_sub_trigger_open(
        &self,
        trigger_id: String,
        hover_open_fn: Rc<dyn Fn()>,
        keyboard_open_fn: Rc<dyn Fn()>,
    ) {
        let mut sig = self.registered_sub_triggers;
        if let Ok(mut list) = sig.try_write() {
            list.retain(|(id, _, _)| id != &trigger_id);
            list.push((trigger_id, hover_open_fn, keyboard_open_fn));
        }
    }

    pub fn unregister_sub_trigger_open(&self, trigger_id: &str) {
        let mut sig = self.registered_sub_triggers;
        if let Ok(mut list) = sig.try_write() {
            list.retain(|(id, _, _)| id != trigger_id);
        }
    }

    pub fn close_other_child_submenus(&self, except_sub_id: Option<&str>) {
        let active = self
            .active_submenu_close
            .try_read()
            .ok()
            .and_then(|g| g.clone());
        if let Some((active_id, close_fn)) = active
            && except_sub_id != Some(active_id.as_str())
        {
            close_fn();
        }
    }

    pub fn is_submenu_grace_blocking(&self, target_item_id: Option<&str>) -> bool {
        let active_corridor = self
            .submenu_intent
            .try_read()
            .map(|m| m.is_active())
            .unwrap_or(false);
        if !active_corridor {
            return false;
        }
        if let (Some(target), Some(sub)) = (
            target_item_id,
            self.active_corridor_sub.try_read().ok().and_then(|g| g.clone()),
        ) {
            if target == sub.trigger_id {
                return false;
            }
        }
        true
    }

    pub fn highlight_item(&self, id: Option<String>, focus_dom: bool, using_keyboard: bool) {
        let mut hl = self.highlighted_id;
        let changed = hl.try_peek().map(|g| *g != id).unwrap_or(true);
        if changed {
            if let Ok(mut w) = hl.try_write() {
                *w = id.clone();
            }
        }
        if focus_dom {
            if let Some(target_id) = id {
                let prevent_scroll = !using_keyboard;
                let focus_opts =
                    FocusOptions::with_focus_visible(prevent_scroll, Some(using_keyboard));
                focus_element_reliable(&target_id, focus_opts);
                if using_keyboard {
                    let cid = self.current_content_id();
                    scroll_menu_item_into_view(&cid, &target_id);
                }
            } else {
                let cid = self.current_content_id();
                if !cid.is_empty() {
                    let focus_opts = FocusOptions::with_focus_visible(true, Some(using_keyboard));
                    focus_element_reliable(&cid, focus_opts);
                }
            }
        }
    }

    pub fn apply_open_focus(&self, root: &MenuRootContext) {
        let expected_gen = self.open_focus_gen.get().wrapping_add(1);
        self.open_focus_gen.set(expected_gen);

        let target_policy = self
            .open_focus_target
            .try_peek()
            .map(|g| *g)
            .unwrap_or(MenuOpenFocusTarget::ContentContainer);
        if target_policy == MenuOpenFocusTarget::None {
            return;
        }

        let evt = MenuPreventableEvent::new();
        if let Some(cb) = self
            .on_open_auto_focus
            .try_peek()
            .ok()
            .and_then(|g| g.clone())
        {
            cb.call(evt.clone());
        }
        if evt.is_default_prevented() {
            return;
        }

        let using_kb = root.using_keyboard.try_peek().map(|g| *g).unwrap_or(false);
        let surface = self.clone();
        let gen_ref = self.open_focus_gen.clone();

        spawn_root_task(async move {
            for attempt in 0..5 {
                let _ = oxidase::frame::next_frame().await;
                if gen_ref.get() != expected_gen
                    || !surface.open.try_peek().is_ok_and(|o| *o)
                {
                    return;
                }
                let content_id = surface
                    .content_id
                    .try_peek()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                if content_id.is_empty() {
                    continue;
                }
                surface.sync_items_dom_order();
                let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::dom::document);
                let Some(el) = doc.element_by_id(&content_id) else {
                    continue;
                };
                if !el.is_connected() {
                    continue;
                }
                let items = surface
                    .items
                    .try_peek()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                let focus_target = match target_policy {
                    MenuOpenFocusTarget::FirstItem => first_enabled_item_id(&items),
                    MenuOpenFocusTarget::LastItem => last_enabled_item_id(&items),
                    MenuOpenFocusTarget::ContentContainer => None,
                    MenuOpenFocusTarget::None => return,
                };
                if matches!(
                    target_policy,
                    MenuOpenFocusTarget::FirstItem | MenuOpenFocusTarget::LastItem
                ) && focus_target.is_none()
                    && attempt < 4
                {
                    continue;
                }
                surface.highlight_item(focus_target, true, using_kb);
                break;
            }
        });
    }

    pub fn is_typeahead_active(&self) -> bool {
        let now_ms = monotonic_ms();
        self.typeahead
            .try_read()
            .map(|buf| buf.is_active(now_ms))
            .unwrap_or(false)
    }

    pub fn reset_typeahead(&self) {
        let mut ta = self.typeahead;
        if let Ok(mut w) = ta.try_write() {
            w.reset();
        }
    }

    pub fn push_typeahead_char(&self, ch: char, using_keyboard: bool) -> bool {
        self.sync_items_dom_order();
        let now_ms = monotonic_ms();
        let items = self
            .items
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_default();
        let current = self.current_highlighted_id();

        let matched = {
            let mut ta = self.typeahead;
            let Ok(mut buf) = ta.try_write() else {
                return false;
            };
            buf.push_char(ch, now_ms, &items, current.as_deref())
        };

        if let Some(target_id) = matched {
            self.highlight_item(Some(target_id), true, using_keyboard);
        }

        let is_active = self.is_typeahead_active();
        if is_active {
            let next_gen = self.typeahead_gen.get().wrapping_add(1);
            self.typeahead_gen.set(next_gen);
            if let Some(prev) = self.typeahead_task.borrow_mut().take() {
                prev.cancel();
            }
            let gen_ref = self.typeahead_gen.clone();
            let surface = self.clone();
            let task = spawn_root_task(async move {
                Delay::new(Duration::from_millis(MENU_TYPEAHEAD_TIMEOUT_MS)).await;
                if gen_ref.get() == next_gen {
                    surface.reset_typeahead();
                }
            });
            *self.typeahead_task.borrow_mut() = Some(task);
            true
        } else {
            false
        }
    }

    pub fn start_corridor(
        &self,
        sub: ActiveCorridorSubmenu,
        exit_point: Point2D,
        from_content_to_trigger: bool,
    ) {
        let Some(trigger_rect) = get_element_client_rect(&sub.trigger_id) else {
            return;
        };
        let Some(content_rect) = get_element_client_rect(&sub.content_id) else {
            return;
        };
        let side = sub.side.try_read().map(|g| *g).unwrap_or(PlacementSide::Right);
        let now_ms = monotonic_ms();

        {
            let mut intent_sig = self.submenu_intent;
            if let Ok(mut intent) = intent_sig.try_write() {
                if from_content_to_trigger {
                    intent.begin_content_to_trigger(
                        exit_point,
                        content_rect,
                        trigger_rect,
                        side,
                        now_ms,
                    );
                } else {
                    intent.begin_trigger_to_sub(
                        exit_point,
                        trigger_rect,
                        content_rect,
                        side,
                        now_ms,
                    );
                }
            }
        }

        let mut active_sig = self.active_corridor_sub;
        if let Ok(mut w) = active_sig.try_write() {
            *w = Some(sub.clone());
        }

        let next_gen = self.corridor_gen.get().wrapping_add(1);
        self.corridor_gen.set(next_gen);
        if let Some(prev) = self.corridor_task.borrow_mut().take() {
            prev.cancel();
        }

        let surface_stream = self.clone();
        let guard = start_document_pointer_stream(move |point| {
            surface_stream.evaluate_corridor_pointer(point);
        });
        let mut ps = self.pointer_stream;
        if let Ok(mut w) = ps.try_write() {
            *w = Some(guard);
        }

        let surface_timer = self.clone();
        let gen_ref = self.corridor_gen.clone();
        let timeout_ms = self
            .submenu_intent
            .try_read()
            .map(|m| m.config().fallback_timeout_ms)
            .unwrap_or(100);
        let task = spawn_root_task(async move {
            Delay::new(Duration::from_millis(timeout_ms + 5)).await;
            if gen_ref.get() != next_gen {
                return;
            }
            let status = {
                let mut intent_sig = surface_timer.submenu_intent;
                intent_sig
                    .try_write()
                    .map(|mut m| m.check_timeout(monotonic_ms()))
                    .unwrap_or(SubmenuPointerEvaluation::Idle)
            };
            if matches!(
                status,
                SubmenuPointerEvaluation::Stalled | SubmenuPointerEvaluation::ExitedCorridor
            ) {
                surface_timer.disengage_corridor_and_close(None);
            }
        });
        *self.corridor_task.borrow_mut() = Some(task);
    }

    pub fn clear_corridor(&self) {
        self.corridor_gen.set(self.corridor_gen.get().wrapping_add(1));
        if let Some(task) = self.corridor_task.borrow_mut().take() {
            task.cancel();
        }
        let mut ps = self.pointer_stream;
        if let Ok(mut w) = ps.try_write() {
            *w = None;
        }
        let mut intent_sig = self.submenu_intent;
        if let Ok(mut m) = intent_sig.try_write() {
            m.clear();
        }
        let mut active_sig = self.active_corridor_sub;
        if let Ok(mut w) = active_sig.try_write() {
            *w = None;
        }
    }

    fn evaluate_corridor_pointer(&self, point: Point2D) {
        let Some(sub) = self
            .active_corridor_sub
            .try_read()
            .ok()
            .and_then(|g| g.clone())
        else {
            self.clear_corridor();
            return;
        };

        let in_own_trigger = get_element_client_rect(&sub.trigger_id)
            .is_some_and(|r| r.contains_point(point));
        let in_sub_content = get_element_client_rect(&sub.content_id)
            .is_some_and(|r| r.contains_point(point));
        let in_descendant = self
            .child_subcontents
            .try_read()
            .map(|ids| {
                ids.iter().any(|cid| {
                    get_element_client_rect(cid).is_some_and(|r| r.contains_point(point))
                })
            })
            .unwrap_or(false);

        let eval = {
            let mut intent_sig = self.submenu_intent;
            let Ok(mut intent) = intent_sig.try_write() else {
                return;
            };
            intent.evaluate_pointer_move(
                point,
                monotonic_ms(),
                in_sub_content || in_descendant,
                in_own_trigger,
                false,
            )
        };

        match eval {
            SubmenuPointerEvaluation::ReachedTarget => {
                self.clear_corridor();
            }
            SubmenuPointerEvaluation::ExitedCorridor | SubmenuPointerEvaluation::Stalled => {
                self.disengage_corridor_and_close(Some(point));
            }
            SubmenuPointerEvaluation::Idle | SubmenuPointerEvaluation::InCorridor => {}
        }
    }

    fn disengage_corridor_and_close(&self, pointer: Option<Point2D>) {
        let sub = self
            .active_corridor_sub
            .try_read()
            .ok()
            .and_then(|g| g.clone());
        self.clear_corridor();
        if let Some(sub) = sub {
            (sub.close_sub)();
        }
        if let Some(pt) = pointer {
            let items = self
                .items
                .try_read()
                .map(|g| g.clone())
                .unwrap_or_default();
            for item in items {
                if !item.disabled
                    && get_element_client_rect(&item.id).is_some_and(|r| r.contains_point(pt))
                {
                    self.highlight_item(Some(item.id.clone()), true, false);
                    if let Ok(triggers) = self.registered_sub_triggers.try_read()
                        && let Some((_, hover_open_fn, _)) =
                            triggers.iter().find(|(tid, _, _)| tid == &item.id)
                    {
                        hover_open_fn();
                    }
                    break;
                }
            }
        }
    }

    pub fn handle_keydown(&self, evt: &KeyboardEvent, root: &MenuRootContext) {
        if evt.modifiers().ctrl() || evt.modifiers().alt() || evt.modifiers().meta() {
            return;
        }
        let Some(key) = translate_menu_key(&evt.key()) else {
            return;
        };
        let using_kb = true;
        let is_tab_shift = evt.modifiers().shift();
        self.sync_items_dom_order();

        let items = self
            .items
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_default();
        let highlighted_id = self.current_highlighted_id();
        let highlighted_entry = highlighted_id
            .as_deref()
            .and_then(|hid| items.iter().find(|i| i.id == hid));
        let on_sub_trigger = highlighted_entry
            .is_some_and(|entry| !entry.disabled && entry.kind == super::types::MenuItemKind::SubTrigger);

        // Menubar cross-menu ArrowLeft / ArrowRight hook (`MB-6`)
        if let Some(bridge) = root.menubar_bridge.as_ref()
            && (bridge.on_cross_menu_arrow)(key, on_sub_trigger, self.is_submenu)
        {
            evt.prevent_default();
            evt.stop_propagation();
            return;
        }

        // Escape dismisses full menu tree back to root trigger (`3B D-2`)
        if matches!(key, MenuKey::Escape) {
            let root_cid = root.root_surface.current_content_id();
            if crate::foundation::overlay::is_overlay_escape_consumed()
                || !crate::foundation::overlay::is_topmost_overlay_layer(&root_cid)
            {
                return;
            }
            evt.prevent_default();
            evt.stop_propagation();
            let esc_evt = MenuPreventableEvent::new();
            if let Some(cb) = self
                .on_escape_keydown
                .try_read()
                .ok()
                .and_then(|g| g.clone())
            {
                cb.call(esc_evt.clone());
            }
            crate::foundation::overlay::mark_overlay_escape_consumed();
            if !esc_evt.is_default_prevented() {
                root.close_menu_tree(true);
            }
            return;
        }

        // Typeahead search (Space only continues an active typeahead search; empty Space triggers selection)
        match key {
            MenuKey::Space if self.is_typeahead_active() => {
                evt.prevent_default();
                evt.stop_propagation();
                self.push_typeahead_char(' ', using_kb);
                return;
            }
            MenuKey::Char(ch) => {
                evt.prevent_default();
                evt.stop_propagation();
                self.push_typeahead_char(ch, using_kb);
                return;
            }
            _ => {}
        }

        let loop_focus = self.loop_focus.try_read().map(|g| *g).unwrap_or(true);
        let step = resolve_menu_navigation(
            &items,
            highlighted_id.as_deref(),
            key,
            root.direction(),
            loop_focus,
            root.is_modal(),
            self.is_submenu,
            is_tab_shift,
        );

        match step {
            MenuNavigationStep::None => {}
            MenuNavigationStep::HighlightItem { id } => {
                evt.prevent_default();
                evt.stop_propagation();
                self.highlight_item(Some(id), true, using_kb);
            }
            MenuNavigationStep::FocusContentContainer => {
                evt.prevent_default();
                evt.stop_propagation();
                self.highlight_item(None, true, using_kb);
            }
            MenuNavigationStep::TabTrap => {
                evt.prevent_default();
                evt.stop_propagation();
            }
            MenuNavigationStep::TabOut { forward } => {
                evt.prevent_default();
                evt.stop_propagation();
                let trigger_id = root
                    .trigger_id
                    .try_read()
                    .map(|g| g.clone())
                    .unwrap_or_else(|_| root.relationships.trigger_id().to_owned());
                let open_menu_ids = collect_open_menu_ids(root);
                focus_adjacent_tabbable(&trigger_id, &open_menu_ids, forward);
                root.close_menu_tree(false);
            }
            MenuNavigationStep::OpenSubmenuAndFocusFirst { sub_trigger_id } => {
                evt.prevent_default();
                evt.stop_propagation();
                if let Ok(triggers) = self.registered_sub_triggers.try_read()
                    && let Some((_, _, kb_open_fn)) =
                        triggers.iter().find(|(tid, _, _)| tid == &sub_trigger_id)
                {
                    kb_open_fn();
                }
            }
            MenuNavigationStep::CloseSubmenuAndFocusTrigger => {
                evt.prevent_default();
                evt.stop_propagation();
                if let Some(close_fn) = self
                    .on_close_submenu
                    .try_read()
                    .ok()
                    .and_then(|g| g.clone())
                {
                    close_fn();
                }
            }
            MenuNavigationStep::CloseRootMenu => {
                evt.prevent_default();
                evt.stop_propagation();
                root.close_menu_tree(true);
            }
        }
    }

    pub fn sync_content_z_index(&self) {
        let content_id = self.current_content_id();
        let next = crate::foundation::overlay::normalize_hoisted_z_index(
            crate::foundation::browser::get_element_computed_z_index(&content_id).as_deref(),
        );
        let mut sig = self.content_z_index;
        if sig.try_peek().is_ok_and(|cur| *cur != next) {
            if let Ok(mut w) = sig.try_write() {
                *w = next;
            }
        }
    }

    pub fn clear_content_z_index(&self) {
        let mut sig = self.content_z_index;
        if sig.try_peek().is_ok_and(|cur| cur.is_some()) {
            if let Ok(mut w) = sig.try_write() {
                *w = None;
            }
        }
    }

    pub fn update_placement(&self, root: &MenuRootContext) {
        if !self.should_render_content() {
            return;
        }

        self.sync_content_z_index();

        let content_id = self.current_content_id();
        let wrapper_id = self.current_wrapper_id();
        let arrow_id = self.arrow_id.try_peek().ok().and_then(|g| g.clone());
        let anchor_id = self
            .anchor_id
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_default();

        let virtual_point = if !self.is_submenu && root.variant == MenuVariant::ContextMenu {
            root.virtual_anchor.try_peek().ok().and_then(|g| *g)
        } else {
            None
        };

        let anchor = if let Some((x, y)) = virtual_point {
            OverlayAnchor::VirtualPoint {
                x: x as f32,
                y: y as f32,
                width: 0.0,
                height: 0.0,
                context_element_id: (!anchor_id.is_empty()).then_some(anchor_id.as_str()),
            }
        } else {
            if anchor_id.is_empty() {
                return;
            }
            OverlayAnchor::Element(anchor_id.as_str())
        };

        let sticky = matches!(
            self.sticky
                .try_peek()
                .map(|g| *g)
                .unwrap_or(StickyStrategy::Partial),
            StickyStrategy::Always
        );

        let computed = compute_virtual_overlay_position(&VirtualOverlayPositionConfig {
            anchor,
            content_id: content_id.as_str(),
            arrow_id: arrow_id.as_deref(),
            boundary_id: None,
            side: self
                .side
                .try_peek()
                .map(|g| *g)
                .unwrap_or(PlacementSide::Bottom),
            align: self
                .align
                .try_peek()
                .map(|g| *g)
                .unwrap_or(PlacementAlign::Start),
            side_offset: self.side_offset.try_peek().map(|g| *g).unwrap_or(4.0),
            align_offset: self.align_offset.try_peek().map(|g| *g).unwrap_or(0.0),
            avoid_collisions: self.avoid_collisions.try_peek().map(|g| *g).unwrap_or(true),
            collision_padding: self.collision_padding.try_peek().map(|g| *g).unwrap_or(0.0),
            arrow_padding: self.arrow_padding.try_peek().map(|g| *g).unwrap_or(0.0),
            hide_when_detached: self
                .hide_when_detached
                .try_peek()
                .map(|g| *g)
                .unwrap_or(false),
            sticky,
            namespace: Some(root.variant.geometry_namespace()),
        });

        let _ = wrapper_id;
        let mut placement_sig = self.placement;
        let mut readiness_sig = self.readiness;
        if let Some(p) = computed {
            let geom = p.geometry();
            let has_dims = geom.content_width() > 0.0 && geom.content_height() > 0.0;
            let next_readiness = if has_dims {
                FloatingReadiness::Ready
            } else {
                FloatingReadiness::Measuring
            };
            if readiness_sig.try_peek().is_ok_and(|r| *r != next_readiness) {
                let _ = readiness_sig.try_write().map(|mut w| *w = next_readiness);
            }
            if placement_sig
                .try_peek()
                .is_ok_and(|cur| cur.as_ref() != Some(&p))
            {
                let _ = placement_sig.try_write().map(|mut w| *w = Some(p));
            }
        }
    }

    /// Start auto-update position monitor on the mounted `content_id` and compute initial placement.
    pub fn ensure_position_tracking(&self, root: &MenuRootContext) {
        if !self.should_render_content() {
            let mut mon = self.position_monitor;
            if mon.try_peek().is_ok_and(|g| g.is_some()) {
                let _ = mon.try_write().map(|mut w| *w = None);
            }
            let mut pl = self.placement;
            if pl.try_peek().is_ok_and(|g| g.is_some()) {
                let _ = pl.try_write().map(|mut w| *w = None);
            }
            let mut rd = self.readiness;
            if rd
                .try_peek()
                .is_ok_and(|g| *g != FloatingReadiness::Measuring)
            {
                let _ = rd.try_write().map(|mut w| *w = FloatingReadiness::Measuring);
            }
            self.clear_content_z_index();
            return;
        }

        self.sync_content_z_index();
        self.update_placement(root);

        let anchor_id = self
            .anchor_id
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_default();
        let content_id = self.current_content_id();
        let auto_surface = self.clone();
        let auto_root = root.clone();
        let anchors: Vec<&str> = if anchor_id.is_empty() {
            vec![]
        } else {
            vec![anchor_id.as_str()]
        };
        let guard = start_floating_auto_update_monitor(&anchors, &content_id, move |_| {
            auto_surface.update_placement(&auto_root);
        });
        let mut mon = self.position_monitor;
        let _ = mon.try_write().map(|mut w| *w = Some(guard));

        let frame_surface = self.clone();
        let frame_root = root.clone();
        spawn_root_task(async move {
            let _ = oxidase::frame::next_frame().await;
            frame_surface.update_placement(&frame_root);
        });
    }

    pub fn stop_monitors(&self) {
        let mut presence_monitor = self.presence_monitor;
        if let Ok(mut w) = presence_monitor.try_write() {
            *w = None;
        }
        let mut position_monitor = self.position_monitor;
        if let Ok(mut w) = position_monitor.try_write() {
            *w = None;
        }
        let mut dismiss_monitor = self.dismiss_monitor;
        if let Ok(mut w) = dismiss_monitor.try_write() {
            *w = None;
        }
        let mut blur_monitor = self.blur_monitor;
        if let Ok(mut w) = blur_monitor.try_write() {
            *w = None;
        }
        let mut modal_shield = self.modal_shield;
        if let Ok(mut w) = modal_shield.try_write() {
            *w = None;
        }
        if self.scroll_lock_held.try_read().map(|g| *g).unwrap_or(false) {
            release_scroll_lock(&self.surface_key, None);
            let mut scroll_lock_held = self.scroll_lock_held;
            if let Ok(mut w) = scroll_lock_held.try_write() {
                *w = false;
            }
        }
        self.clear_corridor();
    }
}

/// Provide lifecycle hooks (`presence`, positioning, modal shield, scroll lock, dismiss, window blur)
/// for a root or submenu `MenuSurfaceRuntime`.
pub fn use_menu_surface_lifecycle(surface: MenuSurfaceRuntime, root: MenuRootContext) {
    let cleanup_surface = surface.clone();
    dioxus::core::use_drop(move || {
        cleanup_surface.stop_monitors();
        if let Some(task) = cleanup_surface.typeahead_task.borrow_mut().take() {
            task.cancel();
        }
    });

    // Presence synchronization (`PresenceController` + `start_presence_monitor` + `MB-5` fast skip)
    let presence_surface = surface.clone();
    let presence_root = root.clone();
    use_effect(move || {
        let is_open = (presence_surface.open)();
        let skip_exit = presence_root
            .skip_exit_animation
            .try_read()
            .map(|g| *g)
            .unwrap_or(false);
        let update = {
            let mut lane = presence_surface.presence_lane;
            lane.write().sync(is_open)
        };

        if let Some(cycle) = update.started_close_cycle() {
            if skip_exit {
                {
                    let mut lane = presence_surface.presence_lane;
                    lane.write().complete_close_cycle(cycle);
                }
                let mut skip_sig = presence_root.skip_exit_animation;
                let _ = skip_sig.try_write().map(|mut w| *w = false);
                if let Some(cb) = presence_root.on_open_change_complete.as_ref()
                    && !presence_surface.is_submenu
                {
                    cb.call(false);
                }
            } else {
                let mut active_cycle = presence_surface.active_close_cycle;
                active_cycle.set(Some(cycle));
                let content_id = presence_surface.current_content_id();
                let mut mon_surface = presence_surface.clone();
                let mon_root = presence_root.clone();
                let guard = start_presence_monitor(&content_id, cycle, move |evt| {
                    if matches!(
                        evt,
                        PresenceMonitorEvent::AnimationEnd { .. }
                            | PresenceMonitorEvent::AnimationCancel { .. }
                            | PresenceMonitorEvent::TransitionEnd { .. }
                            | PresenceMonitorEvent::TransitionCancel { .. }
                            | PresenceMonitorEvent::Fallback { .. }
                            | PresenceMonitorEvent::Stopped { .. }
                    ) {
                        let done = {
                            let mut lane = mon_surface.presence_lane;
                            lane.write().complete_close_cycle(cycle)
                        };
                        if done {
                            if let Ok(mut w) = mon_surface.presence_monitor.try_write() {
                                *w = None;
                            }
                            if let Some(cb) = mon_root.on_open_change_complete.as_ref()
                                && !mon_surface.is_submenu
                            {
                                cb.call(false);
                            }
                        }
                    }
                });
                let mut mon = presence_surface.presence_monitor;
                mon.set(Some(guard));
            }
        } else if update.invalidated_close_cycle().is_some()
            || matches!(update.state(), PresenceState::Mounted | PresenceState::Unmounted)
        {
            let mut mon = presence_surface.presence_monitor;
            if mon.peek().is_some() {
                mon.set(None);
            }
            if matches!(update.state(), PresenceState::Mounted)
                && !presence_surface.is_submenu
                && let Some(cb) = presence_root.on_open_change_complete.as_ref()
            {
                cb.call(true);
            }
        }
    });

    // Floating position cleanup / virtual-anchor re-tracking
    let pos_surface = surface.clone();
    let pos_root = root.clone();
    use_effect(move || {
        let should_render = pos_surface.should_render_content();
        let _vanchor = if !pos_surface.is_submenu && pos_root.variant == MenuVariant::ContextMenu {
            (pos_root.virtual_anchor)()
        } else {
            None
        };
        let _open = (pos_surface.open)();

        if !should_render {
            let mut mon = pos_surface.position_monitor;
            if mon.try_peek().is_ok_and(|g| g.is_some()) {
                let _ = mon.try_write().map(|mut w| *w = None);
            }
            let mut pl = pos_surface.placement;
            if pl.try_peek().is_ok_and(|g| g.is_some()) {
                let _ = pl.try_write().map(|mut w| *w = None);
            }
            let mut rd = pos_surface.readiness;
            if rd
                .try_peek()
                .is_ok_and(|g| *g != FloatingReadiness::Measuring)
            {
                let _ = rd.try_write().map(|mut w| *w = FloatingReadiness::Measuring);
            }
            pos_surface.clear_content_z_index();
            return;
        }

        pos_surface.ensure_position_tracking(&pos_root);
    });

    // Root-level modal shield, scroll lock, outside dismiss, window blur, and open/close autofocus
    if !surface.is_submenu {
        let modal_surface = surface.clone();
        let modal_root = root.clone();
        use_effect(move || {
            let is_open = (modal_surface.open)();
            let is_modal = (modal_root.modal)();
            let prevent_scroll = (modal_surface.prevent_scroll)();
            let subcontents = (modal_surface.child_subcontents)();
            let branches = (modal_surface.branches)();

            if !is_open {
                let next_gen = modal_surface.open_focus_gen.get().wrapping_add(1);
                modal_surface.open_focus_gen.set(next_gen);
                let mut shield = modal_surface.modal_shield;
                if shield.peek().is_some() {
                    shield.set(None);
                }
                if modal_surface
                    .scroll_lock_held
                    .try_read()
                    .map(|g| *g)
                    .unwrap_or(false)
                {
                    release_scroll_lock(&modal_surface.surface_key, None);
                    let mut sl = modal_surface.scroll_lock_held;
                    sl.set(false);
                }
                let mut dm = modal_surface.dismiss_monitor;
                if dm.peek().is_some() {
                    dm.set(None);
                }
                let mut bm = modal_surface.blur_monitor;
                if bm.peek().is_some() {
                    bm.set(None);
                }

                // Close autofocus back to root trigger unless suppressed
                let armed = modal_surface
                    .focus_on_open_armed
                    .try_read()
                    .map(|g| *g)
                    .unwrap_or(false);
                if armed {
                    let mut arm_sig = modal_surface.focus_on_open_armed;
                    let _ = arm_sig.try_write().map(|mut w| *w = false);
                    let mut prev_sig = modal_root.previously_focused_id;
                    let prev_focus_id = prev_sig
                        .try_write()
                        .ok()
                        .and_then(|mut w| w.take());

                    let close_evt = MenuPreventableEvent::new();
                    if let Some(cb) = modal_surface
                        .on_close_auto_focus
                        .try_read()
                        .ok()
                        .and_then(|g| g.clone())
                    {
                        cb.call(close_evt.clone());
                    }
                    let ignore = modal_root
                        .ignore_close_auto_focus
                        .try_read()
                        .map(|g| *g)
                        .unwrap_or(false);
                    if !ignore && !close_evt.is_default_prevented() {
                        if modal_root.variant == MenuVariant::ContextMenu {
                            let trigger_id = modal_root
                                .trigger_id
                                .try_read()
                                .map(|g| g.clone())
                                .unwrap_or_else(|_| {
                                    modal_root.relationships.trigger_id().to_owned()
                                });
                            if let Some(prev_id) = prev_focus_id
                                && (prev_id != trigger_id || is_element_tabbable_by_id(&trigger_id))
                            {
                                let focus_visible = modal_root
                                    .using_keyboard
                                    .try_read()
                                    .map(|g| *g)
                                    .unwrap_or(false);
                                let opts =
                                    FocusOptions::with_focus_visible(false, Some(focus_visible));
                                restore_focus_element_by_id_with_options(&prev_id, opts);
                            }
                        } else {
                            modal_root.focus_root_trigger();
                        }
                    }
                    let mut ig = modal_root.ignore_close_auto_focus;
                    let _ = ig.try_write().map(|mut w| *w = false);
                }
                return;
            }

            // Arm close-focus restoration for when this open cycle ends
            {
                let mut arm_sig = modal_surface.focus_on_open_armed;
                let was_armed = arm_sig.peek().to_owned();
                if !was_armed {
                    arm_sig.set(true);
                    let prev_focus = oxidase::dom::Document::current()
                        .and_then(|d| d.active_element())
                        .filter(|e| !matches!(e.tag_name().as_deref(), Some("body" | "html")))
                        .and_then(|e| e.id())
                        .filter(|id| !id.is_empty());
                    let mut prev_sig = modal_root.previously_focused_id;
                    let _ = prev_sig.try_write().map(|mut w| *w = prev_focus);
                    let mut ig = modal_root.ignore_close_auto_focus;
                    let _ = ig.try_write().map(|mut w| *w = false);
                    modal_surface.apply_open_focus(&modal_root);
                }
            }

            // Build boundary IDs & modal keep IDs
            let content_id = modal_surface.current_content_id();
            let wrapper_id = modal_surface.current_wrapper_id();
            let portal_id_opt = modal_surface.portal_id.try_read().ok().and_then(|g| g.clone());
            let trigger_id = modal_root
                .trigger_id
                .try_read()
                .map(|g| g.clone())
                .unwrap_or_default();

            let mut keep_ids = vec![content_id.clone(), wrapper_id.clone()];
            if let Some(ref pid) = portal_id_opt {
                keep_ids.push(pid.clone());
            }
            for cid in &subcontents {
                if !keep_ids.contains(cid) {
                    keep_ids.push(cid.clone());
                }
            }
            for bid in &branches {
                if !keep_ids.contains(bid) {
                    keep_ids.push(bid.clone());
                }
            }
            let keep_refs: Vec<&str> = keep_ids.iter().map(String::as_str).collect();

            if is_modal {
                if modal_root.variant == MenuVariant::ContextMenu {
                    // Use aria-hidden shielding without `inert` on ContextMenu so right-clicking
                    // a sibling ContextMenuTrigger (`CM-6`) cleanly switches menus.
                    let guard = oxidase::dom::hide_others(&keep_refs);
                    let mut shield = modal_surface.modal_shield;
                    shield.set(Some(guard));
                    acquire_scroll_lock_with_options(&modal_surface.surface_key, false);
                } else {
                    let guard = suppress_external_dom(&keep_refs);
                    let mut shield = modal_surface.modal_shield;
                    shield.set(Some(guard));
                    acquire_scroll_lock_with_options(&modal_surface.surface_key, true);
                }
                let mut sl = modal_surface.scroll_lock_held;
                sl.set(true);
            } else {
                let mut shield = modal_surface.modal_shield;
                if shield.peek().is_some() {
                    shield.set(None);
                }
                if prevent_scroll {
                    acquire_scroll_lock_with_options(&modal_surface.surface_key, false);
                    let mut sl = modal_surface.scroll_lock_held;
                    sl.set(true);
                }
            }

            let mut boundaries = vec![content_id, wrapper_id];
            if let Some(pid) = portal_id_opt {
                boundaries.push(pid);
            }
            for cid in &subcontents {
                if !boundaries.contains(cid) {
                    boundaries.push(cid.clone());
                }
            }
            for bid in &branches {
                if !boundaries.contains(bid) {
                    boundaries.push(bid.clone());
                }
            }
            match modal_root.variant {
                MenuVariant::DropdownMenu => {
                    if !trigger_id.is_empty() && !boundaries.contains(&trigger_id) {
                        boundaries.push(trigger_id);
                    }
                }
                MenuVariant::Menubar => {
                    if let Some(bridge) = modal_root.menubar_bridge.as_ref() {
                        for tid in (bridge.all_trigger_ids)() {
                            if !boundaries.contains(&tid) {
                                boundaries.push(tid);
                            }
                        }
                    }
                }
                MenuVariant::ContextMenu | MenuVariant::Menu => {}
            }

            let boundary_refs: Vec<&str> = boundaries.iter().map(String::as_str).collect();
            let dm_surface = modal_surface.clone();
            let dm_root = modal_root.clone();
            let dm_guard = start_document_dismiss_monitor_with_boundaries(&boundary_refs, move |evt| {
                match evt {
                    DocumentDismissEvent::Escape => {
                        let root_cid = dm_surface.current_content_id();
                        if crate::foundation::overlay::is_overlay_escape_consumed()
                            || !crate::foundation::overlay::is_topmost_overlay_layer(&root_cid)
                        {
                            return;
                        }
                        let esc_evt = MenuPreventableEvent::new();
                        if let Some(cb) = dm_surface
                            .on_escape_keydown
                            .try_read()
                            .ok()
                            .and_then(|g| g.clone())
                        {
                            cb.call(esc_evt.clone());
                        }
                        crate::foundation::overlay::mark_overlay_escape_consumed();
                        if !esc_evt.is_default_prevented() {
                            dm_root.close_menu_tree(true);
                        }
                    }
                    DocumentDismissEvent::PointerDown { path_ids } => {
                        let inside_live_surface = path_ids.iter().any(|pid| {
                            dm_surface.content_id.try_peek().is_ok_and(|c| c.as_str() == pid)
                                || dm_surface.wrapper_id.try_peek().is_ok_and(|w| w.as_str() == pid)
                                || dm_surface.portal_id.try_peek().is_ok_and(|p| p.as_deref() == Some(pid.as_str()))
                                || dm_surface.child_subcontents.try_peek().is_ok_and(|subs| subs.contains(pid))
                                || dm_surface.branches.try_peek().is_ok_and(|brs| brs.contains(pid))
                        });
                        if inside_live_surface {
                            return;
                        }
                        let target_id = path_ids.first().cloned();
                        let out_evt = MenuOutsideEvent::new(target_id);
                        if let Some(cb) = dm_surface
                            .on_pointer_down_outside
                            .try_read()
                            .ok()
                            .and_then(|g| g.clone())
                        {
                            cb.call(out_evt.clone());
                        }
                        if let Some(cb) = dm_surface
                            .on_interact_outside
                            .try_read()
                            .ok()
                            .and_then(|g| g.clone())
                        {
                            cb.call(out_evt.clone());
                        }
                        if !out_evt.is_default_prevented() {
                            if let Some(bridge) = dm_root.menubar_bridge.as_ref() {
                                (bridge.close_menubar)();
                            } else {
                                dm_root.close_menu_tree(false);
                            }
                        }
                    }
                    DocumentDismissEvent::FocusIn { path_ids } => {
                        let inside_live_surface = path_ids.iter().any(|pid| {
                            dm_surface.content_id.try_peek().is_ok_and(|c| c.as_str() == pid)
                                || dm_surface.wrapper_id.try_peek().is_ok_and(|w| w.as_str() == pid)
                                || dm_surface.portal_id.try_peek().is_ok_and(|p| p.as_deref() == Some(pid.as_str()))
                                || dm_surface.child_subcontents.try_peek().is_ok_and(|subs| subs.contains(pid))
                                || dm_surface.branches.try_peek().is_ok_and(|brs| brs.contains(pid))
                        });
                        if inside_live_surface {
                            return;
                        }
                        if let Some(bridge) = dm_root.menubar_bridge.as_ref() {
                            let tids = (bridge.all_trigger_ids)();
                            if path_ids.iter().any(|pid| tids.contains(pid)) {
                                return;
                            }
                        }
                        let target_id = path_ids.first().cloned();
                        let out_evt = MenuOutsideEvent::new(target_id);
                        if let Some(cb) = dm_surface
                            .on_focus_outside
                            .try_read()
                            .ok()
                            .and_then(|g| g.clone())
                        {
                            cb.call(out_evt.clone());
                        }
                        if let Some(cb) = dm_surface
                            .on_interact_outside
                            .try_read()
                            .ok()
                            .and_then(|g| g.clone())
                        {
                            cb.call(out_evt.clone());
                        }
                        if !out_evt.is_default_prevented() {
                            if let Some(bridge) = dm_root.menubar_bridge.as_ref() {
                                (bridge.close_menubar)();
                            } else {
                                dm_root.close_menu_tree(false);
                            }
                        }
                    }
                    DocumentDismissEvent::Click { .. } => {}
                }
            });
            let mut dm = modal_surface.dismiss_monitor;
            dm.set(Some(dm_guard));

            // Window blur dismiss (`M-15`)
            let blur_root = modal_root.clone();
            let blur_guard = start_window_blur_monitor(move || {
                if let Some(bridge) = blur_root.menubar_bridge.as_ref() {
                    (bridge.close_menubar)();
                } else {
                    blur_root.close_menu_tree(false);
                }
            });
            let mut bm = modal_surface.blur_monitor;
            bm.set(Some(blur_guard));
        });
    } else {
        // Submenu open autofocus once mounted
        let sub_surface = surface.clone();
        let sub_root = root.clone();
        use_effect(move || {
            let is_open = (sub_surface.open)();
            if !is_open {
                let next_gen = sub_surface.open_focus_gen.get().wrapping_add(1);
                sub_surface.open_focus_gen.set(next_gen);
                let mut arm = sub_surface.focus_on_open_armed;
                let _ = arm.try_write().map(|mut w| *w = false);
                return;
            }
            let mut arm = sub_surface.focus_on_open_armed;
            if !arm.peek().to_owned() {
                arm.set(true);
                sub_surface.apply_open_focus(&sub_root);
            }
        });
    }
}

/// Shared context for descendant items/groups/indicators inside either `Content` or `SubContent`.
#[derive(Clone)]
pub struct MenuSurfaceContext {
    pub surface: MenuSurfaceRuntime,
    pub parent_surface: Option<MenuSurfaceRuntime>,
    pub close_this_submenu: Option<Rc<dyn Fn(bool)>>,
}

/// Group context providing automatic `aria-labelledby` linkage between `Group` and `GroupHeading` (`3B D-6`).
#[derive(Clone)]
pub struct MenuGroupContext {
    pub group_id: String,
    pub heading_id: Signal<Option<String>>,
    pub default_heading_id: String,
}

/// Checkbox group context supporting `MenuCheckboxGroup` multi-select (`3B D-4`).
#[derive(Clone)]
pub struct MenuCheckboxGroupContext {
    pub value: Signal<Vec<String>>,
    pub disabled: Signal<bool>,
    pub on_toggle: Rc<dyn Fn(&str)>,
}

/// Radio group context supporting `MenuRadioGroup` single-select (`M-9`).
#[derive(Clone)]
pub struct MenuRadioGroupContext {
    pub value: Signal<String>,
    pub on_select_value: Rc<dyn Fn(&str)>,
}

/// Indicator state provided by `CheckboxItem` and `RadioItem` to `ItemIndicator` (`M-10`).
#[derive(Clone)]
pub struct MenuIndicatorContext {
    pub checked: CheckedState,
    pub disabled: bool,
}

/// Submenu state provided by `MenuSub` to `SubTrigger`, `Portal`, `SubContent`, and `SubContentStatic`.
#[derive(Clone)]
pub struct MenuSubContext {
    pub sub_id: String,
    pub trigger_id: String,
    pub content_id: String,
    pub wrapper_id: String,
    pub open: Signal<bool>,
    pub is_controlled: bool,
    pub on_open_change: Option<EventHandler<bool>>,
    pub open_delay_ms: u64,
    pub parent_surface: MenuSurfaceRuntime,
    pub surface: MenuSurfaceRuntime,
    pub hover_task: Rc<RefCell<Option<dioxus::core::Task>>>,
    pub hover_gen: Rc<Cell<u64>>,
    pub opened_by_pointer_move: Signal<bool>,
}

impl MenuSubContext {
    pub fn is_open(&self) -> bool {
        self.open.try_read().map(|g| *g).unwrap_or(false)
    }

    pub fn cancel_hover_timer(&self) {
        self.hover_gen.set(self.hover_gen.get().wrapping_add(1));
        if let Some(t) = self.hover_task.borrow_mut().take() {
            t.cancel();
        }
    }

    pub fn request_open(&self, next: bool, focus_target: MenuOpenFocusTarget) {
        self.cancel_hover_timer();
        {
            let mut tgt = self.surface.open_focus_target;
            let _ = tgt.try_write().map(|mut w| *w = focus_target);
        }
        if next {
            self.parent_surface
                .close_other_child_submenus(Some(&self.sub_id));
            let mut active = self.parent_surface.active_submenu_id;
            let _ = active.try_write().map(|mut w| *w = Some(self.sub_id.clone()));
            let close_ctx = self.clone();
            let mut close_sig = self.parent_surface.active_submenu_close;
            let _ = close_sig.try_write().map(|mut w| {
                *w = Some((
                    self.sub_id.clone(),
                    Rc::new(move || {
                        close_ctx.request_open(false, MenuOpenFocusTarget::None);
                    }),
                ))
            });
            self.parent_surface
                .register_child_subcontent(
                    self.surface
                        .content_id
                        .try_peek()
                        .ok()
                        .map(|g| g.clone())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| self.content_id.clone()),
                );
        } else {
            self.surface.close_other_child_submenus(None);
            let mut active = self.parent_surface.active_submenu_id;
            if active.try_peek().is_ok_and(|g| g.as_deref() == Some(&self.sub_id)) {
                let _ = active.try_write().map(|mut w| *w = None);
            }
            let mut close_sig = self.parent_surface.active_submenu_close;
            if close_sig
                .try_peek()
                .is_ok_and(|g| g.as_ref().is_some_and(|(id, _)| id == &self.sub_id))
            {
                let _ = close_sig.try_write().map(|mut w| *w = None);
            }
            self.parent_surface
                .unregister_child_subcontent(&self.content_id);
            if let Ok(live_cid) = self.surface.content_id.try_peek()
                && live_cid.as_str() != self.content_id.as_str()
            {
                self.parent_surface.unregister_child_subcontent(&live_cid);
            }
        }
        if self.is_open() == next {
            return;
        }
        if !self.is_controlled {
            let mut o = self.open;
            let _ = o.try_write().map(|mut w| *w = next);
        }
        if let Some(cb) = self.on_open_change.as_ref() {
            cb.call(next);
        }
    }

    pub fn close_and_focus_trigger(&self, focus_trigger: bool, using_keyboard: bool) {
        self.request_open(false, MenuOpenFocusTarget::None);
        if focus_trigger {
            let trigger_id = self
                .surface
                .anchor_id
                .try_peek()
                .ok()
                .map(|g| g.clone())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| self.trigger_id.clone());
            self.parent_surface.highlight_item(
                Some(trigger_id),
                true,
                using_keyboard,
            );
        }
    }
}

pub fn focus_element_reliable(target_id: &str, options: FocusOptions) {
    focus_element_sync_or_defer(target_id, options);
    if !is_element_focused_by_id(target_id) {
        focus_element_by_id_with_options(target_id, options);
    }
}

async fn try_scroll_menu_item_within_container(content_id: &str, item_id: &str) -> bool {
    if content_id.is_empty() {
        scroll_element_into_view_nearest(item_id);
        return true;
    }
    let Some(container_rect) = get_element_client_rect(content_id) else {
        return false;
    };
    let Some(item_rect) = get_element_client_rect(item_id) else {
        return false;
    };
    if item_rect.height <= 0.0 || container_rect.height <= 0.0 {
        return false;
    }

    let is_scrollable = get_element_scroll_metrics(content_id)
        .is_some_and(|m| m.client_height > 0.0 && m.scroll_height > m.client_height + 1.0);
    if !is_scrollable {
        scroll_element_into_view_nearest(item_id);
        return true;
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

    let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::dom::document);
    let Some(container_el) = doc.element_by_id(content_id) else {
        return false;
    };
    container_el.scroll_by(0.0, delta_y).await.is_ok()
}

fn scroll_menu_item_into_view(content_id: &str, item_id: &str) {
    let content_id = content_id.to_owned();
    let item_id = item_id.to_owned();
    spawn_root_task(async move {
        for attempt in 0..3 {
            if try_scroll_menu_item_within_container(&content_id, &item_id).await {
                break;
            }
            if attempt < 2 {
                let _ = oxidase::frame::next_frame().await;
            }
        }
    });
}

pub fn monotonic_ms() -> u64 {
    oxidase::time::monotonic_now_ms().max(0.0) as u64
}

fn collect_open_menu_ids(root: &MenuRootContext) -> Vec<String> {
    let mut ids = vec![root.root_surface.current_content_id()];
    if let Ok(subcontents) = root.root_surface.child_subcontents.try_read() {
        for cid in subcontents.iter() {
            if !ids.contains(cid) {
                ids.push(cid.clone());
            }
        }
    }
    ids
}

fn focus_adjacent_tabbable(trigger_id: &str, open_menu_ids: &[String], forward: bool) {
    let Some(doc) = oxidase::dom::Document::current() else {
        return;
    };
    let Some(body) = doc.query_selector("body") else {
        return;
    };

    let open_containers: Vec<oxidase::dom::Element> = open_menu_ids
        .iter()
        .filter_map(|id| doc.element_by_id(id))
        .collect();

    let all_candidates = body.query_selector_all(TABBABLE_SELECTOR);
    let filtered: Vec<oxidase::dom::Element> = all_candidates
        .into_iter()
        .filter(|el| {
            let inside_menu = open_containers
                .iter()
                .any(|menu| menu.id() == el.id() || menu.contains(el));
            !inside_menu && !el.has_attribute("inert") && !el.has_attribute("aria-hidden")
        })
        .collect();

    if let Some(pos) = filtered
        .iter()
        .position(|el| el.id().as_deref() == Some(trigger_id))
    {
        let next = if forward {
            filtered.get(pos + 1)
        } else {
            pos.checked_sub(1).and_then(|idx| filtered.get(idx))
        };
        if let Some(target) = next {
            let _ = target.focus_sync_with_options(FocusOptions::with_focus_visible(false, Some(true)));
        }
        return;
    }

    // Fallback when trigger itself is non-tabbable (e.g. tabindex="-1")
    if let Some(trigger_el) = doc.element_by_id(trigger_id) {
        let fallback_selector = format!("[id=\"{trigger_id}\"], {TABBABLE_SELECTOR}");
        let all_with_trigger: Vec<oxidase::dom::Element> = body
            .query_selector_all(&fallback_selector)
            .into_iter()
            .filter(|el| {
                let inside_menu = open_containers
                    .iter()
                    .any(|menu| menu.id() == el.id() || menu.contains(el));
                el.id().as_deref() == Some(trigger_id)
                    || (!inside_menu && !el.has_attribute("inert") && !el.has_attribute("aria-hidden"))
            })
            .collect();
        if let Some(anchor_pos) = all_with_trigger
            .iter()
            .position(|el| el.id().as_deref() == trigger_el.id().as_deref())
        {
            let target = if forward {
                all_with_trigger.get(anchor_pos + 1)
            } else {
                anchor_pos
                    .checked_sub(1)
                    .and_then(|idx| all_with_trigger.get(idx))
            };
            if let Some(el) = target {
                let _ = el.focus_sync_with_options(FocusOptions::with_focus_visible(false, Some(true)));
            }
        }
    }
}


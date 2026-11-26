use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;
use monoxus_core::foundation::typeahead::{TypeaheadBuffer, TypeaheadCandidate, TypeaheadConfig};

pub(crate) use crate::foundation::browser::focus_element_reliable;
use crate::foundation::{
    browser::{
        FocusOptions, get_element_client_rect, get_element_scroll_metrics,
        is_element_tabbable_by_id, query_descendant_order,
        restore_focus_element_by_id_with_options, scroll_element_into_view_nearest,
        spawn_root_task, start_window_blur_monitor, use_is_using_keyboard,
    },
    compose::MountedHandle,
    keyboard::is_ime_composing_key,
    overlay::{
        CancellableDelay, DismissCallbacks, DismissConfig, DismissEvent, DismissLayerHandle,
        FloatingHandle, FloatingMonitorSpec, FloatingPlacement, FloatingReadiness,
        FloatingScrollPolicy, FocusScopeConfig, FocusScopeHandle, GraceCorridorHandle,
        GraceCorridorOutcome, GraceTransit, ModalLayerConfig, ModalLayerHandle, ModalShieldPolicy,
        PlacementAlign, PlacementSide, Point2D, PresenceHandle, PresenceState, Rect2D,
        SUBMENU_INTENT_MIN_PROGRESS_PX, StickyStrategy, is_overlay_escape_consumed,
        is_topmost_overlay_layer, mark_overlay_escape_consumed, use_cancellable_delay,
        use_dismissable_layer, use_floating_layer, use_focus_scope, use_grace_corridor,
        use_modal_layer, use_presence, use_watcher_slot,
    },
    shared::Direction,
    state::sync_signal,
};

use super::{
    relationships::MenuRelationships,
    state::{first_enabled_item_id, last_enabled_item_id, menu_css_vars, resolve_menu_navigation},
    types::{
        CheckedState, MENU_DEFAULT_SUBMENU_OPEN_DELAY_MS, MENU_SUBMENU_CORRIDOR_BUFFER_PX,
        MENU_SUBMENU_CORRIDOR_FALLBACK_MS, MENU_SUBMENU_GRACE_PADDING_PX,
        MENU_SUBMENU_RETURN_DISENGAGE_MS, MENU_SUBMENU_TRIGGER_INSET_PX, MENU_TYPEAHEAD_TIMEOUT_MS,
        MenuItemEntry, MenuKey, MenuNavigationStep, MenuOpenFocusTarget, MenuOutsideEvent,
        MenuPreventableEvent, MenuVariant, PressedItemSnapshot, translate_menu_key,
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
    pub side: PlacementSide,
    pub close_sub: Rc<dyn Fn()>,
}

/// Shared root context provided by `DropdownMenuRoot`, `ContextMenuRoot`, `MenubarMenu`, or internal `MenuRoot`.
#[derive(Clone)]
pub struct MenuRootContext {
    pub variant: MenuVariant,
    pub relationships: MenuRelationships,
    pub open: Signal<bool>,
    pub is_controlled: bool,
    pub on_open_change: CopyValue<Option<EventHandler<bool>>>,
    pub on_open_change_complete: CopyValue<Option<EventHandler<bool>>>,
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
        let default_side = match variant {
            MenuVariant::ContextMenu => PlacementSide::Right,
            _ => PlacementSide::Bottom,
        };
        let default_side_offset = match variant {
            MenuVariant::ContextMenu => super::types::MENU_CONTEXT_DEFAULT_SIDE_OFFSET,
            _ => super::types::MENU_DEFAULT_SIDE_OFFSET,
        };
        let root_surface = MenuSurfaceRuntime::new(
            relationships.root_id().to_owned(),
            false,
            open,
            relationships.content_id().to_owned(),
            relationships.wrapper_id().to_owned(),
            relationships.trigger_id().to_owned(),
            default_side,
            PlacementAlign::Start,
            default_side_offset,
            0.0,
            loop_focus,
        );

        let mut on_open_change_slot = use_hook(|| CopyValue::new(on_open_change));
        on_open_change_slot.set(on_open_change);
        let mut on_open_change_complete_slot =
            use_hook(|| CopyValue::new(on_open_change_complete));
        on_open_change_complete_slot.set(on_open_change_complete);

        Self {
            variant,
            relationships: relationships.clone(),
            open,
            is_controlled,
            on_open_change: on_open_change_slot,
            on_open_change_complete: on_open_change_complete_slot,
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
        if let Some(cb) = self.on_open_change.try_read().ok().and_then(|s| *s) {
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
    pub is_static_surface: Signal<bool>,
    pub items: Signal<Vec<MenuItemEntry>>,
    pub highlighted_id: Signal<Option<String>>,
    pub pressed_item: Signal<Option<PressedItemSnapshot>>,
    pub pointer_up_consumed_click: Signal<Option<String>>,
    pub typeahead: Signal<TypeaheadBuffer>,
    pub typeahead_delay: CancellableDelay,
    pub active_submenu_id: Signal<Option<String>>,
    pub active_submenu_close: Signal<Option<(String, Rc<dyn Fn()>)>>,
    pub active_corridor_sub: Signal<Option<ActiveCorridorSubmenu>>,
    pub child_subcontents: Signal<Vec<String>>,
    pub branches: Signal<Vec<String>>,
    pub registered_sub_triggers: Signal<Vec<(String, Rc<dyn Fn()>, Rc<dyn Fn()>)>>,
    pub on_close_submenu: Signal<Option<Rc<dyn Fn()>>>,
    pub grace_corridor: GraceCorridorHandle,
    pub presence: CopyValue<Option<PresenceHandle>>,
    pub floating: CopyValue<Option<FloatingHandle>>,
    pub dismiss: CopyValue<Option<DismissLayerHandle>>,
    pub modal_layer: CopyValue<Option<ModalLayerHandle>>,
    pub focus_scope: CopyValue<Option<FocusScopeHandle>>,
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
    pub on_escape_key_down: CopyValue<Option<EventHandler<MenuPreventableEvent>>>,
    pub on_pointer_down_outside: CopyValue<Option<EventHandler<MenuOutsideEvent>>>,
    pub on_focus_outside: CopyValue<Option<EventHandler<MenuOutsideEvent>>>,
    pub on_interact_outside: CopyValue<Option<EventHandler<MenuOutsideEvent>>>,
    pub on_open_auto_focus: CopyValue<Option<EventHandler<MenuPreventableEvent>>>,
    pub on_close_auto_focus: CopyValue<Option<EventHandler<MenuPreventableEvent>>>,
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
        let grace_corridor = use_grace_corridor();
        let typeahead_delay = use_cancellable_delay();

        Self {
            surface_key,
            is_submenu,
            open,
            content_id: use_signal(|| content_id),
            wrapper_id: use_signal(|| wrapper_id),
            portal_id: use_signal(|| None),
            arrow_id: use_signal(|| None),
            anchor_id: use_signal(|| anchor_id),
            is_static_surface: use_signal(|| false),
            items: use_signal(Vec::new),
            highlighted_id: use_signal(|| None),
            pressed_item: use_signal(|| None),
            pointer_up_consumed_click: use_signal(|| None),
            typeahead: use_signal(|| TypeaheadBuffer::new(TypeaheadConfig::MENU)),
            typeahead_delay,
            active_submenu_id: use_signal(|| None),
            active_submenu_close: use_signal(|| None),
            active_corridor_sub: use_signal(|| None),
            child_subcontents: use_signal(Vec::new),
            branches: use_signal(Vec::new),
            registered_sub_triggers: use_signal(Vec::new),
            on_close_submenu: use_signal(|| None),
            grace_corridor,
            presence: use_hook(|| CopyValue::new(None)),
            floating: use_hook(|| CopyValue::new(None)),
            dismiss: use_hook(|| CopyValue::new(None)),
            modal_layer: use_hook(|| CopyValue::new(None)),
            focus_scope: use_hook(|| CopyValue::new(None)),
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
            on_escape_key_down: use_hook(|| CopyValue::new(None)),
            on_pointer_down_outside: use_hook(|| CopyValue::new(None)),
            on_focus_outside: use_hook(|| CopyValue::new(None)),
            on_interact_outside: use_hook(|| CopyValue::new(None)),
            on_open_auto_focus: use_hook(|| CopyValue::new(None)),
            on_close_auto_focus: use_hook(|| CopyValue::new(None)),
        }
    }

    pub fn should_render_content(&self) -> bool {
        let force = self.force_mount.try_read().map(|g| *g).unwrap_or(false);
        let is_open = self.open.try_read().map(|g| *g).unwrap_or(false);
        let present = self
            .presence
            .try_read()
            .ok()
            .and_then(|h| *h)
            .is_some_and(|h| h.should_render());
        force || is_open || present
    }

    pub fn presence_state(&self) -> PresenceState {
        self.presence
            .try_read()
            .ok()
            .and_then(|h| *h)
            .map(|h| h.state())
            .unwrap_or_else(|| {
                if self.open.try_read().map(|g| *g).unwrap_or(false) {
                    PresenceState::Mounted
                } else {
                    PresenceState::Unmounted
                }
            })
    }

    pub fn placement(&self) -> Option<FloatingPlacement> {
        self.floating
            .try_read()
            .ok()
            .and_then(|h| *h)
            .and_then(|h| h.placement())
    }

    pub fn resolved_side(&self) -> PlacementSide {
        self.placement()
            .map(|p| p.side())
            .unwrap_or_else(|| self.side.try_peek().map(|g| *g).unwrap_or(PlacementSide::Right))
    }

    pub fn readiness(&self) -> FloatingReadiness {
        self.floating
            .try_read()
            .ok()
            .and_then(|h| *h)
            .map(|h| h.readiness())
            .unwrap_or_default()
    }

    pub fn content_z_index(&self) -> Option<String> {
        self.floating
            .try_read()
            .ok()
            .and_then(|h| *h)
            .and_then(|h| h.content_z_index())
    }

    pub fn register_arrow_id(&self, arrow_id: &str) {
        let mut aid = self.arrow_id;
        sync_signal(&mut aid, Some(arrow_id.to_owned()));
        if let Some(f) = self.floating.try_read().ok().and_then(|h| *h) {
            f.register_arrow_id(arrow_id);
        }
    }

    pub fn unregister_arrow_id(&self, arrow_id: &str) {
        let mut aid = self.arrow_id;
        if aid.try_peek().is_ok_and(|g| g.as_deref() == Some(arrow_id))
            && let Ok(mut w) = aid.try_write()
        {
            *w = None;
        }
        if let Some(f) = self.floating.try_read().ok().and_then(|h| *h) {
            f.unregister_arrow_id(arrow_id);
        }
    }

    pub fn capture_content_mount(&self, handle: MountedHandle) {
        if let Some(f) = self.floating.try_read().ok().and_then(|h| *h) {
            f.capture_content(handle);
        }
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

    pub fn record_pointer(&self, x: f64, y: f64) {
        self.grace_corridor.record_pointer(x, y);
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
        if let Ok(mut list) = sig.try_write()
            && !list.contains(&id)
        {
            list.push(id);
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
        if !already
            && let Ok(mut list) = sig.try_write()
            && !list.contains(&id)
        {
            list.push(id.clone());
        }
        if let Some(d) = self.dismiss.try_read().ok().and_then(|h| *h) {
            d.register_branch(id);
        }
    }

    pub fn unregister_branch(&self, id: &str) {
        let mut sig = self.branches;
        let present = sig
            .try_peek()
            .is_ok_and(|list| list.iter().any(|entry| entry == id));
        if present
            && let Ok(mut list) = sig.try_write()
        {
            list.retain(|entry| entry != id);
        }
        if let Some(d) = self.dismiss.try_read().ok().and_then(|h| *h) {
            d.unregister_branch(id);
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
        if !self.grace_corridor.is_active() {
            return false;
        }
        if let (Some(target), Some(sub)) = (
            target_item_id,
            self.active_corridor_sub.try_read().ok().and_then(|g| g.clone()),
        ) && target == sub.trigger_id
        {
            return false;
        }
        true
    }

    pub fn highlight_item(&self, id: Option<String>, focus_dom: bool, using_keyboard: bool) {
        let mut hl = self.highlighted_id;
        let changed = hl.try_peek().map(|g| *g != id).unwrap_or(true);
        if changed
            && let Ok(mut w) = hl.try_write()
        {
            *w = id.clone();
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

        let evt = MenuPreventableEvent::empty();
        if let Some(cb) = self.on_open_auto_focus.try_read().ok().and_then(|s| *s) {
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
                if gen_ref.get() != expected_gen || !surface.open.try_peek().is_ok_and(|o| *o) {
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
        let candidates: Vec<TypeaheadCandidate<'_>> = items
            .iter()
            .map(|item| TypeaheadCandidate::label_only(&item.id, &item.text_value, item.disabled))
            .collect();
        let current = self.current_highlighted_id();

        let matched = {
            let mut ta = self.typeahead;
            let Ok(mut buf) = ta.try_write() else {
                return false;
            };
            buf.push_char(ch, now_ms, &candidates, current.as_deref())
        };

        if let Some(target_id) = matched {
            self.highlight_item(Some(target_id), true, using_keyboard);
        }

        let is_active = self.is_typeahead_active();
        if is_active {
            let surface = self.clone();
            self.typeahead_delay.schedule(MENU_TYPEAHEAD_TIMEOUT_MS, move || {
                surface.reset_typeahead();
            });
            true
        } else {
            false
        }
    }

    /// Recomputes whether a coalesced `pointerleave` horizontal delta (`clientX - last_pointer_x`)
    /// is moving strictly away from the target submenu surface (`radix menu.tsx:488`).
    pub fn is_pointer_leaving_away_from_submenu(
        side: PlacementSide,
        delta_x: f64,
        from_content_to_trigger: bool,
    ) -> bool {
        if from_content_to_trigger {
            match side {
                PlacementSide::Right => delta_x > SUBMENU_INTENT_MIN_PROGRESS_PX,
                PlacementSide::Left => delta_x < -SUBMENU_INTENT_MIN_PROGRESS_PX,
                _ => false,
            }
        } else {
            match side {
                PlacementSide::Right => delta_x < -SUBMENU_INTENT_MIN_PROGRESS_PX,
                PlacementSide::Left => delta_x > SUBMENU_INTENT_MIN_PROGRESS_PX,
                _ => false,
            }
        }
    }

    pub fn start_corridor(
        &self,
        mut sub: ActiveCorridorSubmenu,
        exit_point: Point2D,
        from_content_to_trigger: bool,
    ) {
        let Some(trigger_rect) = get_element_client_rect(&sub.trigger_id) else {
            return;
        };
        let Some(content_rect) = get_element_client_rect(&sub.content_id) else {
            return;
        };

        if (!from_content_to_trigger && content_rect.contains_point(exit_point))
            || (from_content_to_trigger && trigger_rect.contains_point(exit_point))
        {
            self.record_pointer(exit_point.x, exit_point.y);
            self.clear_corridor();
            return;
        }

        let trigger_center_x = trigger_rect.left() + trigger_rect.width * 0.5;
        let content_center_x = content_rect.left() + content_rect.width * 0.5;
        let side = if content_rect.right() <= trigger_rect.left() + 1.0 {
            PlacementSide::Left
        } else if content_rect.left() >= trigger_rect.right() - 1.0 {
            PlacementSide::Right
        } else if (content_center_x - trigger_center_x).abs() >= 1.0 {
            if content_center_x < trigger_center_x {
                PlacementSide::Left
            } else {
                PlacementSide::Right
            }
        } else {
            sub.side
        };
        sub.side = side;

        // Recompute horizontal leave direction from `clientX - last_pointer_x` (`radix menu.tsx:488`).
        if let Some(delta_x) = self.grace_corridor.horizontal_leave_delta(exit_point.x)
            && Self::is_pointer_leaving_away_from_submenu(side, delta_x, from_content_to_trigger)
        {
            self.grace_corridor.record_pointer(exit_point.x, exit_point.y);
            if !from_content_to_trigger {
                let mut active_sig = self.active_corridor_sub;
                if let Ok(mut w) = active_sig.try_write() {
                    *w = Some(sub);
                }
                self.disengage_corridor_and_close(Some(exit_point));
            }
            return;
        }
        self.grace_corridor.record_pointer(exit_point.x, exit_point.y);

        let (from_rect, to_rect, timeout_ms) = if from_content_to_trigger {
            (
                content_rect,
                expand_rect2d(&trigger_rect, MENU_SUBMENU_CORRIDOR_BUFFER_PX),
                MENU_SUBMENU_RETURN_DISENGAGE_MS,
            )
        } else {
            (
                inset_rect2d(&trigger_rect, MENU_SUBMENU_TRIGGER_INSET_PX),
                expand_rect2d(&content_rect, MENU_SUBMENU_CORRIDOR_BUFFER_PX),
                MENU_SUBMENU_CORRIDOR_FALLBACK_MS,
            )
        };

        let transit = GraceTransit::new(
            exit_point,
            from_rect,
            to_rect,
            Some(MENU_SUBMENU_GRACE_PADDING_PX),
            false,
        );

        let mut active_sig = self.active_corridor_sub;
        if let Ok(mut w) = active_sig.try_write() {
            *w = Some(sub);
        }

        let surface_cb = self.clone();
        self.grace_corridor.start(transit, Some(timeout_ms), move |outcome| match outcome {
            GraceCorridorOutcome::InTransit(_) => {}
            GraceCorridorOutcome::ReachedTarget(_) => {
                surface_cb.clear_corridor();
            }
            GraceCorridorOutcome::ExitedCorridor(pt) => {
                surface_cb.disengage_corridor_and_close(Some(pt));
            }
            GraceCorridorOutcome::Stalled => {
                surface_cb.disengage_corridor_and_close(None);
            }
        });
    }

    pub fn clear_corridor(&self) {
        self.grace_corridor.clear();
        let mut active_sig = self.active_corridor_sub;
        if let Ok(mut w) = active_sig.try_write() {
            *w = None;
        }
    }

    fn disengage_corridor_and_close(&self, pointer: Option<Point2D>) {
        let sub = self
            .active_corridor_sub
            .try_read()
            .ok()
            .and_then(|g| g.clone());
        self.clear_corridor();
        if let Some(ref sub) = sub {
            (sub.close_sub)();
        }
        let mut matched_item = false;
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
                    matched_item = true;
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
        if !matched_item
            && let Some(sub) = sub
        {
            let pointer_in_trigger = pointer.is_some_and(|pt| {
                get_element_client_rect(&sub.trigger_id).is_some_and(|r| r.contains_point(pt))
            });
            if !pointer_in_trigger
                && self.current_highlighted_id().as_deref() == Some(sub.trigger_id.as_str())
            {
                let focus_container = self.open.try_peek().is_ok_and(|o| *o);
                self.highlight_item(None, focus_container, false);
            }
        }
    }

    pub fn handle_keydown(&self, evt: &KeyboardEvent, root: &MenuRootContext) {
        // Gate Escape/Tab and menu navigation during IME composition (`radix#4153`).
        if is_ime_composing_key(evt) {
            return;
        }
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
        let on_sub_trigger = highlighted_entry.is_some_and(|entry| {
            !entry.disabled && entry.kind == super::types::MenuItemKind::SubTrigger
        });

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
            if is_overlay_escape_consumed() || !is_topmost_overlay_layer(&root_cid) {
                return;
            }
            evt.prevent_default();
            evt.stop_propagation();
            let esc_evt = MenuPreventableEvent::empty();
            if let Some(cb) = self.on_escape_key_down.try_read().ok().and_then(|s| *s) {
                cb.call(esc_evt.clone());
            } else if self.is_submenu
                && let Some(cb) = root
                    .root_surface
                    .on_escape_key_down
                    .try_read()
                    .ok()
                    .and_then(|s| *s)
            {
                cb.call(esc_evt.clone());
            }
            mark_overlay_escape_consumed();
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

    pub fn stop_monitors(&self) {
        if let Some(p) = self.presence.try_read().ok().and_then(|h| *h) {
            p.stop();
        }
        if let Some(f) = self.floating.try_read().ok().and_then(|h| *h) {
            f.clear();
        }
        if let Some(d) = self.dismiss.try_read().ok().and_then(|h| *h) {
            d.stop();
        }
        if let Some(m) = self.modal_layer.try_read().ok().and_then(|h| *h) {
            m.release();
        }
        self.clear_corridor();
        self.typeahead_delay.cancel();
    }
}

/// Provide lifecycle hooks (`use_presence`, `use_floating_layer`, `use_modal_layer`,
/// `use_dismissable_layer`, `use_focus_scope`, window blur) for a root or submenu `MenuSurfaceRuntime`.
pub fn use_menu_surface_lifecycle(surface: MenuSurfaceRuntime, root: MenuRootContext) {
    let is_open = (surface.open)();
    let content_id = surface.current_content_id();
    let wrapper_id = surface.current_wrapper_id();
    let portal_id_opt = surface.portal_id.read().clone();
    let is_static = *surface.is_static_surface.read();

    // 1. Presence (`use_presence` + Menubar fast-switch skip `MB-5`)
    let on_complete = (!surface.is_submenu).then(|| {
        let complete_slot = root.on_open_change_complete;
        EventHandler::new(move |open: bool| {
            if let Some(cb) = complete_slot.try_read().ok().and_then(|s| *s) {
                cb.call(open);
            }
        })
    });
    let presence = use_presence(is_open, &content_id, true, on_complete);
    if !is_open && root.skip_exit_animation.try_peek().is_ok_and(|s| *s) {
        if let Some(cycle) = presence.active_close_cycle() {
            presence.complete_close_cycle(cycle);
        }
        let mut skip_sig = root.skip_exit_animation;
        let _ = skip_sig.try_write().map(|mut w| *w = false);
    }
    let mut presence_slot = surface.presence;
    presence_slot.set(Some(presence));

    // 2. Floating layer (`use_floating_layer` + z-index hoisting + VirtualPoint + Hide)
    let should_render = surface.should_render_content();
    let anchor_id = surface.anchor_id.read().clone();
    let virtual_anchor = if !surface.is_submenu && root.variant == MenuVariant::ContextMenu {
        (*root.virtual_anchor.read()).map(|(x, y)| {
            (
                x as f32,
                y as f32,
                0.0,
                0.0,
                (!anchor_id.is_empty()).then(|| anchor_id.clone()),
            )
        })
    } else {
        None
    };
    let mut floating_spec = FloatingMonitorSpec::new(anchor_id, content_id.clone());
    floating_spec.virtual_anchor = virtual_anchor;
    floating_spec.arrow_id = surface.arrow_id.read().clone();
    floating_spec.side = *surface.side.read();
    floating_spec.align = *surface.align.read();
    floating_spec.direction = root.direction();
    floating_spec.side_offset = *surface.side_offset.read();
    floating_spec.align_offset = *surface.align_offset.read();
    floating_spec.avoid_collisions = *surface.avoid_collisions.read();
    floating_spec.collision_padding = *surface.collision_padding.read();
    floating_spec.arrow_padding = *surface.arrow_padding.read();
    floating_spec.hide_when_detached = *surface.hide_when_detached.read();
    floating_spec.sticky = matches!(*surface.sticky.read(), StickyStrategy::Always);
    floating_spec.namespace = Some(root.variant.geometry_namespace().to_owned());
    floating_spec.scroll_policy = FloatingScrollPolicy::Reposition;

    let floating = use_floating_layer(should_render && !is_static, floating_spec, None);
    let mut floating_slot = surface.floating;
    floating_slot.set(Some(floating));

    use_effect(use_reactive((&should_render,), move |(render,)| {
        if !render {
            floating.clear();
        }
    }));

    // 3. Modal shielding + scroll lock (`ModalShieldPolicy::AriaHiddenOnly` for ContextMenu)
    let is_modal = (root.modal)();
    let prevent_scroll = *surface.prevent_scroll.read();
    let subcontents = surface.child_subcontents.read().clone();
    let branches = surface.branches.read().clone();

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

    let modal_active = !surface.is_submenu && is_open && (is_modal || prevent_scroll);
    let shield_policy = if !is_modal {
        ModalShieldPolicy::None
    } else if root.variant == MenuVariant::ContextMenu {
        ModalShieldPolicy::AriaHiddenOnly
    } else {
        ModalShieldPolicy::InertAndAriaHidden
    };
    let modal_cfg = ModalLayerConfig {
        lock_id: surface.surface_key.clone(),
        keep_ids: keep_ids.clone(),
        shield_policy,
        prevent_scroll: is_modal || prevent_scroll,
        prevent_body_pointer_events: is_modal && root.variant != MenuVariant::ContextMenu,
    };
    let modal_handle = use_modal_layer(modal_active, modal_cfg);
    let mut modal_slot = surface.modal_layer;
    modal_slot.set(Some(modal_handle));

    // 4. Dismissable layer (`enabled: false` on submenus so MenuSubContent stays grouped with root tree)
    let mut boundaries = keep_ids;
    let trigger_id = root.trigger_id.read().clone();
    match root.variant {
        MenuVariant::DropdownMenu => {
            if !trigger_id.is_empty() && !boundaries.contains(&trigger_id) {
                boundaries.push(trigger_id.clone());
            }
        }
        MenuVariant::Menubar => {
            if let Some(bridge) = root.menubar_bridge.as_ref() {
                for tid in (bridge.all_trigger_ids)() {
                    if !boundaries.contains(&tid) {
                        boundaries.push(tid);
                    }
                }
            }
        }
        MenuVariant::ContextMenu | MenuVariant::Menu => {}
    }

    let dismiss_cfg = DismissConfig::new(content_id.clone())
        .with_boundaries(boundaries)
        .with_modal(is_modal)
        .with_disable_outside_pointer_events(false);

    let esc_slot = surface.on_escape_key_down;
    let pd_slot = surface.on_pointer_down_outside;
    let fo_slot = surface.on_focus_outside;
    let io_slot = surface.on_interact_outside;
    let dismiss_root = root.clone();
    let dismiss_callbacks = DismissCallbacks {
        on_escape_key_down: Some(EventHandler::new(move |evt| {
            if let Some(cb) = esc_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
        on_pointer_down_outside: Some(EventHandler::new(move |evt| {
            if let Some(cb) = pd_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
        on_focus_outside: Some(EventHandler::new(move |evt| {
            if let Some(cb) = fo_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
        on_interact_outside: Some(EventHandler::new(move |evt| {
            if let Some(cb) = io_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
        on_dismiss: Some(EventHandler::new(move |evt: DismissEvent| match evt {
            DismissEvent::Escape => dismiss_root.close_menu_tree(true),
            DismissEvent::PointerDownOutside | DismissEvent::FocusOutside => {
                if let Some(bridge) = dismiss_root.menubar_bridge.as_ref() {
                    (bridge.close_menubar)();
                } else {
                    dismiss_root.close_menu_tree(false);
                }
            }
        })),
    };
    let dismiss_handle = use_dismissable_layer(
        !surface.is_submenu && is_open,
        dismiss_cfg,
        dismiss_callbacks,
    );
    let mut dismiss_slot = surface.dismiss;
    dismiss_slot.set(Some(dismiss_handle));

    // 5. Focus scope (`use_focus_scope` for close focus restoration + custom open-item focus)
    let oa_slot = surface.on_open_auto_focus;
    let ca_slot = surface.on_close_auto_focus;
    let mut focus_cfg = FocusScopeConfig::new(content_id);
    focus_cfg.auto_focus_on_open = false;
    focus_cfg.restore_focus_on_close = !surface.is_submenu;
    let focus_scope = use_focus_scope(
        is_open,
        focus_cfg,
        Some(EventHandler::new(move |evt| {
            if let Some(cb) = oa_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
        Some(EventHandler::new(move |evt| {
            if let Some(cb) = ca_slot.try_read().ok().and_then(|s| *s) {
                cb.call(evt);
            }
        })),
    );
    let mut focus_slot = surface.focus_scope;
    focus_slot.set(Some(focus_scope));

    // 6. Window blur dismiss (`M-15`) via `WatcherSlot`
    let blur_watcher = use_watcher_slot();
    {
        let blur_root = root.clone();
        let is_root_menu = !surface.is_submenu;
        use_effect(use_reactive((&is_open,), move |(open_now,)| {
            if is_root_menu && open_now {
                let br = blur_root.clone();
                blur_watcher.arm_with(move |_| {
                    start_window_blur_monitor(move || {
                        if let Some(bridge) = br.menubar_bridge.as_ref() {
                            (bridge.close_menubar)();
                        } else {
                            br.close_menu_tree(false);
                        }
                    })
                });
            } else {
                blur_watcher.clear();
            }
        }));
    }

    // 7. Open/close focus transitions
    {
        let focus_surface = surface.clone();
        let focus_root = root.clone();
        use_effect(move || {
            let open_now = (focus_surface.open)();
            if !open_now {
                let next_gen = focus_surface.open_focus_gen.get().wrapping_add(1);
                focus_surface.open_focus_gen.set(next_gen);

                let armed = focus_surface
                    .focus_on_open_armed
                    .try_read()
                    .map(|g| *g)
                    .unwrap_or(false);
                if armed {
                    let mut arm_sig = focus_surface.focus_on_open_armed;
                    let _ = arm_sig.try_write().map(|mut w| *w = false);

                    if !focus_surface.is_submenu {
                        let mut prev_sig = focus_root.previously_focused_id;
                        let prev_focus_id =
                            prev_sig.try_write().ok().and_then(|mut w| w.take());

                        let ignore = focus_root
                            .ignore_close_auto_focus
                            .try_read()
                            .map(|g| *g)
                            .unwrap_or(false);

                        if !ignore {
                            let restore_target = if focus_root.variant == MenuVariant::ContextMenu {
                                let trig = focus_root
                                    .trigger_id
                                    .try_read()
                                    .map(|g| g.clone())
                                    .unwrap_or_else(|_| {
                                        focus_root.relationships.trigger_id().to_owned()
                                    });
                                prev_focus_id
                                    .filter(|pid| pid != &trig || is_element_tabbable_by_id(&trig))
                            } else {
                                Some(
                                    focus_root
                                        .trigger_id
                                        .try_read()
                                        .map(|g| g.clone())
                                        .unwrap_or_else(|_| {
                                            focus_root.relationships.trigger_id().to_owned()
                                        }),
                                )
                            };

                            focus_scope.set_restore_target(restore_target);
                            let using_kb = focus_root
                                .using_keyboard
                                .try_read()
                                .map(|g| *g)
                                .unwrap_or(false);
                            focus_scope.apply_close_focus(!using_kb);
                        }

                        let mut ig = focus_root.ignore_close_auto_focus;
                        let _ = ig.try_write().map(|mut w| *w = false);
                    }
                }
                return;
            }

            let mut arm_sig = focus_surface.focus_on_open_armed;
            let was_armed = arm_sig.try_peek().map(|g| *g).unwrap_or(false);
            if !was_armed {
                let _ = arm_sig.try_write().map(|mut w| *w = true);
                if !focus_surface.is_submenu {
                    let prev_focus = oxidase::dom::Document::current()
                        .and_then(|d| d.active_element())
                        .filter(|e| !matches!(e.tag_name().as_deref(), Some("body" | "html")))
                        .and_then(|e| e.id())
                        .filter(|id| !id.is_empty());
                    let mut prev_sig = focus_root.previously_focused_id;
                    let _ = prev_sig.try_write().map(|mut w| *w = prev_focus);
                    let mut ig = focus_root.ignore_close_auto_focus;
                    let _ = ig.try_write().map(|mut w| *w = false);
                }
                focus_surface.apply_open_focus(&focus_root);
            }
        });
    }

    let cleanup_surface = surface;
    use_drop(move || {
        cleanup_surface.clear_corridor();
        cleanup_surface.typeahead_delay.cancel();
    });
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
    pub on_open_change: CopyValue<Option<EventHandler<bool>>>,
    pub open_delay_ms: u64,
    pub parent_surface: MenuSurfaceRuntime,
    pub surface: MenuSurfaceRuntime,
    pub hover_delay: CancellableDelay,
    pub opened_by_pointer_move: Signal<bool>,
}

impl MenuSubContext {
    pub fn is_open(&self) -> bool {
        self.open.try_read().map(|g| *g).unwrap_or(false)
    }

    pub fn cancel_hover_timer(&self) {
        self.hover_delay.cancel();
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
            self.parent_surface.register_child_subcontent(
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
            if active
                .try_peek()
                .is_ok_and(|g| g.as_deref() == Some(&self.sub_id))
            {
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
        if let Some(cb) = self.on_open_change.try_read().ok().and_then(|s| *s) {
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
            self.parent_surface
                .highlight_item(Some(trigger_id), true, using_keyboard);
        }
    }
}

fn inset_rect2d(rect: &Rect2D, inset: f64) -> Rect2D {
    let clamped_x = inset.max(0.0).min(rect.width * 0.45);
    let clamped_y = inset.max(0.0).min(rect.height * 0.45);
    Rect2D::new(
        rect.x + clamped_x,
        rect.y + clamped_y,
        (rect.width - clamped_x * 2.0).max(0.0),
        (rect.height - clamped_y * 2.0).max(0.0),
    )
}

fn expand_rect2d(rect: &Rect2D, padding: f64) -> Rect2D {
    let p = padding.max(0.0);
    Rect2D::new(
        rect.x - p,
        rect.y - p,
        rect.width + p * 2.0,
        rect.height + p * 2.0,
    )
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
                    || (!inside_menu
                        && !el.has_attribute("inert")
                        && !el.has_attribute("aria-hidden"))
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

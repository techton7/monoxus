use dioxus::prelude::*;

pub use monoxus_core::foundation::overlay::focus::*;

use crate::foundation::{
    browser::{
        DEFAULT_FOCUSABLE_SELECTOR, FocusOptions, ModalShieldGuard, ModalShieldMode,
        acquire_scroll_lock_with_options, focus_element_sync_or_defer, get_tabbable_candidates,
        is_element_focused_by_id, is_focus_within_element, release_scroll_lock,
        remove_from_tab_order, restore_focus_element_by_id_with_options, restore_tab_order,
        shield_external_dom, spawn_root_task,
    },
    keyboard::{NavKey, is_ime_composing_key, translate_nav_key},
    shared::{CloseAutoFocusEvent, OpenAutoFocusEvent, PreventableEvent},
};

use super::dismiss::{
    acquire_modal_pointer_lock, is_modal_pointer_lock_active, release_modal_pointer_lock,
};

/// External DOM shielding policy for [`use_modal_layer`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ModalShieldPolicy {
    #[default]
    None,
    /// Applies `aria-hidden="true"` only (e.g. `ContextMenu` so sibling right-clicks remain active).
    AriaHiddenOnly,
    /// Applies both `aria-hidden="true"` and `inert=""` (standard modal Dialog/Popover/Menu default).
    InertAndAriaHidden,
}

impl ModalShieldPolicy {
    #[inline]
    pub const fn to_shield_mode(self) -> Option<ModalShieldMode> {
        match self {
            Self::None => None,
            Self::AriaHiddenOnly => Some(ModalShieldMode::AriaHiddenOnly),
            Self::InertAndAriaHidden => Some(ModalShieldMode::Suppress),
        }
    }
}

/// Configuration for [`use_modal_layer`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalLayerConfig {
    pub lock_id: String,
    pub keep_ids: Vec<String>,
    pub shield_policy: ModalShieldPolicy,
    pub prevent_scroll: bool,
    pub prevent_body_pointer_events: bool,
}

impl ModalLayerConfig {
    pub fn new(lock_id: impl Into<String>, keep_ids: Vec<String>) -> Self {
        Self {
            lock_id: lock_id.into(),
            keep_ids,
            shield_policy: ModalShieldPolicy::InertAndAriaHidden,
            prevent_scroll: true,
            prevent_body_pointer_events: true,
        }
    }
}

/// Zero-overhead `Copy` handle returned by [`use_modal_layer`].
#[derive(Clone, Copy, PartialEq)]
pub struct ModalLayerHandle {
    shield_guard: CopyValue<Option<ModalShieldGuard>>,
    scroll_lock_held: CopyValue<bool>,
    pointer_lock_held: CopyValue<bool>,
    lock_id: CopyValue<String>,
}

impl ModalLayerHandle {
    pub fn should_force_pointer_events_auto(&self) -> bool {
        is_modal_pointer_lock_active()
    }

    /// Immediately releases external DOM shielding, scroll lock, and render-time pointer lock.
    pub fn release(&self) {
        let mut shield_slot = self.shield_guard;
        let old_shield = shield_slot.try_write().ok().and_then(|mut w| w.take());
        drop(old_shield);

        let mut sl_slot = self.scroll_lock_held;
        let was_scroll_locked = sl_slot
            .try_write()
            .map(|mut w| {
                let prev = *w;
                *w = false;
                prev
            })
            .unwrap_or(false);
        if was_scroll_locked
            && let Ok(lock_id) = self.lock_id.try_read().map(|g| g.clone())
            && !lock_id.is_empty()
        {
            release_scroll_lock(&lock_id, None);
        }

        let mut pl_slot = self.pointer_lock_held;
        let was_pointer_locked = pl_slot
            .try_write()
            .map(|mut w| {
                let prev = *w;
                *w = false;
                prev
            })
            .unwrap_or(false);
        if was_pointer_locked {
            release_modal_pointer_lock();
        }
    }
}

/// Shared modal layer hook coordinating external DOM shielding via `browser::shield_external_dom`,
/// body scroll lock, and render-time `MODAL_POINTER_LOCK_COUNT` (`radix#4105`).
pub fn use_modal_layer(active: bool, config: ModalLayerConfig) -> ModalLayerHandle {
    let shield_guard = use_hook(|| CopyValue::new(Option::<ModalShieldGuard>::None));
    let scroll_lock_held = use_hook(|| CopyValue::new(false));
    let mut pointer_lock_held = use_hook(|| CopyValue::new(false));
    let mut lock_id_slot = use_hook(|| CopyValue::new(config.lock_id.clone()));

    if lock_id_slot
        .try_read()
        .is_ok_and(|cur| cur.as_str() != config.lock_id.as_str())
    {
        lock_id_slot.set(config.lock_id.clone());
    }

    let wants_pointer_lock = active && config.prevent_scroll && config.prevent_body_pointer_events;
    let is_pointer_locked = pointer_lock_held.try_read().map(|g| *g).unwrap_or(false);
    if wants_pointer_lock && !is_pointer_locked {
        acquire_modal_pointer_lock();
        pointer_lock_held.set(true);
    } else if !wants_pointer_lock && is_pointer_locked {
        release_modal_pointer_lock();
        pointer_lock_held.set(false);
    }

    let handle = ModalLayerHandle {
        shield_guard,
        scroll_lock_held,
        pointer_lock_held,
        lock_id: lock_id_slot,
    };

    use_effect(use_reactive((&active, &config), move |(is_active, cur_cfg)| {
        let mut shield_slot = handle.shield_guard;
        let mut sl_slot = handle.scroll_lock_held;

        if !is_active {
            let old = shield_slot.try_write().ok().and_then(|mut w| w.take());
            drop(old);
            let was_held = sl_slot
                .try_write()
                .map(|mut w| {
                    let prev = *w;
                    *w = false;
                    prev
                })
                .unwrap_or(false);
            if was_held && !cur_cfg.lock_id.is_empty() {
                release_scroll_lock(&cur_cfg.lock_id, None);
            }
            return;
        }

        if let Some(mode) = cur_cfg.shield_policy.to_shield_mode() {
            let keep_refs: Vec<&str> = cur_cfg
                .keep_ids
                .iter()
                .filter(|id| !id.is_empty())
                .map(String::as_str)
                .collect();
            let old = shield_slot.try_write().ok().and_then(|mut w| w.take());
            drop(old);
            let next_guard = shield_external_dom(&keep_refs, mode);
            if let Ok(mut w) = shield_slot.try_write() {
                *w = Some(next_guard);
            }
        } else {
            let old = shield_slot.try_write().ok().and_then(|mut w| w.take());
            drop(old);
        }

        if cur_cfg.prevent_scroll && !cur_cfg.lock_id.is_empty() {
            acquire_scroll_lock_with_options(
                &cur_cfg.lock_id,
                cur_cfg.prevent_body_pointer_events,
            );
            if let Ok(mut w) = sl_slot.try_write() {
                *w = true;
            }
        } else {
            let was_held = sl_slot
                .try_write()
                .map(|mut w| {
                    let prev = *w;
                    *w = false;
                    prev
                })
                .unwrap_or(false);
            if was_held && !cur_cfg.lock_id.is_empty() {
                release_scroll_lock(&cur_cfg.lock_id, None);
            }
        }
    }));

    use_drop(move || {
        handle.release();
    });

    handle
}

/// Configuration for [`use_focus_scope`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusScopeConfig {
    pub container_id: String,
    pub restore_target_id: Option<String>,
    pub open_focus_target_id: Option<String>,
    pub focusable_selector: Option<String>,
    pub trap_focus: bool,
    pub loop_focus: bool,
    pub auto_focus_on_open: bool,
    pub restore_focus_on_close: bool,
    /// When `true` (e.g. `HoverCard`), suppresses tabbable descendants via `remove_from_tab_order`
    /// while active and restores them via `restore_tab_order` on close/drop.
    pub suppress_tab_order: bool,
}

impl FocusScopeConfig {
    pub fn new(container_id: impl Into<String>) -> Self {
        Self {
            container_id: container_id.into(),
            restore_target_id: None,
            open_focus_target_id: None,
            focusable_selector: None,
            trap_focus: false,
            loop_focus: true,
            auto_focus_on_open: true,
            restore_focus_on_close: true,
            suppress_tab_order: false,
        }
    }
}

/// Zero-overhead `Copy` handle returned by [`use_focus_scope`].
#[derive(Clone, Copy, PartialEq)]
pub struct FocusScopeHandle {
    alive: CopyValue<bool>,
    open_gen: CopyValue<u64>,
    was_active: CopyValue<bool>,
    restore_target: CopyValue<Option<String>>,
    config: CopyValue<FocusScopeConfig>,
    on_open_auto_focus: CopyValue<Option<EventHandler<OpenAutoFocusEvent>>>,
    on_close_auto_focus: CopyValue<Option<EventHandler<CloseAutoFocusEvent>>>,
}

impl FocusScopeHandle {
    pub fn is_alive(&self) -> bool {
        self.alive.try_read().map(|g| *g).unwrap_or(false)
    }

    /// Captures or overrides the element ID that should receive restored focus on close.
    pub fn set_restore_target(&self, target_id: Option<String>) {
        let mut slot = self.restore_target;
        if let Ok(mut w) = slot.try_write() {
            *w = target_id;
        }
    }

    /// Synchronously updates the target container ID when a child `*Content` mounts with a custom `id`.
    pub fn rename_container_id(&self, new_id: &str) {
        if new_id.is_empty() {
            return;
        }
        let mut cfg_slot = self.config;
        if let Ok(mut cfg) = cfg_slot.try_write()
            && cfg.container_id != new_id
        {
            cfg.container_id = new_id.to_owned();
        }
    }

    /// Fires `on_open_auto_focus` (checking `defaultPrevented`, `bits#2219`) and, unless prevented,
    /// focuses the target with a bounded 5-frame `next_frame()` retry loop.
    pub fn apply_open_focus(&self) {
        if !self.is_alive() {
            return;
        }
        let Ok(cfg) = self.config.try_read().map(|g| g.clone()) else {
            return;
        };
        if !cfg.auto_focus_on_open || cfg.container_id.is_empty() {
            return;
        }
        if !super::is_topmost_overlay_layer(&cfg.container_id) {
            return;
        }

        let evt: OpenAutoFocusEvent = PreventableEvent::empty();
        if let Some(handler) = self.on_open_auto_focus.try_read().ok().and_then(|s| *s) {
            handler.call(evt.clone());
        }
        if evt.is_default_prevented() {
            return;
        }

        let mut gen_slot = self.open_gen;
        let expected_gen = gen_slot
            .try_write()
            .map(|mut g| {
                *g = g.wrapping_add(1);
                *g
            })
            .unwrap_or(0);

        let container_id = cfg.container_id;
        let preferred_id = cfg.open_focus_target_id.filter(|id| !id.is_empty());
        let selector = cfg
            .focusable_selector
            .unwrap_or_else(|| DEFAULT_FOCUSABLE_SELECTOR.to_owned());

        if try_focus_scope_target_sync(&container_id, preferred_id.as_deref(), &selector) {
            return;
        }

        let handle = *self;
        spawn_root_task(async move {
            for _ in 0..5 {
                let _ = oxidase::frame::next_frame().await;
                if !handle.is_alive()
                    || handle.open_gen.try_read().map(|g| *g).unwrap_or(0) != expected_gen
                {
                    return;
                }
                let live_cid = handle
                    .config
                    .try_read()
                    .map(|g| g.container_id.clone())
                    .unwrap_or_else(|_| container_id.clone());
                if !super::is_topmost_overlay_layer(&live_cid) {
                    return;
                }
                if try_focus_scope_target_sync(&live_cid, preferred_id.as_deref(), &selector) {
                    return;
                }
            }
        });
    }

    /// Fires `on_close_auto_focus` and, unless prevented, restores focus using
    /// `FocusOptions::with_focus_visible(true, Some(!was_pointer_dismiss))`.
    pub fn apply_close_focus(&self, was_pointer_dismiss: bool) {
        let mut gen_slot = self.open_gen;
        let _ = gen_slot.try_write().map(|mut g| *g = g.wrapping_add(1));

        let Ok(cfg) = self.config.try_read().map(|g| g.clone()) else {
            return;
        };
        if !cfg.restore_focus_on_close {
            return;
        }

        let evt: CloseAutoFocusEvent = PreventableEvent::empty();
        if let Some(handler) = self.on_close_auto_focus.try_read().ok().and_then(|s| *s) {
            handler.call(evt.clone());
        }
        if evt.is_default_prevented() {
            return;
        }

        let target = self
            .restore_target
            .try_read()
            .ok()
            .and_then(|g| g.clone())
            .or(cfg.restore_target_id)
            .filter(|id| !id.is_empty());
        if let Some(target_id) = target {
            let options = FocusOptions::with_focus_visible(true, Some(!was_pointer_dismiss));
            restore_focus_element_by_id_with_options(&target_id, options);
        }
    }

    /// Traps `Tab` / `Shift+Tab` inside `container_id` when `trap_focus && loop_focus` is enabled,
    /// filtering IME composition (`radix#4153`). Returns `true` if the Tab key was intercepted.
    pub fn handle_tab_keydown(&self, evt: &KeyboardEvent) -> bool {
        if !self.is_alive() || is_ime_composing_key(evt) {
            return false;
        }
        if translate_nav_key(&evt.key()) != Some(NavKey::Tab) {
            return false;
        }
        let Ok(cfg) = self.config.try_read().map(|g| g.clone()) else {
            return false;
        };
        if !cfg.trap_focus || !cfg.loop_focus || cfg.container_id.is_empty() {
            return false;
        }

        let candidates = get_tabbable_candidates(&cfg.container_id);
        if candidates.is_empty() {
            evt.prevent_default();
            focus_element_sync_or_defer(&cfg.container_id, FocusOptions::new(true));
            return true;
        }

        let shift = evt.modifiers().shift();
        let active_idx = candidates.iter().position(|id| is_element_focused_by_id(id));

        let next_target = match (active_idx, shift) {
            (Some(0), true) => candidates.last(),
            (Some(idx), false) if idx + 1 == candidates.len() => candidates.first(),
            (None, true) => candidates.last(),
            (None, false) => candidates.first(),
            _ => None,
        };

        if let Some(target_id) = next_target {
            evt.prevent_default();
            focus_element_sync_or_defer(target_id, FocusOptions::with_focus_visible(true, Some(true)));
            true
        } else {
            false
        }
    }
}

fn try_focus_scope_target_sync(
    container_id: &str,
    preferred_id: Option<&str>,
    selector: &str,
) -> bool {
    let Some(doc) = oxidase::dom::Document::current() else {
        return false;
    };
    let options = FocusOptions::new(true);

    if let Some(pid) = preferred_id
        && let Some(el) = doc.element_by_id(pid)
    {
        let _ = el.focus_sync_with_options(options);
        if el.has_focus() || is_focus_within_element(pid) {
            return true;
        }
    }

    let Some(root) = doc.element_by_id(container_id) else {
        return false;
    };
    let target = if root.matches(selector) {
        root.clone()
    } else {
        root.query_selector(selector)
            .unwrap_or_else(|| root.clone())
    };
    if target.id() == root.id() && !root.has_attribute("tabindex") {
        root.set_attribute("tabindex", "-1");
    }
    let _ = target.focus_sync_with_options(options);
    target.has_focus() || is_focus_within_element(container_id)
}

/// Shared zero-overhead focus scope hook with bounded 5-frame `next_frame()` open-focus retry,
/// `on_open_auto_focus` `defaultPrevented` gating (`bits#2219`), close-focus restoration with
/// `FocusOptions::with_focus_visible`, and optional tab-order suppression (for `HoverCard`).
pub fn use_focus_scope(
    active: bool,
    config: FocusScopeConfig,
    on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
    on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
) -> FocusScopeHandle {
    let mut alive = use_hook(|| CopyValue::new(true));
    let open_gen = use_hook(|| CopyValue::new(0_u64));
    let was_active = use_hook(|| CopyValue::new(false));
    let restore_target = use_hook(|| CopyValue::new(config.restore_target_id.clone()));
    let mut config_slot = use_hook(|| CopyValue::new(config.clone()));
    let mut on_open_slot = use_hook(|| CopyValue::new(on_open_auto_focus));
    let mut on_close_slot = use_hook(|| CopyValue::new(on_close_auto_focus));

    config_slot.set(config.clone());
    on_open_slot.set(on_open_auto_focus);
    on_close_slot.set(on_close_auto_focus);

    let handle = FocusScopeHandle {
        alive,
        open_gen,
        was_active,
        restore_target,
        config: config_slot,
        on_open_auto_focus: on_open_slot,
        on_close_auto_focus: on_close_slot,
    };

    use_effect(use_reactive((&active, &config), move |(is_active, cur_cfg)| {
        let prev_active = handle.was_active.try_read().map(|g| *g).unwrap_or(false);
        if is_active && !prev_active {
            let mut wa = handle.was_active;
            if let Ok(mut w) = wa.try_write() {
                *w = true;
            }
            if cur_cfg.restore_target_id.is_some() {
                handle.set_restore_target(cur_cfg.restore_target_id.clone());
            }
            if cur_cfg.suppress_tab_order && !cur_cfg.container_id.is_empty() {
                remove_from_tab_order(&cur_cfg.container_id);
                let cid = cur_cfg.container_id.clone();
                let h = handle;
                spawn_root_task(async move {
                    let _ = oxidase::frame::next_frame().await;
                    if h.is_alive() && h.was_active.try_read().is_ok_and(|a| *a) {
                        remove_from_tab_order(&cid);
                    }
                });
            } else {
                handle.apply_open_focus();
            }
        } else if !is_active && prev_active {
            let mut wa = handle.was_active;
            if let Ok(mut w) = wa.try_write() {
                *w = false;
            }
            let mut gen_slot = handle.open_gen;
            let _ = gen_slot.try_write().map(|mut g| *g = g.wrapping_add(1));
            if cur_cfg.suppress_tab_order && !cur_cfg.container_id.is_empty() {
                restore_tab_order(&cur_cfg.container_id);
            }
        }
    }));

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        let mut gen_slot = handle.open_gen;
        let _ = gen_slot.try_write().map(|mut g| *g = g.wrapping_add(1));
        if let Ok(cfg) = handle.config.try_read().map(|g| g.clone())
            && cfg.suppress_tab_order
            && !cfg.container_id.is_empty()
        {
            restore_tab_order(&cfg.container_id);
        }
    });

    handle
}

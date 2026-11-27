use dioxus::prelude::*;

use crate::foundation::{
    browser::{FocusOptions, focus_element_sync_or_defer, query_descendant_order},
    keyboard::is_ime_composing_key,
    shared::Direction,
    state::{ControllableSignal, sync_signal},
};
use crate::menu::{MenuKey, MenuOpenFocusTarget, translate_menu_key};

use super::{
    relationships::MenubarRelationships,
    state::MenubarState,
    types::{MenubarCrossMenuResult, MenubarMenuEntry, MenubarNavigationStep},
};

#[derive(Clone)]
pub struct MenubarMenuHandles {
    pub value: String,
    pub skip_exit_animation: Signal<bool>,
    pub ignore_close_auto_focus: Signal<bool>,
    pub open_focus_target: Signal<MenuOpenFocusTarget>,
}

/// Shared coordinator runtime for `Menubar.Root`, child `Menubar.Menu`s, and `Menubar.Trigger`s (`MB-1`..`MB-7`).
#[derive(Clone)]
pub struct MenubarRuntimeContext {
    pub relationships: MenubarRelationships,
    pub root_id: Signal<String>,
    pub value_state: ControllableSignal<String>,
    pub state: Signal<MenubarState>,
    pub handles: Signal<Vec<MenubarMenuHandles>>,
    pub submenu_open_delay_ms: Signal<Option<u64>>,
}

impl MenubarRuntimeContext {
    pub fn new(
        relationships: MenubarRelationships,
        value_state: ControllableSignal<String>,
        dir: Direction,
        loop_focus: bool,
        submenu_open_delay_ms: Option<u64>,
    ) -> Self {
        let initial_active = value_state
            .try_peek_cloned()
            .filter(|v| !v.is_empty());
        let initial_state = MenubarState::new(relationships.clone())
            .with_active_value(initial_active)
            .with_direction(dir)
            .with_loop_focus(loop_focus);
        let root_id_sig = use_signal(|| relationships.root_id().to_owned());
        let mut delay_sig = use_signal(|| submenu_open_delay_ms);
        sync_signal(&mut delay_sig, submenu_open_delay_ms);
        Self {
            relationships,
            root_id: root_id_sig,
            value_state,
            state: use_signal(|| initial_state),
            handles: use_signal(Vec::new),
            submenu_open_delay_ms: delay_sig,
        }
    }

    pub fn current_root_id(&self) -> String {
        self.root_id
            .try_peek()
            .map(|g| g.clone())
            .unwrap_or_else(|_| self.relationships.root_id().to_owned())
    }

    pub fn set_root_id(&self, id: String) {
        let mut sig = self.root_id;
        sync_signal(&mut sig, id);
    }

    pub fn active_value(&self) -> Option<String> {
        self.state
            .try_read()
            .ok()
            .and_then(|s| s.active_value().map(str::to_owned))
    }

    pub fn is_menu_open(&self, value: &str) -> bool {
        self.active_value().as_deref() == Some(value)
    }

    pub fn all_trigger_ids(&self) -> Vec<String> {
        self.state
            .try_read()
            .map(|s| s.menus().iter().map(|m| m.trigger_id.clone()).collect())
            .unwrap_or_default()
    }

    pub fn register_menu(
        &self,
        entry: MenubarMenuEntry,
        menu_handles: MenubarMenuHandles,
    ) {
        {
            let mut st = self.state;
            if let Ok(mut w) = st.try_write() {
                w.register_menu(entry);
            }
        }
        {
            let mut hs = self.handles;
            if let Ok(mut w) = hs.try_write() {
                w.retain(|h| h.value != menu_handles.value);
                w.push(menu_handles);
            }
        }
        self.sync_dom_order();
    }

    pub fn unregister_menu(&self, value: &str) {
        {
            let mut st = self.state;
            if let Ok(mut w) = st.try_write() {
                w.unregister_menu(value);
            }
        }
        {
            let mut hs = self.handles;
            if let Ok(mut w) = hs.try_write() {
                w.retain(|h| h.value != value);
            }
        }
    }

    pub fn sync_dom_order(&self) {
        let trigger_ids: Vec<String> = self
            .state
            .try_peek()
            .map(|s| s.menus().iter().map(|m| m.trigger_id.clone()).collect())
            .unwrap_or_default();
        if trigger_ids.len() <= 1 {
            return;
        }
        let trigger_refs: Vec<&str> = trigger_ids.iter().map(String::as_str).collect();
        let live_root_id = self.current_root_id();
        if let Ok(ordered) = query_descendant_order(&live_root_id, &trigger_refs) {
            let needs_update = self.state.try_peek().is_ok_and(|s| {
                s.menus().iter().enumerate().any(|(idx, m)| {
                    let next_pos = ordered
                        .iter()
                        .position(|id| id == &m.trigger_id)
                        .unwrap_or(m.dom_order);
                    next_pos != m.dom_order || (idx > 0 && s.menus()[idx - 1].dom_order > next_pos)
                })
            });
            if !needs_update {
                return;
            }
            let mut st = self.state;
            if let Ok(mut w) = st.try_write() {
                w.sync_dom_order(&ordered);
            }
        }
    }

    pub fn sync_controlled_value(&self, controlled: Option<&str>) {
        let next = controlled.filter(|v| !v.is_empty()).map(str::to_owned);
        let prev = self.active_value();
        if prev == next {
            return;
        }
        if let (Some(prev_val), Some(_)) = (prev.as_deref(), next.as_deref()) {
            self.mark_fast_swap_cleanup(prev_val);
        }
        if let Some(ref target) = next {
            let mut st = self.state;
            let _ = st.try_write().map(|mut w| w.open_menu(target));
        } else {
            let mut st = self.state;
            let _ = st.try_write().map(|mut w| w.close_all());
        }
    }

    fn mark_fast_swap_cleanup(&self, previous_value: &str) {
        if let Ok(hs) = self.handles.try_read()
            && let Some(h) = hs.iter().find(|h| h.value == previous_value)
        {
            let mut skip = h.skip_exit_animation;
            let _ = skip.try_write().map(|mut w| *w = true);
            let mut ig = h.ignore_close_auto_focus;
            let _ = ig.try_write().map(|mut w| *w = true);
        }
    }

    fn set_menu_open_focus_target(&self, value: &str, target: MenuOpenFocusTarget) {
        if let Ok(hs) = self.handles.try_read()
            && let Some(h) = hs.iter().find(|h| h.value == value)
        {
            let mut tgt = h.open_focus_target;
            let _ = tgt.try_write().map(|mut w| *w = target);
        }
    }

    pub fn open_menu(&self, value: &str, focus_target: MenuOpenFocusTarget) {
        let prev = self.active_value();
        self.set_menu_open_focus_target(value, focus_target);
        if let Some(prev_val) = prev.as_deref()
            && prev_val != value
        {
            self.mark_fast_swap_cleanup(prev_val);
        }
        if !self.value_state.is_controlled() {
            let mut st = self.state;
            let _ = st.try_write().map(|mut w| w.open_menu(value));
        } else {
            let mut st = self.state;
            let _ = st
                .try_write()
                .map(|mut w| w.set_current_tabstop_value(Some(value.to_owned())));
        }
        self.value_state.set(value.to_owned());
    }

    pub fn close_menu(&self, value: &str) {
        if self.active_value().as_deref() == Some(value) {
            self.close_all();
        }
    }

    pub fn close_all(&self) {
        if !self.value_state.is_controlled() {
            let mut st = self.state;
            let _ = st.try_write().map(|mut w| w.close_all());
        }
        self.value_state.set(String::new());
    }

    pub fn on_trigger_pointer_enter(&self, value: &str) {
        let entry = self
            .state
            .try_read()
            .ok()
            .and_then(|s| s.on_trigger_pointer_enter(value).cloned());
        if let Some(next) = entry {
            self.open_menu(&next.value, MenuOpenFocusTarget::ContentContainer);
        }
    }

    pub fn on_trigger_key(&self, current_value: &str, evt: &KeyboardEvent) {
        if is_ime_composing_key(evt) {
            return;
        }
        let Some(key) = translate_menu_key(&evt.key()) else {
            return;
        };
        self.sync_dom_order();
        let step = self
            .state
            .try_read()
            .map(|s| s.on_trigger_key(current_value, key))
            .unwrap_or(MenubarNavigationStep::None);

        match step {
            MenubarNavigationStep::None => {}
            MenubarNavigationStep::FocusTrigger { value, trigger_id } => {
                evt.prevent_default();
                evt.stop_propagation();
                {
                    let mut st = self.state;
                    let _ = st
                        .try_write()
                        .map(|mut w| w.set_current_tabstop_value(Some(value)));
                }
                focus_element_sync_or_defer(
                    &trigger_id,
                    FocusOptions::with_focus_visible(false, Some(true)),
                );
            }
            MenubarNavigationStep::FocusAndOpenMenu { value, trigger_id: _ } => {
                evt.prevent_default();
                evt.stop_propagation();
                self.open_menu(&value, MenuOpenFocusTarget::FirstItem);
            }
            MenubarNavigationStep::OpenCurrentAndFocusFirst { value } => {
                evt.prevent_default();
                evt.stop_propagation();
                self.open_menu(&value, MenuOpenFocusTarget::FirstItem);
            }
            MenubarNavigationStep::OpenCurrentAndFocusLast { value } => {
                evt.prevent_default();
                evt.stop_propagation();
                self.open_menu(&value, MenuOpenFocusTarget::LastItem);
            }
            MenubarNavigationStep::CloseAll => {
                evt.prevent_default();
                evt.stop_propagation();
                self.close_all();
            }
        }
    }

    pub fn handle_cross_menu_arrow(
        &self,
        current_value: &str,
        key: MenuKey,
        on_sub_trigger: bool,
        in_submenu: bool,
    ) -> bool {
        self.sync_dom_order();
        let res = self
            .state
            .try_read()
            .map(|s| s.resolve_cross_menu_arrow(current_value, key, on_sub_trigger, in_submenu))
            .unwrap_or(MenubarCrossMenuResult::None);

        match res {
            MenubarCrossMenuResult::SwitchToMenu { value, .. } => {
                self.open_menu(&value, MenuOpenFocusTarget::FirstItem);
                true
            }
            MenubarCrossMenuResult::None => false,
        }
    }
}

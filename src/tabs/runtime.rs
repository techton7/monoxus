use std::rc::Rc;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};
pub use crate::foundation::state::{ControllableStateProps, use_controllable_state};

use crate::foundation::{
    browser::focus_element_by_id,
    roving_focus::{RovingFocusRuntime, RovingItemMeta, RovingModifiers},
    state::{ControllableSignal, sync_signal},
};

use super::{
    attrs::{TabsContentAttributes, TabsListAttributes, TabsRootAttributes, TabsTriggerAttributes},
    state::Tabs,
    types::TabsValueChangeHandler,
};

#[derive(Clone)]
pub struct TabsRuntimeState {
    pub active_value: Signal<String>,
    pub roving: RovingFocusRuntime,
    pub content_ids: Signal<Vec<(String, String)>>,
    pub mounted_content_ids: Signal<Vec<String>>,
}

#[derive(Clone)]
pub struct TabsRuntime {
    tabs: Signal<Tabs>,
    value_ctl: Option<ControllableSignal<String>>,
    on_value_change: CopyValue<Option<TabsValueChangeHandler>>,
    state: TabsRuntimeState,
}

pub fn use_tabs_runtime<F>(tabs: Tabs, on_value_change: Option<F>) -> TabsRuntime
where
    F: Fn(String) + 'static,
{
    let synced_value_change: Option<TabsValueChangeHandler> =
        on_value_change.map(|f| Rc::new(f) as TabsValueChangeHandler);
    let mut on_value_change_slot: CopyValue<Option<TabsValueChangeHandler>> =
        use_hook(|| CopyValue::new(None));
    on_value_change_slot.set(synced_value_change);

    let incoming_active = tabs.active_value().to_owned();
    let incoming_tab_stop = (!tabs.current_tab_stop().is_empty())
        .then(|| tabs.current_tab_stop().to_owned());

    let roving_cfg = tabs.roving_config().with_enabled(!tabs.is_disabled());
    let roving = RovingFocusRuntime::new(
        roving_cfg,
        incoming_tab_stop.clone(),
        tabs.relationships().root_id(),
    );
    let mut synced_cfg = *roving.config.peek();
    synced_cfg.orientation = roving_cfg.orientation;
    synced_cfg.dir = roving_cfg.dir;
    synced_cfg.enabled = !tabs.is_disabled();
    roving.set_config(synced_cfg);
    roving.set_root_dom_id(tabs.relationships().root_id());

    let state = TabsRuntimeState {
        active_value: use_signal(|| incoming_active.clone()),
        roving,
        content_ids: use_signal(Vec::new),
        mounted_content_ids: use_signal(Vec::new),
    };

    let mut tabs_sig = use_signal(|| tabs.clone());
    sync_signal(&mut tabs_sig, tabs);

    let mut prev_incoming_active = use_hook(|| CopyValue::new(incoming_active.clone()));
    if prev_incoming_active
        .try_read()
        .is_ok_and(|prev| *prev != incoming_active)
    {
        prev_incoming_active.set(incoming_active.clone());
        let mut active = state.active_value;
        sync_signal(&mut active, incoming_active);
        if incoming_tab_stop.is_some() {
            state.roving.set_current_tab_stop(incoming_tab_stop);
        }
    }

    TabsRuntime {
        tabs: tabs_sig,
        value_ctl: None,
        on_value_change: on_value_change_slot,
        state,
    }
}

pub(crate) fn use_tabs_runtime_controllable(
    tabs: Tabs,
    value_ctl: ControllableSignal<String>,
) -> TabsRuntime {
    let on_value_change_slot: CopyValue<Option<TabsValueChangeHandler>> =
        use_hook(|| CopyValue::new(None));

    let incoming_tab_stop = (!tabs.current_tab_stop().is_empty())
        .then(|| tabs.current_tab_stop().to_owned());

    let roving_cfg = tabs.roving_config().with_enabled(!tabs.is_disabled());
    let roving = RovingFocusRuntime::new(
        roving_cfg,
        incoming_tab_stop,
        tabs.relationships().root_id(),
    );
    let mut synced_cfg = *roving.config.peek();
    synced_cfg.orientation = roving_cfg.orientation;
    synced_cfg.dir = roving_cfg.dir;
    synced_cfg.enabled = !tabs.is_disabled();
    roving.set_config(synced_cfg);
    roving.set_root_dom_id(tabs.relationships().root_id());

    let state = TabsRuntimeState {
        active_value: value_ctl.signal(),
        roving,
        content_ids: use_signal(Vec::new),
        mounted_content_ids: use_signal(Vec::new),
    };

    let mut tabs_sig = use_signal(|| tabs.clone());
    sync_signal(&mut tabs_sig, tabs);

    TabsRuntime {
        tabs: tabs_sig,
        value_ctl: Some(value_ctl),
        on_value_change: on_value_change_slot,
        state,
    }
}

impl TabsRuntime {
    pub fn roving(&self) -> RovingFocusRuntime {
        self.state.roving
    }

    pub fn tabs(&self) -> Tabs {
        let mut updated = self.tabs.read().clone();
        updated.select_tab(self.state.active_value.read().clone());
        updated.set_tab_stop(self.current_tab_stop());
        let loop_focus = self.state.roving.config.read().loop_focus;
        updated.with_loop_focus(loop_focus)
    }

    pub fn set_loop_focus(&self, loop_focus: bool) {
        self.state.roving.set_loop_focus(loop_focus);
    }

    pub fn active_value(&self) -> String {
        self.state.active_value.read().clone()
    }

    pub fn resolve_tab_stop(&self) -> Option<String> {
        if self.tabs.read().is_disabled() {
            return None;
        }
        let active = self.state.active_value.read().clone();
        self.state
            .roving
            .resolve_tab_stop_with(|key, _meta| key == active)
    }

    pub fn current_tab_stop(&self) -> String {
        if self.state.roving.registry.read().is_empty() {
            self.state
                .roving
                .current_tab_stop
                .read()
                .clone()
                .unwrap_or_default()
        } else {
            self.resolve_tab_stop().unwrap_or_default()
        }
    }

    pub fn root(&self) -> TabsRootAttributes {
        self.tabs().root()
    }

    pub fn list(&self) -> TabsListAttributes {
        self.tabs().list()
    }

    pub fn trigger(&self, value: &str, disabled: bool) -> TabsTriggerAttributes {
        let mut attrs = self.tabs().trigger(value, disabled);
        if !self.state.roving.registry.read().is_empty() {
            let active = self.state.active_value.read().clone();
            attrs.tabindex = self.state.roving.tab_index_for_with(
                value,
                !attrs.disabled,
                |key, _meta| key == active,
            );
        }
        let is_active = self.active_value() == value;
        let is_mounted = self
            .state
            .mounted_content_ids
            .read()
            .contains(&value.to_string());
        if is_active || is_mounted {
            attrs.aria_controls = Some(self.content_element_id(value));
        } else {
            attrs.aria_controls = None;
        }
        attrs
    }

    pub fn register_content_mounted(&self, value: &str) {
        if self
            .state
            .mounted_content_ids
            .try_peek()
            .is_ok_and(|list| list.iter().any(|v| v == value))
        {
            return;
        }
        let mut mounted_sig = self.state.mounted_content_ids;
        if let Ok(mut list) = mounted_sig.try_write()
            && !list.iter().any(|v| v == value)
        {
            list.push(value.to_string());
        }
    }

    pub fn unregister_content_mounted(&self, value: &str) {
        if !self
            .state
            .mounted_content_ids
            .try_peek()
            .is_ok_and(|list| list.iter().any(|v| v == value))
        {
            return;
        }
        let mut mounted_sig = self.state.mounted_content_ids;
        if let Ok(mut list) = mounted_sig.try_write() {
            list.retain(|v| v != value);
        }
    }

    pub fn unregister_content_id(&self, value: &str) {
        if !self
            .state
            .content_ids
            .try_peek()
            .is_ok_and(|list| list.iter().any(|(v, _)| v == value))
        {
            return;
        }
        let mut cids_sig = self.state.content_ids;
        if let Ok(mut list) = cids_sig.try_write() {
            list.retain(|(v, _)| v != value);
        }
    }

    pub fn content(&self, value: &str) -> TabsContentAttributes {
        self.tabs().content(value)
    }

    pub fn select_tab(&self, value: &str) {
        if self.tabs.peek().is_disabled() {
            return;
        }
        if let Some(ctl) = self.value_ctl {
            ctl.set(value.to_string());
            self.state
                .roving
                .set_current_tab_stop(Some(value.to_string()));
            return;
        }
        let changed = *self.state.active_value.peek() != value;
        if changed {
            let mut active = self.state.active_value;
            active.set(value.to_string());
            let callback = self
                .on_value_change
                .try_read()
                .ok()
                .and_then(|slot| slot.as_ref().cloned());
            if let Some(callback) = callback {
                callback(value.to_string());
            }
        }
        self.state
            .roving
            .set_current_tab_stop(Some(value.to_string()));
    }

    pub fn trigger_element_id(&self, value: &str) -> String {
        self.state
            .roving
            .item_dom_id(value)
            .unwrap_or_else(|| self.tabs.read().relationships().trigger_id(value))
    }

    pub fn content_element_id(&self, value: &str) -> String {
        let map = self.state.content_ids.read();
        map.iter()
            .find(|(val, _)| val == value)
            .map(|(_, id)| id.clone())
            .unwrap_or_else(|| self.tabs.read().relationships().content_id(value))
    }

    pub fn register_trigger_id(&self, value: &str, id: &str) {
        let existing = self
            .state
            .roving
            .registry
            .peek()
            .get(&value.to_string())
            .cloned();
        let focusable = existing
            .as_ref()
            .map(|m| m.focusable)
            .unwrap_or(!self.tabs.peek().is_disabled());
        let active = *self.state.active_value.peek() == value;
        self.state
            .roving
            .register_item(value, RovingItemMeta::new(id, focusable, active));
    }

    pub fn register_content_id(&self, value: &str, id: &str) {
        if self
            .state
            .content_ids
            .try_peek()
            .is_ok_and(|ids| ids.iter().any(|(v, existing_id)| v == value && existing_id == id))
        {
            return;
        }
        let mut list = self.state.content_ids;
        list.with_mut(|ids| {
            if let Some(existing) = ids.iter_mut().find(|(v, _)| v == value) {
                existing.1 = id.to_string();
            } else {
                ids.push((value.to_string(), id.to_string()));
            }
        });
    }

    pub fn move_tab_stop(&self, value: &str) {
        if self.tabs.peek().is_disabled() {
            return;
        }
        if !self.state.roving.focus_item(value) {
            let target_id = self.trigger_element_id(value);
            focus_element_by_id(&target_id);
        }

        if self.tabs.peek().activation_mode().is_automatic() {
            self.select_tab(value);
        }
    }

    pub fn register_trigger(&self, value: &str, disabled: bool) {
        let tabs = self.tabs.peek();
        let dom_id = self
            .state
            .roving
            .registry
            .peek()
            .get(&value.to_string())
            .map(|m| m.dom_id.clone())
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| tabs.relationships().trigger_id(value));
        let focusable = !(tabs.is_disabled() || disabled);
        drop(tabs);
        let active = *self.state.active_value.peek() == value;
        self.state
            .roving
            .register_item(value, RovingItemMeta::new(dom_id, focusable, active));
    }

    pub fn unregister_trigger(&self, value: &str) {
        self.state.roving.unregister_item(value);
    }

    pub fn reconcile_order(&self) {
        self.state.roving.sync_dom_order();
    }

    pub fn navigate_key(&self, key: &str) -> Option<String> {
        if self.tabs.peek().is_disabled() {
            return None;
        }
        let current = self.current_tab_stop();
        let current_opt = (!current.is_empty()).then_some(current.as_str());
        let next_tab = self
            .state
            .roving
            .navigate_key(current_opt, key, RovingModifiers::none());

        if let Some(ref target) = next_tab {
            self.move_tab_stop(target);
        }

        next_tab
    }

    pub fn navigate_arrow(&self, key: &str) -> Option<String> {
        self.navigate_key(key)
    }

    pub fn navigate_boundary(&self, first: bool) -> Option<String> {
        self.navigate_key(if first { "Home" } else { "End" })
    }
}

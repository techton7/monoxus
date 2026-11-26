use std::rc::Rc;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};
pub use crate::foundation::state::{ControllableStateProps, use_controllable_state};

use crate::foundation::{
    browser::focus_element_by_id,
    roving_focus::{RovingFocusRuntime, RovingItemMeta, RovingModifiers, use_roving_focus_group},
    state::sync_signal,
};

use super::{
    attrs::{
        AccordionContentAttributes, AccordionHeaderAttributes, AccordionItemAttributes,
        AccordionRootAttributes, AccordionTriggerAttributes,
    },
    state::Accordion,
    types::AccordionValueChangeHandler,
};

#[derive(Clone)]
pub struct AccordionRuntimeState {
    pub active_values: Signal<Vec<String>>,
    pub roving: RovingFocusRuntime,
    pub mounted_content_ids: Signal<Vec<String>>,
}

#[derive(Clone)]
pub struct AccordionRuntime {
    accordion: Accordion,
    on_value_change: Option<AccordionValueChangeHandler>,
    state: AccordionRuntimeState,
}

pub fn use_accordion_runtime<F>(
    accordion: Accordion,
    on_value_change: Option<F>,
) -> AccordionRuntime
where
    F: Fn(Vec<String>) + 'static,
{
    let synced_value_change: Option<AccordionValueChangeHandler> =
        on_value_change.map(|f| Rc::new(f) as AccordionValueChangeHandler);
    let initial_active = accordion.active_values().to_vec();
    let initial_tab_stop = (!accordion.current_tab_stop().is_empty())
        .then(|| accordion.current_tab_stop().to_owned());

    let roving = use_roving_focus_group(
        accordion.roving_config(),
        initial_tab_stop,
        accordion.relationships().root_id(),
    );

    let state = AccordionRuntimeState {
        active_values: use_signal(|| initial_active),
        roving,
        mounted_content_ids: use_signal(Vec::new),
    };

    let effect_state = state.clone();

    use_effect(use_reactive((&accordion,), move |(accordion,)| {
        let mut active = effect_state.active_values;
        sync_signal(&mut active, accordion.active_values().to_vec());
        let desired_stop = (!accordion.current_tab_stop().is_empty())
            .then(|| accordion.current_tab_stop().to_owned());
        if desired_stop.is_some() {
            effect_state.roving.set_current_tab_stop(desired_stop);
        }
    }));

    AccordionRuntime {
        accordion,
        on_value_change: synced_value_change,
        state,
    }
}

impl AccordionRuntime {
    pub fn roving(&self) -> RovingFocusRuntime {
        self.state.roving
    }

    pub fn accordion(&self) -> Accordion {
        let active = self.state.active_values.read().clone();
        self.accordion
            .clone()
            .with_values(active)
            .with_tab_stop(self.current_tab_stop())
    }

    pub fn active_values(&self) -> Vec<String> {
        self.state.active_values.read().clone()
    }

    pub fn is_open(&self, value: &str) -> bool {
        self.state.active_values.read().iter().any(|v| v == value)
    }

    pub fn resolve_tab_stop(&self) -> Option<String> {
        if self.accordion.is_disabled() {
            return None;
        }
        let active = self.state.active_values.read();
        self.state
            .roving
            .resolve_tab_stop_with(|key, _meta| active.iter().any(|v| v == key))
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

    pub fn root(&self) -> AccordionRootAttributes {
        self.accordion().root()
    }

    pub fn item(&self, value: &str, disabled: bool) -> AccordionItemAttributes {
        self.accordion().item(value, disabled)
    }

    pub fn header(&self, value: &str, disabled: bool) -> AccordionHeaderAttributes {
        self.accordion().header(value, disabled)
    }

    pub fn header_with_level(
        &self,
        value: &str,
        disabled: bool,
        level: u8,
    ) -> AccordionHeaderAttributes {
        self.accordion().header_with_level(value, disabled, level)
    }

    pub fn trigger(&self, value: &str, disabled: bool) -> AccordionTriggerAttributes {
        let mut attrs = self.accordion().trigger(value, disabled);
        let is_open = self.is_open(value);
        let is_mounted = self
            .state
            .mounted_content_ids
            .read()
            .iter()
            .any(|v| v == value);
        attrs.aria_controls = (is_open || is_mounted)
            .then(|| self.accordion.relationships().content_id(value));
        attrs
    }

    pub fn register_content_mounted(&self, value: &str) {
        let mut mounted_sig = self.state.mounted_content_ids;
        if let Ok(mut list) = mounted_sig.try_write()
            && !list.iter().any(|v| v == value)
        {
            list.push(value.to_string());
        }
    }

    pub fn unregister_content_mounted(&self, value: &str) {
        let mut mounted_sig = self.state.mounted_content_ids;
        if let Ok(mut list) = mounted_sig.try_write() {
            list.retain(|v| v != value);
        }
    }

    pub fn content(&self, value: &str) -> AccordionContentAttributes {
        self.accordion().content(value)
    }

    pub fn toggle_item(&self, value: &str) {
        let mut updated = self.accordion();
        if updated.toggle_item(value) {
            let new_values = updated.active_values().to_vec();
            let mut active = self.state.active_values;
            active.set(new_values.clone());
            self.state
                .roving
                .set_current_tab_stop(Some(value.to_string()));
            if let Some(ref callback) = self.on_value_change {
                callback(new_values);
            }
        }
    }

    pub fn trigger_element_id(&self, value: &str) -> String {
        self.state
            .roving
            .item_dom_id(value)
            .unwrap_or_else(|| self.accordion.relationships().trigger_id(value))
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
            .unwrap_or(!self.accordion.is_disabled());
        let active = self.state.active_values.peek().iter().any(|v| v == value);
        self.state
            .roving
            .register_item(value, RovingItemMeta::new(id, focusable, active));
    }

    pub fn move_tab_stop(&self, value: &str) {
        if !self.state.roving.focus_item(value) {
            let target_id = self.trigger_element_id(value);
            focus_element_by_id(&target_id);
        }
    }

    pub fn register_item(&self, value: &str, disabled: bool) {
        let dom_id = self
            .state
            .roving
            .registry
            .peek()
            .get(&value.to_string())
            .map(|m| m.dom_id.clone())
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| self.accordion.relationships().trigger_id(value));
        let focusable = !(self.accordion.is_disabled() || disabled);
        let active = self.state.active_values.peek().iter().any(|v| v == value);
        self.state
            .roving
            .register_item(value, RovingItemMeta::new(dom_id, focusable, active));
    }

    pub fn unregister_item(&self, value: &str) {
        self.state.roving.unregister_item(value);
        self.unregister_content_mounted(value);
    }

    pub fn reconcile_order(&self) {
        self.state.roving.sync_dom_order();
    }

    pub fn navigate_key(&self, key: &str) -> Option<String> {
        if self.accordion.is_disabled() {
            return None;
        }
        let current = self.current_tab_stop();
        let current_opt = (!current.is_empty()).then_some(current.as_str());
        let next_item = self
            .state
            .roving
            .navigate_key(current_opt, key, RovingModifiers::none());
        if let Some(ref target) = next_item {
            self.move_tab_stop(target);
        }
        next_item
    }

    pub fn navigate_arrow(&self, key: &str) -> Option<String> {
        self.navigate_key(key)
    }

    pub fn navigate_boundary(&self, first: bool) -> Option<String> {
        self.navigate_key(if first { "Home" } else { "End" })
    }
}

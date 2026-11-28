use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    browser::focus_element_by_id,
    roving_focus::{RovingFocusRuntime, RovingItemMeta, RovingModifiers, use_roving_focus_group},
    state::{ControllableSignal, sync_signal},
};

use super::{
    attrs::{
        AccordionContentAttributes, AccordionHeaderAttributes, AccordionItemAttributes,
        AccordionRootAttributes, AccordionTriggerAttributes,
    },
    state::Accordion,
    types::AccordionOrientation,
};

type AccordionValueChangeHandler = Rc<dyn Fn(Vec<String>)>;

#[derive(Clone)]
pub struct AccordionRuntimeState {
    pub active_values: Signal<Vec<String>>,
    pub roving: RovingFocusRuntime,
    pub mounted_content_ids: Signal<Vec<String>>,
}

#[derive(Clone)]
pub struct AccordionRuntime {
    accordion: Signal<Accordion>,
    active_ctl: Option<ControllableSignal<Vec<String>>>,
    on_value_change: CopyValue<Option<AccordionValueChangeHandler>>,
    on_single_change: CopyValue<Option<EventHandler<String>>>,
    on_multiple_change: CopyValue<Option<EventHandler<Vec<String>>>>,
    state: AccordionRuntimeState,
}

pub fn use_accordion_runtime<F>(
    accordion: Accordion,
    on_value_change: Option<F>,
) -> AccordionRuntime
where
    F: Fn(Vec<String>) + 'static,
{
    use_accordion_runtime_with_control(accordion, true, on_value_change)
}

pub(crate) fn use_accordion_runtime_with_control<F>(
    accordion: Accordion,
    controlled: bool,
    on_value_change: Option<F>,
) -> AccordionRuntime
where
    F: Fn(Vec<String>) + 'static,
{
    let synced_value_change: Option<AccordionValueChangeHandler> =
        on_value_change.map(|f| Rc::new(f) as AccordionValueChangeHandler);
    let mut on_value_change_slot: CopyValue<Option<AccordionValueChangeHandler>> =
        use_hook(|| CopyValue::new(None));
    on_value_change_slot.set(synced_value_change);
    let on_single_change_slot: CopyValue<Option<EventHandler<String>>> =
        use_hook(|| CopyValue::new(None));
    let on_multiple_change_slot: CopyValue<Option<EventHandler<Vec<String>>>> =
        use_hook(|| CopyValue::new(None));

    let initial_active = accordion.active_values().to_vec();
    let initial_tab_stop = (!accordion.current_tab_stop().is_empty())
        .then(|| accordion.current_tab_stop().to_owned());

    let roving = use_roving_focus_group(
        accordion.roving_config(),
        initial_tab_stop,
        accordion.relationships().root_id(),
    );

    let state = AccordionRuntimeState {
        active_values: use_signal(|| initial_active.clone()),
        roving,
        mounted_content_ids: use_signal(Vec::new),
    };

    let mut accordion_sig = use_signal(|| accordion.clone());
    sync_signal(&mut accordion_sig, accordion.clone());

    let mut prev_incoming_values = use_hook(|| CopyValue::new(initial_active));
    if controlled {
        let incoming_values = accordion.active_values().to_vec();
        if prev_incoming_values
            .try_read()
            .is_ok_and(|prev| *prev != incoming_values)
        {
            prev_incoming_values.set(incoming_values.clone());
            let mut active = state.active_values;
            sync_signal(&mut active, incoming_values);
            let desired_stop = (!accordion.current_tab_stop().is_empty())
                .then(|| accordion.current_tab_stop().to_owned());
            if desired_stop.is_some() {
                state.roving.set_current_tab_stop(desired_stop);
            }
        }
    }

    AccordionRuntime {
        accordion: accordion_sig,
        active_ctl: None,
        on_value_change: on_value_change_slot,
        on_single_change: on_single_change_slot,
        on_multiple_change: on_multiple_change_slot,
        state,
    }
}

pub(crate) fn use_accordion_runtime_controllable(
    accordion: Accordion,
    active_ctl: ControllableSignal<Vec<String>>,
    on_value_change: Option<EventHandler<String>>,
    on_values_change: Option<EventHandler<Vec<String>>>,
) -> AccordionRuntime {
    let on_value_change_slot: CopyValue<Option<AccordionValueChangeHandler>> =
        use_hook(|| CopyValue::new(None));
    let mut on_single_change_slot: CopyValue<Option<EventHandler<String>>> =
        use_hook(|| CopyValue::new(None));
    on_single_change_slot.set(on_value_change);
    let mut on_multiple_change_slot: CopyValue<Option<EventHandler<Vec<String>>>> =
        use_hook(|| CopyValue::new(None));
    on_multiple_change_slot.set(on_values_change);

    let initial_tab_stop = (!accordion.current_tab_stop().is_empty())
        .then(|| accordion.current_tab_stop().to_owned());

    let roving = use_roving_focus_group(
        accordion.roving_config(),
        initial_tab_stop,
        accordion.relationships().root_id(),
    );

    let state = AccordionRuntimeState {
        active_values: active_ctl.signal(),
        roving,
        mounted_content_ids: use_signal(Vec::new),
    };

    let mut accordion_sig = use_signal(|| accordion.clone());
    sync_signal(&mut accordion_sig, accordion);

    AccordionRuntime {
        accordion: accordion_sig,
        active_ctl: Some(active_ctl),
        on_value_change: on_value_change_slot,
        on_single_change: on_single_change_slot,
        on_multiple_change: on_multiple_change_slot,
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
            .read()
            .clone()
            .with_values(active)
            .with_tab_stop(self.current_tab_stop())
    }

    pub fn is_disabled(&self) -> bool {
        self.accordion.read().is_disabled()
    }

    pub(crate) fn is_disabled_peek(&self) -> bool {
        self.accordion.peek().is_disabled()
    }

    pub(crate) fn orientation_peek(&self) -> AccordionOrientation {
        self.accordion.peek().orientation()
    }

    pub fn active_values(&self) -> Vec<String> {
        self.state.active_values.read().clone()
    }

    pub fn is_open(&self, value: &str) -> bool {
        self.state.active_values.read().iter().any(|v| v == value)
    }

    pub fn resolve_tab_stop(&self) -> Option<String> {
        if self.accordion.read().is_disabled() {
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
        let accordion = self.accordion();
        let mut attrs = accordion.trigger(value, disabled);
        let is_open = self.is_open(value);
        let is_mounted = self
            .state
            .mounted_content_ids
            .read()
            .iter()
            .any(|v| v == value);
        attrs.aria_controls = (is_open || is_mounted)
            .then(|| accordion.relationships().content_id(value));
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

    pub fn content(&self, value: &str) -> AccordionContentAttributes {
        self.accordion().content(value)
    }

    pub fn content_with_disabled(&self, value: &str, disabled: bool) -> AccordionContentAttributes {
        self.accordion().content_with_disabled(value, disabled)
    }

    pub fn toggle_item(&self, value: &str) {
        if self.accordion.peek().is_disabled() {
            return;
        }
        let mut updated = self
            .accordion
            .peek()
            .clone()
            .with_values(self.state.active_values.peek().clone())
            .with_tab_stop(self.current_tab_stop());
        if updated.toggle_item(value) {
            let new_values = updated.active_values().to_vec();
            self.state
                .roving
                .set_current_tab_stop(Some(value.to_string()));
            if let Some(ctl) = self.active_ctl {
                ctl.set(new_values.clone());
                if let Some(handler) = self.on_single_change.try_read().ok().and_then(|s| *s) {
                    handler.call(new_values.first().cloned().unwrap_or_default());
                }
                if let Some(handler) = self.on_multiple_change.try_read().ok().and_then(|s| *s) {
                    handler.call(new_values);
                }
                return;
            }
            let mut active = self.state.active_values;
            active.set(new_values.clone());
            let callback = self
                .on_value_change
                .try_read()
                .ok()
                .and_then(|slot| slot.as_ref().cloned());
            if let Some(callback) = callback {
                callback(new_values);
            }
        }
    }

    pub fn trigger_element_id(&self, value: &str) -> String {
        self.state
            .roving
            .item_dom_id(value)
            .unwrap_or_else(|| self.accordion.read().relationships().trigger_id(value))
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
            .unwrap_or(!self.accordion.peek().is_disabled());
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
        let acc = self.accordion.peek();
        let dom_id = self
            .state
            .roving
            .registry
            .peek()
            .get(&value.to_string())
            .map(|m| m.dom_id.clone())
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| acc.relationships().trigger_id(value));
        let focusable = !(acc.is_disabled() || disabled);
        drop(acc);
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
        if self.accordion.peek().is_disabled() {
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

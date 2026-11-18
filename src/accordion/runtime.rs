use std::rc::Rc;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};
pub use crate::foundation::state::{ControllableStateProps, use_controllable_state};

use crate::foundation::browser::focus_element_by_id;

use super::{
    attrs::{
        AccordionContentAttributes, AccordionHeaderAttributes, AccordionItemAttributes,
        AccordionRootAttributes, AccordionTriggerAttributes,
    },
    state::Accordion,
    types::{AccordionItemRegistration, AccordionValueChangeHandler},
};

#[derive(Clone)]
pub struct AccordionRuntimeState {
    pub active_values: Signal<Vec<String>>,
    pub current_tab_stop: Signal<String>,
    pub registered_items: Signal<Vec<AccordionItemRegistration>>,
    pub trigger_ids: Signal<Vec<(String, String)>>,
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
    let initial_tab_stop = accordion.current_tab_stop().to_owned();

    let state = AccordionRuntimeState {
        active_values: use_signal(|| initial_active),
        current_tab_stop: use_signal(|| initial_tab_stop),
        registered_items: use_signal(Vec::new),
        trigger_ids: use_signal(Vec::new),
        mounted_content_ids: use_signal(Vec::new),
    };

    let effect_state = state.clone();

    use_effect(use_reactive((&accordion,), move |(accordion,)| {
        let mut active = effect_state.active_values;
        if *active.peek() != accordion.active_values() {
            active.set(accordion.active_values().to_vec());
        }
        let mut tab_stop = effect_state.current_tab_stop;
        if *tab_stop.peek() != accordion.current_tab_stop() {
            tab_stop.set(accordion.current_tab_stop().to_owned());
        }
    }));

    AccordionRuntime {
        accordion,
        on_value_change: synced_value_change,
        state,
    }
}

impl AccordionRuntime {
    pub fn accordion(&self) -> Accordion {
        let active = self.state.active_values.read().clone();
        let tab_stop = self.state.current_tab_stop.read().clone();
        self.accordion
            .clone()
            .with_values(active)
            .with_tab_stop(tab_stop)
    }

    pub fn active_values(&self) -> Vec<String> {
        self.state.active_values.read().clone()
    }

    pub fn is_open(&self, value: &str) -> bool {
        self.state.active_values.read().iter().any(|v| v == value)
    }

    pub fn current_tab_stop(&self) -> String {
        self.state.current_tab_stop.read().clone()
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

    pub fn header_with_level(&self, value: &str, disabled: bool, level: u8) -> AccordionHeaderAttributes {
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
        if is_open || is_mounted {
            attrs.aria_controls = Some(self.accordion.relationships().content_id(value));
        } else {
            attrs.aria_controls = None;
        }
        attrs
    }

    pub fn register_content_mounted(&self, value: &str) {
        let mut mounted_sig = self.state.mounted_content_ids;
        if let Ok(mut list) = mounted_sig.try_write() {
            if !list.iter().any(|v| v == value) {
                list.push(value.to_string());
            }
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
        let changed = updated.toggle_item(value);
        if changed {
            let new_values = updated.active_values().to_vec();
            let mut active = self.state.active_values;
            active.set(new_values.clone());

            let mut tab_stop = self.state.current_tab_stop;
            tab_stop.set(value.to_string());

            if let Some(ref callback) = self.on_value_change {
                callback(new_values);
            }
        }
    }

    pub fn trigger_element_id(&self, value: &str) -> String {
        let map = self.state.trigger_ids.read();
        map.iter()
            .find(|(val, _)| val == value)
            .map(|(_, id)| id.clone())
            .unwrap_or_else(|| self.accordion.relationships().trigger_id(value))
    }

    pub fn register_trigger_id(&self, value: &str, id: &str) {
        let mut list = self.state.trigger_ids;
        list.with_mut(|ids| {
            if let Some(existing) = ids.iter_mut().find(|(val, _)| val == value) {
                existing.1 = id.to_string();
            } else {
                ids.push((value.to_string(), id.to_string()));
            }
        });
    }

    pub fn move_tab_stop(&self, value: &str) {
        let mut tab_stop = self.state.current_tab_stop;
        tab_stop.set(value.to_string());

        let target_id = self.trigger_element_id(value);
        focus_element_by_id(&target_id);
    }

    pub fn register_item(&self, value: &str, disabled: bool) {
        let mut list = self.state.registered_items;
        list.with_mut(|items| {
            if let Some(existing) = items.iter_mut().find(|i| i.value == value) {
                existing.disabled = disabled;
            } else {
                items.push(AccordionItemRegistration {
                    value: value.to_string(),
                    disabled,
                });
            }
        });

        // If tab stop is empty, initialize it with first item
        let mut tab_stop = self.state.current_tab_stop;
        if tab_stop.peek().is_empty() {
            tab_stop.set(value.to_string());
        }
    }

    pub fn unregister_item(&self, value: &str) {
        let mut reg_sig = self.state.registered_items;
        if let Ok(mut list) = reg_sig.try_write() {
            list.retain(|i| i.value != value);
        }
        let mut trig_sig = self.state.trigger_ids;
        if let Ok(mut list) = trig_sig.try_write() {
            list.retain(|(v, _)| v != value);
        }
        let mut mounted_sig = self.state.mounted_content_ids;
        if let Ok(mut mounted) = mounted_sig.try_write() {
            mounted.retain(|v| v != value);
        }
        let matches_current = self
            .state
            .current_tab_stop
            .try_peek()
            .map(|curr| *curr == value)
            .unwrap_or(false);
        if matches_current {
            let next_stop = self
                .state
                .registered_items
                .try_peek()
                .ok()
                .and_then(|items| {
                    items
                        .iter()
                        .find(|i| !i.disabled)
                        .or_else(|| items.first())
                        .map(|i| i.value.clone())
                })
                .unwrap_or_default();
            let mut tab_stop_sig = self.state.current_tab_stop;
            if let Ok(mut tab_stop) = tab_stop_sig.try_write() {
                *tab_stop = next_stop;
            }
        }
    }

    pub fn reconcile_order(&self) {
        let root_id = self.accordion.relationships().root_id();
        let trigger_ids = self.state.trigger_ids.peek().clone();
        if trigger_ids.len() <= 1 {
            return;
        }
        let candidate_ids: Vec<&str> = trigger_ids.iter().map(|(_, id)| id.as_str()).collect();
        if let Ok(ordered) = crate::foundation::browser::query_descendant_order(root_id, &candidate_ids) {
            let mut ordered_values = Vec::new();
            for el_id in ordered {
                if let Some((val, _)) = trigger_ids.iter().find(|(_, id)| id == &el_id) {
                    ordered_values.push(val.clone());
                }
            }
            if !ordered_values.is_empty() {
                let mut reg_sig = self.state.registered_items;
                if let Ok(mut items) = reg_sig.try_write() {
                    items.sort_by_key(|item| {
                        ordered_values
                            .iter()
                            .position(|v| v == &item.value)
                            .unwrap_or(usize::MAX)
                    });
                }
            }
        }
    }

    pub fn navigate_key(&self, key: &str) -> Option<String> {
        self.reconcile_order();
        let items = self.state.registered_items.read();
        let current = self.state.current_tab_stop.read().clone();
        let next_item = self.accordion.resolve_key_navigation(&items, &current, key);
        drop(items);

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

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

    pub fn trigger(&self, value: &str, disabled: bool) -> AccordionTriggerAttributes {
        self.accordion().trigger(value, disabled)
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

    pub fn move_tab_stop(&self, value: &str) {
        let mut tab_stop = self.state.current_tab_stop;
        tab_stop.set(value.to_string());

        let target_id = self.accordion.relationships().trigger_id(value);
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
        let mut list = self.state.registered_items;
        list.with_mut(|items| {
            items.retain(|i| i.value != value);
        });
    }

    pub fn navigate_key(&self, key: &str) -> Option<String> {
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

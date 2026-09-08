use std::rc::Rc;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};
pub use crate::foundation::state::{ControllableStateProps, use_controllable_state};

use crate::foundation::browser::focus_element_by_id;

use super::{
    attrs::{TabsContentAttributes, TabsListAttributes, TabsRootAttributes, TabsTriggerAttributes},
    state::Tabs,
    types::{TabsValueChangeHandler, TriggerRegistration},
};

#[derive(Clone)]
pub struct TabsRuntimeState {
    pub active_value: Signal<String>,
    pub current_tab_stop: Signal<String>,
    pub registered_triggers: Signal<Vec<TriggerRegistration>>,
}

#[derive(Clone)]
pub struct TabsRuntime {
    tabs: Tabs,
    on_value_change: Option<TabsValueChangeHandler>,
    state: TabsRuntimeState,
}

pub fn use_tabs_runtime<F>(tabs: Tabs, on_value_change: Option<F>) -> TabsRuntime
where
    F: Fn(String) + 'static,
{
    let synced_value_change: Option<TabsValueChangeHandler> =
        on_value_change.map(|f| Rc::new(f) as TabsValueChangeHandler);
    let initial_active = tabs.active_value().to_owned();
    let initial_tab_stop = tabs.current_tab_stop().to_owned();

    let state = TabsRuntimeState {
        active_value: use_signal(|| initial_active),
        current_tab_stop: use_signal(|| initial_tab_stop),
        registered_triggers: use_signal(Vec::new),
    };

    let effect_state = state.clone();

    use_effect(use_reactive((&tabs,), move |(tabs,)| {
        let mut active = effect_state.active_value;
        if *active.peek() != tabs.active_value() {
            active.set(tabs.active_value().to_owned());
        }
        let mut tab_stop = effect_state.current_tab_stop;
        if *tab_stop.peek() != tabs.current_tab_stop() {
            tab_stop.set(tabs.current_tab_stop().to_owned());
        }
    }));

    TabsRuntime {
        tabs,
        on_value_change: synced_value_change,
        state,
    }
}

impl TabsRuntime {
    pub fn tabs(&self) -> Tabs {
        let mut updated = self.tabs.clone();
        updated.select_tab(self.state.active_value.read().clone());
        let tab_stop = self.state.current_tab_stop.read().clone();
        updated.set_tab_stop(tab_stop);
        updated
    }

    pub fn active_value(&self) -> String {
        self.state.active_value.read().clone()
    }

    pub fn current_tab_stop(&self) -> String {
        self.state.current_tab_stop.read().clone()
    }

    pub fn root(&self) -> TabsRootAttributes {
        self.tabs().root()
    }

    pub fn list(&self) -> TabsListAttributes {
        self.tabs().list()
    }

    pub fn trigger(&self, value: &str, disabled: bool) -> TabsTriggerAttributes {
        self.tabs().trigger(value, disabled)
    }

    pub fn content(&self, value: &str) -> TabsContentAttributes {
        self.tabs().content(value)
    }

    pub fn select_tab(&self, value: &str) {
        let changed = *self.state.active_value.read() != value;
        if changed {
            let mut active = self.state.active_value;
            active.set(value.to_string());
            if let Some(ref callback) = self.on_value_change {
                callback(value.to_string());
            }
        }
        let mut tab_stop = self.state.current_tab_stop;
        tab_stop.set(value.to_string());
    }

    pub fn move_tab_stop(&self, value: &str) {
        let mut tab_stop = self.state.current_tab_stop;
        tab_stop.set(value.to_string());

        let target_id = self.tabs.relationships().trigger_id(value);
        focus_element_by_id(&target_id);

        if self.tabs.activation_mode().is_automatic() {
            self.select_tab(value);
        }
    }

    pub fn register_trigger(&self, value: &str, disabled: bool) {
        let mut list = self.state.registered_triggers;
        list.with_mut(|triggers| {
            if let Some(existing) = triggers.iter_mut().find(|t| t.value == value) {
                existing.disabled = disabled;
            } else {
                triggers.push(TriggerRegistration {
                    value: value.to_string(),
                    disabled,
                });
            }
        });
    }

    pub fn unregister_trigger(&self, value: &str) {
        let mut list = self.state.registered_triggers;
        list.with_mut(|triggers| {
            triggers.retain(|t| t.value != value);
        });
    }

    pub fn navigate_key(&self, key: &str) -> Option<String> {
        let triggers = self.state.registered_triggers.read();
        let current = self.state.current_tab_stop.read().clone();
        let next_tab = self.tabs.resolve_key_navigation(&triggers, &current, key);
        drop(triggers);

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

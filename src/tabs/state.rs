use crate::foundation::{
    shared::{CollectionRegistry, RovingFocusController, RovingFocusKey, ScopeHandle},
    state::DataState,
};

use super::{
    attrs::{TabsContentAttributes, TabsListAttributes, TabsRootAttributes, TabsTriggerAttributes},
    relationships::TabsRelationships,
    types::{
        TABS_PARTS, TabsActivationMode, TabsDirection, TabsOrientation, TabsPart,
        TriggerRegistration,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tabs {
    relationships: TabsRelationships,
    orientation: TabsOrientation,
    dir: TabsDirection,
    activation_mode: TabsActivationMode,
    loop_focus: bool,
    disabled: bool,
    active_value: String,
    current_tab_stop: String,
}

impl Tabs {
    pub fn new(scope: ScopeHandle, initial_value: impl Into<String>) -> Self {
        let initial_value = initial_value.into();
        Self {
            relationships: TabsRelationships::new(scope),
            orientation: TabsOrientation::Horizontal,
            dir: TabsDirection::Ltr,
            activation_mode: TabsActivationMode::Automatic,
            loop_focus: true,
            disabled: false,
            active_value: initial_value.clone(),
            current_tab_stop: initial_value,
        }
    }

    pub const fn parts() -> &'static [TabsPart] {
        &TABS_PARTS
    }

    pub fn relationships(&self) -> &TabsRelationships {
        &self.relationships
    }

    pub fn orientation(&self) -> TabsOrientation {
        self.orientation
    }

    pub fn dir(&self) -> TabsDirection {
        self.dir
    }

    pub fn activation_mode(&self) -> TabsActivationMode {
        self.activation_mode
    }

    pub fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn active_value(&self) -> &str {
        &self.active_value
    }

    pub fn current_tab_stop(&self) -> &str {
        &self.current_tab_stop
    }

    pub fn with_orientation(mut self, orientation: TabsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_direction(mut self, dir: TabsDirection) -> Self {
        self.dir = dir;
        self
    }

    pub fn with_activation_mode(mut self, mode: TabsActivationMode) -> Self {
        self.activation_mode = mode;
        self
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn set_tab_stop(&mut self, value: impl Into<String>) {
        self.current_tab_stop = value.into();
    }

    pub fn select_tab(&mut self, value: impl Into<String>) -> bool {
        let val = value.into();
        let changed = self.active_value != val;
        self.active_value = val.clone();
        self.current_tab_stop = val;
        changed
    }

    pub fn is_navigation_key(key: &str) -> bool {
        matches!(
            key,
            "ArrowLeft"
                | "ArrowRight"
                | "ArrowUp"
                | "ArrowDown"
                | "Home"
                | "End"
                | "PageUp"
                | "PageDown"
        )
    }

    pub fn resolve_key_navigation(
        &self,
        registered_triggers: &[TriggerRegistration],
        current_tab_stop: &str,
        key: &str,
    ) -> Option<String> {
        if self.disabled {
            return None;
        }

        let roving_key = match key {
            "ArrowLeft" => Some(RovingFocusKey::ArrowLeft),
            "ArrowRight" => Some(RovingFocusKey::ArrowRight),
            "ArrowUp" => Some(RovingFocusKey::ArrowUp),
            "ArrowDown" => Some(RovingFocusKey::ArrowDown),
            "Home" | "PageUp" => Some(RovingFocusKey::Home),
            "End" | "PageDown" => Some(RovingFocusKey::End),
            _ => None,
        }?;

        let mut registry = CollectionRegistry::new();
        for trigger in registered_triggers {
            registry.register(trigger.value.clone(), trigger.disabled);
        }

        let controller = RovingFocusController::new(self.dir.into())
            .with_orientation(self.orientation.into())
            .with_looping(self.loop_focus);

        controller.navigate_by_key(
            &registry,
            Some(&current_tab_stop.to_string()),
            roving_key,
            |&disabled| !disabled,
        )
    }

    pub fn root(&self) -> TabsRootAttributes {
        TabsRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_orientation: self.orientation.as_str(),
        }
    }

    pub fn list(&self) -> TabsListAttributes {
        TabsListAttributes {
            id: self.relationships.list_id().to_owned(),
            role: "tablist",
            aria_orientation: self.orientation.as_str(),
            data_orientation: self.orientation.as_str(),
            disabled: self.disabled,
        }
    }

    pub fn trigger(&self, value: &str, trigger_disabled: bool) -> TabsTriggerAttributes {
        let is_selected = self.active_value == value;
        let is_tab_stop = self.current_tab_stop == value;
        let effective_disabled = self.disabled || trigger_disabled;
        let tabindex = if is_tab_stop && !effective_disabled {
            0
        } else {
            -1
        };
        let data_state = if is_selected {
            DataState::Active
        } else {
            DataState::Inactive
        };

        TabsTriggerAttributes {
            id: self.relationships.trigger_id(value),
            role: "tab",
            is_selected,
            aria_selected: if is_selected { "true" } else { "false" },
            aria_controls: self.relationships.content_id(value),
            tabindex,
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
            disabled: effective_disabled,
        }
    }

    pub fn content(&self, value: &str) -> TabsContentAttributes {
        let is_active = self.active_value == value;
        let data_state = if is_active {
            DataState::Active
        } else {
            DataState::Inactive
        };

        TabsContentAttributes {
            id: self.relationships.content_id(value),
            role: "tabpanel",
            aria_labelledby: self.relationships.trigger_id(value),
            tabindex: 0,
            hidden: !is_active,
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
        }
    }
}

use std::rc::Rc;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};
pub use crate::foundation::state::{ControllableStateProps, use_controllable_state};

use crate::foundation::{
    browser::focus_element_by_id,
    shared::{
        CollectionRegistry, Direction, Orientation, RovingFocusController, RovingFocusKey,
        ScopeHandle,
    },
    state::DataState,
};

pub const TABS_PARTS: [TabsPart; 4] = [
    TabsPart::Root,
    TabsPart::List,
    TabsPart::Trigger,
    TabsPart::Content,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TabsPart {
    Root,
    List,
    Trigger,
    Content,
}

impl TabsPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::List => "list",
            Self::Trigger => "trigger",
            Self::Content => "content",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TabsOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl TabsOrientation {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }

    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }
}

impl From<TabsOrientation> for Orientation {
    fn from(orientation: TabsOrientation) -> Self {
        match orientation {
            TabsOrientation::Horizontal => Orientation::Horizontal,
            TabsOrientation::Vertical => Orientation::Vertical,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TabsDirection {
    #[default]
    Ltr,
    Rtl,
}

impl TabsDirection {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }

    pub const fn is_rtl(&self) -> bool {
        matches!(self, Self::Rtl)
    }
}

impl From<TabsDirection> for Direction {
    fn from(direction: TabsDirection) -> Self {
        match direction {
            TabsDirection::Ltr => Direction::Ltr,
            TabsDirection::Rtl => Direction::Rtl,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TabsActivationMode {
    #[default]
    Automatic,
    Manual,
}

impl TabsActivationMode {
    pub const fn is_automatic(&self) -> bool {
        matches!(self, Self::Automatic)
    }

    pub const fn is_manual(&self) -> bool {
        matches!(self, Self::Manual)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsRelationships {
    scope: ScopeHandle,
    root_id: String,
    list_id: String,
}

impl TabsRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            list_id: scope.qualify("list"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn list_id(&self) -> &str {
        &self.list_id
    }

    pub fn trigger_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("trigger-{value}"))
    }

    pub fn content_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("content-{value}"))
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsRootAttributes {
    id: String,
    data_orientation: &'static str,
}

impl TabsRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsListAttributes {
    id: String,
    role: &'static str,
    aria_orientation: &'static str,
    data_orientation: &'static str,
    disabled: bool,
}

impl TabsListAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_orientation(&self) -> &'static str {
        self.aria_orientation
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsTriggerAttributes {
    id: String,
    role: &'static str,
    is_selected: bool,
    aria_selected: &'static str,
    aria_controls: String,
    tabindex: i32,
    data_state: DataState,
    data_orientation: &'static str,
    data_value: String,
    disabled: bool,
}

impl TabsTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn is_selected(&self) -> bool {
        self.is_selected
    }

    pub fn aria_selected(&self) -> &'static str {
        self.aria_selected
    }

    pub fn aria_controls(&self) -> &str {
        &self.aria_controls
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Active => "active",
            _ => "inactive",
        }
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsContentAttributes {
    id: String,
    role: &'static str,
    aria_labelledby: String,
    tabindex: i32,
    hidden: bool,
    data_state: DataState,
    data_orientation: &'static str,
    data_value: String,
}

impl TabsContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_labelledby(&self) -> &str {
        &self.aria_labelledby
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Active => "active",
            _ => "inactive",
        }
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriggerRegistration {
    pub value: String,
    pub disabled: bool,
}

pub type TabsValueChangeHandler = Rc<dyn Fn(String)>;

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
        updated.active_value = self.state.active_value.read().clone();
        updated.current_tab_stop = self.state.current_tab_stop.read().clone();
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

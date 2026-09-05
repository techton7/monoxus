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

pub const ACCORDION_PARTS: [AccordionPart; 5] = [
    AccordionPart::Root,
    AccordionPart::Item,
    AccordionPart::Header,
    AccordionPart::Trigger,
    AccordionPart::Content,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccordionPart {
    Root,
    Item,
    Header,
    Trigger,
    Content,
}

impl AccordionPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Item => "item",
            Self::Header => "header",
            Self::Trigger => "trigger",
            Self::Content => "content",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AccordionOrientation {
    #[default]
    Vertical,
    Horizontal,
}

impl AccordionOrientation {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }

    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }
}

impl From<AccordionOrientation> for Orientation {
    fn from(orientation: AccordionOrientation) -> Self {
        match orientation {
            AccordionOrientation::Vertical => Orientation::Vertical,
            AccordionOrientation::Horizontal => Orientation::Horizontal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AccordionDirection {
    #[default]
    Ltr,
    Rtl,
}

impl AccordionDirection {
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

impl From<AccordionDirection> for Direction {
    fn from(direction: AccordionDirection) -> Self {
        match direction {
            AccordionDirection::Ltr => Direction::Ltr,
            AccordionDirection::Rtl => Direction::Rtl,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccordionMode {
    Single { collapsible: bool },
    Multiple,
}

impl Default for AccordionMode {
    fn default() -> Self {
        Self::Single { collapsible: false }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionRelationships {
    scope: ScopeHandle,
    root_id: String,
}

impl AccordionRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn item_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("item-{value}"))
    }

    pub fn header_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("header-{value}"))
    }

    pub fn trigger_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("trigger-{value}"))
    }

    pub fn content_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("content-{value}"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accordion {
    relationships: AccordionRelationships,
    mode: AccordionMode,
    orientation: AccordionOrientation,
    dir: AccordionDirection,
    loop_focus: bool,
    disabled: bool,
    active_values: Vec<String>,
    current_tab_stop: String,
}

impl Accordion {
    pub fn new(scope: ScopeHandle, mode: AccordionMode) -> Self {
        Self {
            relationships: AccordionRelationships::new(scope),
            mode,
            orientation: AccordionOrientation::Vertical,
            dir: AccordionDirection::Ltr,
            loop_focus: true,
            disabled: false,
            active_values: Vec::new(),
            current_tab_stop: String::new(),
        }
    }

    pub const fn parts() -> &'static [AccordionPart] {
        &ACCORDION_PARTS
    }

    pub fn relationships(&self) -> &AccordionRelationships {
        &self.relationships
    }

    pub fn mode(&self) -> &AccordionMode {
        &self.mode
    }

    pub fn orientation(&self) -> AccordionOrientation {
        self.orientation
    }

    pub fn dir(&self) -> AccordionDirection {
        self.dir
    }

    pub fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn active_values(&self) -> &[String] {
        &self.active_values
    }

    pub fn is_open(&self, value: &str) -> bool {
        self.active_values.iter().any(|v| v == value)
    }

    pub fn current_tab_stop(&self) -> &str {
        &self.current_tab_stop
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        let val = value.into();
        self.active_values = vec![val.clone()];
        if self.current_tab_stop.is_empty() {
            self.current_tab_stop = val;
        }
        self
    }

    pub fn with_values(mut self, values: Vec<String>) -> Self {
        if self.current_tab_stop.is_empty() {
            if let Some(first) = values.first() {
                self.current_tab_stop = first.clone();
            }
        }
        self.active_values = values;
        self
    }

    pub fn with_tab_stop(mut self, tab_stop: impl Into<String>) -> Self {
        self.current_tab_stop = tab_stop.into();
        self
    }

    pub fn with_orientation(mut self, orientation: AccordionOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_direction(mut self, dir: AccordionDirection) -> Self {
        self.dir = dir;
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

    pub fn toggle_item(&mut self, value: &str) -> bool {
        let was_open = self.is_open(value);
        match self.mode {
            AccordionMode::Single { collapsible } => {
                if was_open {
                    if collapsible {
                        self.active_values.clear();
                        true
                    } else {
                        // In single non-collapsible mode, open item cannot be collapsed
                        false
                    }
                } else {
                    self.active_values = vec![value.to_string()];
                    self.current_tab_stop = value.to_string();
                    true
                }
            }
            AccordionMode::Multiple => {
                if was_open {
                    self.active_values.retain(|v| v != value);
                } else {
                    self.active_values.push(value.to_string());
                    self.current_tab_stop = value.to_string();
                }
                true
            }
        }
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
        registered_items: &[AccordionItemRegistration],
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
        for item in registered_items {
            registry.register(item.value.clone(), item.disabled);
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

    pub fn root(&self) -> AccordionRootAttributes {
        AccordionRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_orientation: self.orientation.as_str(),
            disabled: self.disabled,
        }
    }

    pub fn item(&self, value: &str, item_disabled: bool) -> AccordionItemAttributes {
        let is_open = self.is_open(value);
        let effective_disabled = self.disabled || item_disabled;
        let data_state = if is_open {
            DataState::Open
        } else {
            DataState::Closed
        };

        AccordionItemAttributes {
            id: self.relationships.item_id(value),
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
            disabled: effective_disabled,
        }
    }

    pub fn header(&self, value: &str, item_disabled: bool) -> AccordionHeaderAttributes {
        let is_open = self.is_open(value);
        let effective_disabled = self.disabled || item_disabled;
        let data_state = if is_open {
            DataState::Open
        } else {
            DataState::Closed
        };

        AccordionHeaderAttributes {
            id: self.relationships.header_id(value),
            role: "heading",
            aria_level: 3,
            data_heading_level: 3,
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
            disabled: effective_disabled,
        }
    }

    pub fn trigger(&self, value: &str, trigger_disabled: bool) -> AccordionTriggerAttributes {
        let is_open = self.is_open(value);
        let is_tab_stop = self.current_tab_stop == value;
        let effective_disabled = self.disabled || trigger_disabled;
        let tabindex = if is_tab_stop && !effective_disabled {
            0
        } else {
            -1
        };
        let data_state = if is_open {
            DataState::Open
        } else {
            DataState::Closed
        };

        let aria_disabled = if !effective_disabled
            && is_open
            && matches!(self.mode, AccordionMode::Single { collapsible: false })
        {
            Some("true")
        } else {
            None
        };

        AccordionTriggerAttributes {
            id: self.relationships.trigger_id(value),
            role: "button",
            is_open,
            aria_expanded: if is_open { "true" } else { "false" },
            aria_controls: self.relationships.content_id(value),
            aria_disabled,
            tabindex,
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
            disabled: effective_disabled,
        }
    }

    pub fn content(&self, value: &str) -> AccordionContentAttributes {
        let is_open = self.is_open(value);
        let data_state = if is_open {
            DataState::Open
        } else {
            DataState::Closed
        };

        AccordionContentAttributes {
            id: self.relationships.content_id(value),
            role: "region",
            aria_labelledby: self.relationships.trigger_id(value),
            hidden: !is_open,
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionRootAttributes {
    id: String,
    data_orientation: &'static str,
    disabled: bool,
}

impl AccordionRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionItemAttributes {
    id: String,
    data_state: DataState,
    data_orientation: &'static str,
    data_value: String,
    disabled: bool,
}

impl AccordionItemAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
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
pub struct AccordionHeaderAttributes {
    id: String,
    role: &'static str,
    aria_level: u32,
    data_heading_level: u32,
    data_state: DataState,
    data_orientation: &'static str,
    data_value: String,
    disabled: bool,
}

impl AccordionHeaderAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_level(&self) -> u32 {
        self.aria_level
    }

    pub fn data_heading_level(&self) -> u32 {
        self.data_heading_level
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
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
pub struct AccordionTriggerAttributes {
    id: String,
    role: &'static str,
    is_open: bool,
    aria_expanded: &'static str,
    aria_controls: String,
    aria_disabled: Option<&'static str>,
    tabindex: i32,
    data_state: DataState,
    data_orientation: &'static str,
    data_value: String,
    disabled: bool,
}

impl AccordionTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
    }

    pub fn aria_controls(&self) -> &str {
        &self.aria_controls
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
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
pub struct AccordionContentAttributes {
    id: String,
    role: &'static str,
    aria_labelledby: String,
    hidden: bool,
    data_state: DataState,
    data_orientation: &'static str,
    data_value: String,
}

impl AccordionContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_labelledby(&self) -> &str {
        &self.aria_labelledby
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
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
pub struct AccordionItemRegistration {
    pub value: String,
    pub disabled: bool,
}

pub type AccordionValueChangeHandler = Rc<dyn Fn(Vec<String>)>;

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

pub fn use_accordion_runtime<F>(accordion: Accordion, on_value_change: Option<F>) -> AccordionRuntime
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
        let mut updated = self.accordion.clone();
        updated.active_values = self.state.active_values.read().clone();
        updated.current_tab_stop = self.state.current_tab_stop.read().clone();
        updated
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

// -------------------------------------------------------------------------
// Dioxus Context & Declarative Components
// -------------------------------------------------------------------------

#[derive(Clone)]
pub struct AccordionContext {
    pub runtime: AccordionRuntime,
}

#[component]
pub fn AccordionRoot(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] mode: AccordionMode,
    #[props(default)] orientation: AccordionOrientation,
    #[props(default)] dir: AccordionDirection,
    #[props(default = true)] loop_focus: bool,
    #[props(default = false)] disabled: bool,
    #[props(default)] default_value: Option<String>,
    #[props(default)] default_values: Option<Vec<String>>,
    #[props(default)] on_value_change: Option<EventHandler<Vec<String>>>,
    children: Element,
) -> Element {
    let scope_id = id.clone().unwrap_or_else(|| "accordion".to_string());
    let scope = ScopeHandle::root("accordion").child(scope_id);

    let mut accordion = Accordion::new(scope, mode)
        .with_orientation(orientation)
        .with_direction(dir)
        .with_loop_focus(loop_focus)
        .with_disabled(disabled);

    if let Some(vals) = default_values {
        accordion = accordion.with_values(vals);
    } else if let Some(val) = default_value {
        accordion = accordion.with_value(val);
    }

    let change_handler = on_value_change.map(|handler| move |vals| handler.call(vals));
    let runtime = use_accordion_runtime(accordion, change_handler);

    use_context_provider(|| AccordionContext {
        runtime: runtime.clone(),
    });

    let attrs = runtime.root();

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            {children}
        }
    }
}

#[derive(Clone)]
pub struct AccordionItemContext {
    pub value: String,
    pub disabled: bool,
}

#[component]
pub fn AccordionItem(
    value: String,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default = false)] disabled: bool,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_value = value.clone();

    use_context_provider(|| AccordionItemContext {
        value: item_value.clone(),
        disabled,
    });

    use_effect(use_reactive((&item_value, &disabled), {
        let runtime = ctx.runtime.clone();
        move |(val, dis)| {
            runtime.register_item(&val, dis);
        }
    }));

    let attrs = ctx.runtime.item(&value, disabled);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            {children}
        }
    }
}

#[component]
pub fn AccordionHeader(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let attrs = ctx.runtime.header(&item_ctx.value, item_ctx.disabled);

    rsx! {
        h3 {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            role: "{attrs.role()}",
            "aria-level": "{attrs.aria_level()}",
            "data-heading-level": "{attrs.data_heading_level()}",
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            {children}
        }
    }
}

#[component]
pub fn AccordionTrigger(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let attrs = ctx.runtime.trigger(&item_ctx.value, item_ctx.disabled);

    let click_runtime = ctx.runtime.clone();
    let click_val = item_ctx.value.clone();

    let key_runtime = ctx.runtime.clone();

    rsx! {
        button {
            r#type: "button",
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "aria-expanded": "{attrs.aria_expanded()}",
            "aria-controls": "{attrs.aria_controls()}",
            "aria-disabled": attrs.aria_disabled().unwrap_or_default(),
            tabindex: attrs.tabindex(),
            disabled: attrs.is_disabled(),
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            onclick: move |_| {
                click_runtime.toggle_item(&click_val);
            },
            onkeydown: move |evt: KeyboardEvent| {
                let key = evt.key();
                let key_str = key.to_string();
                if Accordion::is_navigation_key(&key_str) {
                    evt.prevent_default();
                    key_runtime.navigate_key(&key_str);
                }
            },
            {children}
        }
    }
}

#[component]
pub fn AccordionContent(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let attrs = ctx.runtime.content(&item_ctx.value);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            role: "{attrs.role()}",
            "aria-labelledby": "{attrs.aria_labelledby()}",
            hidden: attrs.is_hidden(),
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            {children}
        }
    }
}

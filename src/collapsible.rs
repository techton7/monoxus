use std::rc::Rc;

use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child,
};
pub use crate::foundation::state::{ControllableStateProps, use_controllable_state};

use crate::foundation::{shared::ScopeHandle, state::DataState};

pub const COLLAPSIBLE_PARTS: [CollapsiblePart; 3] = [
    CollapsiblePart::Root,
    CollapsiblePart::Trigger,
    CollapsiblePart::Content,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollapsiblePart {
    Root,
    Trigger,
    Content,
}

impl CollapsiblePart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Content => "content",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    content_id: String,
}

impl CollapsibleRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            content_id: scope.qualify("content"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn trigger_id(&self) -> &str {
        &self.trigger_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collapsible {
    relationships: CollapsibleRelationships,
    open: bool,
    disabled: bool,
}

impl Collapsible {
    pub fn new(scope: ScopeHandle, initial_open: bool) -> Self {
        Self {
            relationships: CollapsibleRelationships::new(scope),
            open: initial_open,
            disabled: false,
        }
    }

    pub const fn parts() -> &'static [CollapsiblePart] {
        &COLLAPSIBLE_PARTS
    }

    pub fn relationships(&self) -> &CollapsibleRelationships {
        &self.relationships
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn set_open(&mut self, open: bool) -> bool {
        let changed = self.open != open;
        self.open = open;
        changed
    }

    pub fn toggle(&mut self) -> bool {
        self.open = !self.open;
        true
    }

    pub fn root(&self) -> CollapsibleRootAttributes {
        let data_state = if self.open {
            DataState::Open
        } else {
            DataState::Closed
        };

        CollapsibleRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_state,
            disabled: self.disabled,
        }
    }

    pub fn trigger(&self) -> CollapsibleTriggerAttributes {
        let data_state = if self.open {
            DataState::Open
        } else {
            DataState::Closed
        };

        CollapsibleTriggerAttributes {
            id: self.relationships.trigger_id().to_owned(),
            aria_controls: self.relationships.content_id().to_owned(),
            aria_expanded: if self.open { "true" } else { "false" },
            data_state,
            disabled: self.disabled,
        }
    }

    pub fn content(&self) -> CollapsibleContentAttributes {
        let data_state = if self.open {
            DataState::Open
        } else {
            DataState::Closed
        };

        CollapsibleContentAttributes {
            id: self.relationships.content_id().to_owned(),
            hidden: !self.open,
            data_state,
            disabled: self.disabled,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleRootAttributes {
    id: String,
    data_state: DataState,
    disabled: bool,
}

impl CollapsibleRootAttributes {
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

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleTriggerAttributes {
    id: String,
    aria_controls: String,
    aria_expanded: &'static str,
    data_state: DataState,
    disabled: bool,
}

impl CollapsibleTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn aria_controls(&self) -> &str {
        &self.aria_controls
    }

    pub fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
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

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleContentAttributes {
    id: String,
    hidden: bool,
    data_state: DataState,
    disabled: bool,
}

impl CollapsibleContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
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

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

pub type CollapsibleOpenChangeHandler = Rc<dyn Fn(bool)>;

#[derive(Clone)]
pub struct CollapsibleRuntimeState {
    pub open: Signal<bool>,
}

#[derive(Clone)]
pub struct CollapsibleRuntime {
    collapsible: Collapsible,
    on_open_change: Option<CollapsibleOpenChangeHandler>,
    state: CollapsibleRuntimeState,
}

pub fn use_collapsible_runtime<F>(
    collapsible: Collapsible,
    on_open_change: Option<F>,
) -> CollapsibleRuntime
where
    F: Fn(bool) + 'static,
{
    let synced_change: Option<CollapsibleOpenChangeHandler> =
        on_open_change.map(|f| Rc::new(f) as CollapsibleOpenChangeHandler);
    let initial_open = collapsible.is_open();

    let state = CollapsibleRuntimeState {
        open: use_signal(|| initial_open),
    };

    let effect_state = state.clone();

    use_effect(use_reactive((&collapsible,), move |(collapsible,)| {
        let mut open_sig = effect_state.open;
        if *open_sig.peek() != collapsible.is_open() {
            open_sig.set(collapsible.is_open());
        }
    }));

    CollapsibleRuntime {
        collapsible,
        on_open_change: synced_change,
        state,
    }
}

impl CollapsibleRuntime {
    pub fn collapsible(&self) -> Collapsible {
        let mut updated = self.collapsible.clone();
        updated.open = *self.state.open.read();
        updated
    }

    pub fn is_open(&self) -> bool {
        *self.state.open.read()
    }

    pub fn is_disabled(&self) -> bool {
        self.collapsible.is_disabled()
    }

    pub fn root(&self) -> CollapsibleRootAttributes {
        self.collapsible().root()
    }

    pub fn trigger(&self) -> CollapsibleTriggerAttributes {
        self.collapsible().trigger()
    }

    pub fn content(&self) -> CollapsibleContentAttributes {
        self.collapsible().content()
    }

    pub fn set_open(&self, open: bool) {
        if self.collapsible.is_disabled() {
            return;
        }
        let changed = *self.state.open.read() != open;
        if changed {
            let mut open_sig = self.state.open;
            open_sig.set(open);
            if let Some(ref callback) = self.on_open_change {
                callback(open);
            }
        }
    }

    pub fn toggle(&self) {
        let current = self.is_open();
        self.set_open(!current);
    }
}

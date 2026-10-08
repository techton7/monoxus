use crate::foundation::state::DataState;

use super::types::CollapsibleStateRequest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleRootAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
    pub(crate) disabled: bool,
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
    pub(crate) id: String,
    pub(crate) aria_controls: String,
    pub(crate) aria_expanded: &'static str,
    pub(crate) data_state: DataState,
    pub(crate) disabled: bool,
    pub(crate) state_request: CollapsibleStateRequest,
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

    pub const fn state_request(&self) -> CollapsibleStateRequest {
        self.state_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleContentAttributes {
    pub(crate) id: String,
    pub(crate) hidden: bool,
    pub(crate) data_state: DataState,
    pub(crate) disabled: bool,
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

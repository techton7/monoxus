use crate::foundation::{shared::data_disabled_attr, state::DataState};

use super::types::CollapsibleStateRequest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleRootAttributes {
    pub id: String,
    pub data_state: DataState,
    pub disabled: bool,
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

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleTriggerAttributes {
    pub id: String,
    pub content_id: String,
    pub aria_controls: Option<String>,
    pub aria_expanded: &'static str,
    pub data_state: DataState,
    pub disabled: bool,
    pub state_request: CollapsibleStateRequest,
}

impl CollapsibleTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn target_content_id(&self) -> &str {
        &self.content_id
    }

    pub fn aria_controls(&self) -> Option<&str> {
        self.aria_controls.as_deref()
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

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }

    pub const fn state_request(&self) -> CollapsibleStateRequest {
        self.state_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleContentAttributes {
    pub id: String,
    pub hidden: bool,
    pub data_state: DataState,
    pub disabled: bool,
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

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

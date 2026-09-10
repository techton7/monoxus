use super::types::PopoverStateRequest;
use crate::foundation::{overlay::PortalHost, state::DataState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverRootAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
}

impl PopoverRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverTriggerAttributes {
    pub(crate) id: String,
    pub(crate) aria_controls: String,
    pub(crate) aria_expanded: bool,
    pub(crate) data_state: DataState,
    pub(crate) open_request: PopoverStateRequest,
}

impl PopoverTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn aria_controls(&self) -> &str {
        &self.aria_controls
    }

    pub const fn aria_expanded(&self) -> bool {
        self.aria_expanded
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn open_request(&self) -> PopoverStateRequest {
        self.open_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverAnchorAttributes {
    pub(crate) id: String,
}

impl PopoverAnchorAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverPortalAttributes {
    pub(crate) host: PortalHost,
}

impl PopoverPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverContentAttributes {
    pub(crate) id: String,
    pub(crate) role: &'static str,
    pub(crate) aria_modal: bool,
    pub(crate) data_state: DataState,
    pub(crate) data_side: &'static str,
    pub(crate) data_align: &'static str,
}

impl PopoverContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub const fn aria_modal(&self) -> bool {
        self.aria_modal
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_side(&self) -> &'static str {
        self.data_side
    }

    pub const fn data_align(&self) -> &'static str {
        self.data_align
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverArrowAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
    pub(crate) data_side: &'static str,
    pub(crate) data_align: &'static str,
}

impl PopoverArrowAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_side(&self) -> &'static str {
        self.data_side
    }

    pub const fn data_align(&self) -> &'static str {
        self.data_align
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverCloseAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
    pub(crate) close_request: PopoverStateRequest,
}

impl PopoverCloseAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn close_request(&self) -> PopoverStateRequest {
        self.close_request
    }
}

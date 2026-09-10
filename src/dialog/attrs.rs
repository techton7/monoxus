use super::types::DialogStateRequest;
use crate::foundation::{overlay::PortalHost, state::DataState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogRootAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
}

impl DialogRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogTriggerAttributes {
    pub(crate) id: String,
    pub(crate) aria_controls: String,
    pub(crate) aria_expanded: bool,
    pub(crate) data_state: DataState,
    pub(crate) open_request: DialogStateRequest,
}

impl DialogTriggerAttributes {
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

    pub const fn open_request(&self) -> DialogStateRequest {
        self.open_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogPortalAttributes {
    pub(crate) host: PortalHost,
}

impl DialogPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogOverlayAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
}

impl DialogOverlayAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogContentAttributes {
    pub(crate) id: String,
    pub(crate) role: &'static str,
    pub(crate) aria_modal: bool,
    pub(crate) aria_labelledby: String,
    pub(crate) aria_describedby: String,
    pub(crate) data_state: DataState,
}

impl DialogContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub const fn aria_modal(&self) -> bool {
        self.aria_modal
    }

    pub fn aria_labelledby(&self) -> &str {
        &self.aria_labelledby
    }

    pub fn aria_describedby(&self) -> &str {
        &self.aria_describedby
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogTitleAttributes {
    pub(crate) id: String,
}

impl DialogTitleAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogDescriptionAttributes {
    pub(crate) id: String,
}

impl DialogDescriptionAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogCloseAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
    pub(crate) close_request: DialogStateRequest,
}

impl DialogCloseAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn close_request(&self) -> DialogStateRequest {
        self.close_request
    }
}

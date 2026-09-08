use crate::foundation::{overlay::PortalHost, state::DataState};
use super::types::TooltipStateRequest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipRootAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
}

impl TooltipRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipTriggerAttributes {
    pub(crate) id: String,
    pub(crate) aria_describedby: Option<String>,
    pub(crate) provider_id: Option<String>,
    pub(crate) data_state: DataState,
    pub(crate) open_request: TooltipStateRequest,
    pub(crate) close_request: TooltipStateRequest,
}

impl TooltipTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn aria_describedby(&self) -> Option<&str> {
        self.aria_describedby.as_deref()
    }

    pub fn provider_id(&self) -> Option<&str> {
        self.provider_id.as_deref()
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn open_request(&self) -> TooltipStateRequest {
        self.open_request
    }

    pub const fn close_request(&self) -> TooltipStateRequest {
        self.close_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipPortalAttributes {
    pub(crate) host: PortalHost,
}

impl TooltipPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipContentAttributes {
    pub(crate) id: String,
    pub(crate) role: &'static str,
    pub(crate) data_state: DataState,
    pub(crate) data_side: &'static str,
    pub(crate) data_align: &'static str,
    pub(crate) autofocus_suppressed: bool,
}

impl TooltipContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
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

    pub const fn autofocus_suppressed(&self) -> bool {
        self.autofocus_suppressed
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipArrowAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
    pub(crate) data_side: &'static str,
    pub(crate) data_align: &'static str,
}

impl TooltipArrowAttributes {
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

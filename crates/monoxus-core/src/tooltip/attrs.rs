use super::types::TooltipStateRequest;
use crate::foundation::{overlay::PortalHost, state::DataState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipRootAttributes {
    pub id: String,
    pub data_state: DataState,
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
    pub id: String,
    pub aria_describedby: Option<String>,
    pub provider_id: Option<String>,
    pub disabled: bool,
    pub data_state: DataState,
    pub open_request: TooltipStateRequest,
    pub close_request: TooltipStateRequest,
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

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        crate::foundation::shared::data_disabled_attr(self.disabled)
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
    pub host: PortalHost,
}

impl TooltipPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooltipContentAttributes {
    pub id: String,
    pub wrapper_id: String,
    pub role: &'static str,
    pub aria_label: Option<String>,
    pub data_state: DataState,
    pub data_side: &'static str,
    pub data_align: &'static str,
    pub autofocus_suppressed: bool,
}

impl TooltipContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn wrapper_id(&self) -> &str {
        &self.wrapper_id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub fn with_aria_label(mut self, aria_label: Option<String>) -> Self {
        self.aria_label = aria_label;
        self
    }

    pub fn aria_label(&self) -> Option<&str> {
        self.aria_label.as_deref()
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
    pub id: String,
    pub data_state: DataState,
    pub data_side: &'static str,
    pub data_align: &'static str,
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

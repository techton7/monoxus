use super::types::DialogStateRequest;
use crate::foundation::{overlay::PortalHost, shared::data_disabled_attr, state::DataState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogRootAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
}

impl DialogRootAttributes {
    pub fn new(id: impl Into<String>, data_state: DataState) -> Self {
        Self {
            id: id.into(),
            data_state,
        }
    }

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
    pub(crate) target_content_id: String,
    pub(crate) aria_controls: Option<String>,
    pub(crate) aria_expanded: bool,
    pub(crate) disabled: bool,
    pub(crate) data_state: DataState,
    pub(crate) open_request: DialogStateRequest,
}

impl DialogTriggerAttributes {
    pub fn new(
        id: impl Into<String>,
        target_content_id: impl Into<String>,
        aria_controls: Option<String>,
        aria_expanded: bool,
        disabled: bool,
        data_state: DataState,
        open_request: DialogStateRequest,
    ) -> Self {
        Self {
            id: id.into(),
            target_content_id: target_content_id.into(),
            aria_controls,
            aria_expanded,
            disabled,
            data_state,
            open_request,
        }
    }

    pub fn with_target_content_id(mut self, target_content_id: impl Into<String>) -> Self {
        self.target_content_id = target_content_id.into();
        self
    }

    pub fn set_target_content_id(&mut self, target_content_id: impl Into<String>) {
        self.target_content_id = target_content_id.into();
    }

    pub fn with_aria_controls(mut self, aria_controls: Option<String>) -> Self {
        self.aria_controls = aria_controls;
        self
    }

    pub fn set_aria_controls(&mut self, aria_controls: Option<String>) {
        self.aria_controls = aria_controls;
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn target_content_id(&self) -> &str {
        &self.target_content_id
    }

    pub fn aria_controls(&self) -> Option<&str> {
        self.aria_controls.as_deref()
    }

    pub const fn aria_expanded(&self) -> bool {
        self.aria_expanded
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
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
    pub fn new(host: PortalHost) -> Self {
        Self { host }
    }

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
    pub fn new(id: impl Into<String>, data_state: DataState) -> Self {
        Self {
            id: id.into(),
            data_state,
        }
    }

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
    pub fn new(
        id: impl Into<String>,
        role: &'static str,
        aria_modal: bool,
        aria_labelledby: impl Into<String>,
        aria_describedby: impl Into<String>,
        data_state: DataState,
    ) -> Self {
        Self {
            id: id.into(),
            role,
            aria_modal,
            aria_labelledby: aria_labelledby.into(),
            aria_describedby: aria_describedby.into(),
            data_state,
        }
    }

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
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogDescriptionAttributes {
    pub(crate) id: String,
}

impl DialogDescriptionAttributes {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogCloseAttributes {
    pub(crate) id: String,
    pub(crate) disabled: bool,
    pub(crate) data_state: DataState,
    pub(crate) close_request: DialogStateRequest,
}

impl DialogCloseAttributes {
    pub fn new(
        id: impl Into<String>,
        disabled: bool,
        data_state: DataState,
        close_request: DialogStateRequest,
    ) -> Self {
        Self {
            id: id.into(),
            disabled,
            data_state,
            close_request,
        }
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn close_request(&self) -> DialogStateRequest {
        self.close_request
    }
}

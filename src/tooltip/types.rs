use crate::foundation::state::DataState;

pub const TOOLTIP_GEOMETRY_NAMESPACE: &str = "tooltip";
pub(crate) const TOOLTIP_HOVER_TRANSFER_GRACE_MS: u64 = 40;

pub const TOOLTIP_PARTS: [TooltipPart; 6] = [
    TooltipPart::Root,
    TooltipPart::Trigger,
    TooltipPart::Portal,
    TooltipPart::Content,
    TooltipPart::Arrow,
    TooltipPart::Provider,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TooltipPart {
    Root,
    Trigger,
    Portal,
    Content,
    Arrow,
    Provider,
}

impl TooltipPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Portal => "portal",
            Self::Content => "content",
            Self::Arrow => "arrow",
            Self::Provider => "provider",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TooltipStateRequest {
    Open,
    Close,
}

impl TooltipStateRequest {
    pub const fn next_open(self) -> bool {
        matches!(self, Self::Open)
    }

    pub fn data_state(self) -> DataState {
        if self.next_open() {
            DataState::Open
        } else {
            DataState::Closed
        }
    }
}

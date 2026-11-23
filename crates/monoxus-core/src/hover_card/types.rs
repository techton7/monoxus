use crate::foundation::state::DataState;

pub const HOVER_CARD_GEOMETRY_NAMESPACE: &str = "hover-card";
pub const HOVER_CARD_RADIX_COMPATIBILITY_PREFIX: &str = "radix-hover-card";
pub const HOVER_CARD_RADIX_ANCHOR_LABEL: &str = "trigger";
pub const HOVER_CARD_DEFAULT_OPEN_DELAY_MS: u64 = 700;
pub const HOVER_CARD_DEFAULT_CLOSE_DELAY_MS: u64 = 300;
pub const HOVER_CARD_DEFAULT_SIDE_OFFSET: f32 = 4.0;

pub const HOVER_CARD_PARTS: [HoverCardPart; 6] = [
    HoverCardPart::Root,
    HoverCardPart::Trigger,
    HoverCardPart::Portal,
    HoverCardPart::Content,
    HoverCardPart::ContentStatic,
    HoverCardPart::Arrow,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HoverCardPart {
    Root,
    Trigger,
    Portal,
    Content,
    ContentStatic,
    Arrow,
}

impl HoverCardPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Portal => "portal",
            Self::Content => "content",
            Self::ContentStatic => "content-static",
            Self::Arrow => "arrow",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HoverCardOutsideBehavior {
    #[default]
    Close,
    Ignore,
}

impl HoverCardOutsideBehavior {
    pub const fn should_close(self) -> bool {
        matches!(self, Self::Close)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::Ignore => "ignore",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HoverCardStateRequest {
    Open,
    Close,
}

impl HoverCardStateRequest {
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

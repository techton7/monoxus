use std::rc::Rc;

use crate::foundation::state::DataState;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollapsibleStateRequest {
    Open,
    Close,
    Toggle,
}

impl CollapsibleStateRequest {
    pub const fn next_open(self, current_open: bool) -> bool {
        match self {
            Self::Open => true,
            Self::Close => false,
            Self::Toggle => !current_open,
        }
    }

    pub fn data_state(self, current_open: bool) -> DataState {
        if self.next_open(current_open) {
            DataState::Open
        } else {
            DataState::Closed
        }
    }
}

pub type CollapsibleOpenChangeHandler = Rc<dyn Fn(bool)>;

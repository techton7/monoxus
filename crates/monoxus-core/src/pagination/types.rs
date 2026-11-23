pub use crate::foundation::shared::{Direction, Orientation};

pub const PAGINATION_PARTS: [PaginationPart; 4] = [
    PaginationPart::Root,
    PaginationPart::PrevButton,
    PaginationPart::NextButton,
    PaginationPart::Page,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaginationPart {
    Root,
    PrevButton,
    NextButton,
    Page,
}

impl PaginationPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::PrevButton => "prev-button",
            Self::NextButton => "next-button",
            Self::Page => "page",
        }
    }
}

pub const fn orientation_as_str(orientation: Orientation) -> &'static str {
    match orientation {
        Orientation::Vertical => "vertical",
        Orientation::Horizontal | Orientation::Both => "horizontal",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PageData {
    pub value: usize,
}

impl PageData {
    pub const fn new(value: usize) -> Self {
        Self { value }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PageItem {
    Page { value: usize, key: String },
    Ellipsis { key: String },
}

impl PageItem {
    pub fn key(&self) -> &str {
        match self {
            Self::Page { key, .. } | Self::Ellipsis { key } => key,
        }
    }

    pub const fn is_page(&self) -> bool {
        matches!(self, Self::Page { .. })
    }

    pub const fn is_ellipsis(&self) -> bool {
        matches!(self, Self::Ellipsis { .. })
    }

    pub const fn page_value(&self) -> Option<usize> {
        match self {
            Self::Page { value, .. } => Some(*value),
            Self::Ellipsis { .. } => None,
        }
    }

    pub const fn as_page_data(&self) -> Option<PageData> {
        match self {
            Self::Page { value, .. } => Some(PageData { value: *value }),
            Self::Ellipsis { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PageRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationSnapshot {
    pub pages: Vec<PageItem>,
    pub range: PageRange,
    pub current_page: usize,
    pub total_pages: usize,
    pub has_prev_page: bool,
    pub has_next_page: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationItemRegistration {
    pub key: String,
    pub id: String,
    pub disabled: bool,
}

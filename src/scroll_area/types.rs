pub const SCROLL_AREA_MIN_THUMB_SIZE: f64 = 18.0;
pub const DEFAULT_SCROLL_HIDE_DELAY_MS: u64 = 600;

pub const SCROLL_AREA_PARTS: [ScrollAreaPart; 6] = [
    ScrollAreaPart::Root,
    ScrollAreaPart::Viewport,
    ScrollAreaPart::Content,
    ScrollAreaPart::Scrollbar,
    ScrollAreaPart::Thumb,
    ScrollAreaPart::Corner,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScrollAreaPart {
    Root,
    Viewport,
    Content,
    Scrollbar,
    Thumb,
    Corner,
}

impl ScrollAreaPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Viewport => "viewport",
            Self::Content => "content",
            Self::Scrollbar => "scrollbar",
            Self::Thumb => "thumb",
            Self::Corner => "corner",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrollAreaType {
    #[default]
    Hover,
    Scroll,
    Auto,
    Always,
}

impl ScrollAreaType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Hover => "hover",
            Self::Scroll => "scroll",
            Self::Auto => "auto",
            Self::Always => "always",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrollAreaVisibilityState {
    #[default]
    Hidden,
    Showing,
    Scrolling,
    Interacting,
    Idle,
    Visible,
}

pub type ScrollbarVisibilityState = ScrollAreaVisibilityState;

impl ScrollAreaVisibilityState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Hidden => "hidden",
            Self::Showing => "showing",
            Self::Scrolling => "scrolling",
            Self::Interacting => "interacting",
            Self::Idle => "idle",
            Self::Visible => "visible",
        }
    }

    pub const fn is_visible(&self) -> bool {
        !matches!(self, Self::Hidden)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScrollAreaStateEvent {
    Scroll,
    ScrollEnd,
    PointerEnter,
    PointerLeave,
    Hide,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollbarPadding {
    pub start: f64,
    pub end: f64,
}

impl ScrollbarPadding {
    pub const fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    pub const fn total(&self) -> f64 {
        self.start + self.end
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollAreaSizes {
    pub content: f64,
    pub viewport: f64,
    pub scrollbar_size: f64,
    pub padding: ScrollbarPadding,
}

impl ScrollAreaSizes {
    pub const fn new(
        content: f64,
        viewport: f64,
        scrollbar_size: f64,
        padding: ScrollbarPadding,
    ) -> Self {
        Self {
            content,
            viewport,
            scrollbar_size,
            padding,
        }
    }

    pub fn with_content(mut self, content: f64) -> Self {
        self.content = content;
        self
    }

    pub fn with_viewport(mut self, viewport: f64) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn with_scrollbar_size(mut self, scrollbar_size: f64) -> Self {
        self.scrollbar_size = scrollbar_size;
        self
    }

    pub fn with_padding(mut self, padding: ScrollbarPadding) -> Self {
        self.padding = padding;
        self
    }
}

pub use crate::foundation::shared::Direction as ScrollAreaDirection;
pub use crate::foundation::shared::Orientation as ScrollAreaOrientation;
pub use crate::foundation::shared::{Direction, Orientation};

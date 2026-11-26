pub mod corner;
pub mod root;
pub mod scrollbar;
pub mod thumb;
pub mod viewport;

pub use self::{
    corner::{ScrollAreaCorner, ScrollAreaCornerProps},
    root::{ScrollArea, ScrollAreaRoot, ScrollAreaRootProps},
    scrollbar::{ScrollAreaScrollbar, ScrollAreaScrollbarProps},
    thumb::{ScrollAreaThumb, ScrollAreaThumbProps},
    viewport::{
        ScrollAreaContent, ScrollAreaContentProps, ScrollAreaViewport, ScrollAreaViewportProps,
    },
};

use dioxus::prelude::*;

use crate::foundation::{
    shared::{Direction, Orientation},
};

use super::{
    relationships::ScrollAreaRelationships,
    runtime::ScrollAreaRuntime,
    types::{ScrollAreaSizes, ScrollAreaType},
};

#[derive(Clone, Copy)]
pub struct ScrollAreaContext {
    pub runtime: ScrollAreaRuntime,
    pub relationships: Signal<ScrollAreaRelationships>,
    pub r#type: Signal<ScrollAreaType>,
    pub dir: Signal<Direction>,
    pub scroll_hide_delay: Signal<u64>,
}

impl ScrollAreaContext {
    pub fn scroll_type(&self) -> ScrollAreaType {
        (self.r#type)()
    }

    pub fn dir(&self) -> Direction {
        (self.dir)()
    }

    pub fn scroll_hide_delay(&self) -> u64 {
        (self.scroll_hide_delay)()
    }
}

#[derive(Clone, Copy)]
pub struct ScrollbarContext {
    pub orientation: Orientation,
    pub scrollbar_id: Signal<String>,
    pub thumb_id: Signal<String>,
    pub sizes: Signal<ScrollAreaSizes>,
    pub pointer_offset: Signal<f64>,
    pub is_dragging: Signal<bool>,
    pub thumb_pointer_active: Signal<bool>,
    pub force_mount: bool,
}

impl ScrollbarContext {
    pub fn sizes(&self) -> ScrollAreaSizes {
        (self.sizes)()
    }

    pub fn pointer_offset(&self) -> f64 {
        (self.pointer_offset)()
    }

    pub fn is_dragging(&self) -> bool {
        (self.is_dragging)()
    }

    pub fn thumb_pointer_active(&self) -> bool {
        (self.thumb_pointer_active)()
    }
}


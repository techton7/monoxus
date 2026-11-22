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
    compose::{Slottable, project_as_child},
    shared::{Direction, Orientation},
};

use super::{
    attrs::{
        ScrollAreaContentAttributes, ScrollAreaCornerAttributes, ScrollAreaRootAttributes,
        ScrollAreaScrollbarAttributes, ScrollAreaThumbAttributes, ScrollAreaViewportAttributes,
    },
    relationships::ScrollAreaRelationships,
    runtime::ScrollAreaRuntime,
    types::{ScrollAreaSizes, ScrollAreaType},
};

#[derive(Clone, Copy)]
pub struct ScrollAreaContext {
    pub runtime: ScrollAreaRuntime,
    pub relationships: Signal<ScrollAreaRelationships>,
    pub r#type: ScrollAreaType,
    pub dir: Direction,
    pub scroll_hide_delay: u64,
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

pub fn project_scroll_area_root_as_child<T, C>(
    state: &ScrollAreaRootAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ScrollAreaRootAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.clone(), content)
}

pub fn project_scroll_area_viewport_as_child<T, C>(
    state: &ScrollAreaViewportAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ScrollAreaViewportAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.clone(), content)
}

pub fn project_scroll_area_content_as_child<T, C>(
    state: &ScrollAreaContentAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ScrollAreaContentAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.clone(), content)
}

pub fn project_scroll_area_scrollbar_as_child<T, C>(
    state: &ScrollAreaScrollbarAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ScrollAreaScrollbarAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.clone(), content)
}

pub fn project_scroll_area_thumb_as_child<T, C>(
    state: &ScrollAreaThumbAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ScrollAreaThumbAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.clone(), content)
}

pub fn project_scroll_area_corner_as_child<T, C>(
    state: &ScrollAreaCornerAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, ScrollAreaCornerAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, state.clone(), content)
}

use crate::foundation::shared::{Direction, Orientation};

use super::{
    relationships::ScrollAreaRelationships,
    types::{
        DEFAULT_SCROLL_HIDE_DELAY_MS, SCROLL_AREA_MIN_THUMB_SIZE, ScrollAreaSizes,
        ScrollAreaStateEvent, ScrollAreaType, ScrollAreaVisibilityState,
    },
};

pub fn thumb_ratio(viewport: f64, content: f64) -> f64 {
    if content <= 0.0 || viewport <= 0.0 {
        return 0.0;
    }
    let ratio = viewport / content;
    if ratio.is_nan() || ratio.is_infinite() {
        0.0
    } else {
        ratio
    }
}

pub fn has_thumb(sizes: ScrollAreaSizes) -> bool {
    let ratio = thumb_ratio(sizes.viewport, sizes.content);
    ratio > 0.0 && ratio < 1.0
}

pub fn is_overflowing(sizes: ScrollAreaSizes) -> bool {
    sizes.content > sizes.viewport && sizes.viewport > 0.0
}

pub fn get_thumb_size(sizes: ScrollAreaSizes) -> f64 {
    let track_usable = (sizes.scrollbar_size - sizes.padding.total()).max(0.0);
    let ratio = thumb_ratio(sizes.viewport, sizes.content);
    let raw_size = track_usable * ratio;
    raw_size.max(SCROLL_AREA_MIN_THUMB_SIZE)
}

pub fn linear_scale(input: (f64, f64), output: (f64, f64), value: f64) -> f64 {
    if (input.0 - input.1).abs() < f64::EPSILON || (output.0 - output.1).abs() < f64::EPSILON {
        return output.0;
    }
    let ratio = (output.1 - output.0) / (input.1 - input.0);
    output.0 + ratio * (value - input.0)
}

pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    let (real_min, real_max) = if min <= max { (min, max) } else { (max, min) };
    value.max(real_min).min(real_max)
}

pub fn get_thumb_offset_from_scroll(
    scroll_pos: f64,
    sizes: ScrollAreaSizes,
    dir: Direction,
) -> f64 {
    let thumb_size = get_thumb_size(sizes);
    let scrollbar = (sizes.scrollbar_size - sizes.padding.total()).max(0.0);
    let max_scroll_pos = (sizes.content - sizes.viewport).max(0.0);
    let max_thumb_pos = (scrollbar - thumb_size).max(0.0);
    let scroll_clamp_range = match dir {
        Direction::Ltr => (0.0, max_scroll_pos),
        Direction::Rtl => (-max_scroll_pos, 0.0),
    };
    let scroll_without_momentum = clamp(scroll_pos, scroll_clamp_range.0, scroll_clamp_range.1);
    linear_scale(
        (0.0, max_scroll_pos),
        (0.0, max_thumb_pos),
        scroll_without_momentum,
    )
}

pub fn get_scroll_position_from_pointer(
    pointer_pos: f64,
    pointer_offset: f64,
    sizes: ScrollAreaSizes,
    dir: Direction,
) -> f64 {
    let thumb_size = get_thumb_size(sizes);
    let thumb_center = thumb_size / 2.0;
    let offset = if pointer_offset == 0.0 {
        thumb_center
    } else {
        pointer_offset
    };
    let thumb_offset_from_end = thumb_size - offset;
    let min_pointer_pos = sizes.padding.start + offset;
    let max_pointer_pos = sizes.scrollbar_size - sizes.padding.end - thumb_offset_from_end;
    let max_scroll_pos = (sizes.content - sizes.viewport).max(0.0);
    let scroll_range = match dir {
        Direction::Ltr => (0.0, max_scroll_pos),
        Direction::Rtl => (-max_scroll_pos, 0.0),
    };
    linear_scale(
        (min_pointer_pos, max_pointer_pos),
        scroll_range,
        pointer_pos,
    )
}

pub fn is_scrolling_within_scrollbar_bounds(scroll_pos: f64, max_scroll_pos: f64) -> bool {
    scroll_pos > 0.0 && scroll_pos < max_scroll_pos
}

pub fn transition_scroll_state(
    current: ScrollAreaVisibilityState,
    event: ScrollAreaStateEvent,
) -> ScrollAreaVisibilityState {
    match current {
        ScrollAreaVisibilityState::Hidden => match event {
            ScrollAreaStateEvent::Scroll => ScrollAreaVisibilityState::Scrolling,
            _ => ScrollAreaVisibilityState::Hidden,
        },
        ScrollAreaVisibilityState::Scrolling => match event {
            ScrollAreaStateEvent::ScrollEnd => ScrollAreaVisibilityState::Idle,
            ScrollAreaStateEvent::PointerEnter => ScrollAreaVisibilityState::Interacting,
            _ => ScrollAreaVisibilityState::Scrolling,
        },
        ScrollAreaVisibilityState::Interacting => match event {
            ScrollAreaStateEvent::PointerLeave => ScrollAreaVisibilityState::Idle,
            _ => ScrollAreaVisibilityState::Interacting,
        },
        ScrollAreaVisibilityState::Idle => match event {
            ScrollAreaStateEvent::Hide => ScrollAreaVisibilityState::Hidden,
            ScrollAreaStateEvent::Scroll => ScrollAreaVisibilityState::Scrolling,
            ScrollAreaStateEvent::PointerEnter => ScrollAreaVisibilityState::Interacting,
            _ => ScrollAreaVisibilityState::Idle,
        },
        other => other,
    }
}

pub fn is_scrollbar_visible(
    r#type: ScrollAreaType,
    state: ScrollAreaVisibilityState,
    hover_visible: bool,
    overflowing: bool,
) -> bool {
    match r#type {
        ScrollAreaType::Hover => hover_visible && overflowing,
        ScrollAreaType::Scroll => state != ScrollAreaVisibilityState::Hidden,
        ScrollAreaType::Auto => overflowing,
        ScrollAreaType::Always => true,
    }
}

pub fn has_corner(
    r#type: ScrollAreaType,
    scrollbar_x_visible: bool,
    scrollbar_y_visible: bool,
) -> bool {
    r#type != ScrollAreaType::Scroll && scrollbar_x_visible && scrollbar_y_visible
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollArea {
    relationships: ScrollAreaRelationships,
    r#type: ScrollAreaType,
    dir: Direction,
    scroll_hide_delay: u64,
    sizes_x: ScrollAreaSizes,
    sizes_y: ScrollAreaSizes,
    scrollbar_x_enabled: bool,
    scrollbar_y_enabled: bool,
    hover_visible: bool,
    scroll_state: ScrollAreaVisibilityState,
}

impl ScrollArea {
    pub fn new(relationships: ScrollAreaRelationships, r#type: ScrollAreaType) -> Self {
        Self {
            relationships,
            r#type,
            dir: Direction::Ltr,
            scroll_hide_delay: DEFAULT_SCROLL_HIDE_DELAY_MS,
            sizes_x: ScrollAreaSizes::default(),
            sizes_y: ScrollAreaSizes::default(),
            scrollbar_x_enabled: false,
            scrollbar_y_enabled: false,
            hover_visible: false,
            scroll_state: ScrollAreaVisibilityState::Hidden,
        }
    }

    pub fn with_dir(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }

    pub fn with_scroll_hide_delay(mut self, delay: u64) -> Self {
        self.scroll_hide_delay = delay;
        self
    }

    pub fn with_sizes(mut self, orientation: Orientation, sizes: ScrollAreaSizes) -> Self {
        match orientation {
            Orientation::Horizontal => self.sizes_x = sizes,
            Orientation::Vertical => self.sizes_y = sizes,
            Orientation::Both => {
                self.sizes_x = sizes;
                self.sizes_y = sizes;
            }
        }
        self
    }

    pub fn with_scrollbar_enabled(mut self, orientation: Orientation, enabled: bool) -> Self {
        match orientation {
            Orientation::Horizontal => self.scrollbar_x_enabled = enabled,
            Orientation::Vertical => self.scrollbar_y_enabled = enabled,
            Orientation::Both => {
                self.scrollbar_x_enabled = enabled;
                self.scrollbar_y_enabled = enabled;
            }
        }
        self
    }

    pub fn with_hover_visible(mut self, hover_visible: bool) -> Self {
        self.hover_visible = hover_visible;
        self
    }

    pub fn with_scroll_state(mut self, scroll_state: ScrollAreaVisibilityState) -> Self {
        self.scroll_state = scroll_state;
        self
    }

    pub fn relationships(&self) -> &ScrollAreaRelationships {
        &self.relationships
    }

    pub fn r#type(&self) -> ScrollAreaType {
        self.r#type
    }

    pub fn dir(&self) -> Direction {
        self.dir
    }

    pub fn scroll_hide_delay(&self) -> u64 {
        self.scroll_hide_delay
    }

    pub fn sizes(&self, orientation: Orientation) -> ScrollAreaSizes {
        match orientation {
            Orientation::Horizontal => self.sizes_x,
            Orientation::Vertical | Orientation::Both => self.sizes_y,
        }
    }

    pub fn is_scrollbar_enabled(&self, orientation: Orientation) -> bool {
        match orientation {
            Orientation::Horizontal => self.scrollbar_x_enabled,
            Orientation::Vertical | Orientation::Both => self.scrollbar_y_enabled,
        }
    }

    pub fn is_overflowing(&self, orientation: Orientation) -> bool {
        is_overflowing(self.sizes(orientation))
    }

    pub fn is_scrollbar_visible(&self, orientation: Orientation) -> bool {
        let sizes = self.sizes(orientation);
        let overflowing = is_overflowing(sizes);
        is_scrollbar_visible(
            self.r#type,
            self.scroll_state,
            self.hover_visible,
            overflowing,
        )
    }

    pub fn has_corner(&self) -> bool {
        self.scrollbar_x_enabled
            && self.scrollbar_y_enabled
            && has_corner(
                self.r#type,
                self.is_scrollbar_visible(Orientation::Horizontal),
                self.is_scrollbar_visible(Orientation::Vertical),
            )
    }

    pub fn thumb_size(&self, orientation: Orientation) -> f64 {
        get_thumb_size(self.sizes(orientation))
    }
}

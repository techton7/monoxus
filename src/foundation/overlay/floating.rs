use std::borrow::Cow;

use crate::foundation::shared::Direction;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn x(&self) -> f32 {
        self.x
    }

    pub const fn y(&self) -> f32 {
        self.y
    }

    pub const fn width(&self) -> f32 {
        self.width
    }

    pub const fn height(&self) -> f32 {
        self.height
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn center_x(&self) -> f32 {
        self.x + (self.width / 2.0)
    }

    pub fn center_y(&self) -> f32 {
        self.y + (self.height / 2.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const fn width(&self) -> f32 {
        self.width
    }

    pub const fn height(&self) -> f32 {
        self.height
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PlacementSide {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

impl PlacementSide {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PlacementAlign {
    Start,
    #[default]
    Center,
    End,
}

impl PlacementAlign {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloatingLayer {
    side: PlacementSide,
    align: PlacementAlign,
    direction: Direction,
    side_offset: f32,
    align_offset: f32,
    available_size: Option<Size>,
    hide_when_detached: bool,
    namespace: Cow<'static, str>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FloatingArrowPosition {
    x: Option<f32>,
    y: Option<f32>,
    hidden: bool,
}

impl FloatingArrowPosition {
    pub const fn new(x: Option<f32>, y: Option<f32>, hidden: bool) -> Self {
        Self { x, y, hidden }
    }

    pub const fn x(&self) -> Option<f32> {
        self.x
    }

    pub const fn y(&self) -> Option<f32> {
        self.y
    }

    pub const fn hidden(&self) -> bool {
        self.hidden
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloatingPlacement {
    side: PlacementSide,
    align: PlacementAlign,
    geometry: GeometryVars,
    arrow: FloatingArrowPosition,
    reference_hidden: bool,
}

impl FloatingPlacement {
    pub fn new(
        side: PlacementSide,
        align: PlacementAlign,
        geometry: GeometryVars,
        arrow: FloatingArrowPosition,
        reference_hidden: bool,
    ) -> Self {
        Self {
            side,
            align,
            geometry,
            arrow,
            reference_hidden,
        }
    }

    pub const fn side(&self) -> PlacementSide {
        self.side
    }

    pub const fn align(&self) -> PlacementAlign {
        self.align
    }

    pub fn geometry(&self) -> &GeometryVars {
        &self.geometry
    }

    pub const fn arrow(&self) -> &FloatingArrowPosition {
        &self.arrow
    }

    pub const fn reference_hidden(&self) -> bool {
        self.reference_hidden
    }

    pub fn hide_reference(mut self) -> Self {
        self.reference_hidden = true;
        self.arrow = FloatingArrowPosition::new(None, None, true);
        self
    }
}

impl Default for FloatingLayer {
    fn default() -> Self {
        Self::new(PlacementSide::default())
    }
}

impl FloatingLayer {
    pub fn new(side: PlacementSide) -> Self {
        Self {
            side,
            align: PlacementAlign::Center,
            direction: Direction::Ltr,
            side_offset: 0.0,
            align_offset: 0.0,
            available_size: None,
            hide_when_detached: false,
            namespace: Cow::Borrowed(GeometryVars::DEFAULT_NAMESPACE),
        }
    }

    pub fn with_align(mut self, align: PlacementAlign) -> Self {
        self.align = align;
        self
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_side_offset(mut self, side_offset: f32) -> Self {
        self.side_offset = side_offset;
        self
    }

    pub fn with_align_offset(mut self, align_offset: f32) -> Self {
        self.align_offset = align_offset;
        self
    }

    pub fn with_available_space(mut self, available_size: Size) -> Self {
        self.available_size = Some(available_size);
        self
    }

    pub fn with_hide_when_detached(mut self, hide_when_detached: bool) -> Self {
        self.hide_when_detached = hide_when_detached;
        self
    }

    pub fn with_namespace(mut self, namespace: impl Into<Cow<'static, str>>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub const fn side(&self) -> PlacementSide {
        self.side
    }

    pub const fn align(&self) -> PlacementAlign {
        self.align
    }

    pub const fn direction(&self) -> Direction {
        self.direction
    }

    pub const fn hide_when_detached(&self) -> bool {
        self.hide_when_detached
    }

    pub fn data_side(&self) -> &'static str {
        self.side.as_str()
    }

    pub fn data_align(&self) -> &'static str {
        self.align.as_str()
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.position(anchor, content).geometry().clone()
    }

    pub fn position(&self, anchor: Rect, content: Size) -> FloatingPlacement {
        let available_size = self.available_size.unwrap_or_else(|| {
            default_available_size(anchor, content, self.side, self.side_offset)
        });

        self.position_with_available_size(anchor, content, available_size)
    }

    pub fn position_with_available_size(
        &self,
        anchor: Rect,
        content: Size,
        available_size: Size,
    ) -> FloatingPlacement {
        let resolved_inline_align = resolve_inline_align(self.align, self.direction);
        let side = resolve_side(self.side, anchor, content, available_size, self.side_offset);
        let max_horizontal_offset = (available_size.width() - content.width()).max(0.0);
        let max_vertical_offset = (available_size.height() - content.height()).max(0.0);
        let x = match side {
            PlacementSide::Top | PlacementSide::Bottom => clamp_to_extent(
                aligned_horizontal(anchor, content, resolved_inline_align) + self.align_offset,
                max_horizontal_offset,
            ),
            PlacementSide::Right => anchor.right() + self.side_offset,
            PlacementSide::Left => anchor.x() - content.width() - self.side_offset,
        };
        let y = match side {
            PlacementSide::Left | PlacementSide::Right => clamp_to_extent(
                aligned_vertical(anchor, content, self.align) + self.align_offset,
                max_vertical_offset,
            ),
            PlacementSide::Bottom => anchor.bottom() + self.side_offset,
            PlacementSide::Top => anchor.y() - content.height() - self.side_offset,
        };

        let transform_origin_x = match side {
            PlacementSide::Right => 0.0,
            PlacementSide::Left => content.width(),
            PlacementSide::Top | PlacementSide::Bottom => clamp_to_extent(
                aligned_horizontal_point(anchor, resolved_inline_align) - x,
                content.width(),
            ),
        };
        let transform_origin_y = match side {
            PlacementSide::Bottom => 0.0,
            PlacementSide::Top => content.height(),
            PlacementSide::Left | PlacementSide::Right => clamp_to_extent(
                aligned_vertical_point(anchor, self.align) - y,
                content.height(),
            ),
        };
        let geometry = GeometryVars::new(
            self.namespace.clone(),
            x,
            y,
            transform_origin_x,
            transform_origin_y,
            available_size.width(),
            available_size.height(),
            anchor.width(),
            anchor.height(),
            content.width(),
            content.height(),
        );
        let reference_hidden =
            self.hide_when_detached && reference_is_hidden(anchor, available_size);
        let arrow = if reference_hidden {
            FloatingArrowPosition::new(None, None, true)
        } else {
            arrow_position(side, anchor, content, x, y)
        };

        FloatingPlacement::new(side, self.align, geometry, arrow, reference_hidden)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryVars {
    namespace: Cow<'static, str>,
    x: f32,
    y: f32,
    transform_origin_x: f32,
    transform_origin_y: f32,
    available_width: f32,
    available_height: f32,
    anchor_width: f32,
    anchor_height: f32,
    content_width: f32,
    content_height: f32,
}

impl GeometryVars {
    pub const DEFAULT_NAMESPACE: &'static str = "overlay";
    const VAR_FLOATING_X: &'static str = "floating-x";
    const VAR_FLOATING_Y: &'static str = "floating-y";
    const VAR_TRANSFORM_ORIGIN_X: &'static str = "transform-origin-x";
    const VAR_TRANSFORM_ORIGIN_Y: &'static str = "transform-origin-y";
    const VAR_AVAILABLE_WIDTH: &'static str = "available-width";
    const VAR_AVAILABLE_HEIGHT: &'static str = "available-height";
    const VAR_ANCHOR_WIDTH: &'static str = "anchor-width";
    const VAR_ANCHOR_HEIGHT: &'static str = "anchor-height";
    const VAR_CONTENT_WIDTH: &'static str = "content-width";
    const VAR_CONTENT_HEIGHT: &'static str = "content-height";

    pub fn new(
        namespace: impl Into<Cow<'static, str>>,
        x: f32,
        y: f32,
        transform_origin_x: f32,
        transform_origin_y: f32,
        available_width: f32,
        available_height: f32,
        anchor_width: f32,
        anchor_height: f32,
        content_width: f32,
        content_height: f32,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            x,
            y,
            transform_origin_x,
            transform_origin_y,
            available_width,
            available_height,
            anchor_width,
            anchor_height,
            content_width,
            content_height,
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub const fn x(&self) -> f32 {
        self.x
    }

    pub const fn y(&self) -> f32 {
        self.y
    }

    pub const fn transform_origin_x(&self) -> f32 {
        self.transform_origin_x
    }

    pub const fn transform_origin_y(&self) -> f32 {
        self.transform_origin_y
    }

    pub const fn available_width(&self) -> f32 {
        self.available_width
    }

    pub const fn available_height(&self) -> f32 {
        self.available_height
    }

    pub const fn anchor_width(&self) -> f32 {
        self.anchor_width
    }

    pub const fn anchor_height(&self) -> f32 {
        self.anchor_height
    }

    pub const fn content_width(&self) -> f32 {
        self.content_width
    }

    pub const fn content_height(&self) -> f32 {
        self.content_height
    }

    pub fn variable_name(namespace: &str, suffix: &str) -> String {
        format!("--monoxus-{namespace}-{suffix}")
    }

    pub fn namespace_variable_name(&self, suffix: &str) -> String {
        Self::variable_name(self.namespace(), suffix)
    }

    pub fn get(&self, name: &str) -> Option<f32> {
        self.iter()
            .find_map(|(candidate, value)| (candidate == name).then_some(value))
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, f32)> {
        self.entries().into_iter()
    }

    fn entries(&self) -> Vec<(String, f32)> {
        vec![
            (self.namespace_variable_name(Self::VAR_FLOATING_X), self.x),
            (self.namespace_variable_name(Self::VAR_FLOATING_Y), self.y),
            (
                self.namespace_variable_name(Self::VAR_TRANSFORM_ORIGIN_X),
                self.transform_origin_x,
            ),
            (
                self.namespace_variable_name(Self::VAR_TRANSFORM_ORIGIN_Y),
                self.transform_origin_y,
            ),
            (
                self.namespace_variable_name(Self::VAR_AVAILABLE_WIDTH),
                self.available_width,
            ),
            (
                self.namespace_variable_name(Self::VAR_AVAILABLE_HEIGHT),
                self.available_height,
            ),
            (
                self.namespace_variable_name(Self::VAR_ANCHOR_WIDTH),
                self.anchor_width,
            ),
            (
                self.namespace_variable_name(Self::VAR_ANCHOR_HEIGHT),
                self.anchor_height,
            ),
            (
                self.namespace_variable_name(Self::VAR_CONTENT_WIDTH),
                self.content_width,
            ),
            (
                self.namespace_variable_name(Self::VAR_CONTENT_HEIGHT),
                self.content_height,
            ),
        ]
    }
}

fn resolve_inline_align(align: PlacementAlign, direction: Direction) -> PlacementAlign {
    match (align, direction) {
        (PlacementAlign::Start, Direction::Rtl) => PlacementAlign::End,
        (PlacementAlign::End, Direction::Rtl) => PlacementAlign::Start,
        _ => align,
    }
}

fn default_available_size(
    anchor: Rect,
    content: Size,
    side: PlacementSide,
    side_offset: f32,
) -> Size {
    let width = match side {
        PlacementSide::Right => anchor.right() + side_offset + content.width(),
        _ => anchor.right().max(content.width()),
    };
    let height = match side {
        PlacementSide::Bottom => anchor.bottom() + side_offset + content.height(),
        _ => anchor.bottom().max(content.height()),
    };

    Size::new(width.max(content.width()), height.max(content.height()))
}

fn resolve_side(
    preferred_side: PlacementSide,
    anchor: Rect,
    content: Size,
    available_size: Size,
    side_offset: f32,
) -> PlacementSide {
    let preferred_space = available_side_space(preferred_side, anchor, available_size);
    let required_space = required_side_space(preferred_side, content, side_offset);
    if preferred_space >= required_space {
        return preferred_side;
    }

    let opposite_side = preferred_side.opposite();
    let opposite_space = available_side_space(opposite_side, anchor, available_size);
    if opposite_space > preferred_space {
        return opposite_side;
    }

    preferred_side
}

fn available_side_space(side: PlacementSide, anchor: Rect, available_size: Size) -> f32 {
    match side {
        PlacementSide::Top => anchor.y(),
        PlacementSide::Right => available_size.width() - anchor.right(),
        PlacementSide::Bottom => available_size.height() - anchor.bottom(),
        PlacementSide::Left => anchor.x(),
    }
}

fn required_side_space(side: PlacementSide, content: Size, side_offset: f32) -> f32 {
    match side {
        PlacementSide::Top | PlacementSide::Bottom => content.height() + side_offset,
        PlacementSide::Right | PlacementSide::Left => content.width() + side_offset,
    }
}

fn reference_is_hidden(anchor: Rect, available_size: Size) -> bool {
    anchor.width() <= 0.0
        || anchor.height() <= 0.0
        || anchor.right() <= 0.0
        || anchor.bottom() <= 0.0
        || anchor.x() >= available_size.width()
        || anchor.y() >= available_size.height()
}

fn arrow_position(
    side: PlacementSide,
    anchor: Rect,
    content: Size,
    x: f32,
    y: f32,
) -> FloatingArrowPosition {
    const ARROW_EDGE_MARGIN: f32 = 12.0;

    match side {
        PlacementSide::Top | PlacementSide::Bottom => {
            let center_x = anchor.center_x() - x;
            let can_center = content.width() >= ARROW_EDGE_MARGIN * 2.0
                && center_x >= ARROW_EDGE_MARGIN
                && center_x <= content.width() - ARROW_EDGE_MARGIN;
            FloatingArrowPosition::new(can_center.then_some(center_x), None, !can_center)
        }
        PlacementSide::Right | PlacementSide::Left => {
            let center_y = anchor.center_y() - y;
            let can_center = content.height() >= ARROW_EDGE_MARGIN * 2.0
                && center_y >= ARROW_EDGE_MARGIN
                && center_y <= content.height() - ARROW_EDGE_MARGIN;
            FloatingArrowPosition::new(None, can_center.then_some(center_y), !can_center)
        }
    }
}

fn aligned_horizontal(anchor: Rect, content: Size, align: PlacementAlign) -> f32 {
    match align {
        PlacementAlign::Start => anchor.x(),
        PlacementAlign::Center => anchor.x() + ((anchor.width() - content.width()) / 2.0),
        PlacementAlign::End => anchor.right() - content.width(),
    }
}

fn aligned_vertical(anchor: Rect, content: Size, align: PlacementAlign) -> f32 {
    match align {
        PlacementAlign::Start => anchor.y(),
        PlacementAlign::Center => anchor.y() + ((anchor.height() - content.height()) / 2.0),
        PlacementAlign::End => anchor.bottom() - content.height(),
    }
}

fn aligned_horizontal_point(anchor: Rect, align: PlacementAlign) -> f32 {
    match align {
        PlacementAlign::Start => anchor.x(),
        PlacementAlign::Center => anchor.center_x(),
        PlacementAlign::End => anchor.right(),
    }
}

fn aligned_vertical_point(anchor: Rect, align: PlacementAlign) -> f32 {
    match align {
        PlacementAlign::Start => anchor.y(),
        PlacementAlign::Center => anchor.center_y(),
        PlacementAlign::End => anchor.bottom(),
    }
}

fn clamp_to_extent(value: f32, extent: f32) -> f32 {
    value.clamp(0.0, extent.max(0.0))
}

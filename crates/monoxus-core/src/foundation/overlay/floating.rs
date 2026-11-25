use std::borrow::Cow;

use crate::foundation::shared::Direction;

pub const ARROW_EDGE_MARGIN: f32 = 12.0;

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

impl From<(f32, f32, f32, f32)> for Rect {
    fn from((x, y, width, height): (f32, f32, f32, f32)) -> Self {
        Self::new(x, y, width, height)
    }
}

impl From<[f32; 4]> for Rect {
    fn from([x, y, width, height]: [f32; 4]) -> Self {
        Self::new(x, y, width, height)
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

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Self::new(width, height)
    }
}

impl From<[f32; 2]> for Size {
    fn from([width, height]: [f32; 2]) -> Self {
        Self::new(width, height)
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StickyStrategy {
    #[default]
    Partial,
    Always,
}

pub type StickyMode = StickyStrategy;

impl StickyStrategy {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Partial => "partial",
            Self::Always => "always",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FloatingReadiness {
    #[default]
    Measuring,
    Ready,
}

impl FloatingReadiness {
    pub const fn from_is_positioned(is_positioned: bool) -> Self {
        if is_positioned {
            Self::Ready
        } else {
            Self::Measuring
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Measuring => "measuring",
            Self::Ready => "ready",
        }
    }

    pub const fn positioning_state(&self) -> &'static str {
        match self {
            Self::Measuring => "unpositioned",
            Self::Ready => "positioned",
        }
    }

    pub const fn is_positioned(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Standard unpositioned CSS style hiding the element during initial measurement.
    pub const fn unpositioned_style() -> &'static str {
        "position: fixed; left: 0px; top: 0px; min-width: max-content; width: max-content; visibility: hidden; pointer-events: none;"
    }

    /// Returns standard position style given readiness and placement.
    pub fn position_style(&self, placement: Option<&FloatingPlacement>) -> String {
        match (self, placement) {
            (Self::Ready, Some(p)) => {
                let geom = p.geometry();
                format!(
                    "position: fixed; left: {}px; top: {}px; visibility: visible; pointer-events: auto;",
                    geom.x(),
                    geom.y()
                )
            }
            _ => Self::unpositioned_style().to_string(),
        }
    }

    /// Canonical outer wrapper style during measurement before coordinates are locked.
    pub fn wrapper_measuring_style(z_index: Option<&str>) -> String {
        let z = z_index
            .map(|v| format!(" z-index: {v};"))
            .unwrap_or_default();
        format!("{}{}", Self::unpositioned_style(), z)
    }
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

    /// Canonical outer wrapper style combining positioned/measuring style, min-width, z-index, and reference_hidden.
    pub fn wrapper_style(&self, readiness: FloatingReadiness, z_index: Option<&str>) -> String {
        let z = z_index
            .map(|v| format!(" z-index: {v};"))
            .unwrap_or_default();
        let base = readiness.position_style(Some(self));
        let ref_hidden = if self.reference_hidden() {
            " visibility: hidden; pointer-events: none;"
        } else {
            ""
        };
        let min_w = if base.contains("min-width: max-content;") {
            ""
        } else {
            " min-width: max-content;"
        };
        format!("{base}{min_w}{z}{ref_hidden}")
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
    avoid_collisions: bool,
    collision_padding: f32,
    arrow_padding: f32,
    sticky: StickyStrategy,
    hide_when_detached: bool,
    namespace: Cow<'static, str>,
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
            avoid_collisions: true,
            collision_padding: 0.0,
            arrow_padding: 0.0,
            sticky: StickyStrategy::Partial,
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

    pub fn with_avoid_collisions(mut self, avoid_collisions: bool) -> Self {
        self.avoid_collisions = avoid_collisions;
        self
    }

    pub fn with_collision_padding(mut self, collision_padding: f32) -> Self {
        self.collision_padding = collision_padding;
        self
    }

    pub fn with_arrow_padding(mut self, arrow_padding: f32) -> Self {
        self.arrow_padding = arrow_padding;
        self
    }

    pub fn with_sticky(mut self, sticky: StickyStrategy) -> Self {
        self.sticky = sticky;
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

    pub const fn side_offset(&self) -> f32 {
        self.side_offset
    }

    pub const fn align_offset(&self) -> f32 {
        self.align_offset
    }

    pub const fn direction(&self) -> Direction {
        self.direction
    }

    pub const fn available_size(&self) -> Option<Size> {
        self.available_size
    }

    pub const fn avoid_collisions(&self) -> bool {
        self.avoid_collisions
    }

    pub const fn collision_padding(&self) -> f32 {
        self.collision_padding
    }

    pub const fn arrow_padding(&self) -> f32 {
        self.arrow_padding
    }

    pub const fn sticky(&self) -> StickyStrategy {
        self.sticky
    }

    pub const fn hide_when_detached(&self) -> bool {
        self.hide_when_detached
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
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
        self.position_in_boundary(
            anchor,
            content,
            Rect::new(0.0, 0.0, available_size.width(), available_size.height()),
        )
    }

    pub fn position_in_boundary(
        &self,
        anchor: Rect,
        content: Size,
        boundary: Rect,
    ) -> FloatingPlacement {
        let padding = self.collision_padding.max(0.0);
        let b_min_x = boundary.x() + padding;
        let b_max_x = (boundary.right() - padding).max(b_min_x);
        let b_min_y = boundary.y() + padding;
        let b_max_y = (boundary.bottom() - padding).max(b_min_y);

        let resolved_inline_align = resolve_inline_align(self.align, self.direction);

        let side = if self.avoid_collisions {
            resolve_side(
                self.side,
                anchor,
                content,
                b_min_x,
                b_max_x,
                b_min_y,
                b_max_y,
                self.side_offset,
            )
        } else {
            self.side
        };

        let (x, y) = match side {
            PlacementSide::Top | PlacementSide::Bottom => {
                let raw_x =
                    aligned_horizontal(anchor, content, resolved_inline_align) + self.align_offset;
                let x = if self.avoid_collisions {
                    let max_x = (b_max_x - content.width()).max(b_min_x);
                    let mut clamped = raw_x.clamp(b_min_x, max_x);
                    if self.sticky == StickyStrategy::Partial {
                        let limit_min_x = anchor.x() - content.width();
                        let limit_max_x = anchor.right();
                        let lo = limit_min_x.min(limit_max_x);
                        let hi = limit_min_x.max(limit_max_x);
                        clamped = clamped.clamp(lo, hi);
                    }
                    clamped
                } else {
                    raw_x
                };
                let y = match side {
                    PlacementSide::Top => anchor.y() - content.height() - self.side_offset,
                    PlacementSide::Bottom => anchor.bottom() + self.side_offset,
                    _ => unreachable!(),
                };
                (x, y)
            }
            PlacementSide::Left | PlacementSide::Right => {
                let raw_y = aligned_vertical(anchor, content, self.align) + self.align_offset;
                let y = if self.avoid_collisions {
                    let max_y = (b_max_y - content.height()).max(b_min_y);
                    let mut clamped = raw_y.clamp(b_min_y, max_y);
                    if self.sticky == StickyStrategy::Partial {
                        let limit_min_y = anchor.y() - content.height();
                        let limit_max_y = anchor.bottom();
                        let lo = limit_min_y.min(limit_max_y);
                        let hi = limit_min_y.max(limit_max_y);
                        clamped = clamped.clamp(lo, hi);
                    }
                    clamped
                } else {
                    raw_y
                };
                let x = match side {
                    PlacementSide::Left => anchor.x() - content.width() - self.side_offset,
                    PlacementSide::Right => anchor.right() + self.side_offset,
                    _ => unreachable!(),
                };
                (x, y)
            }
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
            boundary.width(),
            boundary.height(),
            anchor.width(),
            anchor.height(),
            content.width(),
            content.height(),
        );

        let reference_hidden =
            self.hide_when_detached && reference_is_hidden_in_boundary(anchor, boundary);
        let arrow = if reference_hidden {
            FloatingArrowPosition::new(None, None, true)
        } else {
            arrow_position(side, anchor, content, x, y, self.arrow_padding)
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

    #[allow(clippy::too_many_arguments)]
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

    pub fn css_get(&self, name: &str) -> Option<String> {
        self.css_iter()
            .find_map(|(candidate, value)| (candidate == name).then_some(value))
    }

    pub fn get(&self, name: &str) -> Option<f32> {
        self.iter()
            .find_map(|(candidate, value)| (candidate == name).then_some(value))
    }

    pub fn css_iter(&self) -> impl Iterator<Item = (String, String)> {
        self.css_entries().into_iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, f32)> {
        self.entries().into_iter()
    }

    pub fn compatibility_alias_iter<'a>(
        &'a self,
        prefix: &'a str,
        anchor_label: &'a str,
    ) -> impl Iterator<Item = (String, String)> + 'a {
        self.compatibility_alias_entries(prefix, anchor_label)
            .into_iter()
    }

    pub fn transform_origin_css_value(&self) -> String {
        format!(
            "{} {}",
            Self::css_length(self.transform_origin_x),
            Self::css_length(self.transform_origin_y),
        )
    }

    fn compatibility_alias_variable_name(prefix: &str, suffix: &str) -> String {
        format!("--{prefix}-{suffix}")
    }

    fn css_entries(&self) -> Vec<(String, String)> {
        self.entries()
            .into_iter()
            .map(|(name, value)| (name, Self::css_length(value)))
            .collect()
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

    fn compatibility_alias_entries(
        &self,
        prefix: &str,
        anchor_label: &str,
    ) -> Vec<(String, String)> {
        vec![
            (
                Self::compatibility_alias_variable_name(prefix, "content-transform-origin"),
                self.transform_origin_css_value(),
            ),
            (
                Self::compatibility_alias_variable_name(prefix, "content-available-width"),
                Self::css_length(self.available_width),
            ),
            (
                Self::compatibility_alias_variable_name(prefix, "content-available-height"),
                Self::css_length(self.available_height),
            ),
            (
                Self::compatibility_alias_variable_name(prefix, &format!("{anchor_label}-width")),
                Self::css_length(self.anchor_width),
            ),
            (
                Self::compatibility_alias_variable_name(prefix, &format!("{anchor_label}-height")),
                Self::css_length(self.anchor_height),
            ),
        ]
    }

    fn css_length(value: f32) -> String {
        format!("{value}px")
    }
}

pub fn resolve_inline_align(align: PlacementAlign, direction: Direction) -> PlacementAlign {
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

#[allow(clippy::too_many_arguments)]
pub fn resolve_side(
    preferred_side: PlacementSide,
    anchor: Rect,
    content: Size,
    b_min_x: f32,
    b_max_x: f32,
    b_min_y: f32,
    b_max_y: f32,
    side_offset: f32,
) -> PlacementSide {
    let preferred_space =
        available_side_space(preferred_side, anchor, b_min_x, b_max_x, b_min_y, b_max_y);
    let required_space = required_side_space(preferred_side, content, side_offset);
    if preferred_space >= required_space {
        return preferred_side;
    }

    let opposite_side = preferred_side.opposite();
    let opposite_space =
        available_side_space(opposite_side, anchor, b_min_x, b_max_x, b_min_y, b_max_y);
    if opposite_space > preferred_space {
        return opposite_side;
    }

    preferred_side
}

fn available_side_space(
    side: PlacementSide,
    anchor: Rect,
    b_min_x: f32,
    b_max_x: f32,
    b_min_y: f32,
    b_max_y: f32,
) -> f32 {
    match side {
        PlacementSide::Top => anchor.y() - b_min_y,
        PlacementSide::Right => b_max_x - anchor.right(),
        PlacementSide::Bottom => b_max_y - anchor.bottom(),
        PlacementSide::Left => anchor.x() - b_min_x,
    }
}

fn required_side_space(side: PlacementSide, content: Size, side_offset: f32) -> f32 {
    match side {
        PlacementSide::Top | PlacementSide::Bottom => content.height() + side_offset,
        PlacementSide::Right | PlacementSide::Left => content.width() + side_offset,
    }
}

pub fn reference_is_hidden(anchor: Rect, available_size: Size) -> bool {
    reference_is_hidden_in_boundary(
        anchor,
        Rect::new(0.0, 0.0, available_size.width(), available_size.height()),
    )
}

pub fn reference_is_hidden_in_boundary(anchor: Rect, boundary: Rect) -> bool {
    anchor.width() <= 0.0
        || anchor.height() <= 0.0
        || anchor.right() <= boundary.x()
        || anchor.bottom() <= boundary.y()
        || anchor.x() >= boundary.right()
        || anchor.y() >= boundary.bottom()
}

pub fn arrow_position(
    side: PlacementSide,
    anchor: Rect,
    content: Size,
    x: f32,
    y: f32,
    arrow_padding: f32,
) -> FloatingArrowPosition {
    let margin = if arrow_padding > 0.0 {
        arrow_padding
    } else {
        ARROW_EDGE_MARGIN
    };

    match side {
        PlacementSide::Top | PlacementSide::Bottom => {
            let center_x = anchor.center_x() - x;
            let can_center = content.width() >= margin * 2.0
                && center_x >= margin
                && center_x <= content.width() - margin;
            FloatingArrowPosition::new(can_center.then_some(center_x), None, !can_center)
        }
        PlacementSide::Right | PlacementSide::Left => {
            let center_y = anchor.center_y() - y;
            let can_center = content.height() >= margin * 2.0
                && center_y >= margin
                && center_y <= content.height() - margin;
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

pub fn aligned_horizontal_point(anchor: Rect, align: PlacementAlign) -> f32 {
    match align {
        PlacementAlign::Start => anchor.x(),
        PlacementAlign::Center => anchor.center_x(),
        PlacementAlign::End => anchor.right(),
    }
}

pub fn aligned_vertical_point(anchor: Rect, align: PlacementAlign) -> f32 {
    match align {
        PlacementAlign::Start => anchor.y(),
        PlacementAlign::Center => anchor.center_y(),
        PlacementAlign::End => anchor.bottom(),
    }
}

pub fn clamp_to_extent(value: f32, extent: f32) -> f32 {
    value.clamp(0.0, extent.max(0.0))
}

#[derive(Clone, Debug, PartialEq)]
pub struct PureOverlayPositionConfig<'a> {
    pub anchor: Rect,
    pub content: Size,
    pub boundary: Option<Rect>,
    pub side: PlacementSide,
    pub align: PlacementAlign,
    pub direction: Direction,
    pub side_offset: f32,
    pub align_offset: f32,
    pub avoid_collisions: bool,
    pub collision_padding: f32,
    pub arrow_padding: f32,
    pub sticky: StickyStrategy,
    pub hide_when_detached: bool,
    pub namespace: Option<&'a str>,
}

pub fn compute_pure_overlay_position(config: &PureOverlayPositionConfig) -> FloatingPlacement {
    let mut layer = FloatingLayer::new(config.side)
        .with_align(config.align)
        .with_direction(config.direction)
        .with_side_offset(config.side_offset)
        .with_align_offset(config.align_offset)
        .with_avoid_collisions(config.avoid_collisions)
        .with_collision_padding(config.collision_padding)
        .with_arrow_padding(config.arrow_padding)
        .with_sticky(config.sticky)
        .with_hide_when_detached(config.hide_when_detached);

    if let Some(ns) = config.namespace {
        layer = layer.with_namespace(ns.to_string());
    }

    if let Some(boundary) = config.boundary {
        layer.position_in_boundary(config.anchor, config.content, boundary)
    } else {
        layer.position(config.anchor, config.content)
    }
}

/// Normalizes a CSSOM computed `z-index` string for floating wrapper hoisting.
///
/// Returns `None` for `None`, empty/whitespace-only strings, or `"auto"` (case-insensitive)
/// so the outer floating wrapper omits `z-index:` when no explicit stacking layer is set.
/// Preserves explicit integer values including `"0"` and negative integers like `"-1"`.
pub fn normalize_hoisted_z_index(raw: Option<&str>) -> Option<String> {
    let trimmed = raw?.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_hoisted_z_index_distinguishes_auto_zero_and_negative() {
        assert_eq!(normalize_hoisted_z_index(None), None);
        assert_eq!(normalize_hoisted_z_index(Some("")), None);
        assert_eq!(normalize_hoisted_z_index(Some("   ")), None);
        assert_eq!(normalize_hoisted_z_index(Some("auto")), None);
        assert_eq!(normalize_hoisted_z_index(Some("AUTO")), None);
        assert_eq!(normalize_hoisted_z_index(Some("  Auto  ")), None);

        assert_eq!(normalize_hoisted_z_index(Some("0")), Some("0".to_string()));
        assert_eq!(normalize_hoisted_z_index(Some("-1")), Some("-1".to_string()));
        assert_eq!(normalize_hoisted_z_index(Some("50")), Some("50".to_string()));
        assert_eq!(
            normalize_hoisted_z_index(Some(" 9999 ")),
            Some("9999".to_string())
        );
    }

    #[test]
    fn wrapper_style_emits_or_omits_z_index_cleanly() {
        let placement = FloatingLayer::new(PlacementSide::Bottom)
            .position(Rect::new(10.0, 20.0, 100.0, 32.0), Size::new(160.0, 80.0));

        let ready_none = placement.wrapper_style(FloatingReadiness::Ready, None);
        assert!(!ready_none.contains("z-index:"));

        let ready_50 = placement.wrapper_style(FloatingReadiness::Ready, Some("50"));
        assert!(ready_50.contains(" z-index: 50;"));

        let ready_0 = placement.wrapper_style(FloatingReadiness::Ready, Some("0"));
        assert!(ready_0.contains(" z-index: 0;"));

        let ready_neg = placement.wrapper_style(FloatingReadiness::Ready, Some("-1"));
        assert!(ready_neg.contains(" z-index: -1;"));

        let measuring_none = FloatingReadiness::wrapper_measuring_style(None);
        assert!(!measuring_none.contains("z-index:"));

        let measuring_50 = FloatingReadiness::wrapper_measuring_style(Some("50"));
        assert!(measuring_50.contains(" z-index: 50;"));
    }
}

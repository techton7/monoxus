//! Thin compatibility shim over [`ui_sdk::foundation::overlay::floating`]
//! preserving legacy `--monoxus-{namespace}-{suffix}` CSS custom property
//! naming during Strangler Fig migration waves.

use std::borrow::Cow;
use std::ops::Deref;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::foundation::shared::Direction;

pub use ui_sdk::foundation::overlay::floating::{
    FloatingArrowPosition, PlacementAlign, PlacementSide, Rect, Size, StickyMode, StickyStrategy,
    aligned_horizontal_point, aligned_vertical_point, arrow_position, clamp_to_extent,
    compute_arrow_inline_style, normalize_hoisted_z_index, resolve_inline_align,
    serialize_css_custom_properties,
};

const MONOXUS_CSS_PREFIX: &str = "monoxus";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    pub const fn unpositioned_style() -> &'static str {
        ui_sdk::foundation::overlay::floating::FloatingReadiness::unpositioned_style()
    }

    pub fn position_style(&self, placement: Option<&FloatingPlacement>) -> String {
        self.to_ui_sdk()
            .position_style(placement.map(|p| &p.0))
    }

    pub fn wrapper_measuring_style(z_index: Option<&str>) -> String {
        ui_sdk::foundation::overlay::floating::FloatingReadiness::wrapper_measuring_style(z_index)
    }

    const fn to_ui_sdk(self) -> ui_sdk::foundation::overlay::floating::FloatingReadiness {
        match self {
            Self::Measuring => ui_sdk::foundation::overlay::floating::FloatingReadiness::Measuring,
            Self::Ready => ui_sdk::foundation::overlay::floating::FloatingReadiness::Ready,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryVars(ui_sdk::foundation::overlay::floating::GeometryVars);

impl Deref for GeometryVars {
    type Target = ui_sdk::foundation::overlay::floating::GeometryVars;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ui_sdk::foundation::overlay::floating::GeometryVars> for GeometryVars {
    fn from(inner: ui_sdk::foundation::overlay::floating::GeometryVars) -> Self {
        Self(inner.with_css_prefix(MONOXUS_CSS_PREFIX))
    }
}

impl GeometryVars {
    pub const DEFAULT_NAMESPACE: &'static str =
        ui_sdk::foundation::overlay::floating::GeometryVars::DEFAULT_NAMESPACE;

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
        Self(
            ui_sdk::foundation::overlay::floating::GeometryVars::new(
                namespace,
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
            )
            .with_css_prefix(MONOXUS_CSS_PREFIX),
        )
    }

    pub fn variable_name(namespace: &str, suffix: &str) -> String {
        ui_sdk::foundation::overlay::floating::GeometryVars::variable_name_with_prefix(
            MONOXUS_CSS_PREFIX,
            namespace,
            suffix,
        )
    }

    pub fn namespace_variable_name(&self, suffix: &str) -> String {
        Self::variable_name(self.namespace(), suffix)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloatingPlacement(
    ui_sdk::foundation::overlay::floating::FloatingPlacement,
    GeometryVars,
);

impl FloatingPlacement {
    pub fn new(
        side: PlacementSide,
        align: PlacementAlign,
        geometry: GeometryVars,
        arrow: FloatingArrowPosition,
        reference_hidden: bool,
    ) -> Self {
        let inner = ui_sdk::foundation::overlay::floating::FloatingPlacement::new(
            side,
            align,
            geometry.0.clone(),
            arrow,
            reference_hidden,
        );
        Self(inner, geometry)
    }

    fn from_inner(inner: ui_sdk::foundation::overlay::floating::FloatingPlacement) -> Self {
        let geom = GeometryVars(inner.geometry().clone().with_css_prefix(MONOXUS_CSS_PREFIX));
        Self(inner, geom)
    }

    pub const fn side(&self) -> PlacementSide {
        self.0.side()
    }

    pub const fn align(&self) -> PlacementAlign {
        self.0.align()
    }

    pub fn geometry(&self) -> &GeometryVars {
        &self.1
    }

    pub const fn arrow(&self) -> &FloatingArrowPosition {
        self.0.arrow()
    }

    pub const fn reference_hidden(&self) -> bool {
        self.0.reference_hidden()
    }

    pub fn hide_reference(self) -> Self {
        Self::from_inner(self.0.hide_reference())
    }

    pub fn wrapper_style(&self, readiness: FloatingReadiness, z_index: Option<&str>) -> String {
        self.0.wrapper_style(readiness.to_ui_sdk(), z_index)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloatingLayer(ui_sdk::foundation::overlay::floating::FloatingLayer);

impl Default for FloatingLayer {
    fn default() -> Self {
        Self::new(PlacementSide::default())
    }
}

impl Deref for FloatingLayer {
    type Target = ui_sdk::foundation::overlay::floating::FloatingLayer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FloatingLayer {
    pub fn new(side: PlacementSide) -> Self {
        Self(
            ui_sdk::foundation::overlay::floating::FloatingLayer::new(side)
                .with_css_prefix(MONOXUS_CSS_PREFIX),
        )
    }

    pub fn with_align(self, align: PlacementAlign) -> Self {
        Self(self.0.with_align(align))
    }

    pub fn with_direction(self, direction: Direction) -> Self {
        Self(self.0.with_direction(direction))
    }

    pub fn with_side_offset(self, side_offset: f32) -> Self {
        Self(self.0.with_side_offset(side_offset))
    }

    pub fn with_align_offset(self, align_offset: f32) -> Self {
        Self(self.0.with_align_offset(align_offset))
    }

    pub fn with_available_space(self, available_size: Size) -> Self {
        Self(self.0.with_available_space(available_size))
    }

    pub fn with_avoid_collisions(self, avoid_collisions: bool) -> Self {
        Self(self.0.with_avoid_collisions(avoid_collisions))
    }

    pub fn with_collision_padding(self, collision_padding: f32) -> Self {
        Self(self.0.with_collision_padding(collision_padding))
    }

    pub fn with_arrow_padding(self, arrow_padding: f32) -> Self {
        Self(self.0.with_arrow_padding(arrow_padding))
    }

    pub fn with_sticky(self, sticky: StickyStrategy) -> Self {
        Self(self.0.with_sticky(sticky))
    }

    pub fn with_hide_when_detached(self, hide_when_detached: bool) -> Self {
        Self(self.0.with_hide_when_detached(hide_when_detached))
    }

    pub fn with_namespace(self, namespace: impl Into<Cow<'static, str>>) -> Self {
        Self(self.0.with_namespace(namespace))
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        GeometryVars::from(self.0.geometry_vars(anchor, content))
    }

    pub fn position(&self, anchor: Rect, content: Size) -> FloatingPlacement {
        FloatingPlacement::from_inner(self.0.position(anchor, content))
    }

    pub fn position_with_available_size(
        &self,
        anchor: Rect,
        content: Size,
        available_size: Size,
    ) -> FloatingPlacement {
        FloatingPlacement::from_inner(
            self.0
                .position_with_available_size(anchor, content, available_size),
        )
    }

    pub fn position_in_boundary(
        &self,
        anchor: Rect,
        content: Size,
        boundary: Rect,
    ) -> FloatingPlacement {
        FloatingPlacement::from_inner(self.0.position_in_boundary(anchor, content, boundary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compat_shim_preserves_monoxus_css_variable_prefix() {
        let geom = FloatingLayer::new(PlacementSide::Bottom)
            .with_namespace("popover")
            .geometry_vars(Rect::new(10.0, 20.0, 80.0, 30.0), Size::new(120.0, 40.0));
        assert!(geom.css_get("--monoxus-popover-floating-x").is_some());
        assert_eq!(
            GeometryVars::variable_name("dialog", "floating-x"),
            "--monoxus-dialog-floating-x"
        );
    }
}

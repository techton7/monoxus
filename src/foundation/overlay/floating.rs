pub use monoxus_core::foundation::overlay::floating::*;

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayPositionConfig<'a> {
    pub anchor_id: &'a str,
    pub content_id: &'a str,
    pub arrow_id: Option<&'a str>,
    pub boundary_id: Option<&'a str>,
    pub side: PlacementSide,
    pub align: PlacementAlign,
    pub side_offset: f32,
    pub align_offset: f32,
    pub avoid_collisions: bool,
    pub collision_padding: f32,
    pub arrow_padding: f32,
    pub hide_when_detached: bool,
    pub sticky: bool,
    pub namespace: Option<&'a str>,
}

pub fn compute_overlay_position(config: &OverlayPositionConfig) -> Option<FloatingPlacement> {
    use oxidase_floating_ui::platform::FloatingPlatform;

    let platform = oxidase::floating::HostFloatingPlatform::current()?;

    let placement = match (config.side, config.align) {
        (PlacementSide::Top, PlacementAlign::Start) => oxidase_floating_ui::geometry::Placement::TopStart,
        (PlacementSide::Top, PlacementAlign::Center) => oxidase_floating_ui::geometry::Placement::Top,
        (PlacementSide::Top, PlacementAlign::End) => oxidase_floating_ui::geometry::Placement::TopEnd,
        (PlacementSide::Right, PlacementAlign::Start) => oxidase_floating_ui::geometry::Placement::RightStart,
        (PlacementSide::Right, PlacementAlign::Center) => oxidase_floating_ui::geometry::Placement::Right,
        (PlacementSide::Right, PlacementAlign::End) => oxidase_floating_ui::geometry::Placement::RightEnd,
        (PlacementSide::Bottom, PlacementAlign::Start) => oxidase_floating_ui::geometry::Placement::BottomStart,
        (PlacementSide::Bottom, PlacementAlign::Center) => oxidase_floating_ui::geometry::Placement::Bottom,
        (PlacementSide::Bottom, PlacementAlign::End) => oxidase_floating_ui::geometry::Placement::BottomEnd,
        (PlacementSide::Left, PlacementAlign::Start) => oxidase_floating_ui::geometry::Placement::LeftStart,
        (PlacementSide::Left, PlacementAlign::Center) => oxidase_floating_ui::geometry::Placement::Left,
        (PlacementSide::Left, PlacementAlign::End) => oxidase_floating_ui::geometry::Placement::LeftEnd,
    };

    let mut middlewares: Vec<Box<dyn oxidase_floating_ui::types::Middleware<String, ()>>> = Vec::new();

    middlewares.push(Box::new(oxidase_floating_ui::middleware::Offset::new(
        oxidase_floating_ui::middleware::OffsetOptions::Values(
            oxidase_floating_ui::middleware::OffsetOptionsValues::default()
                .main_axis(config.side_offset as f64)
                .cross_axis(config.align_offset as f64),
        ),
    )));

    if config.avoid_collisions {
        middlewares.push(Box::new(oxidase_floating_ui::middleware::Flip::new(
            oxidase_floating_ui::middleware::FlipOptions::default(),
        )));

        let mut shift_overflow = oxidase_floating_ui::detect_overflow::DetectOverflowOptions::default()
            .padding(oxidase_floating_ui::geometry::Padding::All(config.collision_padding as f64));
        if let Some(bid) = config.boundary_id {
            shift_overflow = shift_overflow.boundary(oxidase_floating_ui::types::Boundary::Element(bid.to_string()));
        } else {
            shift_overflow = shift_overflow.boundary(oxidase_floating_ui::types::Boundary::ClippingAncestors);
        }
        middlewares.push(Box::new(oxidase_floating_ui::middleware::Shift::new(
            oxidase_floating_ui::middleware::ShiftOptions::default().detect_overflow(shift_overflow),
        )));
    }

    if let Some(aid) = config.arrow_id {
        middlewares.push(Box::new(oxidase_floating_ui::middleware::Arrow::new(
            oxidase_floating_ui::middleware::ArrowOptions::new(aid.to_string())
                .padding(oxidase_floating_ui::geometry::Padding::All(config.arrow_padding as f64)),
        )));
    }

    if config.hide_when_detached {
        let mut hide_overflow = oxidase_floating_ui::detect_overflow::DetectOverflowOptions::default();
        if let Some(bid) = config.boundary_id {
            hide_overflow = hide_overflow
                .boundary(oxidase_floating_ui::types::Boundary::Element(bid.to_string()))
                .padding(oxidase_floating_ui::geometry::Padding::All(config.collision_padding as f64));
        } else {
            hide_overflow = hide_overflow
                .boundary(oxidase_floating_ui::types::Boundary::ClippingAncestors)
                .padding(oxidase_floating_ui::geometry::Padding::All(0.0));
        }
        middlewares.push(Box::new(oxidase_floating_ui::middleware::Hide::new(
            oxidase_floating_ui::middleware::HideOptions::default().detect_overflow(hide_overflow),
        )));
    }

    let anchor_str = config.anchor_id.to_string();
    let content_str = config.content_id.to_string();

    let element_rects = platform.get_element_rects(&anchor_str, &content_str).ok()?;
    let boundary = if let Some(bid) = config.boundary_id {
        oxidase_floating_ui::types::Boundary::Element(bid.to_string())
    } else {
        oxidase_floating_ui::types::Boundary::ClippingAncestors
    };
    let clip_rect = platform.get_clipping_rect(&content_str, boundary).unwrap_or_default();
    let available_size = Size::new(clip_rect.width as f32, clip_rect.height as f32);

    let compute_config = oxidase_floating_ui::types::ComputePositionConfig::new(&platform)
        .placement(placement)
        .strategy(oxidase_floating_ui::geometry::Strategy::Fixed)
        .middleware(middlewares);
    let reference = oxidase_floating_ui::geometry::ElementOrVirtual::Element(&anchor_str);
    let result = oxidase_floating_ui::compute_position(reference, &content_str, compute_config);

    let (side, align) = match result.placement {
        oxidase_floating_ui::geometry::Placement::Top => (PlacementSide::Top, PlacementAlign::Center),
        oxidase_floating_ui::geometry::Placement::TopStart => (PlacementSide::Top, PlacementAlign::Start),
        oxidase_floating_ui::geometry::Placement::TopEnd => (PlacementSide::Top, PlacementAlign::End),
        oxidase_floating_ui::geometry::Placement::Right => (PlacementSide::Right, PlacementAlign::Center),
        oxidase_floating_ui::geometry::Placement::RightStart => (PlacementSide::Right, PlacementAlign::Start),
        oxidase_floating_ui::geometry::Placement::RightEnd => (PlacementSide::Right, PlacementAlign::End),
        oxidase_floating_ui::geometry::Placement::Bottom => (PlacementSide::Bottom, PlacementAlign::Center),
        oxidase_floating_ui::geometry::Placement::BottomStart => (PlacementSide::Bottom, PlacementAlign::Start),
        oxidase_floating_ui::geometry::Placement::BottomEnd => (PlacementSide::Bottom, PlacementAlign::End),
        oxidase_floating_ui::geometry::Placement::Left => (PlacementSide::Left, PlacementAlign::Center),
        oxidase_floating_ui::geometry::Placement::LeftStart => (PlacementSide::Left, PlacementAlign::Start),
        oxidase_floating_ui::geometry::Placement::LeftEnd => (PlacementSide::Left, PlacementAlign::End),
    };

    let x = result.x as f32;
    let y = result.y as f32;

    let anchor_rect = Rect::new(
        element_rects.reference.x as f32,
        element_rects.reference.y as f32,
        element_rects.reference.width as f32,
        element_rects.reference.height as f32,
    );
    let content_size = Size::new(
        element_rects.floating.width as f32,
        element_rects.floating.height as f32,
    );

    let transform_origin_x = match side {
        PlacementSide::Right => 0.0,
        PlacementSide::Left => content_size.width(),
        PlacementSide::Top | PlacementSide::Bottom => clamp_to_extent(
            aligned_horizontal_point(anchor_rect, align) - x,
            content_size.width(),
        ),
    };
    let transform_origin_y = match side {
        PlacementSide::Bottom => 0.0,
        PlacementSide::Top => content_size.height(),
        PlacementSide::Left | PlacementSide::Right => clamp_to_extent(
            aligned_vertical_point(anchor_rect, align) - y,
            content_size.height(),
        ),
    };

    let ns = config.namespace.unwrap_or(GeometryVars::DEFAULT_NAMESPACE);
    let geometry = GeometryVars::new(
        ns.to_string(),
        x,
        y,
        transform_origin_x,
        transform_origin_y,
        available_size.width(),
        available_size.height(),
        anchor_rect.width(),
        anchor_rect.height(),
        content_size.width(),
        content_size.height(),
    );

    let hide_data: Option<oxidase_floating_ui::middleware::HideData> =
        result.middleware_data.get_as(oxidase_floating_ui::middleware::HIDE_NAME);
    let reference_hidden = if config.sticky {
        false
    } else {
        hide_data.and_then(|h| h.reference_hidden).unwrap_or(false)
    };

    let arrow_data: Option<oxidase_floating_ui::middleware::ArrowData> =
        result.middleware_data.get_as(oxidase_floating_ui::middleware::ARROW_NAME);
    let arrow = if reference_hidden {
        FloatingArrowPosition::new(None, None, true)
    } else if let Some(data) = arrow_data {
        let arrow_x = data.x.map(|v| v as f32);
        let arrow_y = data.y.map(|v| v as f32);
        let hidden = arrow_x.is_none() && arrow_y.is_none();
        FloatingArrowPosition::new(arrow_x, arrow_y, hidden)
    } else {
        arrow_position(
            side,
            anchor_rect,
            content_size,
            x,
            y,
            config.arrow_padding,
        )
    };

    Some(FloatingPlacement::new(side, align, geometry, arrow, reference_hidden))
}

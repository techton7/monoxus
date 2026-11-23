pub use monoxus_core::foundation::overlay::floating::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OverlayAnchor<'a> {
    Element(&'a str),
    VirtualPoint {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        context_element_id: Option<&'a str>,
    },
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct VirtualOverlayPositionConfig<'a> {
    pub anchor: OverlayAnchor<'a>,
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

impl<'a> From<&OverlayPositionConfig<'a>> for VirtualOverlayPositionConfig<'a> {
    fn from(config: &OverlayPositionConfig<'a>) -> Self {
        Self {
            anchor: OverlayAnchor::Element(config.anchor_id),
            content_id: config.content_id,
            arrow_id: config.arrow_id,
            boundary_id: config.boundary_id,
            side: config.side,
            align: config.align,
            side_offset: config.side_offset,
            align_offset: config.align_offset,
            avoid_collisions: config.avoid_collisions,
            collision_padding: config.collision_padding,
            arrow_padding: config.arrow_padding,
            hide_when_detached: config.hide_when_detached,
            sticky: config.sticky,
            namespace: config.namespace,
        }
    }
}

pub fn compute_overlay_position(config: &OverlayPositionConfig) -> Option<FloatingPlacement> {
    compute_virtual_overlay_position(&VirtualOverlayPositionConfig::from(config))
}

pub fn compute_virtual_overlay_position(
    config: &VirtualOverlayPositionConfig,
) -> Option<FloatingPlacement> {
    use oxidase_floating_ui::geometry::{
        ClientRectObject, DefaultVirtualElement, ElementOrVirtual, Rect as FloatingRect,
    };
    use oxidase_floating_ui::platform::FloatingPlatform;
    use std::rc::Rc;

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

    let can_detect_reference_overflow = match config.anchor {
        OverlayAnchor::Element(_) => true,
        OverlayAnchor::VirtualPoint {
            context_element_id, ..
        } => context_element_id.is_some(),
    };

    if config.hide_when_detached && can_detect_reference_overflow {
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

    let content_str = config.content_id.to_string();
    let (anchor_rect, content_size, anchor_id_owned) = match config.anchor {
        OverlayAnchor::Element(anchor_id) => {
            let anchor_str = anchor_id.to_string();
            let element_rects = platform.get_element_rects(&anchor_str, &content_str).ok()?;
            (
                Rect::new(
                    element_rects.reference.x as f32,
                    element_rects.reference.y as f32,
                    element_rects.reference.width as f32,
                    element_rects.reference.height as f32,
                ),
                Size::new(
                    element_rects.floating.width as f32,
                    element_rects.floating.height as f32,
                ),
                Some(anchor_str),
            )
        }
        OverlayAnchor::VirtualPoint {
            x,
            y,
            width,
            height,
            ..
        } => {
            let dims = platform.get_dimensions(&content_str).ok()?;
            (
                Rect::new(x, y, width, height),
                Size::new(dims.width as f32, dims.height as f32),
                None,
            )
        }
    };

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
    let reference = match (&anchor_id_owned, config.anchor) {
        (Some(anchor_str), _) => ElementOrVirtual::Element(anchor_str),
        (
            None,
            OverlayAnchor::VirtualPoint {
                x,
                y,
                width,
                height,
                context_element_id,
            },
        ) => {
            let virtual_rect =
                FloatingRect::new(x as f64, y as f64, width as f64, height as f64);
            let mut virtual_el =
                DefaultVirtualElement::new(Rc::new(move || ClientRectObject::from(virtual_rect)));
            if let Some(ctx_id) = context_element_id {
                virtual_el = virtual_el.context_element(ctx_id.to_string());
            }
            ElementOrVirtual::VirtualElement(Box::new(virtual_el))
        }
        (None, OverlayAnchor::Element(_)) => unreachable!(),
    };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_overlay_position_returns_none_without_active_platform() {
        let config = VirtualOverlayPositionConfig {
            anchor: OverlayAnchor::VirtualPoint {
                x: 120.0,
                y: 80.0,
                width: 0.0,
                height: 0.0,
                context_element_id: None,
            },
            content_id: "menu-content",
            arrow_id: None,
            boundary_id: None,
            side: PlacementSide::Bottom,
            align: PlacementAlign::Start,
            side_offset: 4.0,
            align_offset: 0.0,
            avoid_collisions: true,
            collision_padding: 8.0,
            arrow_padding: 0.0,
            hide_when_detached: true,
            sticky: false,
            namespace: Some("context-menu"),
        };
        assert_eq!(compute_virtual_overlay_position(&config), None);
    }

    #[cfg(feature = "native")]
    #[test]
    fn native_virtual_overlay_position_supports_hide_with_and_without_context_element() {
        use blitz_dom::{Attribute, BaseDocument, DocumentConfig, LocalName, QualName, ns};
        use oxidase::dom::Document;
        use std::cell::RefCell;
        use std::rc::Rc;

        let qname = |tag: &'static str| QualName::new(None, ns!(html), LocalName::from(tag));
        let attr = |name: &'static str, val: &str| Attribute {
            name: QualName::new(None, ns!(), LocalName::from(name)),
            value: val.to_string(),
        };

        let mut base_doc_raw = BaseDocument::new(DocumentConfig::default());
        {
            let mut vp = base_doc_raw.viewport_mut();
            vp.window_size = (800, 600);
            vp.hidpi_scale = 1.0;
        }
        let base_doc = Rc::new(RefCell::new(base_doc_raw));
        let doc = Document::from_base(base_doc.clone());

        {
            let mut base = base_doc.borrow_mut();
            let root_id = base.root_node().id;
            let mut mutator = base.mutate();
            let html = mutator.create_element(qname("html"), vec![]);
            let body = mutator.create_element(
                qname("body"),
                vec![attr("style", "margin: 0; padding: 0;")],
            );
            let ctx_trigger = mutator.create_element(
                qname("div"),
                vec![
                    attr("id", "ctx-trigger"),
                    attr("style", "width: 200px; height: 100px; display: block;"),
                ],
            );
            let content = mutator.create_element(
                qname("div"),
                vec![
                    attr("id", "ctx-content"),
                    attr(
                        "style",
                        "position: fixed; width: 120px; height: 80px; display: block;",
                    ),
                ],
            );
            mutator.append_children(body, &[ctx_trigger, content]);
            mutator.append_children(html, &[body]);
            mutator.append_children(root_id, &[html]);
        }
        doc.resolve(0.0);

        Document::with_current(doc, || {
            // 1. VirtualPoint with context_element_id + hide_when_detached: true
            let with_ctx = compute_virtual_overlay_position(&VirtualOverlayPositionConfig {
                anchor: OverlayAnchor::VirtualPoint {
                    x: 150.0,
                    y: 90.0,
                    width: 0.0,
                    height: 0.0,
                    context_element_id: Some("ctx-trigger"),
                },
                content_id: "ctx-content",
                arrow_id: None,
                boundary_id: None,
                side: PlacementSide::Bottom,
                align: PlacementAlign::Start,
                side_offset: 6.0,
                align_offset: 0.0,
                avoid_collisions: true,
                collision_padding: 8.0,
                arrow_padding: 0.0,
                hide_when_detached: true,
                sticky: false,
                namespace: Some("context-menu"),
            })
            .expect("virtual placement with context_element_id must succeed");

            assert_eq!(with_ctx.side(), PlacementSide::Bottom);
            assert_eq!(with_ctx.align(), PlacementAlign::Start);
            assert!((with_ctx.geometry().x() - 150.0).abs() < 1.0);
            assert!((with_ctx.geometry().y() - 96.0).abs() < 1.0);
            assert!(!with_ctx.reference_hidden());

            // 2. VirtualPoint without context_element_id + hide_when_detached: true (must NOT panic!)
            let without_ctx = compute_virtual_overlay_position(&VirtualOverlayPositionConfig {
                anchor: OverlayAnchor::VirtualPoint {
                    x: 200.0,
                    y: 120.0,
                    width: 0.0,
                    height: 0.0,
                    context_element_id: None,
                },
                content_id: "ctx-content",
                arrow_id: None,
                boundary_id: None,
                side: PlacementSide::Right,
                align: PlacementAlign::Start,
                side_offset: 4.0,
                align_offset: 0.0,
                avoid_collisions: true,
                collision_padding: 8.0,
                arrow_padding: 0.0,
                hide_when_detached: true,
                sticky: false,
                namespace: Some("context-menu"),
            })
            .expect("virtual placement without context_element_id must not panic");

            assert_eq!(without_ctx.side(), PlacementSide::Right);
            assert!((without_ctx.geometry().x() - 204.0).abs() < 1.0);
            assert!((without_ctx.geometry().y() - 120.0).abs() < 1.0);
            assert!(!without_ctx.reference_hidden());
        });
    }
}

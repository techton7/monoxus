use dioxus::prelude::*;

pub use monoxus_core::foundation::overlay::floating::*;

use crate::foundation::{
    browser::{
        FloatingAutoUpdateEvent, computed_z_index_for_element, get_element_computed_z_index,
        get_viewport_size, spawn_root_task, start_floating_auto_update_monitor,
    },
    compose::MountedHandle,
    shared::Direction,
    state::sync_signal,
};

use super::watcher::{WatcherSlot, use_watcher_slot};

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

/// Returns the viewport origin `(x, y)` of the innermost transformed ancestor
/// `[data-monoxus-floating-content-wrapper]` containing `content_id`'s wrapper, if any.
///
/// Under CSS Transforms Level 1, any ancestor with `transform != none` establishes the
/// containing block for `position: fixed` descendants. When a nested floating overlay
/// remains a DOM descendant of an outer transformed floating wrapper (for example,
/// non-reparented Native portals or inline portals), subtracting this containing-block
/// origin converts viewport-space coordinates into wrapper-local `translate(x, y)` coords.
fn fixed_containing_wrapper_offset(content_id: &str) -> (f32, f32) {
    let Some(doc) = oxidase::dom::Document::current() else {
        return (0.0, 0.0);
    };
    let Some(el) = doc.element_by_id(content_id) else {
        return (0.0, 0.0);
    };
    let own_wrapper = el.closest("[data-monoxus-floating-content-wrapper]");
    let own_wrapper_id = own_wrapper.as_ref().and_then(|w| w.id());
    let target = own_wrapper.as_ref().unwrap_or(&el);

    let Some(root) = doc
        .query_selector("body")
        .or_else(|| doc.query_selector("html"))
    else {
        return (0.0, 0.0);
    };

    let mut nearest_ancestor_id: Option<String> = None;
    for candidate in root.query_selector_all("[data-monoxus-floating-content-wrapper]") {
        let Some(cand_id) = candidate.id() else {
            continue;
        };
        if own_wrapper_id.as_deref() == Some(cand_id.as_str()) {
            continue;
        }
        if candidate.get_attribute("data-positioning").as_deref() != Some("positioned") {
            continue;
        }
        if candidate.contains(target) {
            nearest_ancestor_id = Some(cand_id);
        }
    }

    if let Some(anc_id) = nearest_ancestor_id
        && let Some(rect) = doc.measure_rect(&anc_id)
    {
        return (rect.x as f32, rect.y as f32);
    }
    (0.0, 0.0)
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
        let padding = config.collision_padding.max(0.0);
        let mut overflow_opts =
            oxidase_floating_ui::detect_overflow::DetectOverflowOptions::default()
                .padding(oxidase_floating_ui::geometry::Padding::All(padding as f64));
        if let Some(bid) = config.boundary_id {
            overflow_opts = overflow_opts
                .boundary(oxidase_floating_ui::types::Boundary::Element(bid.to_string()));
        } else {
            overflow_opts =
                overflow_opts.boundary(oxidase_floating_ui::types::Boundary::ClippingAncestors);
        }

        middlewares.push(Box::new(oxidase_floating_ui::middleware::Flip::new(
            oxidase_floating_ui::middleware::FlipOptions::default()
                .detect_overflow(overflow_opts.clone()),
        )));

        middlewares.push(Box::new(oxidase_floating_ui::middleware::Shift::new(
            oxidase_floating_ui::middleware::ShiftOptions::default()
                .detect_overflow(overflow_opts)
                .cross_axis(true),
        )));
    }

    if let Some(aid) = config.arrow_id {
        middlewares.push(Box::new(oxidase_floating_ui::middleware::Arrow::new(
            oxidase_floating_ui::middleware::ArrowOptions::new(aid.to_string())
                .padding(oxidase_floating_ui::geometry::Padding::All(config.arrow_padding as f64)),
        )));
    }

    // `oxidase-floating-ui v0.4.7` resolves `VirtualPoint` + `Hide` safely even when `context_element_id` is `None`.
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

    let mut x = result.x as f32;
    let mut y = result.y as f32;
    let pad = config.collision_padding.max(0.0);

    if config.avoid_collisions && clip_rect.width > 0.0 && clip_rect.height > 0.0 {
        let min_x = clip_rect.x as f32 + pad;
        let max_x = ((clip_rect.x + clip_rect.width) as f32 - content_size.width() - pad).max(min_x);
        let min_y = clip_rect.y as f32 + pad;
        let max_y = ((clip_rect.y + clip_rect.height) as f32 - content_size.height() - pad).max(min_y);
        x = x.clamp(min_x, max_x);
        y = y.clamp(min_y, max_y);
    }

    let available_size = if clip_rect.width > 0.0 && clip_rect.height > 0.0 {
        let clip_left = clip_rect.x as f32 + pad;
        let clip_top = clip_rect.y as f32 + pad;
        let clip_right = ((clip_rect.x + clip_rect.width) as f32 - pad).max(clip_left);
        let clip_bottom = ((clip_rect.y + clip_rect.height) as f32 - pad).max(clip_top);
        let avail_w = match side {
            PlacementSide::Left => (anchor_rect.x() - config.side_offset - clip_left).max(0.0),
            PlacementSide::Right => {
                (clip_right - (anchor_rect.right() + config.side_offset)).max(0.0)
            }
            PlacementSide::Top | PlacementSide::Bottom => (clip_right - clip_left).max(0.0),
        };
        let avail_h = match side {
            PlacementSide::Top => (anchor_rect.y() - config.side_offset - clip_top).max(0.0),
            PlacementSide::Bottom => {
                (clip_bottom - (anchor_rect.bottom() + config.side_offset)).max(0.0)
            }
            PlacementSide::Left | PlacementSide::Right => (clip_bottom - clip_top).max(0.0),
        };
        Size::new(avail_w, avail_h)
    } else {
        Size::new(clip_rect.width as f32, clip_rect.height as f32)
    };

    let direction = if platform.is_rtl(&content_str) {
        Direction::Rtl
    } else {
        Direction::Ltr
    };
    let resolved_inline_align = resolve_inline_align(align, direction);

    let transform_origin_x = match side {
        PlacementSide::Right => 0.0,
        PlacementSide::Left => content_size.width(),
        PlacementSide::Top | PlacementSide::Bottom => clamp_to_extent(
            aligned_horizontal_point(anchor_rect, resolved_inline_align) - x,
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

    let (cb_x, cb_y) = fixed_containing_wrapper_offset(&content_str);
    let ns = config.namespace.unwrap_or(GeometryVars::DEFAULT_NAMESPACE);
    let geometry = GeometryVars::new(
        ns.to_string(),
        x - cb_x,
        y - cb_y,
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

/// Scroll reaction policy for [`use_floating_layer`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FloatingScrollPolicy {
    /// Reposition continuously on ancestor scroll (Popover, Select, Combobox, Menu, HoverCard).
    #[default]
    Reposition,
    /// Close on scroll unless a 1-frame focus-scroll guard is armed (Tooltip `FM-OVL-7`).
    CloseOnScrollUnlessGuarded,
}

/// Owned specification passed to [`use_floating_layer`] driving live sync, 1-frame, and async
/// placement measurement plus z-index hoisting and auto-update monitoring.
#[derive(Clone, Debug, PartialEq)]
pub struct FloatingMonitorSpec {
    pub anchor_id: String,
    pub virtual_anchor: Option<(f32, f32, f32, f32, Option<String>)>,
    pub content_id: String,
    pub measurement_id: Option<String>,
    pub arrow_id: Option<String>,
    pub boundary_id: Option<String>,
    pub side: PlacementSide,
    pub align: PlacementAlign,
    pub direction: Direction,
    pub side_offset: f32,
    pub align_offset: f32,
    pub avoid_collisions: bool,
    pub collision_padding: f32,
    pub arrow_padding: f32,
    pub hide_when_detached: bool,
    pub sticky: bool,
    pub namespace: Option<String>,
    pub scroll_policy: FloatingScrollPolicy,
}

impl FloatingMonitorSpec {
    pub fn new(anchor_id: impl Into<String>, content_id: impl Into<String>) -> Self {
        Self {
            anchor_id: anchor_id.into(),
            virtual_anchor: None,
            content_id: content_id.into(),
            measurement_id: None,
            arrow_id: None,
            boundary_id: None,
            side: PlacementSide::Bottom,
            align: PlacementAlign::Center,
            direction: Direction::Ltr,
            side_offset: 0.0,
            align_offset: 0.0,
            avoid_collisions: true,
            collision_padding: 0.0,
            arrow_padding: 0.0,
            hide_when_detached: false,
            sticky: false,
            namespace: None,
            scroll_policy: FloatingScrollPolicy::Reposition,
        }
    }

    pub fn effective_measurement_id(&self) -> &str {
        self.measurement_id
            .as_deref()
            .filter(|id| !id.is_empty())
            .unwrap_or(self.content_id.as_str())
    }
}

/// Zero-overhead `Copy` handle returned by [`use_floating_layer`] encapsulating live placement,
/// readiness, z-index hoisting (with `computed_z_index_for_element` fallback), `arrow_id`
/// registration (`B11`), focus-scroll guard (`FM-OVL-7`), and async `MountedHandle` fallback.
#[derive(Clone, Copy, PartialEq)]
pub struct FloatingHandle {
    placement: Signal<Option<FloatingPlacement>>,
    readiness: Signal<FloatingReadiness>,
    content_z_index: Signal<Option<String>>,
    anchor_handle: Signal<Option<MountedHandle>>,
    content_handle: Signal<Option<MountedHandle>>,
    arrow_id: Signal<Option<String>>,
    focus_scroll_guard: CopyValue<bool>,
    watcher: WatcherSlot,
    spec: CopyValue<FloatingMonitorSpec>,
    on_scroll_close: CopyValue<Option<EventHandler<()>>>,
    alive: CopyValue<bool>,
}

impl FloatingHandle {
    pub fn placement(&self) -> Option<FloatingPlacement> {
        self.placement.cloned()
    }

    pub fn readiness(&self) -> FloatingReadiness {
        *self.readiness.read()
    }

    pub fn content_z_index(&self) -> Option<String> {
        self.content_z_index.cloned()
    }

    pub fn wrapper_style(&self) -> String {
        let z_index = self.content_z_index.read();
        match self.placement() {
            Some(ref placement) => placement.wrapper_style(self.readiness(), z_index.as_deref()),
            None => FloatingReadiness::wrapper_measuring_style(z_index.as_deref()),
        }
    }

    /// Serializes `--monoxus-<ns>-*` variables (plus optional Radix compatibility aliases)
    /// via `monoxus_core::foundation::overlay::floating::serialize_css_custom_properties`.
    pub fn css_custom_properties(&self, compat_alias: Option<(&str, &str)>) -> String {
        let Some(placement) = self.placement() else {
            return String::new();
        };
        let geom = placement.geometry();
        let vars: Vec<(String, String)> = match compat_alias {
            Some((prefix, anchor_label)) => geom
                .css_iter()
                .chain(geom.compatibility_alias_iter(prefix, anchor_label))
                .collect(),
            None => geom.css_iter().collect(),
        };
        serialize_css_custom_properties(&vars)
    }

    pub fn arrow_id(&self) -> Option<String> {
        self.arrow_id.cloned()
    }

    pub fn register_arrow_id(&self, id: impl Into<String>) {
        let id = id.into();
        if id.is_empty() {
            return;
        }
        let mut sig = self.arrow_id;
        sync_signal(&mut sig, Some(id));
    }

    pub fn unregister_arrow_id(&self, id: &str) {
        let mut sig = self.arrow_id;
        if sig.try_peek().is_ok_and(|cur| cur.as_deref() == Some(id))
            && let Ok(mut w) = sig.try_write()
        {
            *w = None;
        }
    }

    pub fn capture_anchor(&self, handle: MountedHandle) {
        let mut sig = self.anchor_handle;
        if let Ok(mut w) = sig.try_write() {
            *w = Some(handle);
        }
        self.refresh();
    }

    pub fn capture_content(&self, handle: MountedHandle) {
        let mut sig = self.content_handle;
        if let Ok(mut w) = sig.try_write() {
            *w = Some(handle);
        }
        self.sync_z_index();
        self.refresh();
    }

    /// Arms a 1-frame scroll-close guard so focus-triggered scrolls reposition rather than close (`FM-OVL-7`).
    pub fn arm_focus_scroll_guard(&self) {
        let mut guard = self.focus_scroll_guard;
        if let Ok(mut w) = guard.try_write() {
            *w = true;
        }
        let alive = self.alive;
        spawn_root_task(async move {
            let _ = oxidase::frame::next_frame().await;
            if alive.try_read().is_ok_and(|a| *a)
                && let Ok(mut w) = guard.try_write()
            {
                *w = false;
            }
        });
    }

    pub fn sync_z_index(&self) {
        let Ok(spec) = self.spec.try_read() else {
            return;
        };
        let measurement_id = spec.effective_measurement_id().to_owned();
        let content_id = spec.content_id.clone();
        drop(spec);

        let next =
            normalize_hoisted_z_index(get_element_computed_z_index(&measurement_id).as_deref())
                .or_else(|| {
                    (content_id != measurement_id)
                        .then(|| {
                            normalize_hoisted_z_index(
                                get_element_computed_z_index(&content_id).as_deref(),
                            )
                        })
                        .flatten()
                })
                .or_else(|| {
                    let handle = self
                        .content_handle
                        .try_peek()
                        .ok()
                        .and_then(|h| h.clone())?;
                    let el = oxidase::Element::from_mounted(handle);
                    normalize_hoisted_z_index(computed_z_index_for_element(&el).as_deref())
                });

        let mut sig = self.content_z_index;
        sync_signal(&mut sig, next);
    }

    pub fn clear_z_index(&self) {
        let mut sig = self.content_z_index;
        sync_signal(&mut sig, None);
    }

    /// Computes live placement synchronously via `compute_virtual_overlay_position`.
    /// Returns `true` when positive content dimensions were measured and placement is ready.
    pub fn recalculate_sync(&self) -> bool {
        let Ok(spec) = self.spec.try_read().map(|g| g.clone()) else {
            return false;
        };
        let registered_arrow = self.arrow_id.try_peek().ok().and_then(|a| a.clone());
        let effective_arrow = registered_arrow.or_else(|| spec.arrow_id.clone());

        let anchor = match &spec.virtual_anchor {
            Some((x, y, width, height, ctx_id)) => OverlayAnchor::VirtualPoint {
                x: *x,
                y: *y,
                width: *width,
                height: *height,
                context_element_id: ctx_id.as_deref(),
            },
            None => OverlayAnchor::Element(spec.anchor_id.as_str()),
        };

        let config = VirtualOverlayPositionConfig {
            anchor,
            content_id: spec.effective_measurement_id(),
            arrow_id: effective_arrow.as_deref(),
            boundary_id: spec.boundary_id.as_deref(),
            side: spec.side,
            align: spec.align,
            side_offset: spec.side_offset,
            align_offset: spec.align_offset,
            avoid_collisions: spec.avoid_collisions,
            collision_padding: spec.collision_padding,
            arrow_padding: spec.arrow_padding,
            hide_when_detached: spec.hide_when_detached,
            sticky: spec.sticky,
            namespace: spec.namespace.as_deref(),
        };

        self.sync_z_index();

        let Some(computed) = compute_virtual_overlay_position(&config) else {
            return false;
        };

        let geom = computed.geometry();
        let next_readiness = if geom.content_width() > 0.0 && geom.content_height() > 0.0 {
            FloatingReadiness::Ready
        } else {
            FloatingReadiness::Measuring
        };
        let is_ready = next_readiness == FloatingReadiness::Ready;

        let mut readiness_sig = self.readiness;
        sync_signal(&mut readiness_sig, next_readiness);

        let mut placement_sig = self.placement;
        sync_signal(&mut placement_sig, Some(computed));

        is_ready
    }

    /// Runs [`Self::recalculate_sync`] and falls back to async `MountedHandle::get_client_rect`
    /// measurement when synchronous DOM measurement is unavailable.
    pub async fn recalculate_with_async_fallback(&self) {
        if self.recalculate_sync() {
            return;
        }
        let anchor_h = self.anchor_handle.try_peek().ok().and_then(|h| h.clone());
        let content_h = self.content_handle.try_peek().ok().and_then(|h| h.clone());
        let (Some(ah), Some(ch)) = (anchor_h, content_h) else {
            return;
        };
        let (Ok(anchor_rect), Ok(content_rect)) =
            (ah.get_client_rect().await, ch.get_client_rect().await)
        else {
            return;
        };
        if !self.alive.try_read().is_ok_and(|a| *a) {
            return;
        }
        let Ok(spec) = self.spec.try_read().map(|g| g.clone()) else {
            return;
        };

        let anchor = Rect::new(
            anchor_rect.origin.x as f32,
            anchor_rect.origin.y as f32,
            anchor_rect.width() as f32,
            anchor_rect.height() as f32,
        );
        let content_size = Size::new(content_rect.width() as f32, content_rect.height() as f32);
        let avail_size = get_viewport_size()
            .await
            .ok()
            .map(|vp| Size::new(vp[0] as f32, vp[1] as f32));

        let mut layer = FloatingLayer::new(spec.side)
            .with_align(spec.align)
            .with_direction(spec.direction)
            .with_side_offset(spec.side_offset)
            .with_align_offset(spec.align_offset)
            .with_avoid_collisions(spec.avoid_collisions)
            .with_collision_padding(spec.collision_padding)
            .with_arrow_padding(spec.arrow_padding)
            .with_sticky(if spec.sticky {
                StickyStrategy::Always
            } else {
                StickyStrategy::Partial
            })
            .with_hide_when_detached(spec.hide_when_detached);
        if let Some(ns) = spec.namespace {
            layer = layer.with_namespace(ns);
        }

        let placement = match avail_size {
            Some(avail) => layer
                .with_available_space(avail)
                .position_with_available_size(anchor, content_size, avail),
            None => layer.position(anchor, content_size),
        };

        self.sync_z_index();

        let geom = placement.geometry();
        let next_readiness = if geom.content_width() > 0.0 && geom.content_height() > 0.0 {
            FloatingReadiness::Ready
        } else {
            FloatingReadiness::Measuring
        };
        let mut readiness_sig = self.readiness;
        sync_signal(&mut readiness_sig, next_readiness);

        let mut placement_sig = self.placement;
        sync_signal(&mut placement_sig, Some(placement));
    }

    /// Triggers an immediate sync placement pass plus a 1-frame deferred measurement pass.
    pub fn refresh(&self) {
        let _ = self.recalculate_sync();
        let handle = *self;
        let expected_gen = self.watcher.generation();
        spawn_root_task(async move {
            let _ = oxidase::frame::next_frame().await;
            if handle.alive.try_read().is_ok_and(|a| *a)
                && handle.watcher.generation() == expected_gen
            {
                handle.recalculate_with_async_fallback().await;
            }
        });
    }

    /// Stops live auto-update watching and resets placement, z-index, and readiness state.
    pub fn clear(&self) {
        self.watcher.clear();
        let mut content_h = self.content_handle;
        if content_h.try_peek().is_ok_and(|h| h.is_some())
            && let Ok(mut w) = content_h.try_write()
        {
            *w = None;
        }
        let mut placement_sig = self.placement;
        sync_signal(&mut placement_sig, None);
        self.clear_z_index();
        let mut readiness_sig = self.readiness;
        sync_signal(&mut readiness_sig, FloatingReadiness::Measuring);
    }
}

/// Internal context provided by [`use_floating_layer`] so child portal teleportation hooks
/// can immediately recompute placement after reparenting out of a transformed ancestor wrapper.
#[derive(Clone, Copy)]
pub(crate) struct PortalTeleportRecalcContext(pub(crate) FloatingHandle);

/// Shared zero-overhead floating placement runtime hook unifying sync measurement, 1-frame
/// deferred layout refinement, async `MountedHandle` fallback, z-index hoisting, arrow ID
/// tracking, and scroll policy handling.
pub fn use_floating_layer(
    active: bool,
    spec: FloatingMonitorSpec,
    on_scroll_close: Option<EventHandler<()>>,
) -> FloatingHandle {
    let placement = use_signal(|| None);
    let readiness = use_signal(FloatingReadiness::default);
    let content_z_index = use_signal(|| None);
    let anchor_handle = use_signal(|| None);
    let content_handle = use_signal(|| None);
    let arrow_id = use_signal(|| spec.arrow_id.clone());
    let focus_scroll_guard = use_hook(|| CopyValue::new(false));
    let watcher = use_watcher_slot();
    let mut spec_slot = use_hook(|| CopyValue::new(spec.clone()));
    let mut on_scroll_close_slot = use_hook(|| CopyValue::new(on_scroll_close));
    let mut alive = use_hook(|| CopyValue::new(true));

    spec_slot.set(spec.clone());
    on_scroll_close_slot.set(on_scroll_close);

    let handle = FloatingHandle {
        placement,
        readiness,
        content_z_index,
        anchor_handle,
        content_handle,
        arrow_id,
        focus_scroll_guard,
        watcher,
        spec: spec_slot,
        on_scroll_close: on_scroll_close_slot,
        alive,
    };

    use_context_provider(|| PortalTeleportRecalcContext(handle));

    use_effect(use_reactive((&active, &spec), move |(is_active, cur_spec)| {
        let _ = handle.arrow_id.read();
        if !is_active {
            handle.watcher.clear();
            return;
        }

        handle.refresh();

        let anchor_ids: Vec<String> = match &cur_spec.virtual_anchor {
            Some((_, _, _, _, Some(ctx_id))) if !ctx_id.is_empty() => vec![ctx_id.clone()],
            Some(_) => Vec::new(),
            None if !cur_spec.anchor_id.is_empty() => vec![cur_spec.anchor_id.clone()],
            None => Vec::new(),
        };
        let content_id = cur_spec.effective_measurement_id().to_owned();
        let scroll_policy = cur_spec.scroll_policy;

        handle.watcher.arm_with(move |expected_gen| {
            let anchor_refs: Vec<&str> = anchor_ids.iter().map(String::as_str).collect();
            start_floating_auto_update_monitor(&anchor_refs, &content_id, move |event| {
                if !handle.alive.try_read().is_ok_and(|a| *a)
                    || handle.watcher.generation() != expected_gen
                {
                    return;
                }
                match event {
                    FloatingAutoUpdateEvent::Scroll => match scroll_policy {
                        FloatingScrollPolicy::Reposition => {
                            let _ = handle.recalculate_sync();
                        }
                        FloatingScrollPolicy::CloseOnScrollUnlessGuarded => {
                            let guarded = handle
                                .focus_scroll_guard
                                .try_read()
                                .map(|g| *g)
                                .unwrap_or(false);
                            if guarded {
                                let _ = handle.recalculate_sync();
                            } else if let Some(cb) =
                                handle.on_scroll_close.try_read().ok().and_then(|s| *s)
                            {
                                cb.call(());
                            }
                        }
                    },
                    FloatingAutoUpdateEvent::Update => {
                        let _ = handle.recalculate_sync();
                    }
                }
            })
        });
    }));

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        handle.watcher.clear();
        handle.clear_z_index();
    });

    handle
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

    #[cfg(feature = "native")]
    #[test]
    fn native_virtual_overlay_position_resolves_rtl_transform_origin_x() {
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
            let anchor = mutator.create_element(
                qname("div"),
                vec![
                    attr("id", "rtl-anchor"),
                    attr(
                        "style",
                        "position: fixed; left: 300px; top: 50px; width: 80px; height: 30px; display: block;",
                    ),
                ],
            );
            let content = mutator.create_element(
                qname("div"),
                vec![
                    attr("id", "rtl-content"),
                    attr("dir", "rtl"),
                    attr(
                        "style",
                        "direction: rtl; position: fixed; width: 200px; height: 60px; display: block;",
                    ),
                ],
            );
            mutator.append_children(body, &[anchor, content]);
            mutator.append_children(html, &[body]);
            mutator.append_children(root_id, &[html]);
        }
        doc.resolve(0.0);

        Document::with_current(doc, || {
            let start_rtl = compute_overlay_position(&OverlayPositionConfig {
                anchor_id: "rtl-anchor",
                content_id: "rtl-content",
                arrow_id: None,
                boundary_id: None,
                side: PlacementSide::Bottom,
                align: PlacementAlign::Start,
                side_offset: 4.0,
                align_offset: 0.0,
                avoid_collisions: false,
                collision_padding: 0.0,
                arrow_padding: 0.0,
                hide_when_detached: false,
                sticky: false,
                namespace: Some("hover-card"),
            })
            .expect("RTL BottomStart placement must succeed");

            // Under RTL, Start aligns to anchor right edge (380) => content x=180, transform_origin_x=200.
            assert!(
                (start_rtl.geometry().transform_origin_x() - 200.0).abs() < 1.0,
                "Expected RTL Start transform_origin_x ~ 200.0, got {}",
                start_rtl.geometry().transform_origin_x()
            );

            let end_rtl = compute_overlay_position(&OverlayPositionConfig {
                anchor_id: "rtl-anchor",
                content_id: "rtl-content",
                arrow_id: None,
                boundary_id: None,
                side: PlacementSide::Bottom,
                align: PlacementAlign::End,
                side_offset: 4.0,
                align_offset: 0.0,
                avoid_collisions: false,
                collision_padding: 0.0,
                arrow_padding: 0.0,
                hide_when_detached: false,
                sticky: false,
                namespace: Some("hover-card"),
            })
            .expect("RTL BottomEnd placement must succeed");

            // Under RTL, End aligns to anchor left edge (300) => content x=300, transform_origin_x=0.
            assert!(
                (end_rtl.geometry().transform_origin_x() - 0.0).abs() < 1.0,
                "Expected RTL End transform_origin_x ~ 0.0, got {}",
                end_rtl.geometry().transform_origin_x()
            );
        });
    }
}

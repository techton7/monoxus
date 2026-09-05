use std::borrow::Cow;

use super::shared::Direction;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PortalHost {
    Default,
    Inline,
    Named(Cow<'static, str>),
}

impl PortalHost {
    pub fn named(id: impl Into<Cow<'static, str>>) -> Self {
        Self::Named(id.into())
    }

    pub const fn inline() -> Self {
        Self::Inline
    }

    pub const fn default_host() -> Self {
        Self::Default
    }

    pub fn resolve(preferred: Option<Self>, inherited: Option<&Self>) -> Self {
        preferred.or_else(|| inherited.cloned()).unwrap_or_default()
    }

    pub const fn is_inline(&self) -> bool {
        matches!(self, Self::Inline)
    }

    pub const fn is_default_host(&self) -> bool {
        matches!(self, Self::Default)
    }

    pub const fn is_portalled(&self) -> bool {
        !self.is_inline()
    }

    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Default | Self::Inline => None,
            Self::Named(id) => Some(id.as_ref()),
        }
    }
}

impl Default for PortalHost {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresenceState {
    Unmounted,
    Mounted,
    Suspended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Presence {
    desired_present: bool,
    retain_mount: bool,
    state: PresenceState,
}

impl Presence {
    pub const fn new(desired_present: bool) -> Self {
        Self {
            desired_present,
            retain_mount: false,
            state: if desired_present {
                PresenceState::Mounted
            } else {
                PresenceState::Unmounted
            },
        }
    }

    pub const fn with_retained_mount(mut self, retain_mount: bool) -> Self {
        self.retain_mount = retain_mount;
        self
    }

    pub const fn desired_present(&self) -> bool {
        self.desired_present
    }

    pub const fn retain_mount(&self) -> bool {
        self.retain_mount
    }

    pub const fn state(&self) -> PresenceState {
        self.state
    }

    pub const fn is_mounted(&self) -> bool {
        !matches!(self.state, PresenceState::Unmounted)
    }

    pub fn sync(&mut self, desired_present: bool) -> PresenceState {
        self.desired_present = desired_present;
        self.state = match desired_present {
            true => PresenceState::Mounted,
            false if self.retain_mount && self.is_mounted() => PresenceState::Suspended,
            false => PresenceState::Unmounted,
        };
        self.state
    }

    pub fn complete_unmount(&mut self) -> bool {
        if self.desired_present || self.state != PresenceState::Suspended {
            return false;
        }

        self.state = PresenceState::Unmounted;
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusScope<T> {
    root: T,
    branches: Vec<T>,
    restore_focus_to: Option<T>,
    autofocus_target: Option<T>,
    autofocus_enabled: bool,
    last_focused: Option<T>,
    trap_focus: bool,
    loop_focus: bool,
    active: bool,
    parent_paused: bool,
}

impl<T> FocusScope<T>
where
    T: Clone + PartialEq,
{
    pub fn new(root: T) -> Self {
        Self {
            root,
            branches: Vec::new(),
            restore_focus_to: None,
            autofocus_target: None,
            autofocus_enabled: true,
            last_focused: None,
            trap_focus: false,
            loop_focus: false,
            active: false,
            parent_paused: false,
        }
    }

    pub fn root(&self) -> &T {
        &self.root
    }

    pub fn branches(&self) -> &[T] {
        &self.branches
    }

    pub fn with_trap_focus(mut self, trap_focus: bool) -> Self {
        self.trap_focus = trap_focus;
        self
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub fn set_trap_focus(&mut self, trap_focus: bool) {
        self.trap_focus = trap_focus;
    }

    pub fn set_loop_focus(&mut self, loop_focus: bool) {
        self.loop_focus = loop_focus;
    }

    pub fn traps_focus(&self) -> bool {
        self.trap_focus
    }

    pub fn loops_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_parent_paused(&self) -> bool {
        self.parent_paused
    }

    pub fn set_autofocus_enabled(&mut self, enabled: bool) {
        self.autofocus_enabled = enabled;
    }

    pub fn autofocus_enabled(&self) -> bool {
        self.autofocus_enabled
    }

    pub fn set_autofocus_target(&mut self, target: Option<T>) {
        self.autofocus_target = target;
    }

    pub fn autofocus_target(&self) -> Option<&T> {
        self.autofocus_target.as_ref()
    }

    pub fn capture_restore_target(&mut self, target: Option<T>) {
        self.restore_focus_to = target;
    }

    pub fn restore_target(&self) -> Option<&T> {
        self.restore_focus_to.as_ref()
    }

    pub fn restore_focus(&self) -> Option<T> {
        self.restore_focus_to
            .clone()
            .or_else(|| self.last_focused.clone())
    }

    pub fn last_focused(&self) -> Option<&T> {
        self.last_focused.as_ref()
    }

    pub fn register_branch(&mut self, branch: T) -> bool {
        if self.root == branch || self.branches.iter().any(|candidate| candidate == &branch) {
            return false;
        }

        self.branches.push(branch);
        true
    }

    pub fn unregister_branch(&mut self, branch: &T) -> bool {
        let Some(position) = self
            .branches
            .iter()
            .position(|candidate| candidate == branch)
        else {
            return false;
        };

        self.branches.remove(position);
        true
    }

    pub fn contains(&self, node: &T) -> bool {
        &self.root == node || self.branches.iter().any(|branch| branch == node)
    }

    pub fn activate(&mut self) -> Option<T> {
        self.active = true;
        self.parent_paused = self.trap_focus;

        if !self.autofocus_enabled {
            return None;
        }

        let target = self
            .autofocus_target
            .clone()
            .filter(|node| self.contains(node))
            .or_else(|| self.last_focused.clone())
            .or_else(|| Some(self.root.clone()));

        if let Some(target) = target.clone() {
            self.last_focused = Some(target);
        }

        target
    }

    pub fn deactivate(&mut self) -> Option<T> {
        self.active = false;
        self.parent_paused = false;
        self.restore_focus()
    }

    pub fn focus(&mut self, node: T) -> bool {
        if !self.contains(&node) {
            return false;
        }

        self.last_focused = Some(node);
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FocusGuardSide {
    Before,
    After,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusGuards<T> {
    before: T,
    after: T,
    retain_count: usize,
}

impl<T> FocusGuards<T>
where
    T: PartialEq,
{
    pub const fn new(before: T, after: T) -> Self {
        Self {
            before,
            after,
            retain_count: 0,
        }
    }

    pub fn before(&self) -> &T {
        &self.before
    }

    pub fn after(&self) -> &T {
        &self.after
    }

    pub fn retain_count(&self) -> usize {
        self.retain_count
    }

    pub fn is_installed(&self) -> bool {
        self.retain_count > 0
    }

    pub fn retain(&mut self) -> usize {
        self.retain_count = self.retain_count.saturating_add(1);
        self.retain_count
    }

    pub fn release(&mut self) -> usize {
        self.retain_count = self.retain_count.saturating_sub(1);
        self.retain_count
    }

    pub fn contains(&self, node: &T) -> bool {
        &self.before == node || &self.after == node
    }

    pub fn side_of(&self, node: &T) -> Option<FocusGuardSide> {
        if &self.before == node {
            Some(FocusGuardSide::Before)
        } else if &self.after == node {
            Some(FocusGuardSide::After)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DismissEvent {
    Escape,
    PointerDownOutside,
    FocusOutside,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DismissLayer<T> {
    id: T,
    branches: Vec<T>,
    modal: bool,
    dismiss_on_escape: bool,
    dismiss_on_pointer_down_outside: bool,
    dismiss_on_focus_outside: bool,
}

impl<T> DismissLayer<T>
where
    T: PartialEq,
{
    pub fn new(id: T) -> Self {
        Self {
            id,
            branches: Vec::new(),
            modal: false,
            dismiss_on_escape: true,
            dismiss_on_pointer_down_outside: true,
            dismiss_on_focus_outside: true,
        }
    }

    pub fn id(&self) -> &T {
        &self.id
    }

    pub fn branches(&self) -> &[T] {
        &self.branches
    }

    pub fn with_modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    pub fn set_modal(&mut self, modal: bool) {
        self.modal = modal;
    }

    pub fn with_escape_dismiss(mut self, enabled: bool) -> Self {
        self.dismiss_on_escape = enabled;
        self
    }

    pub fn set_escape_dismiss(&mut self, enabled: bool) {
        self.dismiss_on_escape = enabled;
    }

    pub fn with_pointer_down_outside_dismiss(mut self, enabled: bool) -> Self {
        self.dismiss_on_pointer_down_outside = enabled;
        self
    }

    pub fn set_pointer_down_outside_dismiss(&mut self, enabled: bool) {
        self.dismiss_on_pointer_down_outside = enabled;
    }

    pub fn with_focus_outside_dismiss(mut self, enabled: bool) -> Self {
        self.dismiss_on_focus_outside = enabled;
        self
    }

    pub fn set_focus_outside_dismiss(&mut self, enabled: bool) {
        self.dismiss_on_focus_outside = enabled;
    }

    pub fn is_modal(&self) -> bool {
        self.modal
    }

    pub fn register_branch(&mut self, branch: T) -> bool {
        if self.id == branch || self.branches.iter().any(|candidate| candidate == &branch) {
            return false;
        }

        self.branches.push(branch);
        true
    }

    pub fn unregister_branch(&mut self, branch: &T) -> bool {
        let Some(position) = self
            .branches
            .iter()
            .position(|candidate| candidate == branch)
        else {
            return false;
        };

        self.branches.remove(position);
        true
    }

    pub fn contains(&self, target: &T) -> bool {
        &self.id == target || self.branches.iter().any(|branch| branch == target)
    }

    pub fn is_topmost(&self, stack: &[T]) -> bool {
        stack.last().is_some_and(|top| top == &self.id)
    }

    pub fn should_dismiss_escape(&self, stack: &[T]) -> bool {
        self.dismiss_on_escape && self.is_topmost(stack)
    }

    pub fn should_dismiss_outside_pointer(&self, target: Option<&T>, stack: &[T]) -> bool {
        self.dismiss_on_pointer_down_outside
            && self.is_topmost(stack)
            && target.map(|target| !self.contains(target)).unwrap_or(true)
    }

    pub fn should_dismiss_outside_focus(&self, target: Option<&T>, stack: &[T]) -> bool {
        self.dismiss_on_focus_outside
            && self.is_topmost(stack)
            && target.map(|target| !self.contains(target)).unwrap_or(true)
    }

    pub fn blocks_outside_interaction(&self) -> bool {
        self.modal
    }
}

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

#[cfg(test)]
mod tests {
    use super::{
        DismissLayer, FloatingLayer, FocusScope, PlacementAlign, PlacementSide, PortalHost,
        Presence, PresenceState, Rect, Size,
    };
    use crate::foundation::shared::Direction;

    #[test]
    fn portal_hosts_resolve_local_then_inherited_then_default_host() {
        let inherited = PortalHost::named("dialog-root");

        assert_eq!(
            PortalHost::resolve(Some(PortalHost::named("popover-root")), Some(&inherited)),
            PortalHost::named("popover-root"),
        );
        assert_eq!(PortalHost::resolve(None, Some(&inherited)), inherited);
        assert_eq!(PortalHost::resolve(None, None), PortalHost::default_host());
        assert!(PortalHost::default_host().is_default_host());
        assert!(PortalHost::inline().is_inline());
    }

    #[test]
    fn retained_presence_waits_for_completion_before_unmounting() {
        let mut presence = Presence::new(true).with_retained_mount(true);

        assert_eq!(presence.sync(false), PresenceState::Suspended);
        assert!(presence.is_mounted());
        assert!(presence.complete_unmount());
        assert_eq!(presence.state(), PresenceState::Unmounted);
        assert!(!presence.is_mounted());
        assert!(!presence.complete_unmount());

        assert_eq!(presence.sync(true), PresenceState::Mounted);
        assert!(presence.is_mounted());
    }

    #[test]
    fn focus_scope_tracks_activation_restore_and_branch_state() {
        let mut scope = FocusScope::new("content")
            .with_trap_focus(true)
            .with_loop_focus(true);
        scope.capture_restore_target(Some("trigger"));
        scope.set_autofocus_target(Some("content"));

        assert!(scope.register_branch("portal"));
        assert!(scope.activate().is_some());
        assert!(scope.is_active());
        assert!(scope.is_parent_paused());
        assert!(scope.focus("portal"));
        assert_eq!(scope.last_focused(), Some(&"portal"));
        assert_eq!(scope.restore_focus(), Some("trigger"));
        assert!(!scope.focus("outside"));
        assert_eq!(scope.deactivate(), Some("trigger"));
        assert!(!scope.is_active());
        assert!(!scope.is_parent_paused());
    }

    #[test]
    fn focus_guards_reference_count_their_lifecycle() {
        let mut guards = super::FocusGuards::new("before", "after");

        assert_eq!(guards.retain_count(), 0);
        assert!(!guards.is_installed());
        assert_eq!(guards.retain(), 1);
        assert!(guards.is_installed());
        assert_eq!(guards.release(), 0);
        assert!(!guards.is_installed());
    }

    #[test]
    fn dismiss_layers_account_for_stack_position_branch_exceptions_and_modal_state() {
        let mut dialog = DismissLayer::new("dialog").with_modal(true);
        let mut popover = DismissLayer::new("popover");
        let stack = ["dialog", "popover"];

        assert!(dialog.register_branch("dialog-portal"));
        assert!(dialog.contains(&"dialog-portal"));
        assert!(dialog.blocks_outside_interaction());
        assert!(dialog.should_dismiss_escape(&["dialog"]));
        assert!(!dialog.should_dismiss_escape(&stack));
        assert!(popover.register_branch("popover-branch"));
        assert!(popover.should_dismiss_escape(&["popover"]));
        assert!(popover.should_dismiss_outside_pointer(Some(&"outside"), &["popover"]));
        assert!(!popover.should_dismiss_outside_pointer(Some(&"popover-branch"), &["popover"]));
        assert!(popover.should_dismiss_outside_focus(Some(&"outside"), &["popover"]));
    }

    #[test]
    fn floating_layers_publish_namespaced_geometry_variables() {
        let layer = FloatingLayer::new(PlacementSide::Bottom)
            .with_align(PlacementAlign::Start)
            .with_direction(Direction::Rtl)
            .with_side_offset(8.0)
            .with_align_offset(4.0)
            .with_available_space(Size::new(120.0, 80.0))
            .with_namespace("dialog");
        let geometry =
            layer.geometry_vars(Rect::new(10.0, 20.0, 40.0, 16.0), Size::new(30.0, 12.0));

        assert_eq!(layer.data_side(), "bottom");
        assert_eq!(layer.data_align(), "start");
        assert_eq!(geometry.namespace(), "dialog");
        assert_eq!(geometry.x(), 24.0);
        assert_eq!(geometry.y(), 44.0);
        assert_eq!(geometry.transform_origin_x(), 26.0);
        assert_eq!(geometry.transform_origin_y(), 0.0);
        assert_eq!(geometry.available_width(), 120.0);
        assert_eq!(geometry.available_height(), 80.0);
        assert_eq!(geometry.get("--monoxus-dialog-floating-x"), Some(24.0));
        assert_eq!(
            geometry.get("--monoxus-dialog-available-width"),
            Some(120.0)
        );
        assert_eq!(
            geometry.iter().collect::<Vec<_>>(),
            vec![
                ("--monoxus-dialog-floating-x".to_string(), 24.0),
                ("--monoxus-dialog-floating-y".to_string(), 44.0),
                ("--monoxus-dialog-transform-origin-x".to_string(), 26.0),
                ("--monoxus-dialog-transform-origin-y".to_string(), 0.0),
                ("--monoxus-dialog-available-width".to_string(), 120.0),
                ("--monoxus-dialog-available-height".to_string(), 80.0),
                ("--monoxus-dialog-anchor-width".to_string(), 40.0),
                ("--monoxus-dialog-anchor-height".to_string(), 16.0),
                ("--monoxus-dialog-content-width".to_string(), 30.0),
                ("--monoxus-dialog-content-height".to_string(), 12.0),
            ],
        );
    }

    #[test]
    fn floating_layers_flip_to_the_opposite_side_and_publish_runtime_arrow_geometry() {
        let layer = FloatingLayer::new(PlacementSide::Top)
            .with_align(PlacementAlign::Start)
            .with_side_offset(8.0)
            .with_namespace("tooltip");
        let placement = layer.position_with_available_size(
            Rect::new(24.0, 8.0, 120.0, 38.0),
            Size::new(184.0, 72.0),
            Size::new(240.0, 140.0),
        );

        assert_eq!(placement.side(), PlacementSide::Bottom);
        assert_eq!(placement.align(), PlacementAlign::Start);
        assert_eq!(placement.geometry().x(), 24.0);
        assert_eq!(placement.geometry().y(), 54.0);
        assert_eq!(placement.arrow().x(), Some(60.0));
        assert_eq!(placement.arrow().y(), None);
        assert!(!placement.arrow().hidden());
        assert!(!placement.reference_hidden());
    }

    #[test]
    fn floating_layers_hide_the_arrow_when_the_live_geometry_cannot_center_it() {
        let layer = FloatingLayer::new(PlacementSide::Bottom)
            .with_align(PlacementAlign::Center)
            .with_side_offset(8.0);
        let placement = layer.position_with_available_size(
            Rect::new(0.0, 30.0, 16.0, 20.0),
            Size::new(100.0, 40.0),
            Size::new(120.0, 120.0),
        );

        assert_eq!(placement.side(), PlacementSide::Bottom);
        assert!(placement.arrow().hidden());
        assert_eq!(placement.arrow().x(), None);
        assert_eq!(placement.arrow().y(), None);
        assert!(!placement.reference_hidden());
    }

    #[test]
    fn floating_layers_can_hide_when_the_reference_detaches_from_the_viewport() {
        let visible = FloatingLayer::new(PlacementSide::Bottom).position_with_available_size(
            Rect::new(24.0, -8.0, 120.0, 38.0),
            Size::new(184.0, 72.0),
            Size::new(240.0, 140.0),
        );
        let hidden = FloatingLayer::new(PlacementSide::Bottom)
            .with_hide_when_detached(true)
            .position_with_available_size(
                Rect::new(24.0, -80.0, 120.0, 38.0),
                Size::new(184.0, 72.0),
                Size::new(240.0, 140.0),
            );

        assert!(!visible.reference_hidden());
        assert!(hidden.reference_hidden());
        assert!(hidden.arrow().hidden());
        assert_eq!(hidden.arrow().x(), None);
        assert_eq!(hidden.arrow().y(), None);
    }

    #[test]
    fn floating_layers_keep_attached_popovers_offscreen_with_their_anchor() {
        let placement = FloatingLayer::new(PlacementSide::Bottom)
            .with_align(PlacementAlign::Start)
            .with_side_offset(12.0)
            .position_with_available_size(
                Rect::new(24.0, -198.0, 120.0, 35.0),
                Size::new(184.0, 209.0),
                Size::new(1280.0, 720.0),
            );

        assert_eq!(placement.side(), PlacementSide::Bottom);
        assert!(placement.geometry().y() < 0.0);
        assert_eq!(placement.geometry().y(), -151.0);
        assert!(!placement.reference_hidden());
    }
}

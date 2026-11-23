use crate::foundation::{
    overlay::{
        DismissLayer, FloatingLayer, GeometryVars, PlacementAlign, PlacementSide, Point2D,
        PortalHost, Presence, Rect2D, StickyStrategy, create_grace_area, is_point_in_polygon,
    },
    shared::Direction,
    state::DataState,
};

use super::types::{
    CheckedState, MENU_DEFAULT_COLLISION_PADDING, MENU_DEFAULT_LOOP_FOCUS, MENU_DEFAULT_MODAL,
    MENU_DEFAULT_SIDE_OFFSET, MenuItemEntry, MenuItemKind, MenuKey, MenuNavigationStep,
    MenuVariant, PressedItemSnapshot, SubmenuCorridorDirection, SubmenuIntentConfig,
    SubmenuPointerEvaluation, TypeaheadConfig,
};

/// Emit canonical `--monoxus-<variant>-*` and `--radix-<variant>-*` CSS variables without duplicates.
pub fn menu_css_vars(geometry: &GeometryVars, variant: MenuVariant) -> Vec<(String, String)> {
    let monoxus_prefix = format!("monoxus-{}", variant.as_str());
    let mut vars: Vec<(String, String)> = geometry.css_iter().collect();

    for (name, value) in geometry
        .compatibility_alias_iter(&monoxus_prefix, "anchor")
        .chain(geometry.compatibility_alias_iter(
            variant.radix_compatibility_prefix(),
            variant.radix_anchor_label(),
        ))
    {
        if !vars.iter().any(|(existing, _)| existing == &name) {
            vars.push((name, value));
        }
    }

    vars
}

/// Toggle `value` inside a controlled checkbox group selection vector.
pub fn toggle_checkbox_group_value(current: &[String], value: &str) -> Vec<String> {
    let mut next = current.to_vec();
    if let Some(pos) = next.iter().position(|entry| entry == value) {
        next.remove(pos);
    } else {
        next.push(value.to_owned());
    }
    next
}

/// Resolve whether `value` is checked inside a `CheckboxGroup` value slice.
pub fn checkbox_group_item_state(current: &[String], value: &str) -> CheckedState {
    if current.iter().any(|entry| entry == value) {
        CheckedState::Checked
    } else {
        CheckedState::Unchecked
    }
}

/// Stateful typeahead prefix matcher supporting repeated-character cycling and multi-character buffering (`M-13`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeaheadBuffer {
    config: TypeaheadConfig,
    search: String,
    last_typed_ms: Option<u64>,
}

impl TypeaheadBuffer {
    pub fn new(config: TypeaheadConfig) -> Self {
        Self {
            config,
            search: String::new(),
            last_typed_ms: None,
        }
    }

    pub fn config(&self) -> TypeaheadConfig {
        self.config
    }

    pub fn search(&self) -> &str {
        &self.search
    }

    pub fn last_typed_ms(&self) -> Option<u64> {
        self.last_typed_ms
    }

    pub fn is_active(&self, now_ms: u64) -> bool {
        !self.search.is_empty()
            && self
                .last_typed_ms
                .is_some_and(|typed_at| now_ms.saturating_sub(typed_at) < self.config.timeout_ms)
    }

    pub fn expire_if_elapsed(&mut self, now_ms: u64) -> bool {
        if self.search.is_empty() {
            return false;
        }
        let elapsed = self
            .last_typed_ms
            .map(|typed_at| now_ms.saturating_sub(typed_at))
            .unwrap_or(self.config.timeout_ms);
        if elapsed >= self.config.timeout_ms {
            self.reset();
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.search.clear();
        self.last_typed_ms = None;
    }

    /// Push a printable character and return the matching enabled item's ID, if any.
    pub fn push_char(
        &mut self,
        ch: char,
        now_ms: u64,
        items: &[MenuItemEntry],
        current_highlighted_id: Option<&str>,
    ) -> Option<String> {
        if ch.is_control() {
            return None;
        }

        self.expire_if_elapsed(now_ms);

        // Do not start a new typeahead search on Space; Space activates the highlighted item
        // when no search buffer is active, and narrows multi-word searches once active.
        if self.search.is_empty() && ch == ' ' {
            return None;
        }

        self.search.push(ch);
        self.last_typed_ms = Some(now_ms);

        match_next_typeahead_item(items, &self.search, current_highlighted_id)
    }
}

impl Default for TypeaheadBuffer {
    fn default() -> Self {
        Self::new(TypeaheadConfig::default())
    }
}

/// Pure Radix/Bits-compatible typeahead matcher over ordered menu items.
pub fn match_next_typeahead_item(
    items: &[MenuItemEntry],
    raw_search: &str,
    current_highlighted_id: Option<&str>,
) -> Option<String> {
    if raw_search.trim().is_empty() {
        return None;
    }

    let enabled: Vec<&MenuItemEntry> = items
        .iter()
        .filter(|item| !item.disabled && !item.text_value.trim().is_empty())
        .collect();
    if enabled.is_empty() {
        return None;
    }

    let lowered = raw_search.to_lowercase();
    let mut chars = lowered.chars();
    let first_char = chars.next()?;
    let is_repeated = lowered.chars().count() > 1 && chars.all(|c| c == first_char);

    let normalized = if is_repeated {
        first_char.to_string()
    } else {
        lowered
    };
    let single_char_cycle = normalized.chars().count() == 1;

    let current_index = current_highlighted_id
        .and_then(|current| enabled.iter().position(|item| item.id == current));

    let start_index = match current_index {
        Some(idx) if single_char_cycle => (idx + 1) % enabled.len(),
        Some(idx) => idx,
        None => 0,
    };

    for offset in 0..enabled.len() {
        let candidate = enabled[(start_index + offset) % enabled.len()];
        if candidate
            .text_value
            .trim_start()
            .to_lowercase()
            .starts_with(&normalized)
        {
            return Some(candidate.id.clone());
        }
    }

    None
}

/// Resolve enabled navigation steps inside a single menu surface.
pub fn resolve_menu_navigation(
    items: &[MenuItemEntry],
    highlighted_id: Option<&str>,
    key: MenuKey,
    direction: Direction,
    loop_focus: bool,
    modal: bool,
    is_submenu: bool,
    tab_shift: bool,
) -> MenuNavigationStep {
    if matches!(key, MenuKey::Escape) {
        return MenuNavigationStep::CloseRootMenu;
    }

    if matches!(key, MenuKey::Tab) {
        return if modal {
            MenuNavigationStep::TabTrap
        } else {
            MenuNavigationStep::TabOut {
                forward: !tab_shift,
            }
        };
    }

    let enabled: Vec<&MenuItemEntry> = items.iter().filter(|item| !item.disabled).collect();
    let highlighted_entry = highlighted_id.and_then(|id| {
        enabled
            .iter()
            .copied()
            .find(|entry| entry.id == id)
            .or_else(|| items.iter().find(|entry| entry.id == id))
    });

    if key.is_submenu_open_key(direction)
        && let Some(entry) = highlighted_entry
        && !entry.disabled
        && entry.kind == MenuItemKind::SubTrigger
    {
        return MenuNavigationStep::OpenSubmenuAndFocusFirst {
            sub_trigger_id: entry.id.clone(),
        };
    }

    if is_submenu && key.is_submenu_close_key(direction) {
        return MenuNavigationStep::CloseSubmenuAndFocusTrigger;
    }

    if enabled.is_empty() {
        return if key.is_vertical_nav_key() {
            MenuNavigationStep::FocusContentContainer
        } else {
            MenuNavigationStep::None
        };
    }

    let current_pos =
        highlighted_id.and_then(|id| enabled.iter().position(|entry| entry.id == id));

    let next_entry = match key {
        MenuKey::ArrowDown => match current_pos {
            Some(idx) if idx + 1 < enabled.len() => Some(enabled[idx + 1]),
            Some(_) if loop_focus => Some(enabled[0]),
            Some(_) => None,
            None => Some(enabled[0]),
        },
        MenuKey::ArrowUp => match current_pos {
            Some(idx) if idx > 0 => Some(enabled[idx - 1]),
            Some(_) if loop_focus => Some(enabled[enabled.len() - 1]),
            Some(_) => None,
            None => Some(enabled[enabled.len() - 1]),
        },
        MenuKey::Home | MenuKey::PageUp => Some(enabled[0]),
        MenuKey::End | MenuKey::PageDown => Some(enabled[enabled.len() - 1]),
        _ => None,
    };

    match next_entry {
        Some(entry) => MenuNavigationStep::HighlightItem {
            id: entry.id.clone(),
        },
        None => MenuNavigationStep::None,
    }
}

/// First enabled item ID in DOM order.
pub fn first_enabled_item_id(items: &[MenuItemEntry]) -> Option<String> {
    items
        .iter()
        .find(|item| !item.disabled)
        .map(|item| item.id.clone())
}

/// Last enabled item ID in DOM order.
pub fn last_enabled_item_id(items: &[MenuItemEntry]) -> Option<String> {
    items
        .iter()
        .rev()
        .find(|item| !item.disabled)
        .map(|item| item.id.clone())
}

/// Bidirectional submenu triangle/grace corridor state machine (`M-11`).
#[derive(Clone, Debug, PartialEq)]
pub struct SubmenuIntentMachine {
    config: SubmenuIntentConfig,
    direction: SubmenuCorridorDirection,
    side: PlacementSide,
    exit_point: Option<Point2D>,
    last_pointer: Option<Point2D>,
    trigger_rect: Option<Rect2D>,
    content_rect: Option<Rect2D>,
    corridor_polygon: Vec<Point2D>,
    last_progress_ms: u64,
}

impl SubmenuIntentMachine {
    pub fn new(config: SubmenuIntentConfig) -> Self {
        Self {
            config,
            direction: SubmenuCorridorDirection::Idle,
            side: PlacementSide::Right,
            exit_point: None,
            last_pointer: None,
            trigger_rect: None,
            content_rect: None,
            corridor_polygon: Vec::new(),
            last_progress_ms: 0,
        }
    }

    pub fn config(&self) -> SubmenuIntentConfig {
        self.config
    }

    pub fn direction(&self) -> SubmenuCorridorDirection {
        self.direction
    }

    pub fn side(&self) -> PlacementSide {
        self.side
    }

    pub fn corridor_polygon(&self) -> &[Point2D] {
        &self.corridor_polygon
    }

    pub fn is_active(&self) -> bool {
        self.direction != SubmenuCorridorDirection::Idle
    }

    pub fn clear(&mut self) {
        self.direction = SubmenuCorridorDirection::Idle;
        self.exit_point = None;
        self.last_pointer = None;
        self.trigger_rect = None;
        self.content_rect = None;
        self.corridor_polygon.clear();
        self.last_progress_ms = 0;
    }

    /// Engage a grace corridor when leaving `trigger_rect` toward an open `content_rect`.
    pub fn begin_trigger_to_sub(
        &mut self,
        exit_point: Point2D,
        trigger_rect: Rect2D,
        content_rect: Rect2D,
        side: PlacementSide,
        now_ms: u64,
    ) {
        let inset_trigger = inset_rect(&trigger_rect, self.config.trigger_inset_px);
        let padded_target = expand_rect(&content_rect, self.config.corridor_buffer_px);

        self.direction = SubmenuCorridorDirection::TriggerToContent;
        self.side = side;
        self.exit_point = Some(exit_point);
        self.last_pointer = Some(exit_point);
        self.trigger_rect = Some(trigger_rect);
        self.content_rect = Some(content_rect);
        self.corridor_polygon = create_grace_area(
            exit_point,
            &inset_trigger,
            &padded_target,
            Some(self.config.grace_padding_px),
        );
        self.last_progress_ms = now_ms;
    }

    /// Engage a reverse grace corridor when leaving `content_rect` back toward `trigger_rect`.
    pub fn begin_content_to_trigger(
        &mut self,
        exit_point: Point2D,
        content_rect: Rect2D,
        trigger_rect: Rect2D,
        side: PlacementSide,
        now_ms: u64,
    ) {
        let padded_trigger = expand_rect(&trigger_rect, self.config.corridor_buffer_px);

        self.direction = SubmenuCorridorDirection::ContentToTrigger;
        self.side = side;
        self.exit_point = Some(exit_point);
        self.last_pointer = Some(exit_point);
        self.trigger_rect = Some(trigger_rect);
        self.content_rect = Some(content_rect);
        self.corridor_polygon = create_grace_area(
            exit_point,
            &content_rect,
            &padded_trigger,
            Some(self.config.grace_padding_px),
        );
        self.last_progress_ms = now_ms;
    }

    pub fn contains_point(&self, point: Point2D) -> bool {
        if self.corridor_polygon.is_empty() {
            return false;
        }
        is_point_in_polygon(point, &self.corridor_polygon)
    }

    /// Evaluate pointer movement across the active corridor.
    pub fn evaluate_pointer_move(
        &mut self,
        pointer: Point2D,
        now_ms: u64,
        pointer_in_sub_tree: bool,
        pointer_on_own_trigger: bool,
        pointer_in_other_subcontent: bool,
    ) -> SubmenuPointerEvaluation {
        if self.direction == SubmenuCorridorDirection::Idle {
            return SubmenuPointerEvaluation::Idle;
        }

        if pointer_in_other_subcontent {
            self.clear();
            return SubmenuPointerEvaluation::ExitedCorridor;
        }

        match self.direction {
            SubmenuCorridorDirection::TriggerToContent if pointer_in_sub_tree => {
                self.clear();
                return SubmenuPointerEvaluation::ReachedTarget;
            }
            SubmenuCorridorDirection::ContentToTrigger if pointer_on_own_trigger => {
                self.clear();
                return SubmenuPointerEvaluation::ReachedTarget;
            }
            _ => {}
        }

        if !self.contains_point(pointer) {
            self.clear();
            return SubmenuPointerEvaluation::ExitedCorridor;
        }

        let moved_toward_target = self
            .last_pointer
            .is_some_and(|prev| self.is_moving_toward_target(prev, pointer));

        self.last_pointer = Some(pointer);

        if moved_toward_target {
            self.last_progress_ms = now_ms;
            return SubmenuPointerEvaluation::InCorridor;
        }

        if now_ms.saturating_sub(self.last_progress_ms) >= self.config.fallback_timeout_ms {
            self.clear();
            return SubmenuPointerEvaluation::Stalled;
        }

        SubmenuPointerEvaluation::InCorridor
    }

    /// Check whether a stall fallback deadline has expired at `now_ms`.
    pub fn check_timeout(&mut self, now_ms: u64) -> SubmenuPointerEvaluation {
        if self.direction == SubmenuCorridorDirection::Idle {
            return SubmenuPointerEvaluation::Idle;
        }
        let threshold = match self.direction {
            SubmenuCorridorDirection::TriggerToContent => self.config.fallback_timeout_ms,
            SubmenuCorridorDirection::ContentToTrigger => self.config.return_disengage_ms,
            SubmenuCorridorDirection::Idle => return SubmenuPointerEvaluation::Idle,
        };
        if now_ms.saturating_sub(self.last_progress_ms) >= threshold {
            self.clear();
            SubmenuPointerEvaluation::Stalled
        } else {
            SubmenuPointerEvaluation::InCorridor
        }
    }

    fn is_moving_toward_target(&self, prev: Point2D, next: Point2D) -> bool {
        let min = self.config.min_progress_px;
        let forward = match self.side {
            PlacementSide::Right => (next.x - prev.x) >= min,
            PlacementSide::Left => (prev.x - next.x) >= min,
            PlacementSide::Bottom => (next.y - prev.y) >= min,
            PlacementSide::Top => (prev.y - next.y) >= min,
        };
        let backward = match self.side {
            PlacementSide::Right => (prev.x - next.x) >= min,
            PlacementSide::Left => (next.x - prev.x) >= min,
            PlacementSide::Bottom => (prev.y - next.y) >= min,
            PlacementSide::Top => (next.y - prev.y) >= min,
        };
        match self.direction {
            SubmenuCorridorDirection::TriggerToContent => forward,
            SubmenuCorridorDirection::ContentToTrigger => backward,
            SubmenuCorridorDirection::Idle => false,
        }
    }
}

impl Default for SubmenuIntentMachine {
    fn default() -> Self {
        Self::new(SubmenuIntentConfig::default())
    }
}

fn inset_rect(rect: &Rect2D, inset: f64) -> Rect2D {
    let clamped_x = inset.max(0.0).min(rect.width * 0.45);
    let clamped_y = inset.max(0.0).min(rect.height * 0.45);
    Rect2D::new(
        rect.x + clamped_x,
        rect.y + clamped_y,
        (rect.width - clamped_x * 2.0).max(0.0),
        (rect.height - clamped_y * 2.0).max(0.0),
    )
}

fn expand_rect(rect: &Rect2D, padding: f64) -> Rect2D {
    let p = padding.max(0.0);
    Rect2D::new(
        rect.x - p,
        rect.y - p,
        rect.width + p * 2.0,
        rect.height + p * 2.0,
    )
}

/// Core configuration and headless lifecycle state for a menu or submenu content surface.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuContentState {
    variant: MenuVariant,
    direction: Direction,
    loop_focus: bool,
    modal: bool,
    portal_host: PortalHost,
    presence: Presence,
    dismiss_layer: DismissLayer<String>,
    floating: FloatingLayer,
    highlighted_id: Option<String>,
    pressed_item: Option<PressedItemSnapshot>,
    items: Vec<MenuItemEntry>,
    typeahead: TypeaheadBuffer,
}

impl MenuContentState {
    pub fn new(variant: MenuVariant, content_id: impl Into<String>, open: bool) -> Self {
        let content_id = content_id.into();
        let floating = FloatingLayer::new(PlacementSide::Bottom)
            .with_align(PlacementAlign::Start)
            .with_side_offset(MENU_DEFAULT_SIDE_OFFSET)
            .with_collision_padding(MENU_DEFAULT_COLLISION_PADDING)
            .with_sticky(StickyStrategy::Partial)
            .with_namespace(variant.geometry_namespace());

        Self {
            variant,
            direction: Direction::Ltr,
            loop_focus: MENU_DEFAULT_LOOP_FOCUS,
            modal: MENU_DEFAULT_MODAL,
            portal_host: PortalHost::default_host(),
            presence: Presence::new(open).with_retained_mount(true),
            dismiss_layer: DismissLayer::new(content_id).with_modal(MENU_DEFAULT_MODAL),
            floating,
            highlighted_id: None,
            pressed_item: None,
            items: Vec::new(),
            typeahead: TypeaheadBuffer::default(),
        }
    }

    pub fn variant(&self) -> MenuVariant {
        self.variant
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self.floating = self.floating.with_direction(direction);
        self
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn with_modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self.dismiss_layer = self.dismiss_layer.clone().with_modal(modal);
        self
    }

    pub fn is_modal(&self) -> bool {
        self.modal
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.portal_host = portal_host;
        self
    }

    pub fn portal_host(&self) -> &PortalHost {
        &self.portal_host
    }

    pub fn with_floating(mut self, floating: FloatingLayer) -> Self {
        self.floating = floating.with_namespace(self.variant.geometry_namespace());
        self
    }

    pub fn floating(&self) -> &FloatingLayer {
        &self.floating
    }

    pub fn presence(&self) -> &Presence {
        &self.presence
    }

    pub fn presence_mut(&mut self) -> &mut Presence {
        &mut self.presence
    }

    pub fn dismiss_layer(&self) -> &DismissLayer<String> {
        &self.dismiss_layer
    }

    pub fn dismiss_layer_mut(&mut self) -> &mut DismissLayer<String> {
        &mut self.dismiss_layer
    }

    pub fn is_open(&self) -> bool {
        self.presence.desired_present()
    }

    pub fn data_state(&self) -> DataState {
        if self.is_open() {
            DataState::Open
        } else {
            DataState::Closed
        }
    }

    pub fn highlighted_id(&self) -> Option<&str> {
        self.highlighted_id.as_deref()
    }

    pub fn set_highlighted_id(&mut self, id: Option<String>) {
        self.highlighted_id = id;
    }

    pub fn pressed_item(&self) -> Option<&PressedItemSnapshot> {
        self.pressed_item.as_ref()
    }

    pub fn set_pressed_item(&mut self, pressed: Option<PressedItemSnapshot>) {
        self.pressed_item = pressed;
    }

    pub fn items(&self) -> &[MenuItemEntry] {
        &self.items
    }

    pub fn register_item(&mut self, item: MenuItemEntry) {
        if let Some(existing) = self.items.iter_mut().find(|entry| entry.id == item.id) {
            *existing = item;
        } else {
            self.items.push(item);
        }
        self.items.sort_by_key(|entry| entry.dom_order);
    }

    pub fn unregister_item(&mut self, id: &str) {
        self.items.retain(|entry| entry.id != id);
        if self.highlighted_id.as_deref() == Some(id) {
            self.highlighted_id = None;
        }
    }

    pub fn sync_dom_order(&mut self, ordered_ids: &[String]) {
        for item in &mut self.items {
            if let Some(pos) = ordered_ids.iter().position(|id| id == &item.id) {
                item.dom_order = pos;
            }
        }
        self.items.sort_by_key(|entry| entry.dom_order);
    }

    pub fn typeahead(&self) -> &TypeaheadBuffer {
        &self.typeahead
    }

    pub fn typeahead_mut(&mut self) -> &mut TypeaheadBuffer {
        &mut self.typeahead
    }

    pub fn navigate(
        &mut self,
        key: MenuKey,
        is_submenu: bool,
        tab_shift: bool,
    ) -> MenuNavigationStep {
        let step = resolve_menu_navigation(
            &self.items,
            self.highlighted_id.as_deref(),
            key,
            self.direction,
            self.loop_focus,
            self.modal,
            is_submenu,
            tab_shift,
        );
        if let MenuNavigationStep::HighlightItem { ref id } = step {
            self.highlighted_id = Some(id.clone());
        } else if matches!(step, MenuNavigationStep::FocusContentContainer) {
            self.highlighted_id = None;
        }
        step
    }
}

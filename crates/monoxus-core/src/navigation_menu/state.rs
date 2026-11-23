use crate::foundation::{
    overlay::{DismissLayer, Presence},
    roving_focus::{RovingFocusConfig, RovingModifiers, resolve_next_stop_in_registry},
    shared::{CollectionRegistry, Direction, Orientation, ScopeHandle, data_disabled_attr},
    state::DataState,
};

use super::{
    attrs::{
        NavigationMenuAriaOwnsAttributes, NavigationMenuContentAttributes,
        NavigationMenuFocusProxyAttributes, NavigationMenuIndicatorAttributes,
        NavigationMenuItemAttributes, NavigationMenuLinkAttributes, NavigationMenuListAttributes,
        NavigationMenuListTrackAttributes, NavigationMenuRootAttributes,
        NavigationMenuSubAttributes, NavigationMenuTriggerAttributes,
        NavigationMenuViewportAttributes,
    },
    relationships::NavigationMenuRelationships,
    types::{
        NAVIGATION_MENU_CLOSE_GRACE_MS, NAVIGATION_MENU_DEFAULT_DELAY_MS,
        NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS, NAVIGATION_MENU_PARTS, NavigationMenuActivationMode,
        NavigationMenuFocusProxySide, NavigationMenuGeometryVars, NavigationMenuIndicatorPosition,
        NavigationMenuIndicatorState, NavigationMenuMotion, NavigationMenuPart,
        NavigationMenuViewportSize,
    },
};

/// Normalizes `Orientation::Both` to `Orientation::Horizontal` for root/sub navigation menus.
pub const fn normalize_navigation_menu_orientation(orientation: Orientation) -> Orientation {
    match orientation {
        Orientation::Vertical => Orientation::Vertical,
        Orientation::Horizontal | Orientation::Both => Orientation::Horizontal,
    }
}

pub const fn orientation_str(orientation: Orientation) -> &'static str {
    match normalize_navigation_menu_orientation(orientation) {
        Orientation::Vertical => "vertical",
        Orientation::Horizontal | Orientation::Both => "horizontal",
    }
}

/// Resolves whether hover interactions are enabled for an item given menu activation mode and
/// optional per-item `open_on_hover` override (`3C D-1`).
pub const fn is_item_hover_enabled(
    activation_mode: NavigationMenuActivationMode,
    item_open_on_hover: Option<bool>,
) -> bool {
    match item_open_on_hover {
        Some(open_on_hover) => open_on_hover,
        None => activation_mode.is_automatic(),
    }
}

/// Pure immutable definition for `NavigationMenuRoot` (and `NavigationMenuSub` via `is_root_menu = false`).
#[derive(Clone, Debug, PartialEq)]
pub struct NavigationMenuDef {
    scope: ScopeHandle,
    relationships: NavigationMenuRelationships,
    is_root_menu: bool,
    value: String,
    previous_value: String,
    orientation: Orientation,
    dir: Direction,
    activation_mode: NavigationMenuActivationMode,
    delay_duration_ms: u64,
    skip_delay_duration_ms: u64,
    close_grace_ms: u64,
    aria_label: Option<String>,
    disable_toggle: bool,
    presence: Presence,
    dismiss_layer: DismissLayer<String>,
}

impl NavigationMenuDef {
    pub fn new(scope: ScopeHandle) -> Self {
        let relationships = NavigationMenuRelationships::new(scope.clone());
        let dismiss_layer = DismissLayer::new(relationships.root_id().to_owned())
            .with_modal(false)
            .with_focus_outside_dismiss(true);

        Self {
            scope,
            relationships,
            is_root_menu: true,
            value: String::new(),
            previous_value: String::new(),
            orientation: Orientation::Horizontal,
            dir: Direction::Ltr,
            activation_mode: NavigationMenuActivationMode::Automatic,
            delay_duration_ms: NAVIGATION_MENU_DEFAULT_DELAY_MS,
            skip_delay_duration_ms: NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS,
            close_grace_ms: NAVIGATION_MENU_CLOSE_GRACE_MS,
            aria_label: Some("Main".to_string()),
            disable_toggle: false,
            presence: Presence::new(false).with_retained_mount(true),
            dismiss_layer,
        }
    }

    pub const fn parts() -> &'static [NavigationMenuPart] {
        &NAVIGATION_MENU_PARTS
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn relationships(&self) -> &NavigationMenuRelationships {
        &self.relationships
    }

    pub const fn is_root_menu(&self) -> bool {
        self.is_root_menu
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn previous_value(&self) -> &str {
        &self.previous_value
    }

    pub fn is_open(&self) -> bool {
        !self.value.is_empty()
    }

    pub const fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub const fn dir(&self) -> Direction {
        self.dir
    }

    pub const fn activation_mode(&self) -> NavigationMenuActivationMode {
        self.activation_mode
    }

    pub const fn delay_duration_ms(&self) -> u64 {
        self.delay_duration_ms
    }

    pub const fn skip_delay_duration_ms(&self) -> u64 {
        self.skip_delay_duration_ms
    }

    pub const fn close_grace_ms(&self) -> u64 {
        self.close_grace_ms
    }

    pub fn aria_label(&self) -> Option<&str> {
        self.aria_label.as_deref()
    }

    pub const fn disable_toggle(&self) -> bool {
        self.disable_toggle
    }

    pub fn presence(&self) -> &Presence {
        &self.presence
    }

    pub fn dismiss_layer(&self) -> &DismissLayer<String> {
        &self.dismiss_layer
    }

    pub fn with_is_root_menu(mut self, is_root_menu: bool) -> Self {
        self.is_root_menu = is_root_menu;
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        let next = value.into();
        self.presence.sync(!next.is_empty());
        self.value = next;
        self
    }

    pub fn with_previous_value(mut self, previous_value: impl Into<String>) -> Self {
        self.previous_value = previous_value.into();
        self
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = normalize_navigation_menu_orientation(orientation);
        self
    }

    pub const fn with_dir(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }

    pub const fn with_activation_mode(mut self, mode: NavigationMenuActivationMode) -> Self {
        self.activation_mode = mode;
        self
    }

    pub const fn with_delay_duration_ms(mut self, delay_duration_ms: u64) -> Self {
        self.delay_duration_ms = delay_duration_ms;
        self
    }

    pub const fn with_skip_delay_duration_ms(mut self, skip_delay_duration_ms: u64) -> Self {
        self.skip_delay_duration_ms = skip_delay_duration_ms;
        self
    }

    pub const fn with_close_grace_ms(mut self, close_grace_ms: u64) -> Self {
        self.close_grace_ms = close_grace_ms;
        self
    }

    pub fn with_aria_label(mut self, aria_label: Option<String>) -> Self {
        self.aria_label = aria_label;
        self
    }

    pub const fn with_disable_toggle(mut self, disable_toggle: bool) -> Self {
        self.disable_toggle = disable_toggle;
        self
    }

    pub const fn list_roving_config(&self) -> RovingFocusConfig {
        RovingFocusConfig::new()
            .with_orientation(self.orientation)
            .with_direction(self.dir)
            .with_loop_focus(false)
            .with_enabled(true)
    }

    pub const fn content_roving_config(&self) -> RovingFocusConfig {
        RovingFocusConfig::new()
            .with_orientation(Orientation::Both)
            .with_direction(self.dir)
            .with_loop_focus(false)
            .with_enabled(true)
    }

    pub fn toggle_value(&self, item_value: &str) -> String {
        if self.is_root_menu {
            if self.value == item_value {
                String::new()
            } else {
                item_value.to_string()
            }
        } else if self.disable_toggle || self.value != item_value {
            item_value.to_string()
        } else {
            String::new()
        }
    }

    pub fn root_attributes(&self, id_override: Option<String>) -> NavigationMenuRootAttributes {
        NavigationMenuRootAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.root_id().to_owned()),
            aria_label: self.aria_label.clone(),
            dir: self.dir.as_str(),
            data_orientation: orientation_str(self.orientation),
            data_monoxus_navigation_menu_root: "",
        }
    }

    pub fn sub_attributes(&self, id_override: Option<String>) -> NavigationMenuSubAttributes {
        NavigationMenuSubAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.root_id().to_owned()),
            data_orientation: orientation_str(self.orientation),
            data_monoxus_navigation_menu_sub: "",
        }
    }

    pub fn list_track_attributes(
        &self,
        id_override: Option<String>,
    ) -> NavigationMenuListTrackAttributes {
        NavigationMenuListTrackAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.track_id().to_owned()),
            style: "position: relative;".to_string(),
            data_monoxus_navigation_menu_track: "",
        }
    }

    pub fn list_attributes(&self, id_override: Option<String>) -> NavigationMenuListAttributes {
        NavigationMenuListAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.list_id().to_owned()),
            data_orientation: orientation_str(self.orientation),
            data_monoxus_navigation_menu_list: "",
        }
    }

    pub fn item_attributes(
        &self,
        value: &str,
        id_override: Option<String>,
    ) -> NavigationMenuItemAttributes {
        NavigationMenuItemAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.item_id(value)),
            data_monoxus_navigation_menu_item: "",
        }
    }

    pub fn trigger_attributes(
        &self,
        value: &str,
        disabled: bool,
        id_override: Option<String>,
        content_id_override: Option<String>,
    ) -> NavigationMenuTriggerAttributes {
        let open = self.value == value;
        let content_id =
            content_id_override.unwrap_or_else(|| self.relationships.content_id(value));
        NavigationMenuTriggerAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.trigger_id(value)),
            r#type: "button",
            disabled,
            aria_expanded: if open { "true" } else { "false" },
            aria_controls: open.then_some(content_id),
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_disabled: data_disabled_attr(disabled),
            data_value: value.to_string(),
            data_monoxus_navigation_menu_trigger: "",
        }
    }

    pub fn focus_proxy_attributes(&self, value: &str) -> NavigationMenuFocusProxyAttributes {
        NavigationMenuFocusProxyAttributes {
            id: self.relationships.focus_proxy_id(value),
            aria_hidden: "true",
            tabindex: 0,
        }
    }

    pub fn aria_owns_attributes(
        &self,
        value: &str,
        content_id_override: Option<String>,
    ) -> NavigationMenuAriaOwnsAttributes {
        let content_id =
            content_id_override.unwrap_or_else(|| self.relationships.content_id(value));
        NavigationMenuAriaOwnsAttributes {
            id: self.relationships.aria_owns_id(value),
            aria_owns: content_id,
        }
    }

    pub fn content_attributes(
        &self,
        value: &str,
        open: bool,
        motion: Option<NavigationMenuMotion>,
        inline_mode: bool,
        id_override: Option<String>,
        trigger_id_override: Option<String>,
        user_style: Option<&str>,
    ) -> NavigationMenuContentAttributes {
        let trigger_id =
            trigger_id_override.unwrap_or_else(|| self.relationships.trigger_id(value));
        let style = if inline_mode && !open && self.is_root_menu {
            append_inline_style(user_style, "pointer-events: none;")
        } else {
            user_style.unwrap_or_default().to_string()
        };
        NavigationMenuContentAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.content_id(value)),
            aria_labelledby: trigger_id,
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_orientation: orientation_str(self.orientation),
            data_motion: motion.map(|m| m.as_str()),
            style,
            data_monoxus_navigation_menu_content: "",
        }
    }

    pub fn link_attributes(
        &self,
        key: &str,
        href: Option<String>,
        target: Option<String>,
        rel: Option<String>,
        active: bool,
        focused: bool,
        id_override: Option<String>,
    ) -> NavigationMenuLinkAttributes {
        NavigationMenuLinkAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.link_id(key)),
            href,
            target,
            rel,
            tabindex: 0,
            aria_current: active.then_some("page"),
            data_active: active.then_some(""),
            data_focused: focused.then_some(""),
            data_monoxus_navigation_menu_link: "",
        }
    }

    pub fn indicator_attributes(
        &self,
        is_visible: bool,
        position: NavigationMenuIndicatorPosition,
        id_override: Option<String>,
        user_style: Option<&str>,
    ) -> NavigationMenuIndicatorAttributes {
        let style = build_indicator_style(self.orientation, position, user_style);
        NavigationMenuIndicatorAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.indicator_id().to_owned()),
            aria_hidden: "true",
            data_state: if is_visible {
                NavigationMenuIndicatorState::Visible.as_str()
            } else {
                NavigationMenuIndicatorState::Hidden.as_str()
            },
            data_orientation: orientation_str(self.orientation),
            style,
            data_monoxus_navigation_menu_indicator: "",
        }
    }

    pub fn viewport_attributes(
        &self,
        open: bool,
        size: Option<NavigationMenuViewportSize>,
        id_override: Option<String>,
        user_style: Option<&str>,
    ) -> NavigationMenuViewportAttributes {
        let style = build_viewport_style(open, self.is_root_menu, size, user_style);
        NavigationMenuViewportAttributes {
            id: id_override.unwrap_or_else(|| self.relationships.viewport_id().to_owned()),
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_orientation: orientation_str(self.orientation),
            style,
            data_monoxus_navigation_menu_viewport: "",
        }
    }
}

/// Sub-menu builder wrapper inheriting parent direction and activation mode.
#[derive(Clone, Debug, PartialEq)]
pub struct NavigationMenuSubDef {
    inner: NavigationMenuDef,
}

impl NavigationMenuSubDef {
    pub fn new(
        scope: ScopeHandle,
        inherited_dir: Direction,
        inherited_activation_mode: NavigationMenuActivationMode,
    ) -> Self {
        Self {
            inner: NavigationMenuDef::new(scope)
                .with_is_root_menu(false)
                .with_dir(inherited_dir)
                .with_activation_mode(inherited_activation_mode)
                .with_aria_label(None)
                .with_disable_toggle(true),
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.inner = self.inner.with_value(value);
        self
    }

    pub fn with_previous_value(mut self, previous_value: impl Into<String>) -> Self {
        self.inner = self.inner.with_previous_value(previous_value);
        self
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.inner = self.inner.with_orientation(orientation);
        self
    }

    pub fn with_activation_mode(mut self, mode: Option<NavigationMenuActivationMode>) -> Self {
        if let Some(m) = mode {
            self.inner = self.inner.with_activation_mode(m);
        }
        self
    }

    pub fn with_disable_toggle(mut self, disable_toggle: bool) -> Self {
        self.inner = self.inner.with_disable_toggle(disable_toggle);
        self
    }

    pub fn as_def(&self) -> &NavigationMenuDef {
        &self.inner
    }

    pub fn into_def(self) -> NavigationMenuDef {
        self.inner
    }
}

/// Formats inline CSS for `NavigationMenuIndicator`, emitting both `--monoxus-*` and `--radix-*`
/// translation variables plus `position: absolute`, `left`/`top`, `width`/`height`, and `transform`.
pub fn build_indicator_style(
    orientation: Orientation,
    position: NavigationMenuIndicatorPosition,
    user_style: Option<&str>,
) -> String {
    let vars =
        NavigationMenuGeometryVars::new(orientation).with_indicator_position(Some(position));
    let mut base = match normalize_navigation_menu_orientation(orientation) {
        Orientation::Vertical => format!(
            "position: absolute; top: 0; height: {}px; transform: translateY({}px);",
            position.size, position.offset
        ),
        Orientation::Horizontal | Orientation::Both => format!(
            "position: absolute; left: 0; width: {}px; transform: translateX({}px);",
            position.size, position.offset
        ),
    };
    for (name, val) in vars.indicator_css_variables() {
        base.push(' ');
        base.push_str(&name);
        base.push_str(": ");
        base.push_str(&val);
        base.push(';');
    }
    append_inline_style(Some(&base), user_style.unwrap_or_default())
}

/// Formats inline CSS for `NavigationMenuViewport`, emitting both `--monoxus-*` and `--radix-*`
/// viewport dimension variables and `pointer-events: none;` when closing in a root menu.
pub fn build_viewport_style(
    open: bool,
    is_root_menu: bool,
    size: Option<NavigationMenuViewportSize>,
    user_style: Option<&str>,
) -> String {
    let vars = NavigationMenuGeometryVars::new(Orientation::Horizontal).with_viewport_size(size);
    let mut base = String::new();
    if !open && is_root_menu {
        base.push_str("pointer-events: none;");
    }
    for (name, val) in vars.viewport_css_variables() {
        if !base.is_empty() {
            base.push(' ');
        }
        base.push_str(&name);
        base.push_str(": ");
        base.push_str(&val);
        base.push(';');
    }
    if base.is_empty() {
        user_style.unwrap_or_default().to_string()
    } else {
        append_inline_style(Some(&base), user_style.unwrap_or_default())
    }
}

fn append_inline_style(first: Option<&str>, second: &str) -> String {
    let first = first.map(str::trim).unwrap_or_default();
    let second = second.trim();
    match (first.is_empty(), second.is_empty()) {
        (true, true) => String::new(),
        (false, true) => first.to_string(),
        (true, false) => second.to_string(),
        (false, false) => {
            let mut out = first.to_string();
            if !out.ends_with(';') {
                out.push(';');
            }
            out.push(' ');
            out.push_str(second);
            out
        }
    }
}

/// Timer side-effect commands emitted by [`NavigationMenuHoverMachine`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationMenuTimerCommand {
    CancelOpenTimer,
    StartOpenTimer {
        item_value: String,
        delay_ms: u64,
        generation: u64,
    },
    CancelCloseTimer,
    StartCloseTimer {
        delay_ms: u64,
        generation: u64,
    },
    CancelSkipDelayTimer,
    StartSkipDelayTimer {
        delay_ms: u64,
        generation: u64,
    },
}

/// Actions accepted by [`NavigationMenuHoverMachine`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationMenuHoverAction {
    TriggerEnter {
        item_value: String,
        activation_mode: NavigationMenuActivationMode,
        item_open_on_hover: Option<bool>,
    },
    TriggerLeave {
        activation_mode: NavigationMenuActivationMode,
        item_open_on_hover: Option<bool>,
    },
    ContentEnter {
        activation_mode: NavigationMenuActivationMode,
    },
    ContentLeave {
        activation_mode: NavigationMenuActivationMode,
        item_open_on_hover: Option<bool>,
    },
    OpenTimerFired {
        item_value: String,
        generation: u64,
    },
    CloseTimerFired {
        generation: u64,
    },
    SkipDelayTimerFired {
        generation: u64,
    },
    ValueCommitted {
        next_value: String,
    },
}

/// Result of a [`NavigationMenuHoverMachine::dispatch`] step.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NavigationMenuHoverTransition {
    pub next_value: Option<String>,
    pub commands: Vec<NavigationMenuTimerCommand>,
}

/// Pure 3-timer Radix hover state machine (`openTimer`, `closeTimer`, `skipDelayTimer`)
/// with `3C D-1` / `3C D-5` `Manual` & `open_on_hover` gating.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationMenuHoverMachine {
    delay_duration_ms: u64,
    skip_delay_duration_ms: u64,
    close_grace_ms: u64,
    is_open_delayed: bool,
    current_value: String,
    open_timer_target: Option<String>,
    open_timer_gen: u64,
    close_timer_gen: u64,
    skip_delay_timer_gen: u64,
    close_timer_active: bool,
    skip_delay_timer_active: bool,
}

impl Default for NavigationMenuHoverMachine {
    fn default() -> Self {
        Self::new(
            NAVIGATION_MENU_DEFAULT_DELAY_MS,
            NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS,
            NAVIGATION_MENU_CLOSE_GRACE_MS,
        )
    }
}

impl NavigationMenuHoverMachine {
    pub const fn new(
        delay_duration_ms: u64,
        skip_delay_duration_ms: u64,
        close_grace_ms: u64,
    ) -> Self {
        Self {
            delay_duration_ms,
            skip_delay_duration_ms,
            close_grace_ms,
            is_open_delayed: true,
            current_value: String::new(),
            open_timer_target: None,
            open_timer_gen: 0,
            close_timer_gen: 0,
            skip_delay_timer_gen: 0,
            close_timer_active: false,
            skip_delay_timer_active: false,
        }
    }

    pub const fn is_open_delayed(&self) -> bool {
        self.is_open_delayed
    }

    pub fn current_value(&self) -> &str {
        &self.current_value
    }

    pub const fn open_timer_gen(&self) -> u64 {
        self.open_timer_gen
    }

    pub const fn close_timer_gen(&self) -> u64 {
        self.close_timer_gen
    }

    pub const fn skip_delay_timer_gen(&self) -> u64 {
        self.skip_delay_timer_gen
    }

    pub fn update_durations(&mut self, delay_duration_ms: u64, skip_delay_duration_ms: u64) {
        self.delay_duration_ms = delay_duration_ms;
        self.skip_delay_duration_ms = skip_delay_duration_ms;
    }

    pub fn cancel_all_timers(&mut self) -> Vec<NavigationMenuTimerCommand> {
        let mut commands = Vec::new();
        self.cancel_open_timer(&mut commands);
        self.cancel_close_timer(&mut commands);
        self.cancel_skip_delay_timer(&mut commands);
        commands
    }

    pub fn dispatch(&mut self, action: NavigationMenuHoverAction) -> NavigationMenuHoverTransition {
        let mut out = NavigationMenuHoverTransition::default();

        match action {
            NavigationMenuHoverAction::TriggerEnter {
                item_value,
                activation_mode,
                item_open_on_hover,
            } => {
                if !is_item_hover_enabled(activation_mode, item_open_on_hover) {
                    return out;
                }
                self.cancel_open_timer(&mut out.commands);

                if !self.is_open_delayed {
                    self.cancel_close_timer(&mut out.commands);
                    if self.current_value != item_value {
                        out.next_value = Some(item_value);
                    }
                } else if self.current_value == item_value {
                    self.cancel_close_timer(&mut out.commands);
                } else if self.delay_duration_ms == 0 {
                    self.cancel_close_timer(&mut out.commands);
                    out.next_value = Some(item_value);
                } else {
                    self.open_timer_gen = self.open_timer_gen.saturating_add(1);
                    self.open_timer_target = Some(item_value.clone());
                    out.commands.push(NavigationMenuTimerCommand::StartOpenTimer {
                        item_value,
                        delay_ms: self.delay_duration_ms,
                        generation: self.open_timer_gen,
                    });
                }
            }
            NavigationMenuHoverAction::TriggerLeave {
                activation_mode,
                item_open_on_hover,
            } => {
                if !is_item_hover_enabled(activation_mode, item_open_on_hover) {
                    return out;
                }
                self.cancel_open_timer(&mut out.commands);
                self.start_close_timer(&mut out.commands);
            }
            NavigationMenuHoverAction::ContentEnter { activation_mode } => {
                if activation_mode.is_automatic() {
                    self.cancel_close_timer(&mut out.commands);
                }
            }
            NavigationMenuHoverAction::ContentLeave {
                activation_mode,
                item_open_on_hover,
            } => {
                if !is_item_hover_enabled(activation_mode, item_open_on_hover) {
                    return out;
                }
                self.start_close_timer(&mut out.commands);
            }
            NavigationMenuHoverAction::OpenTimerFired {
                item_value,
                generation,
            } => {
                if generation == self.open_timer_gen
                    && self.open_timer_target.as_deref() == Some(item_value.as_str())
                {
                    self.open_timer_target = None;
                    self.cancel_close_timer(&mut out.commands);
                    if self.current_value != item_value {
                        out.next_value = Some(item_value);
                    }
                }
            }
            NavigationMenuHoverAction::CloseTimerFired { generation } => {
                if self.close_timer_active && generation == self.close_timer_gen {
                    self.close_timer_active = false;
                    if !self.current_value.is_empty() {
                        out.next_value = Some(String::new());
                    }
                }
            }
            NavigationMenuHoverAction::SkipDelayTimerFired { generation } => {
                if self.skip_delay_timer_active && generation == self.skip_delay_timer_gen {
                    self.skip_delay_timer_active = false;
                    self.is_open_delayed = true;
                }
            }
            NavigationMenuHoverAction::ValueCommitted { next_value } => {
                if self.current_value == next_value {
                    return out;
                }
                self.current_value = next_value.clone();
                self.cancel_skip_delay_timer(&mut out.commands);

                if !next_value.is_empty() {
                    if self.skip_delay_duration_ms > 0 {
                        self.is_open_delayed = false;
                    }
                } else if self.skip_delay_duration_ms == 0 {
                    self.is_open_delayed = true;
                } else {
                    self.skip_delay_timer_gen = self.skip_delay_timer_gen.saturating_add(1);
                    self.skip_delay_timer_active = true;
                    out.commands
                        .push(NavigationMenuTimerCommand::StartSkipDelayTimer {
                            delay_ms: self.skip_delay_duration_ms,
                            generation: self.skip_delay_timer_gen,
                        });
                }
            }
        }

        out
    }

    fn cancel_open_timer(&mut self, commands: &mut Vec<NavigationMenuTimerCommand>) {
        if self.open_timer_target.take().is_some() {
            self.open_timer_gen = self.open_timer_gen.saturating_add(1);
            commands.push(NavigationMenuTimerCommand::CancelOpenTimer);
        }
    }

    fn cancel_close_timer(&mut self, commands: &mut Vec<NavigationMenuTimerCommand>) {
        if self.close_timer_active {
            self.close_timer_active = false;
            self.close_timer_gen = self.close_timer_gen.saturating_add(1);
            commands.push(NavigationMenuTimerCommand::CancelCloseTimer);
        }
    }

    fn start_close_timer(&mut self, commands: &mut Vec<NavigationMenuTimerCommand>) {
        self.close_timer_gen = self.close_timer_gen.saturating_add(1);
        self.close_timer_active = true;
        commands.push(NavigationMenuTimerCommand::StartCloseTimer {
            delay_ms: self.close_grace_ms,
            generation: self.close_timer_gen,
        });
    }

    fn cancel_skip_delay_timer(&mut self, commands: &mut Vec<NavigationMenuTimerCommand>) {
        if self.skip_delay_timer_active {
            self.skip_delay_timer_active = false;
            self.skip_delay_timer_gen = self.skip_delay_timer_gen.saturating_add(1);
            commands.push(NavigationMenuTimerCommand::CancelSkipDelayTimer);
        }
    }
}

/// Computes `data-motion` for `NavigationMenuContent` using DOM-ordered trigger values and RTL awareness.
pub fn compute_motion_direction(
    ordered_trigger_values: &[String],
    dir: Direction,
    current_value: &str,
    previous_value: &str,
    item_value: &str,
    prev_motion: Option<NavigationMenuMotion>,
) -> Option<NavigationMenuMotion> {
    if current_value.is_empty() && previous_value.is_empty() {
        return None;
    }

    let mut values = ordered_trigger_values.to_vec();
    if dir == Direction::Rtl {
        values.reverse();
    }

    let index = (!current_value.is_empty())
        .then(|| values.iter().position(|v| v == current_value))
        .flatten();
    let prev_index = (!previous_value.is_empty())
        .then(|| values.iter().position(|v| v == previous_value))
        .flatten();
    let is_selected = !current_value.is_empty() && item_value == current_value;
    let was_selected = prev_index.is_some()
        && values.iter().position(|v| v == item_value) == prev_index;

    if !is_selected && !was_selected {
        return prev_motion;
    }

    if index != prev_index {
        if is_selected && let Some(p_idx) = prev_index && let Some(idx) = index {
            return Some(if idx > p_idx {
                NavigationMenuMotion::FromEnd
            } else {
                NavigationMenuMotion::FromStart
            });
        }
        if was_selected && let Some(idx) = index && let Some(p_idx) = prev_index {
            return Some(if idx > p_idx {
                NavigationMenuMotion::ToStart
            } else {
                NavigationMenuMotion::ToEnd
            });
        }
    }

    None
}

/// Returns `true` when `key` opens/enters the currently open trigger's content (`3C D-3`).
pub fn is_content_entry_key(orientation: Orientation, dir: Direction, key: &str) -> bool {
    match normalize_navigation_menu_orientation(orientation) {
        Orientation::Horizontal | Orientation::Both => key == "ArrowDown",
        Orientation::Vertical => match dir {
            Direction::Ltr => key == "ArrowRight",
            Direction::Rtl => key == "ArrowLeft",
        },
    }
}

/// Resolves the next tabbable element ID inside `NavigationMenuContent` for 4-way arrow + Home/End
/// navigation (`3C D-3`).
pub fn resolve_content_arrow_target(
    candidates: &[String],
    current_id: Option<&str>,
    key: &str,
    dir: Direction,
) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }
    let mut registry = CollectionRegistry::new();
    for id in candidates {
        registry.register(id.clone(), true);
    }
    let config = RovingFocusConfig::new()
        .with_orientation(Orientation::Both)
        .with_direction(dir)
        .with_loop_focus(false)
        .with_enabled(true);
    resolve_next_stop_in_registry(
        &config,
        &registry,
        current_id,
        key,
        RovingModifiers::none(),
        |focusable| *focusable,
    )
}

/// Decision for `FocusProxy` focus-entry when `FocusEvent.relatedTarget` is tracked at runtime (`ENG D-2`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationMenuFocusProxyDecision {
    Enter(NavigationMenuFocusProxySide),
    Ignore,
}

pub fn resolve_focus_proxy_decision(
    last_focused_id: Option<&str>,
    trigger_id: &str,
    was_focus_from_content: bool,
) -> NavigationMenuFocusProxyDecision {
    if last_focused_id == Some(trigger_id) {
        NavigationMenuFocusProxyDecision::Enter(NavigationMenuFocusProxySide::Start)
    } else if !was_focus_from_content {
        NavigationMenuFocusProxyDecision::Enter(NavigationMenuFocusProxySide::End)
    } else {
        NavigationMenuFocusProxyDecision::Ignore
    }
}

/// Radix `#3473` focus-outside guard: only dismiss when focus actually leaves the menu after
/// originating from inside the menu.
pub const fn should_dismiss_on_focus_outside(
    focus_moved_into_menu: bool,
    focus_came_from_menu: bool,
) -> bool {
    !focus_moved_into_menu && focus_came_from_menu
}

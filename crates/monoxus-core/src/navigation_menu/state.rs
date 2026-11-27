use crate::foundation::{
    overlay::{DismissLayer, Presence},
    roving_focus::RovingFocusConfig,
    shared::{Direction, Orientation, ScopeHandle},
};

pub use ui_sdk::navigation_menu::state::{
    NavigationMenuFocusProxyDecision, NavigationMenuHoverAction, NavigationMenuHoverMachine,
    NavigationMenuHoverTransition, NavigationMenuTimerCommand, build_indicator_style_with_prefix,
    build_viewport_style_with_prefix, compute_motion_direction, is_content_entry_key,
    is_item_hover_enabled, normalize_navigation_menu_orientation, orientation_str,
    resolve_content_arrow_target, resolve_focus_proxy_decision, should_dismiss_on_focus_outside,
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
        NavigationMenuActivationMode, NavigationMenuGeometryVars, NavigationMenuIndicatorPosition,
        NavigationMenuMotion, NavigationMenuPart, NavigationMenuViewportSize,
    },
};

// [KEEP-MONOXUS-BRAND] Compat state wrapper preserving `data_monoxus_navigation_menu_*` attrs
// and dual `--monoxus-navigation-menu-*` / `--radix-navigation-menu-*` inline styles.
#[derive(Clone, Debug, PartialEq)]
pub struct NavigationMenuDef(ui_sdk::navigation_menu::state::NavigationMenuDef);

impl From<ui_sdk::navigation_menu::state::NavigationMenuDef> for NavigationMenuDef {
    fn from(inner: ui_sdk::navigation_menu::state::NavigationMenuDef) -> Self {
        Self(inner)
    }
}

impl NavigationMenuDef {
    pub fn new(scope: ScopeHandle) -> Self {
        Self(ui_sdk::navigation_menu::state::NavigationMenuDef::new(scope))
    }

    pub fn as_inner(&self) -> &ui_sdk::navigation_menu::state::NavigationMenuDef {
        &self.0
    }

    pub fn into_inner(self) -> ui_sdk::navigation_menu::state::NavigationMenuDef {
        self.0
    }

    pub const fn parts() -> &'static [NavigationMenuPart] {
        ui_sdk::navigation_menu::state::NavigationMenuDef::parts()
    }

    pub fn scope(&self) -> &ScopeHandle {
        self.0.scope()
    }

    pub fn relationships(&self) -> &NavigationMenuRelationships {
        self.0.relationships()
    }

    pub const fn is_root_menu(&self) -> bool {
        self.0.is_root_menu()
    }

    pub fn value(&self) -> &str {
        self.0.value()
    }

    pub fn previous_value(&self) -> &str {
        self.0.previous_value()
    }

    pub fn is_open(&self) -> bool {
        self.0.is_open()
    }

    pub const fn orientation(&self) -> Orientation {
        self.0.orientation()
    }

    pub const fn dir(&self) -> Direction {
        self.0.dir()
    }

    pub const fn activation_mode(&self) -> NavigationMenuActivationMode {
        self.0.activation_mode()
    }

    pub const fn delay_duration_ms(&self) -> u64 {
        self.0.delay_duration_ms()
    }

    pub const fn skip_delay_duration_ms(&self) -> u64 {
        self.0.skip_delay_duration_ms()
    }

    pub const fn close_grace_ms(&self) -> u64 {
        self.0.close_grace_ms()
    }

    pub fn aria_label(&self) -> Option<&str> {
        self.0.aria_label()
    }

    pub const fn disable_toggle(&self) -> bool {
        self.0.disable_toggle()
    }

    pub fn presence(&self) -> &Presence {
        self.0.presence()
    }

    pub fn dismiss_layer(&self) -> &DismissLayer<String> {
        self.0.dismiss_layer()
    }

    pub fn with_is_root_menu(self, is_root_menu: bool) -> Self {
        Self(self.0.with_is_root_menu(is_root_menu))
    }

    pub fn with_value(self, value: impl Into<String>) -> Self {
        Self(self.0.with_value(value))
    }

    pub fn with_previous_value(self, previous_value: impl Into<String>) -> Self {
        Self(self.0.with_previous_value(previous_value))
    }

    pub fn with_orientation(self, orientation: Orientation) -> Self {
        Self(self.0.with_orientation(orientation))
    }

    pub fn with_dir(self, dir: Direction) -> Self {
        Self(self.0.with_dir(dir))
    }

    pub fn with_activation_mode(self, mode: NavigationMenuActivationMode) -> Self {
        Self(self.0.with_activation_mode(mode))
    }

    pub fn with_delay_duration_ms(self, delay_duration_ms: u64) -> Self {
        Self(self.0.with_delay_duration_ms(delay_duration_ms))
    }

    pub fn with_skip_delay_duration_ms(self, skip_delay_duration_ms: u64) -> Self {
        Self(self.0.with_skip_delay_duration_ms(skip_delay_duration_ms))
    }

    pub fn with_close_grace_ms(self, close_grace_ms: u64) -> Self {
        Self(self.0.with_close_grace_ms(close_grace_ms))
    }

    pub fn with_aria_label(self, aria_label: Option<String>) -> Self {
        Self(self.0.with_aria_label(aria_label))
    }

    pub fn with_disable_toggle(self, disable_toggle: bool) -> Self {
        Self(self.0.with_disable_toggle(disable_toggle))
    }

    pub const fn list_roving_config(&self) -> RovingFocusConfig {
        self.0.list_roving_config()
    }

    pub const fn content_roving_config(&self) -> RovingFocusConfig {
        self.0.content_roving_config()
    }

    pub fn toggle_value(&self, item_value: &str) -> String {
        self.0.toggle_value(item_value)
    }

    pub fn root_attributes(&self, id_override: Option<String>) -> NavigationMenuRootAttributes {
        self.0.root_attributes(id_override).into()
    }

    pub fn sub_attributes(&self, id_override: Option<String>) -> NavigationMenuSubAttributes {
        self.0.sub_attributes(id_override).into()
    }

    pub fn list_track_attributes(
        &self,
        id_override: Option<String>,
    ) -> NavigationMenuListTrackAttributes {
        self.0.list_track_attributes(id_override).into()
    }

    pub fn list_attributes(&self, id_override: Option<String>) -> NavigationMenuListAttributes {
        self.0.list_attributes(id_override).into()
    }

    pub fn item_attributes(
        &self,
        value: &str,
        id_override: Option<String>,
    ) -> NavigationMenuItemAttributes {
        self.0.item_attributes(value, id_override).into()
    }

    pub fn trigger_attributes(
        &self,
        value: &str,
        disabled: bool,
        id_override: Option<String>,
        content_id_override: Option<String>,
    ) -> NavigationMenuTriggerAttributes {
        self.0
            .trigger_attributes(value, disabled, id_override, content_id_override)
            .into()
    }

    pub fn focus_proxy_attributes(&self, value: &str) -> NavigationMenuFocusProxyAttributes {
        self.0.focus_proxy_attributes(value)
    }

    pub fn aria_owns_attributes(
        &self,
        value: &str,
        content_id_override: Option<String>,
    ) -> NavigationMenuAriaOwnsAttributes {
        self.0.aria_owns_attributes(value, content_id_override)
    }

    #[allow(clippy::too_many_arguments)]
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
        self.0
            .content_attributes(
                value,
                open,
                motion,
                inline_mode,
                id_override,
                trigger_id_override,
                user_style,
            )
            .into()
    }

    #[allow(clippy::too_many_arguments)]
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
        self.0
            .link_attributes(key, href, target, rel, active, focused, id_override)
            .into()
    }

    pub fn indicator_attributes(
        &self,
        is_visible: bool,
        position: NavigationMenuIndicatorPosition,
        id_override: Option<String>,
        user_style: Option<&str>,
    ) -> NavigationMenuIndicatorAttributes {
        let mut attrs: NavigationMenuIndicatorAttributes = self
            .0
            .indicator_attributes(is_visible, position, id_override, user_style)
            .into();
        attrs.style = build_indicator_style(self.orientation(), position, user_style);
        attrs
    }

    pub fn viewport_attributes(
        &self,
        open: bool,
        size: Option<NavigationMenuViewportSize>,
        id_override: Option<String>,
        user_style: Option<&str>,
    ) -> NavigationMenuViewportAttributes {
        let mut attrs: NavigationMenuViewportAttributes = self
            .0
            .viewport_attributes(open, size, id_override, user_style)
            .into();
        attrs.style = build_viewport_style(open, self.is_root_menu(), size, user_style);
        attrs
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
            inner: NavigationMenuDef::from(
                ui_sdk::navigation_menu::state::NavigationMenuSubDef::new(
                    scope,
                    inherited_dir,
                    inherited_activation_mode,
                )
                .into_def(),
            ),
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

// [KEEP-MONOXUS-BRAND] Emit both `--monoxus-navigation-menu-*` and `--radix-navigation-menu-*`.
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

// [KEEP-MONOXUS-BRAND] Emit both `--monoxus-navigation-menu-*` and `--radix-navigation-menu-*`.
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

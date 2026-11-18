use crate::foundation::{
    roving_focus::{
        self, Direction, Orientation, RovingCandidate, RovingFocusConfig, RovingModifiers,
        compute_tab_index, is_navigation_key_for_orientation, resolve_next_stop_in_registry,
    },
    shared::{CollectionRegistry, ScopeHandle, data_disabled_attr},
    state::DataState,
};

use super::{
    attrs::{ToggleGroupItemAttributes, ToggleGroupRootAttributes},
    relationships::ToggleGroupRelationships,
    types::{
        TOGGLE_GROUP_PARTS, ToggleGroupItemRegistration, ToggleGroupMode, ToggleGroupOrientation,
        ToggleGroupPart,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleGroup {
    relationships: ToggleGroupRelationships,
    mode: ToggleGroupMode,
    orientation: ToggleGroupOrientation,
    dir: Direction,
    roving_focus: bool,
    loop_focus: bool,
    disabled: bool,
    single_value: String,
    multiple_values: Vec<String>,
    current_tab_stop: Option<String>,
}

impl ToggleGroup {
    pub fn new(scope: ScopeHandle, mode: ToggleGroupMode) -> Self {
        Self {
            relationships: ToggleGroupRelationships::new(scope),
            mode,
            orientation: ToggleGroupOrientation::Horizontal,
            dir: Direction::Ltr,
            roving_focus: true,
            loop_focus: true,
            disabled: false,
            single_value: String::new(),
            multiple_values: Vec::new(),
            current_tab_stop: None,
        }
    }

    pub const fn parts() -> &'static [ToggleGroupPart] {
        &TOGGLE_GROUP_PARTS
    }

    pub fn relationships(&self) -> &ToggleGroupRelationships {
        &self.relationships
    }

    pub fn mode(&self) -> ToggleGroupMode {
        self.mode
    }

    pub fn orientation(&self) -> ToggleGroupOrientation {
        self.orientation
    }

    pub fn dir(&self) -> Direction {
        self.dir
    }

    pub fn roving_focus(&self) -> bool {
        self.roving_focus
    }

    pub fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn single_value(&self) -> &str {
        &self.single_value
    }

    pub fn multiple_values(&self) -> &[String] {
        &self.multiple_values
    }

    pub fn current_tab_stop(&self) -> Option<&str> {
        self.current_tab_stop.as_deref()
    }

    pub const fn roving_config(&self) -> RovingFocusConfig {
        RovingFocusConfig::new()
            .with_orientation(match self.orientation {
                ToggleGroupOrientation::Horizontal => Orientation::Horizontal,
                ToggleGroupOrientation::Vertical => Orientation::Vertical,
            })
            .with_direction(self.dir)
            .with_loop_focus(self.loop_focus)
            .with_enabled(self.roving_focus)
    }

    pub fn with_orientation(mut self, orientation: ToggleGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_direction(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }

    pub fn with_roving_focus(mut self, roving_focus: bool) -> Self {
        self.roving_focus = roving_focus;
        self
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_single_value(mut self, value: impl Into<String>) -> Self {
        self.single_value = value.into();
        self
    }

    pub fn with_multiple_values(mut self, values: Vec<String>) -> Self {
        self.multiple_values = values;
        self
    }

    pub fn with_current_tab_stop(mut self, tab_stop: Option<String>) -> Self {
        self.current_tab_stop = tab_stop;
        self
    }

    pub fn set_current_tab_stop(&mut self, tab_stop: Option<String>) {
        self.current_tab_stop = tab_stop;
    }

    pub fn is_item_pressed(&self, value: &str) -> bool {
        match self.mode {
            ToggleGroupMode::Single => !self.single_value.is_empty() && self.single_value == value,
            ToggleGroupMode::Multiple => self.multiple_values.iter().any(|v| v == value),
        }
    }

    pub fn is_item_disabled(&self, item_disabled: bool) -> bool {
        self.disabled || item_disabled
    }

    pub fn toggle_single(&mut self, value: &str) -> String {
        if self.single_value == value {
            self.single_value.clear();
        } else {
            self.single_value = value.to_string();
        }
        self.current_tab_stop = Some(value.to_string());
        self.single_value.clone()
    }

    pub fn toggle_multiple(&mut self, value: &str) -> Vec<String> {
        if let Some(pos) = self.multiple_values.iter().position(|v| v == value) {
            self.multiple_values.remove(pos);
        } else {
            self.multiple_values.push(value.to_string());
        }
        self.current_tab_stop = Some(value.to_string());
        self.multiple_values.clone()
    }

    pub fn resolve_tab_stop(
        &self,
        registry: &CollectionRegistry<String, ToggleGroupItemRegistration>,
     ) -> Option<String> {
        if self.disabled {
            return None;
        }

        let candidates: Vec<RovingCandidate> = registry
            .iter()
            .map(|entry| {
                let reg = entry.value();
                RovingCandidate::new(
                    entry.key().clone(),
                    reg.id.clone(),
                    !self.is_item_disabled(reg.disabled),
                    self.is_item_pressed(entry.key()),
                )
            })
            .collect();

        roving_focus::resolve_tab_stop(self.current_tab_stop.as_deref(), &candidates)
    }

    pub fn tabindex_for_item(
        &self,
        value: &str,
        item_disabled: bool,
        resolved_tab_stop: Option<&str>,
    ) -> i32 {
        compute_tab_index(
            &self.roving_config(),
            value,
            !self.is_item_disabled(item_disabled),
            resolved_tab_stop,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn navigate_key(
        &self,
        registry: &CollectionRegistry<String, ToggleGroupItemRegistration>,
        current_value: &str,
        key: &str,
        alt_key: bool,
        ctrl_key: bool,
        meta_key: bool,
        shift_key: bool,
    ) -> Option<String> {
        let config = self.roving_config();
        if !is_navigation_key_for_orientation(config.orientation, key) {
            return None;
        }

        resolve_next_stop_in_registry(
            &config,
            registry,
            Some(current_value),
            key,
            RovingModifiers::new(alt_key, ctrl_key, meta_key, shift_key),
            |reg| !self.is_item_disabled(reg.disabled),
        )
    }

    pub fn root_attributes(&self, custom_role: Option<&'static str>) -> ToggleGroupRootAttributes {
        let role = custom_role.unwrap_or(match self.mode {
            ToggleGroupMode::Single => "radiogroup",
            ToggleGroupMode::Multiple => "group",
        });
        ToggleGroupRootAttributes {
            id: self.relationships.root_id().to_string(),
            role,
            dir: self.dir.as_str(),
            data_orientation: self.orientation.as_str(),
            data_disabled: data_disabled_attr(self.disabled),
            data_toggle_group_root: "",
        }
    }

    pub fn item_attributes(
        &self,
        value: &str,
        item_id: Option<String>,
        item_disabled: bool,
        resolved_tab_stop: Option<&str>,
    ) -> ToggleGroupItemAttributes {
        let effective_disabled = self.disabled || item_disabled;
        let is_pressed = self.is_item_pressed(value);
        let id = item_id.unwrap_or_else(|| self.relationships.item_id(value));
        let tabindex = self.tabindex_for_item(value, item_disabled, resolved_tab_stop);

        let (role, aria_checked, aria_pressed) = match self.mode {
            ToggleGroupMode::Single => (
                Some("radio"),
                Some(if is_pressed { "true" } else { "false" }),
                None,
            ),
            ToggleGroupMode::Multiple => (
                None,
                None,
                Some(if is_pressed { "true" } else { "false" }),
            ),
        };

        ToggleGroupItemAttributes {
            id,
            role,
            aria_checked,
            aria_pressed,
            tabindex,
            disabled: effective_disabled,
            data_disabled: data_disabled_attr(effective_disabled),
            data_state: if is_pressed {
                DataState::On
            } else {
                DataState::Off
            },
            data_value: value.to_string(),
            data_orientation: self.orientation.as_str(),
            data_toggle_group_item: "",
        }
    }
}

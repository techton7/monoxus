use crate::foundation::{
    roving_focus::{
        self, Direction, Orientation, RovingCandidate, RovingFocusConfig, RovingModifiers,
        resolve_next_stop_in_registry,
    },
    shared::{CollectionRegistry, ScopeHandle, data_disabled_attr},
    state::DataState,
};

use super::{
    attrs::{RadioGroupIndicatorAttributes, RadioGroupItemAttributes, RadioGroupRootAttributes},
    relationships::RadioGroupRelationships,
    types::{
        RADIO_GROUP_PARTS, RadioGroupItemRegistration, RadioGroupOrientation, RadioGroupPart,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioGroup {
    relationships: RadioGroupRelationships,
    orientation: Option<RadioGroupOrientation>,
    dir: Direction,
    loop_focus: bool,
    disabled: bool,
    required: bool,
    read_only: bool,
    value: Option<String>,
    current_tab_stop: Option<String>,
}

impl RadioGroup {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            relationships: RadioGroupRelationships::new(scope),
            orientation: None, // Default: None -> Orientation::Both
            dir: Direction::Ltr,
            loop_focus: true,
            disabled: false,
            required: false,
            read_only: false,
            value: None,
            current_tab_stop: None,
        }
    }

    pub const fn parts() -> &'static [RadioGroupPart] {
        &RADIO_GROUP_PARTS
    }

    pub fn relationships(&self) -> &RadioGroupRelationships {
        &self.relationships
    }

    pub fn orientation(&self) -> Option<RadioGroupOrientation> {
        self.orientation
    }

    pub fn dir(&self) -> Direction {
        self.dir
    }

    pub fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn current_tab_stop(&self) -> Option<&str> {
        self.current_tab_stop.as_deref()
    }

    pub fn with_orientation(mut self, orientation: Option<RadioGroupOrientation>) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_direction(mut self, dir: Direction) -> Self {
        self.dir = dir;
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

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_value(mut self, value: Option<String>) -> Self {
        self.value = value.filter(|v| !v.is_empty());
        self
    }

    pub fn with_current_tab_stop(mut self, tab_stop: Option<String>) -> Self {
        self.current_tab_stop = tab_stop.filter(|s| !s.is_empty());
        self
    }

    pub fn set_current_tab_stop(&mut self, tab_stop: Option<String>) {
        self.current_tab_stop = tab_stop.filter(|s| !s.is_empty());
    }

    pub fn set_value(&mut self, value: Option<String>) {
        self.value = value.filter(|v| !v.is_empty());
    }

    pub fn is_item_checked(&self, item_value: &str) -> bool {
        self.value.as_deref() == Some(item_value)
    }

    pub fn is_item_disabled(&self, item_disabled: bool) -> bool {
        self.disabled || item_disabled
    }

    /// Irreversible item activation (RG-2):
    /// Returns `true` if state changed to checked.
    /// Re-clicking or selecting an already-checked item is a no-op (returns `false`).
    /// If disabled or read_only, returns `false`.
    pub fn check_item(&mut self, item_value: &str) -> bool {
        if self.disabled || self.read_only {
            return false;
        }

        if self.value.as_deref() == Some(item_value) {
            // Already checked; cannot uncheck (RG-2)
            return false;
        }

        self.value = Some(item_value.to_string());
        self.current_tab_stop = Some(item_value.to_string());
        true
    }

    pub const fn roving_config(&self) -> RovingFocusConfig {
        let mapped_orientation = match self.orientation {
            None => Orientation::Both,
            Some(RadioGroupOrientation::Horizontal) => Orientation::Horizontal,
            Some(RadioGroupOrientation::Vertical) => Orientation::Vertical,
        };
        RovingFocusConfig::new()
            .with_orientation(mapped_orientation)
            .with_direction(self.dir)
            .with_loop_focus(self.loop_focus)
            .with_enabled(!self.disabled)
    }

    pub fn resolve_tab_stop<V>(
        &self,
        registry: &CollectionRegistry<String, V>,
        is_focusable: impl Fn(&V) -> bool,
    ) -> Option<String> {
        if self.disabled {
            return None;
        }

        let candidates: Vec<RovingCandidate> = registry
            .iter()
            .map(|entry| {
                RovingCandidate::new(
                    entry.key().clone(),
                    String::new(),
                    is_focusable(entry.value()),
                    self.is_item_checked(entry.key()),
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
        if self.is_item_disabled(item_disabled) {
            -1
        } else if resolved_tab_stop == Some(value) {
            0
        } else {
            -1
        }
    }

    pub fn root_attributes(&self, custom_role: Option<&'static str>) -> RadioGroupRootAttributes {
        let orientation_str = self.orientation.map(|o| o.as_str());
        RadioGroupRootAttributes {
            id: self.relationships.root_id().to_string(),
            role: custom_role.unwrap_or("radiogroup"),
            dir: self.dir.as_str(),
            aria_orientation: orientation_str,
            aria_required: if self.required { Some("true") } else { None },
            aria_disabled: if self.disabled { Some("true") } else { None },
            aria_readonly: if self.read_only { Some("true") } else { None },
            aria_invalid: None,
            aria_describedby: None,
            data_orientation: orientation_str,
            data_disabled: data_disabled_attr(self.disabled),
            data_required: if self.required { Some("") } else { None },
            data_readonly: if self.read_only { Some("") } else { None },
            data_invalid: None,
            data_radio_group_root: "",
        }
    }

    pub fn item_attributes(
        &self,
        value: &str,
        item_id: Option<String>,
        item_disabled: bool,
        resolved_tab_stop: Option<&str>,
    ) -> RadioGroupItemAttributes {
        let effective_disabled = self.is_item_disabled(item_disabled);
        let checked = self.is_item_checked(value);
        let tabindex = self.tabindex_for_item(value, item_disabled, resolved_tab_stop);
        let orientation_str = self.orientation.map(|o| o.as_str());

        RadioGroupItemAttributes {
            id: item_id.unwrap_or_else(|| self.relationships.item_id(value)),
            role: "radio",
            aria_checked: if checked { "true" } else { "false" },
            tabindex,
            disabled: effective_disabled,
            data_state: if checked {
                DataState::Checked
            } else {
                DataState::Unchecked
            },
            data_disabled: data_disabled_attr(effective_disabled),
            data_readonly: if self.read_only { Some("") } else { None },
            data_invalid: None,
            data_orientation: orientation_str,
            data_value: value.to_string(),
            data_radio_group_item: "",
        }
    }

    pub fn indicator_attributes(
        &self,
        value: &str,
        indicator_id: Option<String>,
        item_disabled: bool,
    ) -> RadioGroupIndicatorAttributes {
        let effective_disabled = self.is_item_disabled(item_disabled);
        let checked = self.is_item_checked(value);
        let orientation_str = self.orientation.map(|o| o.as_str());

        RadioGroupIndicatorAttributes {
            id: indicator_id.unwrap_or_else(|| self.relationships.indicator_id(value)),
            data_state: if checked {
                DataState::Checked
            } else {
                DataState::Unchecked
            },
            data_disabled: data_disabled_attr(effective_disabled),
            data_readonly: if self.read_only { Some("") } else { None },
            data_orientation: orientation_str,
            data_radio_group_indicator: "",
            style: "pointer-events: none;",
        }
    }

    pub fn navigate_key(
        &self,
        registry: &CollectionRegistry<String, RadioGroupItemRegistration>,
        current_value: &str,
        key: &str,
        alt: bool,
        ctrl: bool,
        meta: bool,
        shift: bool,
    ) -> Option<String> {
        let config = self.roving_config();
        let modifiers = RovingModifiers::new(alt, ctrl, meta, shift);
        resolve_next_stop_in_registry(
            &config,
            registry,
            Some(current_value),
            key,
            modifiers,
            |reg| !self.is_item_disabled(reg.disabled),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_registry() -> CollectionRegistry<String, RadioGroupItemRegistration> {
        let mut reg = CollectionRegistry::new();
        reg.register(
            "r1".to_string(),
            RadioGroupItemRegistration {
                value: "r1".to_string(),
                id: "id-r1".to_string(),
                disabled: false,
            },
        );
        reg.register(
            "r2".to_string(),
            RadioGroupItemRegistration {
                value: "r2".to_string(),
                id: "id-r2".to_string(),
                disabled: true, // disabled item
            },
        );
        reg.register(
            "r3".to_string(),
            RadioGroupItemRegistration {
                value: "r3".to_string(),
                id: "id-r3".to_string(),
                disabled: false,
            },
        );
        reg.register(
            "r4".to_string(),
            RadioGroupItemRegistration {
                value: "r4".to_string(),
                id: "id-r4".to_string(),
                disabled: false,
            },
        );
        reg
    }

    #[test]
    fn test_radio_group_irreversible_selection() {
        let scope = ScopeHandle::root("rg-test-irrev");
        let mut rg = RadioGroup::new(scope);

        assert_eq!(rg.value(), None);
        assert!(!rg.is_item_checked("r1"));

        // First check -> true
        assert!(rg.check_item("r1"));
        assert_eq!(rg.value(), Some("r1"));
        assert_eq!(rg.current_tab_stop(), Some("r1"));
        assert!(rg.is_item_checked("r1"));

        // Second check on already-checked item -> false (no-op, never unchecks)
        assert!(!rg.check_item("r1"));
        assert_eq!(rg.value(), Some("r1"));

        // Selecting a different item -> true
        assert!(rg.check_item("r3"));
        assert_eq!(rg.value(), Some("r3"));
        assert!(!rg.is_item_checked("r1"));
        assert!(rg.is_item_checked("r3"));
    }

    #[test]
    fn test_radio_group_disabled_and_readonly_guards() {
        let scope = ScopeHandle::root("rg-test-guards");
        let mut rg = RadioGroup::new(scope).with_disabled(true);

        // Disabled group blocks selection
        assert!(!rg.check_item("r1"));
        assert_eq!(rg.value(), None);

        // Read-only group blocks selection
        let mut rg_ro = RadioGroup::new(ScopeHandle::root("rg-test-ro")).with_read_only(true);
        assert!(!rg_ro.check_item("r1"));
        assert_eq!(rg_ro.value(), None);
    }

    #[test]
    fn test_radio_group_roving_both_default_and_rtl() {
        let scope = ScopeHandle::root("rg-test-roving-both");
        let rg = RadioGroup::new(scope).with_loop_focus(true); // default orientation is None -> Both
        let reg = create_sample_registry();

        // ArrowRight advances, skipping disabled r2 to r3
        assert_eq!(
            rg.navigate_key(&reg, "r1", "ArrowRight", false, false, false, false),
            Some("r3".to_string())
        );
        // ArrowDown also advances (Both orientation)
        assert_eq!(
            rg.navigate_key(&reg, "r1", "ArrowDown", false, false, false, false),
            Some("r3".to_string())
        );
        // ArrowLeft loops back to r4
        assert_eq!(
            rg.navigate_key(&reg, "r1", "ArrowLeft", false, false, false, false),
            Some("r4".to_string())
        );
        // ArrowUp also loops back to r4
        assert_eq!(
            rg.navigate_key(&reg, "r1", "ArrowUp", false, false, false, false),
            Some("r4".to_string())
        );

        // RTL flips ArrowLeft and ArrowRight
        let rg_rtl = RadioGroup::new(ScopeHandle::root("rg-test-rtl"))
            .with_direction(Direction::Rtl)
            .with_loop_focus(true);

        // In RTL, ArrowRight goes previous (loops to r4)
        assert_eq!(
            rg_rtl.navigate_key(&reg, "r1", "ArrowRight", false, false, false, false),
            Some("r4".to_string())
        );
        // In RTL, ArrowLeft goes next (skips disabled r2 to r3)
        assert_eq!(
            rg_rtl.navigate_key(&reg, "r1", "ArrowLeft", false, false, false, false),
            Some("r3".to_string())
        );
    }

    #[test]
    fn test_radio_group_tab_stop_priority() {
        let scope = ScopeHandle::root("rg-test-tabstop");
        let reg = create_sample_registry();

        // 1. Unselected group with no current_tab_stop -> falls back to first focusable item "r1"
        let rg_unselected = RadioGroup::new(scope.clone());
        let tab_stop = rg_unselected.resolve_tab_stop(&reg, |r| !r.disabled);
        assert_eq!(tab_stop, Some("r1".to_string()));

        // 2. Group with current_tab_stop "r4" -> resolves to "r4"
        let rg_with_curr = RadioGroup::new(scope.clone()).with_current_tab_stop(Some("r4".to_string()));
        let tab_stop = rg_with_curr.resolve_tab_stop(&reg, |r| !r.disabled);
        assert_eq!(tab_stop, Some("r4".to_string()));

        // 3. Group with checked item "r3" and no current_tab_stop -> resolves to checked "r3"
        let rg_checked = RadioGroup::new(scope.clone()).with_value(Some("r3".to_string()));
        let tab_stop = rg_checked.resolve_tab_stop(&reg, |r| !r.disabled);
        assert_eq!(tab_stop, Some("r3".to_string()));

        // 4. Disabled group -> resolves to None
        let rg_disabled = RadioGroup::new(scope).with_disabled(true);
        let tab_stop = rg_disabled.resolve_tab_stop(&reg, |r| !r.disabled);
        assert_eq!(tab_stop, None);
    }
}

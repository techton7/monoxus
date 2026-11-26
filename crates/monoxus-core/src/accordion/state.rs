use crate::foundation::{
    roving_focus::{
        RovingCandidate, RovingFocusConfig, RovingModifiers, is_navigation_key,
        is_navigation_key_for_orientation, resolve_next_stop,
    },
    shared::ScopeHandle,
    state::DataState,
};

use super::{
    attrs::{
        AccordionContentAttributes, AccordionHeaderAttributes, AccordionItemAttributes,
        AccordionRootAttributes, AccordionTriggerAttributes,
    },
    relationships::AccordionRelationships,
    types::{
        ACCORDION_PARTS, AccordionDirection, AccordionItemRegistration, AccordionMode,
        AccordionOrientation, AccordionPart,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accordion {
    relationships: AccordionRelationships,
    mode: AccordionMode,
    orientation: AccordionOrientation,
    dir: AccordionDirection,
    loop_focus: bool,
    disabled: bool,
    active_values: Vec<String>,
    current_tab_stop: String,
}

impl Accordion {
    pub fn new(scope: ScopeHandle, mode: AccordionMode) -> Self {
        Self {
            relationships: AccordionRelationships::new(scope),
            mode,
            orientation: AccordionOrientation::Vertical,
            dir: AccordionDirection::Ltr,
            loop_focus: true,
            disabled: false,
            active_values: Vec::new(),
            current_tab_stop: String::new(),
        }
    }

    pub const fn parts() -> &'static [AccordionPart] {
        &ACCORDION_PARTS
    }

    pub fn relationships(&self) -> &AccordionRelationships {
        &self.relationships
    }

    pub fn mode(&self) -> &AccordionMode {
        &self.mode
    }

    pub fn orientation(&self) -> AccordionOrientation {
        self.orientation
    }

    pub fn dir(&self) -> AccordionDirection {
        self.dir
    }

    pub fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn active_values(&self) -> &[String] {
        &self.active_values
    }

    pub fn is_open(&self, value: &str) -> bool {
        self.active_values.iter().any(|v| v == value)
    }

    pub fn current_tab_stop(&self) -> &str {
        &self.current_tab_stop
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        let val = value.into();
        if val.is_empty() {
            self.active_values.clear();
        } else {
            self.active_values = vec![val.clone()];
            if self.current_tab_stop.is_empty() {
                self.current_tab_stop = val;
            }
        }
        self
    }

    pub fn with_values(mut self, values: Vec<String>) -> Self {
        if self.current_tab_stop.is_empty()
            && let Some(first) = values.iter().find(|v| !v.is_empty())
        {
            self.current_tab_stop = first.clone();
        }
        self.active_values = values;
        self
    }

    pub fn with_tab_stop(mut self, tab_stop: impl Into<String>) -> Self {
        self.current_tab_stop = tab_stop.into();
        self
    }

    pub fn with_orientation(mut self, orientation: AccordionOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_direction(mut self, dir: AccordionDirection) -> Self {
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

    pub fn set_tab_stop(&mut self, value: impl Into<String>) {
        self.current_tab_stop = value.into();
    }

    pub fn toggle_item(&mut self, value: &str) -> bool {
        let was_open = self.is_open(value);
        match self.mode {
            AccordionMode::Single { collapsible } => {
                if was_open {
                    if collapsible {
                        self.active_values.clear();
                        true
                    } else {
                        false
                    }
                } else {
                    self.active_values = vec![value.to_string()];
                    self.current_tab_stop = value.to_string();
                    true
                }
            }
            AccordionMode::Multiple => {
                if was_open {
                    self.active_values.retain(|v| v != value);
                } else {
                    self.active_values.push(value.to_string());
                    self.current_tab_stop = value.to_string();
                }
                true
            }
        }
    }

    pub fn roving_config(&self) -> RovingFocusConfig {
        RovingFocusConfig::new()
            .with_orientation(self.orientation.into())
            .with_direction(self.dir.into())
            .with_loop_focus(self.loop_focus)
            .with_enabled(!self.disabled)
    }

    pub fn is_navigation_key(key: &str) -> bool {
        is_navigation_key(key)
    }

    pub fn is_navigation_key_for_orientation(orientation: AccordionOrientation, key: &str) -> bool {
        is_navigation_key_for_orientation(orientation.into(), key)
    }

    pub fn resolve_key_navigation(
        &self,
        registered_items: &[AccordionItemRegistration],
        current_tab_stop: &str,
        key: &str,
    ) -> Option<String> {
        if self.disabled {
            return None;
        }
        let candidates: Vec<RovingCandidate> = registered_items
            .iter()
            .map(|item| {
                RovingCandidate::new(
                    item.value.clone(),
                    String::new(),
                    !item.disabled,
                    self.is_open(&item.value),
                )
            })
            .collect();
        let current = (!current_tab_stop.is_empty()).then_some(current_tab_stop);
        resolve_next_stop(
            &self.roving_config(),
            &candidates,
            current,
            key,
            RovingModifiers::none(),
        )
    }

    pub fn root(&self) -> AccordionRootAttributes {
        AccordionRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_orientation: self.orientation.as_str(),
            disabled: self.disabled,
        }
    }

    pub fn item(&self, value: &str, item_disabled: bool) -> AccordionItemAttributes {
        let is_open = self.is_open(value);
        let effective_disabled = self.disabled || item_disabled;
        let data_state = if is_open {
            DataState::Open
        } else {
            DataState::Closed
        };

        AccordionItemAttributes {
            id: self.relationships.item_id(value),
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
            disabled: effective_disabled,
        }
    }

    pub fn header(&self, value: &str, item_disabled: bool) -> AccordionHeaderAttributes {
        self.header_with_level(value, item_disabled, 3)
    }

    pub fn header_with_level(
        &self,
        value: &str,
        item_disabled: bool,
        level: u8,
    ) -> AccordionHeaderAttributes {
        let is_open = self.is_open(value);
        let effective_disabled = self.disabled || item_disabled;
        let data_state = if is_open {
            DataState::Open
        } else {
            DataState::Closed
        };

        let clamped = level.clamp(1, 6) as u32;

        AccordionHeaderAttributes {
            id: self.relationships.header_id(value),
            role: "heading",
            aria_level: clamped,
            data_heading_level: clamped,
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
            disabled: effective_disabled,
        }
    }

    pub fn trigger(&self, value: &str, trigger_disabled: bool) -> AccordionTriggerAttributes {
        let is_open = self.is_open(value);
        let is_tab_stop = self.current_tab_stop == value;
        let effective_disabled = self.disabled || trigger_disabled;
        let tabindex = if is_tab_stop && !effective_disabled {
            0
        } else {
            -1
        };
        let data_state = if is_open {
            DataState::Open
        } else {
            DataState::Closed
        };

        let aria_disabled = if !effective_disabled
            && is_open
            && matches!(self.mode, AccordionMode::Single { collapsible: false })
        {
            Some("true")
        } else {
            None
        };

        let content_id = self.relationships.content_id(value);
        let aria_controls = if is_open {
            Some(content_id.clone())
        } else {
            None
        };

        AccordionTriggerAttributes {
            id: self.relationships.trigger_id(value),
            role: "button",
            is_open,
            aria_expanded: if is_open { "true" } else { "false" },
            content_id,
            aria_controls,
            aria_disabled,
            tabindex,
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
            disabled: effective_disabled,
        }
    }

    pub fn content(&self, value: &str) -> AccordionContentAttributes {
        self.content_with_disabled(value, false)
    }

    pub fn content_with_disabled(
        &self,
        value: &str,
        item_disabled: bool,
    ) -> AccordionContentAttributes {
        let is_open = self.is_open(value);
        let effective_disabled = self.disabled || item_disabled;
        let data_state = if is_open {
            DataState::Open
        } else {
            DataState::Closed
        };

        AccordionContentAttributes {
            id: self.relationships.content_id(value),
            role: "region",
            aria_labelledby: self.relationships.trigger_id(value),
            hidden: !is_open,
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
            disabled: effective_disabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::shared::{Direction, Orientation};

    #[test]
    fn v_acc_1_part_inventory_matches_exhaustive_surface() {
        let parts: Vec<_> = Accordion::parts()
            .iter()
            .map(AccordionPart::as_str)
            .collect();
        assert_eq!(parts, vec!["root", "item", "header", "trigger", "content"]);
        assert_eq!(ACCORDION_PARTS.len(), 5);
    }

    #[test]
    fn v_acc_1_relationships_produce_deterministic_collision_safe_ids() {
        let scope = ScopeHandle::root("accordion-test").child("scope");
        let relationships = AccordionRelationships::new(scope.clone());

        assert_eq!(relationships.scope(), &scope);
        assert_eq!(relationships.root_id(), scope.token());
        assert_eq!(
            relationships.item_id("item-1"),
            scope.qualify("item-item-1")
        );
        assert_eq!(
            relationships.header_id("item-1"),
            scope.qualify("header-item-1")
        );
        assert_eq!(
            relationships.trigger_id("item-1"),
            scope.qualify("trigger-item-1")
        );
        assert_eq!(
            relationships.content_id("item-1"),
            scope.qualify("content-item-1")
        );

        let rel_a = AccordionRelationships::new(ScopeHandle::root("a|b").child("c"));
        let rel_b = AccordionRelationships::new(ScopeHandle::root("a").child("b|c"));
        assert_ne!(rel_a.root_id(), rel_b.root_id());
        assert_ne!(rel_a.trigger_id("x"), rel_b.trigger_id("x"));
    }

    #[test]
    fn v_acc_1_orientation_and_direction_conversions() {
        assert_eq!(
            AccordionOrientation::default(),
            AccordionOrientation::Vertical
        );
        assert_eq!(AccordionOrientation::Vertical.as_str(), "vertical");
        assert!(AccordionOrientation::Vertical.is_vertical());
        assert!(!AccordionOrientation::Vertical.is_horizontal());
        assert_eq!(AccordionOrientation::Horizontal.as_str(), "horizontal");
        assert!(AccordionOrientation::Horizontal.is_horizontal());
        assert!(!AccordionOrientation::Horizontal.is_vertical());
        assert_eq!(
            Orientation::from(AccordionOrientation::Vertical),
            Orientation::Vertical
        );
        assert_eq!(
            Orientation::from(AccordionOrientation::Horizontal),
            Orientation::Horizontal
        );

        assert_eq!(AccordionDirection::default(), AccordionDirection::Ltr);
        assert_eq!(AccordionDirection::Ltr.as_str(), "ltr");
        assert!(!AccordionDirection::Ltr.is_rtl());
        assert_eq!(AccordionDirection::Rtl.as_str(), "rtl");
        assert!(AccordionDirection::Rtl.is_rtl());
        assert_eq!(Direction::from(AccordionDirection::Rtl), Direction::Rtl);
        assert_eq!(
            AccordionDirection::from(Direction::Rtl),
            AccordionDirection::Rtl
        );
        assert_eq!(
            AccordionDirection::from(Direction::Ltr),
            AccordionDirection::Ltr
        );
    }

    #[test]
    fn accordion_empty_value_guard_and_content_disabled() {
        let scope = ScopeHandle::root("accordion").child("superset");
        let acc = Accordion::new(scope, AccordionMode::Single { collapsible: true })
            .with_value("")
            .with_values(vec![String::new(), "item-2".to_string()]);
        assert_eq!(acc.current_tab_stop(), "item-2");

        let cleared = Accordion::new(
            ScopeHandle::root("accordion").child("cleared"),
            AccordionMode::Single { collapsible: true },
        )
        .with_value("item-1")
        .with_value("");
        assert!(cleared.active_values().is_empty());

        let content = acc.content_with_disabled("item-2", true);
        assert!(content.is_disabled());
        assert_eq!(content.data_disabled(), Some(""));
    }
}

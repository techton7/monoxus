use crate::foundation::{
    shared::{CollectionRegistry, RovingFocusController, RovingFocusKey, ScopeHandle},
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
        self.active_values = vec![val.clone()];
        if self.current_tab_stop.is_empty() {
            self.current_tab_stop = val;
        }
        self
    }

    pub fn with_values(mut self, values: Vec<String>) -> Self {
        if self.current_tab_stop.is_empty() {
            if let Some(first) = values.first() {
                self.current_tab_stop = first.clone();
            }
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
                        // In single non-collapsible mode, open item cannot be collapsed
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

    pub fn is_navigation_key(key: &str) -> bool {
        matches!(
            key,
            "ArrowLeft"
                | "ArrowRight"
                | "ArrowUp"
                | "ArrowDown"
                | "Home"
                | "End"
                | "PageUp"
                | "PageDown"
        )
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

        let roving_key = match key {
            "ArrowLeft" => Some(RovingFocusKey::ArrowLeft),
            "ArrowRight" => Some(RovingFocusKey::ArrowRight),
            "ArrowUp" => Some(RovingFocusKey::ArrowUp),
            "ArrowDown" => Some(RovingFocusKey::ArrowDown),
            "Home" | "PageUp" => Some(RovingFocusKey::Home),
            "End" | "PageDown" => Some(RovingFocusKey::End),
            _ => None,
        }?;

        let mut registry = CollectionRegistry::new();
        for item in registered_items {
            registry.register(item.value.clone(), item.disabled);
        }

        let controller = RovingFocusController::new(self.dir.into())
            .with_orientation(self.orientation.into())
            .with_looping(self.loop_focus);

        controller.navigate_by_key(
            &registry,
            Some(&current_tab_stop.to_string()),
            roving_key,
            |&disabled| !disabled,
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
        let is_open = self.is_open(value);
        let effective_disabled = self.disabled || item_disabled;
        let data_state = if is_open {
            DataState::Open
        } else {
            DataState::Closed
        };

        AccordionHeaderAttributes {
            id: self.relationships.header_id(value),
            role: "heading",
            aria_level: 3,
            data_heading_level: 3,
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

        AccordionTriggerAttributes {
            id: self.relationships.trigger_id(value),
            role: "button",
            is_open,
            aria_expanded: if is_open { "true" } else { "false" },
            aria_controls: self.relationships.content_id(value),
            aria_disabled,
            tabindex,
            data_state,
            data_orientation: self.orientation.as_str(),
            data_value: value.to_owned(),
            disabled: effective_disabled,
        }
    }

    pub fn content(&self, value: &str) -> AccordionContentAttributes {
        let is_open = self.is_open(value);
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
        }
    }
}

use crate::foundation::{
    overlay::{PlacementAlign, PlacementSide, PortalHost},
    shared::ScopeHandle,
    state::DataState,
};

use super::{
    attrs::{
        SelectContentAttributes, SelectItemAttributes, SelectPortalAttributes,
        SelectRootAttributes, SelectTriggerAttributes,
    },
    relationships::SelectRelationships,
    types::{SelectItemData, SelectMode, SelectPart, SELECT_PARTS},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Select {
    scope: ScopeHandle,
    relationships: SelectRelationships,
    mode: SelectMode,
    value: Option<String>,
    default_value: Option<String>,
    values: Vec<String>,
    default_values: Vec<String>,
    open: bool,
    disabled: bool,
    required: bool,
    name: Option<String>,
    form: Option<String>,
    autocomplete: Option<String>,
    portal_host: PortalHost,
    items: Vec<SelectItemData>,
    loop_selection: bool,
}

impl Select {
    pub fn parts() -> &'static [SelectPart] {
        &SELECT_PARTS
    }

    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            relationships: SelectRelationships::new(scope.clone()),
            scope,
            mode: SelectMode::default(),
            value: None,
            default_value: None,
            values: Vec::new(),
            default_values: Vec::new(),
            open: false,
            disabled: false,
            required: false,
            name: None,
            form: None,
            autocomplete: None,
            portal_host: PortalHost::default_host(),
            items: Vec::new(),
            loop_selection: false,
        }
    }

    pub fn with_mode(mut self, mode: SelectMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_multiple(mut self, multiple: bool) -> Self {
        self.mode = if multiple {
            SelectMode::Multiple
        } else {
            SelectMode::Single {
                allow_deselect: false,
            }
        };
        self
    }

    pub fn with_value(mut self, val: Option<String>) -> Self {
        self.value = val.clone();
        if self.default_value.is_none() {
            self.default_value = val.clone();
            self.default_values = val.as_ref().map(|v| vec![v.clone()]).unwrap_or_default();
        }
        self.values = val.map(|v| vec![v]).unwrap_or_default();
        self
    }

    pub fn with_default_value(mut self, val: Option<String>) -> Self {
        self.default_value = val.clone();
        if self.value.is_none() {
            self.value = val.clone();
            self.values = val.map(|v| vec![v]).unwrap_or_default();
            self.default_values = self.values.clone();
        }
        self
    }

    pub fn with_values(mut self, vals: Vec<String>) -> Self {
        self.value = vals.first().cloned();
        if self.default_values.is_empty() {
            self.default_value = self.value.clone();
            self.default_values = vals.clone();
        }
        self.values = vals;
        self
    }

    pub fn with_default_values(mut self, vals: Vec<String>) -> Self {
        self.default_values = vals.clone();
        if self.values.is_empty() {
            self.value = vals.first().cloned();
            self.default_value = self.value.clone();
            self.values = vals;
        }
        self
    }

    pub fn with_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn with_allow_deselect(mut self, allow: bool) -> Self {
        self.mode = SelectMode::Single {
            allow_deselect: allow,
        };
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

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_form(mut self, form: impl Into<String>) -> Self {
        self.form = Some(form.into());
        self
    }

    pub fn with_autocomplete(mut self, autocomplete: impl Into<String>) -> Self {
        self.autocomplete = Some(autocomplete.into());
        self
    }

    pub fn with_items(mut self, items: Vec<SelectItemData>) -> Self {
        self.items = items;
        self
    }

    pub fn with_loop(mut self, loop_selection: bool) -> Self {
        self.loop_selection = loop_selection;
        self
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.portal_host = portal_host;
        self
    }

    pub fn mode(&self) -> SelectMode {
        self.mode
    }

    pub fn is_multiple(&self) -> bool {
        matches!(self.mode, SelectMode::Multiple)
    }

    pub fn portal_host(&self) -> &PortalHost {
        &self.portal_host
    }

    pub fn portal_attributes(&self) -> SelectPortalAttributes {
        SelectPortalAttributes {
            host: self.portal_host.clone(),
        }
    }

    pub fn relationships(&self) -> &SelectRelationships {
        &self.relationships
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn default_value(&self) -> Option<&str> {
        self.default_value.as_deref()
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }

    pub fn default_values(&self) -> &[String] {
        &self.default_values
    }

    pub fn items(&self) -> &[SelectItemData] {
        &self.items
    }

    pub fn loop_selection(&self) -> bool {
        self.loop_selection
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn allows_deselect(&self) -> bool {
        matches!(
            self.mode,
            SelectMode::Single {
                allow_deselect: true
            }
        )
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn form(&self) -> Option<&str> {
        self.form.as_deref()
    }

    pub fn autocomplete(&self) -> Option<&str> {
        self.autocomplete.as_deref()
    }

    pub fn root_attributes(&self) -> SelectRootAttributes {
        SelectRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_state: if self.open {
                DataState::Open
            } else {
                DataState::Closed
            },
            disabled: self.disabled,
        }
    }

    pub fn trigger_attributes(&self) -> SelectTriggerAttributes {
        let is_placeholder = match self.mode {
            SelectMode::Single { .. } => {
                self.value.is_none() || self.value.as_deref() == Some("")
            }
            SelectMode::Multiple => self.values.is_empty(),
        };

        SelectTriggerAttributes {
            id: self.relationships.trigger_id().to_owned(),
            role: "combobox",
            aria_haspopup: "listbox",
            aria_expanded: if self.open { "true" } else { "false" },
            aria_controls: self.relationships.content_id().to_owned(),
            aria_disabled: if self.disabled { Some("true") } else { None },
            aria_required: if self.required { Some("true") } else { None },
            data_state: if self.open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_placeholder: is_placeholder,
            disabled: self.disabled,
            tabindex: if self.disabled { -1 } else { 0 },
        }
    }

    pub fn content_attributes(&self, activedescendant: Option<String>) -> SelectContentAttributes {
        self.content_attributes_with_side_and_align(
            activedescendant,
            PlacementSide::Bottom,
            PlacementAlign::Start,
        )
    }

    pub fn content_attributes_with_side(
        &self,
        activedescendant: Option<String>,
        side: PlacementSide,
    ) -> SelectContentAttributes {
        self.content_attributes_with_side_and_align(
            activedescendant,
            side,
            PlacementAlign::Start,
        )
    }

    pub fn content_attributes_with_side_and_align(
        &self,
        activedescendant: Option<String>,
        side: PlacementSide,
        align: PlacementAlign,
    ) -> SelectContentAttributes {
        SelectContentAttributes {
            id: self.relationships.content_id().to_owned(),
            role: "listbox",
            tabindex: -1,
            aria_activedescendant: activedescendant,
            data_state: if self.open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_side: side,
            data_align: align,
            reference_hidden: false,
        }
    }

    pub fn item_attributes(
        &self,
        item_val: &str,
        is_highlighted: bool,
        disabled: bool,
    ) -> SelectItemAttributes {
        let is_selected = match self.mode {
            SelectMode::Single { .. } => {
                self.value.as_deref() == Some(item_val) && !item_val.is_empty()
            }
            SelectMode::Multiple => {
                self.values.iter().any(|v| v == item_val) && !item_val.is_empty()
            }
        };
        let eff_disabled = self.disabled || disabled;

        SelectItemAttributes {
            id: self.relationships.item_id(item_val),
            role: "option",
            aria_selected: if is_selected { "true" } else { "false" },
            aria_disabled: if eff_disabled { Some("true") } else { None },
            data_state: if is_selected { "checked" } else { "unchecked" },
            is_highlighted,
            disabled: eff_disabled,
            value: item_val.to_owned(),
            label: item_val.to_owned(),
        }
    }
}

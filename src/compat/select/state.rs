// [STAGE1-DROP]
use ui_sdk::foundation::{
    overlay::{DismissBehavior, PlacementAlign, PlacementSide, PortalHost, Presence},
    shared::ScopeHandle,
};

pub use ui_sdk::select::state::{SelectLifecycle, SelectToggleOutcome};

use super::{
    attrs::{
        SelectContentAttributes, SelectItemAttributes, SelectPortalAttributes,
        SelectRootAttributes, SelectTriggerAttributes,
    },
    registry::MountedOptionRegistry,
    relationships::SelectRelationships,
    types::{SelectItemData, SelectMode, SelectPart},
};

// [STAGE1-DROP] Compat wrapper around `ui_sdk::select::Select` preserving legacy `SelectMode` API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Select(ui_sdk::select::state::Select);

impl From<ui_sdk::select::state::Select> for Select {
    fn from(inner: ui_sdk::select::state::Select) -> Self {
        Self(inner)
    }
}

impl From<Select> for ui_sdk::select::state::Select {
    fn from(select: Select) -> Self {
        select.0
    }
}

impl Select {
    pub fn parts() -> &'static [SelectPart] {
        ui_sdk::select::state::Select::parts()
    }

    pub fn new(scope: ScopeHandle) -> Self {
        Self(ui_sdk::select::state::Select::new(scope))
    }

    pub fn as_inner(&self) -> &ui_sdk::select::state::Select {
        &self.0
    }

    pub fn into_inner(self) -> ui_sdk::select::state::Select {
        self.0
    }

    pub fn scope(&self) -> &ScopeHandle {
        self.0.scope()
    }

    pub fn lifecycle(&self) -> &SelectLifecycle {
        self.0.lifecycle()
    }

    pub fn lifecycle_mut(&mut self) -> &mut SelectLifecycle {
        self.0.lifecycle_mut()
    }

    pub fn with_lifecycle(self, lifecycle: SelectLifecycle) -> Self {
        Self(self.0.with_lifecycle(lifecycle))
    }

    pub fn with_trigger_id(self, trigger_id: impl Into<String>) -> Self {
        Self(self.0.with_trigger_id(trigger_id))
    }

    pub fn with_content_id(self, content_id: impl Into<String>) -> Self {
        Self(self.0.with_content_id(content_id))
    }

    pub fn with_mode(self, mode: SelectMode) -> Self {
        match mode {
            SelectMode::Single { allow_deselect } => Self(
                self.0
                    .with_mode(ui_sdk::select::SelectMode::Single)
                    .with_allow_deselect(allow_deselect),
            ),
            SelectMode::Multiple => Self(self.0.with_mode(ui_sdk::select::SelectMode::Multiple)),
        }
    }

    pub fn with_multiple(self, multiple: bool) -> Self {
        if multiple {
            Self(self.0.with_multiple(true))
        } else {
            Self(self.0.with_multiple(false).with_allow_deselect(false))
        }
    }

    pub fn with_allow_deselect(self, allow: bool) -> Self {
        Self(self.0.with_allow_deselect(allow))
    }

    pub fn with_value_controlled(self, controlled: bool) -> Self {
        Self(self.0.with_value_controlled(controlled))
    }

    pub fn with_values_controlled(self, controlled: bool) -> Self {
        Self(self.0.with_values_controlled(controlled))
    }

    pub fn with_value(self, val: Option<String>) -> Self {
        Self(self.0.with_value(val))
    }

    pub fn with_default_value(self, val: Option<String>) -> Self {
        Self(self.0.with_default_value(val))
    }

    pub fn with_values(self, vals: Vec<String>) -> Self {
        Self(self.0.with_values(vals))
    }

    pub fn with_default_values(self, vals: Vec<String>) -> Self {
        Self(self.0.with_default_values(vals))
    }

    pub fn with_open(self, open: bool) -> Self {
        Self(self.0.with_open(open))
    }

    pub fn with_retained_mount(self, retain_mount: bool) -> Self {
        Self(self.0.with_retained_mount(retain_mount))
    }

    pub fn with_presence(self, presence: Presence) -> Self {
        Self(self.0.with_presence(presence))
    }

    pub fn with_escape_key_down_behavior(self, behavior: DismissBehavior) -> Self {
        Self(self.0.with_escape_key_down_behavior(behavior))
    }

    pub fn with_interact_outside_behavior(self, behavior: DismissBehavior) -> Self {
        Self(self.0.with_interact_outside_behavior(behavior))
    }

    pub fn with_disabled(self, disabled: bool) -> Self {
        Self(self.0.with_disabled(disabled))
    }

    pub fn with_read_only(self, read_only: bool) -> Self {
        Self(self.0.with_read_only(read_only))
    }

    pub fn with_required(self, required: bool) -> Self {
        Self(self.0.with_required(required))
    }

    pub fn with_highlight_item_on_hover(self, highlight: bool) -> Self {
        Self(self.0.with_highlight_item_on_hover(highlight))
    }

    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self(self.0.with_name(name))
    }

    pub fn with_form(self, form: impl Into<String>) -> Self {
        Self(self.0.with_form(form))
    }

    pub fn with_autocomplete(self, autocomplete: impl Into<String>) -> Self {
        Self(self.0.with_autocomplete(autocomplete))
    }

    pub fn with_items(self, items: Vec<SelectItemData>) -> Self {
        Self(self.0.with_items(items))
    }

    pub fn with_loop(self, loop_selection: bool) -> Self {
        Self(self.0.with_loop(loop_selection))
    }

    pub fn with_portal_host(self, portal_host: PortalHost) -> Self {
        Self(self.0.with_portal_host(portal_host))
    }

    pub const fn mode(&self) -> SelectMode {
        match self.0.mode() {
            ui_sdk::select::SelectMode::Single => SelectMode::Single {
                allow_deselect: self.0.allows_deselect(),
            },
            ui_sdk::select::SelectMode::Multiple => SelectMode::Multiple,
        }
    }

    pub const fn is_multiple(&self) -> bool {
        self.0.is_multiple()
    }

    pub fn portal_host(&self) -> &PortalHost {
        self.0.portal_host()
    }

    pub fn presence(&self) -> &Presence {
        self.0.presence()
    }

    pub const fn escape_key_down_behavior(&self) -> DismissBehavior {
        self.0.escape_key_down_behavior()
    }

    pub const fn interact_outside_behavior(&self) -> DismissBehavior {
        self.0.interact_outside_behavior()
    }

    pub fn portal_attributes(&self) -> SelectPortalAttributes {
        self.0.portal_attributes()
    }

    pub fn relationships(&self) -> &SelectRelationships {
        self.0.relationships()
    }

    pub fn value(&self) -> Option<&str> {
        self.0.value()
    }

    pub fn default_value(&self) -> Option<&str> {
        self.0.default_value()
    }

    pub fn values(&self) -> &[String] {
        self.0.values()
    }

    pub fn default_values(&self) -> &[String] {
        self.0.default_values()
    }

    pub const fn is_value_controlled(&self) -> bool {
        self.0.is_value_controlled()
    }

    pub const fn is_values_controlled(&self) -> bool {
        self.0.is_values_controlled()
    }

    pub fn items(&self) -> &[SelectItemData] {
        self.0.items()
    }

    pub fn registry(&self) -> MountedOptionRegistry {
        self.0.registry()
    }

    pub const fn loop_selection(&self) -> bool {
        self.0.loop_selection()
    }

    pub const fn is_open(&self) -> bool {
        self.0.is_open()
    }

    pub const fn allows_deselect(&self) -> bool {
        self.0.allows_deselect()
    }

    pub const fn is_disabled(&self) -> bool {
        self.0.is_disabled()
    }

    pub const fn is_read_only(&self) -> bool {
        self.0.is_read_only()
    }

    pub const fn is_required(&self) -> bool {
        self.0.is_required()
    }

    pub const fn highlight_item_on_hover(&self) -> bool {
        self.0.highlight_item_on_hover()
    }

    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    pub fn form(&self) -> Option<&str> {
        self.0.form()
    }

    pub fn autocomplete(&self) -> Option<&str> {
        self.0.autocomplete()
    }

    pub fn is_value_selected(&self, val: &str) -> bool {
        self.0.is_value_selected(val)
    }

    pub fn has_selected_value(&self) -> bool {
        self.0.has_selected_value()
    }

    pub fn apply_toggle(&self, val: &str) -> SelectToggleOutcome {
        self.0.apply_toggle(val)
    }

    pub fn root_attributes(&self) -> SelectRootAttributes {
        self.0.root_attributes()
    }

    pub fn trigger_attributes(&self) -> SelectTriggerAttributes {
        self.0.trigger_attributes()
    }

    pub fn content_attributes(&self, activedescendant: Option<String>) -> SelectContentAttributes {
        self.0.content_attributes(activedescendant)
    }

    pub fn content_attributes_with_side(
        &self,
        activedescendant: Option<String>,
        side: PlacementSide,
    ) -> SelectContentAttributes {
        self.0.content_attributes_with_side(activedescendant, side)
    }

    pub fn content_attributes_with_side_and_align(
        &self,
        activedescendant: Option<String>,
        side: PlacementSide,
        align: PlacementAlign,
    ) -> SelectContentAttributes {
        self.0
            .content_attributes_with_side_and_align(activedescendant, side, align)
    }

    pub fn item_attributes(
        &self,
        item_val: &str,
        is_highlighted: bool,
        disabled: bool,
    ) -> SelectItemAttributes {
        self.0.item_attributes(item_val, is_highlighted, disabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_shim_preserves_legacy_mode_deselect_and_delegates_state() {
        let scope = ScopeHandle::root("select").child("shim");

        let single_deselect = Select::new(scope.clone())
            .with_mode(SelectMode::Single {
                allow_deselect: true,
            })
            .with_value(Some("alpha".to_owned()))
            .with_open(true)
            .with_required(true);
        assert_eq!(
            single_deselect.mode(),
            SelectMode::Single {
                allow_deselect: true
            }
        );
        assert!(single_deselect.allows_deselect());
        assert!(!single_deselect.is_multiple());
        assert!(single_deselect.is_open());
        assert!(single_deselect.is_required());
        assert_eq!(single_deselect.value(), Some("alpha"));
        assert_eq!(
            single_deselect.trigger_attributes().aria_expanded(),
            "true"
        );
        assert_eq!(
            single_deselect
                .item_attributes("alpha", true, false)
                .aria_selected(),
            "true"
        );

        let cleared = single_deselect.apply_toggle("alpha");
        assert!(cleared.changed);
        assert_eq!(cleared.next_value, None);
        assert!(cleared.should_close);

        // Builder order: `with_allow_deselect` must not clobber `Multiple`.
        let multi = Select::new(scope)
            .with_mode(SelectMode::Multiple)
            .with_allow_deselect(true)
            .with_values(vec!["a".to_owned(), "b".to_owned()])
            .with_portal_host(PortalHost::named("custom-host"))
            .with_retained_mount(false);
        assert_eq!(multi.mode(), SelectMode::Multiple);
        assert!(multi.is_multiple());
        assert_eq!(multi.values(), &["a", "b"]);
        assert_eq!(multi.portal_host().id(), Some("custom-host"));
        assert!(!multi.presence().retain_mount());

        // `with_multiple(false)` returns to legacy Single { allow_deselect: false }.
        let back_to_single = multi.with_multiple(false);
        assert_eq!(
            back_to_single.mode(),
            SelectMode::Single {
                allow_deselect: false
            }
        );
        assert!(!back_to_single.allows_deselect());
    }
}

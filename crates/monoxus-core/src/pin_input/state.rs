// [KEEP-MONOXUS-BRAND]
use crate::foundation::shared::ScopeHandle;

pub use ui_sdk::pin_input::state::{
    PinEditSegment, PinShiftNavKey, PinSpliceOutcome, apply_pin_splice, char_index_to_utf16,
    collapse_pin_selection_to_single_cell, compute_pin_cells, extend_pin_selection_with_shift,
    extract_pin_edit, filter_pin_chars_with, format_pin_cell_display, initial_focus_selection,
    normalize_pin_selection, project_pin_cell_char, replace_pin_value_from_paste_with,
    resolve_pin_cell_hit, resolve_pin_drag_selection, resolve_pin_pointer_selection,
    sanitize_pin_value_with, should_fire_on_complete, should_prevent_keydown_with,
    step_pin_single_cell, target_selection_for_cell_click, truncate_pin_chars,
    utf16_to_char_index,
};

use super::{
    attrs::{PinInputCellAttributes, PinInputInputAttributes, PinInputRootAttributes},
    relationships::PinInputRelationships,
    types::{
        PinInputAutoComplete, PinInputBuiltinPattern, PinInputMode, PinInputPart,
        PinInputSelection, PinInputSnapshot, PinInputTextAlign,
    },
};

/// Pure headless configuration and snapshot/attribute builder for `PinInput`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinInputDefinition(ui_sdk::pin_input::state::PinInputDefinition);

impl PinInputDefinition {
    pub fn new(scope: ScopeHandle) -> Self {
        Self(ui_sdk::pin_input::state::PinInputDefinition::new(scope))
    }

    pub const fn parts() -> &'static [PinInputPart] {
        ui_sdk::pin_input::state::PinInputDefinition::parts()
    }

    pub fn relationships(&self) -> &PinInputRelationships {
        self.0.relationships()
    }

    pub const fn max_length(&self) -> usize {
        self.0.max_length()
    }

    pub const fn builtin_pattern(&self) -> PinInputBuiltinPattern {
        self.0.builtin_pattern()
    }

    pub const fn text_align(&self) -> PinInputTextAlign {
        self.0.text_align()
    }

    pub const fn input_mode(&self) -> PinInputMode {
        self.0.input_mode()
    }

    pub const fn autocomplete(&self) -> PinInputAutoComplete {
        self.0.autocomplete()
    }

    pub const fn is_masked(&self) -> bool {
        self.0.is_masked()
    }

    pub const fn mask_char(&self) -> char {
        self.0.mask_char()
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

    pub const fn is_invalid(&self) -> bool {
        self.0.is_invalid()
    }

    pub const fn is_auto_focus(&self) -> bool {
        self.0.is_auto_focus()
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.0.placeholder()
    }

    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    pub fn form(&self) -> Option<&str> {
        self.0.form()
    }

    pub fn with_max_length(self, max_length: usize) -> Self {
        Self(self.0.with_max_length(max_length))
    }

    pub fn with_builtin_pattern(self, pattern: PinInputBuiltinPattern) -> Self {
        Self(self.0.with_builtin_pattern(pattern))
    }

    pub fn with_text_align(self, text_align: PinInputTextAlign) -> Self {
        Self(self.0.with_text_align(text_align))
    }

    pub fn with_input_mode(self, input_mode: PinInputMode) -> Self {
        Self(self.0.with_input_mode(input_mode))
    }

    pub fn with_autocomplete(self, autocomplete: PinInputAutoComplete) -> Self {
        Self(self.0.with_autocomplete(autocomplete))
    }

    pub fn with_mask(self, mask: bool) -> Self {
        Self(self.0.with_mask(mask))
    }

    pub fn with_mask_char(self, mask_char: char) -> Self {
        Self(self.0.with_mask_char(mask_char))
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

    pub fn with_invalid(self, invalid: bool) -> Self {
        Self(self.0.with_invalid(invalid))
    }

    pub fn with_auto_focus(self, auto_focus: bool) -> Self {
        Self(self.0.with_auto_focus(auto_focus))
    }

    pub fn with_placeholder(self, placeholder: Option<String>) -> Self {
        Self(self.0.with_placeholder(placeholder))
    }

    pub fn with_name(self, name: Option<String>) -> Self {
        Self(self.0.with_name(name))
    }

    pub fn with_form(self, form: Option<String>) -> Self {
        Self(self.0.with_form(form))
    }

    pub fn snapshot(
        &self,
        value: &str,
        selection: PinInputSelection,
        is_focused: bool,
        is_hovering: bool,
    ) -> PinInputSnapshot {
        self.0.snapshot(value, selection, is_focused, is_hovering)
    }

    pub fn root_attributes(&self, custom_id: Option<String>) -> PinInputRootAttributes {
        self.0.root_attributes(custom_id).into()
    }

    pub fn input_attributes(
        &self,
        custom_input_id: Option<String>,
        value: &str,
        selection: PinInputSelection,
        html_pattern: Option<&'static str>,
    ) -> PinInputInputAttributes {
        self.0
            .input_attributes(custom_input_id, value, selection, html_pattern)
            .into()
    }

    pub fn cell_attributes(
        &self,
        index: usize,
        custom_id: Option<String>,
        is_active: bool,
    ) -> PinInputCellAttributes {
        self.0.cell_attributes(index, custom_id, is_active).into()
    }
}

pub type PinInput = PinInputDefinition;

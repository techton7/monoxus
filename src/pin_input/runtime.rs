use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use dioxus::prelude::keyboard_types::Key;
use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        get_element_client_rect, get_input_selection_range, set_input_selection_range,
        set_input_value_by_id,
    },
    keyboard::{NavKey, translate_nav_key},
    shared::ScopeHandle,
    state::{ControllableSignal, sync_signal, use_controllable_signal_with},
};

use super::{
    attrs::{PinInputCellAttributes, PinInputInputAttributes, PinInputRootAttributes},
    relationships::PinInputRelationships,
    state::{
        PinInputDefinition, PinShiftNavKey, char_index_to_utf16,
        collapse_pin_selection_to_single_cell, extract_pin_edit, initial_focus_selection,
        normalize_pin_selection, project_pin_cell_char,
        replace_pin_value_from_paste_with_callbacks, resolve_pin_cell_hit, sanitize_pin_value,
        should_fire_on_complete, should_prevent_keydown_with, step_pin_single_cell,
        target_selection_for_cell_click, truncate_pin_chars, utf16_to_char_index,
    },
    types::{
        PinInputAutoComplete, PinInputBuiltinPattern, PinInputCellData, PinInputMode,
        PinInputPattern, PinInputSelection, PinInputSnapshot, PinInputTextAlign,
    },
};

/// Reactive runtime managing `PinInput` value, DOM selection mirroring, pattern filtering,
/// paste splicing, and `on_complete` edge detection.
#[derive(Clone)]
pub struct PinInputRuntime {
    pub value: Signal<String>,
    pub(crate) value_ctl: ControllableSignal<String>,
    pub is_controlled: Signal<bool>,
    pub max_length: Signal<usize>,
    pub pattern: Rc<RefCell<PinInputPattern>>,
    pub paste_transformer: CopyValue<Option<Callback<String, String>>>,
    pub on_value_change: CopyValue<Option<EventHandler<String>>>,
    pub on_complete: CopyValue<Option<EventHandler<String>>>,
    pub last_complete_value: Rc<RefCell<String>>,
    pub selection: Signal<PinInputSelection>,
    pub prev_selection: Rc<RefCell<PinInputSelection>>,
    pub is_focused: Signal<bool>,
    pub is_hovering: Signal<bool>,
    pub text_align: Signal<PinInputTextAlign>,
    pub input_mode: Signal<PinInputMode>,
    pub autocomplete: Signal<PinInputAutoComplete>,
    pub mask: Signal<bool>,
    pub mask_char: Signal<char>,
    pub disabled: Signal<bool>,
    pub read_only: Signal<bool>,
    pub required: Signal<bool>,
    pub auto_focus: Signal<bool>,
    pub placeholder: Signal<Option<String>>,
    pub name: Signal<Option<String>>,
    pub form: Signal<Option<String>>,
    pub scope_id: Signal<String>,
    pub root_dom_id: Signal<String>,
    pub input_dom_id: Signal<String>,
    pub cell_ids: Rc<RefCell<BTreeMap<usize, String>>>,
    pub pointer_target_selection: Rc<RefCell<Option<PinInputSelection>>>,
    pub pointer_down_active: Rc<RefCell<bool>>,
}

impl PinInputRuntime {
    /// Returns the deterministic [`PinInputRelationships`] for the current scope.
    pub fn relationships(&self) -> PinInputRelationships {
        let scope = ScopeHandle::root("pin-input").child((self.scope_id)());
        PinInputRelationships::new(scope)
    }

    /// Builds an immutable [`PinInputDefinition`] reflecting current reactive configuration.
    pub fn definition(&self) -> PinInputDefinition {
        let scope = ScopeHandle::root("pin-input").child((self.scope_id)());
        let builtin = self
            .pattern
            .borrow()
            .as_builtin()
            .unwrap_or(PinInputBuiltinPattern::Any);
        PinInputDefinition::new(scope)
            .with_max_length((self.max_length)())
            .with_builtin_pattern(builtin)
            .with_text_align((self.text_align)())
            .with_input_mode((self.input_mode)())
            .with_autocomplete((self.autocomplete)())
            .with_mask((self.mask)())
            .with_mask_char((self.mask_char)())
            .with_disabled((self.disabled)())
            .with_read_only((self.read_only)())
            .with_required((self.required)())
            .with_auto_focus((self.auto_focus)())
            .with_placeholder((self.placeholder)())
            .with_name((self.name)())
            .with_form((self.form)())
    }

    /// Returns the current value string.
    pub fn current_value(&self) -> String {
        (self.value)()
    }

    /// Returns a full reactive [`PinInputSnapshot`] for consumers and `render_children`.
    pub fn snapshot(&self) -> PinInputSnapshot {
        self.definition().snapshot(
            &(self.value)(),
            (self.selection)(),
            (self.is_focused)(),
            (self.is_hovering)(),
        )
    }

    /// Returns projected cell state for `index`.
    pub fn cell_data(&self, index: usize) -> PinInputCellData {
        self.snapshot()
            .cells
            .into_iter()
            .nth(index)
            .unwrap_or(PinInputCellData {
                index,
                char: None,
                is_active: false,
                has_fake_caret: false,
            })
    }

    /// Projects the rendered display character for `cell`, applying `mask` and `mask_char`.
    pub fn display_char_for_cell(&self, cell: &PinInputCellData) -> Option<char> {
        project_pin_cell_char(cell.char, (self.mask)(), (self.mask_char)())
    }

    /// Derives [`PinInputRootAttributes`] for `PinInputRoot`.
    pub fn root_attributes(&self, custom_id: Option<String>) -> PinInputRootAttributes {
        let resolved_id = custom_id.unwrap_or_else(|| (self.root_dom_id)());
        self.definition().root_attributes(Some(resolved_id))
    }

    /// Derives [`PinInputInputAttributes`] for the internal `<input>`.
    pub fn input_attributes(&self) -> PinInputInputAttributes {
        let html_pattern = self.pattern.borrow().html_pattern();
        self.definition().input_attributes(
            Some((self.input_dom_id)()),
            &(self.value)(),
            (self.selection)(),
            html_pattern,
        )
    }

    /// Derives [`PinInputCellAttributes`] for `PinInputCell` at `index`.
    pub fn cell_attributes(
        &self,
        index: usize,
        custom_id: Option<String>,
    ) -> PinInputCellAttributes {
        let cell = self.cell_data(index);
        self.definition()
            .cell_attributes(index, custom_id, cell.is_active)
    }

    fn read_dom_selection_chars(&self, reference_text: &str) -> Option<(usize, usize)> {
        let input_id = self.input_dom_id.peek().clone();
        let (u16_s, u16_e) = get_input_selection_range(&input_id)?;
        let cs = utf16_to_char_index(reference_text, u16_s);
        let ce = utf16_to_char_index(reference_text, u16_e);
        Some((cs.min(ce), ce.max(cs)))
    }

    fn write_dom_selection_chars(&self, reference_text: &str, start: usize, end: usize) {
        let input_id = self.input_dom_id.peek().clone();
        let u16_s = char_index_to_utf16(reference_text, start);
        let u16_e = char_index_to_utf16(reference_text, end);
        let _ = set_input_selection_range(&input_id, u16_s, u16_e);
    }

    fn dom_selection_target(norm: PinInputSelection) -> Option<(usize, usize)> {
        Some((norm.start?, norm.end?))
    }

    fn write_dom_selection_for_norm(&self, reference_text: &str, norm: PinInputSelection) {
        if let Some((ds, de)) = Self::dom_selection_target(norm) {
            self.write_dom_selection_chars(reference_text, ds, de);
        }
    }

    fn apply_selection_state(&mut self, next: PinInputSelection) {
        *self.prev_selection.borrow_mut() = next;
        sync_signal(&mut self.selection, next);
    }

    /// Synchronizes selection state from the host DOM `<input>` element.
    pub fn sync_selection_from_dom(&mut self) {
        if !*self.is_focused.peek() || *self.disabled.peek() {
            self.apply_selection_state(PinInputSelection::empty());
            return;
        }
        let val = self.value.peek().clone();
        let max_len = *self.max_length.peek();
        let prev = *self.prev_selection.borrow();

        if let Some((cs, ce)) = self.read_dom_selection_chars(&val) {
            let norm = collapse_pin_selection_to_single_cell(
                &val,
                normalize_pin_selection(&val, Some(cs), Some(ce), prev, max_len),
                max_len,
            );
            if let Some((target_s, target_e)) = Self::dom_selection_target(norm)
                && (target_s, target_e) != (cs, ce)
            {
                self.write_dom_selection_chars(&val, target_s, target_e);
            }
            self.apply_selection_state(norm);
        } else if self.selection.peek().start.is_none() {
            let fallback = collapse_pin_selection_to_single_cell(
                &val,
                initial_focus_selection(&val, max_len),
                max_len,
            );
            self.apply_selection_state(fallback);
        }
    }

    /// Registers a `PinInputCell` DOM element ID for pointer-to-cell hit resolution.
    pub fn register_cell(&self, index: usize, id: String) {
        let mut cells = self.cell_ids.borrow_mut();
        if cells.get(&index) != Some(&id) {
            cells.insert(index, id);
        }
    }

    /// Unregisters a `PinInputCell` DOM element ID when the cell unmounts.
    pub fn unregister_cell(&self, index: usize, id: &str) {
        let mut cells = self.cell_ids.borrow_mut();
        if cells.get(&index).map(String::as_str) == Some(id) {
            cells.remove(&index);
        }
    }

    fn cell_rects(&self) -> Vec<(usize, (f64, f64, f64, f64))> {
        let max_len = *self.max_length.peek();
        let cell_entries: Vec<(usize, String)> = {
            let cells = self.cell_ids.borrow();
            cells
                .iter()
                .filter(|(idx, _)| **idx < max_len)
                .map(|(idx, id)| (*idx, id.clone()))
                .collect()
        };
        cell_entries
            .into_iter()
            .filter_map(|(idx, id)| {
                let rect = get_element_client_rect(&id)?;
                Some((idx, (rect.left(), rect.top(), rect.right(), rect.bottom())))
            })
            .collect()
    }

    /// Resolves pointer coordinates to a target cell selection on `pointerdown`.
    pub fn handle_pointer_down(&mut self, client_x: f64, client_y: f64) -> bool {
        if *self.disabled.peek() {
            *self.pointer_target_selection.borrow_mut() = None;
            *self.pointer_down_active.borrow_mut() = false;
            return false;
        }
        let max_len = *self.max_length.peek();
        let cell_rects = self.cell_rects();
        let val = self.value.peek().clone();
        if let Some(idx) = resolve_pin_cell_hit(&cell_rects, client_x, client_y) {
            *self.pointer_down_active.borrow_mut() = true;
            let target = target_selection_for_cell_click(idx, val.chars().count(), max_len);
            *self.pointer_target_selection.borrow_mut() = Some(target);
            self.write_dom_selection_for_norm(&val, target);
            self.apply_selection_state(target);
            true
        } else {
            *self.pointer_down_active.borrow_mut() = false;
            *self.pointer_target_selection.borrow_mut() = None;
            false
        }
    }

    /// Suppresses host drag-selection while a pointer press started inside a cell.
    pub fn handle_pointer_move(
        &mut self,
        _client_x: f64,
        _client_y: f64,
        buttons_held: bool,
    ) -> bool {
        if *self.disabled.peek() || !buttons_held {
            *self.pointer_down_active.borrow_mut() = false;
            return false;
        }
        *self.pointer_down_active.borrow()
    }

    /// Finalizes pointer-driven cell selection on `pointerup`.
    pub fn handle_pointer_up(&mut self, _client_x: f64, _client_y: f64) {
        *self.pointer_down_active.borrow_mut() = false;
        if *self.disabled.peek() {
            *self.pointer_target_selection.borrow_mut() = None;
            self.sync_selection_from_dom();
            return;
        }
        if !*self.is_focused.peek() {
            return;
        }
        let Some(target) = self.pointer_target_selection.borrow_mut().take() else {
            self.sync_selection_from_dom();
            return;
        };
        let val = self.value.peek().clone();
        let max_len = *self.max_length.peek();
        let single = collapse_pin_selection_to_single_cell(&val, target, max_len);
        self.write_dom_selection_for_norm(&val, single);
        self.apply_selection_state(single);
    }

    /// Handles `<input>` focus gain.
    pub fn handle_focus(&mut self) {
        if *self.disabled.peek() {
            *self.pointer_target_selection.borrow_mut() = None;
            return;
        }
        self.is_focused.set(true);
        let val = self.value.peek().clone();
        let max_len = *self.max_length.peek();
        let init_sel = collapse_pin_selection_to_single_cell(
            &val,
            self.pointer_target_selection
                .borrow_mut()
                .take()
                .unwrap_or_else(|| initial_focus_selection(&val, max_len)),
            max_len,
        );
        self.write_dom_selection_for_norm(&val, init_sel);
        self.apply_selection_state(init_sel);
    }

    /// Handles `<input>` blur.
    pub fn handle_blur(&mut self) {
        *self.pointer_target_selection.borrow_mut() = None;
        *self.pointer_down_active.borrow_mut() = false;
        self.is_focused.set(false);
        self.apply_selection_state(PinInputSelection::empty());
    }

    /// Handles `<input>` `keydown`; returns `true` when `prevent_default()` should be called.
    pub fn handle_keydown(
        &mut self,
        key: &Key,
        ctrl_or_meta: bool,
        shift: bool,
        is_composing: bool,
    ) -> bool {
        *self.pointer_target_selection.borrow_mut() = None;
        if *self.disabled.peek() || *self.read_only.peek() {
            return false;
        }
        let nav = translate_nav_key(key);
        if ctrl_or_meta && !is_composing {
            if matches!(key, Key::Character(s) if s.eq_ignore_ascii_case("a")) {
                return true;
            }
            if matches!(nav, Some(NavKey::Backspace | NavKey::Delete)) {
                let input_id = self.input_dom_id.peek().clone();
                let _ = set_input_value_by_id(&input_id, "");
                let collapsed = PinInputSelection::collapsed(0);
                self.write_dom_selection_chars("", 0, 0);
                self.apply_selection_state(collapsed);
                self.commit_value(String::new());
                return true;
            }
        }
        if shift && !ctrl_or_meta && !is_composing {
            let nav_key = match nav {
                Some(NavKey::ArrowLeft) => Some(PinShiftNavKey::ArrowLeft),
                Some(NavKey::ArrowRight) => Some(PinShiftNavKey::ArrowRight),
                Some(NavKey::Home) => Some(PinShiftNavKey::Home),
                Some(NavKey::End) => Some(PinShiftNavKey::End),
                _ => None,
            };
            if let Some(nav_key) = nav_key {
                let val = self.value.peek().clone();
                let max_len = *self.max_length.peek();
                let current = *self.selection.peek();
                let next = step_pin_single_cell(&val, max_len, current, nav_key);
                self.write_dom_selection_for_norm(&val, next);
                self.apply_selection_state(next);
                return true;
            }
        }
        if let Key::Character(s) = key {
            let pat = self.pattern.borrow().clone();
            if should_prevent_keydown_with(s, ctrl_or_meta, is_composing, |k| pat.matches_str(k)) {
                return true;
            }
        }
        if matches!(
            nav,
            Some(
                NavKey::ArrowLeft
                    | NavKey::ArrowRight
                    | NavKey::ArrowUp
                    | NavKey::ArrowDown
                    | NavKey::Home
                    | NavKey::End
            )
        ) && dioxus::core::Runtime::try_current().is_some()
        {
            let mut runtime = self.clone();
            spawn(async move {
                let _ = oxidase::frame::next_frame().await;
                runtime.sync_selection_from_dom();
            });
        }
        false
    }

    fn extract_inserted_segment(
        prev_val: &str,
        raw: &str,
        prev_sel: PinInputSelection,
    ) -> String {
        if let (Some(s), Some(e)) = (prev_sel.start, prev_sel.end) {
            let prev_chars: Vec<char> = prev_val.chars().collect();
            let raw_chars: Vec<char> = raw.chars().collect();
            let s = s.min(prev_chars.len());
            let e = e.clamp(s, prev_chars.len());
            let suffix_len = prev_chars.len() - e;
            if raw_chars.len() >= s + suffix_len
                && raw_chars[..s] == prev_chars[..s]
                && raw_chars[raw_chars.len() - suffix_len..] == prev_chars[e..]
            {
                return raw_chars[s..raw_chars.len() - suffix_len].iter().collect();
            }
        }
        extract_pin_edit(prev_val, raw).inserted
    }

    fn apply_paste(&mut self, raw_paste: &str) -> bool {
        let input_id = self.input_dom_id.peek().clone();
        *self.pointer_target_selection.borrow_mut() = None;
        *self.pointer_down_active.borrow_mut() = false;
        let max_len = *self.max_length.peek();
        let pat = self.pattern.borrow().clone();
        let transformer = self.paste_transformer.try_read().ok().and_then(|s| *s);

        let Some((next_val, next_caret)) = replace_pin_value_from_paste_with_callbacks(
            raw_paste,
            max_len,
            transformer.as_ref(),
            &pat,
        ) else {
            return false;
        };

        let norm = collapse_pin_selection_to_single_cell(
            &next_val,
            normalize_pin_selection(
                &next_val,
                Some(next_caret),
                Some(next_caret),
                PinInputSelection::empty(),
                max_len,
            ),
            max_len,
        );
        let _ = set_input_value_by_id(&input_id, &next_val);
        self.write_dom_selection_for_norm(&next_val, norm);
        self.apply_selection_state(norm);
        self.commit_value(next_val);
        true
    }

    /// Handles `<input>` `oninput`, covering single-char edits, deletions, and native paste/autofill.
    pub fn handle_input(&mut self, raw: String) {
        let prev_val = self.value.peek().clone();
        let prev_sel = *self.prev_selection.borrow();
        let input_id = self.input_dom_id.peek().clone();
        *self.pointer_target_selection.borrow_mut() = None;
        *self.pointer_down_active.borrow_mut() = false;
        let max_len = *self.max_length.peek();

        if *self.disabled.peek() || *self.read_only.peek() {
            let _ = set_input_value_by_id(&input_id, &prev_val);
            return;
        }

        let inserted = Self::extract_inserted_segment(&prev_val, &raw, prev_sel);
        if inserted.chars().count() > 1 {
            if !self.apply_paste(&inserted) {
                let _ = set_input_value_by_id(&input_id, &prev_val);
                let restored = collapse_pin_selection_to_single_cell(&prev_val, prev_sel, max_len);
                self.write_dom_selection_for_norm(&prev_val, restored);
                self.apply_selection_state(restored);
            }
            return;
        }

        let pat = self.pattern.borrow().clone();
        let dom_sel = self.read_dom_selection_chars(&raw);
        let next_val = sanitize_pin_value(&raw, max_len, &pat);
        let target_sel = if next_val != raw {
            let _ = set_input_value_by_id(&input_id, &next_val);
            let next_chars = next_val.chars().count();
            let norm = if next_val == prev_val {
                let raw_s = prev_sel.start.unwrap_or(next_chars).min(next_chars);
                let raw_e = prev_sel.end.unwrap_or(next_chars).min(next_chars);
                normalize_pin_selection(&next_val, Some(raw_s), Some(raw_e), prev_sel, max_len)
            } else {
                let (ds, de) = dom_sel.unwrap_or((next_chars, next_chars));
                normalize_pin_selection(
                    &next_val,
                    Some(ds.min(next_chars)),
                    Some(de.min(next_chars)),
                    PinInputSelection::empty(),
                    max_len,
                )
            };
            let single = collapse_pin_selection_to_single_cell(&next_val, norm, max_len);
            self.write_dom_selection_for_norm(&next_val, single);
            single
        } else {
            let (ds, de) = dom_sel.unwrap_or_else(|| {
                let l = next_val.chars().count();
                (l, l)
            });
            let norm = collapse_pin_selection_to_single_cell(
                &next_val,
                normalize_pin_selection(
                    &next_val,
                    Some(ds),
                    Some(de),
                    PinInputSelection::empty(),
                    max_len,
                ),
                max_len,
            );
            if let Some((target_s, target_e)) = Self::dom_selection_target(norm)
                && (target_s, target_e) != (ds, de)
            {
                self.write_dom_selection_chars(&next_val, target_s, target_e);
            }
            norm
        };

        self.apply_selection_state(target_sel);
        self.commit_value(next_val);
    }

    /// Handles Web `<input>` `onpaste` with `paste_transformer` and flush-from-0 replacement.
    pub fn handle_paste(&mut self, pasted: &str) {
        if *self.disabled.peek() || *self.read_only.peek() {
            return;
        }
        let _ = self.apply_paste(pasted);
    }

    /// Commits a sanitized value, notifying `on_value_change` and `on_complete` (uncontrolled).
    pub fn commit_value(&mut self, next: String) {
        let max_len = *self.max_length.peek();
        let controlled = self.value_ctl.is_controlled();
        if !self.value_ctl.set(next.clone()) {
            return;
        }
        if !controlled {
            self.check_and_fire_on_complete(&next, max_len);
        } else if dioxus::core::Runtime::try_current().is_some() {
            let runtime = self.clone();
            spawn(async move {
                let _ = oxidase::frame::next_frame().await;
                let expected = runtime.value.peek().clone();
                let input_id = runtime.input_dom_id.peek().clone();
                let _ = set_input_value_by_id(&input_id, &expected);
                if *runtime.is_focused.peek() {
                    let sel = *runtime.selection.peek();
                    if let (Some(s), Some(e)) = (sel.start, sel.end) {
                        let len = expected.chars().count();
                        runtime.write_dom_selection_chars(&expected, s.min(len), e.min(len));
                    }
                }
            });
        }
    }

    /// Evaluates edge-transition completion against `last_complete_value` and invokes `on_complete`.
    pub fn check_and_fire_on_complete(&self, current: &str, max_len: usize) {
        let should_fire = {
            let mut last = self.last_complete_value.borrow_mut();
            let fire = should_fire_on_complete(&last, current, max_len);
            *last = current.to_string();
            fire
        };
        if should_fire
            && let Some(handler) = self.on_complete.try_read().ok().and_then(|s| *s)
        {
            handler.call(current.to_string());
        }
    }

    /// Restores value and DOM `<input>` state in response to a native `<form>` reset event.
    pub fn reset_to(&mut self, initial: &str) {
        *self.pointer_target_selection.borrow_mut() = None;
        *self.pointer_down_active.borrow_mut() = false;
        let max_len = *self.max_length.peek();
        let pat = self.pattern.borrow().clone();
        let sanitized = truncate_pin_chars(&sanitize_pin_value(initial, max_len, &pat), max_len);
        let input_id = self.input_dom_id.peek().clone();
        let _ = set_input_value_by_id(&input_id, &sanitized);

        if *self.is_focused.peek() {
            let sel = initial_focus_selection(&sanitized, max_len);
            self.write_dom_selection_for_norm(&sanitized, sel);
            self.apply_selection_state(sel);
        } else {
            self.apply_selection_state(PinInputSelection::empty());
        }

        *self.last_complete_value.borrow_mut() = sanitized.clone();
        self.value_ctl.set(sanitized);
    }
}

/// Initializes reactive signals and non-reactive callback slots for `PinInputRoot`.
#[allow(clippy::too_many_arguments)]
pub fn use_pin_input_runtime(
    value: Option<String>,
    default_value: String,
    max_length: usize,
    pattern: PinInputPattern,
    paste_transformer: Option<Callback<String, String>>,
    on_value_change: Option<EventHandler<String>>,
    on_complete: Option<EventHandler<String>>,
    text_align: PinInputTextAlign,
    input_mode: PinInputMode,
    autocomplete: PinInputAutoComplete,
    mask: bool,
    mask_char: char,
    disabled: bool,
    read_only: bool,
    required: bool,
    auto_focus: bool,
    placeholder: Option<String>,
    name: Option<String>,
    form: Option<String>,
    scope_id: String,
    root_dom_id: String,
    input_dom_id: String,
) -> PinInputRuntime {
    let effective_max_length = max_length.max(1);
    let pat_for_sanitize = pattern.clone();
    let value_ctl = use_controllable_signal_with(value, default_value, on_value_change, move |v| {
        sanitize_pin_value(&v, effective_max_length, &pat_for_sanitize)
    });

    if !value_ctl.is_controlled() {
        let mut sig = value_ctl.signal();
        let current = sig.peek().clone();
        let clamped = sanitize_pin_value(&current, effective_max_length, &pattern);
        sync_signal(&mut sig, clamped);
    }

    let pattern_slot = use_hook(|| Rc::new(RefCell::new(pattern.clone())));
    *pattern_slot.borrow_mut() = pattern;
    let mut paste_slot = use_hook(|| CopyValue::new(paste_transformer));
    paste_slot.set(paste_transformer);
    let mut change_slot = use_hook(|| CopyValue::new(on_value_change));
    change_slot.set(on_value_change);
    let mut complete_slot = use_hook(|| CopyValue::new(on_complete));
    complete_slot.set(on_complete);
    let last_complete_slot = use_hook(|| Rc::new(RefCell::new(value_ctl.peek())));
    let prev_sel_slot = use_hook(|| Rc::new(RefCell::new(PinInputSelection::empty())));
    let cell_ids_slot = use_hook(|| Rc::new(RefCell::new(BTreeMap::new())));
    let pointer_target_slot = use_hook(|| Rc::new(RefCell::new(None)));
    let pointer_down_slot = use_hook(|| Rc::new(RefCell::new(false)));

    let is_controlled = value_ctl.is_controlled();
    let mut rt = PinInputRuntime {
        value: value_ctl.signal(),
        value_ctl,
        is_controlled: use_signal(|| is_controlled),
        max_length: use_signal(|| effective_max_length),
        pattern: pattern_slot,
        paste_transformer: paste_slot,
        on_value_change: change_slot,
        on_complete: complete_slot,
        last_complete_value: last_complete_slot,
        selection: use_signal(PinInputSelection::empty),
        prev_selection: prev_sel_slot,
        is_focused: use_signal(|| false),
        is_hovering: use_signal(|| false),
        text_align: use_signal(|| text_align),
        input_mode: use_signal(|| input_mode),
        autocomplete: use_signal(|| autocomplete),
        mask: use_signal(|| mask),
        mask_char: use_signal(|| mask_char),
        disabled: use_signal(|| disabled),
        read_only: use_signal(|| read_only),
        required: use_signal(|| required),
        auto_focus: use_signal(|| auto_focus),
        placeholder: use_signal(|| placeholder.clone()),
        name: use_signal(|| name.clone()),
        form: use_signal(|| form.clone()),
        scope_id: use_signal(|| scope_id.clone()),
        root_dom_id: use_signal(|| root_dom_id.clone()),
        input_dom_id: use_signal(|| input_dom_id.clone()),
        cell_ids: cell_ids_slot,
        pointer_target_selection: pointer_target_slot,
        pointer_down_active: pointer_down_slot,
    };

    sync_signal(&mut rt.is_controlled, is_controlled);
    sync_signal(&mut rt.max_length, effective_max_length);
    sync_signal(&mut rt.text_align, text_align);
    sync_signal(&mut rt.input_mode, input_mode);
    sync_signal(&mut rt.autocomplete, autocomplete);
    sync_signal(&mut rt.mask, mask);
    sync_signal(&mut rt.mask_char, mask_char);
    sync_signal(&mut rt.disabled, disabled);
    sync_signal(&mut rt.read_only, read_only);
    sync_signal(&mut rt.required, required);
    sync_signal(&mut rt.auto_focus, auto_focus);
    sync_signal(&mut rt.placeholder, placeholder);
    sync_signal(&mut rt.name, name);
    sync_signal(&mut rt.form, form);
    sync_signal(&mut rt.scope_id, scope_id);
    sync_signal(&mut rt.root_dom_id, root_dom_id);
    sync_signal(&mut rt.input_dom_id, input_dom_id);

    if disabled && *rt.is_focused.peek() {
        rt.is_focused.set(false);
        *rt.prev_selection.borrow_mut() = PinInputSelection::empty();
        rt.selection.set(PinInputSelection::empty());
    }

    rt
}

/// Context providing [`PinInputRuntime`] to [`crate::pin_input::PinInputCell`] and consumers.
#[derive(Clone)]
pub struct PinInputContext {
    pub runtime: PinInputRuntime,
}

impl PinInputContext {
    pub fn snapshot(&self) -> PinInputSnapshot {
        self.runtime.snapshot()
    }

    pub fn cell(&self, index: usize) -> PinInputCellData {
        self.runtime.cell_data(index)
    }

    pub fn value(&self) -> String {
        self.runtime.current_value()
    }
}

/// Accesses [`PinInputContext`] inside a [`crate::pin_input::PinInputRoot`] subtree.
pub fn use_pin_input_context() -> PinInputContext {
    use_context::<PinInputContext>()
}

/// Optional variant of [`use_pin_input_context`].
pub fn try_use_pin_input_context() -> Option<PinInputContext> {
    try_consume_context::<PinInputContext>()
}

/// Context exposing the active [`PinInputCellData`] and resolved display character for a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinInputCellContext {
    pub cell: PinInputCellData,
    pub display_char: Option<char>,
    pub disabled: bool,
}

/// Accesses [`PinInputCellContext`] inside a [`crate::pin_input::PinInputCell`] subtree.
pub fn use_pin_input_cell_context() -> PinInputCellContext {
    use_context::<PinInputCellContext>()
}

/// Optional variant of [`use_pin_input_cell_context`].
pub fn try_use_pin_input_cell_context() -> Option<PinInputCellContext> {
    try_consume_context::<PinInputCellContext>()
}

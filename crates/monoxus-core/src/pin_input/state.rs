use crate::foundation::shared::ScopeHandle;

use super::{
    attrs::{PinInputCellAttributes, PinInputInputAttributes, PinInputRootAttributes},
    relationships::PinInputRelationships,
    types::{
        PIN_INPUT_PARTS, PinInputAutoComplete, PinInputBuiltinPattern, PinInputCellData,
        PinInputMode, PinInputPart, PinInputSelection, PinInputSelectionDirection,
        PinInputSnapshot, PinInputTextAlign,
    },
};

/// Truncates `raw` to at most `max_length.max(1)` Unicode scalar values.
pub fn truncate_pin_chars(raw: &str, max_length: usize) -> String {
    let limit = max_length.max(1);
    raw.chars().take(limit).collect()
}

/// Converts a character index in `s` to a UTF-16 code-unit offset (clamped to end of `s`).
pub fn char_index_to_utf16(s: &str, char_idx: usize) -> usize {
    s.chars().take(char_idx).map(char::len_utf16).sum()
}

/// Converts a UTF-16 code-unit offset in `s` to the nearest character index (clamped to `s`).
pub fn utf16_to_char_index(s: &str, utf16_idx: usize) -> usize {
    let mut utf16_acc = 0usize;
    let mut char_acc = 0usize;
    for ch in s.chars() {
        let len16 = ch.len_utf16();
        if utf16_acc + len16 > utf16_idx {
            break;
        }
        utf16_acc += len16;
        char_acc += 1;
    }
    char_acc
}

/// Filters `raw` character-by-character using `is_valid_char` and truncates to `max_length.max(1)`.
pub fn filter_pin_chars_with(
    raw: &str,
    max_length: usize,
    is_valid_char: impl Fn(&str) -> bool,
) -> String {
    let limit = max_length.max(1);
    let mut out = String::new();
    let mut count = 0usize;
    let mut buf = [0u8; 4];
    for ch in raw.chars() {
        if count >= limit {
            break;
        }
        let s = ch.encode_utf8(&mut buf);
        if is_valid_char(s) {
            out.push(ch);
            count += 1;
        }
    }
    out
}

/// Sanitizes `raw` against `is_valid` and `max_length.max(1)`.
///
/// Accepts the truncated string directly if it already satisfies `is_valid`,
/// otherwise filters invalid characters individually so partial pastes keep valid entries.
pub fn sanitize_pin_value_with(
    raw: &str,
    max_length: usize,
    is_valid: impl Fn(&str) -> bool,
) -> String {
    let truncated = truncate_pin_chars(raw, max_length);
    if truncated.is_empty() || is_valid(&truncated) {
        return truncated;
    }
    let filtered = filter_pin_chars_with(raw, max_length, &is_valid);
    if filtered.is_empty() || is_valid(&filtered) {
        filtered
    } else {
        String::new()
    }
}

/// Returns `true` when a `keydown` event's `key` string should have `prevent_default()` applied.
///
/// Named/control keys, IME composition, and `Ctrl`/`Cmd` shortcuts are ignored.
pub fn should_prevent_keydown_with(
    key: &str,
    ctrl_or_meta: bool,
    is_composing: bool,
    is_valid: impl Fn(&str) -> bool,
) -> bool {
    if ctrl_or_meta || is_composing || key.chars().count() != 1 {
        return false;
    }
    !is_valid(key)
}

/// Minimal edit diff extracted between two strings (`prev` -> `next`) in character indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinEditSegment {
    pub prefix_chars: usize,
    pub prev_end_chars: usize,
    pub inserted: String,
}

/// Computes the common-prefix / common-suffix edit segment between `prev` and `next`.
pub fn extract_pin_edit(prev: &str, next: &str) -> PinEditSegment {
    let prev_chars: Vec<char> = prev.chars().collect();
    let next_chars: Vec<char> = next.chars().collect();
    let min_len = prev_chars.len().min(next_chars.len());

    let mut prefix = 0usize;
    while prefix < min_len && prev_chars[prefix] == next_chars[prefix] {
        prefix += 1;
    }

    let mut suffix = 0usize;
    while prefix + suffix < min_len
        && prev_chars[prev_chars.len() - 1 - suffix] == next_chars[next_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let prev_end_chars = prev_chars.len() - suffix;
    let next_end_chars = next_chars.len() - suffix;
    let inserted: String = next_chars[prefix..next_end_chars].iter().collect();

    PinEditSegment {
        prefix_chars: prefix,
        prev_end_chars,
        inserted,
    }
}

/// Result of splicing pasted/autofilled text into an existing `PinInput` value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinSpliceOutcome {
    pub value: String,
    pub selection_start: usize,
    pub selection_end: usize,
    pub changed: bool,
}

/// Splices `inserted` into `current` across `selection_start..selection_end`, applying
/// `transform` (paste transformer), `is_valid` pattern filtering, and `max_length` truncation.
pub fn apply_pin_splice(
    current: &str,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
    inserted: &str,
    max_length: usize,
    transform: impl Fn(&str) -> String,
    is_valid: impl Fn(&str) -> bool,
) -> PinSpliceOutcome {
    let max_len = max_length.max(1);
    let current_chars: Vec<char> = current.chars().collect();
    let start = selection_start
        .unwrap_or(current_chars.len())
        .min(current_chars.len());
    let end = selection_end
        .unwrap_or(start)
        .clamp(start, current_chars.len());

    let transformed = transform(inserted);
    let sanitized_insert = sanitize_pin_value_with(&transformed, max_len, &is_valid);

    if !transformed.is_empty() && sanitized_insert.is_empty() {
        return PinSpliceOutcome {
            value: truncate_pin_chars(current, max_len),
            selection_start: start,
            selection_end: end,
            changed: false,
        };
    }

    let prefix: String = current_chars[..start].iter().collect();
    let suffix: String = current_chars[end..].iter().collect();
    let combined = format!("{prefix}{sanitized_insert}{suffix}");
    let next_value = truncate_pin_chars(&combined, max_len);
    let new_len = next_value.chars().count();

    let (next_start, next_end) = if new_len == max_len {
        (max_len - 1, max_len)
    } else {
        let pos = (start + sanitized_insert.chars().count()).min(new_len);
        (pos, pos)
    };

    PinSpliceOutcome {
        changed: next_value != current,
        value: next_value,
        selection_start: next_start,
        selection_end: next_end,
    }
}

/// Normalizes raw DOM caret/selection coordinates into single-cell or range selection.
///
/// Mirrors Bits UI's selection expansion so mid-string navigation highlights a single active
/// replacement cell while end-of-partial-input insertion retains a collapsed caret for the
/// fake caret indicator.
pub fn normalize_pin_selection(
    value: &str,
    raw_start: Option<usize>,
    raw_end: Option<usize>,
    prev: PinInputSelection,
    max_length: usize,
) -> PinInputSelection {
    let max_len = max_length.max(1);
    let val_len = value.chars().count().min(max_len);

    let (Some(s_raw), Some(e_raw)) = (raw_start, raw_end) else {
        return PinInputSelection::empty();
    };

    let s = s_raw.min(val_len);
    let e = e_raw.clamp(s, val_len);

    if val_len == 0 {
        return PinInputSelection::collapsed(0);
    }

    if s < e {
        return PinInputSelection::range(s, e, prev.direction);
    }

    let c = s;
    let is_insert_mode = c == val_len && val_len < max_len;
    if is_insert_mode {
        return PinInputSelection::collapsed(c);
    }

    if c == 0 {
        return PinInputSelection::range(0, 1, PinInputSelectionDirection::Forward);
    }

    if c >= max_len {
        return PinInputSelection::range(max_len - 1, max_len, PinInputSelectionDirection::Backward);
    }

    let mut offset: isize = 0;
    let mut dir = PinInputSelectionDirection::Forward;
    if let (Some(ps), Some(pe)) = (prev.start, prev.end) {
        dir = if c < pe {
            PinInputSelectionDirection::Backward
        } else {
            PinInputSelectionDirection::Forward
        };
        let was_previously_inserting = ps == pe && ps < max_len;
        if dir == PinInputSelectionDirection::Backward && !was_previously_inserting {
            offset = -1;
        }
    }

    let start = ((c as isize) + offset).clamp(0, (val_len - 1) as isize) as usize;
    let end = (start + 1).min(val_len);
    PinInputSelection::range(start, end, dir)
}

/// Computes the initial selection range when the `PinInput` receives focus.
pub fn initial_focus_selection(value: &str, max_length: usize) -> PinInputSelection {
    let max_len = max_length.max(1);
    let val_len = value.chars().count().min(max_len);
    let start = val_len.min(max_len - 1);
    PinInputSelection::range(start, val_len, PinInputSelectionDirection::None)
}

/// Resolves a pointer coordinate `(client_x, client_y)` against rendered cell bounding boxes
/// `(index, (left, top, right, bottom))`.
///
/// 1. If the pointer falls inside a cell's bounds (`left <= x <= right && top <= y <= bottom`),
///    returns that cell's `index`.
/// 2. Otherwise (e.g., clicking in the gap or separator between cells), prefers the horizontally
///    closest cell whose vertical span contains `client_y`, or falls back to the nearest cell
///    overall by distance to its bounding rectangle (tie-broken by horizontal center distance).
pub fn resolve_pin_cell_hit(
    cell_rects: &[(usize, (f64, f64, f64, f64))],
    client_x: f64,
    client_y: f64,
) -> Option<usize> {
    if cell_rects.is_empty() {
        return None;
    }

    if let Some(&(index, _)) = cell_rects.iter().find(|&&(_, (left, top, right, bottom))| {
        client_x >= left && client_x <= right && client_y >= top && client_y <= bottom
    }) {
        return Some(index);
    }

    let any_vertical_match = cell_rects
        .iter()
        .any(|&(_, (_, top, _, bottom))| client_y >= top && client_y <= bottom);

    cell_rects
        .iter()
        .filter(|&&(_, (_, top, _, bottom))| {
            !any_vertical_match || (client_y >= top && client_y <= bottom)
        })
        .min_by(|&&(_, (l1, t1, r1, b1)), &&(_, (l2, t2, r2, b2))| {
            let metric = |left: f64, top: f64, right: f64, bottom: f64| {
                let dx = if client_x < left {
                    left - client_x
                } else if client_x > right {
                    client_x - right
                } else {
                    0.0
                };
                let dy = if client_y < top {
                    top - client_y
                } else if client_y > bottom {
                    client_y - bottom
                } else {
                    0.0
                };
                let dist = dx.hypot(dy);
                let center_dx = ((left + right) * 0.5 - client_x).abs();
                (dist, center_dx)
            };
            let (d1, c1) = metric(l1, t1, r1, b1);
            let (d2, c2) = metric(l2, t2, r2, b2);
            d1.total_cmp(&d2).then_with(|| c1.total_cmp(&c2))
        })
        .map(|&(index, _)| index)
}

/// Computes the target [`PinInputSelection`] when clicking cell `clicked_idx`.
///
/// Clamps `clicked_idx` to `val_len.min(max_len - 1)`:
/// - Clicking an already-filled cell (`target < val_len`) selects `[target, target + 1)`
///   with `Forward` direction for immediate single-character replacement.
/// - Clicking the next empty slot (`target == val_len`) places a collapsed caret at `val_len`
///   so the fake caret renders in insert mode.
pub fn target_selection_for_cell_click(
    clicked_idx: usize,
    val_len: usize,
    max_length: usize,
) -> PinInputSelection {
    let max_len = max_length.max(1);
    let clamped_val_len = val_len.min(max_len);
    let target = clicked_idx.min(clamped_val_len).min(max_len - 1);
    if target < clamped_val_len {
        PinInputSelection::range(target, target + 1, PinInputSelectionDirection::Forward)
    } else {
        PinInputSelection::collapsed(clamped_val_len)
    }
}

/// Resolves a pointer coordinate against cell bounding boxes and computes the target
/// [`PinInputSelection`] for `value` and `max_length`.
pub fn resolve_pin_pointer_selection(
    cell_rects: &[(usize, (f64, f64, f64, f64))],
    client_x: f64,
    client_y: f64,
    value: &str,
    max_length: usize,
) -> Option<PinInputSelection> {
    let clicked_idx = resolve_pin_cell_hit(cell_rects, client_x, client_y)?;
    let val_len = value.chars().count();
    Some(target_selection_for_cell_click(
        clicked_idx,
        val_len,
        max_length,
    ))
}

/// Host-neutral navigation keys supported for `Shift+<key>` selection extension in `PinInput`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PinShiftNavKey {
    ArrowLeft,
    ArrowRight,
    Home,
    End,
}

/// Resolves a pointer drag from `anchor_idx` to `current_idx` into a [`PinInputSelection`].
pub fn resolve_pin_drag_selection(
    anchor_idx: usize,
    current_idx: usize,
    value: &str,
    max_length: usize,
) -> PinInputSelection {
    let max_len = max_length.max(1);
    let val_len = value.chars().count().min(max_len);
    if val_len == 0 {
        return PinInputSelection::collapsed(0);
    }
    if anchor_idx == current_idx {
        return target_selection_for_cell_click(anchor_idx, val_len, max_len);
    }
    if anchor_idx >= val_len && current_idx >= val_len {
        return PinInputSelection::collapsed(val_len);
    }
    let lo = anchor_idx.min(current_idx).min(val_len - 1);
    let hi = (anchor_idx.max(current_idx) + 1).min(val_len);
    let dir = if current_idx < anchor_idx {
        PinInputSelectionDirection::Backward
    } else {
        PinInputSelectionDirection::Forward
    };
    PinInputSelection::range(lo, hi, dir)
}

/// Extends or shrinks `PinInputSelection` in response to `Shift+<PinShiftNavKey>`,
/// tracking `(anchor, focus)` across pivot crossings and empty-slot boundaries.
pub fn extend_pin_selection_with_shift(
    value: &str,
    max_length: usize,
    current: PinInputSelection,
    shift_anchor: Option<(usize, usize)>,
    key: PinShiftNavKey,
) -> (PinInputSelection, Option<(usize, usize)>) {
    let max_len = max_length.max(1);
    let val_len = value.chars().count().min(max_len);
    if val_len == 0 {
        return (PinInputSelection::collapsed(0), None);
    }

    let base = if current.start.is_some() && current.end.is_some() {
        current
    } else {
        initial_focus_selection(value, max_len)
    };
    let s = base.start.unwrap_or(0).min(val_len);
    let e = base.end.unwrap_or(s).clamp(s, val_len);

    // Case 1: Starting from collapsed empty insert caret (val_len, val_len).
    if shift_anchor.is_none() && s == val_len && e == val_len {
        return match key {
            PinShiftNavKey::ArrowLeft => (
                PinInputSelection::range(val_len - 1, val_len, PinInputSelectionDirection::Backward),
                Some((val_len, val_len - 1)),
            ),
            PinShiftNavKey::Home => (
                PinInputSelection::range(0, val_len, PinInputSelectionDirection::Backward),
                Some((val_len, 0)),
            ),
            PinShiftNavKey::ArrowRight | PinShiftNavKey::End => {
                (PinInputSelection::collapsed(val_len), None)
            }
        };
    }

    // Case 2: Starting from single filled cell [s, s + 1) (or mid-string collapsed caret).
    if shift_anchor.is_none() && (e == s + 1 || (s == e && s < val_len)) {
        return match key {
            PinShiftNavKey::ArrowRight => {
                let focus = (s + 2).min(val_len);
                (
                    PinInputSelection::range(s, focus, PinInputSelectionDirection::Forward),
                    Some((s, focus)),
                )
            }
            PinShiftNavKey::End => (
                PinInputSelection::range(s, val_len, PinInputSelectionDirection::Forward),
                Some((s, val_len)),
            ),
            PinShiftNavKey::ArrowLeft => {
                let anchor = s + 1;
                let focus = s.saturating_sub(1);
                (
                    PinInputSelection::range(focus, anchor, PinInputSelectionDirection::Backward),
                    Some((anchor, focus)),
                )
            }
            PinShiftNavKey::Home => {
                let anchor = s + 1;
                (
                    PinInputSelection::range(0, anchor, PinInputSelectionDirection::Backward),
                    Some((anchor, 0)),
                )
            }
        };
    }

    // Case 3: Continuing an active shift extension or an existing multi-cell range.
    let (anchor, focus) = if let Some((a, f)) = shift_anchor {
        (a.min(val_len), f.min(val_len))
    } else if base.direction == PinInputSelectionDirection::Backward {
        (e, s)
    } else {
        (s, e)
    };

    if key == PinShiftNavKey::ArrowRight && anchor == focus + 1 {
        if anchor == val_len && val_len < max_len {
            return (PinInputSelection::collapsed(val_len), None);
        }
        let new_anchor = focus;
        let new_focus = (anchor + 1).min(val_len);
        return (
            PinInputSelection::range(new_anchor, new_focus, PinInputSelectionDirection::Forward),
            Some((new_anchor, new_focus)),
        );
    }

    if key == PinShiftNavKey::ArrowLeft && focus == anchor + 1 {
        let new_anchor = focus;
        let new_focus = anchor.saturating_sub(1);
        return (
            PinInputSelection::range(new_focus, new_anchor, PinInputSelectionDirection::Backward),
            Some((new_anchor, new_focus)),
        );
    }

    let new_focus = match key {
        PinShiftNavKey::ArrowLeft => focus.saturating_sub(1),
        PinShiftNavKey::ArrowRight => (focus + 1).min(val_len),
        PinShiftNavKey::Home => 0,
        PinShiftNavKey::End => val_len,
    };

    if new_focus == anchor {
        if anchor == val_len && val_len < max_len {
            return (PinInputSelection::collapsed(val_len), None);
        }
        let pivot = anchor.min(val_len - 1);
        let dir = if anchor == val_len {
            PinInputSelectionDirection::Backward
        } else {
            PinInputSelectionDirection::Forward
        };
        return (PinInputSelection::range(pivot, pivot + 1, dir), None);
    }

    let lo = anchor.min(new_focus);
    let hi = anchor.max(new_focus);
    let dir = if new_focus < anchor {
        PinInputSelectionDirection::Backward
    } else {
        PinInputSelectionDirection::Forward
    };
    (PinInputSelection::range(lo, hi, dir), Some((anchor, new_focus)))
}

/// Collapses any multi-cell [`PinInputSelection`] to a single active cell or collapsed insert caret.
pub fn collapse_pin_selection_to_single_cell(
    value: &str,
    sel: PinInputSelection,
    max_length: usize,
) -> PinInputSelection {
    let max_len = max_length.max(1);
    let (Some(s_raw), Some(e_raw)) = (sel.start, sel.end) else {
        return PinInputSelection::empty();
    };
    let val_len = value.chars().count().min(max_len);
    if val_len == 0 {
        return PinInputSelection::collapsed(0);
    }
    let s = s_raw.min(val_len);
    let e = e_raw.clamp(s, val_len);
    if s == e {
        return normalize_pin_selection(value, Some(s), Some(e), sel, max_len);
    }
    if e - s <= 1 {
        return if s == s_raw && e == e_raw {
            sel
        } else {
            PinInputSelection::range(s, e, sel.direction)
        };
    }
    let target = s.min(val_len - 1);
    PinInputSelection::range(target, target + 1, PinInputSelectionDirection::Forward)
}

/// Navigates by a single cell (or endpoint) for `Shift+<PinShiftNavKey>` without creating a
/// multi-cell selection range.
pub fn step_pin_single_cell(
    value: &str,
    max_length: usize,
    current: PinInputSelection,
    key: PinShiftNavKey,
) -> PinInputSelection {
    let max_len = max_length.max(1);
    let val_len = value.chars().count().min(max_len);
    if val_len == 0 {
        return PinInputSelection::collapsed(0);
    }
    let cur = current
        .start
        .unwrap_or_else(|| val_len.min(max_len - 1))
        .min(val_len);
    match key {
        PinShiftNavKey::Home => PinInputSelection::range(0, 1, PinInputSelectionDirection::Forward),
        PinShiftNavKey::End => {
            if val_len < max_len {
                PinInputSelection::collapsed(val_len)
            } else {
                PinInputSelection::range(
                    max_len - 1,
                    max_len,
                    PinInputSelectionDirection::Backward,
                )
            }
        }
        PinShiftNavKey::ArrowLeft => {
            if cur == 0 {
                PinInputSelection::range(0, 1, PinInputSelectionDirection::Forward)
            } else {
                let prev = cur.saturating_sub(1).min(val_len - 1);
                PinInputSelection::range(prev, prev + 1, PinInputSelectionDirection::Backward)
            }
        }
        PinShiftNavKey::ArrowRight => {
            if cur + 1 < val_len {
                let next = cur + 1;
                PinInputSelection::range(next, next + 1, PinInputSelectionDirection::Forward)
            } else if val_len < max_len {
                PinInputSelection::collapsed(val_len)
            } else {
                PinInputSelection::range(
                    max_len - 1,
                    max_len,
                    PinInputSelectionDirection::Forward,
                )
            }
        }
    }
}

/// Transforms, filters, and truncates `raw_paste` to flush and replace the entire `PinInput`
/// value from index `0`.
///
/// Returns `None` when the resulting string is empty so invalid/empty pastes leave the
/// existing value untouched.
pub fn replace_pin_value_from_paste_with<T, F>(
    raw_paste: &str,
    max_length: usize,
    transform: Option<T>,
    allow_char: F,
) -> Option<(String, usize)>
where
    T: Fn(&str) -> String,
    F: Fn(&str) -> bool,
{
    let transformed = match transform {
        Some(ref tf) => tf(raw_paste),
        None => raw_paste.to_string(),
    };
    let next_val = filter_pin_chars_with(&transformed, max_length, allow_char);
    if next_val.is_empty() {
        None
    } else {
        let len = next_val.chars().count();
        Some((next_val, len))
    }
}

/// Projects `value`, `selection`, `is_focused`, and `disabled` into `max_length.max(1)` cell snapshots.
pub fn compute_pin_cells(
    value: &str,
    max_length: usize,
    selection: PinInputSelection,
    is_focused: bool,
    disabled: bool,
) -> Vec<PinInputCellData> {
    let max_len = max_length.max(1);
    let chars: Vec<char> = value.chars().take(max_len).collect();

    (0..max_len)
        .map(|index| {
            let ch = chars.get(index).copied();
            let is_active = if !is_focused || disabled {
                false
            } else {
                match (selection.start, selection.end) {
                    (Some(s), Some(e)) => (s == e && index == s) || (index >= s && index < e),
                    _ => false,
                }
            };
            PinInputCellData {
                index,
                char: ch,
                is_active,
                has_fake_caret: is_active && ch.is_none(),
            }
        })
        .collect()
}

/// Returns `true` strictly on the edge transition from `< max_length` to `== max_length`.
pub fn should_fire_on_complete(prev_value: &str, next_value: &str, max_length: usize) -> bool {
    let max_len = max_length.max(1);
    prev_value != next_value
        && prev_value.chars().count() < max_len
        && next_value.chars().count() == max_len
}

/// Projects an optional cell character through `mask` and `mask_char`.
pub const fn project_pin_cell_char(
    ch: Option<char>,
    mask: bool,
    mask_char: char,
) -> Option<char> {
    match ch {
        Some(_) if mask => Some(mask_char),
        Some(c) => Some(c),
        None => None,
    }
}

/// Formats an optional cell character into its rendered string (empty string when `None`).
pub fn format_pin_cell_display(ch: Option<char>, mask: bool, mask_char: char) -> String {
    match project_pin_cell_char(ch, mask, mask_char) {
        Some(c) => c.to_string(),
        None => String::new(),
    }
}

/// Pure headless configuration and snapshot/attribute builder for `PinInput`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinInputDefinition {
    relationships: PinInputRelationships,
    max_length: usize,
    builtin_pattern: PinInputBuiltinPattern,
    text_align: PinInputTextAlign,
    input_mode: PinInputMode,
    autocomplete: PinInputAutoComplete,
    mask: bool,
    mask_char: char,
    disabled: bool,
    read_only: bool,
    required: bool,
    invalid: bool,
    auto_focus: bool,
    placeholder: Option<String>,
    name: Option<String>,
    form: Option<String>,
}

impl PinInputDefinition {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            relationships: PinInputRelationships::new(scope),
            max_length: 6,
            builtin_pattern: PinInputBuiltinPattern::Any,
            text_align: PinInputTextAlign::Left,
            input_mode: PinInputMode::Numeric,
            autocomplete: PinInputAutoComplete::OneTimeCode,
            mask: false,
            mask_char: '•',
            disabled: false,
            read_only: false,
            required: false,
            invalid: false,
            auto_focus: false,
            placeholder: None,
            name: None,
            form: None,
        }
    }

    pub const fn parts() -> &'static [PinInputPart] {
        &PIN_INPUT_PARTS
    }

    pub fn relationships(&self) -> &PinInputRelationships {
        &self.relationships
    }

    pub const fn max_length(&self) -> usize {
        self.max_length
    }

    pub const fn builtin_pattern(&self) -> PinInputBuiltinPattern {
        self.builtin_pattern
    }

    pub const fn text_align(&self) -> PinInputTextAlign {
        self.text_align
    }

    pub const fn input_mode(&self) -> PinInputMode {
        self.input_mode
    }

    pub const fn autocomplete(&self) -> PinInputAutoComplete {
        self.autocomplete
    }

    pub const fn is_masked(&self) -> bool {
        self.mask
    }

    pub const fn mask_char(&self) -> char {
        self.mask_char
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub const fn is_required(&self) -> bool {
        self.required
    }

    pub const fn is_invalid(&self) -> bool {
        self.invalid
    }

    pub const fn is_auto_focus(&self) -> bool {
        self.auto_focus
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn form(&self) -> Option<&str> {
        self.form.as_deref()
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length.max(1);
        self
    }

    pub fn with_builtin_pattern(mut self, pattern: PinInputBuiltinPattern) -> Self {
        self.builtin_pattern = pattern;
        self
    }

    pub fn with_text_align(mut self, text_align: PinInputTextAlign) -> Self {
        self.text_align = text_align;
        self
    }

    pub fn with_input_mode(mut self, input_mode: PinInputMode) -> Self {
        self.input_mode = input_mode;
        self
    }

    pub fn with_autocomplete(mut self, autocomplete: PinInputAutoComplete) -> Self {
        self.autocomplete = autocomplete;
        self
    }

    pub fn with_mask(mut self, mask: bool) -> Self {
        self.mask = mask;
        self
    }

    pub fn with_mask_char(mut self, mask_char: char) -> Self {
        self.mask_char = mask_char;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn with_auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    pub fn with_placeholder(mut self, placeholder: Option<String>) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_form(mut self, form: Option<String>) -> Self {
        self.form = form;
        self
    }

    pub fn snapshot(
        &self,
        value: &str,
        selection: PinInputSelection,
        is_focused: bool,
        is_hovering: bool,
    ) -> PinInputSnapshot {
        let clamped_value = truncate_pin_chars(value, self.max_length);
        let cells = compute_pin_cells(
            &clamped_value,
            self.max_length,
            selection,
            is_focused,
            self.disabled,
        );
        PinInputSnapshot {
            value: clamped_value,
            cells,
            is_focused: is_focused && !self.disabled,
            is_hovering: is_hovering && !self.disabled,
            disabled: self.disabled,
            mask: self.mask,
        }
    }

    pub fn root_attributes(&self, custom_id: Option<String>) -> PinInputRootAttributes {
        let id = custom_id.unwrap_or_else(|| self.relationships.root_id().to_string());
        PinInputRootAttributes::new(id, self.disabled).with_invalid(self.invalid)
    }

    pub fn input_attributes(
        &self,
        custom_input_id: Option<String>,
        value: &str,
        selection: PinInputSelection,
        html_pattern: Option<&'static str>,
    ) -> PinInputInputAttributes {
        let id = custom_input_id.unwrap_or_else(|| self.relationships.input_id().to_string());
        PinInputInputAttributes::new(
            id,
            self.mask,
            self.autocomplete.as_str(),
            self.input_mode.as_str(),
            html_pattern,
            self.max_length,
            truncate_pin_chars(value, self.max_length),
            self.placeholder.clone(),
            self.disabled,
            self.read_only,
            self.required,
            self.auto_focus,
            self.name.clone(),
            self.form.clone(),
            self.text_align.as_str(),
            selection.start,
            selection.end,
        )
        .with_invalid(self.invalid)
    }

    pub fn cell_attributes(
        &self,
        index: usize,
        custom_id: Option<String>,
        is_active: bool,
    ) -> PinInputCellAttributes {
        let id = custom_id.unwrap_or_else(|| self.relationships.cell_id(index));
        PinInputCellAttributes::new(id, index, is_active, self.disabled).with_invalid(self.invalid)
    }
}

pub type PinInput = PinInputDefinition;

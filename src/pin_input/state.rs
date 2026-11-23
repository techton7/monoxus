use dioxus::prelude::Callback;

pub use monoxus_core::pin_input::state::*;

use super::types::PinInputPattern;

/// Sanitizes `raw` against a [`PinInputPattern`] and `max_length`.
pub fn sanitize_pin_value(raw: &str, max_length: usize, pattern: &PinInputPattern) -> String {
    sanitize_pin_value_with(raw, max_length, |s| pattern.matches_str(s))
}

/// Filters `raw` character-by-character against a [`PinInputPattern`] and truncates to `max_length`.
pub fn filter_pin_chars(raw: &str, max_length: usize, pattern: &PinInputPattern) -> String {
    filter_pin_chars_with(raw, max_length, |s| pattern.matches_str(s))
}

/// Applies a paste/autofill splice using optional Dioxus [`Callback`] transformer and [`PinInputPattern`].
pub fn apply_pin_splice_with_callbacks(
    current: &str,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
    inserted: &str,
    max_length: usize,
    paste_transformer: Option<&Callback<String, String>>,
    pattern: &PinInputPattern,
) -> PinSpliceOutcome {
    apply_pin_splice(
        current,
        selection_start,
        selection_end,
        inserted,
        max_length,
        |raw| match paste_transformer {
            Some(cb) => cb.call(raw.to_string()),
            None => raw.to_string(),
        },
        |s| pattern.matches_str(s),
    )
}

/// Replaces the `PinInput` value from index `0` using optional Dioxus [`Callback`] transformer
/// and [`PinInputPattern`].
pub fn replace_pin_value_from_paste_with_callbacks(
    raw_paste: &str,
    max_length: usize,
    paste_transformer: Option<&Callback<String, String>>,
    pattern: &PinInputPattern,
) -> Option<(String, usize)> {
    let transform = paste_transformer.map(|cb| move |raw: &str| cb.call(raw.to_string()));
    replace_pin_value_from_paste_with(raw_paste, max_length, transform, |s| {
        pattern.matches_str(s)
    })
}

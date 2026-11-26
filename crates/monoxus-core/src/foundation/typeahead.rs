//! Unified headless typeahead buffering and prefix matching for menu and select collections.

/// Configurable behavior for [`TypeaheadBuffer`] and [`match_next_typeahead_item`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeaheadConfig {
    /// Idle window in milliseconds before the buffered search query resets.
    pub timeout_ms: u64,
    /// When `true`, typing a single character or repeating the same character cycles through
    /// matching items starting after the currently highlighted item (Radix/Bits menu contract).
    /// When `false`, searches sequentially from the first enabled item without collapsing
    /// repeated characters (Select contract).
    pub cycle_repeated_char: bool,
    /// When `true`, a leading space is accepted into an empty query buffer; when `false`,
    /// a leading space is ignored so `Space` can activate the currently highlighted item.
    /// In both modes, mid-search spaces (e.g. `"new y"` / `"more t"`) narrow an active query.
    pub allow_space_in_query: bool,
}

impl TypeaheadConfig {
    /// Canonical menu preset (`1000ms` timeout, repeated-char cycling, Space activates when idle).
    pub const MENU: Self = Self {
        timeout_ms: 1000,
        cycle_repeated_char: true,
        allow_space_in_query: false,
    };

    /// Canonical select preset (`500ms` timeout, direct prefix matching, spaces accepted).
    pub const SELECT: Self = Self {
        timeout_ms: 500,
        cycle_repeated_char: false,
        allow_space_in_query: true,
    };
}

impl Default for TypeaheadConfig {
    fn default() -> Self {
        Self::MENU
    }
}

/// Trait implemented by collection items participating in typeahead prefix search.
pub trait TypeaheadItem {
    /// Stable identifier returned on match (e.g. menu item DOM ID or select option value).
    fn typeahead_id(&self) -> &str;

    /// Primary human-readable label text matched case-insensitively by prefix.
    fn typeahead_label(&self) -> &str;

    /// Optional secondary value token matched case-insensitively by prefix (used by `Select`).
    fn typeahead_value(&self) -> Option<&str> {
        None
    }

    /// Returns `true` when the item is disabled and must be skipped.
    fn typeahead_disabled(&self) -> bool {
        false
    }
}

/// Borrowed zero-allocation adapter implementing [`TypeaheadItem`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeaheadCandidate<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub value: Option<&'a str>,
    pub disabled: bool,
}

impl<'a> TypeaheadCandidate<'a> {
    #[inline]
    pub const fn label_only(id: &'a str, label: &'a str, disabled: bool) -> Self {
        Self {
            id,
            label,
            value: None,
            disabled,
        }
    }

    #[inline]
    pub const fn with_value(id: &'a str, label: &'a str, value: &'a str, disabled: bool) -> Self {
        Self {
            id,
            label,
            value: Some(value),
            disabled,
        }
    }
}

impl TypeaheadItem for TypeaheadCandidate<'_> {
    #[inline]
    fn typeahead_id(&self) -> &str {
        self.id
    }

    #[inline]
    fn typeahead_label(&self) -> &str {
        self.label
    }

    #[inline]
    fn typeahead_value(&self) -> Option<&str> {
        self.value
    }

    #[inline]
    fn typeahead_disabled(&self) -> bool {
        self.disabled
    }
}

/// Stateful typeahead prefix buffer supporting both Menu and Select contracts.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeaheadBuffer {
    config: TypeaheadConfig,
    search: String,
    last_typed_ms: Option<u64>,
}

impl TypeaheadBuffer {
    pub const fn new(config: TypeaheadConfig) -> Self {
        Self {
            config,
            search: String::new(),
            last_typed_ms: None,
        }
    }

    #[inline]
    pub const fn config(&self) -> TypeaheadConfig {
        self.config
    }

    #[inline]
    pub fn search(&self) -> &str {
        &self.search
    }

    #[inline]
    pub const fn last_typed_ms(&self) -> Option<u64> {
        self.last_typed_ms
    }

    /// Returns `true` while the buffer holds a non-empty query within `config.timeout_ms`.
    #[inline]
    pub fn is_active(&self, now_ms: u64) -> bool {
        !self.search.is_empty()
            && self
                .last_typed_ms
                .is_some_and(|typed_at| now_ms.saturating_sub(typed_at) < self.config.timeout_ms)
    }

    /// Clears the buffer if `config.timeout_ms` has elapsed since the last keypress.
    /// Returns `true` if a non-empty buffer was cleared.
    pub fn expire_if_elapsed(&mut self, now_ms: u64) -> bool {
        if self.search.is_empty() {
            return false;
        }
        let elapsed = self
            .last_typed_ms
            .map(|typed_at| now_ms.saturating_sub(typed_at))
            .unwrap_or(self.config.timeout_ms);
        if elapsed >= self.config.timeout_ms {
            self.reset();
            true
        } else {
            false
        }
    }

    /// Resets the buffered query and timestamp immediately.
    pub fn reset(&mut self) {
        self.search.clear();
        self.last_typed_ms = None;
    }

    /// Appends a character to the query buffer according to [`TypeaheadConfig`] and returns
    /// the updated query slice if the character was accepted.
    pub fn push_query_char(&mut self, ch: char, now_ms: u64) -> Option<&str> {
        if ch.is_control() {
            return None;
        }

        self.expire_if_elapsed(now_ms);

        if self.search.is_empty() && ch == ' ' && !self.config.allow_space_in_query {
            return None;
        }

        self.search.push(ch);
        self.last_typed_ms = Some(now_ms);
        Some(&self.search)
    }

    /// Pushes a printable character and returns the matching enabled item's ID, if any.
    pub fn push_char<T: TypeaheadItem>(
        &mut self,
        ch: char,
        now_ms: u64,
        items: &[T],
        current_highlighted_id: Option<&str>,
    ) -> Option<String> {
        let query = self.push_query_char(ch, now_ms)?.to_owned();
        match_next_typeahead_item(
            items,
            &query,
            current_highlighted_id,
            self.config.cycle_repeated_char,
        )
    }
}

impl Default for TypeaheadBuffer {
    fn default() -> Self {
        Self::new(TypeaheadConfig::default())
    }
}

/// Pure case-insensitive typeahead prefix matcher supporting both single-label (`Menu`) and
/// dual `label` + `value` (`Select`) matching, repeated-character cycling, and mid-search spaces.
pub fn match_next_typeahead_item<T: TypeaheadItem>(
    items: &[T],
    raw_search: &str,
    current_highlighted_id: Option<&str>,
    cycle_repeated_char: bool,
) -> Option<String> {
    if raw_search.trim().is_empty() {
        return None;
    }

    let enabled: Vec<&T> = items
        .iter()
        .filter(|item| {
            !item.typeahead_disabled()
                && (!item.typeahead_label().trim().is_empty()
                    || item
                        .typeahead_value()
                        .is_some_and(|value| !value.trim().is_empty()))
        })
        .collect();
    if enabled.is_empty() {
        return None;
    }

    let lowered = raw_search.to_lowercase();
    let mut chars = lowered.chars();
    let first_char = chars.next()?;
    let is_repeated =
        cycle_repeated_char && lowered.chars().count() > 1 && chars.all(|c| c == first_char);

    let normalized = if is_repeated {
        first_char.to_string()
    } else {
        lowered
    };
    let single_char_cycle = cycle_repeated_char && normalized.chars().count() == 1;

    let current_index = if cycle_repeated_char {
        current_highlighted_id
            .and_then(|current| enabled.iter().position(|item| item.typeahead_id() == current))
    } else {
        None
    };

    let start_index = match current_index {
        Some(idx) if single_char_cycle => (idx + 1) % enabled.len(),
        Some(idx) => idx,
        None => 0,
    };

    for offset in 0..enabled.len() {
        let candidate = enabled[(start_index + offset) % enabled.len()];
        let label_matches = candidate
            .typeahead_label()
            .trim_start()
            .to_lowercase()
            .starts_with(&normalized);
        let value_matches = candidate.typeahead_value().is_some_and(|value| {
            value
                .trim_start()
                .to_lowercase()
                .starts_with(&normalized)
        });
        if label_matches || value_matches {
            return Some(candidate.typeahead_id().to_owned());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_preset_cycles_repeated_chars_guards_leading_space_and_allows_mid_space() {
        let items = [
            TypeaheadCandidate::label_only("i-1", "Cut", false),
            TypeaheadCandidate::label_only("i-2", "Copy", true),
            TypeaheadCandidate::label_only("i-3", "Paste", false),
            TypeaheadCandidate::label_only("i-4", "Print", false),
            TypeaheadCandidate::label_only("i-5", "More Tools", false),
        ];

        let mut buffer = TypeaheadBuffer::new(TypeaheadConfig::MENU);
        assert_eq!(buffer.config().timeout_ms, 1000);
        assert!(buffer.config().cycle_repeated_char);
        assert!(!buffer.config().allow_space_in_query);

        // Leading space does not start a search in menu mode.
        assert_eq!(buffer.push_char(' ', 100, &items, Some("i-1")), None);
        assert!(!buffer.is_active(100));

        // Repeated 'p' cycles across enabled items ("Paste" -> "Print" -> "Paste").
        assert_eq!(
            buffer.push_char('p', 200, &items, None),
            Some("i-3".to_owned())
        );
        assert!(buffer.is_active(250));
        assert_eq!(
            buffer.push_char('p', 300, &items, Some("i-3")),
            Some("i-4".to_owned())
        );
        assert_eq!(
            buffer.push_char('p', 400, &items, Some("i-4")),
            Some("i-3".to_owned())
        );

        // 1000ms timeout resets buffer; multi-word query with mid-search space matches "More Tools".
        assert!(buffer.expire_if_elapsed(1400));
        assert!(!buffer.is_active(1400));
        assert_eq!(
            buffer.push_char('m', 1500, &items, Some("i-3")),
            Some("i-5".to_owned())
        );
        assert_eq!(
            buffer.push_char('o', 1600, &items, Some("i-5")),
            Some("i-5".to_owned())
        );
        assert_eq!(
            buffer.push_char('r', 1700, &items, Some("i-5")),
            Some("i-5".to_owned())
        );
        assert_eq!(
            buffer.push_char('e', 1800, &items, Some("i-5")),
            Some("i-5".to_owned())
        );
        assert_eq!(
            buffer.push_char(' ', 1850, &items, Some("i-5")),
            Some("i-5".to_owned())
        );
        assert_eq!(
            buffer.push_char('t', 1900, &items, Some("i-5")),
            Some("i-5".to_owned())
        );
    }

    #[test]
    fn select_preset_matches_label_and_value_mid_space_and_500ms_timeout() {
        let items = [
            TypeaheadCandidate::with_value("ny", "New York", "ny", false),
            TypeaheadCandidate::with_value("nj", "New Jersey", "nj", false),
            TypeaheadCandidate::with_value("aa", "Aardvark", "aardvark", false),
            TypeaheadCandidate::with_value("ap", "Apple", "apple", false),
            TypeaheadCandidate::with_value("dis", "New Zealand", "nz", true),
        ];

        let mut buffer = TypeaheadBuffer::new(TypeaheadConfig::SELECT);
        assert_eq!(buffer.config().timeout_ms, 500);
        assert!(!buffer.config().cycle_repeated_char);
        assert!(buffer.config().allow_space_in_query);

        // Multi-word query with mid-search space ("new j" -> "New Jersey").
        assert_eq!(
            buffer.push_char('n', 100, &items, None),
            Some("ny".to_owned())
        );
        assert_eq!(
            buffer.push_char('e', 150, &items, Some("ny")),
            Some("ny".to_owned())
        );
        assert_eq!(
            buffer.push_char('w', 200, &items, Some("ny")),
            Some("ny".to_owned())
        );
        assert_eq!(
            buffer.push_char(' ', 250, &items, Some("ny")),
            Some("ny".to_owned())
        );
        assert_eq!(
            buffer.push_char('j', 300, &items, Some("ny")),
            Some("nj".to_owned())
        );

        // Disabled "New Zealand" is skipped even if typed.
        assert!(!buffer.is_active(800));

        // Dual label+value matching: typing "nj" matches value "nj" even though label is "New Jersey".
        assert_eq!(
            buffer.push_char('n', 900, &items, None),
            Some("ny".to_owned())
        );
        assert_eq!(
            buffer.push_char('j', 950, &items, Some("ny")),
            Some("nj".to_owned())
        );

        // Without repeated-char cycling, "aa" matches "Aardvark" instead of cycling to "Apple".
        buffer.reset();
        assert_eq!(
            buffer.push_char('a', 1500, &items, None),
            Some("aa".to_owned())
        );
        assert_eq!(
            buffer.push_char('a', 1600, &items, Some("aa")),
            Some("aa".to_owned())
        );
    }
}

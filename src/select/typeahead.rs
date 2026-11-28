use dioxus::prelude::keyboard_types::Key;
use dioxus::prelude::*;
use ui_sdk::foundation::typeahead::{TypeaheadCandidate, TypeaheadConfig};

use crate::foundation::keyboard::{NavKey, is_ime_composing_key, translate_nav_key};

use super::runtime::SelectRuntime;

const SELECT_TYPEAHEAD_WINDOW_MS: f64 = TypeaheadConfig::SELECT.timeout_ms as f64;

impl SelectRuntime {
    pub fn typeahead_buffer(&self) -> String {
        self.state.typeahead.read().search().to_owned()
    }

    pub fn has_active_typeahead(&self, now_ms: f64) -> bool {
        let buf = self.state.typeahead.read();
        let last_time = *self.state.last_key_timestamp_ms.read();
        !buf.search().is_empty() && (now_ms - last_time) <= SELECT_TYPEAHEAD_WINDOW_MS
    }

    fn push_select_typeahead_char(&self, ch: char, now_ms: f64, include_select_def: bool) -> Option<String> {
        let last_time = *self.state.last_key_timestamp_ms.read();
        let mut ta_sig = self.state.typeahead;

        let effective_ms_u64 = {
            let mut buf = ta_sig.write();
            if now_ms - last_time > SELECT_TYPEAHEAD_WINDOW_MS {
                buf.reset();
            }
            // Preserve exact inclusive 500.0ms boundary against `TypeaheadBuffer`'s `< 500` check.
            if !buf.search().is_empty()
                && (now_ms - last_time) <= SELECT_TYPEAHEAD_WINDOW_MS
                && let Some(prev_u64) = buf.last_typed_ms()
            {
                let raw = now_ms.max(0.0) as u64;
                if raw.saturating_sub(prev_u64) >= TypeaheadConfig::SELECT.timeout_ms {
                    prev_u64.saturating_add(TypeaheadConfig::SELECT.timeout_ms - 1)
                } else {
                    raw
                }
            } else {
                now_ms.max(0.0) as u64
            }
        };

        let mut time_sig = self.state.last_key_timestamp_ms;
        time_sig.set(now_ms);

        let reg = self.state.registry.read();
        let reg_items: Vec<(String, String, bool)> = reg
            .active_items()
            .map(|i| (i.value.clone(), i.label.clone(), i.disabled))
            .collect();
        drop(reg);

        let items_snapshot = self.state.items.read().clone();
        let def_items = self.select.read().items().to_vec();

        let query = {
            let mut buf = ta_sig.write();
            let Some(q) = buf.push_query_char(ch, effective_ms_u64) else {
                return None;
            };
            q.to_owned()
        };

        if !reg_items.is_empty() {
            let candidates: Vec<TypeaheadCandidate<'_>> = reg_items
                .iter()
                .map(|(v, l, d)| TypeaheadCandidate::with_value(v, l, v, *d))
                .collect();
            if let Some(matched) = ui_sdk::foundation::typeahead::match_next_typeahead_item(
                &candidates,
                &query,
                None,
                TypeaheadConfig::SELECT.cycle_repeated_char,
            ) {
                return Some(matched);
            }
        }

        if !items_snapshot.is_empty() {
            let candidates: Vec<TypeaheadCandidate<'_>> = items_snapshot
                .iter()
                .map(|i| TypeaheadCandidate::with_value(&i.value, &i.text, &i.value, i.disabled))
                .collect();
            if let Some(matched) = ui_sdk::foundation::typeahead::match_next_typeahead_item(
                &candidates,
                &query,
                None,
                TypeaheadConfig::SELECT.cycle_repeated_char,
            ) {
                return Some(matched);
            }
        }

        if include_select_def && !def_items.is_empty() {
            let candidates: Vec<TypeaheadCandidate<'_>> = def_items
                .iter()
                .map(|i| TypeaheadCandidate::with_value(&i.value, &i.text, &i.value, i.disabled))
                .collect();
            return ui_sdk::foundation::typeahead::match_next_typeahead_item(
                &candidates,
                &query,
                None,
                TypeaheadConfig::SELECT.cycle_repeated_char,
            );
        }

        None
    }

    pub fn handle_typeahead(&self, ch: char, now_ms: f64) {
        self.clear_initial_highlight_pending();
        if let Some(matched) = self.push_select_typeahead_char(ch, now_ms, false) {
            self.set_highlighted(Some(matched));
        }
    }

    pub fn handle_closed_typeahead(&self, ch: char, now_ms: f64) {
        if let Some(val) = self.push_select_typeahead_char(ch, now_ms, true) {
            self.select_item(&val);
        }
    }

    pub fn handle_trigger_keydown(&self, event: &KeyboardEvent) {
        if self.is_disabled() || self.is_open() || is_ime_composing_key(event) {
            return;
        }
        let now_ms = oxidase::time::monotonic_now_ms();
        let key = event.key();

        match translate_nav_key(&key) {
            Some(NavKey::ArrowDown | NavKey::ArrowUp | NavKey::Space | NavKey::Enter) => {
                event.prevent_default();
                self.open_dropdown();
            }
            _ => {
                if let Key::Character(ref s) = key {
                    let mut chars = s.chars();
                    if let (Some(ch), None) = (chars.next(), chars.next())
                        && (ch.is_alphanumeric() || ch.is_whitespace())
                    {
                        event.prevent_default();
                        self.handle_closed_typeahead(ch, now_ms);
                    }
                }
            }
        }
    }
}

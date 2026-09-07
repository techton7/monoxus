use dioxus::prelude::*;

use super::{runtime::SelectRuntime, types::SelectItemData};

#[cfg(target_arch = "wasm32")]
pub fn sync_items_with_document_order(content_id: &str, items: &mut Vec<SelectItemData>) {
    let script = format!(
        r#"(function() {{
            const root = document.getElementById({content_id:?});
            if (!root) return [];
            return Array.from(root.querySelectorAll('[role="option"]'))
                .map(el => el.getAttribute('data-value') || "");
        }})()"#
    );
    if let Ok(val) = js_sys::eval(&script) {
        let arr = js_sys::Array::from(&val);
        let order: Vec<String> = arr.iter().filter_map(|v| v.as_string()).collect();
        if !order.is_empty() {
            items.sort_by_key(|item| {
                order
                    .iter()
                    .position(|v| v == &item.value)
                    .unwrap_or(usize::MAX)
            });
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_items_with_document_order(_content_id: &str, _items: &mut Vec<SelectItemData>) {}

impl SelectRuntime {
    pub fn register_item(&self, val: &str, text: &str, disabled: bool) {
        let mut items = self.state.items;
        let mut list = items.write();
        if let Some(existing) = list.iter_mut().find(|i| i.value == val) {
            existing.text = text.to_owned();
            existing.disabled = disabled;
        } else {
            list.push(SelectItemData {
                value: val.to_owned(),
                text: text.to_owned(),
                disabled,
            });
        }
    }

    pub fn item_label(&self, val: &str) -> Option<String> {
        self.state
            .items
            .read()
            .iter()
            .find(|i| i.value == val)
            .map(|i| i.text.clone())
    }

    pub fn sync_dom_order(&self) {
        let content_id = self.relationships().content_id().to_owned();
        let mut items_sig = self.state.items;
        let mut list = items_sig.write();
        sync_items_with_document_order(&content_id, &mut list);
    }

    pub fn highlight_next(&self) {
        let items = self.state.items.read();
        let enabled: Vec<_> = items.iter().filter(|i| !i.disabled).collect();
        if enabled.is_empty() {
            return;
        }

        let loop_selection = self.select.loop_selection();
        let current = self.highlighted_value();
        let curr_idx = current
            .as_ref()
            .and_then(|v| enabled.iter().position(|i| &i.value == v));

        let next_idx = match curr_idx {
            Some(idx) => {
                if idx + 1 < enabled.len() {
                    idx + 1
                } else if loop_selection {
                    0
                } else {
                    idx // Clamp at end per loop = false contract
                }
            }
            None => 0,
        };

        self.set_highlighted(Some(enabled[next_idx].value.clone()));
    }

    pub fn highlight_prev(&self) {
        let items = self.state.items.read();
        let enabled: Vec<_> = items.iter().filter(|i| !i.disabled).collect();
        if enabled.is_empty() {
            return;
        }

        let loop_selection = self.select.loop_selection();
        let current = self.highlighted_value();
        let curr_idx = current
            .as_ref()
            .and_then(|v| enabled.iter().position(|i| &i.value == v));

        let prev_idx = match curr_idx {
            Some(0) => {
                if loop_selection {
                    enabled.len() - 1
                } else {
                    0 // Clamp at start per loop = false contract
                }
            }
            Some(idx) => idx - 1,
            None => enabled.len() - 1,
        };

        self.set_highlighted(Some(enabled[prev_idx].value.clone()));
    }

    pub fn highlight_first(&self) {
        let items = self.state.items.read();
        if let Some(first) = items.iter().find(|i| !i.disabled) {
            self.set_highlighted(Some(first.value.clone()));
        }
    }

    pub fn highlight_last(&self) {
        let items = self.state.items.read();
        if let Some(last) = items.iter().rev().find(|i| !i.disabled) {
            self.set_highlighted(Some(last.value.clone()));
        }
    }

    pub fn handle_content_keydown(&self, event: &KeyboardEvent, now_ms: f64) {
        if !self.is_open() {
            return;
        }
        self.sync_dom_order();
        let key = event.key().to_string();
        match key.as_str() {
            "ArrowDown" => {
                event.prevent_default();
                self.highlight_next();
            }
            "ArrowUp" => {
                event.prevent_default();
                self.highlight_prev();
            }
            "Home" => {
                event.prevent_default();
                self.highlight_first();
            }
            "End" => {
                event.prevent_default();
                self.highlight_last();
            }
            "Enter" | " " => {
                event.prevent_default();
                if let Some(cand) = self.highlighted_value() {
                    self.select_item(&cand);
                }
            }
            "Escape" => {
                event.prevent_default();
                self.close_dropdown();
            }
            "Tab" => {
                // Do NOT prevent_default(), let browser naturally advance tab focus!
                self.close_dropdown_without_restore();
            }
            _ => {
                if key.len() == 1 {
                    let ch = key.chars().next().unwrap();
                    if ch.is_alphanumeric() || ch.is_whitespace() {
                        self.handle_typeahead(ch, now_ms);
                    }
                }
            }
        }
    }
}

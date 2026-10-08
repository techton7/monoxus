use dioxus::prelude::*;

use super::{runtime::SelectRuntime, types::SelectItemData};

pub fn apply_document_order(items: &mut Vec<SelectItemData>, order: &[String]) {
    if !order.is_empty() {
        items.sort_by_key(|item| {
            order
                .iter()
                .position(|v| v == &item.value)
                .unwrap_or(usize::MAX)
        });
    }
}

impl SelectRuntime {
    pub fn item_label(&self, val: &str) -> Option<String> {
        let reg = self.state.registry.read();
        if let Some(item) = reg.active_items().find(|i| i.value == val) {
            return Some(item.label.clone());
        }
        self.state
            .items
            .read()
            .iter()
            .find(|i| i.value == val)
            .map(|i| i.text.clone())
    }

    pub fn highlight_next(&self) {
        let reg = self.state.registry.read();
        let enabled: Vec<String> = if !reg.is_empty() {
            reg.active_items()
                .filter(|i| !i.disabled)
                .map(|i| i.value.clone())
                .collect()
        } else {
            self.state
                .items
                .read()
                .iter()
                .filter(|i| !i.disabled)
                .map(|i| i.value.clone())
                .collect()
        };
        if enabled.is_empty() {
            return;
        }

        let loop_selection = self.select.loop_selection();
        let current = self.highlighted_value();
        let curr_idx = current
            .as_ref()
            .and_then(|v| enabled.iter().position(|i| i == v));

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

        self.set_highlighted(Some(enabled[next_idx].clone()));
    }

    pub fn highlight_prev(&self) {
        let reg = self.state.registry.read();
        let enabled: Vec<String> = if !reg.is_empty() {
            reg.active_items()
                .filter(|i| !i.disabled)
                .map(|i| i.value.clone())
                .collect()
        } else {
            self.state
                .items
                .read()
                .iter()
                .filter(|i| !i.disabled)
                .map(|i| i.value.clone())
                .collect()
        };
        if enabled.is_empty() {
            return;
        }

        let loop_selection = self.select.loop_selection();
        let current = self.highlighted_value();
        let curr_idx = current
            .as_ref()
            .and_then(|v| enabled.iter().position(|i| i == v));

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

        self.set_highlighted(Some(enabled[prev_idx].clone()));
    }

    pub fn highlight_first(&self) {
        let reg = self.state.registry.read();
        if let Some(first) = reg.active_items().find(|i| !i.disabled) {
            self.set_highlighted(Some(first.value.clone()));
            return;
        }
        let items = self.state.items.read();
        if let Some(first) = items.iter().find(|i| !i.disabled) {
            self.set_highlighted(Some(first.value.clone()));
        }
    }

    pub fn highlight_last(&self) {
        let reg = self.state.registry.read();
        let active: Vec<_> = reg.active_items().filter(|i| !i.disabled).collect();
        if let Some(last) = active.last() {
            self.set_highlighted(Some(last.value.clone()));
            return;
        }
        let items = self.state.items.read();
        if let Some(last) = items.iter().rev().find(|i| !i.disabled) {
            self.set_highlighted(Some(last.value.clone()));
        }
    }

    pub fn handle_content_keydown(&self, event: &KeyboardEvent, now_ms: f64) {
        if !self.is_open() {
            return;
        }
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

#[component]
pub fn SelectViewport(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or("overflow-y: auto;"),
            {children}
        }
    }
}

#[component]
pub fn SelectScrollUpButton(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "aria-hidden": "true",
            {children}
        }
    }
}

#[component]
pub fn SelectScrollDownButton(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "aria-hidden": "true",
            {children}
        }
    }
}


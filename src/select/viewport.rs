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
        if let Ok(labels) = self.state.selected_labels.try_read() {
            if let Some(lbl) = labels.get(val) {
                return Some(lbl.clone());
            }
        }
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
        self.clear_initial_highlight_pending();
        let reg = self.state.registry.read();
        let enabled: Vec<String> = if !reg.ordered_item_ids.is_empty() {
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
        self.clear_initial_highlight_pending();
        let reg = self.state.registry.read();
        let enabled: Vec<String> = if !reg.ordered_item_ids.is_empty() {
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
        self.clear_initial_highlight_pending();
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
        self.clear_initial_highlight_pending();
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
            "Enter" => {
                event.prevent_default();
                if let Some(cand) = self.highlighted_value() {
                    self.select_item(&cand);
                }
            }
            " " => {
                event.prevent_default();
                if self.has_active_typeahead(now_ms) {
                    self.handle_typeahead(' ', now_ms);
                } else if let Some(cand) = self.highlighted_value() {
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

pub fn scroll_item_within_viewport(viewport_id: &str, item_id: &str) {
    scroll_item_within_container(viewport_id, viewport_id, item_id);
}

async fn try_scroll_item_within_container(
    viewport_id: &str,
    fallback_content_id: &str,
    item_id: &str,
) -> bool {
    let doc = oxidase::dom::document();

    let has_viewport_geometry =
        crate::foundation::browser::get_element_client_rect(viewport_id).is_some();
    let container_id = if has_viewport_geometry {
        viewport_id
    } else {
        fallback_content_id
    };
    let Some(container_rect) = crate::foundation::browser::get_element_client_rect(container_id)
    else {
        return false;
    };
    let Some(item_rect) = crate::foundation::browser::get_element_client_rect(item_id) else {
        return false;
    };
    if item_rect.height <= 0.0 || container_rect.height <= 0.0 {
        return false;
    }

    const EDGE_INSET: f64 = 1.0;
    let visible_bottom = container_rect.bottom() - EDGE_INSET;
    let delta_y = if item_rect.top() < container_rect.top() {
        item_rect.top() - container_rect.top()
    } else if item_rect.bottom() > visible_bottom {
        item_rect.bottom() - visible_bottom
    } else {
        return true;
    };

    let Some(container_el) = doc.element_by_id(container_id) else {
        return false;
    };
    container_el.scroll_by(0.0, delta_y).await.is_ok()
}

pub(crate) fn scroll_item_within_container(
    viewport_id: &str,
    fallback_content_id: &str,
    item_id: &str,
) {
    let viewport_id = viewport_id.to_owned();
    let fallback_content_id = fallback_content_id.to_owned();
    let item_id = item_id.to_owned();
    crate::foundation::browser::spawn_root_task(async move {
        for attempt in 0..3 {
            if try_scroll_item_within_container(
                &viewport_id,
                &fallback_content_id,
                &item_id,
            )
            .await
            {
                break;
            }
            if attempt < 2 {
                let _ = oxidase::frame::next_frame().await;
            }
        }
    });
}

#[component]
pub fn SelectViewport(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<super::types::SelectContext>();
    let vp_id = id.unwrap_or_else(|| ctx.runtime.relationships().viewport_id());
    let default_style =
        "position: relative; flex: 1 1 auto; min-height: 0; overflow-x: hidden; overflow-y: auto;";
    let combined_style = match style {
        Some(s) => format!("{default_style} {s}"),
        None => default_style.to_string(),
    };
    rsx! {
        div {
            id: "{vp_id}",
            class: class.as_deref().unwrap_or_default(),
            style: "{combined_style}",
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


use std::collections::BTreeMap;

use super::types::{ComboboxItemData, ComboboxRegisteredItem};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComboboxOptionRegistry {
    items: Vec<ComboboxRegisteredItem>,
    label_cache: BTreeMap<String, String>,
    next_order: usize,
}

impl ComboboxOptionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_items(&mut self, items: &[ComboboxItemData]) {
        for item in items {
            self.label_cache
                .insert(item.value.clone(), item.label.clone());
        }
    }

    pub fn register(
        &mut self,
        value: impl Into<String>,
        label: impl Into<String>,
        dom_id: impl Into<String>,
        disabled: bool,
    ) {
        let value = value.into();
        let label = label.into();
        let dom_id = dom_id.into();

        self.label_cache.insert(value.clone(), label.clone());

        if let Some(existing) = self
            .items
            .iter_mut()
            .find(|item| item.dom_id == dom_id || item.value == value)
        {
            existing.value = value;
            existing.label = label;
            existing.dom_id = dom_id;
            existing.disabled = disabled;
            return;
        }

        let order = self.next_order;
        self.next_order = self.next_order.saturating_add(1);
        self.items.push(ComboboxRegisteredItem {
            value,
            label,
            dom_id,
            disabled,
            order,
        });
    }

    pub fn unregister_by_dom_id(&mut self, dom_id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|item| item.dom_id != dom_id);
        self.items.len() != before
    }

    pub fn unregister_by_value(&mut self, value: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|item| item.value != value);
        self.items.len() != before
    }

    pub fn reorder_by_dom_ids(&mut self, ordered_dom_ids: &[String]) {
        if ordered_dom_ids.is_empty() || self.items.len() <= 1 {
            return;
        }
        let mut reordered = Vec::with_capacity(self.items.len());
        for dom_id in ordered_dom_ids {
            if let Some(pos) = self.items.iter().position(|item| &item.dom_id == dom_id) {
                reordered.push(self.items.remove(pos));
            }
        }
        reordered.append(&mut self.items);
        for (idx, item) in reordered.iter_mut().enumerate() {
            item.order = idx;
        }
        self.next_order = reordered.len();
        self.items = reordered;
    }

    pub fn items(&self) -> &[ComboboxRegisteredItem] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn label_for(&self, value: &str) -> Option<&str> {
        if let Some(item) = self.items.iter().find(|item| item.value == value) {
            return Some(item.label.as_str());
        }
        self.label_cache.get(value).map(String::as_str)
    }

    pub fn dom_id_for(&self, value: &str) -> Option<&str> {
        self.items
            .iter()
            .find(|item| item.value == value)
            .map(|item| item.dom_id.as_str())
    }

    pub fn get(&self, value: &str) -> Option<&ComboboxRegisteredItem> {
        self.items.iter().find(|item| item.value == value)
    }

    pub fn is_disabled(&self, value: &str) -> bool {
        self.items
            .iter()
            .find(|item| item.value == value)
            .is_some_and(|item| item.disabled)
    }

    pub fn contains_enabled(&self, value: &str) -> bool {
        self.items
            .iter()
            .any(|item| item.value == value && !item.disabled)
    }

    pub fn enabled_items(&self) -> Vec<&ComboboxRegisteredItem> {
        self.items.iter().filter(|item| !item.disabled).collect()
    }

    pub fn first_enabled(&self) -> Option<&ComboboxRegisteredItem> {
        self.items.iter().find(|item| !item.disabled)
    }

    pub fn last_enabled(&self) -> Option<&ComboboxRegisteredItem> {
        self.items.iter().rev().find(|item| !item.disabled)
    }

    pub fn next_enabled(
        &self,
        current: Option<&str>,
        loop_focus: bool,
    ) -> Option<&ComboboxRegisteredItem> {
        let enabled = self.enabled_items();
        if enabled.is_empty() {
            return None;
        }
        let curr_idx = current.and_then(|val| enabled.iter().position(|item| item.value == val));
        let next_idx = match curr_idx {
            Some(idx) if idx + 1 < enabled.len() => idx + 1,
            Some(_) if loop_focus => 0,
            Some(idx) => idx,
            None => 0,
        };
        enabled.get(next_idx).copied()
    }

    pub fn prev_enabled(
        &self,
        current: Option<&str>,
        loop_focus: bool,
    ) -> Option<&ComboboxRegisteredItem> {
        let enabled = self.enabled_items();
        if enabled.is_empty() {
            return None;
        }
        let curr_idx = current.and_then(|val| enabled.iter().position(|item| item.value == val));
        let prev_idx = match curr_idx {
            Some(0) if loop_focus => enabled.len() - 1,
            Some(0) => 0,
            Some(idx) => idx - 1,
            None => enabled.len() - 1,
        };
        enabled.get(prev_idx).copied()
    }

    pub fn step_enabled(
        &self,
        current: Option<&str>,
        delta: isize,
        loop_focus: bool,
    ) -> Option<&ComboboxRegisteredItem> {
        let enabled = self.enabled_items();
        if enabled.is_empty() {
            return None;
        }
        let last_idx = enabled.len() - 1;
        let curr_idx = current.and_then(|val| enabled.iter().position(|item| item.value == val));
        let next_idx = match curr_idx {
            Some(idx) => {
                if loop_focus && delta > 0 && idx == last_idx {
                    0
                } else if loop_focus && delta < 0 && idx == 0 {
                    last_idx
                } else {
                    let target = idx as isize + delta;
                    if target <= 0 {
                        0
                    } else if target as usize >= last_idx {
                        last_idx
                    } else {
                        target as usize
                    }
                }
            }
            None => {
                if delta >= 0 {
                    0
                } else {
                    last_idx
                }
            }
        };
        enabled.get(next_idx).copied()
    }

    pub fn ordered_dom_ids(&self) -> Vec<&str> {
        self.items.iter().map(|item| item.dom_id.as_str()).collect()
    }
}

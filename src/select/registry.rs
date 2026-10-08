use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectItemRegistration {
    pub id: String,
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectItemRegistration {
    pub fn new(
        id: impl Into<String>,
        value: impl Into<String>,
        label: impl Into<String>,
        disabled: bool,
    ) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            label: label.into(),
            disabled,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MountedOptionRegistry {
    pub items_by_id: HashMap<String, SelectItemRegistration>,
    pub ordered_item_ids: Vec<String>,
}

impl MountedOptionRegistry {
    pub fn new() -> Self {
        Self {
            items_by_id: HashMap::new(),
            ordered_item_ids: Vec::new(),
        }
    }

    pub fn register(&mut self, item: SelectItemRegistration) {
        if !self.items_by_id.contains_key(&item.id)
            && !self.ordered_item_ids.iter().any(|id| id == &item.id)
        {
            self.ordered_item_ids.push(item.id.clone());
        }
        self.items_by_id.insert(item.id.clone(), item);
    }

    pub fn unregister(&mut self, id: &str) {
        self.items_by_id.remove(id);
        self.ordered_item_ids.retain(|item_id| item_id != id);
    }

    pub fn set_tree_order(&mut self, ordered_ids: Vec<String>) {
        let mut new_order = Vec::with_capacity(ordered_ids.len().max(self.ordered_item_ids.len()));
        let mut seen = HashSet::new();
        for id in ordered_ids {
            if seen.insert(id.clone()) {
                new_order.push(id);
            }
        }
        for id in &self.ordered_item_ids {
            if seen.insert(id.clone()) {
                new_order.push(id.clone());
            }
        }
        self.ordered_item_ids = new_order;
    }

    pub fn active_items(&self) -> impl Iterator<Item = &SelectItemRegistration> {
        self.ordered_item_ids
            .iter()
            .filter_map(|id| self.items_by_id.get(id))
    }

    pub fn get(&self, id: &str) -> Option<&SelectItemRegistration> {
        self.items_by_id.get(id)
    }

    pub fn get_by_value(&self, value: &str) -> Option<&SelectItemRegistration> {
        self.items_by_id.values().find(|item| item.value == value)
    }

    pub fn len(&self) -> usize {
        self.items_by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items_by_id.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_registration_and_mount_order() {
        let mut registry = MountedOptionRegistry::new();
        registry.register(SelectItemRegistration::new("opt-1", "1", "One", false));
        registry.register(SelectItemRegistration::new("opt-2", "2", "Two", false));
        registry.register(SelectItemRegistration::new("opt-3", "3", "Three", true));

        let active: Vec<_> = registry.active_items().map(|i| i.id.as_str()).collect();
        assert_eq!(active, vec!["opt-1", "opt-2", "opt-3"]);
        assert_eq!(registry.len(), 3);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_registry_unmount_removal() {
        let mut registry = MountedOptionRegistry::new();
        registry.register(SelectItemRegistration::new("opt-1", "1", "One", false));
        registry.register(SelectItemRegistration::new("opt-2", "2", "Two", false));

        registry.unregister("opt-1");
        let active: Vec<_> = registry.active_items().map(|i| i.id.as_str()).collect();
        assert_eq!(active, vec!["opt-2"]);
        assert_eq!(registry.len(), 1);
        assert!(registry.get("opt-1").is_none());
        assert!(registry.get("opt-2").is_some());
    }

    #[test]
    fn test_registry_update_preserves_order() {
        let mut registry = MountedOptionRegistry::new();
        registry.register(SelectItemRegistration::new("opt-1", "1", "One", false));
        registry.register(SelectItemRegistration::new("opt-2", "2", "Two", false));

        // Update opt-1 label and disabled
        registry.register(SelectItemRegistration::new(
            "opt-1",
            "1",
            "One Updated",
            true,
        ));

        let active: Vec<_> = registry.active_items().collect();
        assert_eq!(active[0].id, "opt-1");
        assert_eq!(active[0].label, "One Updated");
        assert!(active[0].disabled);
        assert_eq!(active[1].id, "opt-2");
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_registry_set_tree_order() {
        let mut registry = MountedOptionRegistry::new();
        registry.register(SelectItemRegistration::new("opt-a", "a", "A", false));
        registry.register(SelectItemRegistration::new("opt-b", "b", "B", false));
        registry.register(SelectItemRegistration::new("opt-c", "c", "C", false));

        // Reorder: B, C, A
        registry.set_tree_order(vec![
            "opt-b".to_string(),
            "opt-c".to_string(),
            "opt-a".to_string(),
        ]);

        let active: Vec<_> = registry.active_items().map(|i| i.id.as_str()).collect();
        assert_eq!(active, vec!["opt-b", "opt-c", "opt-a"]);
    }
}

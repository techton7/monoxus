use dioxus::prelude::*;

use crate::foundation::{
    roving_focus::{
        RovingFocusConfig, RovingFocusRuntime, RovingItemMeta, RovingModifiers,
        use_roving_focus_group,
    },
    shared::{CollectionRegistry, Direction, ScopeHandle},
};

use super::{
    attrs::{ToggleGroupItemAttributes, ToggleGroupRootAttributes},
    state::ToggleGroup,
    types::{
        ToggleGroupItemRegistration, ToggleGroupMode, ToggleGroupMultipleChangeHandler,
        ToggleGroupOrientation, ToggleGroupSingleChangeHandler,
    },
};

#[derive(Clone)]
pub struct ToggleGroupRuntime {
    pub mode: Signal<ToggleGroupMode>,
    pub single_value: Signal<String>,
    pub multiple_values: Signal<Vec<String>>,
    pub single_controlled: Signal<bool>,
    pub multiple_controlled: Signal<bool>,
    pub roving: RovingFocusRuntime,
    pub current_tab_stop: Signal<Option<String>>,
    pub registry: Signal<CollectionRegistry<String, RovingItemMeta>>,
    pub orientation: Signal<ToggleGroupOrientation>,
    pub dir: Signal<Direction>,
    pub roving_focus: Signal<bool>,
    pub loop_focus: Signal<bool>,
    pub disabled: Signal<bool>,
    pub root_id: Signal<String>,
    pub root_dom_id: Signal<String>,
    pub on_single_change: Option<ToggleGroupSingleChangeHandler>,
    pub on_multiple_change: Option<ToggleGroupMultipleChangeHandler>,
}

impl ToggleGroupRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: ToggleGroupMode,
        initial_single: String,
        initial_multiple: Vec<String>,
        single_controlled: bool,
        multiple_controlled: bool,
        orientation: ToggleGroupOrientation,
        dir: Direction,
        roving_focus: bool,
        loop_focus: bool,
        disabled: bool,
        root_id: String,
        root_dom_id: String,
        on_single_change: Option<ToggleGroupSingleChangeHandler>,
        on_multiple_change: Option<ToggleGroupMultipleChangeHandler>,
    ) -> Self {
        let roving_cfg = RovingFocusConfig::new()
            .with_orientation(orientation.into())
            .with_direction(dir)
            .with_loop_focus(loop_focus)
            .with_enabled(roving_focus);
        let roving = use_roving_focus_group(roving_cfg, None, root_dom_id);

        Self {
            mode: use_signal(|| mode),
            single_value: use_signal(|| initial_single),
            multiple_values: use_signal(|| initial_multiple),
            single_controlled: use_signal(|| single_controlled),
            multiple_controlled: use_signal(|| multiple_controlled),
            roving,
            current_tab_stop: roving.current_tab_stop,
            registry: roving.registry,
            orientation: use_signal(|| orientation),
            dir: use_signal(|| dir),
            roving_focus: use_signal(|| roving_focus),
            loop_focus: use_signal(|| loop_focus),
            disabled: use_signal(|| disabled),
            root_id: use_signal(|| root_id),
            root_dom_id: roving.root_dom_id,
            on_single_change,
            on_multiple_change,
        }
    }

    pub fn roving(&self) -> RovingFocusRuntime {
        self.roving
    }

    pub fn state(&self) -> ToggleGroup {
        let scope = ScopeHandle::root("toggle-group").child((self.root_id)());
        ToggleGroup::new(scope, (self.mode)())
            .with_orientation((self.orientation)())
            .with_direction((self.dir)())
            .with_roving_focus((self.roving_focus)())
            .with_loop_focus((self.loop_focus)())
            .with_disabled((self.disabled)())
            .with_single_value((self.single_value)())
            .with_multiple_values((self.multiple_values)())
            .with_current_tab_stop((self.roving.current_tab_stop)())
    }

    pub fn is_item_pressed(&self, value: &str) -> bool {
        match (self.mode)() {
            ToggleGroupMode::Single => {
                let current = (self.single_value)();
                !current.is_empty() && current == value
            }
            ToggleGroupMode::Multiple => (self.multiple_values)().iter().any(|v| v == value),
        }
    }

    pub fn is_effective_disabled(&self, item_disabled: bool) -> bool {
        (self.disabled)() || item_disabled
    }

    pub fn register_item(&self, item: ToggleGroupItemRegistration) {
        let focusable = !self.is_effective_disabled(item.disabled);
        let active = self.is_item_pressed(&item.value);
        self.roving
            .register_item(item.value, RovingItemMeta::new(item.id, focusable, active));
    }

    pub fn unregister_item(&self, value: &str) {
        self.roving.unregister_item(value);
    }

    pub fn reorder_items(&self, ordered_keys: &[String]) {
        self.roving.reorder_items(ordered_keys);
    }

    pub fn sync_dom_order(&self, root_element_id: &str) {
        self.roving.sync_dom_order_for(root_element_id);
    }

    pub fn resolve_tab_stop(&self) -> Option<String> {
        if (self.disabled)() {
            return None;
        }
        self.roving
            .resolve_tab_stop_with(|key, _meta| self.is_item_pressed(key))
    }

    pub fn set_current_tab_stop(&self, tab_stop: Option<String>) {
        self.roving.set_current_tab_stop(tab_stop);
    }

    pub fn toggle_item(&self, value: &str) {
        match (self.mode)() {
            ToggleGroupMode::Single => {
                let current = (self.single_value)();
                let next = if current == value {
                    String::new()
                } else {
                    value.to_string()
                };
                if !*(self.single_controlled).peek() {
                    let mut sig = self.single_value;
                    sig.set(next.clone());
                }
                self.roving.set_current_tab_stop(Some(value.to_string()));
                if let Some(ref handler) = self.on_single_change {
                    handler(next);
                }
            }
            ToggleGroupMode::Multiple => {
                let mut list = (self.multiple_values)();
                if let Some(pos) = list.iter().position(|v| v == value) {
                    list.remove(pos);
                } else {
                    list.push(value.to_string());
                }
                if !*(self.multiple_controlled).peek() {
                    let mut sig = self.multiple_values;
                    sig.set(list.clone());
                }
                self.roving.set_current_tab_stop(Some(value.to_string()));
                if let Some(ref handler) = self.on_multiple_change {
                    handler(list);
                }
            }
        }
    }

    pub fn focus_item(&self, value: &str) {
        let _ = self.roving.focus_item(value);
    }

    pub fn navigate_key(
        &self,
        current_value: &str,
        key: &str,
        alt_key: bool,
        ctrl_key: bool,
        meta_key: bool,
        shift_key: bool,
    ) -> Option<String> {
        self.roving.navigate_key(
            Some(current_value),
            key,
            RovingModifiers::new(alt_key, ctrl_key, meta_key, shift_key),
        )
    }

    pub fn root_attributes(&self, custom_role: Option<&'static str>) -> ToggleGroupRootAttributes {
        self.state().root_attributes(custom_role)
    }

    pub fn item_attributes(
        &self,
        value: &str,
        item_id: Option<String>,
        item_disabled: bool,
    ) -> ToggleGroupItemAttributes {
        let resolved = self.resolve_tab_stop();
        self.state()
            .item_attributes(value, item_id, item_disabled, resolved.as_deref())
    }
}

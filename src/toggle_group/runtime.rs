use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::foundation::{
    roving_focus::{
        RovingFocusConfig, RovingFocusRuntime, RovingItemMeta, RovingModifiers,
        use_roving_focus_group,
    },
    shared::{CollectionRegistry, Direction, ScopeHandle},
    state::{
        ChangeOutcome, ControllableConfig, ControllableSignal, sync_signal,
        use_controllable_signal_config,
    },
};

use super::{
    attrs::{ToggleGroupItemAttributes, ToggleGroupRootAttributes},
    state::ToggleGroup,
    types::{ToggleGroupItemRegistration, ToggleGroupMode, ToggleGroupOrientation},
};

#[derive(Clone)]
pub struct ToggleGroupRuntime {
    pub mode: Signal<ToggleGroupMode>,
    pub single_value: Signal<String>,
    pub multiple_values: Signal<Vec<String>>,
    pub(crate) single_ctl: ControllableSignal<String>,
    pub(crate) multiple_ctl: ControllableSignal<Vec<String>>,
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
    pub invalid: Signal<bool>,
    pub root_id: Signal<String>,
    pub root_dom_id: Signal<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn use_toggle_group_runtime(
    mode: ToggleGroupMode,
    value: Option<String>,
    default_value: Option<String>,
    on_value_change: Option<EventHandler<String>>,
    values: Option<Vec<String>>,
    default_values: Option<Vec<String>>,
    on_values_change: Option<EventHandler<Vec<String>>>,
    orientation: ToggleGroupOrientation,
    dir: Direction,
    roving_focus: bool,
    loop_focus: bool,
    disabled: bool,
    root_id: String,
    root_dom_id: String,
) -> ToggleGroupRuntime {
    use_toggle_group_runtime_with_details(
        mode,
        value,
        default_value,
        on_value_change,
        None,
        values,
        default_values,
        on_values_change,
        None,
        orientation,
        dir,
        roving_focus,
        loop_focus,
        disabled,
        root_id,
        root_dom_id,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn use_toggle_group_runtime_with_details(
    mode: ToggleGroupMode,
    value: Option<String>,
    default_value: Option<String>,
    on_value_change: Option<EventHandler<String>>,
    on_value_change_details: Option<EventHandler<ChangeEvent<String>>>,
    values: Option<Vec<String>>,
    default_values: Option<Vec<String>>,
    on_values_change: Option<EventHandler<Vec<String>>>,
    on_values_change_details: Option<EventHandler<ChangeEvent<Vec<String>>>>,
    orientation: ToggleGroupOrientation,
    dir: Direction,
    roving_focus: bool,
    loop_focus: bool,
    disabled: bool,
    root_id: String,
    root_dom_id: String,
) -> ToggleGroupRuntime {
    let has_explicit_default_value = default_value.is_some();
    let has_explicit_default_values = default_values.is_some();
    let single_ctl = use_controllable_signal_config(
        ControllableConfig::new(value, default_value.unwrap_or_default())
            .with_explicit_default(has_explicit_default_value)
            .with_on_change(on_value_change)
            .with_on_change_details(on_value_change_details)
            .with_component("ToggleGroupRoot")
            .with_prop("value"),
    );
    let multiple_ctl = use_controllable_signal_config(
        ControllableConfig::new(values, default_values.unwrap_or_default())
            .with_explicit_default(has_explicit_default_values)
            .with_on_change(on_values_change)
            .with_on_change_details(on_values_change_details)
            .with_component("ToggleGroupRoot")
            .with_prop("values"),
    );

    let roving_cfg = RovingFocusConfig::new()
        .with_orientation(orientation.into())
        .with_direction(dir)
        .with_loop_focus(loop_focus)
        .with_enabled(roving_focus);
    let roving = use_roving_focus_group(roving_cfg, None, root_dom_id);

    let mut rt = ToggleGroupRuntime {
        mode: use_signal(|| mode),
        single_value: single_ctl.signal(),
        multiple_values: multiple_ctl.signal(),
        single_ctl,
        multiple_ctl,
        single_controlled: use_signal(|| single_ctl.is_controlled()),
        multiple_controlled: use_signal(|| multiple_ctl.is_controlled()),
        roving,
        current_tab_stop: roving.current_tab_stop,
        registry: roving.registry,
        orientation: use_signal(|| orientation),
        dir: use_signal(|| dir),
        roving_focus: use_signal(|| roving_focus),
        loop_focus: use_signal(|| loop_focus),
        disabled: use_signal(|| disabled),
        invalid: use_signal(|| false),
        root_id: use_signal(|| root_id),
        root_dom_id: roving.root_dom_id,
    };

    sync_signal(&mut rt.mode, mode);
    sync_signal(&mut rt.single_controlled, single_ctl.is_controlled());
    sync_signal(&mut rt.multiple_controlled, multiple_ctl.is_controlled());
    sync_signal(&mut rt.orientation, orientation);
    sync_signal(&mut rt.dir, dir);
    sync_signal(&mut rt.roving_focus, roving_focus);
    sync_signal(&mut rt.loop_focus, loop_focus);
    sync_signal(&mut rt.disabled, disabled);

    rt
}

impl ToggleGroupRuntime {
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

    pub fn is_invalid(&self) -> bool {
        (self.invalid)()
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
        let _ = self.toggle_item_with_context(
            value,
            ChangeContext::new(ChangeReason::Toggle, ChangeSource::Programmatic),
        );
    }

    pub fn toggle_item_with_context(&self, value: &str, ctx: ChangeContext) -> ChangeOutcome {
        let outcome = match (self.mode)() {
            ToggleGroupMode::Single => {
                let current = self.single_ctl.peek();
                let next = if current == value {
                    String::new()
                } else {
                    value.to_string()
                };
                self.single_ctl.set_with_context(next, ctx)
            }
            ToggleGroupMode::Multiple => {
                let mut list = self.multiple_ctl.peek();
                if let Some(pos) = list.iter().position(|v| v == value) {
                    list.remove(pos);
                } else {
                    list.push(value.to_string());
                }
                self.multiple_ctl.set_with_context(list, ctx)
            }
        };
        if outcome.is_accepted() {
            self.roving.set_current_tab_stop(Some(value.to_string()));
        }
        outcome
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

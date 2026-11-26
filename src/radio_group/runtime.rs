use dioxus::prelude::*;

use crate::foundation::{
    roving_focus::{
        Orientation, RovingFocusConfig, RovingFocusRuntime, use_roving_focus_group,
    },
    shared::{Direction, ScopeHandle},
    state::{ControllableSignal, sync_signal},
};

use super::{
    attrs::{RadioGroupIndicatorAttributes, RadioGroupItemAttributes, RadioGroupRootAttributes},
    state::RadioGroup,
    types::RadioGroupOrientation,
};

#[derive(Clone)]
pub struct RadioGroupRuntime {
    pub value: Signal<Option<String>>,
    pub(crate) value_ctl: ControllableSignal<Option<String>>,
    pub is_controlled: Signal<bool>,
    pub initial_value: Signal<Option<String>>,
    pub orientation: Signal<Option<RadioGroupOrientation>>,
    pub dir: Signal<Direction>,
    pub loop_focus: Signal<bool>,
    pub disabled: Signal<bool>,
    pub required: Signal<bool>,
    pub read_only: Signal<bool>,
    pub name: Signal<Option<String>>,
    pub form: Signal<Option<String>>,
    pub autocomplete: Signal<Option<String>>,
    pub root_id: Signal<String>,
    pub root_dom_id: Signal<String>,
    pub should_render_hidden_inputs: Signal<bool>,
    pub reset_epoch: Signal<u64>,
    pub roving: RovingFocusRuntime,
    pub on_value_change: CopyValue<Option<EventHandler<String>>>,
}

#[allow(clippy::too_many_arguments)]
pub fn use_radio_group_runtime(
    value_ctl: ControllableSignal<Option<String>>,
    initial_value: Option<String>,
    orientation: Option<RadioGroupOrientation>,
    dir: Direction,
    loop_focus: bool,
    disabled: bool,
    required: bool,
    read_only: bool,
    name: Option<String>,
    form: Option<String>,
    autocomplete: Option<String>,
    root_id: String,
    root_dom_id: String,
    should_render_hidden_inputs: bool,
    on_value_change: Option<EventHandler<String>>,
) -> RadioGroupRuntime {
    let mapped_orientation = match orientation {
        None => Orientation::Both,
        Some(RadioGroupOrientation::Horizontal) => Orientation::Horizontal,
        Some(RadioGroupOrientation::Vertical) => Orientation::Vertical,
    };

    let roving_cfg = RovingFocusConfig::new()
        .with_orientation(mapped_orientation)
        .with_direction(dir)
        .with_loop_focus(loop_focus)
        .with_enabled(!disabled);
    let roving = use_roving_focus_group(roving_cfg, None, root_dom_id);
    let mut on_value_change_slot = use_hook(|| CopyValue::new(on_value_change));
    on_value_change_slot.set(on_value_change);

    let is_controlled = value_ctl.is_controlled();
    let mut rt = RadioGroupRuntime {
        value: value_ctl.signal(),
        value_ctl,
        is_controlled: use_signal(|| is_controlled),
        initial_value: use_signal(|| initial_value),
        orientation: use_signal(|| orientation),
        dir: use_signal(|| dir),
        loop_focus: use_signal(|| loop_focus),
        disabled: use_signal(|| disabled),
        required: use_signal(|| required),
        read_only: use_signal(|| read_only),
        name: use_signal(|| name.clone()),
        form: use_signal(|| form.clone()),
        autocomplete: use_signal(|| autocomplete.clone()),
        root_id: use_signal(|| root_id.clone()),
        root_dom_id: roving.root_dom_id,
        should_render_hidden_inputs: use_signal(|| should_render_hidden_inputs),
        reset_epoch: use_signal(|| 0_u64),
        roving,
        on_value_change: on_value_change_slot,
    };

    sync_signal(&mut rt.root_id, root_id);
    sync_signal(&mut rt.is_controlled, is_controlled);
    sync_signal(&mut rt.should_render_hidden_inputs, should_render_hidden_inputs);
    sync_signal(&mut rt.orientation, orientation);
    sync_signal(&mut rt.dir, dir);
    sync_signal(&mut rt.loop_focus, loop_focus);
    sync_signal(&mut rt.disabled, disabled);
    sync_signal(&mut rt.required, required);
    sync_signal(&mut rt.read_only, read_only);
    sync_signal(&mut rt.name, name);
    sync_signal(&mut rt.form, form);
    sync_signal(&mut rt.autocomplete, autocomplete);

    rt
}

impl RadioGroupRuntime {
    pub fn roving(&self) -> RovingFocusRuntime {
        self.roving
    }

    pub fn state(&self) -> RadioGroup {
        let scope = ScopeHandle::root("radio-group").child((self.root_id)());
        RadioGroup::new(scope)
            .with_orientation((self.orientation)())
            .with_direction((self.dir)())
            .with_loop_focus((self.loop_focus)())
            .with_disabled((self.disabled)())
            .with_required((self.required)())
            .with_read_only((self.read_only)())
            .with_value((self.value)())
            .with_current_tab_stop((self.roving.current_tab_stop)())
    }

    pub fn is_item_checked(&self, value: &str) -> bool {
        (self.value)().as_deref() == Some(value)
    }

    pub fn is_item_disabled(&self, item_disabled: bool) -> bool {
        (self.disabled)() || item_disabled
    }

    pub fn is_read_only(&self) -> bool {
        (self.read_only)()
    }

    /// Selects an item if enabled and not read-only.
    /// Irreversible: checking an already-checked item is a no-op (RG-2).
    pub fn check_item(&self, value: &str) {
        if (self.disabled)() || (self.read_only)() {
            return;
        }

        let current = self.value.peek().clone();
        if current.as_deref() == Some(value) {
            return;
        }

        let next = Some(value.to_string());
        self.value_ctl.set(next.clone());
        self.roving.set_current_tab_stop(next);

        if let Some(handler) = self.on_value_change.try_read().ok().and_then(|s| *s) {
            handler.call(value.to_string());
        }
    }

    pub(crate) fn reset_to(&self, initial: Option<String>) {
        if self.value_ctl.set(initial.clone()) {
            if !self.value_ctl.is_controlled() {
                self.roving.set_current_tab_stop(initial.clone());
            }
            if let Some(val) = initial
                && !val.is_empty()
                && let Some(handler) = self.on_value_change.try_read().ok().and_then(|s| *s)
            {
                handler.call(val);
            }
        }
    }

    pub fn focus_item(&self, value: &str) {
        let _ = self.roving.focus_item(value);
    }

    pub fn resolve_tab_stop(&self) -> Option<String> {
        if (self.disabled)() {
            return None;
        }
        self.roving
            .resolve_tab_stop_with(|key, _meta| self.is_item_checked(key))
    }

    pub fn set_current_tab_stop(&self, tab_stop: Option<String>) {
        self.roving.set_current_tab_stop(tab_stop);
    }

    pub fn root_attributes(&self, custom_role: Option<&'static str>) -> RadioGroupRootAttributes {
        self.state().root_attributes(custom_role)
    }

    pub fn item_attributes(
        &self,
        value: &str,
        item_id: Option<String>,
        item_disabled: bool,
    ) -> RadioGroupItemAttributes {
        let resolved = self.resolve_tab_stop();
        self.state()
            .item_attributes(value, item_id, item_disabled, resolved.as_deref())
    }

    pub fn indicator_attributes(
        &self,
        value: &str,
        indicator_id: Option<String>,
        item_disabled: bool,
    ) -> RadioGroupIndicatorAttributes {
        self.state()
            .indicator_attributes(value, indicator_id, item_disabled)
    }
}

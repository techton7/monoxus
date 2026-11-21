use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::button::ButtonType;
use crate::foundation::{
    browser::{start_presence_monitor, PresenceMonitorEvent, WatcherGuard},
    form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
    overlay::PresenceController,
    shared::{ScopeHandle, use_stable_id},
    state::{use_controllable_state, ControllableStateProps},
};

use super::{
    attrs::{
        CheckboxGroupAttributes, CheckboxGroupLabelAttributes, CheckboxIndicatorAttributes,
        CheckboxRootAttributes,
    },
    relationships::{CheckboxGroupRelationships, CheckboxRelationships},
    runtime::{
        try_use_checkbox_group_context, use_checkbox_context,
        use_checkbox_group_context, CheckboxContext, CheckboxGroupContext,
    },
    types::{
        CheckboxCheckedChangeHandler, CheckboxGroupValuesChangeHandler, CheckedState,
    },
};

/// Props for [`CheckboxRoot`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub checked: Option<CheckedState>,
    #[props(default, into)]
    pub default_checked: CheckedState,
    #[props(default)]
    pub on_checked_change: Option<EventHandler<CheckedState>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub name: Option<String>,
    #[props(default = String::from("on"))]
    pub value: String,
    #[props(default)]
    pub form: Option<String>,
    #[props(default)]
    pub r#type: ButtonType,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Headless interactive `<button role="checkbox">` trigger element.
#[component]
pub fn CheckboxRoot(props: CheckboxRootProps) -> Element {
    let group_ctx = try_use_checkbox_group_context();

    let effective_name = props
        .name
        .clone()
        .or_else(|| group_ctx.as_ref().and_then(|g| g.reactive_name()));
    let effective_form = props
        .form
        .clone()
        .or_else(|| group_ctx.as_ref().and_then(|g| g.reactive_form()));
    let effective_disabled =
        props.disabled || group_ctx.as_ref().is_some_and(|g| g.is_disabled());
    let effective_required =
        props.required || group_ctx.as_ref().is_some_and(|g| g.is_required());
    let effective_read_only =
        props.read_only || group_ctx.as_ref().is_some_and(|g| g.is_read_only());

    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let relationships = {
        let scope = ScopeHandle::root("checkbox").child(stable_id);
        if let Some(ref explicit_id) = props.id {
            CheckboxRelationships::with_explicit_root_id(scope, explicit_id)
        } else {
            CheckboxRelationships::new(scope)
        }
    };

    let root_id = relationships.root_id().to_string();
    let indicator_id = relationships.indicator_id().to_string();

    let is_in_group = group_ctx.is_some() && props.checked.is_none();
    let item_val_for_group = props.value.clone();

    let on_checked_change_cb = props.on_checked_change.as_ref().map(|handler| {
        let handler = *handler;
        Rc::new(move |next: CheckedState| {
            handler.call(next);
        }) as CheckboxCheckedChangeHandler
    });

    let standalone_handle = use_controllable_state(ControllableStateProps {
        value: props.checked,
        default_value: props.default_checked,
        on_change: on_checked_change_cb.clone().map(|cb| {
            move |v: CheckedState| cb(v)
        }),
    });

    let current_checked = if is_in_group {
        let g = group_ctx.as_ref().unwrap();
        if g.is_item_checked(&item_val_for_group) {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        }
    } else {
        *standalone_handle.value()
    };

    let initial_checked_for_reset = props.default_checked;
    let reset_standalone_handle = standalone_handle.clone();
    let on_reset_fn = {
        let group_ctx_opt = group_ctx.clone();
        move || {
            if group_ctx_opt.is_none() {
                reset_standalone_handle.set(initial_checked_for_reset);
            }
        }
    };

    let (should_render_input, local_reset_epoch) = use_form_control_sync_with_epoch(
        &root_id,
        effective_name.as_deref(),
        effective_form.as_deref(),
        on_reset_fn,
    );
    let effective_reset_epoch =
        local_reset_epoch.wrapping_add(group_ctx.as_ref().map_or(0, |g| g.reset_epoch()));

    let mut checked_sig = use_signal(|| current_checked);
    if *checked_sig.peek() != current_checked {
        checked_sig.set(current_checked);
    }
    let mut disabled_sig = use_signal(|| effective_disabled);
    if *disabled_sig.peek() != effective_disabled {
        disabled_sig.set(effective_disabled);
    }
    let mut required_sig = use_signal(|| effective_required);
    if *required_sig.peek() != effective_required {
        required_sig.set(effective_required);
    }
    let mut read_only_sig = use_signal(|| effective_read_only);
    if *read_only_sig.peek() != effective_read_only {
        read_only_sig.set(effective_read_only);
    }

    let context = CheckboxContext {
        root_id: root_id.clone(),
        indicator_id: indicator_id.clone(),
        checked: checked_sig,
        disabled: disabled_sig,
        required: required_sig,
        read_only: read_only_sig,
    };
    use_context_provider(|| context);

    let on_click = {
        let on_click = props.on_click.clone();
        let disabled = effective_disabled;
        let read_only = effective_read_only;
        let standalone_handle = standalone_handle.clone();
        let group_ctx = group_ctx.clone();
        let item_value = props.value.clone();
        let change_cb = on_checked_change_cb.clone();
        move |evt: MouseEvent| {
            if let Some(ref handler) = on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() && !disabled && !read_only {
                if let Some(ref g) = group_ctx {
                    let next = if g.is_item_checked(&item_value) {
                        CheckedState::Unchecked
                    } else {
                        CheckedState::Checked
                    };
                    g.toggle_item(item_value.clone());
                    if let Some(ref cb) = change_cb {
                        cb(next);
                    }
                } else {
                    let current = *checked_sig.peek();
                    standalone_handle.set(current.toggle());
                }
            }
        }
    };

    let on_key_down = {
        let on_key_down = props.on_key_down.clone();
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = on_key_down {
                handler.call(evt.clone());
            }
            if evt.key() == Key::Named(NamedKey::Enter) {
                // Per WAI-ARIA 1.2 Checkbox Pattern, Enter must NOT toggle checkbox.
                evt.prevent_default();
            }
            // Space is left uncancelled so native <button> synthesizes Click on KeyUp with zero double-fire.
        }
    };

    let on_mounted = {
        let on_mounted = props.on_mounted.clone();
        move |evt: MountedEvent| {
            if let Some(ref handler) = on_mounted {
                handler.call(evt);
            }
        }
    };

    let attrs = CheckboxRootAttributes::new(
        Some(root_id.clone()),
        props.r#type.as_str(),
        current_checked,
        effective_disabled,
        effective_required,
        effective_read_only,
    );

    rsx! {
        button {
            id: attrs.id.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            r#type: attrs.r#type,
            role: attrs.role,
            "aria-checked": attrs.aria_checked,
            "aria-required": attrs.aria_required,
            "aria-readonly": attrs.aria_readonly,
            "data-state": attrs.data_state_str(),
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            "data-readonly": attrs.data_readonly,
            "data-checkbox-root": attrs.data_checkbox_root,
            disabled: attrs.disabled,
            value: "{props.value}",
            onclick: on_click,
            onkeydown: on_key_down,
            onmounted: on_mounted,
            ..props.attributes,
            {props.children}
        }
        if should_render_input {
            HiddenFormInput {
                kind: HiddenInputKind::Checkbox,
                name: effective_name,
                value: props.value.clone(),
                checked: current_checked.is_checked(),
                disabled: effective_disabled,
                required: effective_required,
                read_only: effective_read_only,
                form: effective_form,
                focus_target_id: root_id.clone(),
                reset_epoch: effective_reset_epoch,
            }
        }
    }
}

pub use CheckboxRoot as Checkbox;

/// Props for [`CheckboxIndicator`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxIndicatorProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Conditional indicator rendered inside [`CheckboxRoot`], retaining mount across CSS exit animations.
#[component]
pub fn CheckboxIndicator(props: CheckboxIndicatorProps) -> Element {
    let ctx = use_checkbox_context();
    let state = ctx.checked_state();
    let desired_present = props.force_mount || state.is_checked() || state.is_indeterminate();

    let mut presence = use_signal(|| PresenceController::new(desired_present).with_retained_mount(true));
    let watcher_slot: Rc<RefCell<Option<WatcherGuard>>> = use_hook(|| Rc::new(RefCell::new(None)));

    let indicator_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.indicator_id().to_string());

    use_effect(use_reactive((&desired_present,), {
        let indicator_id = indicator_id.clone();
        let watcher_slot = watcher_slot.clone();
        move |(present,)| {
            let update = presence.write().sync(present);
            if let Some(cycle_id) = update.started_close_cycle() {
                let watcher = start_presence_monitor(
                    indicator_id.as_str(),
                    cycle_id,
                    move |event| match event {
                        PresenceMonitorEvent::TransitionEnd { cycle_id: c, .. }
                        | PresenceMonitorEvent::TransitionCancel { cycle_id: c, .. }
                        | PresenceMonitorEvent::AnimationEnd { cycle_id: c, .. }
                        | PresenceMonitorEvent::AnimationCancel { cycle_id: c, .. }
                        | PresenceMonitorEvent::Fallback { cycle_id: c, .. } => {
                            presence.write().complete_close_cycle(c);
                        }
                        PresenceMonitorEvent::Stopped { .. } => {}
                    },
                );
                *watcher_slot.borrow_mut() = Some(watcher);
            } else if !update.should_render() {
                *watcher_slot.borrow_mut() = None;
            }
        }
    }));

    if !presence.read().should_render() {
        return rsx! {};
    }

    let on_mounted = {
        let on_mounted = props.on_mounted.clone();
        move |evt: MountedEvent| {
            if let Some(ref handler) = on_mounted {
                handler.call(evt);
            }
        }
    };

    let attrs = CheckboxIndicatorAttributes::new(
        Some(indicator_id.clone()),
        state,
        ctx.is_disabled(),
        ctx.is_read_only(),
        props.style.as_deref(),
    );

    rsx! {
        span {
            id: attrs.id.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: "{attrs.style}",
            "data-state": attrs.data_state_str(),
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            "data-checkbox-indicator": attrs.data_checkbox_indicator,
            onmounted: on_mounted,
            ..props.attributes,
            {props.children}
        }
    }
}

/// Props for [`CheckboxGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxGroupProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub values: Option<Vec<String>>,
    #[props(default)]
    pub default_values: Vec<String>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Headless `<div role="group">` container managing multi-value checkbox membership and cascading attributes.
#[component]
pub fn CheckboxGroup(props: CheckboxGroupProps) -> Element {
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let relationships = {
        let scope = ScopeHandle::root("checkbox-group").child(stable_id);
        if let Some(ref explicit_id) = props.id {
            CheckboxGroupRelationships::with_explicit_group_id(scope, explicit_id)
        } else {
            CheckboxGroupRelationships::new(scope)
        }
    };

    let group_id = relationships.group_id().to_string();

    let on_values_change_cb = props.on_values_change.as_ref().map(|handler| {
        let handler = *handler;
        Rc::new(move |next: Vec<String>| {
            handler.call(next);
        }) as CheckboxGroupValuesChangeHandler
    });

    let values_handle = use_controllable_state(ControllableStateProps {
        value: props.values.clone(),
        default_value: props.default_values.clone(),
        on_change: on_values_change_cb.clone().map(|cb| {
            move |v: Vec<String>| cb(v)
        }),
    });

    let mut values_sig = use_signal(|| values_handle.value().clone());
    if *values_sig.peek() != *values_handle.value() {
        values_sig.set(values_handle.value().clone());
    }
    let mut name_sig = use_signal(|| props.name.clone());
    if *name_sig.peek() != props.name {
        name_sig.set(props.name.clone());
    }
    let mut form_sig = use_signal(|| props.form.clone());
    if *form_sig.peek() != props.form {
        form_sig.set(props.form.clone());
    }
    let mut disabled_sig = use_signal(|| props.disabled);
    if *disabled_sig.peek() != props.disabled {
        disabled_sig.set(props.disabled);
    }
    let mut required_sig = use_signal(|| props.required);
    if *required_sig.peek() != props.required {
        required_sig.set(props.required);
    }
    let mut read_only_sig = use_signal(|| props.read_only);
    if *read_only_sig.peek() != props.read_only {
        read_only_sig.set(props.read_only);
    }

    let label_id_sig = use_signal(|| None::<String>);

    let initial_values = props.default_values.clone();
    let reset_values_handle = values_handle.clone();
    let (_, group_reset_epoch) = use_form_control_sync_with_epoch(
        &group_id,
        props.name.as_deref(),
        props.form.as_deref(),
        move || {
            let mut sig = values_sig;
            sig.set(initial_values.clone());
            reset_values_handle.set(initial_values.clone());
        },
    );
    let mut reset_epoch_sig = use_signal(|| group_reset_epoch);
    if *reset_epoch_sig.peek() != group_reset_epoch {
        reset_epoch_sig.set(group_reset_epoch);
    }

    let on_toggle_fn: Rc<dyn Fn(String)> = {
        let values_handle = values_handle.clone();
        let disabled = props.disabled;
        let read_only = props.read_only;
        let is_controlled = values_handle.is_controlled();
        Rc::new(move |val: String| {
            if disabled || read_only {
                return;
            }
            let mut list = values_sig.read().clone();
            if let Some(pos) = list.iter().position(|v| v == &val) {
                list.remove(pos);
            } else {
                list.push(val);
            }
            if !is_controlled {
                let mut sig = values_sig;
                sig.set(list.clone());
            }
            values_handle.set(list);
        })
    };
    let on_toggle_slot: Rc<RefCell<Rc<dyn Fn(String)>>> =
        use_hook(|| Rc::new(RefCell::new(on_toggle_fn.clone())));
    *on_toggle_slot.borrow_mut() = on_toggle_fn;

    let on_set_label_id = {
        Rc::new(move |id: Option<String>| {
            let mut sig = label_id_sig;
            sig.set(id);
        })
    };

    let group_context = CheckboxGroupContext {
        group_id: group_id.clone(),
        name: props.name.clone(),
        form: props.form.clone(),
        name_sig,
        form_sig,
        disabled: disabled_sig,
        required: required_sig,
        read_only: read_only_sig,
        values: values_sig,
        label_id: label_id_sig,
        reset_epoch: reset_epoch_sig,
        on_toggle: on_toggle_slot,
        on_set_label_id,
    };
    use_context_provider(|| group_context);

    let attrs = CheckboxGroupAttributes::new(
        Some(group_id.clone()),
        label_id_sig.read().clone(),
        props.disabled,
        props.required,
        props.read_only,
    );

    rsx! {
        div {
            id: attrs.id.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: attrs.role,
            "aria-labelledby": attrs.aria_labelledby.as_deref(),
            "aria-readonly": attrs.aria_readonly,
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            "data-readonly": attrs.data_readonly,
            "data-checkbox-group": attrs.data_checkbox_group,
            ..props.attributes,
            {props.children}
        }
    }
}

/// Props for [`CheckboxGroupLabel`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxGroupLabelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Accessible caption label for [`CheckboxGroup`], conditionally binding `aria-labelledby` on mount.
#[component]
pub fn CheckboxGroupLabel(props: CheckboxGroupLabelProps) -> Element {
    let group_ctx = use_checkbox_group_context();
    let label_id = props
        .id
        .clone()
        .unwrap_or_else(|| format!("{}-label", group_ctx.group_id()));

    {
        let group_ctx = group_ctx.clone();
        let id_to_set = label_id.clone();
        use_hook(|| {
            group_ctx.set_label_id(Some(id_to_set));
        });
    }

    use_drop({
        let group_ctx = group_ctx.clone();
        move || {
            group_ctx.set_label_id(None);
        }
    });

    let attrs = CheckboxGroupLabelAttributes::new(
        Some(label_id.clone()),
        group_ctx.is_disabled(),
        group_ctx.is_read_only(),
    );

    rsx! {
        span {
            id: attrs.id.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            "data-checkbox-group-label": attrs.data_checkbox_group_label,
            ..props.attributes,
            {props.children}
        }
    }
}

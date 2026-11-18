use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{PresenceMonitorEvent, WatcherGuard, start_presence_monitor},
    compose::{Slottable, project_as_child},
    form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
    overlay::PresenceController,
    roving_focus::{Orientation, RovingFocusConfig, RovingItemMeta, use_roving_focus_item},
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    state::{ControllableStateProps, use_controllable_state},
};

use super::{
    attrs::{RadioGroupIndicatorAttributes, RadioGroupItemAttributes, RadioGroupRootAttributes},
    relationships::RadioGroupRelationships,
    runtime::RadioGroupRuntime,
    types::{RadioGroupOrientation, RadioGroupValueChangeHandler},
};

#[derive(Clone)]
pub struct RadioGroupContext {
    pub runtime: RadioGroupRuntime,
}

pub fn use_radio_group_context() -> RadioGroupContext {
    use_context::<RadioGroupContext>()
}

pub fn try_use_radio_group_context() -> Option<RadioGroupContext> {
    try_consume_context::<RadioGroupContext>()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioGroupItemContext {
    pub id: String,
    pub value: String,
    pub disabled: bool,
    pub(crate) disabled_sig: Signal<bool>,
}

impl RadioGroupItemContext {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn is_disabled(&self) -> bool {
        *self.disabled_sig.read()
    }
}

pub fn use_radio_group_item_context() -> RadioGroupItemContext {
    use_context::<RadioGroupItemContext>()
}

pub fn try_use_radio_group_item_context() -> Option<RadioGroupItemContext> {
    try_consume_context::<RadioGroupItemContext>()
}

pub fn project_radio_group_root_as_child<T, C>(
    attrs: &RadioGroupRootAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, RadioGroupRootAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_radio_group_item_as_child<T, C>(
    attrs: &RadioGroupItemAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, RadioGroupItemAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_radio_group_indicator_as_child<T, C>(
    attrs: &RadioGroupIndicatorAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, RadioGroupIndicatorAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub orientation: Option<RadioGroupOrientation>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default = Some(String::from("off")))]
    pub autocomplete: Option<String>,
    #[props(default)]
    pub role: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn RadioGroupRoot(props: RadioGroupRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope_id = stable_id.into_owned();
    let resolved_id = props.id.clone().unwrap_or_else(|| {
        let scope = ScopeHandle::root("radio-group").child(scope_id.clone());
        RadioGroupRelationships::new(scope).root_id().to_string()
    });

    let on_value_change_cb = props.on_value_change.as_ref().map(|handler| {
        let handler = *handler;
        Rc::new(move |next: String| {
            handler.call(next);
        }) as RadioGroupValueChangeHandler
    });

    let val_handle = use_controllable_state(ControllableStateProps {
        value: props.value.clone(),
        default_value: props.default_value.clone().unwrap_or_default(),
        on_change: on_value_change_cb.as_ref().map(|cb| {
            let cb = cb.clone();
            move |v: String| {
                if !v.is_empty() {
                    cb(v);
                }
            }
        }),
    });

    let initial_val_for_reset = use_hook(|| {
        if let Some(ref controlled) = props.value {
            controlled.clone()
        } else if let Some(ref def) = props.default_value {
            def.clone()
        } else {
            String::new()
        }
    });

    let initial_val = if val_handle.value().is_empty() {
        None
    } else {
        Some(val_handle.value().clone())
    };
    let is_controlled = val_handle.is_controlled();
    let reset_initial_str = initial_val_for_reset.clone();
    let reset_initial = if reset_initial_str.is_empty() {
        None
    } else {
        Some(reset_initial_str.clone())
    };
    let handle_for_reset = val_handle.clone();
    let runtime_slot: Rc<RefCell<Option<RadioGroupRuntime>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    let runtime_for_reset = Rc::clone(&runtime_slot);

    let (should_render_hidden, reset_epoch) = use_form_control_sync_with_epoch(
        &resolved_id,
        props.name.as_deref(),
        props.form.as_deref(),
        move || {
            handle_for_reset.set(reset_initial_str.clone());
            if let Some(ref rt) = *runtime_for_reset.borrow() {
                if !*(rt.is_controlled).peek() {
                    let mut val_sig = rt.value;
                    val_sig.set(reset_initial.clone());
                    rt.roving.set_current_tab_stop(reset_initial.clone());
                }
            }
        },
    );

    let on_change_for_runtime: RadioGroupValueChangeHandler = {
        let handle = val_handle.clone();
        Rc::new(move |val: String| {
            handle.set(val);
        })
    };

    let mut runtime = RadioGroupRuntime::new(
        initial_val,
        is_controlled,
        props.orientation,
        resolved_dir,
        props.loop_focus,
        props.disabled,
        props.required,
        props.read_only,
        props.name.clone(),
        props.form.clone(),
        props.autocomplete.clone(),
        scope_id.clone(),
        resolved_id.clone(),
        should_render_hidden,
        Some(on_change_for_runtime.clone()),
    );
    *runtime_slot.borrow_mut() = Some(runtime.clone());

    {
        runtime.set_on_value_change(Some(on_change_for_runtime));
        *runtime_slot.borrow_mut() = Some(runtime.clone());

        if *runtime.root_id.peek() != scope_id {
            runtime.root_id.set(scope_id.clone());
        }

        if *runtime.is_controlled.peek() != is_controlled {
            runtime.is_controlled.set(is_controlled);
        }

        let resolved_val = if val_handle.value().is_empty() {
            None
        } else {
            Some(val_handle.value().clone())
        };
        if *runtime.value.peek() != resolved_val {
            runtime.value.set(resolved_val.clone());
            if is_controlled {
                runtime.roving.set_current_tab_stop(resolved_val);
            }
        }

        if *runtime.should_render_hidden_inputs.peek() != should_render_hidden {
            runtime.should_render_hidden_inputs.set(should_render_hidden);
        }

        if *runtime.reset_epoch.peek() != reset_epoch {
            runtime.reset_epoch.set(reset_epoch);
        }

        if *runtime.orientation.peek() != props.orientation {
            runtime.orientation.set(props.orientation);
        }

        if *runtime.dir.peek() != resolved_dir {
            runtime.dir.set(resolved_dir);
        }

        if *runtime.loop_focus.peek() != props.loop_focus {
            runtime.loop_focus.set(props.loop_focus);
        }

        if *runtime.disabled.peek() != props.disabled {
            runtime.disabled.set(props.disabled);
        }

        if *runtime.required.peek() != props.required {
            runtime.required.set(props.required);
        }

        if *runtime.read_only.peek() != props.read_only {
            runtime.read_only.set(props.read_only);
        }

        if *runtime.name.peek() != props.name {
            runtime.name.set(props.name.clone());
        }

        if *runtime.form.peek() != props.form {
            runtime.form.set(props.form.clone());
        }

        if *runtime.autocomplete.peek() != props.autocomplete {
            runtime.autocomplete.set(props.autocomplete.clone());
        }

        let mapped_orientation = match props.orientation {
            None => Orientation::Both,
            Some(RadioGroupOrientation::Horizontal) => Orientation::Horizontal,
            Some(RadioGroupOrientation::Vertical) => Orientation::Vertical,
        };

        runtime.roving.set_config(
            RovingFocusConfig::new()
                .with_orientation(mapped_orientation)
                .with_direction(resolved_dir)
                .with_loop_focus(props.loop_focus)
                .with_enabled(!props.disabled),
        );
    }

    use_context_provider(|| RadioGroupContext {
        runtime: runtime.clone(),
    });

    let root_attrs = runtime.root_attributes(None);
    let resolved_role = props.role.as_deref().unwrap_or(root_attrs.role());

    rsx! {
        div {
            id: "{resolved_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: "{resolved_role}",
            dir: "{root_attrs.dir}",
            "aria-orientation": root_attrs.aria_orientation,
            "aria-required": root_attrs.aria_required,
            "aria-disabled": root_attrs.aria_disabled,
            "aria-readonly": root_attrs.aria_readonly,
            "data-orientation": root_attrs.data_orientation,
            "data-disabled": root_attrs.data_disabled,
            "data-required": root_attrs.data_required,
            "data-readonly": root_attrs.data_readonly,
            "data-radio-group-root": "{root_attrs.data_radio_group_root}",
            ..props.attributes,
            {props.children}
        }
    }
}

pub use RadioGroupRoot as RadioGroup;

#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupItemProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub value: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn RadioGroupItem(props: RadioGroupItemProps) -> Element {
    let group_ctx = use_radio_group_context();
    let runtime = group_ctx.runtime.clone();

    let item_id = props
        .id
        .clone()
        .unwrap_or_else(|| runtime.state().relationships().item_id(&props.value));
    let hidden_input_id = runtime.state().relationships().hidden_input_id(&props.value);
    let effective_disabled = runtime.is_item_disabled(props.disabled);
    let effective_read_only = runtime.is_read_only();
    let is_checked = runtime.is_item_checked(&props.value);

    use_roving_focus_item(
        runtime.roving,
        &props.value,
        RovingItemMeta::new(item_id.clone(), !effective_disabled, is_checked),
    );

    let mut item_disabled_sig = use_signal(|| effective_disabled);
    if *item_disabled_sig.peek() != effective_disabled {
        item_disabled_sig.set(effective_disabled);
    }

    use_context_provider(|| RadioGroupItemContext {
        id: item_id.clone(),
        value: props.value.clone(),
        disabled: effective_disabled,
        disabled_sig: item_disabled_sig,
    });

    let item_attrs = runtime.item_attributes(&props.value, Some(item_id.clone()), props.disabled);

    let on_click = {
        let runtime = runtime.clone();
        let val = props.value.clone();
        let consumer_on_click = props.on_click;
        move |evt: MouseEvent| {
            if let Some(ref handler) = consumer_on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() && !effective_disabled && !effective_read_only {
                runtime.check_item(&val);
            }
        }
    };

    let on_key_down = {
        let runtime = runtime.clone();
        let val = props.value.clone();
        let consumer_on_keydown = props.on_key_down;
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = consumer_on_keydown {
                handler.call(evt.clone());
            }

            if !evt.default_action_enabled() || effective_disabled {
                return;
            }

            let key = evt.key();
            if key == Key::Named(NamedKey::Enter) {
                // RG-5: Enter suppression on RadioGroupItem
                evt.prevent_default();
                return;
            }

            // RG-3, D-5: Directional roving navigation with immediate auto-selection
            let on_focused = if !effective_read_only {
                let rt = runtime.clone();
                Some(move |next_key: &str| {
                    rt.check_item(next_key);
                })
            } else {
                None
            };

            runtime.roving.handle_keydown(Some(&val), &evt, on_focused);
        }
    };

    let on_focus = {
        let runtime = runtime.clone();
        let val = props.value.clone();
        let consumer_on_focus = props.on_focus;
        move |evt: FocusEvent| {
            if let Some(ref handler) = consumer_on_focus {
                handler.call(evt);
            }
            // RG-4: Tab / pointer / programmatic focus updates current_tab_stop WITHOUT auto-selecting
            runtime.roving.handle_item_focus(&val, !effective_disabled);
        }
    };

    let on_mounted = {
        let consumer_on_mounted = props.on_mounted;
        move |evt: MountedEvent| {
            if let Some(ref handler) = consumer_on_mounted {
                handler.call(evt);
            }
        }
    };

    let should_render_hidden = (runtime.should_render_hidden_inputs)();
    let reset_epoch = (runtime.reset_epoch)();
    let group_name = (runtime.name)();
    let group_form = (runtime.form)();
    let group_required = (runtime.required)();
    let group_autocomplete = (runtime.autocomplete)();

    rsx! {
        button {
            id: "{item_attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            r#type: "button",
            role: "{item_attrs.role}",
            "aria-checked": "{item_attrs.aria_checked}",
            tabindex: item_attrs.tabindex,
            disabled: item_attrs.disabled,
            "data-state": item_attrs.data_state.as_str(),
            "data-disabled": item_attrs.data_disabled,
            "data-readonly": item_attrs.data_readonly,
            "data-orientation": item_attrs.data_orientation,
            "data-value": "{item_attrs.data_value}",
            "data-radio-group-item": "{item_attrs.data_radio_group_item}",
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onmounted: on_mounted,
            ..props.attributes,
            {props.children}
        }
        if should_render_hidden {
            HiddenFormInput {
                id: hidden_input_id,
                kind: HiddenInputKind::Radio,
                name: group_name,
                value: props.value.clone(),
                checked: is_checked,
                disabled: effective_disabled,
                required: group_required,
                read_only: effective_read_only,
                form: group_form,
                autocomplete: group_autocomplete,
                focus_target_id: item_id,
                reset_epoch: reset_epoch,
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupIndicatorProps {
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

#[component]
pub fn RadioGroupIndicator(props: RadioGroupIndicatorProps) -> Element {
    let group_ctx = use_radio_group_context();
    let item_ctx = use_radio_group_item_context();

    let is_checked = group_ctx.runtime.is_item_checked(&item_ctx.value);
    let mut presence =
        use_signal(|| PresenceController::new(is_checked).with_retained_mount(true));
    let watcher_slot: Rc<RefCell<Option<WatcherGuard>>> = use_hook(|| Rc::new(RefCell::new(None)));

    let should_render = is_checked || presence.read().should_render() || props.force_mount;

    let attrs = group_ctx.runtime.indicator_attributes(
        &item_ctx.value,
        props.id.clone(),
        item_ctx.is_disabled(),
    );

    use_effect(use_reactive((&is_checked,), {
        let indicator_id = attrs.id.clone();
        let watcher_slot = Rc::clone(&watcher_slot);
        move |(checked,)| {
            let update = presence.write().sync(checked);
            if let Some(cycle_id) = update.started_close_cycle() {
                if let Some(prev) = watcher_slot.borrow_mut().take() {
                    prev.stop();
                }
                let watcher = start_presence_monitor(
                    indicator_id.as_str(),
                    cycle_id,
                    move |event| match event {
                        PresenceMonitorEvent::TransitionEnd {
                            cycle_id: event_cycle,
                            ..
                        }
                        | PresenceMonitorEvent::TransitionCancel {
                            cycle_id: event_cycle,
                            ..
                        }
                        | PresenceMonitorEvent::AnimationEnd {
                            cycle_id: event_cycle,
                            ..
                        }
                        | PresenceMonitorEvent::AnimationCancel {
                            cycle_id: event_cycle,
                            ..
                        }
                        | PresenceMonitorEvent::Fallback {
                            cycle_id: event_cycle,
                            ..
                        } => {
                            presence.write().complete_close_cycle(event_cycle);
                        }
                        _ => {}
                    },
                );
                *watcher_slot.borrow_mut() = Some(watcher);
            } else if !update.should_render() || update.invalidated_close_cycle().is_some() {
                if let Some(prev) = watcher_slot.borrow_mut().take() {
                    prev.stop();
                }
            }
        }
    }));

    let drop_watcher = Rc::clone(&watcher_slot);
    use_drop(move || {
        if let Some(prev) = drop_watcher.borrow_mut().take() {
            prev.stop();
        }
    });

    if !should_render {
        return rsx! {};
    }

    let on_mounted = {
        let consumer_on_mounted = props.on_mounted;
        move |evt: MountedEvent| {
            if let Some(ref handler) = consumer_on_mounted {
                handler.call(evt);
            }
        }
    };

    let effective_style = if let Some(ref custom) = props.style {
        if custom.is_empty() {
            attrs.style().to_string()
        } else {
            format!("{}; {}", attrs.style(), custom)
        }
    } else {
        attrs.style().to_string()
    };

    rsx! {
        span {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{effective_style}",
            "data-state": attrs.data_state.as_str(),
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            "data-orientation": attrs.data_orientation,
            "data-radio-group-indicator": "{attrs.data_radio_group_indicator}",
            onmounted: on_mounted,
            ..props.attributes,
            {props.children}
        }
    }
}

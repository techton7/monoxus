use std::borrow::Cow;
use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
    SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
};
use crate::button::ButtonType;
use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    browser::focus::use_composite_focus_out,
    compose::{
        MergePropsConfig, collect_user_attrs, extract_aria_invalid_and_describedby, merge_props,
        merge_props_with_config, push_attr_opt, push_event_handler,
        resolve_invalid_and_describedby,
    },
    form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
    keyboard::is_enter_key,
    overlay::use_presence,
    shared::{ScopeHandle, use_stable_id},
    state::{ControllableConfig, sync_signal, use_controllable_signal_config},
};

use super::{
    attrs::{
        CheckboxGroupAttributes, CheckboxGroupLabelAttributes, CheckboxIndicatorAttributes,
        CheckboxRootAttributes,
    },
    relationships::{CheckboxGroupRelationships, CheckboxRelationships},
    runtime::{
        CheckboxContext, CheckboxGroupContext, try_use_checkbox_group_context,
        use_checkbox_context, use_checkbox_group_context,
    },
    types::CheckedState,
};

/// Props for [`CheckboxRoot`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub checked: Option<CheckedState>,
    #[props(default, into)]
    pub default_checked: CheckedState,
    #[props(default)]
    pub on_checked_change: Option<EventHandler<CheckedState>>,
    #[props(default)]
    pub on_checked_change_details: Option<EventHandler<ChangeEvent<CheckedState>>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default, into)]
    pub name: Option<String>,
    #[props(default = String::from("on"), into)]
    pub value: String,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default)]
    pub r#type: ButtonType,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Headless interactive `<button role="checkbox">` trigger element.
#[component]
pub fn CheckboxRoot(props: CheckboxRootProps) -> Element {
    let group_ctx = try_use_checkbox_group_context();
    let field_ctx = try_use_field_context();
    let control_ctx = try_use_field_control_context();

    let effective_name = props
        .name
        .clone()
        .or_else(|| group_ctx.as_ref().and_then(|g| g.reactive_name()))
        .or_else(|| field_ctx.and_then(|f| f.name()));
    let effective_form = props
        .form
        .clone()
        .or_else(|| group_ctx.as_ref().and_then(|g| g.reactive_form()));
    let effective_disabled = props.disabled
        || group_ctx.as_ref().is_some_and(|g| g.is_disabled())
        || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_required = props.required
        || group_ctx.as_ref().is_some_and(|g| g.is_required())
        || field_ctx.is_some_and(|f| f.is_required());
    let effective_read_only = props.read_only
        || group_ctx.as_ref().is_some_and(|g| g.is_read_only())
        || field_ctx.is_some_and(|f| f.is_read_only());

    let explicit_id = props.id.clone().filter(|id| !id.trim().is_empty());
    let bound_field_id = if group_ctx.is_none() {
        if let Some(ref explicit) = explicit_id {
            if let Some(ctrl_ctx) = control_ctx {
                if !ctrl_ctx.is_explicit_id_locked() {
                    ctrl_ctx.sync_control_id(explicit.clone());
                }
            } else if let Some(f_ctx) = field_ctx
                && !f_ctx.is_fieldset()
            {
                f_ctx.set_primary_control_id(Some(explicit.clone()));
            }
            Some(explicit.clone())
        } else if let Some(ctrl_ctx) = control_ctx {
            Some(ctrl_ctx.control_id())
        } else if let Some(f_ctx) = field_ctx
            && !f_ctx.is_fieldset()
        {
            Some(f_ctx.resolved_control_id())
        } else {
            None
        }
    } else {
        explicit_id.clone()
    };

    let stable_id = use_stable_id(bound_field_id.clone().map(Cow::Owned));
    let relationships = {
        let scope = ScopeHandle::root("checkbox").child(stable_id);
        if let Some(ref resolved_explicit_id) = bound_field_id {
            CheckboxRelationships::with_explicit_root_id(scope, resolved_explicit_id)
        } else {
            CheckboxRelationships::new(scope)
        }
    };

    let root_id = relationships.root_id().to_string();
    let indicator_id = relationships.indicator_id().to_string();

    let extracted = extract_aria_invalid_and_describedby(props.attributes);
    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let inherited_invalid = group_ctx
        .as_ref()
        .map(|g| g.is_invalid())
        .or_else(|| field_ctx.map(|f| f.is_invalid()));
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        props.invalid,
        extracted.aria_invalid,
        inherited_invalid,
        false,
        props.aria_describedby.as_deref(),
        extracted.aria_describedby.as_deref(),
        field_describedby.as_deref(),
    );

    let is_in_group = group_ctx.is_some() && props.checked.is_none();
    let item_val_for_group = props.value.clone();

    let standalone_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.checked, props.default_checked)
            .with_explicit_default(props.default_checked != CheckedState::default())
            .with_on_change(props.on_checked_change)
            .with_on_change_details(props.on_checked_change_details)
            .with_component("CheckboxRoot")
            .with_prop("checked"),
    );
    let mut on_checked_change_slot = use_hook(|| CopyValue::new(props.on_checked_change));
    on_checked_change_slot.set(props.on_checked_change);
    let mut on_checked_change_details_slot =
        use_hook(|| CopyValue::new(props.on_checked_change_details));
    on_checked_change_details_slot.set(props.on_checked_change_details);

    let current_checked = if is_in_group {
        let g = group_ctx.as_ref().unwrap();
        if g.is_item_checked(&item_val_for_group) {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        }
    } else {
        standalone_ctl.get()
    };

    let initial_checked_for_reset =
        use_hook(|| props.checked.unwrap_or(props.default_checked));
    let on_reset_fn = {
        let has_group = group_ctx.is_some();
        move || {
            if !has_group {
                standalone_ctl.set_with_context(
                    initial_checked_for_reset,
                    ChangeContext::new(ChangeReason::Reset, ChangeSource::Programmatic),
                );
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
    sync_signal(&mut checked_sig, current_checked);
    let mut disabled_sig = use_signal(|| effective_disabled);
    sync_signal(&mut disabled_sig, effective_disabled);
    let mut required_sig = use_signal(|| effective_required);
    sync_signal(&mut required_sig, effective_required);
    let mut read_only_sig = use_signal(|| effective_read_only);
    sync_signal(&mut read_only_sig, effective_read_only);
    let mut invalid_sig = use_signal(|| effective_invalid);
    sync_signal(&mut invalid_sig, effective_invalid);

    let context = CheckboxContext {
        root_id: root_id.clone(),
        indicator_id: indicator_id.clone(),
        checked: checked_sig,
        disabled: disabled_sig,
        required: required_sig,
        read_only: read_only_sig,
        invalid: invalid_sig,
    };
    use_context_provider(|| context);

    let checked_state_to_snap_val = |state: CheckedState| match state {
        CheckedState::Checked => SnapshotValue::Bool(true),
        CheckedState::Unchecked => SnapshotValue::Bool(false),
        CheckedState::Indeterminate => SnapshotValue::text("indeterminate"),
    };
    let agent_label = effective_name
        .clone()
        .unwrap_or_else(|| root_id.clone());
    let agent_group_ctx = group_ctx.clone();
    let agent_item_value = props.value.clone();
    let agent_reader_group_ctx = group_ctx.clone();
    let agent_reader_item_value = props.value.clone();
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new(),
        || {
            let mut snap = ComponentSnapshot::new()
                .with_field("checked", checked_state_to_snap_val(current_checked));
            if effective_read_only {
                snap = snap.with_field("read_only", SnapshotValue::Bool(true));
            }
            let actions = if effective_read_only {
                Vec::new()
            } else {
                vec![
                    ActionDescriptor::new("toggle", "Toggle checkbox state", false),
                    ActionDescriptor::new("set", "Set checkbox checked state", true),
                ]
            };
            AgentSurface::new("checkbox", agent_label)
                .with_disabled(effective_disabled)
                .with_snapshot(snap)
                .with_actions(actions)
        },
        || {
            Rc::new(move |dispatch| {
                if effective_read_only {
                    return DispatchExecResult::ok().with_host_error("checkbox is read-only");
                }
                let target_state = match dispatch.action.as_str() {
                    "toggle" => {
                        let cur = if is_in_group
                            && let Some(ref g) = agent_group_ctx
                        {
                            if g.is_item_checked(&agent_item_value) {
                                CheckedState::Checked
                            } else {
                                CheckedState::Unchecked
                            }
                        } else {
                            standalone_ctl.peek()
                        };
                        cur.toggle()
                    }
                    "set" => {
                        match dispatch
                            .args
                            .get("checked")
                            .or_else(|| dispatch.args.get("value"))
                        {
                            Some(SnapshotValue::Bool(true)) => CheckedState::Checked,
                            Some(SnapshotValue::Bool(false)) => CheckedState::Unchecked,
                            Some(SnapshotValue::Text(s)) => match s.as_str() {
                                "checked" | "true" => CheckedState::Checked,
                                "unchecked" | "false" => CheckedState::Unchecked,
                                "indeterminate" => CheckedState::Indeterminate,
                                _ => {
                                    return DispatchExecResult::ok().with_host_error(
                                        "invalid 'checked' value; expected bool or 'checked'|'unchecked'|'indeterminate'",
                                    );
                                }
                            },
                            _ => {
                                return DispatchExecResult::ok().with_host_error(
                                    "missing or invalid argument 'checked'",
                                );
                            }
                        }
                    }
                    other => {
                        return DispatchExecResult::ok()
                            .with_host_error(format!("unknown checkbox action '{other}'"));
                    }
                };

                if is_in_group
                    && let Some(ref g) = agent_group_ctx
                {
                    let currently_checked = g.is_item_checked(&agent_item_value);
                    let want_checked = target_state.is_checked();
                    if currently_checked != want_checked {
                        let outcome = g.toggle_item_with_context(
                            agent_item_value.clone(),
                            dispatch.ctx,
                        );
                        if outcome.is_canceled() {
                            return DispatchExecResult::ok().with_canceled(true);
                        }
                        if outcome.is_accepted() {
                            let ev_ctx = outcome.context().unwrap_or(dispatch.ctx);
                            let ev = ChangeEvent::new(target_state, ev_ctx);
                            if let Some(details_cb) =
                                on_checked_change_details_slot.try_read().ok().and_then(|s| *s)
                            {
                                details_cb.call(ev.clone());
                            }
                            if !ev.is_canceled()
                                && let Some(cb) =
                                    on_checked_change_slot.try_read().ok().and_then(|s| *s)
                            {
                                cb.call(target_state);
                            }
                        }
                    }
                    DispatchExecResult::ok()
                        .with_expected("checked", checked_state_to_snap_val(target_state))
                } else {
                    let outcome = standalone_ctl.set_with_context(target_state, dispatch.ctx);
                    let mut res =
                        DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected(
                            "checked",
                            checked_state_to_snap_val(target_state),
                        );
                    }
                    res
                }
            })
        },
        || {
            Rc::new(move || {
                let state = if is_in_group
                    && let Some(ref g) = agent_reader_group_ctx
                {
                    if g.is_item_checked(&agent_reader_item_value) {
                        CheckedState::Checked
                    } else {
                        CheckedState::Unchecked
                    }
                } else {
                    standalone_ctl
                        .try_peek_cloned()
                        .unwrap_or(CheckedState::Unchecked)
                };
                let mut snap = ComponentSnapshot::new()
                    .with_field("checked", checked_state_to_snap_val(state));
                if effective_read_only {
                    snap = snap.with_field("read_only", SnapshotValue::Bool(true));
                }
                snap
            })
        },
    );

    let attrs = CheckboxRootAttributes::new(
        Some(root_id.clone()),
        props.r#type.as_str(),
        current_checked,
        effective_disabled,
        effective_required,
        effective_read_only,
    )
    .with_invalid(effective_invalid)
    .with_aria_describedby(effective_describedby);

    let data_state = attrs.data_state_str();
    let mut base = Vec::with_capacity(20);
    push_attr_opt(&mut base, "id", attrs.id);
    base.push(Attribute::new("type", attrs.r#type, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("aria-checked", attrs.aria_checked, None, false));
    push_attr_opt(&mut base, "aria-required", attrs.aria_required);
    push_attr_opt(&mut base, "aria-readonly", attrs.aria_readonly);
    push_attr_opt(&mut base, "aria-invalid", attrs.aria_invalid);
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    base.push(Attribute::new(
        "data-checkbox-root",
        attrs.data_checkbox_root,
        None,
        false,
    ));
    if attrs.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("value", props.value.clone(), None, false));

    {
        let group_ctx = group_ctx.clone();
        let item_value = props.value.clone();
        push_event_handler(
            &mut base,
            "onclick",
            Some(EventHandler::new(move |_evt: MouseEvent| {
                if !effective_disabled && !effective_read_only {
                    let ctx = ChangeContext::new(ChangeReason::Toggle, ChangeSource::Mouse);
                    if is_in_group
                        && let Some(ref g) = group_ctx
                    {
                        let next = if g.is_item_checked(&item_value) {
                            CheckedState::Unchecked
                        } else {
                            CheckedState::Checked
                        };
                        let outcome = g.toggle_item_with_context(item_value.clone(), ctx);
                        if outcome.is_accepted() {
                            let ev_ctx = outcome.context().unwrap_or(ctx);
                            let ev = ChangeEvent::new(next, ev_ctx);
                            if let Some(details_cb) =
                                on_checked_change_details_slot.try_read().ok().and_then(|s| *s)
                            {
                                details_cb.call(ev.clone());
                            }
                            if !ev.is_canceled()
                                && let Some(cb) =
                                    on_checked_change_slot.try_read().ok().and_then(|s| *s)
                            {
                                cb.call(next);
                            }
                        }
                    } else {
                        let current = standalone_ctl.peek();
                        standalone_ctl.set_with_context(current.toggle(), ctx);
                    }
                }
            })),
        );
    }
    push_event_handler(
        &mut base,
        "onkeydown",
        Some(EventHandler::new(move |evt: KeyboardEvent| {
            if is_enter_key(&evt.key()) {
                // Per WAI-ARIA 1.2 Checkbox Pattern, Enter must NOT toggle checkbox.
                evt.prevent_default();
            }
            // Space is left uncancelled so native <button> synthesizes Click on KeyUp with zero double-fire.
        })),
    );

    let mut user = collect_user_attrs(props.class, props.style, extracted.rest);
    if !effective_disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
    }
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
    push_event_handler(&mut user, "onblur", props.on_blur);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    let root_el = if let Some(child) = props.child {
        child.call(merged)
    } else {
        rsx! {
            button {
                ..merged,
                {props.children}
            }
        }
    };

    rsx! {
        {root_el}
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
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let indicator_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.indicator_id().to_string());

    let presence = use_presence(desired_present, &indicator_id, true, None);

    if !presence.should_render() {
        return rsx! {};
    }

    let attrs = CheckboxIndicatorAttributes::new(
        Some(indicator_id),
        state,
        ctx.is_disabled(),
        ctx.is_read_only(),
        None,
    )
    .with_invalid(ctx.is_invalid());

    let data_state = attrs.data_state_str();
    let mut base = Vec::with_capacity(8);
    push_attr_opt(&mut base, "id", attrs.id);
    if !attrs.style.is_empty() {
        base.push(Attribute::new("style", attrs.style, None, false));
    }
    base.push(Attribute::new("data-state", data_state, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    base.push(Attribute::new(
        "data-checkbox-indicator",
        attrs.data_checkbox_indicator,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        span {
            ..merged,
            {props.children}
        }
    }
}

/// Props for [`CheckboxGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxGroupProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub values: Option<Vec<String>>,
    #[props(default)]
    pub default_values: Vec<String>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default)]
    pub on_values_change_details: Option<EventHandler<ChangeEvent<Vec<String>>>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(default, into)]
    pub name: Option<String>,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Headless `<div role="group">` container managing multi-value checkbox membership and cascading attributes.
#[component]
pub fn CheckboxGroup(props: CheckboxGroupProps) -> Element {
    let field_ctx = try_use_field_context();
    let effective_name = props
        .name
        .clone()
        .or_else(|| field_ctx.and_then(|f| f.name()));
    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_required = props.required || field_ctx.is_some_and(|f| f.is_required());
    let effective_read_only = props.read_only || field_ctx.is_some_and(|f| f.is_read_only());

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

    let extracted = extract_aria_invalid_and_describedby(props.attributes);
    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        props.invalid,
        extracted.aria_invalid,
        field_ctx.map(|f| f.is_invalid()),
        false,
        props.aria_describedby.as_deref(),
        extracted.aria_describedby.as_deref(),
        field_describedby.as_deref(),
    );

    let values_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.values.clone(), props.default_values.clone())
            .with_explicit_default(!props.default_values.is_empty())
            .with_on_change(props.on_values_change)
            .with_on_change_details(props.on_values_change_details)
            .with_component("CheckboxGroup")
            .with_prop("values"),
    );
    let values_sig = values_ctl.signal();

    let mut name_sig = use_signal(|| effective_name.clone());
    sync_signal(&mut name_sig, effective_name.clone());
    let mut form_sig = use_signal(|| props.form.clone());
    sync_signal(&mut form_sig, props.form.clone());
    let mut disabled_sig = use_signal(|| effective_disabled);
    sync_signal(&mut disabled_sig, effective_disabled);
    let mut required_sig = use_signal(|| effective_required);
    sync_signal(&mut required_sig, effective_required);
    let mut read_only_sig = use_signal(|| effective_read_only);
    sync_signal(&mut read_only_sig, effective_read_only);
    let mut invalid_sig = use_signal(|| effective_invalid);
    sync_signal(&mut invalid_sig, effective_invalid);

    let label_id_sig = use_signal(|| None::<String>);

    let initial_values = use_hook(|| {
        props
            .values
            .clone()
            .unwrap_or_else(|| props.default_values.clone())
    });
    let (_, group_reset_epoch) = use_form_control_sync_with_epoch(
        &group_id,
        effective_name.as_deref(),
        props.form.as_deref(),
        move || {
            values_ctl.set_with_context(
                initial_values.clone(),
                ChangeContext::new(ChangeReason::Reset, ChangeSource::Programmatic),
            );
        },
    );
    let mut reset_epoch_sig = use_signal(|| group_reset_epoch);
    sync_signal(&mut reset_epoch_sig, group_reset_epoch);

    let agent_label = effective_name
        .clone()
        .unwrap_or_else(|| group_id.clone());
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new(),
        || {
            let vals = values_ctl
                .try_peek_cloned()
                .unwrap_or_default()
                .into_iter()
                .map(SnapshotValue::Text)
                .collect();
            let mut snap =
                ComponentSnapshot::new().with_field("values", SnapshotValue::List(vals));
            if effective_read_only {
                snap = snap.with_field("read_only", SnapshotValue::Bool(true));
            }
            let actions = if effective_read_only {
                Vec::new()
            } else {
                vec![
                    ActionDescriptor::new("toggle", "Toggle an item in the checkbox group", true),
                    ActionDescriptor::new("set", "Set selected checkbox group values", true),
                ]
            };
            AgentSurface::new("checkbox-group", agent_label)
                .with_disabled(effective_disabled)
                .with_snapshot(snap)
                .with_actions(actions)
        },
        || {
            Rc::new(move |dispatch| {
                if effective_read_only {
                    return DispatchExecResult::ok().with_host_error("checkbox-group is read-only");
                }
                match dispatch.action.as_str() {
                    "toggle" => {
                        let Some(SnapshotValue::Text(val)) = dispatch.args.get("value") else {
                            return DispatchExecResult::ok()
                                .with_host_error("missing or invalid string argument 'value'");
                        };
                        let mut list = values_ctl.peek();
                        if let Some(pos) = list.iter().position(|v| v == val) {
                            list.remove(pos);
                        } else {
                            list.push(val.clone());
                        }
                        let expected: Vec<SnapshotValue> =
                            list.iter().cloned().map(SnapshotValue::Text).collect();
                        let outcome = values_ctl.set_with_context(list, dispatch.ctx);
                        let mut res =
                            DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                        if !outcome.is_canceled() {
                            res = res.with_expected("values", SnapshotValue::List(expected));
                        }
                        res
                    }
                    "set" => {
                        let Some(SnapshotValue::List(items)) = dispatch
                            .args
                            .get("values")
                            .or_else(|| dispatch.args.get("value"))
                        else {
                            return DispatchExecResult::ok()
                                .with_host_error("missing or invalid list argument 'values'");
                        };
                        let mut next = Vec::with_capacity(items.len());
                        for item in items {
                            let SnapshotValue::Text(s) = item else {
                                return DispatchExecResult::ok().with_host_error(
                                    "all elements of 'values' must be strings",
                                );
                            };
                            next.push(s.clone());
                        }
                        let expected: Vec<SnapshotValue> =
                            next.iter().cloned().map(SnapshotValue::Text).collect();
                        let outcome = values_ctl.set_with_context(next, dispatch.ctx);
                        let mut res =
                            DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                        if !outcome.is_canceled() {
                            res = res.with_expected("values", SnapshotValue::List(expected));
                        }
                        res
                    }
                    other => DispatchExecResult::ok()
                        .with_host_error(format!("unknown checkbox-group action '{other}'")),
                }
            })
        },
        || {
            Rc::new(move || {
                let vals = values_ctl
                    .try_peek_cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(SnapshotValue::Text)
                    .collect();
                let mut snap =
                    ComponentSnapshot::new().with_field("values", SnapshotValue::List(vals));
                if effective_read_only {
                    snap = snap.with_field("read_only", SnapshotValue::Bool(true));
                }
                snap
            })
        },
    );

    let group_context = CheckboxGroupContext {
        group_id: group_id.clone(),
        name: effective_name,
        form: props.form.clone(),
        name_sig,
        form_sig,
        disabled: disabled_sig,
        required: required_sig,
        read_only: read_only_sig,
        invalid: invalid_sig,
        values: values_sig,
        values_ctl,
        label_id: label_id_sig,
        reset_epoch: reset_epoch_sig,
    };
    use_context_provider(|| group_context);

    let (handle_focus_in, handle_focus_out) =
        use_composite_focus_out(vec![group_id.clone()], props.on_focus_out);

    let attrs = CheckboxGroupAttributes::new(
        Some(group_id),
        label_id_sig.read().clone(),
        effective_disabled,
        effective_required,
        effective_read_only,
    )
    .with_invalid(effective_invalid)
    .with_aria_describedby(effective_describedby);

    let mut base = Vec::with_capacity(14);
    push_attr_opt(&mut base, "id", attrs.id);
    base.push(Attribute::new("role", attrs.role, None, false));
    push_attr_opt(&mut base, "aria-labelledby", attrs.aria_labelledby);
    push_attr_opt(&mut base, "aria-readonly", attrs.aria_readonly);
    push_attr_opt(&mut base, "aria-invalid", attrs.aria_invalid);
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    base.push(Attribute::new(
        "data-checkbox-group",
        attrs.data_checkbox_group,
        None,
        false,
    ));
    push_event_handler(&mut base, "onfocusin", Some(handle_focus_in));
    push_event_handler(&mut base, "onfocusout", Some(handle_focus_out));

    let user = collect_user_attrs(props.class, props.style, extracted.rest);
    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &["onfocusin", "onfocusout"],
        },
    );

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

/// Props for [`CheckboxGroupLabel`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxGroupLabelProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
        Some(label_id),
        group_ctx.is_disabled(),
        group_ctx.is_read_only(),
    );

    let mut base = Vec::with_capacity(4);
    push_attr_opt(&mut base, "id", attrs.id);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    base.push(Attribute::new(
        "data-checkbox-group-label",
        attrs.data_checkbox_group_label,
        None,
        false,
    ));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        span {
            ..merged,
            {props.children}
        }
    }
}

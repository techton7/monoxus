use std::borrow::Cow;

use dioxus::prelude::*;

use crate::field::try_use_field_context;
use crate::foundation::{
    browser::focus::use_composite_focus_out,
    compose::{
        extract_aria_invalid_and_describedby, join_styles, merge_global_attributes,
        resolve_invalid_and_describedby,
    },
    form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
    keyboard::is_enter_key,
    overlay::use_presence,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    state::{sync_signal, use_controllable_signal},
};

use super::{
    relationships::RadioGroupRelationships,
    runtime::{RadioGroupRuntime, use_radio_group_runtime},
    types::RadioGroupOrientation,
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
    pub invalid: Option<bool>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
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
    let field_ctx = try_use_field_context();
    let effective_name = props
        .name
        .clone()
        .or_else(|| field_ctx.and_then(|f| f.name()));
    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_required = props.required || field_ctx.is_some_and(|f| f.is_required());
    let effective_read_only = props.read_only || field_ctx.is_some_and(|f| f.is_read_only());

    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope_id = stable_id.into_owned();
    let resolved_id = props.id.clone().unwrap_or_else(|| {
        let scope = ScopeHandle::root("radio-group").child(scope_id.clone());
        RadioGroupRelationships::new(scope).root_id().to_string()
    });

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

    let controlled_val = props.value.clone().map(|v| (!v.is_empty()).then_some(v));
    let default_val = props.default_value.clone().filter(|v| !v.is_empty());
    let prev_controlled_val = use_hook(|| CopyValue::new(controlled_val.clone()));

    let value_ctl = use_controllable_signal(controlled_val.clone(), default_val.clone(), None);
    let initial_val_for_reset = use_hook(|| {
        controlled_val
            .clone()
            .unwrap_or_else(|| default_val.clone())
    });

    let mut runtime = use_radio_group_runtime(
        value_ctl,
        initial_val_for_reset.clone(),
        props.orientation,
        resolved_dir,
        props.loop_focus,
        effective_disabled,
        effective_required,
        effective_read_only,
        effective_name.clone(),
        props.form.clone(),
        props.autocomplete.clone(),
        scope_id,
        resolved_id.clone(),
        false,
        props.on_value_change,
    );
    sync_signal(&mut runtime.invalid, effective_invalid);

    if value_ctl.is_controlled() {
        let mut prev_slot = prev_controlled_val;
        if prev_slot.try_read().is_ok_and(|p| *p != controlled_val) {
            prev_slot.set(controlled_val.clone());
            runtime
                .roving
                .set_current_tab_stop(controlled_val.flatten());
        }
    }

    let reset_runtime = runtime.clone();
    let reset_initial = initial_val_for_reset;
    let (should_render_hidden, reset_epoch) = use_form_control_sync_with_epoch(
        &resolved_id,
        effective_name.as_deref(),
        props.form.as_deref(),
        move || {
            reset_runtime.reset_to(reset_initial.clone());
        },
    );
    sync_signal(&mut runtime.should_render_hidden_inputs, should_render_hidden);
    sync_signal(&mut runtime.reset_epoch, reset_epoch);

    use_context_provider(|| RadioGroupContext {
        runtime: runtime.clone(),
    });

    let (handle_focus_in, handle_focus_out) =
        use_composite_focus_out(vec![resolved_id.clone()], props.on_focus_out);

    let root_attrs = runtime
        .root_attributes(None)
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby);
    let resolved_role = props.role.as_deref().unwrap_or(root_attrs.role());
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        extracted.rest,
    );

    rsx! {
        div {
            id: "{resolved_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            role: "{resolved_role}",
            dir: "{root_attrs.dir}",
            "aria-orientation": root_attrs.aria_orientation,
            "aria-required": root_attrs.aria_required,
            "aria-disabled": root_attrs.aria_disabled,
            "aria-readonly": root_attrs.aria_readonly,
            "aria-invalid": root_attrs.aria_invalid,
            "aria-describedby": root_attrs.aria_describedby.as_deref(),
            "data-orientation": root_attrs.data_orientation,
            "data-disabled": root_attrs.data_disabled,
            "data-required": root_attrs.data_required,
            "data-readonly": root_attrs.data_readonly,
            "data-invalid": root_attrs.data_invalid,
            "data-radio-group-root": "{root_attrs.data_radio_group_root}",
            onfocusin: handle_focus_in,
            onfocusout: handle_focus_out,
            ..merged.rest,
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
    pub on_blur: Option<EventHandler<FocusEvent>>,
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
    sync_signal(&mut item_disabled_sig, effective_disabled);

    use_context_provider(|| RadioGroupItemContext {
        id: item_id.clone(),
        value: props.value.clone(),
        disabled: effective_disabled,
        disabled_sig: item_disabled_sig,
    });

    let mut on_blur_slot = use_hook(|| CopyValue::new(props.on_blur));
    on_blur_slot.set(props.on_blur);

    let item_attrs = runtime
        .item_attributes(&props.value, Some(item_id.clone()), props.disabled)
        .with_invalid(runtime.is_invalid());

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

            if is_enter_key(&evt.key()) {
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

    let on_blur = move |evt: FocusEvent| {
        if let Some(handler) = on_blur_slot.try_read().ok().and_then(|s| *s) {
            handler.call(evt);
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
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        button {
            id: "{item_attrs.id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            r#type: "button",
            role: "{item_attrs.role}",
            "aria-checked": "{item_attrs.aria_checked}",
            tabindex: item_attrs.tabindex,
            disabled: item_attrs.disabled,
            "data-state": item_attrs.data_state.as_str(),
            "data-disabled": item_attrs.data_disabled,
            "data-readonly": item_attrs.data_readonly,
            "data-invalid": item_attrs.data_invalid,
            "data-orientation": item_attrs.data_orientation,
            "data-value": "{item_attrs.data_value}",
            "data-radio-group-item": "{item_attrs.data_radio_group_item}",
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onblur: on_blur,
            onmounted: on_mounted,
            ..merged.rest,
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
    let attrs = group_ctx.runtime.indicator_attributes(
        &item_ctx.value,
        props.id.clone(),
        item_ctx.is_disabled(),
    );

    let presence = use_presence(is_checked, &attrs.id, true, None);
    let should_render = is_checked || presence.should_render() || props.force_mount;

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

    let combined_style = join_styles([attrs.style(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        props.attributes,
    );

    rsx! {
        span {
            id: "{attrs.id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            "data-state": attrs.data_state.as_str(),
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            "data-orientation": attrs.data_orientation,
            "data-radio-group-indicator": "{attrs.data_radio_group_indicator}",
            onmounted: on_mounted,
            ..merged.rest,
            {props.children}
        }
    }
}

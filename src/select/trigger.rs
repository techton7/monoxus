use std::borrow::Cow;

use dioxus::core::AttributeValue;
use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    browser::focus::use_composite_focus_out,
    compose::{
        collect_user_attrs, extract_aria_invalid_and_describedby, merge_props, push_attr_opt,
        push_bool_data_attr, push_event_handler, resolve_invalid_and_describedby,
    },
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    state::use_controllable_signal,
};

use super::{
    hidden_input::SelectHiddenInput,
    runtime::use_select_runtime_controllable,
    state::Select,
    types::{SelectContext, SelectItemData, SelectMode},
};

fn extract_aria_labelledby(attrs: Vec<Attribute>) -> (Option<String>, Vec<Attribute>) {
    let has_target = attrs
        .iter()
        .any(|attr| attr.namespace.is_none() && attr.name == "aria-labelledby");
    if !has_target {
        return (None, attrs);
    }
    let mut found: Option<String> = None;
    let mut rest = Vec::with_capacity(attrs.len());
    for attr in attrs {
        if attr.namespace.is_none() && attr.name == "aria-labelledby" {
            match attr.value {
                AttributeValue::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        found = Some(trimmed.to_owned());
                    }
                }
                AttributeValue::None => {}
                other => rest.push(Attribute {
                    value: other,
                    ..attr
                }),
            }
        } else {
            rest.push(attr);
        }
    }
    (found, rest)
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub mode: SelectMode,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub values: Option<Vec<String>>,
    #[props(default)]
    pub default_values: Option<Vec<String>>,
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<String>>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default = false)]
    pub allow_deselect: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default, into)]
    pub name: Option<String>,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default, into)]
    pub autocomplete: Option<String>,
    #[props(default)]
    pub items: Option<Vec<SelectItemData>>,
    #[props(default = false)]
    pub loop_selection: bool,
    #[props(default)]
    pub dir: Option<Direction>,
    pub children: Element,
}

#[component]
pub fn SelectRoot(props: SelectRootProps) -> Element {
    let SelectRootProps {
        id,
        mode,
        value,
        default_value,
        values,
        default_values,
        open,
        default_open,
        on_value_change,
        on_values_change,
        on_open_change,
        on_open_change_complete,
        on_focus_out,
        allow_deselect,
        disabled,
        required,
        invalid,
        name,
        form,
        autocomplete,
        items,
        loop_selection,
        dir,
        children,
    } = props;

    let field_ctx = try_use_field_context();
    let effective_name = name.clone().or_else(|| field_ctx.and_then(|f| f.name()));
    let effective_disabled = disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_required = required || field_ctx.is_some_and(|f| f.is_required());
    let effective_read_only = field_ctx.is_some_and(|f| f.is_read_only());

    let root_invalid_override = invalid;
    let effective_root_invalid = root_invalid_override
        .or_else(|| field_ctx.map(|f| f.is_invalid()))
        .unwrap_or(false);

    let value_ctl = use_controllable_signal(
        value.clone().map(Some),
        default_value.clone(),
        on_value_change,
    );
    let values_ctl = use_controllable_signal(
        values.clone(),
        default_values.clone().unwrap_or_default(),
        on_values_change,
    );
    let open_ctl = use_controllable_signal(open, default_open, on_open_change);

    let scope_id = use_stable_id(id.clone().map(Cow::Owned));
    let is_value_controlled = value.is_some();
    let is_values_controlled = values.is_some();
    let current_val = value_ctl.get();
    let current_vals = values_ctl.get();
    let current_open = open_ctl.get();

    let mut select_def = Select::new(ScopeHandle::root("select").child(scope_id))
        .with_mode(mode)
        .with_allow_deselect(allow_deselect)
        .with_value_controlled(is_value_controlled)
        .with_values_controlled(is_values_controlled)
        .with_value(current_val)
        .with_default_value(default_value)
        .with_values(current_vals)
        .with_open(current_open)
        .with_disabled(effective_disabled)
        .with_required(effective_required)
        .with_loop(loop_selection);

    if let Some(dvals) = default_values {
        select_def = select_def.with_default_values(dvals);
    }
    if let Some(n) = effective_name.clone() {
        select_def = select_def.with_name(n);
    }
    if let Some(f) = form.clone() {
        select_def = select_def.with_form(f);
    }
    if let Some(ac) = autocomplete.clone() {
        select_def = select_def.with_autocomplete(ac);
    }
    if let Some(it) = items {
        select_def = select_def.with_items(it);
    }

    let resolved_dir = use_direction_context(dir);
    let runtime = use_select_runtime_controllable(
        select_def,
        value_ctl,
        values_ctl,
        open_ctl,
        on_open_change_complete,
        resolved_dir,
    );
    runtime.set_root_invalid_override(root_invalid_override);
    runtime.set_invalid(effective_root_invalid);

    let (handle_focus_in, handle_focus_out) = use_composite_focus_out(
        vec![
            runtime.current_trigger_id(),
            runtime.current_content_id(),
        ],
        on_focus_out,
    );
    runtime.set_composite_focus_handlers(Some(handle_focus_in), Some(handle_focus_out));

    use_context_provider(|| SelectContext {
        runtime: runtime.clone(),
        name: effective_name.clone(),
    });

    let eff_req = runtime.select().is_required() || effective_required;
    let eff_form = form.or_else(|| runtime.select().form().map(str::to_owned));
    let eff_auto = autocomplete.or_else(|| runtime.select().autocomplete().map(str::to_owned));

    rsx! {
        {children}
        if let Some(input_name) = effective_name.or_else(|| runtime.select().name().map(str::to_owned)) {
            SelectHiddenInput {
                name: input_name,
                value: runtime.value(),
                values: if runtime.select().is_multiple() { Some(runtime.values()) } else { None },
                disabled: runtime.is_disabled() || effective_disabled,
                required: eff_req,
                read_only: effective_read_only,
                form: eff_form,
                autocomplete: eff_auto,
                focus_target_id: Some(runtime.current_trigger_id()),
                reset_epoch: runtime.reset_epoch(),
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectTriggerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
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

#[component]
pub fn SelectTrigger(props: SelectTriggerProps) -> Element {
    let SelectTriggerProps {
        id,
        disabled,
        invalid,
        aria_describedby,
        aria_labelledby,
        on_blur,
        on_focus,
        class,
        style,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<SelectContext>();
    let field_ctx = try_use_field_context();
    let control_ctx = try_use_field_control_context();

    let explicit_id = id.filter(|s| !s.trim().is_empty());
    let resolved_trigger_id = if let Some(ref explicit) = explicit_id {
        if let Some(ctrl_ctx) = control_ctx {
            if !ctrl_ctx.is_explicit_id_locked() {
                ctrl_ctx.sync_control_id(explicit.clone());
            }
        } else if let Some(f_ctx) = field_ctx
            && !f_ctx.is_fieldset()
        {
            f_ctx.set_primary_control_id(Some(explicit.clone()));
        }
        explicit.clone()
    } else if let Some(ctrl_ctx) = control_ctx {
        ctrl_ctx.control_id()
    } else if let Some(f_ctx) = field_ctx
        && !f_ctx.is_fieldset()
    {
        f_ctx.resolved_control_id()
    } else {
        ctx.runtime.default_trigger_id()
    };

    ctx.runtime.set_trigger_id(resolved_trigger_id.clone());
    let mut owned_trigger_id = use_hook(|| CopyValue::new(resolved_trigger_id.clone()));
    if owned_trigger_id
        .try_read()
        .is_ok_and(|cur| *cur != resolved_trigger_id)
    {
        owned_trigger_id.set(resolved_trigger_id.clone());
    }
    let drop_runtime = ctx.runtime.clone();
    use_drop(move || {
        let default_id = drop_runtime.default_trigger_id();
        if let Ok(owned) = owned_trigger_id.try_read()
            && drop_runtime
                .state
                .trigger_id
                .try_peek()
                .is_ok_and(|cur| *cur == *owned)
        {
            drop_runtime.set_trigger_id(default_id);
        }
    });

    let extracted = extract_aria_invalid_and_describedby(attributes);
    let (attr_labelledby, rest_attrs) = extract_aria_labelledby(extracted.rest);

    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let field_labelledby = if let Some(ctrl_ctx) = control_ctx {
        ctrl_ctx.mounted_label_id().or_else(|| {
            field_ctx
                .filter(|f| !f.is_fieldset())
                .and_then(|f| f.aria_labelledby())
        })
    } else {
        field_ctx.and_then(|f| f.aria_labelledby())
    };
    let field_read_only = field_ctx.is_some_and(|f| f.is_read_only());

    let fallback_invalid = ctx
        .runtime
        .root_invalid_override()
        .or_else(|| field_ctx.map(|f| f.is_invalid()));
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        invalid,
        extracted.aria_invalid,
        fallback_invalid,
        false,
        aria_describedby.as_deref(),
        extracted.aria_describedby.as_deref(),
        field_describedby.as_deref(),
    );
    let effective_labelledby = aria_labelledby
        .filter(|s| !s.trim().is_empty())
        .or(attr_labelledby)
        .or(field_labelledby);

    let mut on_blur_slot = use_hook(|| CopyValue::new(on_blur));
    on_blur_slot.set(on_blur);
    let mut on_focus_slot = use_hook(|| CopyValue::new(on_focus));
    on_focus_slot.set(on_focus);

    let attrs = ctx
        .runtime
        .trigger_attributes()
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby)
        .with_aria_labelledby(effective_labelledby);

    let is_dis = disabled || attrs.is_disabled();
    let trigger_id = attrs.id().to_owned();
    let role_str = attrs.role();
    let aria_haspopup = attrs.aria_haspopup();
    let aria_expanded = attrs.aria_expanded();
    let aria_controls = attrs.aria_controls().map(str::to_owned);
    let aria_required = attrs.aria_required();
    let aria_invalid = attrs.aria_invalid();
    let aria_describedby_val = attrs.aria_describedby().map(str::to_owned);
    let aria_labelledby_val = attrs.aria_labelledby().map(str::to_owned);
    let data_state_str = attrs.data_state_str();
    let data_invalid = attrs.data_invalid();
    let data_placeholder_str = attrs.data_placeholder_str();
    let tabindex_str = if is_dis { "-1" } else { "0" };

    let mut base = Vec::with_capacity(24);
    base.push(Attribute::new("type", "button", None, false));
    base.push(Attribute::new("id", trigger_id, None, false));
    base.push(Attribute::new("role", role_str, None, false));
    base.push(Attribute::new("aria-haspopup", aria_haspopup, None, false));
    base.push(Attribute::new("aria-expanded", aria_expanded, None, false));
    push_attr_opt(&mut base, "aria-controls", aria_controls);
    push_attr_opt(&mut base, "aria-disabled", is_dis.then_some("true"));
    push_attr_opt(&mut base, "aria-required", aria_required);
    push_attr_opt(&mut base, "aria-invalid", aria_invalid);
    push_attr_opt(
        &mut base,
        "aria-readonly",
        field_read_only.then_some("true"),
    );
    push_attr_opt(&mut base, "aria-describedby", aria_describedby_val);
    push_attr_opt(&mut base, "aria-labelledby", aria_labelledby_val);
    base.push(Attribute::new("tabindex", tabindex_str, None, false));
    if is_dis {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("data-state", data_state_str, None, false));
    push_bool_data_attr(&mut base, "data-disabled", is_dis);
    push_attr_opt(&mut base, "data-invalid", data_invalid);
    push_bool_data_attr(&mut base, "data-readonly", field_read_only);
    base.push(Attribute::new(
        "data-placeholder",
        data_placeholder_str,
        None,
        false,
    ));

    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            move |_evt: PointerEvent| {
                if !is_dis && runtime.is_open() {
                    runtime.mark_pointer_dismiss();
                }
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            move |_evt: MouseEvent| {
                if !is_dis {
                    runtime.toggle_open();
                }
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onkeydown",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            move |evt: KeyboardEvent| {
                if !is_dis {
                    runtime.handle_trigger_keydown(&evt);
                }
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onfocus",
        Some(EventHandler::new(move |evt: FocusEvent| {
            if let Some(handler) = on_focus_slot.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onblur",
        Some(EventHandler::new(move |evt: FocusEvent| {
            if let Some(handler) = on_blur_slot.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onfocusin",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            move |evt: FocusEvent| {
                runtime.notify_composite_focus_in(evt);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onfocusout",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            move |evt: FocusEvent| {
                runtime.notify_composite_focus_out(evt);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            move |evt: MountedEvent| {
                runtime.capture_trigger_mount(evt.data());
            }
        })),
    );

    let user = collect_user_attrs(class, style, rest_attrs);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}

pub fn humanize_label(val: &str) -> String {
    if val.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = val
        .split(|c: char| c == '-' || c == '_')
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                format!("{}{}", first.to_uppercase(), chars.as_str())
            } else {
                String::new()
            }
        })
        .collect();

    if parts.is_empty() {
        val.to_string()
    } else {
        parts.join(" ")
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectValueProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub placeholder: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn SelectValue(props: SelectValueProps) -> Element {
    let SelectValueProps {
        id,
        placeholder,
        class,
        style,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<SelectContext>();

    let text_content = match ctx.runtime.select().mode() {
        SelectMode::Single => {
            if let Some(v) = ctx.runtime.value() {
                if v.is_empty() {
                    String::new()
                } else {
                    ctx.runtime
                        .item_label(&v)
                        .filter(|l| !l.is_empty() && l != &v)
                        .unwrap_or_else(|| humanize_label(&v))
                }
            } else {
                String::new()
            }
        }
        SelectMode::Multiple => {
            let vals = ctx.runtime.values();
            if vals.is_empty() {
                String::new()
            } else {
                vals.iter()
                    .map(|v| {
                        ctx.runtime
                            .item_label(v)
                            .filter(|l| !l.is_empty() && l != v)
                            .unwrap_or_else(|| humanize_label(v))
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        }
    };

    let is_empty = text_content.is_empty();
    let display_text = if is_empty {
        placeholder.unwrap_or_default()
    } else {
        text_content
    };

    let mut base = Vec::with_capacity(2);
    push_attr_opt(&mut base, "id", id);
    base.push(Attribute::new(
        "data-placeholder",
        if is_empty { "true" } else { "false" },
        None,
        false,
    ));

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    if let Some(c) = children {
        return rsx! {
            span {
                ..merged,
                {c}
            }
        };
    }

    rsx! {
        span {
            ..merged,
            "{display_text}"
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectIconProps {
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
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn SelectIcon(props: SelectIconProps) -> Element {
    let SelectIconProps {
        id,
        class,
        style,
        child,
        attributes,
        children,
    } = props;
    let mut base = Vec::with_capacity(2);
    push_attr_opt(&mut base, "id", id);
    base.push(Attribute::new("aria-hidden", "true", None, false));

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    if let Some(c) = children {
        return rsx! {
            span {
                ..merged,
                {c}
            }
        };
    }

    rsx! {
        span {
            ..merged,
            "▼"
        }
    }
}

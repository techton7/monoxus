use std::borrow::Cow;

use dioxus::core::AttributeValue;
use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    browser::focus::use_composite_focus_out,
    compose::{
        extract_aria_invalid_and_describedby, merge_global_attributes,
        resolve_invalid_and_describedby,
    },
    shared::{ScopeHandle, use_stable_id},
};

use super::{
    hidden_input::SelectHiddenInput,
    runtime::{SelectRuntime, use_select_runtime_full},
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
    #[props(default)]
    pub runtime: Option<SelectRuntime>,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub mode: Option<SelectMode>,
    #[props(default = false)]
    pub multiple: bool,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub values: Option<Vec<String>>,
    #[props(default)]
    pub default_values: Option<Vec<String>>,
    #[props(default = false)]
    pub open: bool,
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
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default)]
    pub autocomplete: Option<String>,
    #[props(default)]
    pub items: Option<Vec<SelectItemData>>,
    #[props(default = false)]
    pub loop_selection: bool,
    #[props(default)]
    pub dir: Option<crate::foundation::shared::Direction>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn SelectRoot(props: SelectRootProps) -> Element {
    let SelectRootProps {
        runtime,
        id,
        class,
        style,
        mode,
        multiple,
        value,
        default_value,
        values,
        default_values,
        open,
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
        attributes,
        children,
    } = props;

    let field_ctx = try_use_field_context();
    let effective_name = name.clone().or_else(|| field_ctx.and_then(|f| f.name()));
    let effective_disabled = disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_required = required || field_ctx.is_some_and(|f| f.is_required());

    let extracted_root = extract_aria_invalid_and_describedby(attributes);
    let root_invalid_override = invalid.or(extracted_root.aria_invalid);
    let effective_root_invalid = root_invalid_override
        .or_else(|| field_ctx.map(|f| f.is_invalid()))
        .unwrap_or(false);

    let fallback_scope_id = use_stable_id(id.clone().map(Cow::Owned));
    let is_value_controlled = value.is_some();
    let is_values_controlled = values.is_some();
    let fallback_val = value.clone().or(default_value.clone());

    let mut fallback_select = Select::new(ScopeHandle::root("select").child(fallback_scope_id))
        .with_value_controlled(is_value_controlled)
        .with_values_controlled(is_values_controlled)
        .with_value(fallback_val)
        .with_default_value(default_value)
        .with_open(open)
        .with_allow_deselect(allow_deselect)
        .with_disabled(effective_disabled)
        .with_required(effective_required)
        .with_loop(loop_selection);

    if let Some(m) = mode {
        fallback_select = fallback_select.with_mode(m);
    } else if multiple {
        fallback_select = fallback_select.with_multiple(true);
    }

    if let Some(vals) = values.clone() {
        fallback_select = fallback_select.with_values(vals);
    }
    if let Some(dvals) = default_values {
        fallback_select = fallback_select.with_default_values(dvals);
    }
    if let Some(n) = effective_name.clone() {
        fallback_select = fallback_select.with_name(n);
    }
    if let Some(f) = form.clone() {
        fallback_select = fallback_select.with_form(f);
    }
    if let Some(ac) = autocomplete.clone() {
        fallback_select = fallback_select.with_autocomplete(ac);
    }
    if let Some(it) = items {
        fallback_select = fallback_select.with_items(it);
    }

    let val_change = on_value_change.map(|h| move |v| h.call(v));
    let vals_change = on_values_change.map(|h| move |v| h.call(v));
    let open_change = on_open_change.map(|h| move |o| h.call(o));
    let open_complete_change = on_open_change_complete.map(|h| move |o| h.call(o));

    let default_runtime = use_select_runtime_full(
        fallback_select,
        val_change,
        vals_change,
        open_change,
        open_complete_change,
    );

    let runtime = runtime.unwrap_or(default_runtime);
    runtime.set_root_invalid_override(root_invalid_override);
    runtime.set_invalid(effective_root_invalid);

    let (handle_focus_in, handle_focus_out) = use_composite_focus_out(
        vec![
            runtime.current_trigger_id(),
            runtime.relationships().content_id().to_owned(),
        ],
        on_focus_out,
    );
    runtime.set_composite_focus_handlers(Some(handle_focus_in), Some(handle_focus_out));

    use_context_provider(|| SelectContext {
        runtime: runtime.clone(),
        name: effective_name.clone(),
    });

    let attrs = runtime.root_attributes();
    let resolved_dir = crate::foundation::shared::use_direction_context(dir);
    let eff_req = runtime.select().is_required() || effective_required;
    let eff_form = form.or_else(|| runtime.select().form().map(str::to_owned));
    let eff_auto = autocomplete.or_else(|| runtime.select().autocomplete().map(str::to_owned));

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), extracted_root.rest);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            dir: "{resolved_dir.as_str()}",
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": attrs.data_disabled(),
            "data-invalid": attrs.data_invalid(),
            onfocusin: handle_focus_in,
            onfocusout: handle_focus_out,
            ..merged.rest,
            {children}
            if let Some(input_name) = effective_name.or_else(|| runtime.select().name().map(str::to_owned)) {
                SelectHiddenInput {
                    name: input_name,
                    value: runtime.value(),
                    values: if runtime.select().is_multiple() { Some(runtime.values()) } else { None },
                    disabled: runtime.is_disabled() || effective_disabled,
                    required: eff_req,
                    form: eff_form,
                    autocomplete: eff_auto,
                    focus_target_id: Some(runtime.current_trigger_id()),
                    reset_epoch: runtime.reset_epoch(),
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn SelectTrigger(props: SelectTriggerProps) -> Element {
    let SelectTriggerProps {
        id,
        invalid,
        aria_describedby,
        aria_labelledby,
        on_blur,
        on_focus,
        class,
        style,
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
        ctx.runtime.relationships().trigger_id().to_owned()
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
        let default_id = drop_runtime.relationships().trigger_id().to_owned();
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
    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), rest_attrs);

    rsx! {
        button {
            r#type: "button",
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            aria_haspopup: "{attrs.aria_haspopup()}",
            aria_expanded: "{attrs.aria_expanded()}",
            aria_controls: attrs.aria_controls(),
            aria_disabled: attrs.aria_disabled(),
            aria_required: attrs.aria_required(),
            aria_invalid: attrs.aria_invalid(),
            aria_describedby: attrs.aria_describedby(),
            aria_labelledby: attrs.aria_labelledby(),
            tabindex: "{attrs.tabindex()}",
            disabled: attrs.is_disabled(),
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": attrs.data_disabled(),
            "data-invalid": attrs.data_invalid(),
            "data-placeholder": attrs.data_placeholder_str(),
            onpointerdown: {
                let runtime = ctx.runtime.clone();
                let is_dis = attrs.is_disabled();
                move |_| {
                    if !is_dis && runtime.is_open() {
                        runtime.mark_pointer_dismiss();
                    }
                }
            },
            onclick: {
                let runtime = ctx.runtime.clone();
                move |_| {
                    runtime.toggle_open();
                }
            },
            onkeydown: {
                let runtime = ctx.runtime.clone();
                move |evt| {
                    runtime.handle_trigger_keydown(&evt);
                }
            },
            onfocus: move |evt| {
                if let Some(handler) = on_focus_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            onblur: move |evt| {
                if let Some(handler) = on_blur_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            onmounted: {
                let runtime = ctx.runtime.clone();
                move |evt: Event<MountedData>| {
                    runtime.capture_trigger_mount(evt.data());
                }
            },
            ..merged.rest,
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
    #[props(default)]
    pub placeholder: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn SelectValue(props: SelectValueProps) -> Element {
    let SelectValueProps {
        placeholder,
        class,
        style,
        attributes,
        children,
    } = props;
    let ctx = use_context::<SelectContext>();

    let text_content = match ctx.runtime.select().mode() {
        SelectMode::Single { .. } => {
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

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    if let Some(c) = children {
        return rsx! {
            span {
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                "data-placeholder": if is_empty { "true" } else { "false" },
                ..merged.rest,
                {c}
            }
        };
    }

    rsx! {
        span {
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-placeholder": if is_empty { "true" } else { "false" },
            ..merged.rest,
            "{display_text}"
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectIconProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn SelectIcon(props: SelectIconProps) -> Element {
    let SelectIconProps {
        class,
        style,
        attributes,
        children,
    } = props;
    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    if let Some(c) = children {
        return rsx! {
            span {
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                aria_hidden: "true",
                ..merged.rest,
                {c}
            }
        };
    }

    rsx! {
        span {
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            aria_hidden: "true",
            ..merged.rest,
            "▼"
        }
    }
}

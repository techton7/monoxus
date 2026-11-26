use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::{ScopeHandle, use_stable_id},
};

use super::{
    hidden_input::SelectHiddenInput,
    runtime::{SelectRuntime, use_select_runtime_full},
    state::Select,
    types::{SelectContext, SelectItemData, SelectMode},
};

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
    #[props(default = false)]
    pub allow_deselect: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
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
        allow_deselect,
        disabled,
        required,
        name,
        form,
        autocomplete,
        items,
        loop_selection,
        dir,
        attributes,
        children,
    } = props;
    let fallback_scope_id = use_stable_id(id.clone().map(Cow::Owned));
    let fallback_val = value.clone().or(default_value.clone());
    let fallback_vals = values
        .clone()
        .or(default_values.clone())
        .unwrap_or_default();

    let mut fallback_select =
        Select::new(ScopeHandle::root("select").child(fallback_scope_id))
        .with_value(fallback_val)
        .with_default_value(default_value)
        .with_open(open)
        .with_allow_deselect(allow_deselect)
        .with_disabled(disabled)
        .with_required(required)
        .with_loop(loop_selection);

    if let Some(m) = mode {
        fallback_select = fallback_select.with_mode(m);
    } else if multiple {
        fallback_select = fallback_select.with_multiple(true);
    }

    if !fallback_vals.is_empty() {
        fallback_select = fallback_select.with_values(fallback_vals.clone());
    }
    if let Some(dvals) = default_values {
        fallback_select = fallback_select.with_default_values(dvals);
    }
    if let Some(n) = name.clone() {
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

    use_context_provider(|| SelectContext {
        runtime: runtime.clone(),
        name: name.clone(),
    });

    let attrs = runtime.root_attributes();
    let resolved_dir = crate::foundation::shared::use_direction_context(dir);
    let eff_req = runtime.select().is_required() || required;
    let eff_form = form.or_else(|| runtime.select().form().map(str::to_owned));
    let eff_auto = autocomplete.or_else(|| runtime.select().autocomplete().map(str::to_owned));

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            dir: "{resolved_dir.as_str()}",
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": attrs.data_disabled(),
            ..merged.rest,
            {children}
            if let Some(input_name) = name.or_else(|| runtime.select().name().map(str::to_owned)) {
                SelectHiddenInput {
                    name: input_name,
                    value: runtime.value(),
                    values: if runtime.select().is_multiple() { Some(runtime.values()) } else { None },
                    disabled: runtime.is_disabled() || disabled,
                    required: eff_req,
                    form: eff_form,
                    autocomplete: eff_auto,
                    focus_target_id: Some(runtime.relationships().trigger_id().to_owned()),
                    reset_epoch: runtime.reset_epoch(),
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectTriggerProps {
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
        class,
        style,
        attributes,
        children,
    } = props;
    let ctx = use_context::<SelectContext>();
    let attrs = ctx.runtime.trigger_attributes();
    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

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
            tabindex: "{attrs.tabindex()}",
            disabled: attrs.is_disabled(),
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": attrs.data_disabled(),
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

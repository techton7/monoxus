use dioxus::prelude::*;

use crate::foundation::shared::ScopeHandle;

use super::{
    hidden_input::SelectHiddenInput,
    runtime::{SelectRuntime, use_select_runtime_full},
    state::Select,
    types::{SelectContext, SelectItemData, SelectMode},
};

#[component]
pub fn SelectRoot(
    #[props(default)] runtime: Option<SelectRuntime>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] mode: Option<SelectMode>,
    #[props(default = false)] multiple: bool,
    #[props(default)] value: Option<String>,
    #[props(default)] default_value: Option<String>,
    #[props(default)] values: Option<Vec<String>>,
    #[props(default)] default_values: Option<Vec<String>>,
    #[props(default = false)] open: bool,
    #[props(default)] on_value_change: Option<EventHandler<Option<String>>>,
    #[props(default)] on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] on_open_change: Option<EventHandler<bool>>,
    #[props(default)] on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default = false)] allow_deselect: bool,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] required: bool,
    #[props(default)] name: Option<String>,
    #[props(default)] form: Option<String>,
    #[props(default)] autocomplete: Option<String>,
    #[props(default)] items: Option<Vec<SelectItemData>>,
    #[props(default = false)] loop_selection: bool,
    children: Element,
) -> Element {
    let fallback_scope_id = id.clone().unwrap_or_else(|| "select".to_string());
    let fallback_val = value.clone().or(default_value.clone());
    let fallback_vals = values
        .clone()
        .or(default_values.clone())
        .unwrap_or_default();

    let mut fallback_select = Select::new(ScopeHandle::root("select").child(fallback_scope_id))
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
    let eff_req = runtime.select().is_required() || required;
    let eff_form = form.or_else(|| runtime.select().form().map(str::to_owned));
    let eff_auto = autocomplete.or_else(|| runtime.select().autocomplete().map(str::to_owned));

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
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
                }
            }
        }
    }
}

#[component]
pub fn SelectTrigger(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let attrs = ctx.runtime.trigger_attributes();

    rsx! {
        button {
            r#type: "button",
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            aria_haspopup: "{attrs.aria_haspopup()}",
            aria_expanded: "{attrs.aria_expanded()}",
            aria_controls: "{attrs.aria_controls()}",
            aria_disabled: attrs.aria_disabled(),
            aria_required: attrs.aria_required(),
            tabindex: "{attrs.tabindex()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            "data-placeholder": attrs.data_placeholder_str(),
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

#[component]
pub fn SelectValue(
    #[props(default)] placeholder: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Option<Element>,
) -> Element {
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

    if let Some(c) = children {
        return rsx! {
            span {
                class: class.as_deref().unwrap_or_default(),
                style: style.as_deref().unwrap_or_default(),
                "data-placeholder": if is_empty { "true" } else { "false" },
                {c}
            }
        };
    }

    rsx! {
        span {
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-placeholder": if is_empty { "true" } else { "false" },
            "{display_text}"
        }
    }
}

#[component]
pub fn SelectIcon(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Option<Element>,
) -> Element {
    if let Some(c) = children {
        return rsx! {
            span {
                class: class.as_deref().unwrap_or_default(),
                style: style.as_deref().unwrap_or_default(),
                aria_hidden: "true",
                {c}
            }
        };
    }

    rsx! {
        span {
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            aria_hidden: "true",
            "▼"
        }
    }
}

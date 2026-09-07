use dioxus::prelude::*;

use crate::foundation::browser::scroll_element_into_view_nearest;

use super::types::SelectContext;

#[component]
pub fn SelectGroup(
    group_key: String,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let rels = ctx.runtime.relationships();
    let group_id = rels.group_id(&group_key);
    let label_id = rels.label_id(&group_key);

    rsx! {
        div {
            id: "{group_id}",
            role: "group",
            aria_labelledby: "{label_id}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn SelectLabel(
    group_key: String,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let rels = ctx.runtime.relationships();
    let label_id = rels.label_id(&group_key);

    rsx! {
        div {
            id: "{label_id}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn SelectItem(
    value: String,
    #[props(default)] text: Option<String>,
    #[props(default = false)] disabled: bool,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] on_highlight: Option<EventHandler<()>>,
    #[props(default)] on_unhighlight: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let rels = ctx.runtime.relationships();
    let item_id = rels.item_id(&value);
    let text_val = text.clone().unwrap_or_else(|| value.clone());

    // Register item
    {
        let rt = ctx.runtime.clone();
        let val_c = value.clone();
        let txt_c = text_val.clone();
        use_effect(move || {
            rt.register_item(&val_c, &txt_c, disabled);
        });
    }

    let is_hl = ctx.runtime.highlighted_value().as_deref() == Some(&value);
    let attrs = ctx.runtime.item_attributes(&value, is_hl, disabled);
    let is_sel = ctx.runtime.is_selected(&value);

    let mut prev_hl = use_signal(|| false);
    use_effect(use_reactive((&is_hl,), {
        let hl_cb = on_highlight;
        let unhl_cb = on_unhighlight;
        let cid = item_id.clone();
        move |(current_hl,)| {
            let was_hl = *prev_hl.peek();
            if current_hl && !was_hl {
                prev_hl.set(true);
                if let Some(ref cb) = hl_cb {
                    cb.call(());
                }
                // Synchronize scroll-into-view with VDOM commit!
                scroll_element_into_view_nearest(&cid);
            } else if !current_hl && was_hl {
                prev_hl.set(false);
                if let Some(ref cb) = unhl_cb {
                    cb.call(());
                }
            }
        }
    }));

    let label_str = text.unwrap_or_else(|| value.clone());

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            aria_selected: if is_sel { "true" } else { "false" },
            aria_disabled: attrs.aria_disabled(),
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": if is_sel { "checked" } else { "unchecked" },
            "data-highlighted": if is_hl { "true" } else { "false" },
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            "data-value": "{value}",
            "data-label": "{label_str}",
            "data-selected": if is_sel { "true" } else { "false" },
            onclick: {
                let runtime = ctx.runtime.clone();
                let v = value.clone();
                let is_dis = attrs.is_disabled();
                move |evt| {
                    if is_dis {
                        return;
                    }
                    evt.stop_propagation();
                    runtime.select_item(&v);
                }
            },
            onpointermove: {
                let runtime = ctx.runtime.clone();
                let v = value.clone();
                let is_dis = attrs.is_disabled();
                move |_| {
                    if is_dis {
                        return;
                    }
                    runtime.set_highlighted(Some(v.clone()));
                }
            },
            {children}
        }
    }
}

#[component]
pub fn SelectItemText(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        span {
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn SelectItemIndicator(
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
            "✓"
        }
    }
}

#[component]
pub fn SelectSeparator(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
) -> Element {
    rsx! {
        div {
            role: "separator",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or("height: 1px; background-color: #e2e8f0; margin: 4px 0;"),
            aria_hidden: "true",
        }
    }
}

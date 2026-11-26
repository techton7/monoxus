use std::borrow::Cow;

use dioxus::prelude::*;

use crate::accordion::attrs::format_accordion_css_vars;
use crate::collapsible::attrs::merge_disclosure_styles;
use crate::foundation::{
    overlay::use_presence,
    roving_focus::key_to_roving_str,
    shared::{
        ScopeHandle, data_disabled_attr, use_direction_context, use_disclosure_size, use_stable_id,
    },
};

use super::{
    runtime::{AccordionRuntime, use_accordion_runtime},
    state::Accordion,
    types::{AccordionDirection, AccordionMode, AccordionOrientation},
};

#[derive(Clone)]
pub struct AccordionContext {
    pub runtime: AccordionRuntime,
}

#[component]
pub fn AccordionRoot(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] mode: AccordionMode,
    #[props(default)] orientation: AccordionOrientation,
    #[props(default)] dir: Option<AccordionDirection>,
    #[props(default = true)] loop_focus: bool,
    #[props(default = false)] disabled: bool,
    #[props(default)] default_value: Option<String>,
    #[props(default)] default_values: Option<Vec<String>>,
    #[props(default)] on_value_change: Option<EventHandler<Vec<String>>>,
    children: Element,
) -> Element {
    let resolved_dir = use_direction_context(dir.map(Into::into));
    let accordion_dir = AccordionDirection::from(resolved_dir);
    let scope_id = use_stable_id(id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("accordion").child(scope_id);

    let mut accordion = Accordion::new(scope, mode)
        .with_orientation(orientation)
        .with_direction(accordion_dir)
        .with_loop_focus(loop_focus)
        .with_disabled(disabled);

    if let Some(vals) = default_values {
        accordion = accordion.with_values(vals);
    } else if let Some(val) = default_value {
        accordion = accordion.with_value(val);
    }

    let change_handler = on_value_change.map(|handler| move |vals| handler.call(vals));
    let runtime = use_accordion_runtime(accordion, change_handler);

    use_context_provider(|| AccordionContext {
        runtime: runtime.clone(),
    });

    let attrs = runtime.root();

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            dir: "{resolved_dir.as_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": data_disabled_attr(attrs.is_disabled()),
            {children}
        }
    }
}

#[derive(Clone)]
pub struct AccordionItemContext {
    pub value: String,
    pub disabled: bool,
}

#[component]
pub fn AccordionItem(
    value: String,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default = false)] disabled: bool,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_value = value.clone();

    use_context_provider(|| AccordionItemContext {
        value: item_value.clone(),
        disabled,
    });

    use_hook(|| {
        ctx.runtime.register_item(&item_value, disabled);
    });

    use_effect(use_reactive((&item_value, &disabled), {
        let runtime = ctx.runtime.clone();
        move |(val, dis)| {
            runtime.register_item(&val, dis);
        }
    }));

    let drop_val = value.clone();
    let drop_runtime = ctx.runtime.clone();
    use_drop(move || {
        drop_runtime.unregister_item(&drop_val);
    });

    let attrs = ctx.runtime.item(&value, disabled);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            "data-disabled": data_disabled_attr(attrs.is_disabled()),
            {children}
        }
    }
}

#[component]
pub fn AccordionHeader(
    #[props(default = 3)] level: u8,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let clamped = level.clamp(1, 6);
    let attrs = ctx
        .runtime
        .header_with_level(&item_ctx.value, item_ctx.disabled, clamped);

    macro_rules! render_heading {
        ($tag:ident) => {
            rsx! {
                $tag {
                    id: "{attrs.id()}",
                    class: class.as_deref().unwrap_or_default(),
                    style: style.as_deref().unwrap_or_default(),
                    role: "{attrs.role()}",
                    "aria-level": "{clamped}",
                    "data-heading-level": "{clamped}",
                    "data-state": "{attrs.data_state_str()}",
                    "data-orientation": "{attrs.data_orientation()}",
                    "data-value": "{attrs.data_value()}",
                    "data-disabled": data_disabled_attr(attrs.is_disabled()),
                    {children}
                }
            }
        };
    }

    match clamped {
        1 => render_heading!(h1),
        2 => render_heading!(h2),
        3 => render_heading!(h3),
        4 => render_heading!(h4),
        5 => render_heading!(h5),
        _ => render_heading!(h6),
    }
}

#[component]
pub fn AccordionTrigger(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let attrs = ctx.runtime.trigger(&item_ctx.value, item_ctx.disabled);
    let trig_id = id.unwrap_or_else(|| attrs.id().to_string());

    let val = item_ctx.value.clone();
    let trig_id_for_effect = trig_id.clone();
    use_effect(use_reactive((&val, &trig_id_for_effect), {
        let runtime = ctx.runtime.clone();
        move |(v, el_id)| {
            runtime.register_trigger_id(&v, &el_id);
        }
    }));

    let click_runtime = ctx.runtime.clone();
    let click_val = item_ctx.value.clone();
    let focus_runtime = ctx.runtime.clone();
    let focus_val = item_ctx.value.clone();
    let focus_disabled = item_ctx.disabled;
    let key_runtime = ctx.runtime.clone();

    rsx! {
        button {
            r#type: "button",
            id: "{trig_id}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "aria-expanded": "{attrs.aria_expanded()}",
            "aria-controls": attrs.aria_controls(),
            "aria-disabled": attrs.aria_disabled(),
            tabindex: attrs.tabindex(),
            disabled: attrs.is_disabled(),
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            "data-disabled": data_disabled_attr(attrs.is_disabled()),
            onfocus: move |_| {
                if !focus_disabled {
                    focus_runtime
                        .roving()
                        .set_current_tab_stop(Some(focus_val.clone()));
                }
            },
            onclick: move |_| {
                click_runtime.toggle_item(&click_val);
            },
            onkeydown: move |evt: KeyboardEvent| {
                let event_key = evt.key();
                if let Some(key_str) = key_to_roving_str(&event_key)
                    && Accordion::is_navigation_key_for_orientation(
                        key_runtime.accordion().orientation(),
                        key_str,
                    )
                {
                    evt.prevent_default();
                    evt.stop_propagation();
                    key_runtime.navigate_key(key_str);
                }
            },
            {children}
        }
    }
}

#[component]
pub fn AccordionContent(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default = false)] force_mount: bool,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let attrs = ctx.runtime.content(&item_ctx.value);

    let is_open = ctx.runtime.is_open(&item_ctx.value);
    let presence = use_presence(is_open, attrs.id(), true, None);
    let disclosure = use_disclosure_size(attrs.id(), is_open, presence);

    let should_render = is_open || presence.should_render() || force_mount;
    if should_render {
        ctx.runtime.register_content_mounted(&item_ctx.value);
    } else {
        ctx.runtime.unregister_content_mounted(&item_ctx.value);
    }

    use_drop({
        let runtime = ctx.runtime.clone();
        let drop_val = item_ctx.value.clone();
        move || {
            runtime.unregister_content_mounted(&drop_val);
        }
    });

    let hidden = (!should_render).then_some(true);
    let css_vars = disclosure.size().map(format_accordion_css_vars);
    let effective_style = merge_disclosure_styles(
        style.as_deref(),
        disclosure.guard_style(),
        css_vars.as_deref(),
    );

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: effective_style.as_deref().unwrap_or_default(),
            role: "{attrs.role()}",
            "aria-labelledby": "{attrs.aria_labelledby()}",
            hidden: hidden,
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            "data-disabled": data_disabled_attr(item_ctx.disabled),
            onmounted: move |_| disclosure.on_mounted(),
            ontransitionend: move |_| disclosure.on_motion_end(),
            onanimationend: move |_| disclosure.on_motion_end(),
            {should_render.then_some(children)}
        }
    }
}

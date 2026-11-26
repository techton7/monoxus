use std::borrow::Cow;

use dioxus::prelude::*;

use crate::accordion::attrs::format_accordion_css_vars;
use crate::collapsible::attrs::merge_disclosure_styles;
use crate::foundation::{
    compose::merge_global_attributes,
    overlay::use_presence,
    roving_focus::key_to_roving_str,
    shared::{
        ScopeHandle, data_disabled_attr, use_direction_context, use_disclosure_size, use_stable_id,
    },
    state::sync_signal,
};

use super::{
    runtime::{AccordionRuntime, use_accordion_runtime_with_control},
    state::Accordion,
    types::{AccordionDirection, AccordionMode, AccordionOrientation},
};

#[derive(Clone)]
pub struct AccordionContext {
    pub runtime: AccordionRuntime,
}

#[derive(Clone, PartialEq, Props)]
pub struct AccordionRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub mode: AccordionMode,
    #[props(default)]
    pub orientation: AccordionOrientation,
    #[props(default)]
    pub dir: Option<AccordionDirection>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub default_values: Option<Vec<String>>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Vec<String>>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn AccordionRoot(props: AccordionRootProps) -> Element {
    let AccordionRootProps {
        id,
        class,
        style,
        mode,
        orientation,
        dir,
        loop_focus,
        disabled,
        default_value,
        default_values,
        on_value_change,
        attributes,
        children,
    } = props;
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
    let runtime = use_accordion_runtime_with_control(accordion, false, change_handler);

    use_context_provider(|| AccordionContext {
        runtime: runtime.clone(),
    });

    let attrs = runtime.root();
    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            dir: "{resolved_dir.as_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": data_disabled_attr(attrs.is_disabled()),
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, Copy)]
pub struct AccordionItemContext {
    pub value: Signal<String>,
    pub disabled: Signal<bool>,
}

#[derive(Clone, PartialEq, Props)]
pub struct AccordionItemProps {
    pub value: String,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn AccordionItem(props: AccordionItemProps) -> Element {
    let AccordionItemProps {
        value,
        class,
        style,
        disabled,
        attributes,
        children,
    } = props;
    let ctx = use_context::<AccordionContext>();

    let mut value_sig = use_signal(|| value.clone());
    sync_signal(&mut value_sig, value.clone());
    let mut disabled_sig = use_signal(|| disabled);
    sync_signal(&mut disabled_sig, disabled);

    use_context_provider(|| AccordionItemContext {
        value: value_sig,
        disabled: disabled_sig,
    });

    let mut tracked_val = use_hook(|| CopyValue::new(value.clone()));

    use_hook(|| {
        ctx.runtime.register_item(&value, disabled);
    });

    use_effect({
        let runtime = ctx.runtime.clone();
        move || {
            let val = value_sig.read().clone();
            let dis = *disabled_sig.read();
            let _root_disabled = runtime.is_disabled();
            let prev = tracked_val
                .try_read()
                .map(|g| g.clone())
                .unwrap_or_default();
            if prev != val {
                runtime.unregister_item(&prev);
                tracked_val.set(val.clone());
            }
            runtime.register_item(&val, dis);
        }
    });

    let drop_runtime = ctx.runtime.clone();
    use_drop(move || {
        if let Ok(val) = tracked_val.try_read() {
            drop_runtime.unregister_item(&val);
        }
    });

    let attrs = ctx.runtime.item(&value, disabled);
    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            "data-disabled": data_disabled_attr(attrs.is_disabled()),
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AccordionHeaderProps {
    #[props(default = 3)]
    pub level: u8,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn AccordionHeader(props: AccordionHeaderProps) -> Element {
    let AccordionHeaderProps {
        level,
        class,
        style,
        attributes,
        children,
    } = props;
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let item_value = item_ctx.value.read().clone();
    let item_disabled = *item_ctx.disabled.read();
    let clamped = level.clamp(1, 6);
    let attrs = ctx
        .runtime
        .header_with_level(&item_value, item_disabled, clamped);
    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    macro_rules! render_heading {
        ($tag:ident) => {
            rsx! {
                $tag {
                    id: "{attrs.id()}",
                    class: merged.class.as_deref(),
                    style: merged.style.as_deref(),
                    role: "{attrs.role()}",
                    "aria-level": "{clamped}",
                    "data-heading-level": "{clamped}",
                    "data-state": "{attrs.data_state_str()}",
                    "data-orientation": "{attrs.data_orientation()}",
                    "data-value": "{attrs.data_value()}",
                    "data-disabled": data_disabled_attr(attrs.is_disabled()),
                    ..merged.rest,
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

#[derive(Clone, PartialEq, Props)]
pub struct AccordionTriggerProps {
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

#[component]
pub fn AccordionTrigger(props: AccordionTriggerProps) -> Element {
    let AccordionTriggerProps {
        id,
        class,
        style,
        attributes,
        children,
    } = props;
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let item_value = item_ctx.value.read().clone();
    let item_disabled = *item_ctx.disabled.read();
    let attrs = ctx.runtime.trigger(&item_value, item_disabled);
    let trig_id = id.unwrap_or_else(|| attrs.id().to_string());

    let trig_id_for_effect = trig_id.clone();
    use_effect(use_reactive((&item_value, &trig_id_for_effect), {
        let runtime = ctx.runtime.clone();
        move |(v, el_id)| {
            runtime.register_trigger_id(&v, &el_id);
        }
    }));

    let click_runtime = ctx.runtime.clone();
    let focus_runtime = ctx.runtime.clone();
    let key_runtime = ctx.runtime.clone();

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        button {
            r#type: "button",
            id: "{trig_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
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
                if !(*item_ctx.disabled.peek() || focus_runtime.is_disabled_peek()) {
                    let focus_val = item_ctx.value.peek().clone();
                    focus_runtime
                        .roving()
                        .set_current_tab_stop(Some(focus_val));
                }
            },
            onclick: move |_| {
                if *item_ctx.disabled.peek() || click_runtime.is_disabled_peek() {
                    return;
                }
                let click_val = item_ctx.value.peek().clone();
                click_runtime.toggle_item(&click_val);
            },
            onkeydown: move |evt: KeyboardEvent| {
                if *item_ctx.disabled.peek() || key_runtime.is_disabled_peek() {
                    return;
                }
                let event_key = evt.key();
                if let Some(key_str) = key_to_roving_str(&event_key)
                    && Accordion::is_navigation_key_for_orientation(
                        key_runtime.orientation_peek(),
                        key_str,
                    )
                {
                    evt.prevent_default();
                    evt.stop_propagation();
                    key_runtime.navigate_key(key_str);
                }
            },
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AccordionContentProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn AccordionContent(props: AccordionContentProps) -> Element {
    let AccordionContentProps {
        class,
        style,
        force_mount,
        attributes,
        children,
    } = props;
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let item_value = item_ctx.value.read().clone();
    let item_disabled = *item_ctx.disabled.read();
    let attrs = ctx
        .runtime
        .content_with_disabled(&item_value, item_disabled);

    let is_open = ctx.runtime.is_open(&item_value);
    let presence = use_presence(is_open, attrs.id(), true, None);
    let disclosure = use_disclosure_size(attrs.id(), is_open, presence);

    let is_present = presence.should_render();
    let should_render = is_open || is_present || force_mount;

    let mut last_mounted_val: CopyValue<Option<String>> = use_hook(|| CopyValue::new(None));
    let prev_registered = last_mounted_val.try_read().ok().and_then(|g| g.clone());
    if should_render {
        if let Some(prev) = prev_registered.as_deref()
            && prev != item_value.as_str()
        {
            ctx.runtime.unregister_content_mounted(prev);
        }
        ctx.runtime.register_content_mounted(&item_value);
        if prev_registered.as_deref() != Some(item_value.as_str()) {
            last_mounted_val.set(Some(item_value.clone()));
        }
    } else {
        if let Some(prev) = prev_registered.as_deref() {
            ctx.runtime.unregister_content_mounted(prev);
            last_mounted_val.set(None);
        }
        ctx.runtime.unregister_content_mounted(&item_value);
    }

    use_drop({
        let runtime = ctx.runtime.clone();
        move || {
            if let Some(prev) = last_mounted_val.try_read().ok().and_then(|g| g.clone()) {
                runtime.unregister_content_mounted(&prev);
            }
        }
    });

    let hidden = (!(is_open || is_present)).then_some(true);
    let css_vars = disclosure.size().map(format_accordion_css_vars);
    let effective_style = merge_disclosure_styles(
        style.as_deref(),
        disclosure.guard_style(),
        css_vars.as_deref(),
    );

    let merged = merge_global_attributes(class.as_deref(), effective_style.as_deref(), attributes);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: "{attrs.role()}",
            "aria-labelledby": "{attrs.aria_labelledby()}",
            hidden: hidden,
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            "data-disabled": data_disabled_attr(attrs.is_disabled()),
            onmounted: move |_| disclosure.on_mounted(),
            ontransitionend: move |_| disclosure.on_motion_end(),
            onanimationend: move |_| disclosure.on_motion_end(),
            ..merged.rest,
            {should_render.then_some(children)}
        }
    }
}

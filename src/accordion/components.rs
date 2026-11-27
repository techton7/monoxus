use std::borrow::Cow;

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::accordion::attrs::format_accordion_css_vars;
use crate::collapsible::attrs::merge_disclosure_styles;
use crate::foundation::{
    compose::{
        collect_user_attrs, merge_props, push_attr_opt, push_bool_data_attr, push_event_handler,
    },
    overlay::use_presence,
    roving_focus::key_to_roving_str,
    shared::{ScopeHandle, use_direction_context, use_disclosure_size, use_stable_id},
    state::{sync_signal, use_controllable_signal},
};

use super::{
    runtime::{AccordionRuntime, use_accordion_runtime_controllable},
    state::Accordion,
    types::{AccordionDirection, AccordionMode, AccordionOrientation},
};

#[derive(Clone)]
pub struct AccordionContext {
    pub runtime: AccordionRuntime,
}

#[derive(Clone, PartialEq, Props)]
pub struct AccordionRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
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
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub values: Option<Vec<String>>,
    #[props(default)]
    pub default_values: Option<Vec<String>>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
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
        value,
        default_value,
        values,
        default_values,
        on_value_change,
        on_values_change,
        child,
        attributes,
        children,
    } = props;
    let resolved_dir = use_direction_context(dir.map(Into::into));
    let accordion_dir = AccordionDirection::from(resolved_dir);
    let scope_id = use_stable_id(id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("accordion").child(scope_id);

    let controlled_values = match mode {
        AccordionMode::Single { .. } => value.map(|v| {
            if v.is_empty() {
                Vec::new()
            } else {
                vec![v]
            }
        }),
        AccordionMode::Multiple => values,
    };
    let default_vals = match mode {
        AccordionMode::Single { .. } => default_value
            .filter(|v| !v.is_empty())
            .map(|v| vec![v])
            .or(default_values)
            .unwrap_or_default(),
        AccordionMode::Multiple => default_values
            .or_else(|| default_value.filter(|v| !v.is_empty()).map(|v| vec![v]))
            .unwrap_or_default(),
    };
    let active_ctl = use_controllable_signal(controlled_values, default_vals, None);

    let accordion = Accordion::new(scope, mode)
        .with_orientation(orientation)
        .with_direction(accordion_dir)
        .with_loop_focus(loop_focus)
        .with_disabled(disabled)
        .with_values(active_ctl.peek());

    let runtime = use_accordion_runtime_controllable(
        accordion,
        active_ctl,
        on_value_change,
        on_values_change,
    );

    use_context_provider(|| AccordionContext {
        runtime: runtime.clone(),
    });

    let attrs = runtime.root();
    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("id", attrs.id().to_string(), None, false));
    base.push(Attribute::new("dir", resolved_dir.as_str(), None, false));
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", attrs.is_disabled());

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
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
    #[props(into)]
    pub value: String,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn AccordionItem(props: AccordionItemProps) -> Element {
    let AccordionItemProps {
        value,
        class,
        style,
        disabled,
        child,
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
    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", attrs.id().to_string(), None, false));
    base.push(Attribute::new(
        "data-state",
        attrs.data_state_str(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-value",
        attrs.data_value().to_string(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", attrs.is_disabled());

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct AccordionHeaderProps {
    #[props(default = 3)]
    pub level: u8,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn AccordionHeader(props: AccordionHeaderProps) -> Element {
    let AccordionHeaderProps {
        level,
        class,
        style,
        child,
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

    let clamped_str = clamped.to_string();
    let mut base = Vec::with_capacity(9);
    base.push(Attribute::new("id", attrs.id().to_string(), None, false));
    base.push(Attribute::new("role", attrs.role(), None, false));
    base.push(Attribute::new("aria-level", clamped_str.clone(), None, false));
    base.push(Attribute::new("data-heading-level", clamped_str, None, false));
    base.push(Attribute::new(
        "data-state",
        attrs.data_state_str(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-value",
        attrs.data_value().to_string(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", attrs.is_disabled());

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    macro_rules! render_heading {
        ($tag:ident) => {
            rsx! {
                $tag {
                    ..merged,
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
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn AccordionTrigger(props: AccordionTriggerProps) -> Element {
    let AccordionTriggerProps {
        id,
        class,
        style,
        on_click,
        on_key_down,
        on_mounted,
        child,
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

    let mut base = Vec::with_capacity(14);
    base.push(Attribute::new("type", "button", None, false));
    base.push(Attribute::new("id", trig_id, None, false));
    base.push(Attribute::new(
        "aria-expanded",
        attrs.aria_expanded(),
        None,
        false,
    ));
    push_attr_opt(
        &mut base,
        "aria-controls",
        attrs.aria_controls().map(str::to_owned),
    );
    push_attr_opt(&mut base, "aria-disabled", attrs.aria_disabled());
    base.push(Attribute::new("tabindex", attrs.tabindex().to_string(), None, false));
    if attrs.is_disabled() {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new(
        "data-state",
        attrs.data_state_str(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-value",
        attrs.data_value().to_string(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", attrs.is_disabled());
    base.push(Attribute::new(
        "onfocus",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                if !(*item_ctx.disabled.peek() || focus_runtime.is_disabled_peek()) {
                    let focus_val = item_ctx.value.peek().clone();
                    focus_runtime
                        .roving()
                        .set_current_tab_stop(Some(focus_val));
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                if *item_ctx.disabled.peek() || click_runtime.is_disabled_peek() {
                    return;
                }
                if !evt.default_action_enabled() {
                    return;
                }
                let click_val = item_ctx.value.peek().clone();
                click_runtime.toggle_item(&click_val);
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onkeydown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                if *item_ctx.disabled.peek() || key_runtime.is_disabled_peek() {
                    return;
                }
                let kb_evt: KeyboardEvent = evt.map(|d| KeyboardData::from(d));
                if !kb_evt.default_action_enabled() {
                    return;
                }
                let event_key = kb_evt.key();
                if let Some(key_str) = key_to_roving_str(&event_key)
                    && Accordion::is_navigation_key_for_orientation(
                        key_runtime.orientation_peek(),
                        key_str,
                    )
                {
                    kb_evt.prevent_default();
                    kb_evt.stop_propagation();
                    key_runtime.navigate_key(key_str);
                }
            })
            .erase(),
        ),
        None,
        false,
    ));

    let mut user = collect_user_attrs(class, style, attributes);
    if !attrs.is_disabled() {
        push_event_handler(&mut user, "onclick", on_click);
    }
    push_event_handler(&mut user, "onkeydown", on_key_down);
    push_event_handler(&mut user, "onmounted", on_mounted);

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

#[derive(Clone, PartialEq, Props)]
pub struct AccordionContentProps {
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn AccordionContent(props: AccordionContentProps) -> Element {
    let AccordionContentProps {
        class,
        style,
        force_mount,
        child,
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

    let is_hidden = !(is_open || is_present);
    let css_vars = disclosure.size().map(format_accordion_css_vars);
    let base_style =
        merge_disclosure_styles(None, disclosure.guard_style(), css_vars.as_deref());

    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("id", attrs.id().to_string(), None, false));
    push_attr_opt(&mut base, "style", base_style);
    base.push(Attribute::new("role", attrs.role(), None, false));
    base.push(Attribute::new(
        "aria-labelledby",
        attrs.aria_labelledby().to_string(),
        None,
        false,
    ));
    if is_hidden {
        base.push(Attribute::new("hidden", true, None, false));
    }
    base.push(Attribute::new(
        "data-state",
        attrs.data_state_str(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-value",
        attrs.data_value().to_string(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", attrs.is_disabled());
    base.push(Attribute::new(
        "onmounted",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                disclosure.on_mounted();
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "ontransitionend",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                disclosure.on_motion_end();
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onanimationend",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                disclosure.on_motion_end();
            })
            .erase(),
        ),
        None,
        false,
    ));

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {should_render.then_some(children)}
        }
    }
}

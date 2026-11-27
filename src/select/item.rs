use dioxus::prelude::*;

use crate::foundation::{
    compose::{
        collect_user_attrs, merge_props, push_attr_opt, push_bool_data_attr, push_event_handler,
    },
    shared::use_stable_id,
    state::sync_signal,
};

use super::{
    registry::SelectItemRegistration,
    types::{SelectContext, SelectGroupContext, SelectItemContext},
};

#[derive(Clone, PartialEq, Props)]
pub struct SelectGroupProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
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
pub fn SelectGroup(props: SelectGroupProps) -> Element {
    let SelectGroupProps {
        id,
        aria_labelledby,
        class,
        style,
        child,
        attributes,
        children,
    } = props;
    let base_id = use_stable_id(None);
    let group_id = id.unwrap_or_else(|| format!("{base_id}-group"));
    let default_label_id = format!("{base_id}-label");
    let label_id = use_signal(|| None::<String>);

    use_context_provider(|| SelectGroupContext {
        group_id: group_id.clone(),
        label_id,
        default_label_id,
    });

    let effective_labelledby = aria_labelledby.or_else(|| label_id.read().clone());

    let mut base = Vec::with_capacity(3);
    base.push(Attribute::new("id", group_id, None, false));
    base.push(Attribute::new("role", "group", None, false));
    push_attr_opt(&mut base, "aria-labelledby", effective_labelledby);

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
pub struct SelectLabelProps {
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

#[component]
pub fn SelectLabel(props: SelectLabelProps) -> Element {
    let SelectLabelProps {
        id,
        class,
        style,
        child,
        attributes,
        children,
    } = props;
    let group = try_use_context::<SelectGroupContext>();
    let fallback_id = use_stable_id(None);

    let resolved_id = id.unwrap_or_else(|| {
        group
            .as_ref()
            .map(|g| g.default_label_id.clone())
            .unwrap_or_else(|| fallback_id.into_owned())
    });

    if let Some(group_ctx) = group.as_ref() {
        let mut lid = group_ctx.label_id;
        if lid
            .try_peek()
            .is_ok_and(|cur| cur.as_deref() != Some(resolved_id.as_str()))
        {
            let _ = lid.try_write().map(|mut w| *w = Some(resolved_id.clone()));
        }
    }

    let cleanup_group = group.clone();
    let cleanup_id = resolved_id.clone();
    dioxus::core::use_drop(move || {
        if let Some(g) = cleanup_group {
            let mut lid = g.label_id;
            if lid
                .try_peek()
                .is_ok_and(|cur| cur.as_deref() == Some(cleanup_id.as_str()))
            {
                let _ = lid.try_write().map(|mut w| *w = None);
            }
        }
    });

    let base = vec![Attribute::new("id", resolved_id, None, false)];
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
pub struct SelectItemProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(into)]
    pub value: String,
    #[props(default, into)]
    pub text: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub on_highlight: Option<EventHandler<()>>,
    #[props(default)]
    pub on_unhighlight: Option<EventHandler<()>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn SelectItem(props: SelectItemProps) -> Element {
    let SelectItemProps {
        id,
        value,
        text,
        disabled,
        class,
        style,
        on_highlight,
        on_unhighlight,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<SelectContext>();
    let text_val = text.clone().unwrap_or_else(|| value.clone());
    let item_id = id.unwrap_or_else(|| ctx.runtime.relationships().item_id(&value));

    // Register item
    {
        let rt = ctx.runtime.clone();
        let registration = SelectItemRegistration::new(
            item_id.clone(),
            value.clone(),
            text_val.clone(),
            disabled,
        );
        use_effect(use_reactive((&registration,), {
            let rt = rt.clone();
            move |(reg,)| {
                rt.register_item(reg);
            }
        }));
    }

    {
        let rt = ctx.runtime.clone();
        let unregister_id = item_id.clone();
        dioxus::core::use_drop(move || {
            rt.unregister_item(&unregister_id);
        });
    }

    let is_hl = ctx.runtime.highlighted_value().as_deref() == Some(&value);
    let attrs = ctx.runtime.item_attributes(&value, is_hl, disabled);
    let is_sel = ctx.runtime.is_selected(&value);
    let is_dis = attrs.is_disabled();

    let mut item_value_sig = use_signal(|| value.clone());
    let mut item_selected_sig = use_signal(|| is_sel);
    let mut item_disabled_sig = use_signal(|| is_dis);
    sync_signal(&mut item_value_sig, value.clone());
    sync_signal(&mut item_selected_sig, is_sel);
    sync_signal(&mut item_disabled_sig, is_dis);

    use_context_provider(|| SelectItemContext {
        value: item_value_sig,
        selected: item_selected_sig,
        disabled: item_disabled_sig,
    });

    let mut prev_hl = use_signal(|| false);
    use_effect(use_reactive((&is_hl,), {
        let hl_cb = on_highlight;
        let unhl_cb = on_unhighlight;
        move |(current_hl,)| {
            let was_hl = *prev_hl.peek();
            if current_hl && !was_hl {
                prev_hl.set(true);
                if let Some(ref cb) = hl_cb {
                    cb.call(());
                }
            } else if !current_hl && was_hl {
                prev_hl.set(false);
                if let Some(ref cb) = unhl_cb {
                    cb.call(());
                }
            }
        }
    }));

    let label_str = text.unwrap_or_else(|| value.clone());
    let role_str = attrs.role();
    let aria_disabled = attrs.aria_disabled();
    let data_disabled = attrs.data_disabled();

    let mut base = Vec::with_capacity(14);
    base.push(Attribute::new("id", item_id, None, false));
    base.push(Attribute::new("role", role_str, None, false));
    base.push(Attribute::new(
        "aria-selected",
        if is_sel { "true" } else { "false" },
        None,
        false,
    ));
    push_attr_opt(&mut base, "aria-disabled", aria_disabled);
    base.push(Attribute::new(
        "data-state",
        if is_sel { "checked" } else { "unchecked" },
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-highlighted",
        if is_hl { "true" } else { "false" },
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", data_disabled);
    base.push(Attribute::new("data-value", value.clone(), None, false));
    base.push(Attribute::new("data-label", label_str, None, false));
    base.push(Attribute::new(
        "data-selected",
        if is_sel { "true" } else { "false" },
        None,
        false,
    ));

    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            let v = value.clone();
            move |evt: PointerEvent| {
                if !is_dis {
                    runtime.set_suppress_next_click(false);
                    runtime.mark_pointer_dismiss();
                    let coords = evt.client_coordinates();
                    runtime.record_pressed_item(&v, coords.x, coords.y);
                }
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onpointerup",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            let v = value.clone();
            move |evt: PointerEvent| {
                if is_dis {
                    return;
                }
                if let Some(press) = runtime.take_pressed_item() {
                    if press.value != v {
                        let coords = evt.client_coordinates();
                        let dx = coords.x - press.client_x;
                        let dy = coords.y - press.client_y;
                        if dx.hypot(dy) < 4.0 {
                            evt.prevent_default();
                            evt.stop_propagation();
                            runtime.set_suppress_next_click(true);
                            let selected_val = press.value;
                            let rt_for_select = runtime.clone();
                            // Keep Select open, topmost, and registered as a parent overlay
                            // branch until after the trailing capture `click` event from this
                            // pointerup turn finishes so enclosing Popover/DatePicker stays open (`L-4`).
                            crate::foundation::browser::spawn_root_task(async move {
                                let _ = oxidase::frame::next_frame().await;
                                rt_for_select.set_suppress_next_click(false);
                                if rt_for_select.state.open.try_peek().is_ok() {
                                    rt_for_select.select_item(&selected_val);
                                }
                            });
                        }
                    }
                }
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            let v = value.clone();
            move |evt: MouseEvent| {
                if is_dis {
                    return;
                }
                evt.stop_propagation();
                if runtime.take_suppress_next_click() {
                    return;
                }
                runtime.select_item(&v);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onpointermove",
        Some(EventHandler::new({
            let runtime = ctx.runtime.clone();
            let v = value.clone();
            move |evt: PointerEvent| {
                if evt.pointer_type() == "touch" {
                    return;
                }
                runtime.clear_initial_highlight_pending();
                if is_dis {
                    return;
                }
                runtime.set_highlighted_with_scroll(Some(v.clone()), false);
            }
        })),
    );

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
pub struct SelectItemTextProps {
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

#[component]
pub fn SelectItemText(props: SelectItemTextProps) -> Element {
    let SelectItemTextProps {
        id,
        class,
        style,
        child,
        attributes,
        children,
    } = props;
    let mut base = Vec::with_capacity(1);
    push_attr_opt(&mut base, "id", id);
    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        span {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectItemIndicatorProps {
    #[props(default, into)]
    pub id: Option<String>,
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
    pub children: Option<Element>,
}

#[component]
pub fn SelectItemIndicator(props: SelectItemIndicatorProps) -> Element {
    let SelectItemIndicatorProps {
        id,
        class,
        style,
        force_mount,
        child,
        attributes,
        children,
    } = props;
    let item_ctx = try_use_context::<SelectItemContext>();
    let (is_selected, is_disabled) = item_ctx
        .map(|ctx| (*ctx.selected.read(), *ctx.disabled.read()))
        .unwrap_or((false, false));

    if !is_selected && !force_mount {
        return rsx! {};
    }

    let mut base = Vec::with_capacity(4);
    push_attr_opt(&mut base, "id", id);
    base.push(Attribute::new("aria-hidden", "true", None, false));
    base.push(Attribute::new(
        "data-state",
        if is_selected { "checked" } else { "unchecked" },
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", is_disabled);

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
            "✓"
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct SelectSeparatorProps {
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
}

#[component]
pub fn SelectSeparator(props: SelectSeparatorProps) -> Element {
    let SelectSeparatorProps {
        id,
        class,
        style,
        child,
        attributes,
    } = props;
    let mut base = Vec::with_capacity(4);
    push_attr_opt(&mut base, "id", id);
    base.push(Attribute::new("role", "separator", None, false));
    base.push(Attribute::new("aria-orientation", "horizontal", None, false));
    base.push(Attribute::new("aria-hidden", "true", None, false));

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
        }
    }
}

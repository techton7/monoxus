use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::{
    foundation::{
        browser::spawn_root_task,
        compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
        keyboard::is_ime_composing_key,
        roving_focus::{RovingItemMeta, key_to_roving_str, use_roving_focus_item},
    },
    visually_hidden::VisuallyHiddenState,
};

use super::super::{
    runtime::{NavigationMenuContext, NavigationMenuItemContext, advance_guard_token},
    state::{is_content_entry_key, is_item_hover_enabled},
    types::NAVIGATION_MENU_POINTER_CLICK_GUARD_MS,
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuTriggerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavigationMenuTrigger(props: NavigationMenuTriggerProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    let mut item_ctx = use_context::<NavigationMenuItemContext>();
    let item_value = item_ctx.value_str();
    let item_open_on_hover = item_ctx.open_on_hover_opt();

    let effective_content_id = item_ctx.effective_content_id(&ctx.relationships);
    let trigger_attrs = ctx.trigger_attributes(
        &item_value,
        props.disabled,
        props.id.clone(),
        Some(effective_content_id.clone()),
    );
    let trigger_id = trigger_attrs.id().to_owned();
    let is_open = ctx.current_value() == item_value;

    if item_ctx.trigger_id.peek().as_deref() != Some(trigger_id.as_str()) {
        item_ctx.trigger_id.set(Some(trigger_id.clone()));
    }

    ctx.register_trigger(&item_value, &trigger_id, item_open_on_hover);
    use_roving_focus_item(
        ctx.roving,
        &item_value,
        RovingItemMeta::new(trigger_id.clone(), !props.disabled, is_open),
    );

    let drop_ctx = ctx.clone();
    let drop_value = item_value.clone();
    use_drop(move || {
        drop_ctx.unregister_trigger(&drop_value);
    });

    let mut has_pointer_move_opened = use_signal(|| false);
    let pointer_move_guard_token = use_signal(|| 0u64);
    let mut was_click_close = use_signal(|| false);

    let effective_open_on_hover =
        is_item_hover_enabled(ctx.current_activation_mode(), item_open_on_hover);

    let on_pointer_enter = EventHandler::new(move |_: PointerEvent| {
        if *was_click_close.peek() {
            was_click_close.set(false);
        }
        item_ctx.set_was_escape_close(false);
    });

    let on_pointer_move = {
        let ctx = ctx.clone();
        let item_value = item_value.clone();
        let disabled = props.disabled;
        EventHandler::new(move |evt: PointerEvent| {
            if evt.pointer_type() != "mouse" {
                return;
            }
            if disabled
                || *was_click_close.peek()
                || item_ctx.was_escape_close()
                || *has_pointer_move_opened.peek()
                || !effective_open_on_hover
            {
                return;
            }
            ctx.on_trigger_enter(&item_value, item_open_on_hover);
            has_pointer_move_opened.set(true);
            let token = advance_guard_token(pointer_move_guard_token);
            let ctx_alive = ctx.clone();
            spawn_root_task(async move {
                Delay::new(Duration::from_millis(
                    NAVIGATION_MENU_POINTER_CLICK_GUARD_MS,
                ))
                .await;
                if !ctx_alive.is_alive() {
                    return;
                }
                if pointer_move_guard_token
                    .try_peek()
                    .is_ok_and(|cur| *cur == token)
                    && let Ok(mut guard) = has_pointer_move_opened.try_write()
                {
                    *guard = false;
                }
            });
        })
    };

    let on_pointer_leave = {
        let ctx = ctx.clone();
        let disabled = props.disabled;
        EventHandler::new(move |evt: PointerEvent| {
            if evt.pointer_type() != "mouse" || disabled {
                return;
            }
            let _ = advance_guard_token(pointer_move_guard_token);
            if *has_pointer_move_opened.peek() {
                has_pointer_move_opened.set(false);
            }
            if effective_open_on_hover {
                ctx.on_trigger_leave(item_open_on_hover);
            }
        })
    };

    let on_click = {
        let ctx = ctx.clone();
        let item_value = item_value.clone();
        let disabled = props.disabled;
        EventHandler::new(move |evt: MouseEvent| {
            if disabled {
                evt.prevent_default();
                return;
            }
            let currently_open = ctx.value.try_peek().is_ok_and(|v| *v == item_value);
            if *has_pointer_move_opened.peek() && currently_open {
                return;
            }
            let _ = advance_guard_token(pointer_move_guard_token);
            if *has_pointer_move_opened.peek() {
                has_pointer_move_opened.set(false);
            }
            let disable_toggle = ctx.disable_toggle.try_peek().map(|d| *d).unwrap_or(false);
            let should_close = currently_open && (ctx.is_root_menu || !disable_toggle);
            ctx.on_item_select(&item_value);
            was_click_close.set(should_close);
        })
    };

    let on_focus = {
        let ctx = ctx.clone();
        let trigger_id = trigger_id.clone();
        let item_value = item_value.clone();
        let disabled = props.disabled;
        EventHandler::new(move |_: FocusEvent| {
            ctx.record_element_focus(&trigger_id);
            ctx.roving.handle_item_focus(&item_value, !disabled);
        })
    };

    let on_key_down = {
        let ctx = ctx.clone();
        let item_value = item_value.clone();
        let disabled = props.disabled;
        EventHandler::new(move |evt: KeyboardEvent| {
            if disabled || is_ime_composing_key(&evt) {
                return;
            }
            let currently_open = ctx.value.try_peek().is_ok_and(|v| *v == item_value);
            if currently_open
                && let Some(ks) = key_to_roving_str(&evt.key())
                && is_content_entry_key(ctx.current_orientation(), ctx.current_dir(), ks)
            {
                item_ctx.on_entry_key_down(&ctx.relationships);
                evt.prevent_default();
                evt.stop_propagation();
                return;
            }
            ctx.roving
                .handle_keydown(Some(&item_value), &evt, None::<fn(&str)>);
        })
    };

    let focus_proxy_attrs = ctx.focus_proxy_attributes(&item_value);
    let aria_owns_attrs = ctx.aria_owns_attributes(&item_value, Some(effective_content_id.clone()));
    let vh_attrs = VisuallyHiddenState::new().attributes();

    if *item_ctx.focus_proxy_id.peek() != focus_proxy_attrs.id {
        item_ctx.focus_proxy_id.set(focus_proxy_attrs.id.clone());
    }

    let on_proxy_focus = {
        let ctx = ctx.clone();
        let trigger_id = trigger_id.clone();
        let content_id = effective_content_id.clone();
        move |_| {
            ctx.handle_focus_proxy_focus(&item_ctx, &trigger_id, &content_id);
        }
    };

    let show_aria_owns = is_open && ctx.has_viewport();
    let proxies = rsx! {
        if is_open {
            span {
                id: "{focus_proxy_attrs.id}",
                aria_hidden: "{focus_proxy_attrs.aria_hidden}",
                tabindex: "{focus_proxy_attrs.tabindex}",
                style: "{vh_attrs.style}",
                "data-visually-hidden-root": "{vh_attrs.data_visually_hidden_root}",
                onfocus: on_proxy_focus,
            }
            if show_aria_owns {
                span {
                    id: "{aria_owns_attrs.id}",
                    aria_owns: "{aria_owns_attrs.aria_owns}",
                }
            }
        }
    };

    let mut base = Vec::with_capacity(18);
    base.push(Attribute::new(
        "id",
        trigger_attrs.id().to_owned(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "type",
        trigger_attrs.button_type(),
        None,
        false,
    ));
    if trigger_attrs.is_disabled() {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new(
        "aria-expanded",
        trigger_attrs.aria_expanded(),
        None,
        false,
    ));
    push_attr_opt(
        &mut base,
        "aria-controls",
        trigger_attrs.aria_controls().map(str::to_owned),
    );
    base.push(Attribute::new(
        "data-state",
        trigger_attrs.data_state().as_str(),
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", trigger_attrs.data_disabled());
    base.push(Attribute::new(
        "data-value",
        trigger_attrs.data_value().to_owned(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-monoxus-navigation-menu-trigger",
        trigger_attrs.data_monoxus_navigation_menu_trigger(),
        None,
        false,
    ));
    push_event_handler::<PointerData>(&mut base, "onpointerenter", Some(on_pointer_enter));
    push_event_handler::<PointerData>(&mut base, "onpointermove", Some(on_pointer_move));
    push_event_handler::<PointerData>(&mut base, "onpointerleave", Some(on_pointer_leave));
    push_event_handler::<MouseData>(&mut base, "onclick", Some(on_click));
    push_event_handler::<FocusData>(&mut base, "onfocus", Some(on_focus));
    push_event_handler::<KeyboardData>(&mut base, "onkeydown", Some(on_key_down));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler::<MouseData>(&mut user, "onclick", props.on_click);
    push_event_handler::<KeyboardData>(&mut user, "onkeydown", props.on_key_down);
    push_event_handler::<MountedData>(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    let trigger_el = if let Some(child) = props.child {
        child.call(merged)
    } else {
        rsx! {
            button {
                ..merged,
                {props.children}
            }
        }
    };

    rsx! {
        {trigger_el}
        {proxies}
    }
}

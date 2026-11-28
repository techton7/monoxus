use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    shared::Direction,
};
use crate::toast::attrs::ToastViewportAttrs;
use crate::toast::components::{
    ToastAction, ToastClose, ToastDescription, ToastTitle, root::ManagedToastRoot,
};
use crate::toast::layout::ToastStackLayout;
use crate::toast::runtime::TOAST_STORE;
use crate::toast::types::{ToastActionEvent, ToastDirection, ToastPosition};

#[derive(Props, Clone, PartialEq)]
pub struct ToastViewportProps {
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub position: Option<ToastPosition>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

fn schedule_viewport_pointer_press_release() {
    if TOAST_STORE.peek().pointer_down_in_viewport {
        let seq = TOAST_STORE.peek().pointer_down_seq;
        crate::foundation::browser::spawn_root_task(async move {
            let _ = oxidase::frame::next_frame().await;
            if TOAST_STORE.peek().pointer_down_in_viewport
                && TOAST_STORE.peek().pointer_down_seq == seq
            {
                TOAST_STORE.write().end_viewport_pointer_press(seq);
            }
        });
    }
}

/// Landmark container (<ol role="region">) mounting and structuring active notifications.
#[component]
pub fn ToastViewport(props: ToastViewportProps) -> Element {
    let viewport_id = props
        .id
        .clone()
        .unwrap_or_else(|| "monoxus-toast-viewport".to_string());

    use_hook(|| {
        let mut store = TOAST_STORE.write();
        store.interaction_state.hovered = false;
        store.interaction_state.swiping = false;
        if !crate::toast::hotkey::is_focus_within_element(&viewport_id) {
            store.interaction_state.focused = false;
        }
        store.sync_interaction_state();
    });

    use_drop({
        let vp_id = viewport_id.clone();
        move || {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::toast::runtime::reset_toast_watcher();
                if dioxus::core::Runtime::try_current().is_some() {
                    let focus_was_inside = crate::toast::hotkey::is_focus_within_element(&vp_id);
                    let prev_focus_id = {
                        let mut store = TOAST_STORE.write();
                        store.interaction_state.hovered = false;
                        store.interaction_state.focused = false;
                        store.interaction_state.swiping = false;
                        store.clear_viewport_pointer_press();
                        store.sync_interaction_state();
                        if focus_was_inside {
                            store.last_focused_element_id.take()
                        } else {
                            None
                        }
                    };
                    if focus_was_inside
                        && let Some(doc) = oxidase::dom::Document::current()
                    {
                        let focus_opts = crate::foundation::browser::FocusOptions::with_focus_visible(
                            false,
                            Some(false),
                        );
                        if let Some(prev) = prev_focus_id.as_deref()
                            && let Some(el) = doc.element_by_id(prev)
                        {
                            let _ = el.focus_sync_with_options(focus_opts);
                        } else {
                            let _ = doc.blur_active();
                        }
                    }
                }
            }));
        }
    });

    crate::toast::runtime::ensure_toast_watcher(&viewport_id);

    let context_dir = crate::foundation::shared::use_direction_context(props.dir);
    let store = TOAST_STORE.read();
    let is_expanded = store.expanded;
    let dir = if props.dir.is_some() {
        context_dir.as_str()
    } else {
        match store.config.dir {
            ToastDirection::Auto => context_dir.as_str(),
            ToastDirection::Ltr => "ltr",
            ToastDirection::Rtl => "rtl",
        }
    };
    let effective_position = props.position.unwrap_or(store.config.position);

    let visible_items = store.visible_items();
    let heights: Vec<f64> = visible_items
        .iter()
        .map(|(item, _)| store.toast_height(item.id).unwrap_or(0.0))
        .collect();
    let front_toast_height = heights.first().copied().filter(|&h| h > 0.0);

    let attrs = ToastViewportAttrs::with_position_and_front_height(
        &store.config,
        is_expanded,
        dir,
        effective_position,
        front_toast_height,
    );

    let computed_styles = ToastStackLayout::compute(&heights, store.config.gap, is_expanded);

    let content = if let Some(children) = props.children {
        children
    } else {
        rsx! {
            for (idx, (item, _visible)) in visible_items.into_iter().enumerate() {
                ManagedToastRoot {
                    key: "{item.id.0}",
                    id: item.id,
                    toast_type: item.toast_type,
                    phase: item.phase,
                    index: idx,
                    visible_limit: store.config.visible_toasts,
                    expanded: is_expanded,
                    dismissible: item.meta.dismissible != Some(false)
                        && item.toast_type != crate::toast::types::ToastType::Loading,
                    position: item.meta.position.unwrap_or(effective_position),
                    test_id: item.meta.test_id.clone(),
                    computed_style: computed_styles.get(idx).copied(),
                    content_generation: item.content_generation,
                    if let Some(ref custom_render) = item.meta.custom_renderer {
                        {custom_render(item.id)}
                    } else {
                        ToastTitle {
                            "{item.title}"
                        }
                        if let Some(ref desc) = item.description {
                            ToastDescription {
                                "{desc}"
                            }
                        }
                        if let Some(ref action) = item.meta.action {
                            ToastAction {
                                alt_text: action.alt_text.clone(),
                                toast_id: Some(item.id),
                                on_click: action.on_click.map(|cb| {
                                    EventHandler::new(move |evt: MouseEvent| {
                                        let action_evt = ToastActionEvent::new();
                                        cb.call(action_evt.clone());
                                        if action_evt.is_default_prevented() {
                                            evt.prevent_default();
                                        }
                                    })
                                }),
                                "{action.label}"
                            }
                        }
                        if let Some(ref cancel) = item.meta.cancel {
                            ToastAction {
                                alt_text: cancel.alt_text.clone(),
                                toast_id: Some(item.id),
                                on_click: cancel.on_click.map(|cb| {
                                    EventHandler::new(move |evt: MouseEvent| {
                                        let action_evt = ToastActionEvent::new();
                                        cb.call(action_evt.clone());
                                        if action_evt.is_default_prevented() {
                                            evt.prevent_default();
                                        }
                                    })
                                }),
                                cancel: true,
                                "{cancel.label}"
                            }
                        }
                        if item.meta.dismissible != Some(false)
                            && item.meta.close_button.unwrap_or(
                                store.config.close_button
                                    && item.toast_type != crate::toast::types::ToastType::Loading,
                            )
                        {
                            ToastClose {
                                toast_id: Some(item.id),
                                aria_label: Some(store.config.close_button_aria_label.clone()),
                            }
                        }
                    }
                }
            }
        }
    };

    let vp_id_for_focusout = viewport_id.clone();
    let base_style = (!attrs.style.is_empty()).then_some(attrs.style);

    let mut base = Vec::with_capacity(19);
    base.push(Attribute::new("id", viewport_id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("aria-label", attrs.aria_label, None, false));
    base.push(Attribute::new("tabindex", attrs.tabindex, None, false));
    base.push(Attribute::new("dir", attrs.dir, None, false));
    base.push(Attribute::new("data-sonner-toaster", "true", None, false));
    base.push(Attribute::new("data-expanded", attrs.data_expanded, None, false));
    base.push(Attribute::new("data-position", attrs.data_position, None, false));
    base.push(Attribute::new(
        "data-x-position",
        attrs.data_x_position,
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-y-position",
        attrs.data_y_position,
        None,
        false,
    ));
    push_attr_opt(&mut base, "style", base_style);
    base.push(Attribute::new(
        "onmouseenter",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                if TOAST_STORE.peek().has_active_toasts() {
                    TOAST_STORE.write().set_hovered(true);
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onmousemove",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                if TOAST_STORE.peek().has_active_toasts()
                    && !TOAST_STORE.peek().interaction_state.hovered
                {
                    TOAST_STORE.write().set_hovered(true);
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onmouseleave",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                TOAST_STORE.write().handle_viewport_mouse_leave();
                if !TOAST_STORE.peek().interaction_state.swiping {
                    schedule_viewport_pointer_press_release();
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerup",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                schedule_viewport_pointer_press_release();
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointercancel",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                schedule_viewport_pointer_press_release();
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onfocusin",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                if TOAST_STORE.peek().has_active_toasts()
                    && !TOAST_STORE.peek().pointer_down_in_viewport
                {
                    TOAST_STORE.write().set_focused(true);
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onfocusout",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                let vp_check = vp_id_for_focusout.clone();
                crate::foundation::browser::spawn_root_task(async move {
                    let _ = futures_util::future::select(
                        Box::pin(oxidase::frame::next_frame()),
                        Box::pin(futures_timer::Delay::new(std::time::Duration::from_millis(
                            10,
                        ))),
                    )
                    .await;
                    if !crate::toast::hotkey::is_focus_within_element(&vp_check) {
                        TOAST_STORE.write().set_focused(false);
                    }
                });
            })
            .erase(),
        ),
        None,
        false,
    ));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        ol {
            ..merged,
            {content}
        }
    }
}

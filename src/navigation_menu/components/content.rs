use dioxus::prelude::keyboard_types::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{get_tabbable_candidates, remove_from_tab_order, restore_tab_order},
    compose::merge_global_attributes,
    keyboard::{is_escape_key, is_ime_composing_key},
    overlay::{
        OverlayBranchContext, is_overlay_escape_consumed, mark_overlay_escape_consumed,
        use_overlay_branch_provider, use_presence, use_register_parent_overlay_branches,
    },
    roving_focus::key_to_roving_str,
    shared::{
        EscapeKeyDownEvent, FocusOutsideEvent, InteractOutsideEvent, PointerDownOutsideEvent,
        use_mount_animation_guard,
    },
};

use super::super::{
    runtime::{
        ContentDismissConfig, NavigationMenuContentRenderProps, NavigationMenuContext,
        NavigationMenuInContent, NavigationMenuItemContext, NavigationMenuRootDismissContext,
        ViewportContentData, active_element_id, first_data_focused_id, focus_element_reliable,
        focus_first_candidate_id, handle_proxy_focus, is_active_element_inside,
        is_ignored_arrow_element_by_id,
    },
    state::{is_item_hover_enabled, resolve_content_arrow_target},
    types::{NavigationMenuMotion, NavigationMenuOutsideBehavior},
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_focus_outside: Option<EventHandler<FocusOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default)]
    pub escape_keydown_behavior: NavigationMenuOutsideBehavior,
    #[props(default)]
    pub interact_outside_behavior: NavigationMenuOutsideBehavior,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<NavigationMenuContentRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavigationMenuContent(props: NavigationMenuContentProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    let mut item_ctx = use_context::<NavigationMenuItemContext>();
    let item_value = item_ctx.value_str();
    let content_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.content_id(&item_value));

    if item_ctx.content_id.peek().as_deref() != Some(content_id.as_str()) {
        item_ctx.content_id.set(Some(content_id.clone()));
    }
    ctx.register_content_id(&item_value, &content_id);

    let trigger_id = item_ctx.effective_trigger_id(&ctx.relationships);
    let focus_proxy_id = item_ctx.focus_proxy_id.read().clone();
    let has_viewport = ctx.has_viewport();

    let drop_ctx = ctx.clone();
    let drop_val = item_value.clone();
    use_drop(move || {
        drop_ctx.unregister_content_id(&drop_val);
        drop_ctx.unregister_viewport_content(&drop_val);
    });

    let is_open = ctx.current_value() == item_value;
    let desired_present = props.force_mount || is_open;
    let presence = use_presence(
        desired_present && !has_viewport,
        &content_id,
        true,
        None,
    );

    if has_viewport {
        ctx.register_viewport_content(ViewportContentData {
            value: item_value.clone(),
            content_id: content_id.clone(),
            trigger_id: trigger_id.clone(),
            focus_proxy_id: focus_proxy_id.clone(),
            item_state: item_ctx,
            force_mount: props.force_mount,
            class: props.class.clone(),
            style: props.style.clone(),
            as_child: props.as_child,
            render_child: props.render_child,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_focus_outside: props.on_focus_outside,
            on_interact_outside: props.on_interact_outside,
            escape_keydown_behavior: props.escape_keydown_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            on_mounted: props.on_mounted,
            attributes: props.attributes.clone(),
            children: props.children.clone(),
        });
    } else {
        ctx.unregister_viewport_content(&item_value);
    }

    let active = !has_viewport && (desired_present || presence.should_render());

    render_navigation_menu_content_impl(
        &ctx,
        NavigationMenuContentImplSpec {
            active,
            value: item_value,
            content_id,
            trigger_id,
            focus_proxy_id,
            item_state: item_ctx,
            is_open,
            inline_mode: true,
            class: props.class,
            style: props.style,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_focus_outside: props.on_focus_outside,
            on_interact_outside: props.on_interact_outside,
            escape_keydown_behavior: props.escape_keydown_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            as_child: props.as_child,
            render_child: props.render_child,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
        },
    )
}

pub(crate) struct NavigationMenuContentImplSpec {
    pub active: bool,
    pub value: String,
    pub content_id: String,
    pub trigger_id: String,
    pub focus_proxy_id: String,
    pub item_state: NavigationMenuItemContext,
    pub is_open: bool,
    pub inline_mode: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    pub on_focus_outside: Option<EventHandler<FocusOutsideEvent>>,
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    pub escape_keydown_behavior: NavigationMenuOutsideBehavior,
    pub interact_outside_behavior: NavigationMenuOutsideBehavior,
    pub as_child: bool,
    pub render_child: Option<Callback<NavigationMenuContentRenderProps, Element>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust render helper for `NavigationMenuContent` (both inline and viewport-rendered modes).
///
/// Called unconditionally inside `NavigationMenuContent` and `NavigationMenuViewportItem` so
/// Dioxus hook order remains invariant while avoiding an internal `#[component]` with optional
/// callback props (`Dioxus #5894`).
pub(crate) fn render_navigation_menu_content_impl(
    ctx: &NavigationMenuContext,
    spec: NavigationMenuContentImplSpec,
) -> Element {
    let parent_root_dismiss = try_consume_context::<NavigationMenuRootDismissContext>();
    let parent_branch = try_consume_context::<OverlayBranchContext>();
    let guard_style = use_mount_animation_guard(spec.active && spec.is_open);

    use_context_provider(|| NavigationMenuInContent(true));
    use_context_provider(|| spec.item_state);

    use_register_parent_overlay_branches(
        parent_branch,
        spec.active && spec.is_open,
        vec![spec.content_id.clone()],
    );

    {
        let reg_ctx = ctx.clone();
        let unreg_ctx = ctx.clone();
        use_overlay_branch_provider(
            move |branch: String| reg_ctx.register_branch(branch),
            move |branch: String| unreg_ctx.unregister_branch(&branch),
        );
    }

    use_hook(|| {
        if ctx.is_root_menu {
            let dismiss_ctx = ctx.clone();
            let content_id_for_dismiss = spec.content_id.clone();
            let trigger_id_for_dismiss = spec.trigger_id.clone();
            let item_for_dismiss = spec.item_state;
            let on_dismiss = Callback::new(move |()| {
                let was_inside = is_active_element_inside(&content_id_for_dismiss);
                item_for_dismiss.on_root_content_close(&dismiss_ctx.relationships);
                dismiss_ctx.on_item_dismiss();
                if was_inside {
                    focus_element_reliable(&trigger_id_for_dismiss);
                }
            });
            provide_context(NavigationMenuRootDismissContext { on_dismiss });
        }
    });

    let mut prev_motion = use_signal(|| None::<NavigationMenuMotion>);
    let motion = if spec.active {
        let next = ctx.motion_for_item(&spec.value, *prev_motion.peek());
        if *prev_motion.peek() != next {
            prev_motion.set(next);
        }
        next
    } else {
        *prev_motion.peek()
    };

    if spec.active && spec.is_open {
        ctx.set_active_content_config(Some(ContentDismissConfig {
            value: spec.value.clone(),
            content_id: spec.content_id.clone(),
            trigger_id: spec.trigger_id.clone(),
            item_state: Some(spec.item_state),
            escape_keydown_behavior: spec.escape_keydown_behavior,
            interact_outside_behavior: spec.interact_outside_behavior,
            on_escape_key_down: spec.on_escape_key_down,
            on_pointer_down_outside: spec.on_pointer_down_outside,
            on_focus_outside: spec.on_focus_outside,
            on_interact_outside: spec.on_interact_outside,
        }));
    }

    let cid_for_tab_sync = spec.content_id.clone();
    use_effect(use_reactive(
        (&spec.active, &spec.is_open),
        move |(active, open)| {
            if !active {
                return;
            }
            if open {
                restore_tab_order(&cid_for_tab_sync);
            } else {
                remove_from_tab_order(&cid_for_tab_sync);
            }
        },
    ));

    if !spec.active {
        return rsx! {};
    }

    let merged_user_style = match (spec.style.as_deref(), guard_style) {
        (Some(s), Some(g)) if !s.trim().is_empty() => Some(format!("{s}; {g}")),
        (Some(s), None) if !s.trim().is_empty() => Some(s.to_string()),
        (_, Some(g)) => Some(g.to_string()),
        _ => None,
    };

    let content_attrs = ctx.content_attributes(
        &spec.value,
        spec.is_open,
        motion,
        spec.inline_mode,
        Some(spec.content_id.clone()),
        Some(spec.trigger_id.clone()),
        merged_user_style.as_deref(),
    );

    let item_open_on_hover = spec.item_state.open_on_hover_opt();
    let on_pointer_enter = {
        let ctx = ctx.clone();
        let is_open = spec.is_open;
        let inline_mode = spec.inline_mode;
        move |_: Event<PointerData>| {
            if inline_mode && is_open {
                ctx.on_content_enter();
            }
        }
    };

    let on_pointer_leave = {
        let ctx = ctx.clone();
        let is_open = spec.is_open;
        let inline_mode = spec.inline_mode;
        move |evt: Event<PointerData>| {
            if !inline_mode || !is_open || evt.pointer_type() != "mouse" {
                return;
            }
            if is_item_hover_enabled(ctx.current_activation_mode(), item_open_on_hover) {
                ctx.on_content_leave(item_open_on_hover);
            }
        }
    };

    let on_key_down = {
        let ctx = ctx.clone();
        let content_id = spec.content_id.clone();
        let trigger_id = spec.trigger_id.clone();
        let focus_proxy_id = spec.focus_proxy_id.clone();
        let item_state = spec.item_state;
        let escape_behavior = spec.escape_keydown_behavior;
        let on_escape = spec.on_escape_key_down;
        move |evt: KeyboardEvent| {
            if is_ime_composing_key(&evt) {
                return;
            }
            let mods = evt.modifiers();
            let is_meta_or_ctrl = mods.contains(Modifiers::ALT)
                || mods.contains(Modifiers::CONTROL)
                || mods.contains(Modifiers::META);

            if is_escape_key(&evt.key()) {
                if !ctx.is_root_menu && parent_root_dismiss.is_none() {
                    if is_overlay_escape_consumed() {
                        return;
                    }
                    let esc_evt = EscapeKeyDownEvent::empty();
                    if let Some(ref handler) = on_escape {
                        handler.call(esc_evt.clone());
                    }
                    mark_overlay_escape_consumed();
                    if escape_behavior.should_close() && !esc_evt.is_default_prevented() {
                        item_state.set_was_escape_close(true);
                        let was_inside = is_active_element_inside(&content_id);
                        ctx.on_item_dismiss();
                        if was_inside {
                            focus_element_reliable(&trigger_id);
                        }
                        evt.prevent_default();
                        evt.stop_propagation();
                    }
                }
                return;
            }

            if evt.key() == Key::Named(NamedKey::Tab) && !is_meta_or_ctrl {
                let candidates = get_tabbable_candidates(&content_id);
                let focused_id = active_element_id();
                let index = focused_id
                    .as_deref()
                    .and_then(|fid| candidates.iter().position(|c| c == fid));
                let moving_backwards = mods.contains(Modifiers::SHIFT);
                let next_candidates: Vec<String> = if moving_backwards {
                    match index {
                        Some(idx) if idx > 0 => candidates[..idx].iter().rev().cloned().collect(),
                        _ => Vec::new(),
                    }
                } else {
                    match index {
                        Some(idx) if idx + 1 < candidates.len() => candidates[idx + 1..].to_vec(),
                        Some(_) => Vec::new(),
                        None => candidates,
                    }
                };
                if focus_first_candidate_id(&next_candidates) {
                    evt.prevent_default();
                    evt.stop_propagation();
                } else {
                    evt.stop_propagation();
                    handle_proxy_focus(&focus_proxy_id);
                }
                return;
            }

            let focused_id = active_element_id().or_else(|| first_data_focused_id(&content_id));
            if focused_id.as_deref() == Some(trigger_id.as_str()) {
                return;
            }
            if let Some(ref fid) = focused_id
                && is_ignored_arrow_element_by_id(fid)
            {
                return;
            }
            if !is_meta_or_ctrl
                && !mods.contains(Modifiers::SHIFT)
                && let Some(key_str) = key_to_roving_str(&evt.key())
            {
                let candidates = get_tabbable_candidates(&content_id);
                if let Some(next_id) = resolve_content_arrow_target(
                    &candidates,
                    focused_id.as_deref(),
                    key_str,
                    ctx.current_dir(),
                ) {
                    evt.prevent_default();
                    evt.stop_propagation();
                    focus_element_reliable(&next_id);
                }
            }
        }
    };

    if spec.as_child {
        if let Some(render_child) = spec.render_child.as_ref() {
            return render_child.call(NavigationMenuContentRenderProps {
                attrs: content_attrs,
                is_open: spec.is_open,
                class: spec.class,
                style: spec.style,
                attributes: spec.attributes,
                children: spec.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::NavigationMenuContent: as_child requires render_child; rendering the default div"
        );
    }

    let merged = merge_global_attributes(
        spec.class.as_deref(),
        Some(content_attrs.style()),
        spec.attributes,
    );

    rsx! {
        div {
            id: "{content_attrs.id()}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            aria_labelledby: "{content_attrs.aria_labelledby()}",
            "data-state": content_attrs.data_state().as_str(),
            "data-orientation": "{content_attrs.data_orientation()}",
            "data-motion": content_attrs.data_motion(),
            "data-monoxus-navigation-menu-content": "{content_attrs.data_monoxus_navigation_menu_content()}",
            onpointerenter: on_pointer_enter,
            onpointerleave: on_pointer_leave,
            onkeydown: on_key_down,
            onmounted: move |evt| {
                if let Some(ref handler) = spec.on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {spec.children}
        }
    }
}

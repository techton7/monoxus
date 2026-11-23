use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::keyboard_types::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        WatcherGuard, get_tabbable_candidates, remove_from_tab_order, restore_tab_order,
    },
    overlay::{OverlayBranchContext, PresenceController},
    roving_focus::key_to_roving_str,
    shared::use_mount_animation_guard,
};

use super::super::{
    runtime::{
        ContentDismissConfig, NavigationMenuContext, NavigationMenuInContent,
        NavigationMenuItemContext, NavigationMenuRootDismissContext,
        NavigationMenuContentRenderProps, ViewportContentData, active_element_id,
        first_data_focused_id, focus_element_reliable, focus_first_candidate_id,
        handle_proxy_focus, is_active_element_inside, is_ignored_arrow_element_by_id,
        sync_presence_lifecycle,
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
    pub on_escape_key_down: Option<EventHandler<()>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<()>>,
    #[props(default)]
    pub on_focus_outside: Option<EventHandler<()>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<()>>,
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
    let presence =
        use_signal(|| PresenceController::new(desired_present && !has_viewport).with_retained_mount(true));
    let presence_guard = use_hook(|| Rc::new(RefCell::new(None::<WatcherGuard>)));

    let pid = content_id.clone();
    let pg = Rc::clone(&presence_guard);
    use_effect(use_reactive(
        (&desired_present, &has_viewport),
        move |(present, vp)| {
            if vp {
                *pg.borrow_mut() = None;
                return;
            }
            sync_presence_lifecycle(&pid, present, presence, &pg, || {});
        },
    ));
    use_drop({
        let pg = Rc::clone(&presence_guard);
        move || {
            *pg.borrow_mut() = None;
        }
    });

    if has_viewport {
        ctx.register_viewport_content(ViewportContentData {
            value: item_value,
            content_id,
            trigger_id,
            focus_proxy_id,
            item_state: item_ctx,
            force_mount: props.force_mount,
            class: props.class,
            style: props.style,
            as_child: props.as_child,
            render_child: props.render_child,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_focus_outside: props.on_focus_outside,
            on_interact_outside: props.on_interact_outside,
            escape_keydown_behavior: props.escape_keydown_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
        });
        return rsx! {};
    }

    ctx.unregister_viewport_content(&item_value);

    let should_render = desired_present || presence.read().should_render();
    if !should_render {
        return rsx! {};
    }

    rsx! {
        NavigationMenuContentImpl {
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
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuContentImplProps {
    pub value: String,
    pub content_id: String,
    pub trigger_id: String,
    pub focus_proxy_id: String,
    pub item_state: NavigationMenuItemContext,
    pub is_open: bool,
    pub inline_mode: bool,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<()>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<()>>,
    #[props(default)]
    pub on_focus_outside: Option<EventHandler<()>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<()>>,
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
    #[props(default)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavigationMenuContentImpl(props: NavigationMenuContentImplProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    let parent_root_dismiss = try_consume_context::<Option<NavigationMenuRootDismissContext>>()
        .flatten()
        .or_else(|| try_consume_context::<NavigationMenuRootDismissContext>());
    let parent_branch = try_consume_context::<OverlayBranchContext>();
    let guard_style = use_mount_animation_guard(props.is_open);

    use_context_provider(|| NavigationMenuInContent(true));
    use_context_provider(|| props.item_state);

    let ctx_for_branch = ctx.clone();
    use_context_provider(|| {
        let reg_ctx = ctx_for_branch.clone();
        let unreg_ctx = ctx_for_branch;
        OverlayBranchContext::new(
            move |branch: String| reg_ctx.register_branch(branch),
            move |branch: String| unreg_ctx.unregister_branch(&branch),
        )
    });

    let dismiss_ctx_opt = if ctx.is_root_menu {
        let dismiss_ctx = ctx.clone();
        let content_id_for_dismiss = props.content_id.clone();
        let trigger_id_for_dismiss = props.trigger_id.clone();
        let item_for_dismiss = props.item_state;
        let on_dismiss = Callback::new(move |()| {
            let was_inside = is_active_element_inside(&content_id_for_dismiss);
            item_for_dismiss.on_root_content_close(&dismiss_ctx.relationships);
            dismiss_ctx.on_item_dismiss();
            if was_inside {
                focus_element_reliable(&trigger_id_for_dismiss);
            }
        });
        Some(NavigationMenuRootDismissContext { on_dismiss })
    } else {
        None
    };
    use_context_provider(|| dismiss_ctx_opt);

    if let Some(ref pb) = parent_branch {
        pb.register_branch(&props.content_id);
    }
    let drop_branch_id = props.content_id.clone();
    let drop_parent_branch = parent_branch;
    use_drop(move || {
        if let Some(ref pb) = drop_parent_branch {
            pb.unregister_branch(&drop_branch_id);
        }
    });

    let mut prev_motion = use_signal(|| None::<NavigationMenuMotion>);
    let motion = ctx.motion_for_item(&props.value, *prev_motion.peek());
    if *prev_motion.peek() != motion {
        prev_motion.set(motion);
    }

    if props.is_open {
        ctx.set_active_content_config(Some(ContentDismissConfig {
            value: props.value.clone(),
            content_id: props.content_id.clone(),
            trigger_id: props.trigger_id.clone(),
            item_state: Some(props.item_state),
            escape_keydown_behavior: props.escape_keydown_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_focus_outside: props.on_focus_outside,
            on_interact_outside: props.on_interact_outside,
        }));
    }

    let cid_for_tab_sync = props.content_id.clone();
    use_effect(use_reactive((&props.is_open,), move |(open,)| {
        if open {
            restore_tab_order(&cid_for_tab_sync);
        } else {
            remove_from_tab_order(&cid_for_tab_sync);
        }
    }));

    let merged_user_style = match (props.style.as_deref(), guard_style) {
        (Some(s), Some(g)) if !s.trim().is_empty() => Some(format!("{s}; {g}")),
        (Some(s), None) if !s.trim().is_empty() => Some(s.to_string()),
        (_, Some(g)) => Some(g.to_string()),
        _ => None,
    };

    let content_attrs = ctx.content_attributes(
        &props.value,
        props.is_open,
        motion,
        props.inline_mode,
        Some(props.content_id.clone()),
        Some(props.trigger_id.clone()),
        merged_user_style.as_deref(),
    );

    let item_open_on_hover = props.item_state.open_on_hover_opt();
    let on_pointer_enter = {
        let ctx = ctx.clone();
        let is_open = props.is_open;
        let inline_mode = props.inline_mode;
        move |_: Event<PointerData>| {
            if inline_mode && is_open {
                ctx.on_content_enter();
            }
        }
    };

    let on_pointer_leave = {
        let ctx = ctx.clone();
        let is_open = props.is_open;
        let inline_mode = props.inline_mode;
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
        let content_id = props.content_id.clone();
        let trigger_id = props.trigger_id.clone();
        let focus_proxy_id = props.focus_proxy_id.clone();
        let item_state = props.item_state;
        let escape_behavior = props.escape_keydown_behavior;
        let on_escape = props.on_escape_key_down;
        move |evt: KeyboardEvent| {
            let mods = evt.modifiers();
            let is_meta_or_ctrl = mods.contains(Modifiers::ALT)
                || mods.contains(Modifiers::CONTROL)
                || mods.contains(Modifiers::META);

            if evt.key() == Key::Named(NamedKey::Escape) {
                if !ctx.is_root_menu && parent_root_dismiss.is_none() {
                    if let Some(ref handler) = on_escape {
                        handler.call(());
                    }
                    if escape_behavior.should_close() {
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

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(NavigationMenuContentRenderProps {
                attrs: content_attrs,
                is_open: props.is_open,
                class: props.class,
                style: props.style,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::NavigationMenuContent: as_child requires render_child; rendering the default div"
        );
    }

    rsx! {
        div {
            id: "{content_attrs.id()}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{content_attrs.style()}",
            aria_labelledby: "{content_attrs.aria_labelledby()}",
            "data-state": content_attrs.data_state().as_str(),
            "data-orientation": "{content_attrs.data_orientation()}",
            "data-motion": content_attrs.data_motion(),
            "data-monoxus-navigation-menu-content": "{content_attrs.data_monoxus_navigation_menu_content()}",
            onpointerenter: on_pointer_enter,
            onpointerleave: on_pointer_leave,
            onkeydown: on_key_down,
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

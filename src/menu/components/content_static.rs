use dioxus::prelude::*;

use crate::foundation::{
    overlay::{PlacementAlign, PlacementSide},
    shared::Direction,
};
use crate::menu::{
    MenuContentAttributes, MenuPart, MenuPreventableEvent, MenuRootContext, MenuSurfaceContext,
    MenuVariant, merge_attributes_with_part,
};

use super::content::MenuContentRenderProps;

#[derive(Props, Clone, PartialEq)]
pub struct MenuContentStaticProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub loop_focus: Option<bool>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub on_escape_keydown: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MenuContentRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Non-floating static menu surface retaining WAI-ARIA `role="menu"`, focus, typeahead, and keyboard control (`M-16`, `3B D-6`).
#[component]
pub fn MenuContentStatic(props: MenuContentStaticProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let surface = root.root_surface.clone();

    {
        let resolved_id = props
            .id
            .clone()
            .unwrap_or_else(|| root.relationships.content_id().to_owned());
        let mut cid = surface.content_id;
        if cid.try_peek().is_ok_and(|g| *g != resolved_id) {
            let _ = cid.try_write().map(|mut w| *w = resolved_id);
        }
        let mut fm = surface.force_mount;
        if fm.try_peek().is_ok_and(|g| *g != props.force_mount) {
            let _ = fm.try_write().map(|mut w| *w = props.force_mount);
        }
        let effective_loop = props.loop_focus.unwrap_or_else(|| *root.loop_focus.read());
        let mut lf = surface.loop_focus;
        if lf.try_peek().is_ok_and(|g| *g != effective_loop) {
            let _ = lf.try_write().map(|mut w| *w = effective_loop);
        }
        if let Some(d) = props.dir {
            let mut ds = root.dir;
            if ds.try_peek().is_ok_and(|g| *g != d) {
                let _ = ds.try_write().map(|mut w| *w = d);
            }
        }
        let mut esc = surface.on_escape_keydown;
        let _ = esc.try_write().map(|mut w| *w = props.on_escape_keydown.clone());
        let mut oa = surface.on_open_auto_focus;
        let _ = oa.try_write().map(|mut w| *w = props.on_open_auto_focus.clone());
        let mut ca = surface.on_close_auto_focus;
        let _ = ca.try_write().map(|mut w| *w = props.on_close_auto_focus.clone());
    }

    let parent_branch = try_consume_context::<crate::foundation::overlay::OverlayBranchContext>();
    let should_render = surface.should_render_content();
    let is_open = root.is_open();
    let content_id = surface.current_content_id();
    let portal_id = surface
        .portal_id
        .read()
        .clone()
        .unwrap_or_else(|| format!("{}-portal", surface.surface_key));

    crate::foundation::overlay::use_register_parent_overlay_branches(
        parent_branch.clone(),
        should_render,
        vec![content_id.clone(), portal_id],
    );

    let latest_cid = use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(content_id.clone())));
    {
        let mut prev = latest_cid.borrow_mut();
        if *prev != content_id {
            crate::foundation::overlay::unregister_active_overlay_layer(&prev);
            *prev = content_id.clone();
        }
    }
    let mut registered_layer_id = use_signal(|| Option::<String>::None);
    if is_open {
        let prev = registered_layer_id.peek().clone();
        if prev.as_deref() != Some(content_id.as_str()) {
            if let Some(prev_id) = prev.as_deref() {
                crate::foundation::overlay::unregister_active_overlay_layer(prev_id);
            }
            crate::foundation::overlay::register_active_overlay_layer(&content_id);
            registered_layer_id.set(Some(content_id.clone()));
        }
    } else {
        let prev = registered_layer_id.peek().clone();
        if let Some(prev_id) = prev {
            crate::foundation::overlay::unregister_active_overlay_layer(&prev_id);
            registered_layer_id.set(None);
        }
    }
    {
        let drop_cid = latest_cid.clone();
        use_drop(move || {
            let current_cid = drop_cid.borrow().clone();
            crate::foundation::overlay::unregister_active_overlay_layer(&current_cid);
        });
    }

    {
        let surf_reg = surface.clone();
        let surf_unreg = surface.clone();
        let parent_for_reg = parent_branch.clone();
        let parent_for_unreg = parent_branch;
        use_context_provider(|| {
            crate::foundation::overlay::OverlayBranchContext::new(
                move |branch: String| {
                    surf_reg.register_branch(branch.clone());
                    if let Some(ref pb) = parent_for_reg {
                        pb.register_branch(branch);
                    }
                },
                move |branch: String| {
                    surf_unreg.unregister_branch(&branch);
                    if let Some(ref pb) = parent_for_unreg {
                        pb.unregister_branch(branch);
                    }
                },
            )
        });
    }

    use_context_provider(|| MenuSurfaceContext {
        surface: surface.clone(),
        parent_surface: None,
        close_this_submenu: None,
    });

    if !should_render {
        return rsx! {};
    }

    let labelledby = match root.variant {
        MenuVariant::ContextMenu => None,
        _ => Some(root.trigger_id.read().clone()),
    };

    let attrs = MenuContentAttributes::new(
        content_id.clone(),
        surface.current_wrapper_id(),
        labelledby,
        root.direction(),
        is_open,
        PlacementSide::Bottom.as_str(),
        PlacementAlign::Start.as_str(),
        root.variant,
        MenuPart::ContentStatic,
    );

    let merged_attrs = merge_attributes_with_part(
        props.attributes.clone(),
        MenuPart::ContentStatic.static_data_attr_name(root.variant),
    );
    let content_style = props.style.unwrap_or_default();

    let kd_surface = surface.clone();
    let kd_root = root.clone();
    let user_kd = props.on_key_down.clone();
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if let Some(cb) = user_kd.as_ref() {
            cb.call(evt.clone());
        }
        kd_surface.handle_keydown(&evt, &kd_root);
    });

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(MenuContentRenderProps {
            attrs,
            style: content_style,
            class: props.class,
            on_key_down,
            attributes: merged_attrs,
            children: props.children,
        });
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{content_style}",
            role: "{attrs.role}",
            tabindex: "{attrs.tabindex}",
            dir: "{attrs.dir}",
            "aria-orientation": "{attrs.aria_orientation}",
            "aria-labelledby": attrs.aria_labelledby.as_deref(),
            "data-state": "{attrs.data_state.as_str()}",
            "data-side": "{attrs.data_side}",
            "data-align": "{attrs.data_align}",
            onkeydown: move |evt| on_key_down.call(evt),
            ..merged_attrs,
            {props.children}
        }
    }
}

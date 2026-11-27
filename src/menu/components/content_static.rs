use dioxus::prelude::*;

use crate::foundation::{
    overlay::{DismissBehavior, PlacementSide},
    shared::Direction,
    state::sync_signal,
};
use crate::menu::{MenuPart, MenuPreventableEvent, MenuRootContext, MenuVariant};

use super::content::{MenuSurfaceChild, MenuSurfaceViewSpec, render_menu_surface};

#[derive(Props, Clone, PartialEq)]
pub struct MenuContentStaticProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub keep_mounted: bool,
    #[props(default)]
    pub escape_key_down_behavior: DismissBehavior,
    #[props(default)]
    pub interact_outside_behavior: DismissBehavior,
    #[props(default)]
    pub loop_focus: Option<bool>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
        sync_signal(&mut cid, resolved_id);

        let mut fm = surface.force_mount;
        sync_signal(&mut fm, props.force_mount);
        let mut km = surface.keep_mounted;
        sync_signal(&mut km, props.keep_mounted);
        let mut eb = surface.escape_key_down_behavior;
        sync_signal(&mut eb, props.escape_key_down_behavior);
        let mut ob = surface.interact_outside_behavior;
        sync_signal(&mut ob, props.interact_outside_behavior);

        let effective_loop = props.loop_focus.unwrap_or_else(|| *root.loop_focus.read());
        let mut lf = surface.loop_focus;
        sync_signal(&mut lf, effective_loop);

        if let Some(d) = props.dir {
            let mut ds = root.dir;
            sync_signal(&mut ds, d);
        }

        let mut esc = surface.on_escape_key_down;
        esc.set(props.on_escape_key_down);
        let mut oa = surface.on_open_auto_focus;
        oa.set(props.on_open_auto_focus);
        let mut ca = surface.on_close_auto_focus;
        ca.set(props.on_close_auto_focus);
    }

    let is_open = root.is_open();
    let labelledby = match root.variant {
        MenuVariant::ContextMenu => None,
        _ => Some(root.trigger_id.read().clone()),
    };

    render_menu_surface(MenuSurfaceViewSpec {
        root,
        surface,
        parent_surface: None,
        close_this_submenu: None,
        is_floating: false,
        is_open,
        part: MenuPart::ContentStatic,
        labelledby,
        default_static_side: PlacementSide::Bottom,
        hide_when_detached: false,
        class: props.class,
        style: props.style,
        on_key_down: props.on_key_down,
        on_pointer_enter: None,
        on_pointer_move: None,
        on_pointer_leave: None,
        child: MenuSurfaceChild::Static(props.child),
        attributes: props.attributes,
        children: props.children,
    })
}

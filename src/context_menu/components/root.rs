use dioxus::prelude::*;

use crate::foundation::shared::{Direction, ScopeHandle, use_direction_context, use_stable_id};
use crate::menu::{
    MenuActivePortalSurface, MenuRelationships, MenuRootContext, MenuVariant,
    use_menu_surface_lifecycle,
};

#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuRootProps {
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default = true)]
    pub modal: bool,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub submenu_open_delay_ms: Option<u64>,
    pub children: Element,
}

/// Root provider coordinating virtual-point context menu open state, modal policy, direction, and lifecycle.
#[component]
pub fn ContextMenuRoot(props: ContextMenuRootProps) -> Element {
    let effective_dir = use_direction_context(props.dir);
    let token = use_stable_id(None);
    let relationships = use_hook(|| MenuRelationships::new(ScopeHandle::root("context-menu").child(token)));
    let is_controlled = props.open.is_some();
    let open_sig = use_signal(|| props.open.unwrap_or(props.default_open));

    if let Some(controlled) = props.open {
        let mut s = open_sig;
        if s.try_peek().is_ok_and(|cur| *cur != controlled) {
            let _ = s.try_write().map(|mut w| *w = controlled);
        }
    }

    let root = MenuRootContext::new(
        MenuVariant::ContextMenu,
        relationships,
        open_sig,
        is_controlled,
        props.on_open_change,
        props.on_open_change_complete,
        props.modal,
        effective_dir,
        props.loop_focus,
        props.submenu_open_delay_ms,
        None,
    );

    {
        let mut m = root.modal;
        if m.try_peek().is_ok_and(|cur| *cur != props.modal) {
            let _ = m.try_write().map(|mut w| *w = props.modal);
        }
        let mut d = root.dir;
        if d.try_peek().is_ok_and(|cur| *cur != effective_dir) {
            let _ = d.try_write().map(|mut w| *w = effective_dir);
        }
        let mut lf = root.loop_focus;
        if lf.try_peek().is_ok_and(|cur| *cur != props.loop_focus) {
            let _ = lf.try_write().map(|mut w| *w = props.loop_focus);
        }
    }

    use_menu_surface_lifecycle(root.root_surface.clone(), root.clone());
    use_context_provider(|| MenuActivePortalSurface(root.root_surface.clone()));
    use_context_provider(|| root);

    rsx! { {props.children} }
}

pub use ContextMenuRoot as ContextMenu;

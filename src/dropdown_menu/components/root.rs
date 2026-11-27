use dioxus::prelude::*;

use crate::foundation::{
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    state::{sync_signal, use_controllable_signal},
};
use crate::menu::{
    MenuActivePortalSurface, MenuRelationships, MenuRootContext, MenuVariant,
    use_menu_surface_lifecycle,
};

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuRootProps {
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

/// Root provider managing controlled/uncontrolled open state, modal policy, direction, and lifecycle for [`DropdownMenu`](crate::dropdown_menu::DropdownMenuRoot).
#[component]
pub fn DropdownMenuRoot(props: DropdownMenuRootProps) -> Element {
    let effective_dir = use_direction_context(props.dir);
    let token = use_stable_id(None);
    let relationships =
        use_hook(|| MenuRelationships::new(ScopeHandle::root("dropdown-menu").child(token)));
    let open_state = use_controllable_signal(props.open, props.default_open, props.on_open_change);

    let root = MenuRootContext::new(
        MenuVariant::DropdownMenu,
        relationships,
        open_state,
        props.on_open_change_complete,
        props.modal,
        effective_dir,
        props.loop_focus,
        props.submenu_open_delay_ms,
        None,
    );

    {
        let mut m = root.modal;
        sync_signal(&mut m, props.modal);
        let mut d = root.dir;
        sync_signal(&mut d, effective_dir);
        let mut lf = root.loop_focus;
        sync_signal(&mut lf, props.loop_focus);
    }

    use_menu_surface_lifecycle(root.root_surface.clone(), root.clone());
    use_context_provider(|| MenuActivePortalSurface(root.root_surface.clone()));
    use_context_provider(|| root);

    rsx! { {props.children} }
}

pub use DropdownMenuRoot as DropdownMenu;

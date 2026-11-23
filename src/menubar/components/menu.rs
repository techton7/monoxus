use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;
use crate::menu::{
    MenuActivePortalSurface, MenuRootContext, MenuVariant, MenubarMenuBridge,
    use_menu_surface_lifecycle,
};
use crate::menubar::{MenubarMenuEntry, MenubarMenuHandles, MenubarRuntimeContext};

/// Per-menu metadata shared between [`MenubarMenu`] and its [`MenubarTrigger`](super::trigger::MenubarTrigger).
#[derive(Clone)]
pub struct MenubarSingleMenuContext {
    pub value: String,
    pub trigger_id: Signal<String>,
    pub content_id: Signal<String>,
    pub disabled: Signal<bool>,
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarMenuProps {
    #[props(default)]
    pub value: Option<String>,
    pub children: Element,
}

/// Top-level named menu slot owned by [`MenubarRoot`](super::root::MenubarRoot) (`MB-1`).
#[component]
pub fn MenubarMenu(props: MenubarMenuProps) -> Element {
    let menubar = use_context::<MenubarRuntimeContext>();
    let fallback_token = use_stable_id(None);
    let menu_value = use_hook(|| props.value.unwrap_or_else(|| fallback_token.into_owned()));

    let rels = use_hook(|| {
        menubar
            .relationships
            .menu_relationships(menubar.relationships.scope().child(menu_value.clone()))
    });

    let open_sig = use_signal(|| menubar.is_menu_open(&menu_value));
    let is_open_now = menubar.is_menu_open(&menu_value);
    {
        let mut os = open_sig;
        if os.try_peek().is_ok_and(|cur| *cur != is_open_now) {
            let _ = os.try_write().map(|mut w| *w = is_open_now);
        }
    }

    let close_cb_menubar = menubar.clone();
    let close_cb_value = menu_value.clone();
    let on_open_change = use_callback(move |next: bool| {
        if next {
            close_cb_menubar.open_menu(&close_cb_value, crate::menu::MenuOpenFocusTarget::ContentContainer);
        } else {
            close_cb_menubar.close_menu(&close_cb_value);
        }
    });

    let bridge = use_hook(|| {
        let mb_triggers = menubar.clone();
        let mb_arrow = menubar.clone();
        let mb_close = menubar.clone();
        let val_arrow = menu_value.clone();
        MenubarMenuBridge {
            menubar_root_id: menubar.relationships.root_id().to_owned(),
            menu_value: menu_value.clone(),
            all_trigger_ids: Rc::new(move || mb_triggers.all_trigger_ids()),
            on_cross_menu_arrow: Rc::new(move |key, on_sub_trigger, in_submenu| {
                mb_arrow.handle_cross_menu_arrow(&val_arrow, key, on_sub_trigger, in_submenu)
            }),
            close_menubar: Rc::new(move || mb_close.close_all()),
        }
    });

    let dir = menubar.state.read().direction();
    let loop_focus = menubar.state.read().loop_focus();
    let open_delay = *menubar.submenu_open_delay_ms.read();

    let root = MenuRootContext::new(
        MenuVariant::Menubar,
        rels.clone(),
        open_sig,
        true,
        Some(on_open_change),
        None,
        false,
        dir,
        loop_focus,
        open_delay,
        Some(bridge),
    );

    {
        let mut d = root.dir;
        if d.try_peek().is_ok_and(|cur| *cur != dir) {
            let _ = d.try_write().map(|mut w| *w = dir);
        }
        let mut lf = root.loop_focus;
        if lf.try_peek().is_ok_and(|cur| *cur != loop_focus) {
            let _ = lf.try_write().map(|mut w| *w = loop_focus);
        }
    }

    let disabled_sig = use_signal(|| false);
    let tid_str = root.trigger_id.read().clone();
    let cid_str = root.root_surface.current_content_id();
    let disabled_now = *disabled_sig.read();
    let dom_order = menubar
        .state
        .try_peek()
        .ok()
        .and_then(|s| s.menus().iter().find(|m| m.value == menu_value).map(|m| m.dom_order))
        .unwrap_or(usize::MAX);

    let needs_register = menubar.state.try_peek().is_ok_and(|s| {
        !s.menus().iter().any(|m| {
            m.value == menu_value
                && m.trigger_id == tid_str
                && m.content_id == cid_str
                && m.disabled == disabled_now
        })
    });
    if needs_register {
        menubar.register_menu(
            MenubarMenuEntry::new(
                menu_value.clone(),
                tid_str,
                cid_str.clone(),
                disabled_now,
                dom_order,
            ),
            MenubarMenuHandles {
                value: menu_value.clone(),
                skip_exit_animation: root.skip_exit_animation,
                ignore_close_auto_focus: root.ignore_close_auto_focus,
                open_focus_target: root.root_surface.open_focus_target,
            },
        );
    }

    let drop_menubar = menubar;
    let drop_val = menu_value.clone();
    dioxus::core::use_drop(move || {
        drop_menubar.unregister_menu(&drop_val);
    });

    use_menu_surface_lifecycle(root.root_surface.clone(), root.clone());
    use_context_provider(|| MenubarSingleMenuContext {
        value: menu_value,
        trigger_id: root.trigger_id,
        content_id: root.root_surface.content_id,
        disabled: disabled_sig,
    });
    use_context_provider(|| MenuActivePortalSurface(root.root_surface.clone()));
    use_context_provider(|| root);

    rsx! { {props.children} }
}

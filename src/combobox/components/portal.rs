use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    browser::{remove_element_by_id, teleport_element_to_host},
    overlay::PortalHost,
};

use crate::combobox::{runtime::use_combobox_context, types::ComboboxPortalContext};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxPortalProps {
    #[props(default)]
    pub host: Option<PortalHost>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub force_mount: bool,
    pub children: Element,
}

#[component]
pub fn ComboboxPortal(props: ComboboxPortalProps) -> Element {
    let ctx = use_combobox_context();

    let resolved_host = if props.disabled {
        PortalHost::inline()
    } else {
        props.host.unwrap_or_else(|| ctx.runtime.portal_host())
    };

    use_context_provider(|| ComboboxPortalContext {
        host: resolved_host.clone(),
        disabled: props.disabled,
        force_mount: props.force_mount,
    });

    let host_attr = if resolved_host.is_inline() {
        "inline"
    } else {
        resolved_host.id().unwrap_or("default")
    };

    let portal_id = format!("{}-portal-root", ctx.runtime.current_content_id());
    let is_open = ctx.runtime.is_open();
    let force_mount = props.force_mount || ctx.runtime.force_mount();

    // DEF-CBX-12: Track latest dynamic `portal_id` for `use_drop`.
    let tracked_portal_id: Rc<RefCell<String>> =
        use_hook(|| Rc::new(RefCell::new(portal_id.clone())));
    if *tracked_portal_id.borrow() != portal_id {
        *tracked_portal_id.borrow_mut() = portal_id.clone();
    }

    {
        let pid = portal_id.clone();
        let target_host = resolved_host.clone();
        let disabled = props.disabled;
        use_effect(use_reactive(
            (&is_open, &force_mount, &pid),
            move |(open, fm, cur_pid)| {
                if (!open && !fm) || disabled || target_host.is_inline() {
                    return;
                }
                let host_id = match &target_host {
                    PortalHost::Inline => None,
                    PortalHost::Named(name) => Some(name.as_ref()),
                    PortalHost::Default => None,
                };
                teleport_element_to_host(&cur_pid, host_id);
            },
        ));
    }

    {
        let tracked = Rc::clone(&tracked_portal_id);
        use_drop(move || {
            let pid_cleanup = tracked.borrow().clone();
            remove_element_by_id(&pid_cleanup);
        });
    }

    if !is_open && !force_mount {
        return rsx! {};
    }

    let is_teleported = !props.disabled && !resolved_host.is_inline();

    rsx! {
        div {
            id: "{portal_id}",
            style: "display: contents;",
            "data-portal-host": "{host_attr}",
            "data-portal-disabled": if props.disabled { "true" } else { "false" },
            "data-portal-teleported": if is_teleported { "true" } else { "false" },
            {props.children}
        }
    }
}

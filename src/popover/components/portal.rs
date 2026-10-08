use dioxus::prelude::*;

use crate::foundation::overlay::PortalHost;
use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverPortalProps {
    #[props(default)]
    pub host: Option<PortalHost>,
    #[props(default = false)]
    pub force_mount: bool,
    pub children: Element,
}

#[component]
pub fn PopoverPortal(props: PopoverPortalProps) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();
    let should_render = props.force_mount || runtime.should_render_portal();
    if !should_render {
        return rsx! {};
    }

    let resolved_host = props
        .host
        .unwrap_or_else(|| runtime.portal().host().clone());
    let host_attr = if resolved_host.is_inline() {
        "inline"
    } else {
        resolved_host.id().unwrap_or("default")
    };

    let pid = runtime.relationships().scope().qualify("portal");
    let pid_effect = pid.clone();
    let target_host = resolved_host.clone();

    use_effect(use_reactive((&should_render,), move |(render,)| {
        if !render && !props.force_mount {
            return;
        }
        if let PortalHost::Named(name) = &target_host {
            crate::foundation::browser::teleport_element_to_host(
                &pid_effect,
                Some(name.as_ref()),
            );
        }
    }));

    let pid_cleanup = pid.clone();
    let is_named_host = matches!(resolved_host, PortalHost::Named(_));
    dioxus::core::use_drop(move || {
        if is_named_host {
            crate::foundation::browser::remove_element_by_id(&pid_cleanup);
        }
    });

    rsx! {
        div {
            id: "{pid}",
            style: "display: contents;",
            "data-portal-host": "{host_attr}",
            {props.children}
        }
    }
}

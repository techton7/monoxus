use dioxus::prelude::*;

use crate::foundation::overlay::PortalHost;
use crate::hover_card::components::HoverCardRootContext;

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardPortalProps {
    #[props(default)]
    pub host: Option<PortalHost>,
    #[props(default = false)]
    pub force_mount: bool,
    pub children: Element,
}

#[component]
pub fn HoverCardPortal(props: HoverCardPortalProps) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read();
    let should_render = props.force_mount || runtime.should_render_portal();

    let resolved_host = props
        .host
        .unwrap_or_else(|| runtime.portal().host().clone());
    let host_attr = if resolved_host.is_inline() {
        "inline"
    } else {
        resolved_host.id().unwrap_or("default")
    };

    let pid = runtime.relationships().portal_id().to_string();
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
    use_drop(move || {
        if is_named_host {
            crate::foundation::browser::remove_element_by_id(&pid_cleanup);
        }
    });

    if !should_render {
        return rsx! {};
    }

    rsx! {
        div {
            id: "{pid}",
            style: "display: contents;",
            "data-portal-host": "{host_attr}",
            {props.children}
        }
    }
}

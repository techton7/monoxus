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
    crate::foundation::overlay::use_portal_teleport(
        pid.clone(),
        should_render,
        false,
        &resolved_host,
    );

    rsx! {
        div {
            style: "display: contents;",
            "data-portal-anchor": "",
            if should_render {
                div {
                    id: "{pid}",
                    style: "display: contents;",
                    "data-portal-host": "{host_attr}",
                    {props.children}
                }
            }
        }
    }
}

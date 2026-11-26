use dioxus::prelude::*;

use crate::foundation::overlay::{OverlayPortalSpec, PortalHost, render_overlay_portal};
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
    let resolved_host = props
        .host
        .unwrap_or_else(|| runtime.portal().host().clone());

    render_overlay_portal(OverlayPortalSpec {
        portal_id: runtime.relationships().portal_id().to_string(),
        should_render: runtime.should_render_portal(),
        disabled: false,
        force_mount: props.force_mount,
        host: resolved_host,
        emit_status_attrs: false,
        children: props.children,
    })
}

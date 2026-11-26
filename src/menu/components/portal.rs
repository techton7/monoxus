use dioxus::prelude::*;

use crate::foundation::{
    overlay::{OverlayPortalSpec, PortalHost, render_overlay_portal},
    state::sync_signal,
};
use crate::menu::{MenuRootContext, MenuSurfaceRuntime};

/// Context selecting which surface owns the enclosing `Portal` (`Content` vs `SubContent`).
#[derive(Clone)]
pub struct MenuActivePortalSurface(pub MenuSurfaceRuntime);

#[derive(Props, Clone, PartialEq)]
pub struct MenuPortalProps {
    #[props(default)]
    pub host: Option<PortalHost>,
    #[props(default = false)]
    pub force_mount: bool,
    pub children: Element,
}

/// Portal wrapper mounting floating menu surfaces into [`PortalHost`] while preserving presence exit cycles.
#[component]
pub fn MenuPortal(props: MenuPortalProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let portal_surface = try_use_context::<MenuActivePortalSurface>()
        .map(|s| s.0)
        .unwrap_or_else(|| root.root_surface.clone());

    let pid = use_hook(|| format!("{}-portal", portal_surface.surface_key));
    let mut portal_id_sig = portal_surface.portal_id;
    sync_signal(&mut portal_id_sig, Some(pid.clone()));

    render_overlay_portal(OverlayPortalSpec {
        portal_id: pid,
        should_render: portal_surface.should_render_content(),
        disabled: false,
        force_mount: props.force_mount,
        host: props.host.unwrap_or_else(PortalHost::default_host),
        emit_status_attrs: false,
        children: props.children,
    })
}

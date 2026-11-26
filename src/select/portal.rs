use dioxus::prelude::*;

use crate::foundation::overlay::{OverlayPortalSpec, PortalHost, render_overlay_portal};

use super::types::SelectContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectPortalContext {
    pub host: PortalHost,
    pub disabled: bool,
    pub force_mount: bool,
}

#[component]
pub fn SelectPortal(
    #[props(default)] host: Option<PortalHost>,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] force_mount: bool,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();

    let resolved_host = if disabled {
        PortalHost::inline()
    } else {
        host.unwrap_or_else(|| ctx.runtime.select().portal_host().clone())
    };

    use_context_provider(|| SelectPortalContext {
        host: resolved_host.clone(),
        disabled,
        force_mount,
    });

    let portal_id = format!("{}-portal-root", ctx.runtime.relationships().content_id());
    let is_force_mounted = force_mount || ctx.runtime.force_mount();

    render_overlay_portal(OverlayPortalSpec {
        portal_id,
        should_render: ctx.runtime.should_render_portal(),
        disabled,
        force_mount: is_force_mounted,
        host: resolved_host,
        emit_status_attrs: true,
        children,
    })
}

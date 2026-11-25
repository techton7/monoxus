use dioxus::prelude::*;

use crate::foundation::overlay::PortalHost;

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

    let host_attr = if resolved_host.is_inline() {
        "inline"
    } else {
        resolved_host.id().unwrap_or("default")
    };

    let portal_id = format!("{}-portal-root", ctx.runtime.relationships().content_id());
    let should_render_portal = ctx.runtime.should_render_portal();
    let is_force_mounted = force_mount || ctx.runtime.force_mount();
    let should_render = should_render_portal || is_force_mounted;
    let is_teleported = !disabled && !resolved_host.is_inline();

    crate::foundation::overlay::use_portal_teleport(
        portal_id.clone(),
        should_render,
        disabled,
        &resolved_host,
    );

    rsx! {
        div {
            style: "display: contents;",
            "data-portal-anchor": "",
            if should_render {
                div {
                    id: "{portal_id}",
                    style: "display: contents;",
                    "data-portal-host": "{host_attr}",
                    "data-portal-disabled": if disabled { "true" } else { "false" },
                    "data-portal-teleported": if is_teleported { "true" } else { "false" },
                    {children}
                }
            }
        }
    }
}

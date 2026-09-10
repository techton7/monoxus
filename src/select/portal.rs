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

    let pid = portal_id.clone();
    let target_host = resolved_host.clone();
    let should_render_portal = ctx.runtime.should_render_portal();
    use_effect(use_reactive((&should_render_portal,), move |(should_render,)| {
        if !should_render && !force_mount {
            return;
        }
        let host_id = match &target_host {
            PortalHost::Inline => None,
            PortalHost::Named(name) => Some(name.as_ref()),
            PortalHost::Default => None,
        };
        crate::foundation::browser::teleport_element_to_host(&pid, host_id);
    }));

    let pid_cleanup = portal_id.clone();
    dioxus::core::use_drop(move || {
        crate::foundation::browser::remove_element_by_id(&pid_cleanup);
    });

    let is_force_mounted = force_mount || ctx.runtime.force_mount();
    let should_render = should_render_portal || is_force_mounted;
    if !should_render {
        return rsx! {};
    }

    let is_teleported = !disabled && !resolved_host.is_inline();

    rsx! {
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

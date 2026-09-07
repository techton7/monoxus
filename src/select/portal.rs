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
    let is_open = ctx.runtime.is_open();

    if !is_open && !force_mount {
        return rsx! {};
    }

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

    rsx! {
        div {
            style: "display: contents;",
            "data-portal-host": "{host_attr}",
            "data-portal-disabled": if disabled { "true" } else { "false" },
            {children}
        }
    }
}

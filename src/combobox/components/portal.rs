use dioxus::prelude::*;

use crate::foundation::overlay::{PortalHost, use_portal_teleport};

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
    let should_render = is_open || force_mount;
    let is_teleported = !props.disabled && !resolved_host.is_inline();

    use_portal_teleport(
        portal_id.clone(),
        should_render,
        props.disabled,
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
                    "data-portal-disabled": if props.disabled { "true" } else { "false" },
                    "data-portal-teleported": if is_teleported { "true" } else { "false" },
                    {props.children}
                }
            }
        }
    }
}

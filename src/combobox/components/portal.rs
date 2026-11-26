use dioxus::prelude::*;

use crate::foundation::overlay::{OverlayPortalSpec, PortalHost, render_overlay_portal};

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

    let portal_id = format!("{}-portal-root", ctx.runtime.current_content_id());
    let force_mount = props.force_mount || ctx.runtime.force_mount();

    render_overlay_portal(OverlayPortalSpec {
        portal_id,
        should_render: ctx.runtime.should_render_content(),
        disabled: props.disabled,
        force_mount,
        host: resolved_host,
        emit_status_attrs: true,
        children: props.children,
    })
}

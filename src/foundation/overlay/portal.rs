use dioxus::prelude::*;

pub use ui_sdk::foundation::overlay::portal::*;

/// Plain Rust specification for [`render_overlay_portal`] (zero internal `#[component]` overhead,
/// compliant with `Dioxus #5894`).
pub(crate) struct OverlayPortalSpec {
    pub portal_id: String,
    pub should_render: bool,
    pub disabled: bool,
    pub force_mount: bool,
    pub host: PortalHost,
    /// When `true`, also emits `data-portal-disabled` and `data-portal-teleported` (`Select`/`Combobox`).
    pub emit_status_attrs: bool,
    pub children: Element,
}

/// Plain Rust render helper unifying all `*Portal` components (`PRT-1`..`PRT-7`), calling
/// [`super::use_portal_teleport`] unconditionally and emitting `data-portal-anchor`,
/// `data-portal-host`, and optional `data-portal-disabled` / `data-portal-teleported`.
pub(crate) fn render_overlay_portal(spec: OverlayPortalSpec) -> Element {
    let effective_host = if spec.disabled {
        PortalHost::inline()
    } else {
        spec.host
    };
    let effective_render = spec.should_render || spec.force_mount;
    let host_attr = effective_host.host_attr_value();
    let is_teleported = !spec.disabled && !effective_host.is_inline();

    super::use_portal_teleport(
        spec.portal_id.clone(),
        effective_render,
        spec.disabled,
        &effective_host,
    );

    let disabled_attr = spec
        .emit_status_attrs
        .then_some(if spec.disabled { "true" } else { "false" });
    let teleported_attr = spec
        .emit_status_attrs
        .then_some(if is_teleported { "true" } else { "false" });

    rsx! {
        div {
            style: "display: contents;",
            "data-portal-anchor": "",
            if effective_render {
                div {
                    id: "{spec.portal_id}",
                    style: "display: contents;",
                    "data-portal-host": "{host_attr}",
                    "data-portal-disabled": disabled_attr,
                    "data-portal-teleported": teleported_attr,
                    {spec.children}
                }
            }
        }
    }
}

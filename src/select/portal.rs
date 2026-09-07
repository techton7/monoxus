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

    // Physical DOM Teleportation in browser runtime
    #[cfg(target_arch = "wasm32")]
    {
        let pid = portal_id.clone();
        let target_host = resolved_host.clone();
        use_effect(use_reactive((&is_open,), move |(open,)| {
            if !open && !force_mount {
                return;
            }
            let pid = pid.clone();
            let host_id = match &target_host {
                PortalHost::Inline => "",
                PortalHost::Named(name) => name.as_ref(),
                PortalHost::Default => "",
            };
            let script = format!(
                r#"(function() {{
                    const el = document.getElementById({pid:?});
                    if (!el) return;
                    const hostId = {host_id:?};
                    let target = hostId ? document.getElementById(hostId) : null;
                    if (!target) {{
                        target = document.getElementById("portal-root") || document.body;
                    }}
                    if (target && el.parentElement !== target) {{
                        target.appendChild(el);
                    }}
                }})()"#
            );
            let _ = js_sys::eval(&script);
        }));

        let pid_cleanup = portal_id.clone();
        dioxus::core::use_drop(move || {
            let pid = pid_cleanup.clone();
            let script = format!(
                r#"(function() {{
                    const el = document.getElementById({pid:?});
                    if (el && el.parentElement) {{
                        el.remove();
                    }}
                }})()"#
            );
            let _ = js_sys::eval(&script);
        });
    }

    let is_force_mounted = force_mount || ctx.runtime.force_mount();
    if !is_open && !is_force_mounted {
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

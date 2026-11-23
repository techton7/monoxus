use dioxus::prelude::*;

use crate::foundation::overlay::PortalHost;
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

    let should_render = props.force_mount || portal_surface.should_render_content();

    let resolved_host = props.host.unwrap_or_else(PortalHost::default_host);
    let host_attr = if resolved_host.is_inline() {
        "inline"
    } else {
        resolved_host.id().unwrap_or("default")
    };

    let pid = use_hook(|| format!("{}-portal", portal_surface.surface_key));
    {
        let mut portal_id_sig = portal_surface.portal_id;
        let pid_reg = pid.clone();
        if portal_id_sig.try_peek().is_ok_and(|g| g.as_deref() != Some(pid_reg.as_str())) {
            let _ = portal_id_sig.try_write().map(|mut w| *w = Some(pid_reg));
        }
    }

    let pid_effect = pid.clone();
    let target_host = resolved_host.clone();
    use_effect(use_reactive((&should_render, &target_host), move |(render, host)| {
        if !render && !props.force_mount {
            return;
        }
        if !host.is_inline() {
            let host_id = match &host {
                PortalHost::Named(name) => Some(name.as_ref()),
                PortalHost::Default => None,
                PortalHost::Inline => unreachable!(),
            };
            crate::foundation::browser::teleport_element_to_host(&pid_effect, host_id);
        }
    }));

    let pid_cleanup = pid.clone();
    let is_teleported_host = !resolved_host.is_inline();
    dioxus::core::use_drop(move || {
        if is_teleported_host {
            crate::foundation::browser::remove_element_by_id(&pid_cleanup);
        }
    });

    if !should_render {
        return rsx! {};
    }

    rsx! {
        div {
            id: "{pid}",
            style: "display: contents;",
            "data-portal-host": "{host_attr}",
            {props.children}
        }
    }
}

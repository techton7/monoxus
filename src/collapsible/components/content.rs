use dioxus::prelude::*;

use crate::collapsible::attrs::{format_collapsible_css_vars, merge_disclosure_styles};
use crate::collapsible::components::CollapsibleContext;
use crate::foundation::{
    compose::merge_global_attributes,
    overlay::use_presence,
    shared::{data_disabled_attr, use_disclosure_size},
    state::sync_signal,
};

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub hidden_until_found: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CollapsibleContent(props: CollapsibleContentProps) -> Element {
    let ctx = use_context::<CollapsibleContext>();
    let runtime = ctx.runtime.read();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = props.id.clone().unwrap_or_else(|| content.id().to_string());

    use_effect(use_reactive((&props.id,), {
        let mut cid_sig = ctx.content_id;
        move |(custom_id,)| {
            if let Some(id) = custom_id {
                sync_signal(&mut cid_sig, Some(id));
            }
        }
    }));

    let presence = use_presence(is_open, &content_id, true, None);
    let disclosure = use_disclosure_size(&content_id, is_open, presence);

    let is_presence_mounted = presence.should_render();
    let should_render = is_open || is_presence_mounted || props.force_mount;

    {
        let mut mounted_sig = ctx.content_mounted;
        sync_signal(&mut mounted_sig, should_render);
    }

    use_drop({
        let mut mounted_sig = ctx.content_mounted;
        move || {
            if let Ok(mut m) = mounted_sig.try_write() {
                *m = false;
            }
        }
    });

    if !should_render {
        return rsx! {};
    }

    let hidden_attr: Option<&'static str> = if is_open || is_presence_mounted {
        None
    } else if props.hidden_until_found {
        Some("until-found")
    } else {
        Some("true")
    };

    let css_vars = disclosure.size().map(format_collapsible_css_vars);
    let effective_style = merge_disclosure_styles(
        props.style.as_deref(),
        disclosure.guard_style(),
        css_vars.as_deref(),
    );
    let merged = merge_global_attributes(
        props.class.as_deref(),
        effective_style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{content_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            hidden: hidden_attr,
            "data-state": content.data_state_str(),
            "data-disabled": data_disabled_attr(content.is_disabled()),
            onmounted: move |_| disclosure.on_mounted(),
            ontransitionend: move |_| disclosure.on_motion_end(),
            onanimationend: move |_| disclosure.on_motion_end(),
            ..merged.rest,
            {props.children}
        }
    }
}

use dioxus::prelude::*;

use crate::collapsible::components::CollapsibleContext;
use crate::foundation::shared::{data_disabled_attr, use_mount_animation_guard};

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
                if cid_sig.peek().as_ref() != Some(&id) {
                    cid_sig.set(Some(id));
                }
            }
        }
    }));

    let guard_style = use_mount_animation_guard(is_open);
    let should_render = is_open || props.force_mount;

    {
        let mut mounted_sig = ctx.content_mounted;
        if *mounted_sig.peek() != should_render {
            mounted_sig.set(should_render);
        }
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

    let effective_style = match (props.style.as_deref(), guard_style) {
        (Some(custom), Some(guard)) => Some(format!("{custom}; {guard}")),
        (Some(custom), None) => Some(custom.to_string()),
        (None, Some(guard)) => Some(guard.to_string()),
        (None, None) => None,
    };

    rsx! {
        div {
            id: "{content_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: effective_style.as_deref().unwrap_or_default(),
            hidden: (!is_open).then_some(true),
            "data-state": content.data_state_str(),
            "data-disabled": data_disabled_attr(content.is_disabled()),
            {props.children}
        }
    }
}

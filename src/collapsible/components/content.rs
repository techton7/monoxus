use dioxus::prelude::*;

use crate::collapsible::components::CollapsibleContext;

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

    if !is_open && !props.force_mount {
        return rsx! {};
    }

    rsx! {
        div {
            id: "{content_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            hidden: !is_open,
            "data-state": content.data_state_str(),
            "data-disabled": if content.is_disabled() { "true" } else { "false" },
            {props.children}
        }
    }
}

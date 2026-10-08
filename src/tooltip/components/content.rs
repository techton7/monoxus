use dioxus::prelude::*;

use crate::tooltip::components::TooltipRootContext;

#[derive(Props, Clone, PartialEq)]
pub struct TooltipContentProps {
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
pub fn TooltipContent(props: TooltipContentProps) -> Element {
    let ctx = use_context::<TooltipRootContext>();
    let runtime = ctx.runtime.read();
    let should_render = props.force_mount || runtime.should_render_content();
    if !should_render {
        return rsx! {};
    }

    let content = runtime.content();
    let content_id = props.id.unwrap_or_else(|| content.id().to_string());
    let wrapper_id = content.wrapper_id();
    let wrapper_style = runtime.wrapper_style();
    let css_vars = runtime.content_css_custom_properties();

    let inner_style = format!("{css_vars} {}", props.style.as_deref().unwrap_or_default());

    rsx! {
        div {
            id: "{wrapper_id}",
            "data-monoxus-floating-content-wrapper": "",
            style: "{wrapper_style}",
            div {
                id: "{content_id}",
                role: content.role(),
                class: props.class.as_deref().unwrap_or_default(),
                style: "{inner_style}",
                "data-state": content.data_state().as_str(),
                "data-side": content.data_side(),
                "data-align": content.data_align(),
                tabindex: "-1",
                onmounted: runtime.mount_content(),
                onmouseenter: runtime.content_pointer_enter(),
                onmouseleave: runtime.content_pointer_leave(),
                onkeydown: runtime.escape_keydown(),
                {props.children}
            }
        }
    }
}

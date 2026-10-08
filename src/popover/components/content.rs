use dioxus::prelude::*;

use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    pub children: Element,
}

#[component]
pub fn PopoverContent(props: PopoverContentProps) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();
    let should_render = props.force_mount || runtime.should_render_content();
    if !should_render {
        return rsx! {};
    }

    let content = runtime.content();
    let content_id = props.id.unwrap_or_else(|| content.id().to_string());
    let wrapper_id = format!("{}-wrapper", content_id);
    let wrapper_style = runtime.wrapper_style();
    let css_vars = runtime.content_css_custom_properties();

    let title_id = (ctx.title_id)();
    let desc_id = (ctx.description_id)();

    let labelledby = props.aria_labelledby.or(title_id);
    let describedby = props.aria_describedby.or(desc_id);

    let inner_style = format!("{css_vars} {}", props.style.as_deref().unwrap_or_default());

    rsx! {
        div {
            id: "{wrapper_id}",
            "data-monoxus-floating-content-wrapper": "",
            style: "{wrapper_style}",
            div {
                id: "{content_id}",
                role: content.role(),
                aria_modal: content.aria_modal(),
                aria_labelledby: labelledby.as_deref(),
                aria_describedby: describedby.as_deref(),
                class: props.class.as_deref().unwrap_or_default(),
                style: "{inner_style}",
                "data-state": content.data_state().as_str(),
                "data-side": content.data_side(),
                "data-align": content.data_align(),
                tabindex: "-1",
                onmounted: runtime.mount_content(),
                onkeydown: runtime.escape_keydown(),
                onclick: move |evt: MouseEvent| evt.stop_propagation(),
                {props.children}
            }
        }
    }
}

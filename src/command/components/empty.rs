use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props, push_event_handler};

use super::super::{
    attrs::CommandEmptyAttrs,
    runtime::CommandContext,
    state::should_render_empty,
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandEmptyProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandEmpty(props: CommandEmptyProps) -> Element {
    let ctx = use_context::<CommandContext>();
    let count = ctx.registry.read().filtered().count;
    let settled = *ctx.settled.read();

    if !should_render_empty(count, settled, props.force_mount) {
        return rsx! {};
    }

    let resolved_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.peek().empty_id());
    let attrs = CommandEmptyAttrs::new(resolved_id);

    let mut base = Vec::with_capacity(3);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new(
        "data-command-empty",
        attrs.data_command_empty,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_event_handler},
    shared::use_stable_id,
};

use super::super::{
    attrs::CommandSeparatorAttrs,
    runtime::CommandContext,
    state::should_render_separator,
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandSeparatorProps {
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
}

#[component]
pub fn CommandSeparator(props: CommandSeparatorProps) -> Element {
    let ctx = use_context::<CommandContext>();
    let search = ctx.search.read();
    if !should_render_separator(&search, props.force_mount) {
        return rsx! {};
    }

    let fallback_key = use_stable_id(None).into_owned();
    let sep_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.peek().separator_id(&fallback_key));
    let attrs = CommandSeparatorAttrs::new(sep_id);

    let mut base = Vec::with_capacity(3);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new(
        "aria-hidden",
        attrs.aria_hidden,
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-command-separator",
        attrs.data_command_separator,
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
        }
    }
}

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    state::sync_signal,
};

use super::super::{
    attrs::{CommandListAttrs, merge_flex_column_style},
    runtime::CommandContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandListProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub aria_label: Option<String>,
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
pub fn CommandList(props: CommandListProps) -> Element {
    let mut ctx = use_context::<CommandContext>();
    let resolved_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.peek().list_id());
    sync_signal(&mut ctx.list_id, resolved_id.clone());

    let attrs = CommandListAttrs::new(resolved_id, props.aria_label.as_deref());
    let height_px = *ctx.list_height.read();
    let is_grid = ctx.columns.read().is_some();
    let height_style = CommandListAttrs::format_height_style(height_px, None);
    let base_style = merge_flex_column_style(
        (!height_style.is_empty()).then_some(height_style.as_str()),
        is_grid,
    );

    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("aria-label", attrs.aria_label, None, false));
    base.push(Attribute::new(
        "data-command-list",
        attrs.data_command_list,
        None,
        false,
    ));
    push_attr_opt(&mut base, "style", base_style);

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

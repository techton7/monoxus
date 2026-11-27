use dioxus::prelude::*;

use crate::foundation::{
    compose::{
        MergePropsConfig, collect_user_attrs, merge_props_with_config, push_attr_opt,
        push_event_handler,
    },
    state::sync_signal,
};
use crate::rating_group::runtime::{RatingGroupItemContext, use_rating_group_context};

/// Properties for [`RatingGroupItem`].
#[derive(Clone, PartialEq, Props)]
pub struct RatingGroupItemProps {
    pub index: usize,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_pointer_move: Option<EventHandler<PointerEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Presentational item inside a [`crate::rating_group::RatingGroupRoot`] handling pointer hit math
/// and hover preview while exposing its fill state (`active` / `partial` / `inactive`).
#[component]
pub fn RatingGroupItem(props: RatingGroupItemProps) -> Element {
    let ctx = use_rating_group_context();
    let runtime = ctx.0;

    let item_attrs = runtime.item_attrs(props.index, props.id.clone(), props.disabled);
    let item_dom_id = item_attrs.id.clone();
    let effective_disabled = runtime.def.read().disabled || props.disabled;

    let mut item_index_sig = use_signal(|| props.index);
    sync_signal(&mut item_index_sig, props.index);
    let mut item_disabled_sig = use_signal(|| props.disabled);
    sync_signal(&mut item_disabled_sig, props.disabled);

    use_context_provider(|| RatingGroupItemContext {
        runtime: runtime.clone(),
        index: item_index_sig,
        item_disabled: item_disabled_sig,
    });

    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("id", item_dom_id.clone(), None, false));
    base.push(Attribute::new("role", item_attrs.role, None, false));
    base.push(Attribute::new(
        "data-value",
        item_attrs.data_value.to_string(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        item_attrs.data_orientation,
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", item_attrs.data_disabled);
    push_attr_opt(&mut base, "data-readonly", item_attrs.data_readonly);
    push_attr_opt(&mut base, "data-invalid", item_attrs.data_invalid);
    base.push(Attribute::new(
        "data-state",
        item_attrs.data_state.as_str(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-rating-group-item",
        item_attrs.data_rating_group_item,
        None,
        false,
    ));

    {
        let mut runtime = runtime.clone();
        let index = props.index;
        let item_disabled = props.disabled;
        let dom_id = item_dom_id.clone();
        push_event_handler(
            &mut base,
            "onclick",
            Some(EventHandler::new(move |evt: MouseEvent| {
                let coords = evt.client_coordinates();
                runtime.handle_item_click(index, item_disabled, coords.x, coords.y, &dom_id);
            })),
        );
    }
    {
        let mut runtime = runtime.clone();
        let index = props.index;
        let item_disabled = props.disabled;
        let dom_id = item_dom_id;
        push_event_handler(
            &mut base,
            "onpointermove",
            Some(EventHandler::new(move |evt: PointerEvent| {
                let coords = evt.client_coordinates();
                let pointer_type = evt.pointer_type();
                runtime.handle_item_pointer_move(
                    index,
                    item_disabled,
                    &pointer_type,
                    coords.x,
                    coords.y,
                    &dom_id,
                );
            })),
        );
    }

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    if !effective_disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
    }
    push_event_handler(&mut user, "onpointermove", props.on_pointer_move);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &["onpointermove"],
        },
    );

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

use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    overlay::{FloatingLayer, PlacementSide},
    shared::{ScopeHandle, use_stable_id},
    state::use_controllable_signal,
};
use crate::popover::{
    components::PopoverContext,
    relationships::PopoverRelationships,
    runtime::use_popover_runtime,
    state::Popover,
    types::POPOVER_GEOMETRY_NAMESPACE,
};

#[derive(Props, Clone, PartialEq)]
pub struct PopoverRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default)]
    pub modal: Option<bool>,
    pub children: Element,
}

#[component]
pub fn PopoverRoot(props: PopoverRootProps) -> Element {
    let PopoverRootProps {
        id,
        open,
        default_open,
        on_open_change,
        on_open_change_complete,
        modal,
        children,
    } = props;

    let open_ctl = use_controllable_signal(open, default_open, on_open_change);
    let open_signal = open_ctl.signal();

    let scope_id = use_stable_id(id.clone().map(Cow::Owned)).into_owned();
    let scope = ScopeHandle::root("popover").child(scope_id);
    let relationships = PopoverRelationships::new(scope.clone());

    let is_open = open_signal();
    let is_modal = modal.unwrap_or(false);

    let mut popover = if is_modal {
        Popover::new_modal(scope, is_open)
    } else {
        Popover::new(scope, is_open)
    };

    if let Some(m) = modal {
        popover = popover.with_modal(m);
    }

    let floating = FloatingLayer::new(PlacementSide::Bottom)
        .with_side_offset(4.0)
        .with_avoid_collisions(true)
        .with_collision_padding(8.0)
        .with_namespace(POPOVER_GEOMETRY_NAMESPACE);
    popover = popover.with_floating(floating);

    let change_handler = move |next_open: bool| {
        open_ctl.set(next_open);
    };

    let runtime = use_popover_runtime(popover, change_handler);
    runtime.set_on_open_change_complete(on_open_change_complete);
    let mut runtime_sig = use_signal(|| runtime.clone());
    if runtime_sig.peek().popover() != runtime.popover() {
        runtime_sig.set(runtime.clone());
    }

    let title_id = use_signal(|| None);
    let description_id = use_signal(|| None);
    let content_id = use_signal(|| None);

    use_context_provider(|| PopoverContext {
        runtime: runtime_sig,
        open: open_signal,
        open_ctl,
        on_open_change,
        relationships: relationships.clone(),
        title_id,
        description_id,
        content_id,
    });

    rsx! {
        {children}
    }
}

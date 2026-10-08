use dioxus::prelude::*;

use crate::foundation::{
    overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost},
    shared::ScopeHandle,
};
use crate::popover::{
    components::PopoverContext,
    relationships::PopoverRelationships,
    runtime::use_popover_runtime,
    state::Popover,
    types::{
        POPOVER_GEOMETRY_NAMESPACE, PopoverCloseFocusPolicy, PopoverOpenFocusPolicy,
        PopoverOutsideInteractionPolicy, PopoverScrollLockPolicy,
    },
};

#[derive(Props, Clone, PartialEq)]
pub struct PopoverRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub open: Option<Signal<bool>>,
    #[props(default)]
    pub default_open: Option<bool>,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub modal: Option<bool>,
    #[props(default)]
    pub open_focus: Option<PopoverOpenFocusPolicy>,
    #[props(default)]
    pub close_focus: Option<PopoverCloseFocusPolicy>,
    #[props(default)]
    pub scroll_lock: Option<PopoverScrollLockPolicy>,
    #[props(default)]
    pub outside_interaction: Option<PopoverOutsideInteractionPolicy>,
    #[props(default)]
    pub portal_host: Option<PortalHost>,
    #[props(default)]
    pub side: Option<PlacementSide>,
    #[props(default)]
    pub side_offset: Option<f32>,
    #[props(default)]
    pub align: Option<PlacementAlign>,
    #[props(default)]
    pub align_offset: Option<f32>,
    pub children: Element,
}

#[component]
pub fn PopoverRoot(props: PopoverRootProps) -> Element {
    let open_signal = props
        .open
        .unwrap_or_else(|| use_signal(|| props.default_open.unwrap_or(false)));

    let scope_id = props.id.clone().unwrap_or_else(|| "popover".to_string());
    let scope = ScopeHandle::root("popover").child(scope_id);
    let relationships = PopoverRelationships::new(scope.clone());

    let is_open = open_signal();
    let is_modal = props.modal.unwrap_or(false);

    let mut popover = if is_modal {
        Popover::new_modal(scope, is_open)
    } else {
        Popover::new(scope, is_open)
    };

    if let Some(m) = props.modal {
        popover = popover.with_modal(m);
    }
    if let Some(of) = props.open_focus {
        popover = popover.with_open_focus_policy(of);
    }
    if let Some(cf) = props.close_focus {
        popover = popover.with_close_focus_policy(cf);
    }
    if let Some(sl) = props.scroll_lock {
        popover = popover.with_scroll_lock_policy(sl);
    }
    if let Some(oi) = props.outside_interaction {
        popover = popover.with_outside_interaction_policy(oi);
    }
    if let Some(host) = props.portal_host {
        popover = popover.with_portal_host(host);
    }

    let side = props.side.unwrap_or(PlacementSide::Bottom);
    let mut floating = FloatingLayer::new(side)
        .with_side_offset(props.side_offset.unwrap_or(4.0))
        .with_namespace(POPOVER_GEOMETRY_NAMESPACE);
    if let Some(align) = props.align {
        floating = floating.with_align(align);
    }
    if let Some(align_offset) = props.align_offset {
        floating = floating.with_align_offset(align_offset);
    }
    popover = popover.with_floating(floating);

    let on_open_change = props.on_open_change;
    let change_handler = move |next_open: bool| {
        if *open_signal.peek() != next_open {
            let mut sig = open_signal;
            sig.set(next_open);
        }
        if let Some(h) = on_open_change {
            h.call(next_open);
        }
    };

    let runtime = use_popover_runtime(popover, change_handler);
    let mut runtime_sig = use_signal(|| runtime.clone());
    runtime_sig.set(runtime.clone());

    let title_id = use_signal(|| None);
    let description_id = use_signal(|| None);

    use_context_provider(|| PopoverContext {
        runtime: runtime_sig,
        open: open_signal,
        on_open_change,
        relationships: relationships.clone(),
        title_id,
        description_id,
    });

    let attrs = runtime.root();

    rsx! {
        div {
            id: "{attrs.id()}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or("display: contents;"),
            "data-state": attrs.data_state().as_str(),
            {props.children}
        }
    }
}

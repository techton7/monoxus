use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost},
    shared::{ScopeHandle, use_stable_id},
};
use crate::hover_card::{
    components::HoverCardRootContext,
    relationships::HoverCardRelationships,
    runtime::use_hover_card_runtime,
    state::HoverCard as HoverCardState,
    types::{
        HOVER_CARD_DEFAULT_CLOSE_DELAY_MS, HOVER_CARD_DEFAULT_OPEN_DELAY_MS,
        HOVER_CARD_DEFAULT_SIDE_OFFSET, HOVER_CARD_GEOMETRY_NAMESPACE,
    },
};

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub open: Option<Signal<bool>>,
    #[props(default)]
    pub default_open: Option<bool>,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default = HOVER_CARD_DEFAULT_OPEN_DELAY_MS)]
    pub open_delay: u64,
    #[props(default = HOVER_CARD_DEFAULT_CLOSE_DELAY_MS)]
    pub close_delay: u64,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = true)]
    pub ignore_non_keyboard_focus: bool,
    #[props(default)]
    pub portal_host: Option<PortalHost>,
    #[props(default = PlacementSide::Bottom)]
    pub side: PlacementSide,
    #[props(default = HOVER_CARD_DEFAULT_SIDE_OFFSET)]
    pub side_offset: f32,
    #[props(default = PlacementAlign::Center)]
    pub align: PlacementAlign,
    #[props(default = 0.0)]
    pub align_offset: f32,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn HoverCardRoot(props: HoverCardRootProps) -> Element {
    let fallback_open = use_signal(|| props.default_open.unwrap_or(false));
    let open_signal = props.open.unwrap_or(fallback_open);

    let stable_id = use_stable_id(None);
    let scope_id = props.id.clone().unwrap_or_else(|| stable_id.into_owned());
    let scope = ScopeHandle::root("hover-card").child(scope_id);
    let relationships = HoverCardRelationships::new(scope.clone());

    let is_open = open_signal();

    let mut floating = FloatingLayer::new(props.side)
        .with_align(props.align)
        .with_side_offset(props.side_offset)
        .with_align_offset(props.align_offset)
        .with_namespace(HOVER_CARD_GEOMETRY_NAMESPACE);
    let _ = &mut floating;

    let mut hover_card = HoverCardState::new(scope, is_open)
        .with_open_delay_ms(props.open_delay)
        .with_close_delay_ms(props.close_delay)
        .with_disabled(props.disabled)
        .with_ignore_non_keyboard_focus(props.ignore_non_keyboard_focus)
        .with_floating(floating);

    if let Some(host) = props.portal_host {
        hover_card = hover_card.with_portal_host(host);
    }

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

    let on_open_change_complete = props.on_open_change_complete.map(|handler| {
        Rc::new(move |open_val: bool| {
            handler.call(open_val);
        }) as Rc<dyn Fn(bool)>
    });

    let runtime = use_hover_card_runtime(hover_card, change_handler, on_open_change_complete);
    let mut runtime_sig = use_signal(|| runtime.clone());
    runtime_sig.set(runtime.clone());

    use_context_provider(|| HoverCardRootContext {
        runtime: runtime_sig,
        open: open_signal,
        on_open_change,
        relationships: relationships.clone(),
    });

    let attrs = runtime.root();

    let merged = merge_global_attributes(None, Some("display: contents;"), props.attributes);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": attrs.data_state().as_str(),
            ..merged.rest,
            {props.children}
        }
    }
}

pub use HoverCardRoot as HoverCard;

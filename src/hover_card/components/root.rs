use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    shared::{ScopeHandle, use_stable_id},
    state::use_controllable_signal,
};
use crate::hover_card::{
    components::HoverCardRootContext,
    relationships::HoverCardRelationships,
    runtime::use_hover_card_runtime,
    state::HoverCard as HoverCardState,
    types::{HOVER_CARD_DEFAULT_CLOSE_DELAY_MS, HOVER_CARD_DEFAULT_OPEN_DELAY_MS},
};

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardRootProps {
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
    #[props(default = HOVER_CARD_DEFAULT_OPEN_DELAY_MS)]
    pub open_delay: u64,
    #[props(default = HOVER_CARD_DEFAULT_CLOSE_DELAY_MS)]
    pub close_delay: u64,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = true)]
    pub ignore_non_keyboard_focus: bool,
    pub children: Element,
}

#[component]
pub fn HoverCardRoot(props: HoverCardRootProps) -> Element {
    let open_ctl = use_controllable_signal(props.open, props.default_open, props.on_open_change);
    let open_signal = open_ctl.signal();

    let stable_id = use_stable_id(None);
    let scope_id = props.id.clone().unwrap_or_else(|| stable_id.into_owned());
    let scope = ScopeHandle::root("hover-card").child(scope_id);
    let relationships = HoverCardRelationships::new(scope.clone());

    let is_open = open_signal();

    let hover_card = HoverCardState::new(scope, is_open)
        .with_open_delay_ms(props.open_delay)
        .with_close_delay_ms(props.close_delay)
        .with_disabled(props.disabled)
        .with_ignore_non_keyboard_focus(props.ignore_non_keyboard_focus);

    let change_handler = move |next_open: bool| {
        open_ctl.set(next_open);
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
        on_open_change: props.on_open_change,
        relationships: relationships.clone(),
    });

    rsx! {
        {props.children}
    }
}

pub use HoverCardRoot as HoverCard;

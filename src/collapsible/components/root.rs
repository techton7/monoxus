use dioxus::prelude::*;

use crate::collapsible::{
    components::CollapsibleContext,
    relationships::CollapsibleRelationships,
    runtime::use_collapsible_runtime,
    state::Collapsible,
};
use crate::foundation::shared::ScopeHandle;

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleRootProps {
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
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    pub children: Element,
}

#[component]
pub fn CollapsibleRoot(props: CollapsibleRootProps) -> Element {
    let open_signal = props
        .open
        .unwrap_or_else(|| use_signal(|| props.default_open.unwrap_or(false)));

    let disabled_signal = use_signal(|| props.disabled);
    let mut disabled_sig = disabled_signal;
    if *disabled_sig.peek() != props.disabled {
        disabled_sig.set(props.disabled);
    }

    let scope_id = props.id.clone().unwrap_or_else(|| "collapsible".to_string());
    let scope = ScopeHandle::root("collapsible").child(scope_id);
    let relationships = CollapsibleRelationships::new(scope.clone());

    let is_open = open_signal();
    let collapsible = Collapsible::new(scope, is_open).with_disabled(props.disabled);

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

    let runtime = use_collapsible_runtime(collapsible, Some(change_handler));
    let mut runtime_sig = use_signal(|| runtime.clone());
    runtime_sig.set(runtime.clone());

    let content_id = use_signal(|| None);

    use_context_provider(|| CollapsibleContext {
        runtime: runtime_sig,
        open: open_signal,
        disabled: disabled_signal,
        content_id,
        on_open_change,
        relationships: relationships.clone(),
    });

    let attrs = runtime.root();

    rsx! {
        div {
            id: "{attrs.id()}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            "data-state": attrs.data_state_str(),
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            {props.children}
        }
    }
}

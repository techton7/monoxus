use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    shared::{ScopeHandle, use_stable_id},
    state::use_controllable_signal,
};
use crate::tooltip::{
    components::{TooltipProviderContext, TooltipRootContext},
    relationships::TooltipRelationships,
    runtime::{use_tooltip_provider_runtime, use_tooltip_runtime_with_complete},
    state::{Tooltip, TooltipProvider as TooltipProviderDef},
};

#[derive(Props, Clone, PartialEq)]
pub struct TooltipRootProps {
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
    pub delay_duration: Option<u64>,
    #[props(default)]
    pub skip_delay_duration: Option<u64>,
    #[props(default)]
    pub disable_hoverable_content: Option<bool>,
    #[props(default)]
    pub close_on_trigger_click: Option<bool>,
    #[props(default)]
    pub ignore_non_keyboard_focus: Option<bool>,
    pub children: Element,
}

#[component]
pub fn TooltipRoot(props: TooltipRootProps) -> Element {
    let open_ctl = use_controllable_signal(props.open, props.default_open, props.on_open_change);
    let open_signal = open_ctl.signal();

    let scope_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("tooltip").child(scope_id);
    let relationships = TooltipRelationships::new(scope.clone());

    let parent_provider_ctx = try_consume_context::<TooltipProviderContext>();

    let fallback_scope = ScopeHandle::root("tooltip")
        .child(format!("{}-fallback-provider", relationships.root_id()));
    let mut fallback_provider_def = TooltipProviderDef::new(fallback_scope);
    if let Some(delay) = props.delay_duration {
        fallback_provider_def = fallback_provider_def.with_delay_duration_ms(delay);
    }
    if let Some(skip) = props.skip_delay_duration {
        fallback_provider_def = fallback_provider_def.with_skip_delay_duration_ms(skip);
    }
    if let Some(dhc) = props.disable_hoverable_content {
        fallback_provider_def = fallback_provider_def.with_disable_hoverable_content(dhc);
    }
    if let Some(cotc) = props.close_on_trigger_click {
        fallback_provider_def = fallback_provider_def.with_close_on_trigger_click(cotc);
    }
    if let Some(inkf) = props.ignore_non_keyboard_focus {
        fallback_provider_def = fallback_provider_def.with_ignore_non_keyboard_focus(inkf);
    }
    let fallback_provider_runtime = use_tooltip_provider_runtime(fallback_provider_def);

    // Preserve parent TooltipProvider shared skip-delay & active-tooltip state even when
    // per-root override props are supplied (`LD-09`).
    let effective_provider_runtime = match parent_provider_ctx {
        Some(ctx) => ctx.runtime.read().with_overrides(
            props.delay_duration,
            props.skip_delay_duration,
            props.disable_hoverable_content,
            props.close_on_trigger_click,
            props.ignore_non_keyboard_focus,
        ),
        None => fallback_provider_runtime,
    };

    let is_open = open_signal();
    let prev_open = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(false)));
    let was_open = prev_open.get();
    if is_open && !was_open {
        if effective_provider_runtime
            .active_tooltip_id()
            .as_deref()
            != Some(relationships.root_id())
        {
            effective_provider_runtime.request_open(relationships.root_id(), true);
        }
    } else if !is_open
        && was_open
        && effective_provider_runtime
            .active_tooltip_id()
            .as_deref()
            == Some(relationships.root_id())
    {
        effective_provider_runtime.request_close(relationships.root_id());
    }
    prev_open.set(is_open);

    let tooltip = Tooltip::new(scope, is_open)
        .with_provider(effective_provider_runtime.provider().clone());

    let change_handler = move |next_open: bool| {
        open_ctl.set(next_open);
    };

    let runtime = use_tooltip_runtime_with_complete(
        tooltip,
        Some(effective_provider_runtime),
        change_handler,
        props.on_open_change_complete,
    );
    let mut runtime_sig = use_signal(|| runtime.clone());
    runtime_sig.set(runtime.clone());

    use_context_provider(|| TooltipRootContext {
        runtime: runtime_sig,
        open: open_signal,
        on_open_change: props.on_open_change,
        relationships: relationships.clone(),
    });
    use_context_provider(|| runtime.floating_tooltip_context());

    rsx! {
        {props.children}
    }
}

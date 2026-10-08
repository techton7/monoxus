use dioxus::prelude::*;

use crate::foundation::{
    overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost},
    shared::ScopeHandle,
};
use crate::tooltip::{
    components::{TooltipProviderContext, TooltipRootContext},
    relationships::TooltipRelationships,
    runtime::{use_tooltip_provider_runtime, use_tooltip_runtime},
    state::{Tooltip, TooltipProvider as TooltipProviderDef},
    types::TOOLTIP_GEOMETRY_NAMESPACE,
};

#[derive(Props, Clone, PartialEq)]
pub struct TooltipRootProps {
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
    pub delay_duration: Option<u64>,
    #[props(default)]
    pub skip_delay_duration: Option<u64>,
    #[props(default)]
    pub disable_hoverable_content: Option<bool>,
    #[props(default)]
    pub close_on_trigger_click: Option<bool>,
    #[props(default)]
    pub ignore_non_keyboard_focus: Option<bool>,
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
pub fn TooltipRoot(props: TooltipRootProps) -> Element {
    let open_signal = props
        .open
        .unwrap_or_else(|| use_signal(|| props.default_open.unwrap_or(false)));

    let scope_id = props.id.clone().unwrap_or_else(|| "root".to_string());
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

    let effective_provider_runtime = match parent_provider_ctx {
        Some(ctx) => {
            if props.delay_duration.is_some()
                || props.skip_delay_duration.is_some()
                || props.disable_hoverable_content.is_some()
                || props.close_on_trigger_click.is_some()
                || props.ignore_non_keyboard_focus.is_some()
            {
                fallback_provider_runtime
            } else {
                ctx.runtime.read().clone()
            }
        }
        None => fallback_provider_runtime,
    };

    let is_open = open_signal();

    if is_open
        && effective_provider_runtime
            .active_tooltip_id()
            .as_deref()
            != Some(relationships.root_id())
    {
        effective_provider_runtime.request_open(relationships.root_id(), true);
    }

    let mut tooltip = Tooltip::new(scope, is_open)
        .with_provider(effective_provider_runtime.provider().clone());

    if let Some(host) = props.portal_host {
        tooltip = tooltip.with_portal_host(host);
    }

    let side = props.side.unwrap_or(PlacementSide::Top);
    let mut floating = FloatingLayer::new(side)
        .with_side_offset(props.side_offset.unwrap_or(8.0))
        .with_namespace(TOOLTIP_GEOMETRY_NAMESPACE);
    if let Some(align) = props.align {
        floating = floating.with_align(align);
    }
    if let Some(align_offset) = props.align_offset {
        floating = floating.with_align_offset(align_offset);
    }
    tooltip = tooltip.with_floating(floating);

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

    let runtime = use_tooltip_runtime(tooltip, Some(effective_provider_runtime), change_handler);
    let mut runtime_sig = use_signal(|| runtime.clone());
    runtime_sig.set(runtime.clone());

    use_context_provider(|| TooltipRootContext {
        runtime: runtime_sig,
        open: open_signal,
        on_open_change,
        relationships: relationships.clone(),
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

use dioxus::prelude::*;

use crate::foundation::shared::ScopeHandle;
use crate::tooltip::{
    components::TooltipProviderContext,
    runtime::use_tooltip_provider_runtime,
    state::TooltipProvider as TooltipProviderDef,
};

#[derive(Props, Clone, PartialEq)]
pub struct TooltipProviderProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default = TooltipProviderDef::DEFAULT_DELAY_DURATION_MS)]
    pub delay_duration: u64,
    #[props(default = TooltipProviderDef::DEFAULT_SKIP_DELAY_DURATION_MS)]
    pub skip_delay_duration: u64,
    #[props(default = false)]
    pub disable_hoverable_content: bool,
    #[props(default = true)]
    pub close_on_trigger_click: bool,
    #[props(default = false)]
    pub ignore_non_keyboard_focus: bool,
    pub children: Element,
}

#[component]
pub fn TooltipProvider(props: TooltipProviderProps) -> Element {
    let scope_id = props
        .id
        .clone()
        .unwrap_or_else(|| "tooltip-provider".to_string());
    let scope = ScopeHandle::root("tooltip").child(scope_id);

    let provider_def = TooltipProviderDef::new(scope)
        .with_delay_duration_ms(props.delay_duration)
        .with_skip_delay_duration_ms(props.skip_delay_duration)
        .with_disable_hoverable_content(props.disable_hoverable_content)
        .with_close_on_trigger_click(props.close_on_trigger_click)
        .with_ignore_non_keyboard_focus(props.ignore_non_keyboard_focus);

    let runtime = use_tooltip_provider_runtime(provider_def);
    let runtime_sig = use_signal(|| runtime);

    use_context_provider(|| TooltipProviderContext {
        runtime: runtime_sig,
    });

    rsx! {
        {props.children}
    }
}

use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    shared::{ScopeHandle, use_stable_id},
    state::sync_signal,
};

use super::{
    runtime::{ProgressContext, use_progress_context, use_progress_runtime},
    state::Progress as ProgressDefinition,
};

#[derive(Props, Clone, PartialEq)]
pub struct ProgressRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<f64>,
    #[props(default = 100.0)]
    pub max: f64,
    #[props(default)]
    pub get_value_label: Option<Callback<(f64, f64), String>>,
    #[props(default = false)]
    pub loaded_state_alias: bool,
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
pub fn ProgressRoot(props: ProgressRootProps) -> Element {
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("progress").child(stable_id);
    let mut progress = ProgressDefinition::new(scope, props.value, props.max);
    if let Some(id) = props.id.as_ref() {
        progress = progress.with_root_id(id);
    }
    let runtime = use_progress_runtime(progress.clone(), props.loaded_state_alias);
    runtime.report_diagnostics();

    let mut runtime_sig = use_signal(|| runtime.clone());
    sync_signal(&mut runtime_sig, runtime.clone());
    use_context_provider(|| ProgressContext {
        runtime: runtime_sig,
    });

    let value_label = progress
        .value()
        .zip(props.get_value_label.as_ref())
        .map(|(value, callback)| callback.call((value, progress.max())));
    let attrs = runtime.root(value_label);

    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new(
        "aria-valuemin",
        attrs.aria_valuemin,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuemax",
        attrs.aria_valuemax,
        None,
        false,
    ));
    push_attr_opt(&mut base, "aria-valuenow", attrs.aria_valuenow);
    push_attr_opt(&mut base, "aria-valuetext", attrs.aria_valuetext);
    base.push(Attribute::new("data-state", attrs.data_state, None, false));
    push_attr_opt(&mut base, "data-value", attrs.data_value);
    base.push(Attribute::new("data-max", attrs.data_max, None, false));
    base.push(Attribute::new("data-min", attrs.data_min, None, false));
    push_attr_opt(&mut base, "data-indeterminate", attrs.data_indeterminate);
    base.push(Attribute::new(
        "data-progress-root",
        attrs.data_progress_root,
        None,
        false,
    ));

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

pub use ProgressRoot as Progress;

#[derive(Props, Clone, PartialEq)]
pub struct ProgressIndicatorProps {
    #[props(default, into)]
    pub id: Option<String>,
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
pub fn ProgressIndicator(props: ProgressIndicatorProps) -> Element {
    let context = use_progress_context();
    let runtime = context.runtime();
    let mut attrs = runtime.indicator();
    if let Some(id) = props.id {
        attrs.id = id;
    }

    let mut base = Vec::with_capacity(7);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("data-state", attrs.data_state, None, false));
    push_attr_opt(&mut base, "data-value", attrs.data_value);
    base.push(Attribute::new("data-max", attrs.data_max, None, false));
    base.push(Attribute::new("data-min", attrs.data_min, None, false));
    push_attr_opt(&mut base, "data-indeterminate", attrs.data_indeterminate);
    base.push(Attribute::new(
        "data-progress-indicator",
        attrs.data_progress_indicator,
        None,
        false,
    ));

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

pub use ProgressIndicator as Indicator;

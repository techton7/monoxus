use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::shared::{ScopeHandle, use_stable_id};

use super::{
    attrs::{ProgressIndicatorAttributes, ProgressRootAttributes},
    runtime::{ProgressContext, use_progress_context, use_progress_runtime},
    state::Progress as ProgressDefinition,
};

#[derive(Clone)]
pub struct ProgressRootRenderChildProps {
    pub attrs: ProgressRootAttributes,
    pub children: Element,
}

#[derive(Clone)]
pub struct ProgressIndicatorRenderChildProps {
    pub attrs: ProgressIndicatorAttributes,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ProgressRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<f64>,
    #[props(default = 100.0)]
    pub max: f64,
    #[props(default)]
    pub get_value_label: Option<Callback<(f64, f64), String>>,
    #[props(default = false)]
    pub loaded_state_alias: bool,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ProgressRootRenderChildProps, Element>>,
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
    runtime_sig.set(runtime.clone());
    use_context_provider(|| ProgressContext {
        runtime: runtime_sig,
    });

    if props.as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::ProgressRoot: as_child requires render_child; rendering the default div"
        );
    }

    let value_label = progress
        .value()
        .zip(props.get_value_label.as_ref())
        .map(|(value, callback)| callback.call((value, progress.max())));
    let attrs = runtime.root(value_label);

    if let Some(render_child) = props.render_child.as_ref().filter(|_| props.as_child) {
        return render_child.call(ProgressRootRenderChildProps {
            attrs,
            children: props.children,
        });
    }

    rsx! {
        div {
            id: attrs.id.clone(),
            class: props.class,
            style: props.style,
            role: attrs.role,
            "aria-valuemin": attrs.aria_valuemin.clone(),
            "aria-valuemax": attrs.aria_valuemax.clone(),
            "aria-valuenow": attrs.aria_valuenow.clone(),
            "aria-valuetext": attrs.aria_valuetext.clone(),
            "data-state": attrs.data_state,
            "data-value": attrs.data_value.clone(),
            "data-max": attrs.data_max.clone(),
            "data-min": attrs.data_min,
            "data-indeterminate": attrs.data_indeterminate,
            "data-progress-root": attrs.data_progress_root,
            onmounted: move |event| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(event);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

pub use ProgressRoot as Progress;

#[derive(Props, Clone, PartialEq)]
pub struct ProgressIndicatorProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ProgressIndicatorRenderChildProps, Element>>,
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

    if props.as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::ProgressIndicator: as_child requires render_child; rendering the default div"
        );
    }

    if let Some(render_child) = props.render_child.as_ref().filter(|_| props.as_child) {
        return render_child.call(ProgressIndicatorRenderChildProps {
            attrs,
            children: props.children,
        });
    }

    rsx! {
        div {
            id: attrs.id.clone(),
            class: props.class,
            style: props.style,
            "data-state": attrs.data_state,
            "data-value": attrs.data_value.clone(),
            "data-max": attrs.data_max.clone(),
            "data-min": attrs.data_min,
            "data-indeterminate": attrs.data_indeterminate,
            "data-progress-indicator": attrs.data_progress_indicator,
            onmounted: move |event| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(event);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

pub use ProgressIndicator as Indicator;

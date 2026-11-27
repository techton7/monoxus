use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    compose::{merge_attributes_with_part, merge_global_attributes},
    shared::data_disabled_attr,
};
use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverTriggerRenderProps {
    pub id: String,
    pub disabled: bool,
    pub aria_haspopup: &'static str,
    pub aria_expanded: bool,
    pub aria_controls: Option<String>,
    pub data_state: &'static str,
    pub data_disabled: Option<&'static str>,
    pub tabindex: Option<&'static str>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_mounted: EventHandler<MountedEvent>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct PopoverTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<PopoverTriggerRenderProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Specification for [`render_popover_trigger_surface`] (plain Rust function, zero internal
/// `#[component]` overhead per `Dioxus #5894`, shared by `PopoverTrigger`, `DatePickerTrigger`,
/// and `DateRangePickerTrigger`).
#[allow(dead_code)]
pub(crate) struct PopoverTriggerSurfaceSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub disabled: bool,
    pub as_child: bool,
    pub extra_data_attr: Option<&'static str>,
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    pub on_blur: Option<EventHandler<FocusEvent>>,
    pub on_focus_in: Option<EventHandler<FocusEvent>>,
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// View passed to `render_as_child` inside [`render_popover_trigger_surface`].
#[allow(dead_code)]
pub(crate) struct PopoverTriggerSurfaceView {
    pub id: String,
    pub disabled: bool,
    pub is_open: bool,
    pub aria_haspopup: &'static str,
    pub aria_expanded: bool,
    pub aria_controls: Option<String>,
    pub data_state: &'static str,
    pub data_disabled: Option<&'static str>,
    pub tabindex: Option<&'static str>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_mounted: EventHandler<MountedEvent>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust monomorphized render helper for popover triggers (zero internal `#[component]`,
/// compliant with `Dioxus #5894` and `Dioxus #3717`).
pub(crate) fn render_popover_trigger_surface<F>(
    spec: PopoverTriggerSurfaceSpec,
    render_as_child: F,
) -> Element
where
    F: FnOnce(PopoverTriggerSurfaceView) -> Option<Element>,
{
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();
    let trigger = runtime.trigger();
    let trigger_id = spec.id.unwrap_or_else(|| trigger.id().to_string());

    let latest_tid = use_hook(|| Rc::new(RefCell::new(trigger_id.clone())));
    {
        let mut prev = latest_tid.borrow_mut();
        if *prev != trigger_id {
            runtime.unregister_trigger_id(&prev);
            *prev = trigger_id.clone();
        }
    }
    runtime.register_trigger_id(&trigger_id);
    let tid_for_drop = Rc::clone(&latest_tid);
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        let current = tid_for_drop.borrow().clone();
        runtime_for_drop.unregister_trigger_id(&current);
    });

    let disabled = spec.disabled;
    let is_open = runtime.is_open();
    let tabindex = (!disabled).then_some("0");
    let data_disabled = data_disabled_attr(disabled);
    let data_state = match trigger.data_state() {
        crate::foundation::state::DataState::Open => "open",
        _ => "closed",
    };
    let aria_expanded = trigger.aria_expanded();
    let aria_controls = trigger.aria_controls().map(str::to_owned);

    let consumer_mounted = spec.on_mounted;
    let consumer_click = spec.on_click;
    let consumer_key_down = spec.on_key_down;
    let mut blur_slot = use_hook(|| CopyValue::new(spec.on_blur));
    blur_slot.set(spec.on_blur);
    let mut focus_in_slot = use_hook(|| CopyValue::new(spec.on_focus_in));
    focus_in_slot.set(spec.on_focus_in);
    let mut focus_out_slot = use_hook(|| CopyValue::new(spec.on_focus_out));
    focus_out_slot.set(spec.on_focus_out);

    let attrs_with_part = match spec.extra_data_attr {
        Some(part_attr) => merge_attributes_with_part(spec.attributes, part_attr),
        None => spec.attributes,
    };

    if spec.as_child {
        let on_mounted = {
            let mut mount_cb = runtime.mount_trigger();
            EventHandler::new(move |evt: MountedEvent| {
                mount_cb(evt.clone());
                if let Some(handler) = consumer_mounted {
                    handler.call(evt);
                }
            })
        };
        let on_pointer_down = {
            let mut pd_cb = runtime.trigger_pointer_down();
            EventHandler::new(move |evt: PointerEvent| {
                if !disabled {
                    pd_cb(evt);
                }
            })
        };
        let on_click = {
            let mut click_cb = runtime.trigger_click();
            EventHandler::new(move |evt: MouseEvent| {
                if disabled {
                    return;
                }
                click_cb(evt.clone());
                if let Some(handler) = consumer_click {
                    handler.call(evt);
                }
            })
        };
        let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
            if let Some(handler) = consumer_key_down {
                handler.call(evt);
            }
        });
        let view = PopoverTriggerSurfaceView {
            id: trigger_id.clone(),
            disabled,
            is_open,
            aria_haspopup: "dialog",
            aria_expanded,
            aria_controls: aria_controls.clone(),
            data_state,
            data_disabled,
            tabindex,
            class: spec.class.clone(),
            style: spec.style.clone(),
            on_mounted,
            on_pointer_down,
            on_click,
            on_key_down,
            attributes: attrs_with_part.clone(),
            children: spec.children.clone(),
        };
        if let Some(el) = render_as_child(view) {
            return el;
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PopoverTrigger: as_child requires render_child; rendering the default button"
        );
    }

    let merged = merge_global_attributes(
        spec.class.as_deref(),
        spec.style.as_deref(),
        attrs_with_part,
    );

    let mut mount_cb = runtime.mount_trigger();
    let mut pd_cb = runtime.trigger_pointer_down();
    let mut click_cb = runtime.trigger_click();

    rsx! {
        button {
            id: "{trigger_id}",
            r#type: "button",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            disabled: disabled,
            tabindex: tabindex,
            aria_haspopup: "dialog",
            aria_expanded: "{aria_expanded}",
            aria_controls: aria_controls.as_deref(),
            "data-state": data_state,
            "data-disabled": data_disabled,
            onmounted: move |evt: MountedEvent| {
                mount_cb(evt.clone());
                if let Some(handler) = consumer_mounted {
                    handler.call(evt);
                }
            },
            onpointerdown: move |evt: PointerEvent| {
                if !disabled {
                    pd_cb(evt);
                }
            },
            onclick: move |evt: MouseEvent| {
                if disabled {
                    return;
                }
                click_cb(evt.clone());
                if let Some(handler) = consumer_click {
                    handler.call(evt);
                }
            },
            onkeydown: move |evt: KeyboardEvent| {
                if let Some(handler) = consumer_key_down {
                    handler.call(evt);
                }
            },
            onblur: move |evt: FocusEvent| {
                if let Some(handler) = blur_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            onfocusin: move |evt: FocusEvent| {
                if let Some(handler) = focus_in_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            onfocusout: move |evt: FocusEvent| {
                if let Some(handler) = focus_out_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {spec.children}
        }
    }
}

#[component]
pub fn PopoverTrigger(props: PopoverTriggerProps) -> Element {
    let render_child = props.render_child;
    render_popover_trigger_surface(
        PopoverTriggerSurfaceSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            disabled: props.disabled,
            as_child: props.as_child,
            extra_data_attr: None,
            on_click: props.on_click,
            on_key_down: props.on_key_down,
            on_blur: None,
            on_focus_in: None,
            on_focus_out: None,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(PopoverTriggerRenderProps {
                    id: view.id,
                    disabled: view.disabled,
                    aria_haspopup: view.aria_haspopup,
                    aria_expanded: view.aria_expanded,
                    aria_controls: view.aria_controls,
                    data_state: view.data_state,
                    data_disabled: view.data_disabled,
                    tabindex: view.tabindex,
                    class: view.class,
                    style: view.style,
                    on_mounted: view.on_mounted,
                    on_pointer_down: view.on_pointer_down,
                    on_click: view.on_click,
                    on_key_down: view.on_key_down,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}

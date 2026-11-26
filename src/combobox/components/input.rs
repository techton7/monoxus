use dioxus::prelude::*;

use crate::combobox::{attrs::ComboboxInputAttrs, runtime::use_combobox_context};

#[derive(Clone, PartialEq)]
pub struct ComboboxInputRenderProps {
    pub attrs: ComboboxInputAttrs,
    pub value: String,
    pub placeholder: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_input: EventHandler<FormEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_composition_start: EventHandler<CompositionEvent>,
    pub on_composition_end: EventHandler<CompositionEvent>,
    pub on_mounted: EventHandler<MountedEvent>,
    pub attributes: Vec<Attribute>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxInputProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub default_value: Option<String>,
    /// Optional bidirectional override for `ComboboxRootProps.clear_on_deselect` (`4B D-10`).
    #[props(default)]
    pub clear_on_deselect: Option<bool>,
    #[props(default)]
    pub placeholder: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_input: Option<EventHandler<FormEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxInputRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ComboboxInput(props: ComboboxInputProps) -> Element {
    let ctx = use_combobox_context();
    let id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().input_id().to_owned());
    ctx.runtime.set_input_id(id.clone());

    // 4B D-10 / DEF-CBX-09: Bidirectional override (`Some(true)` / `Some(false)` / `None` inherit root).
    ctx.runtime.set_input_clear_on_deselect(props.clear_on_deselect);

    // Mount-only `default_value` seed per bits-ui ComboboxInput contract.
    let seeded = use_hook(|| std::cell::Cell::new(false));
    if !seeded.get() {
        seeded.set(true);
        if let Some(ref default_val) = props.default_value {
            if ctx.runtime.input_value_untracked().is_empty() && !default_val.is_empty() {
                ctx.runtime.set_input_value(default_val.clone(), false);
            }
        }
    }

    let attrs = ctx.runtime.input_attributes(id, props.disabled);
    let current_value = ctx.runtime.input_value();

    let input_rt = ctx.runtime.clone();
    let user_on_input = props.on_input;
    let on_input = EventHandler::new(move |evt: FormEvent| {
        if let Some(ref cb) = user_on_input {
            cb.call(evt.clone());
        }
        input_rt.handle_input_change(evt.value());
    });

    let key_rt = ctx.runtime.clone();
    let user_on_keydown = props.on_key_down;
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if let Some(ref cb) = user_on_keydown {
            cb.call(evt.clone());
        }
        if !evt.default_action_enabled() {
            return;
        }
        key_rt.handle_input_keydown(&evt);
    });

    // 4B D-5: Wire Web composition events alongside `evt.data().is_composing()`.
    let comp_start_rt = ctx.runtime.clone();
    let on_composition_start = EventHandler::new(move |_evt: CompositionEvent| {
        comp_start_rt.set_composing(true);
    });

    let comp_end_rt = ctx.runtime.clone();
    let on_composition_end = EventHandler::new(move |_evt: CompositionEvent| {
        comp_end_rt.set_composing(false);
    });

    let mount_rt = ctx.runtime.clone();
    let on_mounted = EventHandler::new(move |evt: MountedEvent| {
        mount_rt.capture_input_mount(evt.data());
    });

    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            return render_child.call(ComboboxInputRenderProps {
                attrs,
                value: current_value,
                placeholder: props.placeholder,
                class: props.class,
                style: props.style,
                on_input,
                on_key_down,
                on_composition_start,
                on_composition_end,
                on_mounted,
                attributes: props.attributes,
            });
        }
    }

    rsx! {
        input {
            id: "{attrs.id()}",
            r#type: "text",
            role: "{attrs.role()}",
            "aria-autocomplete": "{attrs.aria_autocomplete()}",
            "aria-expanded": "{attrs.aria_expanded()}",
            "aria-controls": attrs.aria_controls(),
            "aria-activedescendant": attrs.aria_activedescendant(),
            disabled: attrs.is_disabled(),
            autocomplete: "{attrs.autocomplete()}",
            "data-combobox-input": "",
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": attrs.data_disabled(),
            value: "{current_value}",
            placeholder: props.placeholder.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            oninput: move |evt| on_input.call(evt),
            onkeydown: move |evt| on_key_down.call(evt),
            oncompositionstart: move |evt| on_composition_start.call(evt),
            oncompositionend: move |evt| on_composition_end.call(evt),
            onmounted: move |evt| on_mounted.call(evt),
            ..props.attributes,
        }
    }
}

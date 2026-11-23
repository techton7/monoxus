use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    browser::{focus_element_by_id, is_element_focused_by_id},
    form::use_form_control_sync_with_epoch,
    shared::{ScopeHandle, use_stable_id},
    state::debug_assert_mode_stable,
};

use super::super::{
    attrs::{PinInputInputAttributes, PinInputRootAttributes},
    relationships::PinInputRelationships,
    runtime::{PinInputContext, use_pin_input_runtime},
    state::sanitize_pin_value,
    types::{
        PinInputAutoComplete, PinInputMode, PinInputPattern, PinInputSelection, PinInputSnapshot,
        PinInputTextAlign,
    },
};

/// Render props passed to `render_child` when [`PinInputRoot`] sets `as_child: Some(true)`.
#[derive(Clone)]
pub struct PinInputRootRenderChildProps {
    pub attrs: PinInputRootAttributes,
    pub input_attrs: PinInputInputAttributes,
    pub snapshot: PinInputSnapshot,
    pub children: Element,
}

/// Properties for [`PinInputRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct PinInputRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub input_id: Option<String>,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_complete: Option<EventHandler<String>>,
    #[props(default)]
    pub paste_transformer: Option<Callback<String, String>>,
    #[props(default = 6)]
    pub max_length: usize,
    #[props(default)]
    pub pattern: PinInputPattern,
    #[props(default)]
    pub text_align: PinInputTextAlign,
    #[props(default)]
    pub input_mode: PinInputMode,
    #[props(default)]
    pub autocomplete: PinInputAutoComplete,
    #[props(default = false)]
    pub mask: bool,
    #[props(default = '•')]
    pub mask_char: char,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub auto_focus: bool,
    #[props(default)]
    pub placeholder: Option<String>,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default)]
    pub as_child: Option<bool>,
    #[props(default)]
    pub render_child: Option<Callback<PinInputRootRenderChildProps, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<PinInputSnapshot, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless single-hidden-`<input>` OTP / PIN input container coordinating value sanitization,
/// selection mirroring into [`crate::pin_input::PinInputCell`] parts, paste transformation, and
/// `<form>` submission/reset sync.
#[component]
pub fn PinInputRoot(props: PinInputRootProps) -> Element {
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope_id = stable_id.into_owned();
    let scope = use_hook(|| ScopeHandle::root("pin-input").child(scope_id.clone()));
    let initial_rels = use_hook(|| PinInputRelationships::new(scope));
    let resolved_root_id = props
        .id
        .clone()
        .unwrap_or_else(|| initial_rels.root_id().to_string());
    let resolved_input_id = props
        .input_id
        .clone()
        .unwrap_or_else(|| initial_rels.input_id().to_string());

    let initially_controlled = use_hook(|| props.value.is_some());
    let is_controlled = props.value.is_some();
    debug_assert_mode_stable(initially_controlled, is_controlled);

    let effective_max_length = props.max_length.max(1);
    let initial_value_for_reset = use_hook(|| {
        sanitize_pin_value(
            props
                .value
                .as_deref()
                .or(props.default_value.as_deref())
                .unwrap_or(""),
            effective_max_length,
            &props.pattern,
        )
    });

    let mut runtime = use_pin_input_runtime(
        initial_value_for_reset.clone(),
        is_controlled,
        effective_max_length,
        props.pattern.clone(),
        props.paste_transformer,
        props.on_value_change,
        props.on_complete,
        props.text_align,
        props.input_mode,
        props.autocomplete,
        props.mask,
        props.mask_char,
        props.disabled,
        props.read_only,
        props.required,
        props.auto_focus,
        props.placeholder.clone(),
        props.name.clone(),
        props.form.clone(),
        scope_id.clone(),
        resolved_root_id.clone(),
        resolved_input_id.clone(),
    );

    // Keep reactive props synchronized with runtime signals.
    {
        *runtime.pattern.borrow_mut() = props.pattern.clone();
        *runtime.paste_transformer.borrow_mut() = props.paste_transformer;
        *runtime.on_value_change.borrow_mut() = props.on_value_change;
        *runtime.on_complete.borrow_mut() = props.on_complete;

        if *runtime.is_controlled.peek() != is_controlled {
            runtime.is_controlled.set(is_controlled);
        }
        if *runtime.max_length.peek() != effective_max_length {
            runtime.max_length.set(effective_max_length);
        }
        if *runtime.text_align.peek() != props.text_align {
            runtime.text_align.set(props.text_align);
        }
        if *runtime.input_mode.peek() != props.input_mode {
            runtime.input_mode.set(props.input_mode);
        }
        if *runtime.autocomplete.peek() != props.autocomplete {
            runtime.autocomplete.set(props.autocomplete);
        }
        if *runtime.mask.peek() != props.mask {
            runtime.mask.set(props.mask);
        }
        if *runtime.mask_char.peek() != props.mask_char {
            runtime.mask_char.set(props.mask_char);
        }
        if *runtime.disabled.peek() != props.disabled {
            runtime.disabled.set(props.disabled);
        }
        if *runtime.read_only.peek() != props.read_only {
            runtime.read_only.set(props.read_only);
        }
        if *runtime.required.peek() != props.required {
            runtime.required.set(props.required);
        }
        if *runtime.auto_focus.peek() != props.auto_focus {
            runtime.auto_focus.set(props.auto_focus);
        }
        if *runtime.placeholder.peek() != props.placeholder {
            runtime.placeholder.set(props.placeholder.clone());
        }
        if *runtime.name.peek() != props.name {
            runtime.name.set(props.name.clone());
        }
        if *runtime.form.peek() != props.form {
            runtime.form.set(props.form.clone());
        }
        if *runtime.scope_id.peek() != scope_id {
            runtime.scope_id.set(scope_id);
        }
        if *runtime.root_dom_id.peek() != resolved_root_id {
            runtime.root_dom_id.set(resolved_root_id.clone());
        }
        if *runtime.input_dom_id.peek() != resolved_input_id {
            runtime.input_dom_id.set(resolved_input_id.clone());
        }

        if let Some(ref controlled_val) = props.value {
            let sanitized =
                sanitize_pin_value(controlled_val, effective_max_length, &props.pattern);
            if *runtime.value.peek() != sanitized {
                runtime.value.set(sanitized);
            }
        } else {
            let current = runtime.value.peek().clone();
            let clamped = sanitize_pin_value(&current, effective_max_length, &props.pattern);
            if clamped != current {
                runtime.value.set(clamped);
            }
        }

        if props.disabled && *runtime.is_focused.peek() {
            runtime.is_focused.set(false);
            *runtime.prev_selection.borrow_mut() = PinInputSelection::empty();
            runtime.selection.set(PinInputSelection::empty());
        }
    }

    // Controlled edge-transition `on_complete` observer (runs post-render).
    use_effect({
        let runtime = runtime.clone();
        move || {
            let current = (runtime.value)();
            let max_len = (runtime.max_length)();
            runtime.check_and_fire_on_complete(&current, max_len);
        }
    });

    // Optional autofocus on mount when enabled.
    let did_autofocus = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(false)));
    use_effect({
        let input_id = resolved_input_id.clone();
        let auto_focus = props.auto_focus;
        let disabled = props.disabled;
        let did_autofocus = std::rc::Rc::clone(&did_autofocus);
        move || {
            if auto_focus && !disabled && !did_autofocus.get() {
                did_autofocus.set(true);
                if !is_element_focused_by_id(&input_id) {
                    focus_element_by_id(&input_id);
                }
            }
        }
    });

    let mut reset_runtime = runtime.clone();
    let reset_initial = initial_value_for_reset.clone();
    let (_should_render_hidden, reset_epoch) = use_form_control_sync_with_epoch(
        &resolved_input_id,
        props.name.as_deref(),
        props.form.as_deref(),
        move || {
            reset_runtime.reset_to(&reset_initial);
        },
    );

    use_context_provider(|| PinInputContext {
        runtime: runtime.clone(),
    });

    let snapshot = runtime.snapshot();
    let root_attrs = runtime.root_attributes(Some(resolved_root_id));
    let input_attrs = runtime.input_attributes();

    let body = if let Some(ref render_children) = props.render_children {
        render_children.call(snapshot.clone())
    } else {
        props.children
    };

    let overlay_input = rsx! {
        for epoch in [reset_epoch] {
            input {
                key: "{epoch}",
                id: "{input_attrs.id}",
                r#type: "{input_attrs.r#type}",
                autocomplete: "{input_attrs.autocomplete}",
                inputmode: "{input_attrs.inputmode}",
                pattern: input_attrs.pattern,
                maxlength: "{input_attrs.maxlength}",
                value: "{input_attrs.value}",
                placeholder: input_attrs.placeholder.as_deref(),
                disabled: input_attrs.disabled,
                readonly: input_attrs.readonly,
                required: input_attrs.required,
                autofocus: input_attrs.autofocus,
                name: input_attrs.name.as_deref(),
                form: input_attrs.form.as_deref(),
                style: "{input_attrs.style}",
                "data-monoxus-pin-input-input": "{input_attrs.data_monoxus_pin_input_input}",
                "data-pin-input-input-mss": input_attrs.data_pin_input_input_mss.as_deref(),
                "data-pin-input-input-mse": input_attrs.data_pin_input_input_mse.as_deref(),
                onfocus: {
                    let mut runtime = runtime.clone();
                    move |_| runtime.handle_focus()
                },
                onblur: {
                    let mut runtime = runtime.clone();
                    move |_| runtime.handle_blur()
                },
                onmouseenter: {
                    let mut runtime = runtime.clone();
                    move |_| runtime.is_hovering.set(true)
                },
                onmouseleave: {
                    let mut runtime = runtime.clone();
                    move |_| runtime.is_hovering.set(false)
                },
                onkeydown: {
                    let mut runtime = runtime.clone();
                    move |evt: KeyboardEvent| {
                        let mods = evt.modifiers();
                        let ctrl_or_meta = mods.contains(Modifiers::CONTROL) || mods.contains(Modifiers::META);
                        let shift = mods.contains(Modifiers::SHIFT);
                        if runtime.handle_keydown(&evt.key(), ctrl_or_meta, shift, evt.data().is_composing()) {
                            evt.prevent_default();
                        }
                    }
                },
                onkeyup: {
                    let mut runtime = runtime.clone();
                    move |_| runtime.sync_selection_from_dom()
                },
                onpointerdown: {
                    let mut runtime = runtime.clone();
                    let input_id = input_attrs.id.clone();
                    move |evt: PointerEvent| {
                        let coords = evt.data().client_coordinates();
                        if runtime.handle_pointer_down(coords.x, coords.y) {
                            evt.prevent_default();
                            if !is_element_focused_by_id(&input_id) {
                                focus_element_by_id(&input_id);
                            }
                        }
                    }
                },
                onpointermove: {
                    let mut runtime = runtime.clone();
                    move |evt: PointerEvent| {
                        let coords = evt.data().client_coordinates();
                        let buttons_held = !evt.data().held_buttons().is_empty();
                        if runtime.handle_pointer_move(coords.x, coords.y, buttons_held) {
                            evt.prevent_default();
                        }
                    }
                },
                onpointerup: {
                    let mut runtime = runtime.clone();
                    move |evt: PointerEvent| {
                        let coords = evt.data().client_coordinates();
                        runtime.handle_pointer_up(coords.x, coords.y);
                    }
                },
                onpaste: {
                    let mut runtime = runtime.clone();
                    move |evt: ClipboardEvent| {
                        if let Some(text) = evt.data().data_transfer().get_as_text() {
                            evt.prevent_default();
                            runtime.handle_paste(&text);
                        }
                    }
                },
                oninput: {
                    let mut runtime = runtime.clone();
                    move |evt: FormEvent| runtime.handle_input(evt.value())
                },
            }
        }
    };

    let as_child = props.as_child.unwrap_or(false);
    if as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PinInputRoot: as_child requires render_child; rendering the default div"
        );
    }
    if let Some(render_child) = props.render_child.as_ref().filter(|_| as_child) {
        let combined_children = rsx! {
            {body}
            {overlay_input}
        };
        return render_child.call(PinInputRootRenderChildProps {
            attrs: root_attrs,
            input_attrs,
            snapshot,
            children: combined_children,
        });
    }

    let merged_style = match props.style.as_deref().filter(|s| !s.is_empty()) {
        Some(extra) => format!("{} {}", root_attrs.style, extra),
        None => root_attrs.style.clone(),
    };

    rsx! {
        div {
            id: "{root_attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{merged_style}",
            aria_disabled: root_attrs.aria_disabled,
            "data-disabled": root_attrs.data_disabled,
            "data-monoxus-pin-input-root": "{root_attrs.data_monoxus_pin_input_root}",
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {body}
            {overlay_input}
        }
    }
}

pub use PinInputRoot as PinInput;

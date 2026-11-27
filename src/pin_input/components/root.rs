use std::borrow::Cow;

use dioxus::core::AttributeValue;
use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    browser::{focus_element_by_id, is_element_focused_by_id},
    compose::{
        extract_aria_invalid_and_describedby, join_styles, merge_global_attributes,
        resolve_invalid_and_describedby,
    },
    form::use_form_control_sync_with_epoch,
    shared::{ScopeHandle, use_stable_id},
    state::sync_signal,
};

use super::super::{
    attrs::{PinInputInputAttributes, PinInputRootAttributes},
    relationships::PinInputRelationships,
    runtime::{PinInputContext, use_pin_input_runtime},
    state::sanitize_pin_value,
    types::{
        PinInputAutoComplete, PinInputMode, PinInputPattern, PinInputSnapshot, PinInputTextAlign,
    },
};

fn extract_aria_label_attrs(
    attrs: Vec<Attribute>,
) -> (Option<String>, Option<String>, Vec<Attribute>) {
    let has_target = attrs.iter().any(|attr| {
        attr.namespace.is_none() && matches!(attr.name, "aria-labelledby" | "aria-label")
    });
    if !has_target {
        return (None, None, attrs);
    }

    let mut labelledby: Option<String> = None;
    let mut label: Option<String> = None;
    let mut rest = Vec::with_capacity(attrs.len());
    for attr in attrs {
        if attr.namespace.is_none() && attr.name == "aria-labelledby" {
            match attr.value {
                AttributeValue::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        labelledby = Some(trimmed.to_owned());
                    }
                }
                AttributeValue::None => {}
                other => rest.push(Attribute {
                    value: other,
                    ..attr
                }),
            }
        } else if attr.namespace.is_none() && attr.name == "aria-label" {
            match attr.value {
                AttributeValue::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        label = Some(trimmed.to_owned());
                    }
                }
                AttributeValue::None => {}
                other => rest.push(Attribute {
                    value: other,
                    ..attr
                }),
            }
        } else {
            rest.push(attr);
        }
    }
    (labelledby, label, rest)
}

/// Render props passed to `render_child` when [`PinInputRoot`] sets `as_child: true`.
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
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
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
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default = false)]
    pub auto_focus: bool,
    #[props(default)]
    pub placeholder: Option<String>,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default)]
    pub as_child: bool,
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
    let field_ctx = try_use_field_context();
    let control_ctx = try_use_field_control_context();

    let effective_name = props
        .name
        .clone()
        .or_else(|| field_ctx.and_then(|f| f.name()));
    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_read_only = props.read_only || field_ctx.is_some_and(|f| f.is_read_only());
    let effective_required = props.required || field_ctx.is_some_and(|f| f.is_required());

    let extracted = extract_aria_invalid_and_describedby(props.attributes);
    let (attr_labelledby, attr_label, rest_attrs) = extract_aria_label_attrs(extracted.rest);

    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let field_labelledby = if let Some(ctrl_ctx) = control_ctx {
        ctrl_ctx.mounted_label_id().or_else(|| {
            field_ctx
                .filter(|f| !f.is_fieldset())
                .and_then(|f| f.aria_labelledby())
        })
    } else {
        field_ctx.and_then(|f| f.aria_labelledby())
    };
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        props.invalid,
        extracted.aria_invalid,
        field_ctx.map(|f| f.is_invalid()),
        false,
        props.aria_describedby.as_deref(),
        extracted.aria_describedby.as_deref(),
        field_describedby.as_deref(),
    );
    let effective_labelledby = props
        .aria_labelledby
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or(attr_labelledby)
        .or(field_labelledby);
    let effective_label = props
        .aria_label
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or(attr_label);

    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope_id = stable_id.into_owned();
    let scope = use_hook(|| ScopeHandle::root("pin-input").child(scope_id.clone()));
    let initial_rels = use_hook(|| PinInputRelationships::new(scope));
    let resolved_root_id = props
        .id
        .clone()
        .unwrap_or_else(|| initial_rels.root_id().to_string());

    let explicit_input_id = props.input_id.clone().filter(|id| !id.trim().is_empty());
    let resolved_input_id = if let Some(ref explicit) = explicit_input_id {
        if let Some(ctrl_ctx) = control_ctx {
            if !ctrl_ctx.is_explicit_id_locked() {
                ctrl_ctx.sync_control_id(explicit.clone());
            }
        } else if let Some(f_ctx) = field_ctx
            && !f_ctx.is_fieldset()
        {
            f_ctx.set_primary_control_id(Some(explicit.clone()));
        }
        explicit.clone()
    } else if let Some(ctrl_ctx) = control_ctx {
        ctrl_ctx.control_id()
    } else if let Some(f_ctx) = field_ctx
        && !f_ctx.is_fieldset()
    {
        f_ctx.resolved_control_id()
    } else {
        initial_rels.input_id().to_string()
    };

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
        props.value.clone(),
        props.default_value.clone().unwrap_or_default(),
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
        effective_disabled,
        effective_read_only,
        effective_required,
        props.auto_focus,
        props.placeholder.clone(),
        effective_name.clone(),
        props.form.clone(),
        scope_id,
        resolved_root_id.clone(),
        resolved_input_id.clone(),
    );
    sync_signal(&mut runtime.invalid, effective_invalid);
    sync_signal(&mut runtime.aria_describedby, effective_describedby);
    sync_signal(&mut runtime.aria_labelledby, effective_labelledby);
    sync_signal(&mut runtime.aria_label, effective_label);

    let mut on_focus_slot = use_hook(|| CopyValue::new(props.on_focus));
    on_focus_slot.set(props.on_focus);
    let mut on_blur_slot = use_hook(|| CopyValue::new(props.on_blur));
    on_blur_slot.set(props.on_blur);

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
        let disabled = effective_disabled;
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
    let reset_initial = initial_value_for_reset;
    let (_should_render_hidden, reset_epoch) = use_form_control_sync_with_epoch(
        &resolved_input_id,
        effective_name.as_deref(),
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
                aria_invalid: input_attrs.aria_invalid,
                aria_describedby: input_attrs.aria_describedby.as_deref(),
                aria_labelledby: input_attrs.aria_labelledby.as_deref(),
                aria_label: input_attrs.aria_label.as_deref(),
                style: "{input_attrs.style}",
                "data-invalid": input_attrs.data_invalid,
                "data-monoxus-pin-input-input": "{input_attrs.data_monoxus_pin_input_input}",
                "data-pin-input-input-mss": input_attrs.data_pin_input_input_mss.as_deref(),
                "data-pin-input-input-mse": input_attrs.data_pin_input_input_mse.as_deref(),
                onfocus: {
                    let mut runtime = runtime.clone();
                    move |evt: FocusEvent| {
                        runtime.handle_focus();
                        if let Some(handler) = on_focus_slot.try_read().ok().and_then(|s| *s) {
                            handler.call(evt);
                        }
                    }
                },
                onblur: {
                    let mut runtime = runtime.clone();
                    move |evt: FocusEvent| {
                        runtime.handle_blur();
                        if let Some(handler) = on_blur_slot.try_read().ok().and_then(|s| *s) {
                            handler.call(evt);
                        }
                    }
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

    if props.as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PinInputRoot: as_child requires render_child; rendering the default div"
        );
    }
    if let Some(render_child) = props.render_child.as_ref().filter(|_| props.as_child) {
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

    let combined_style = join_styles([root_attrs.style.as_str(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        rest_attrs,
    );

    rsx! {
        div {
            id: "{root_attrs.id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            aria_disabled: root_attrs.aria_disabled,
            "data-disabled": root_attrs.data_disabled,
            "data-invalid": root_attrs.data_invalid,
            "data-monoxus-pin-input-root": "{root_attrs.data_monoxus_pin_input_root}",
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {body}
            {overlay_input}
        }
    }
}

pub use PinInputRoot as PinInput;

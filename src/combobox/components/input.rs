use dioxus::prelude::*;

use crate::combobox::{attrs::ComboboxInputAttrs, runtime::use_combobox_context};
use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::compose::{
    extract_aria_invalid_and_describedby, merge_aria_describedby, merge_global_attributes,
    resolve_invalid_and_describedby,
};

#[derive(Clone, PartialEq)]
pub struct ComboboxInputRenderProps {
    pub attrs: ComboboxInputAttrs,
    pub value: String,
    pub placeholder: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_input: EventHandler<FormEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_focus: EventHandler<FocusEvent>,
    pub on_blur: EventHandler<FocusEvent>,
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
    pub invalid: Option<bool>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_input: Option<EventHandler<FormEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
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
    let field_ctx = try_use_field_context();
    let control_ctx = try_use_field_control_context();

    let explicit_id = props.id.clone().filter(|s| !s.trim().is_empty());
    let id = if let Some(ref explicit) = explicit_id {
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
        ctx.runtime.relationships().input_id().to_owned()
    };
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

    let extracted = extract_aria_invalid_and_describedby(props.attributes);
    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let root_describedby = ctx.runtime.root_aria_describedby();
    let combined_context_describedby = merge_aria_describedby([
        field_describedby.as_deref(),
        root_describedby.as_deref(),
    ]);
    let fallback_invalid = ctx
        .runtime
        .root_invalid_override()
        .or_else(|| field_ctx.map(|f| f.is_invalid()));
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        props.invalid,
        extracted.aria_invalid,
        fallback_invalid,
        false,
        props.aria_describedby.as_deref(),
        extracted.aria_describedby.as_deref(),
        combined_context_describedby.as_deref(),
    );

    let attrs = ctx
        .runtime
        .input_attributes(id, props.disabled)
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby);
    let current_value = ctx.runtime.input_value();

    let mut on_focus_slot = use_hook(|| CopyValue::new(props.on_focus));
    on_focus_slot.set(props.on_focus);
    let mut on_blur_slot = use_hook(|| CopyValue::new(props.on_blur));
    on_blur_slot.set(props.on_blur);

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

    let on_focus = EventHandler::new(move |evt: FocusEvent| {
        if let Some(cb) = on_focus_slot.try_read().ok().and_then(|s| *s) {
            cb.call(evt);
        }
    });

    let on_blur = EventHandler::new(move |evt: FocusEvent| {
        if let Some(cb) = on_blur_slot.try_read().ok().and_then(|s| *s) {
            cb.call(evt);
        }
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
                on_focus,
                on_blur,
                on_composition_start,
                on_composition_end,
                on_mounted,
                attributes: extracted.rest,
            });
        }
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        extracted.rest,
    );

    let focus_in_rt = ctx.runtime.clone();
    let focus_out_rt = ctx.runtime.clone();

    rsx! {
        input {
            id: "{attrs.id()}",
            r#type: "text",
            role: "{attrs.role()}",
            "aria-autocomplete": "{attrs.aria_autocomplete()}",
            "aria-expanded": "{attrs.aria_expanded()}",
            "aria-controls": attrs.aria_controls(),
            "aria-activedescendant": attrs.aria_activedescendant(),
            "aria-invalid": attrs.aria_invalid(),
            "aria-describedby": attrs.aria_describedby(),
            "aria-required": attrs.aria_required(),
            disabled: attrs.is_disabled(),
            autocomplete: "{attrs.autocomplete()}",
            "data-combobox-input": "",
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": attrs.data_disabled(),
            "data-invalid": attrs.data_invalid(),
            value: "{current_value}",
            placeholder: props.placeholder.as_deref(),
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            oninput: move |evt| on_input.call(evt),
            onkeydown: move |evt| on_key_down.call(evt),
            onfocus: move |evt| on_focus.call(evt),
            onblur: move |evt| on_blur.call(evt),
            onfocusin: move |evt| focus_in_rt.notify_composite_focus_in(evt),
            onfocusout: move |evt| focus_out_rt.notify_composite_focus_out(evt),
            oncompositionstart: move |evt| on_composition_start.call(evt),
            oncompositionend: move |evt| on_composition_end.call(evt),
            onmounted: move |evt| on_mounted.call(evt),
            ..merged.rest,
        }
    }
}

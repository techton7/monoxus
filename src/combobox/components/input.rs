use dioxus::prelude::*;

use crate::combobox::runtime::use_combobox_context;
use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::compose::{
    collect_user_attrs, extract_aria_invalid_and_describedby, merge_aria_describedby, merge_props,
    push_attr_opt, push_event_handler, resolve_invalid_and_describedby,
};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxInputProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default = false)]
    pub clear_on_deselect: bool,
    #[props(default, into)]
    pub placeholder: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub on_input: Option<EventHandler<FormEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    ctx.runtime.set_clear_on_deselect(props.clear_on_deselect);

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

    let input_rt = ctx.runtime.clone();
    let base_on_input = EventHandler::new(move |evt: FormEvent| {
        input_rt.handle_input_change(evt.value());
    });

    let key_rt = ctx.runtime.clone();
    let base_on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        key_rt.handle_input_keydown(&evt);
    });

    // 4B D-5: Wire Web composition events alongside `evt.data().is_composing()`.
    let comp_start_rt = ctx.runtime.clone();
    let base_on_composition_start = EventHandler::new(move |_evt: CompositionEvent| {
        comp_start_rt.set_composing(true);
    });

    let comp_end_rt = ctx.runtime.clone();
    let base_on_composition_end = EventHandler::new(move |_evt: CompositionEvent| {
        comp_end_rt.set_composing(false);
    });

    let mount_rt = ctx.runtime.clone();
    let base_on_mounted = EventHandler::new(move |evt: MountedEvent| {
        mount_rt.capture_input_mount(evt.data());
    });

    let focus_in_rt = ctx.runtime.clone();
    let base_on_focus_in = EventHandler::new(move |evt: FocusEvent| {
        focus_in_rt.notify_composite_focus_in(evt);
    });
    let focus_out_rt = ctx.runtime.clone();
    let base_on_focus_out = EventHandler::new(move |evt: FocusEvent| {
        focus_out_rt.notify_composite_focus_out(evt);
    });

    let id_str = attrs.id().to_owned();
    let role_str = attrs.role();
    let aria_autocomplete = attrs.aria_autocomplete();
    let aria_expanded = attrs.aria_expanded();
    let aria_controls = attrs.aria_controls().map(str::to_owned);
    let aria_activedescendant = attrs.aria_activedescendant().map(str::to_owned);
    let aria_invalid = attrs.aria_invalid();
    let aria_describedby = attrs.aria_describedby().map(str::to_owned);
    let aria_required = attrs.aria_required();
    let is_disabled = attrs.is_disabled();
    let autocomplete = attrs.autocomplete();
    let data_state = attrs.data_state_str();
    let data_disabled = attrs.data_disabled();
    let data_invalid = attrs.data_invalid();

    let mut base = Vec::with_capacity(24);
    base.push(Attribute::new("id", id_str, None, false));
    base.push(Attribute::new("type", "text", None, false));
    base.push(Attribute::new("role", role_str, None, false));
    base.push(Attribute::new(
        "aria-autocomplete",
        aria_autocomplete,
        None,
        false,
    ));
    base.push(Attribute::new("aria-expanded", aria_expanded, None, false));
    push_attr_opt(&mut base, "aria-controls", aria_controls);
    push_attr_opt(&mut base, "aria-activedescendant", aria_activedescendant);
    push_attr_opt(&mut base, "aria-invalid", aria_invalid);
    push_attr_opt(&mut base, "aria-describedby", aria_describedby);
    push_attr_opt(&mut base, "aria-required", aria_required);
    if is_disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("autocomplete", autocomplete, None, false));
    base.push(Attribute::new("data-combobox-input", "", None, false));
    base.push(Attribute::new("data-state", data_state, None, false));
    push_attr_opt(&mut base, "data-disabled", data_disabled);
    push_attr_opt(&mut base, "data-invalid", data_invalid);
    base.push(Attribute::new("value", current_value, None, false));
    push_attr_opt(&mut base, "placeholder", props.placeholder);
    push_event_handler(&mut base, "oninput", Some(base_on_input));
    push_event_handler(&mut base, "onkeydown", Some(base_on_key_down));
    push_event_handler(&mut base, "onfocusin", Some(base_on_focus_in));
    push_event_handler(&mut base, "onfocusout", Some(base_on_focus_out));
    push_event_handler(
        &mut base,
        "oncompositionstart",
        Some(base_on_composition_start),
    );
    push_event_handler(&mut base, "oncompositionend", Some(base_on_composition_end));
    push_event_handler(&mut base, "onmounted", Some(base_on_mounted));

    let mut user = collect_user_attrs(props.class, props.style, extracted.rest);
    push_event_handler(&mut user, "oninput", props.on_input);
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
    push_event_handler(&mut user, "onfocus", props.on_focus);
    push_event_handler(&mut user, "onblur", props.on_blur);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        input {
            ..merged,
        }
    }
}

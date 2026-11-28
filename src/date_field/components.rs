use std::{borrow::Cow, cell::RefCell, collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;
use ui_sdk::foundation::date_time::field_policy::{
    DateFieldPolicy, FieldRangeRole, FieldRelationships, FieldSegmentItem, SegmentFieldDefinition,
    SegmentFieldState,
};

use crate::{
    field::try_use_field_context,
    foundation::{
        browser::{focus::use_composite_focus_out, local_today},
        compose::{
            extract_aria_invalid_and_describedby, merge_aria_describedby,
            merge_attributes_with_part, merge_global_attributes, resolve_invalid_and_describedby,
        },
        date_time::{
            CalendarDate, CalendarLocale, DateValue, EditableSegmentKind, FieldInvalidEvent,
            FieldSegment, Granularity, HourCycle, SegmentKind,
        },
        form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::ControlledBinding,
    },
    visually_hidden::VisuallyHidden,
};

use super::{
    attrs::{
        DateFieldInputAttrs, DateFieldLabelAttrs, DateFieldSegmentAttrs, SegmentFieldInputAttrs,
        SegmentFieldLabelAttrs, SegmentFieldSegmentAttrs,
    },
    runtime::{
        SegmentFieldContext, SegmentFieldRuntime, SegmentFieldUiPolicy, use_segment_field_context,
    },
    state::default_date_field_placeholder,
    types::{DateFieldRangeRole, DateFieldSegmentItem},
};

/// Plain Rust specification for [`use_segment_field_runtime`] and [`render_segment_field_root`]
/// (zero internal `#[component]` taking `Option<Callback|EventHandler>` per `Dioxus #5894`).
pub(crate) struct SegmentFieldRootSpec<P: SegmentFieldUiPolicy> {
    pub id: Option<String>,
    pub value: Option<Signal<Option<P::Value>>>,
    pub controlled_value: Option<ReadSignal<Option<P::Value>>>,
    pub default_value: Option<P::Value>,
    pub on_value_change: Option<EventHandler<Option<P::Value>>>,
    pub placeholder: Option<Signal<P::Value>>,
    pub controlled_placeholder: Option<ReadSignal<P::Value>>,
    pub on_placeholder_change: Option<EventHandler<P::Value>>,
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    pub granularity: P::GranularityProp,
    pub hour_cycle: Option<HourCycle>,
    pub locale: CalendarLocale,
    pub hide_time_zone: bool,
    pub min_value: Option<P::Value>,
    pub max_value: Option<P::Value>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub readonly_segments: Vec<EditableSegmentKind>,
    pub is_date_unavailable: Option<Callback<CalendarDate, bool>>,
    pub validate: Option<Callback<P::Value, Option<Vec<String>>>>,
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    pub error_message_id: Option<String>,
    pub dir: Option<Direction>,
    pub is_invalid_override: Option<bool>,
    pub range_role: FieldRangeRole,
    pub label_id_override: Option<String>,
    pub on_prev_boundary: Option<EventHandler<()>>,
    pub on_next_boundary: Option<EventHandler<()>>,
    pub children: Element,
}

/// Initializes a [`SegmentFieldRuntime<P>`], wires `B5`-compliant reactive prop/validate effects,
/// and provides `SegmentFieldContext<P>` for descendants.
pub(crate) fn use_segment_field_runtime<P: SegmentFieldUiPolicy>(
    spec: &SegmentFieldRootSpec<P>,
    initial_placeholder: P::Value,
) -> SegmentFieldRuntime<P> {
    let field_ctx = try_use_field_context();
    let effective_disabled = spec.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_read_only = spec.read_only || field_ctx.is_some_and(|f| f.is_read_only());
    let effective_required = spec.required || field_ctx.is_some_and(|f| f.is_required());
    let effective_invalid_override = spec
        .is_invalid_override
        .or_else(|| field_ctx.map(|f| f.is_invalid()));

    let resolved_dir = use_direction_context(spec.dir);
    let stable_id = use_stable_id(spec.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root(P::SCOPE_PREFIX).child(stable_id.clone()));
    let rels = use_hook(|| FieldRelationships::new(scope.clone()));

    let value_binding = ControlledBinding::from_props(spec.value, spec.controlled_value);
    let placeholder_binding =
        ControlledBinding::from_props(spec.placeholder, spec.controlled_placeholder);

    let initial_value = value_binding.initial_or(spec.default_value);

    let def = SegmentFieldDefinition::<P>::new()
        .with_locale(spec.locale.clone())
        .with_granularity(spec.granularity)
        .with_hour_cycle(spec.hour_cycle)
        .with_hide_time_zone(spec.hide_time_zone)
        .with_min_value(spec.min_value)
        .with_max_value(spec.max_value)
        .with_disabled(effective_disabled)
        .with_read_only(effective_read_only)
        .with_required(effective_required)
        .with_readonly_segments(spec.readonly_segments.clone())
        .with_error_message_id(spec.error_message_id.clone())
        .with_dir(resolved_dir);

    let state = use_signal(|| {
        let mut st = SegmentFieldState::<P>::new(def.clone(), initial_value, initial_placeholder);
        if let Some(inv) = effective_invalid_override {
            st.set_external_invalid(inv);
        }
        st
    });

    let input_dom_id = use_signal(|| rels.input_id().to_string());
    let label_id_override = use_signal(|| spec.label_id_override.clone());
    let custom_label_id = use_signal(|| None::<String>);
    let trailing_trigger_id = use_signal(|| None::<String>);
    let announcement = use_signal(String::new);
    let last_invalid_reason = use_signal(|| None);

    let mut range_role = use_hook(|| CopyValue::new(spec.range_role));
    range_role.set(spec.range_role);

    // Freeze mount-time reset baseline (`use_hook`) from controlled-or-default.
    let default_value = use_hook(|| CopyValue::new(initial_value));

    let mut controlled_value = use_hook(|| CopyValue::new(value_binding));
    controlled_value.set(value_binding);

    let mut controlled_placeholder = use_hook(|| CopyValue::new(placeholder_binding));
    controlled_placeholder.set(placeholder_binding);

    let mut on_value_change = use_hook(|| CopyValue::new(spec.on_value_change));
    on_value_change.set(spec.on_value_change);

    let mut on_placeholder_change = use_hook(|| CopyValue::new(spec.on_placeholder_change));
    on_placeholder_change.set(spec.on_placeholder_change);

    let mut is_date_unavailable = use_hook(|| CopyValue::new(spec.is_date_unavailable));
    is_date_unavailable.set(spec.is_date_unavailable);

    let mut validate = use_hook(|| CopyValue::new(spec.validate));
    validate.set(spec.validate);

    let mut on_invalid = use_hook(|| CopyValue::new(spec.on_invalid));
    on_invalid.set(spec.on_invalid);

    let mut on_prev_boundary = use_hook(|| CopyValue::new(spec.on_prev_boundary));
    on_prev_boundary.set(spec.on_prev_boundary);

    let mut on_next_boundary = use_hook(|| CopyValue::new(spec.on_next_boundary));
    on_next_boundary.set(spec.on_next_boundary);

    let mut on_root_focus_out = use_hook(|| CopyValue::new(spec.on_focus_out));
    on_root_focus_out.set(spec.on_focus_out);

    let parent_focus_in = use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));
    let parent_focus_out = use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));

    let segment_dom_ids = use_hook(|| Rc::new(RefCell::new(BTreeMap::new())));

    let mut runtime = SegmentFieldRuntime::<P> {
        state,
        rels,
        range_role,
        input_dom_id,
        label_id_override,
        custom_label_id,
        trailing_trigger_id,
        segment_dom_ids,
        announcement,
        last_invalid_reason,
        controlled_value,
        controlled_placeholder,
        default_value,
        on_value_change,
        on_placeholder_change,
        is_date_unavailable,
        validate,
        on_invalid,
        on_prev_boundary,
        on_next_boundary,
        on_root_focus_out,
        parent_focus_in,
        parent_focus_out,
    };

    // Synchronous render-time reconciliation for external controlled signals & invalid override.
    // Preserves partial segment drafts because `set_value` only fires when `state.value() != val`.
    {
        let ext_val_now = value_binding.read_cloned();
        let ext_ph_now = placeholder_binding.read_cloned();
        let ext_inv_now = effective_invalid_override.unwrap_or(false);
        if let Some(val) = ext_val_now
            && runtime.state.peek().value() != val
        {
            runtime.state.write().set_value(val);
        }
        if let Some(ph) = ext_ph_now
            && runtime.state.peek().placeholder() != ph
        {
            runtime.state.write().set_placeholder(ph);
        }
        if runtime.state.peek().external_invalid() != ext_inv_now {
            runtime.state.write().set_external_invalid(ext_inv_now);
        }
    }

    let mut predicate_rev = use_signal(|| 0_u64);
    let has_pred = spec.is_date_unavailable.is_some() || spec.validate.is_some();
    let mut pred_initialized = use_hook(|| CopyValue::new(false));
    let mut prev_had_pred = use_hook(|| CopyValue::new(has_pred));
    if pred_initialized.try_read().is_ok_and(|init| *init) {
        let had_pred = prev_had_pred.try_read().is_ok_and(|had| *had);
        prev_had_pred.set(has_pred);
        if has_pred || had_pred {
            let next_rev = predicate_rev.try_peek().ok().map(|cur| cur.wrapping_add(1));
            if let Some(next_rev) = next_rev {
                crate::foundation::state::sync_signal(&mut predicate_rev, next_rev);
            }
        }
    } else {
        pred_initialized.set(true);
    }

    // DEF-5C-09 & B5: Reconcile definition, predicates (`validate`, `is_date_unavailable`),
    // controlled signals, and validation inside guarded effects for BOTH DateField and TimeField.
    use_effect(use_reactive(
        (
            &def,
            &spec.label_id_override,
            &spec.is_date_unavailable,
            &spec.validate,
        ),
        {
            let mut rt = runtime.clone();
            move |(next_def, label_override, _next_unavailable, _next_validate)| {
                let _ = *predicate_rev.read();
                if *rt.label_id_override.peek() != label_override {
                    rt.label_id_override.set(label_override);
                }
                if *rt.state.peek().def() != next_def {
                    rt.state.write().sync_definition(next_def);
                }
                rt.sync_validation_and_notify();
            }
        },
    ));

    let eff_inv_bool = effective_invalid_override.unwrap_or(false);
    use_effect(use_reactive(
        (&value_binding, &placeholder_binding, &eff_inv_bool),
        {
            let mut rt = runtime.clone();
            move |(val_bind, ph_bind, inv_override)| {
                let ext_val = val_bind.read_cloned();
                let ext_ph = ph_bind.read_cloned();
                rt.reconcile_external_state(ext_val, ext_ph, Some(inv_override));
            }
        },
    ));

    use_context_provider(|| SegmentFieldContext::<P>(runtime.clone()));
    runtime
}

/// Renders the accessible description and live-region chrome around `body` for a [`SegmentFieldRuntime<P>`].
pub(crate) fn render_segment_field_chrome<P: SegmentFieldUiPolicy>(
    runtime: &SegmentFieldRuntime<P>,
    body: Element,
) -> Element {
    let description_text = runtime.state.read().description_text();
    let announcement_text = (runtime.announcement)();
    let desc_id = runtime.rels.description_id().to_string();
    let live_id = runtime.rels.live_region_id().to_string();

    rsx! {
        if !description_text.is_empty() {
            VisuallyHidden {
                id: Some(desc_id),
                "{description_text}"
            }
        }
        VisuallyHidden {
            id: Some(live_id),
            role: "status",
            "aria-live": "polite",
            "aria-atomic": "true",
            "{announcement_text}"
        }
        {body}
    }
}

/// Monomorphized plain-fn root renderer shared by `DateFieldRoot` and `TimeFieldRoot`.
pub(crate) fn render_segment_field_root<P: SegmentFieldUiPolicy>(
    spec: SegmentFieldRootSpec<P>,
    initial_placeholder: P::Value,
) -> Element {
    let runtime = use_segment_field_runtime::<P>(&spec, initial_placeholder);
    render_segment_field_chrome(&runtime, spec.children)
}

/// Specification for [`render_segment_field_input`] and [`render_segment_field_input_with_runtime`].
pub(crate) struct SegmentFieldInputSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub name: Option<String>,
    pub form: Option<String>,
    pub as_child: bool,
    pub render_children: Option<Callback<Vec<FieldSegmentItem>, Element>>,
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub debug_name: &'static str,
}

/// View passed to `render_as_child` inside [`render_segment_field_input_with_runtime`].
pub(crate) struct SegmentFieldInputView {
    pub attrs: SegmentFieldInputAttrs,
    pub segments: Vec<FieldSegmentItem>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Monomorphized plain-fn input renderer taking an explicit [`SegmentFieldRuntime<P>`]
/// (used by both standalone `*FieldInput` and range endpoint `*RangeFieldInput`).
pub(crate) fn render_segment_field_input_with_runtime<P, F>(
    runtime: SegmentFieldRuntime<P>,
    spec: SegmentFieldInputSpec,
    render_as_child: F,
) -> Element
where
    P: SegmentFieldUiPolicy,
    F: FnOnce(SegmentFieldInputView) -> Option<Element>,
{
    let field_ctx = try_use_field_context();
    let snap = runtime.snapshot();
    let extracted = extract_aria_invalid_and_describedby(spec.attributes);
    let raw_attrs = runtime.input_attrs(spec.id.clone());
    runtime.set_input_dom_id(raw_attrs.id.clone());

    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let merged_internal_and_field_desc = merge_aria_describedby([
        raw_attrs.aria_describedby.as_deref(),
        field_describedby.as_deref(),
    ]);
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        None,
        extracted.aria_invalid,
        Some(raw_attrs.data_invalid.is_some()),
        false,
        None,
        extracted.aria_describedby.as_deref(),
        merged_internal_and_field_desc.as_deref(),
    );
    let attrs = raw_attrs
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby);

    let mut input_focus_out_slot = use_hook(|| CopyValue::new(spec.on_focus_out));
    input_focus_out_slot.set(spec.on_focus_out);
    let root_focus_out_slot = runtime.on_root_focus_out;
    let combined_focus_out = use_callback(move |evt: FocusEvent| {
        if let Some(cb) = input_focus_out_slot.try_peek().ok().and_then(|s| *s) {
            cb.call(evt.clone());
        }
        if let Some(cb) = root_focus_out_slot.try_peek().ok().and_then(|s| *s) {
            cb.call(evt);
        }
    });
    let (handle_focus_in, handle_focus_out) =
        use_composite_focus_out(vec![attrs.id.clone()], Some(combined_focus_out));

    let mut reset_rt = runtime.clone();
    let (show_hidden, reset_epoch) = use_form_control_sync_with_epoch(
        &attrs.id,
        spec.name.as_deref(),
        spec.form.as_deref(),
        move || {
            reset_rt.reset_to_default();
        },
    );

    let has_name = spec.name.as_deref().is_some_and(|n| !n.is_empty());
    let render_hidden =
        has_name || (show_hidden && spec.form.as_deref().is_some_and(|f| !f.is_empty()));

    let body = if let Some(ref render_children) = spec.render_children {
        render_children.call(snap.segments.clone())
    } else {
        spec.children
    };

    let inner = rsx! {
        {body}
        if render_hidden {
            HiddenFormInput {
                id: Some(runtime.rels.hidden_input_id().to_string()),
                kind: HiddenInputKind::Hidden,
                name: spec.name.clone(),
                value: snap.iso_value.clone(),
                disabled: snap.disabled,
                required: snap.required,
                read_only: snap.read_only,
                form: spec.form.clone(),
                focus_target_id: runtime.first_interactive_segment_id(),
                reset_epoch,
            }
        }
    };

    if spec.as_child {
        let view = SegmentFieldInputView {
            attrs: attrs.clone(),
            segments: snap.segments,
            class: spec.class.clone(),
            style: spec.style.clone(),
            on_mounted: spec.on_mounted,
            attributes: extracted.rest.clone(),
            children: inner.clone(),
        };
        if let Some(el) = render_as_child(view) {
            return el;
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::{}: as_child requires render_child; rendering the default div",
            spec.debug_name
        );
    }

    let attrs_with_part = merge_attributes_with_part(extracted.rest, P::INPUT_DATA_ATTR);
    let merged = merge_global_attributes(
        spec.class.as_deref(),
        spec.style.as_deref(),
        attrs_with_part,
    );
    let on_mounted = spec.on_mounted;
    let parent_fi_rt = runtime.clone();
    let parent_fo_rt = runtime;

    rsx! {
        div {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: attrs.role,
            "aria-labelledby": attrs.aria_labelledby,
            "aria-describedby": attrs.aria_describedby,
            "aria-invalid": attrs.aria_invalid,
            "aria-disabled": attrs.aria_disabled,
            "aria-required": attrs.aria_required,
            dir: attrs.dir,
            "data-range-field-part": attrs.data_range_field_part,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            onfocusin: move |evt| {
                handle_focus_in.call(evt.clone());
                parent_fi_rt.notify_parent_focus_in(evt);
            },
            onfocusout: move |evt| {
                handle_focus_out.call(evt.clone());
                parent_fo_rt.notify_parent_focus_out(evt);
            },
            onmounted: move |evt| {
                if let Some(handler) = on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {inner}
        }
    }
}

/// Monomorphized plain-fn input renderer consuming [`SegmentFieldContext<P>`].
pub(crate) fn render_segment_field_input<P, F>(
    spec: SegmentFieldInputSpec,
    render_as_child: F,
) -> Element
where
    P: SegmentFieldUiPolicy,
    F: FnOnce(SegmentFieldInputView) -> Option<Element>,
{
    let runtime = use_segment_field_context::<P>();
    render_segment_field_input_with_runtime(runtime, spec, render_as_child)
}

/// Specification for [`render_segment_field_segment`].
pub(crate) struct SegmentFieldSegmentSpec {
    pub part: SegmentKind,
    pub index: Option<usize>,
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub as_child: bool,
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    pub on_focus: Option<EventHandler<FocusEvent>>,
    pub on_blur: Option<EventHandler<FocusEvent>>,
    pub on_paste: Option<EventHandler<ClipboardEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Option<Element>,
    pub debug_name: &'static str,
}

/// View passed to `render_as_child` inside [`render_segment_field_segment`].
pub(crate) struct SegmentFieldSegmentView {
    pub attrs: SegmentFieldSegmentAttrs,
    pub segment: FieldSegmentItem,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_focus: EventHandler<FocusEvent>,
    pub on_blur: EventHandler<FocusEvent>,
    pub on_paste: EventHandler<ClipboardEvent>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Monomorphized plain-fn segment renderer shared by `DateFieldSegment` and `TimeFieldSegment`.
pub(crate) fn render_segment_field_segment<P, F>(
    spec: SegmentFieldSegmentSpec,
    render_as_child: F,
) -> Element
where
    P: SegmentFieldUiPolicy,
    F: FnOnce(SegmentFieldSegmentView) -> Option<Element>,
{
    let runtime = use_segment_field_context::<P>();
    let segments = runtime.segments();

    let item = spec
        .index
        .and_then(|idx| segments.iter().find(|s| s.index == idx).cloned())
        .or_else(|| segments.iter().find(|s| s.kind() == spec.part).cloned())
        .unwrap_or_else(|| FieldSegmentItem {
            index: spec.index.unwrap_or(0),
            id: runtime
                .rels
                .segment_id(spec.index.unwrap_or(0), spec.part),
            segment: FieldSegment {
                kind: spec.part,
                value: String::new(),
                is_placeholder: false,
                min: None,
                max: None,
                now: None,
                value_text: None,
            },
        });

    let attrs = runtime.segment_attrs(&item, spec.id.clone());
    runtime.register_segment_dom_id(item.index, attrs.id.clone());
    let seg_index = item.index;
    let seg_kind = item.kind();
    let is_interactive = item.is_interactive();

    let on_key_down = {
        let mut rt = runtime.clone();
        let user_cb = spec.on_key_down;
        EventHandler::new(move |evt: KeyboardEvent| {
            if is_interactive {
                rt.handle_segment_keydown(seg_index, seg_kind, &evt);
            }
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    let on_focus = {
        let user_cb = spec.on_focus;
        EventHandler::new(move |evt: FocusEvent| {
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    let on_blur = {
        let mut rt = runtime.clone();
        let user_cb = spec.on_blur;
        EventHandler::new(move |evt: FocusEvent| {
            if is_interactive {
                rt.handle_segment_blur(seg_kind);
            }
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    let on_paste = {
        let mut rt = runtime.clone();
        let user_cb = spec.on_paste;
        EventHandler::new(move |evt: ClipboardEvent| {
            if is_interactive
                && let Some(text) = evt.data().data_transfer().get_as_text()
                && rt.handle_segment_paste(seg_kind, &text)
            {
                evt.prevent_default();
            }
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    let rendered_children = match spec.children {
        Some(el) => el,
        None => rsx! { "{item.segment.value}" },
    };

    if spec.as_child {
        let view = SegmentFieldSegmentView {
            attrs: attrs.clone(),
            segment: item,
            class: spec.class.clone(),
            style: spec.style.clone(),
            on_key_down,
            on_focus,
            on_blur,
            on_paste,
            on_mounted: spec.on_mounted,
            attributes: spec.attributes.clone(),
            children: rendered_children.clone(),
        };
        if let Some(el) = render_as_child(view) {
            return el;
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::{}: as_child requires render_child; rendering the default span",
            spec.debug_name
        );
    }

    let attrs_with_part = merge_attributes_with_part(spec.attributes, P::SEGMENT_DATA_ATTR);
    let merged = merge_global_attributes(
        spec.class.as_deref(),
        spec.style.as_deref(),
        attrs_with_part,
    );
    let on_mounted = spec.on_mounted;

    rsx! {
        span {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: attrs.role,
            tabindex: attrs.tabindex,
            inputmode: attrs.inputmode,
            "aria-label": attrs.aria_label,
            "aria-labelledby": attrs.aria_labelledby,
            "aria-describedby": attrs.aria_describedby,
            "aria-valuemin": attrs.aria_valuemin,
            "aria-valuemax": attrs.aria_valuemax,
            "aria-valuenow": attrs.aria_valuenow,
            "aria-valuetext": attrs.aria_valuetext,
            "aria-invalid": attrs.aria_invalid,
            "aria-disabled": attrs.aria_disabled,
            "aria-readonly": attrs.aria_readonly,
            "aria-hidden": attrs.aria_hidden,
            "data-segment": attrs.data_segment,
            "data-placeholder": attrs.data_placeholder,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            onkeydown: move |evt| on_key_down.call(evt),
            onfocus: move |evt| on_focus.call(evt),
            onblur: move |evt| on_blur.call(evt),
            onpaste: move |evt| on_paste.call(evt),
            onmounted: move |evt| {
                if let Some(handler) = on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {rendered_children}
        }
    }
}

/// Specification for [`render_segment_field_label`].
pub(crate) struct SegmentFieldLabelSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub as_child: bool,
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub debug_name: &'static str,
}

/// View passed to `render_as_child` inside [`render_segment_field_label`].
pub(crate) struct SegmentFieldLabelView {
    pub attrs: SegmentFieldLabelAttrs,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Monomorphized plain-fn label renderer shared by `DateFieldLabel` and `TimeFieldLabel`,
/// with equality-guarded `custom_label_id` drop cleanup preventing keyed swap clobber.
pub(crate) fn render_segment_field_label<P, F>(
    spec: SegmentFieldLabelSpec,
    render_as_child: F,
) -> Element
where
    P: SegmentFieldUiPolicy,
    F: FnOnce(SegmentFieldLabelView) -> Option<Element>,
{
    let mut runtime = use_segment_field_context::<P>();
    let normalized_id = spec.id.clone().filter(|s| !s.is_empty());
    let latest_label_id = use_hook(|| Rc::new(RefCell::new(normalized_id.clone())));
    {
        let mut prev = latest_label_id.borrow_mut();
        if *prev != normalized_id {
            if let Some(old) = prev.as_deref() {
                runtime.unregister_custom_label_id(old);
            }
            *prev = normalized_id;
        }
    }
    use_effect(use_reactive((&spec.id,), {
        let mut rt = runtime.clone();
        move |(custom_id,)| {
            rt.register_custom_label_id(custom_id);
        }
    }));
    {
        let mut rt_drop = runtime.clone();
        let latest_drop = Rc::clone(&latest_label_id);
        use_drop(move || {
            if let Some(id) = latest_drop.borrow().as_deref() {
                rt_drop.unregister_custom_label_id(id);
            }
        });
    }

    let attrs = runtime.label_attrs(spec.id.clone());
    let on_click = {
        let rt = runtime.clone();
        let user_cb = spec.on_click;
        EventHandler::new(move |evt: MouseEvent| {
            if rt.state.try_peek().is_ok_and(|st| !st.def().disabled) {
                rt.focus_first_segment();
            }
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    if spec.as_child {
        let view = SegmentFieldLabelView {
            attrs: attrs.clone(),
            class: spec.class.clone(),
            style: spec.style.clone(),
            on_click,
            on_mounted: spec.on_mounted,
            attributes: spec.attributes.clone(),
            children: spec.children.clone(),
        };
        if let Some(el) = render_as_child(view) {
            return el;
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::{}: as_child requires render_child; rendering the default div",
            spec.debug_name
        );
    }

    let attrs_with_part = merge_attributes_with_part(spec.attributes, P::LABEL_DATA_ATTR);
    let merged = merge_global_attributes(
        spec.class.as_deref(),
        spec.style.as_deref(),
        attrs_with_part,
    );
    let on_mounted = spec.on_mounted;

    rsx! {
        div {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            onclick: move |evt| on_click.call(evt),
            onmounted: move |evt| {
                if let Some(handler) = on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {spec.children}
        }
    }
}

/// Props for [`DateFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<Signal<Option<DateValue>>>,
    #[props(default)]
    pub controlled_value: Option<ReadSignal<Option<DateValue>>>,
    #[props(default)]
    pub default_value: Option<DateValue>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub placeholder: Option<Signal<DateValue>>,
    #[props(default)]
    pub controlled_placeholder: Option<ReadSignal<DateValue>>,
    #[props(default)]
    pub default_placeholder: Option<DateValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<DateValue>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub granularity: Option<Granularity>,
    #[props(default)]
    pub hour_cycle: Option<HourCycle>,
    #[props(default)]
    pub locale: CalendarLocale,
    #[props(default = false)]
    pub hide_time_zone: bool,
    #[props(default)]
    pub min_value: Option<DateValue>,
    #[props(default)]
    pub max_value: Option<DateValue>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub readonly_segments: Vec<EditableSegmentKind>,
    #[props(default)]
    pub is_date_unavailable: Option<Callback<CalendarDate, bool>>,
    #[props(default)]
    pub validate: Option<Callback<DateValue, Option<Vec<String>>>>,
    #[props(default)]
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    #[props(default)]
    pub error_message_id: Option<String>,
    #[props(default)]
    pub dir: Option<Direction>,
    /// External invalid override (e.g., propagated by `DateRangeField` on `RangeOrder`).
    #[props(default)]
    pub is_invalid_override: Option<bool>,
    /// Standalone vs range endpoint role (used by `DateRangeField`).
    #[props(default)]
    pub range_role: DateFieldRangeRole,
    /// Optional external label ID (used by `DateRangeField` to share its `DateRangeFieldLabel`).
    #[props(default)]
    pub label_id_override: Option<String>,
    /// Cross-field boundary callback fired when navigating before the first interactive segment.
    #[props(default)]
    pub on_prev_boundary: Option<EventHandler<()>>,
    /// Cross-field boundary callback fired when navigating past the last interactive segment.
    #[props(default)]
    pub on_next_boundary: Option<EventHandler<()>>,
    #[props(default)]
    pub children: Element,
}

/// Root provider for segmented date / date-time spinbutton fields.
#[component]
pub fn DateFieldRoot(props: DateFieldRootProps) -> Element {
    let today = use_hook(local_today);
    let value_binding = ControlledBinding::from_props(props.value, props.controlled_value);
    let placeholder_binding =
        ControlledBinding::from_props(props.placeholder, props.controlled_placeholder);
    let initial_value = value_binding.initial_or(props.default_value);
    let initial_placeholder = placeholder_binding
        .peek_cloned()
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_date_field_placeholder(
                initial_value,
                props.granularity,
                props.min_value,
                props.max_value,
                today,
            )
        });

    render_segment_field_root::<DateFieldPolicy>(
        SegmentFieldRootSpec {
            id: props.id,
            value: props.value,
            controlled_value: props.controlled_value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            placeholder: props.placeholder,
            controlled_placeholder: props.controlled_placeholder,
            on_placeholder_change: props.on_placeholder_change,
            on_focus_out: props.on_focus_out,
            granularity: props.granularity,
            hour_cycle: props.hour_cycle,
            locale: props.locale,
            hide_time_zone: props.hide_time_zone,
            min_value: props.min_value,
            max_value: props.max_value,
            disabled: props.disabled,
            read_only: props.read_only,
            required: props.required,
            readonly_segments: props.readonly_segments,
            is_date_unavailable: props.is_date_unavailable,
            validate: props.validate,
            on_invalid: props.on_invalid,
            error_message_id: props.error_message_id,
            dir: props.dir,
            is_invalid_override: props.is_invalid_override,
            range_role: props.range_role,
            label_id_override: props.label_id_override,
            on_prev_boundary: props.on_prev_boundary,
            on_next_boundary: props.on_next_boundary,
            children: props.children,
        },
        initial_placeholder,
    )
}

pub use DateFieldRoot as DateField;

/// Render props passed to `as_child` for [`DateFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldInputRenderProps {
    pub attrs: DateFieldInputAttrs,
    pub segments: Vec<DateFieldSegmentItem>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Props for [`DateFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldInputProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<DateFieldInputRenderProps, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<Vec<DateFieldSegmentItem>, Element>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Container for `DateField` segments and automatic hidden `<input>` form synchronization.
#[component]
pub fn DateFieldInput(props: DateFieldInputProps) -> Element {
    let render_child = props.render_child;
    render_segment_field_input::<DateFieldPolicy, _>(
        SegmentFieldInputSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            name: props.name,
            form: props.form,
            as_child: props.as_child,
            render_children: props.render_children,
            on_focus_out: props.on_focus_out,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "DateFieldInput",
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(DateFieldInputRenderProps {
                    attrs: view.attrs,
                    segments: view.segments,
                    class: view.class,
                    style: view.style,
                    on_mounted: view.on_mounted,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}

/// Render props passed to `as_child` for [`DateFieldSegment`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldSegmentRenderProps {
    pub attrs: DateFieldSegmentAttrs,
    pub segment: DateFieldSegmentItem,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_focus: EventHandler<FocusEvent>,
    pub on_blur: EventHandler<FocusEvent>,
    pub on_paste: EventHandler<ClipboardEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Props for [`DateFieldSegment`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldSegmentProps {
    pub part: SegmentKind,
    #[props(default)]
    pub index: Option<usize>,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<DateFieldSegmentRenderProps, Element>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_paste: Option<EventHandler<ClipboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Individual spinbutton or literal segment inside a `DateFieldInput`.
#[component]
pub fn DateFieldSegment(props: DateFieldSegmentProps) -> Element {
    let render_child = props.render_child;
    render_segment_field_segment::<DateFieldPolicy, _>(
        SegmentFieldSegmentSpec {
            part: props.part,
            index: props.index,
            id: props.id,
            class: props.class,
            style: props.style,
            as_child: props.as_child,
            on_key_down: props.on_key_down,
            on_focus: props.on_focus,
            on_blur: props.on_blur,
            on_paste: props.on_paste,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "DateFieldSegment",
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(DateFieldSegmentRenderProps {
                    attrs: view.attrs,
                    segment: view.segment,
                    class: view.class,
                    style: view.style,
                    on_key_down: view.on_key_down,
                    on_focus: view.on_focus,
                    on_blur: view.on_blur,
                    on_paste: view.on_paste,
                    on_mounted: view.on_mounted,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}

/// Render props passed to `as_child` for [`DateFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldLabelRenderProps {
    pub attrs: DateFieldLabelAttrs,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_click: EventHandler<MouseEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Props for [`DateFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldLabelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<DateFieldLabelRenderProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Accessible label for `DateField`; clicking focuses the first interactive segment.
#[component]
pub fn DateFieldLabel(props: DateFieldLabelProps) -> Element {
    let render_child = props.render_child;
    render_segment_field_label::<DateFieldPolicy, _>(
        SegmentFieldLabelSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            as_child: props.as_child,
            on_click: props.on_click,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "DateFieldLabel",
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(DateFieldLabelRenderProps {
                    attrs: view.attrs,
                    class: view.class,
                    style: view.style,
                    on_click: view.on_click,
                    on_mounted: view.on_mounted,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}

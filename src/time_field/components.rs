use std::{borrow::Cow, cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    foundation::{
        date_time::{
            CalendarLocale, EditableSegmentKind, FieldInvalidEvent, HourCycle, SegmentKind,
            TimeGranularity, TimeValue,
        },
        form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    },
    visually_hidden::VisuallyHidden,
};

use super::{
    attrs::{TimeFieldInputAttrs, TimeFieldLabelAttrs, TimeFieldSegmentAttrs},
    relationships::TimeFieldRelationships,
    runtime::{TimeFieldContext, TimeFieldRuntime, use_time_field_context},
    state::{TimeFieldDefinition, TimeFieldState, default_time_field_placeholder},
    types::{TimeFieldRangeRole, TimeFieldSegmentItem},
};

/// Props for [`TimeFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<Signal<Option<TimeValue>>>,
    #[props(default)]
    pub default_value: Option<TimeValue>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<TimeValue>>>,
    #[props(default)]
    pub placeholder: Option<Signal<TimeValue>>,
    #[props(default)]
    pub default_placeholder: Option<TimeValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<TimeValue>>,
    #[props(default)]
    pub granularity: TimeGranularity,
    #[props(default)]
    pub hour_cycle: Option<HourCycle>,
    #[props(default)]
    pub locale: CalendarLocale,
    #[props(default)]
    pub min_value: Option<TimeValue>,
    #[props(default)]
    pub max_value: Option<TimeValue>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub readonly_segments: Vec<EditableSegmentKind>,
    #[props(default)]
    pub validate: Option<Callback<TimeValue, Option<Vec<String>>>>,
    #[props(default)]
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    #[props(default)]
    pub error_message_id: Option<String>,
    #[props(default)]
    pub dir: Option<Direction>,
    /// External invalid override (e.g., propagated by `TimeRangeField` on `RangeOrder`).
    #[props(default)]
    pub is_invalid_override: Option<bool>,
    /// Standalone vs range endpoint role (used by `TimeRangeField`).
    #[props(default)]
    pub range_role: TimeFieldRangeRole,
    /// Optional external label ID (used by `TimeRangeField` to share its `TimeRangeFieldLabel`).
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

/// Root provider for segmented wall-clock time spinbutton fields.
#[component]
pub fn TimeFieldRoot(props: TimeFieldRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("time-field").child(stable_id.clone()));
    let rels = use_hook(|| TimeFieldRelationships::new(scope.clone()));

    let initial_value = props
        .value
        .and_then(|s| *s.peek())
        .or(props.default_value);
    let initial_placeholder = props
        .placeholder
        .map(|s| *s.peek())
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_time_field_placeholder(initial_value, props.min_value, props.max_value)
        });

    let def = TimeFieldDefinition::new()
        .with_locale(props.locale.clone())
        .with_granularity(props.granularity)
        .with_hour_cycle(props.hour_cycle)
        .with_min_value(props.min_value)
        .with_max_value(props.max_value)
        .with_disabled(props.disabled)
        .with_read_only(props.read_only)
        .with_required(props.required)
        .with_readonly_segments(props.readonly_segments.clone())
        .with_error_message_id(props.error_message_id.clone())
        .with_dir(resolved_dir);

    let state = use_signal(|| {
        let mut st = TimeFieldState::new(def.clone(), initial_value, initial_placeholder);
        if let Some(inv) = props.is_invalid_override {
            st.set_external_invalid(inv);
        }
        st
    });

    let mut label_id_override = use_signal(|| props.label_id_override.clone());
    if *label_id_override.peek() != props.label_id_override {
        label_id_override.set(props.label_id_override.clone());
    }

    let announcement = use_signal(String::new);
    let last_invalid_reason = use_signal(|| None);

    let controlled_value = use_hook(|| Rc::new(RefCell::new(props.value)));
    *controlled_value.borrow_mut() = props.value;

    let controlled_placeholder = use_hook(|| Rc::new(RefCell::new(props.placeholder)));
    *controlled_placeholder.borrow_mut() = props.placeholder;

    let on_value_change = use_hook(|| Rc::new(RefCell::new(props.on_value_change)));
    *on_value_change.borrow_mut() = props.on_value_change;

    let on_placeholder_change = use_hook(|| Rc::new(RefCell::new(props.on_placeholder_change)));
    *on_placeholder_change.borrow_mut() = props.on_placeholder_change;

    let validate = use_hook(|| Rc::new(RefCell::new(props.validate)));
    *validate.borrow_mut() = props.validate;

    let on_invalid = use_hook(|| Rc::new(RefCell::new(props.on_invalid)));
    *on_invalid.borrow_mut() = props.on_invalid;

    let on_prev_boundary = use_hook(|| Rc::new(RefCell::new(props.on_prev_boundary)));
    *on_prev_boundary.borrow_mut() = props.on_prev_boundary;

    let on_next_boundary = use_hook(|| Rc::new(RefCell::new(props.on_next_boundary)));
    *on_next_boundary.borrow_mut() = props.on_next_boundary;

    let segment_dom_ids = use_hook(|| Rc::new(RefCell::new(std::collections::BTreeMap::new())));

    let mut runtime = TimeFieldRuntime {
        state,
        rels: rels.clone(),
        range_role: props.range_role,
        label_id_override,
        segment_dom_ids,
        announcement,
        last_invalid_reason,
        controlled_value,
        controlled_placeholder,
        default_value: props.default_value,
        on_value_change,
        on_placeholder_change,
        validate,
        on_invalid,
        on_prev_boundary,
        on_next_boundary,
    };

    if *runtime.state.peek().def() != def {
        runtime.state.write().sync_definition(def);
    }
    let ext_val = props.value.map(|s| *s.read());
    let ext_ph = props.placeholder.map(|s| *s.read());
    runtime.reconcile_external_state(ext_val, ext_ph, props.is_invalid_override);

    use_context_provider(|| TimeFieldContext(runtime.clone()));

    let description_text = runtime.state.read().description_text();
    let announcement_text = (runtime.announcement)();
    let desc_id = rels.description_id().to_string();
    let live_id = rels.live_region_id().to_string();

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
        {props.children}
    }
}

pub use TimeFieldRoot as TimeField;

/// Render props passed to `as_child` for [`TimeFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldInputRenderProps {
    pub attrs: TimeFieldInputAttrs,
    pub segments: Vec<TimeFieldSegmentItem>,
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

/// Props for [`TimeFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldInputProps {
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
    pub render_child: Option<Callback<TimeFieldInputRenderProps, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<Vec<TimeFieldSegmentItem>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Container for `TimeField` segments and automatic hidden `<input>` form synchronization.
#[component]
pub fn TimeFieldInput(props: TimeFieldInputProps) -> Element {
    let runtime = use_time_field_context();
    let snap = runtime.snapshot();
    let attrs = runtime.input_attrs(props.id.clone());

    let mut reset_rt = runtime.clone();
    let (show_hidden, reset_epoch) = use_form_control_sync_with_epoch(
        &attrs.id,
        props.name.as_deref(),
        props.form.as_deref(),
        move || {
            reset_rt.reset_to_default();
        },
    );

    let has_name = props.name.as_deref().is_some_and(|n| !n.is_empty());
    let render_hidden =
        has_name || (show_hidden && props.form.as_deref().is_some_and(|f| !f.is_empty()));

    let body = if let Some(ref render_children) = props.render_children {
        render_children.call(snap.segments.clone())
    } else {
        props.children
    };

    let inner = rsx! {
        {body}
        if render_hidden {
            HiddenFormInput {
                id: Some(runtime.rels.hidden_input_id().to_string()),
                kind: HiddenInputKind::Hidden,
                name: props.name.clone(),
                value: snap.iso_value.clone(),
                disabled: snap.disabled,
                required: snap.required,
                read_only: snap.read_only,
                form: props.form.clone(),
                focus_target_id: runtime.first_interactive_segment_id(),
                reset_epoch,
            }
        }
    };

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(TimeFieldInputRenderProps {
                attrs,
                segments: snap.segments,
                class: props.class,
                style: props.style,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: inner,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::TimeFieldInput: as_child requires render_child; rendering the default div"
        );
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            role: attrs.role,
            "aria-labelledby": attrs.aria_labelledby,
            "aria-describedby": attrs.aria_describedby,
            "aria-disabled": attrs.aria_disabled,
            dir: attrs.dir,
            "data-time-field-input": attrs.data_time_field_input,
            "data-range-field-part": attrs.data_range_field_part,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {inner}
        }
    }
}

/// Render props passed to `as_child` for [`TimeFieldSegment`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldSegmentRenderProps {
    pub attrs: TimeFieldSegmentAttrs,
    pub segment: TimeFieldSegmentItem,
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

/// Props for [`TimeFieldSegment`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldSegmentProps {
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
    pub render_child: Option<Callback<TimeFieldSegmentRenderProps, Element>>,
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

/// Individual spinbutton or literal segment inside a `TimeFieldInput`.
#[component]
pub fn TimeFieldSegment(props: TimeFieldSegmentProps) -> Element {
    let runtime = use_time_field_context();
    let segments = runtime.segments();

    let item = props
        .index
        .and_then(|idx| segments.iter().find(|s| s.index == idx).cloned())
        .or_else(|| segments.iter().find(|s| s.kind() == props.part).cloned())
        .unwrap_or_else(|| TimeFieldSegmentItem {
            index: props.index.unwrap_or(0),
            id: runtime
                .rels
                .segment_id(props.index.unwrap_or(0), props.part),
            segment: crate::foundation::date_time::FieldSegment {
                kind: props.part,
                value: String::new(),
                is_placeholder: false,
                min: None,
                max: None,
                now: None,
                value_text: None,
            },
        });

    let attrs = runtime.segment_attrs(&item, props.id.clone());
    runtime.register_segment_dom_id(item.index, attrs.id.clone());
    let seg_index = item.index;
    let seg_kind = item.kind();
    let is_interactive = item.is_interactive();

    let on_key_down = {
        let mut rt = runtime.clone();
        let user_cb = props.on_key_down;
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
        let user_cb = props.on_focus;
        EventHandler::new(move |evt: FocusEvent| {
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    let on_blur = {
        let mut rt = runtime.clone();
        let user_cb = props.on_blur;
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
        let user_cb = props.on_paste;
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

    let rendered_children = match props.children {
        Some(el) => el,
        None => rsx! { "{item.segment.value}" },
    };

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(TimeFieldSegmentRenderProps {
                attrs,
                segment: item,
                class: props.class,
                style: props.style,
                on_key_down,
                on_focus,
                on_blur,
                on_paste,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: rendered_children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::TimeFieldSegment: as_child requires render_child; rendering the default span"
        );
    }

    rsx! {
        span {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
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
            "data-time-field-segment": attrs.data_time_field_segment,
            "data-placeholder": attrs.data_placeholder,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-readonly": attrs.data_readonly,
            onkeydown: move |evt| on_key_down.call(evt),
            onfocus: move |evt| on_focus.call(evt),
            onblur: move |evt| on_blur.call(evt),
            onpaste: move |evt| on_paste.call(evt),
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {rendered_children}
        }
    }
}

/// Render props passed to `as_child` for [`TimeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldLabelRenderProps {
    pub attrs: TimeFieldLabelAttrs,
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

/// Props for [`TimeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldLabelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<TimeFieldLabelRenderProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Accessible label for `TimeField`; clicking focuses the first interactive segment.
#[component]
pub fn TimeFieldLabel(props: TimeFieldLabelProps) -> Element {
    let runtime = use_time_field_context();
    let attrs = runtime.label_attrs(props.id.clone());

    let on_click = {
        let rt = runtime.clone();
        let user_cb = props.on_click;
        EventHandler::new(move |evt: MouseEvent| {
            if !rt.state.peek().def().disabled {
                rt.focus_first_segment();
            }
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(TimeFieldLabelRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                on_click,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::TimeFieldLabel: as_child requires render_child; rendering the default div"
        );
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            "data-time-field-label": attrs.data_time_field_label,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            onclick: move |evt| on_click.call(evt),
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

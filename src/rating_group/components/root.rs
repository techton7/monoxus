use std::borrow::Cow;

use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    compose::{
        extract_aria_invalid_and_describedby, merge_global_attributes,
        resolve_invalid_and_describedby,
    },
    form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
    shared::{ScopeHandle, use_direction_context, use_stable_id},
    state::sync_signal,
};

use crate::rating_group::{
    attrs::RatingGroupRootAttributes,
    runtime::{RatingGroupContext, translate_keyboard_key, use_rating_group_runtime},
    state::{RatingGroupDefinition, clamp_rating_value, format_rating_value},
    types::{Direction, Orientation, RatingGroupAriaValuetext, RatingGroupItemData},
};

/// Render props passed to `render_child` when `RatingGroupRoot` uses `as_child: true`.
#[derive(Clone, PartialEq, Props)]
pub struct RatingGroupRootRenderProps {
    pub id: String,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub attrs: RatingGroupRootAttributes,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_pointer_leave: EventHandler<PointerEvent>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Properties for [`RatingGroupRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct RatingGroupRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<f64>,
    #[props(default)]
    pub default_value: Option<f64>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<f64>>,
    #[props(default = 0.0)]
    pub min: f64,
    #[props(default = 5)]
    pub max: usize,
    #[props(default = false)]
    pub allow_half: bool,
    #[props(default = true)]
    pub hover_preview: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default)]
    pub orientation: Orientation,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub aria_valuetext: Option<RatingGroupAriaValuetext>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<RatingGroupRootRenderProps, Element>>,
    #[props(default)]
    pub render_items: Option<Callback<Vec<RatingGroupItemData>, Element>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_pointer_leave: Option<EventHandler<PointerEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless slider-based rating group root supporting whole/half-star ratings,
/// hover preview, keyboard/decimal input, RTL/vertical axes, and `<form>` reset sync.
#[component]
pub fn RatingGroupRoot(props: RatingGroupRootProps) -> Element {
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
        .or(field_labelledby);

    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("rating-group").child(stable_id.clone()));
    let rels = use_hook(|| {
        crate::rating_group::relationships::RatingGroupRelationships::new(scope.clone())
    });

    let explicit_id = props.id.clone().filter(|id| !id.trim().is_empty());
    let dom_root_id = if let Some(ref explicit) = explicit_id {
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
        rels.root_id().to_string()
    };

    let def = RatingGroupDefinition::new(scope)
        .with_min(props.min)
        .with_max(props.max)
        .with_allow_half(props.allow_half)
        .with_hover_preview(props.hover_preview)
        .with_disabled(effective_disabled)
        .with_read_only(effective_read_only)
        .with_required(effective_required)
        .with_orientation(props.orientation)
        .with_direction(resolved_dir);

    let initial_value_for_reset = use_hook(|| {
        clamp_rating_value(
            props.value.or(props.default_value).unwrap_or(0.0),
            def.min,
            def.max,
        )
    });

    let mut runtime = use_rating_group_runtime(
        def,
        rels.clone(),
        dom_root_id.clone(),
        props.value,
        props.default_value.unwrap_or(0.0),
        props.on_value_change,
        props.aria_label.clone(),
        effective_labelledby,
        props.aria_valuetext.clone(),
    );
    sync_signal(&mut runtime.invalid, effective_invalid);
    sync_signal(&mut runtime.aria_describedby, effective_describedby);

    let mut reset_runtime = runtime.clone();
    let (should_render_hidden, reset_epoch) = use_form_control_sync_with_epoch(
        &dom_root_id,
        effective_name.as_deref(),
        props.form.as_deref(),
        move || {
            reset_runtime.reset_to(initial_value_for_reset);
        },
    );

    use_context_provider(|| RatingGroupContext(runtime.clone()));

    let root_attrs = runtime.root_attrs();

    let mut consumer_on_key_down_slot = use_hook(|| CopyValue::new(props.on_key_down));
    consumer_on_key_down_slot.set(props.on_key_down);

    let mut consumer_on_pointer_leave_slot = use_hook(|| CopyValue::new(props.on_pointer_leave));
    consumer_on_pointer_leave_slot.set(props.on_pointer_leave);

    let mut consumer_on_blur_slot = use_hook(|| CopyValue::new(props.on_blur));
    consumer_on_blur_slot.set(props.on_blur);

    let mut consumer_on_focus_slot = use_hook(|| CopyValue::new(props.on_focus));
    consumer_on_focus_slot.set(props.on_focus);

    let on_key_down = use_hook(|| {
        let mut runtime = runtime.clone();
        Callback::new(move |evt: KeyboardEvent| {
            if let Some(handler) = consumer_on_key_down_slot.try_read().ok().and_then(|s| *s) {
                handler.call(evt.clone());
            }
            if !evt.default_action_enabled() {
                return;
            }
            let key = translate_keyboard_key(&evt.key());
            if runtime.handle_keydown(key) {
                evt.prevent_default();
            }
        })
    });

    let on_pointer_leave = use_hook(|| {
        let mut runtime = runtime.clone();
        Callback::new(move |evt: PointerEvent| {
            if let Some(handler) = consumer_on_pointer_leave_slot
                .try_read()
                .ok()
                .and_then(|s| *s)
            {
                handler.call(evt);
            }
            runtime.clear_hover_value();
        })
    });

    let content = if let Some(render_items) = props.render_items.as_ref() {
        render_items.call(runtime.items())
    } else {
        props.children
    };

    let hidden_input = if should_render_hidden {
        Some(rsx! {
            HiddenFormInput {
                id: rels.input_id().to_string(),
                kind: HiddenInputKind::Hidden,
                name: effective_name.clone(),
                value: format_rating_value(runtime.current_value()),
                disabled: effective_disabled,
                required: effective_required,
                read_only: effective_read_only,
                form: props.form.clone(),
                focus_target_id: dom_root_id.clone(),
                reset_epoch: reset_epoch,
            }
        })
    } else {
        None
    };

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            let rendered = render_child.call(RatingGroupRootRenderProps {
                id: dom_root_id,
                class: props.class,
                style: props.style,
                attrs: root_attrs,
                on_key_down,
                on_pointer_leave,
                on_blur: props.on_blur,
                on_focus: props.on_focus,
                on_mounted: props.on_mounted,
                attributes: extracted.rest,
                children: content,
            });
            return rsx! {
                {rendered}
                if let Some(input) = hidden_input {
                    {input}
                }
            };
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::RatingGroupRoot: as_child requires render_child; rendering the default div"
        );
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        extracted.rest,
    );

    rsx! {
        div {
            id: "{dom_root_id}",
            class: merged.class,
            style: merged.style,
            role: root_attrs.role,
            dir: root_attrs.dir,
            tabindex: root_attrs.tabindex,
            "aria-valuenow": "{root_attrs.aria_valuenow}",
            "aria-valuemin": "{root_attrs.aria_valuemin}",
            "aria-valuemax": "{root_attrs.aria_valuemax}",
            "aria-valuetext": root_attrs.aria_valuetext,
            "aria-orientation": root_attrs.aria_orientation,
            "aria-required": root_attrs.aria_required,
            "aria-disabled": root_attrs.aria_disabled,
            "aria-invalid": root_attrs.aria_invalid,
            "aria-describedby": root_attrs.aria_describedby.as_deref(),
            "aria-label": root_attrs.aria_label,
            "aria-labelledby": root_attrs.aria_labelledby,
            "data-orientation": root_attrs.data_orientation,
            "data-disabled": root_attrs.data_disabled,
            "data-readonly": root_attrs.data_readonly,
            "data-invalid": root_attrs.data_invalid,
            "data-rating-group-root": root_attrs.data_rating_group_root,
            onkeydown: move |evt| on_key_down.call(evt),
            onpointerleave: move |evt| on_pointer_leave.call(evt),
            onfocus: move |evt| {
                if let Some(handler) = consumer_on_focus_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            onblur: move |evt| {
                if let Some(handler) = consumer_on_blur_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            onmounted: move |evt| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {content}
        }
        if let Some(input) = hidden_input {
            {input}
        }
    }
}

pub use RatingGroupRoot as RatingGroup;

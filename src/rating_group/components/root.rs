use std::borrow::Cow;

use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    compose::{
        MergePropsConfig, collect_user_attrs, extract_aria_invalid_and_describedby,
        merge_props_with_config, push_attr_opt, push_event_handler,
        resolve_invalid_and_describedby,
    },
    form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
    shared::{ScopeHandle, use_direction_context, use_stable_id},
    state::sync_signal,
};

use crate::rating_group::{
    runtime::{RatingGroupContext, translate_keyboard_key, use_rating_group_runtime},
    state::{RatingGroupDefinition, clamp_rating_value, format_rating_value},
    types::{Direction, Orientation, RatingGroupAriaValuetext, RatingGroupItemData},
};

/// Properties for [`RatingGroupRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct RatingGroupRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
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
    #[props(default, into)]
    pub name: Option<String>,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default, into)]
    pub aria_label: Option<String>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub aria_valuetext: Option<RatingGroupAriaValuetext>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let mut base = Vec::with_capacity(24);
    base.push(Attribute::new("id", dom_root_id, None, false));
    base.push(Attribute::new("role", root_attrs.role, None, false));
    base.push(Attribute::new("dir", root_attrs.dir, None, false));
    base.push(Attribute::new("tabindex", root_attrs.tabindex, None, false));
    base.push(Attribute::new(
        "aria-valuenow",
        root_attrs.aria_valuenow.to_string(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuemin",
        root_attrs.aria_valuemin.to_string(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuemax",
        root_attrs.aria_valuemax.to_string(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuetext",
        root_attrs.aria_valuetext,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-orientation",
        root_attrs.aria_orientation,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-required",
        root_attrs.aria_required,
        None,
        false,
    ));
    push_attr_opt(&mut base, "aria-disabled", root_attrs.aria_disabled);
    push_attr_opt(&mut base, "aria-invalid", root_attrs.aria_invalid);
    push_attr_opt(&mut base, "aria-describedby", root_attrs.aria_describedby);
    push_attr_opt(&mut base, "aria-label", root_attrs.aria_label);
    push_attr_opt(&mut base, "aria-labelledby", root_attrs.aria_labelledby);
    base.push(Attribute::new(
        "data-orientation",
        root_attrs.data_orientation,
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", root_attrs.data_disabled);
    push_attr_opt(&mut base, "data-readonly", root_attrs.data_readonly);
    push_attr_opt(&mut base, "data-invalid", root_attrs.data_invalid);
    base.push(Attribute::new(
        "data-rating-group-root",
        root_attrs.data_rating_group_root,
        None,
        false,
    ));

    {
        let mut runtime = runtime.clone();
        push_event_handler(
            &mut base,
            "onkeydown",
            Some(EventHandler::new(move |evt: KeyboardEvent| {
                let key = translate_keyboard_key(&evt.key());
                if runtime.handle_keydown(key) {
                    evt.prevent_default();
                }
            })),
        );
    }
    {
        let mut runtime = runtime.clone();
        push_event_handler(
            &mut base,
            "onpointerleave",
            Some(EventHandler::new(move |_evt: PointerEvent| {
                runtime.clear_hover_value();
            })),
        );
    }

    let mut user = collect_user_attrs(props.class, props.style, extracted.rest);
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
    push_event_handler(&mut user, "onpointerleave", props.on_pointer_leave);
    push_event_handler(&mut user, "onfocus", props.on_focus);
    push_event_handler(&mut user, "onblur", props.on_blur);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &["onpointerleave"],
        },
    );

    let root_el = if let Some(child) = props.child {
        child.call(merged)
    } else {
        rsx! {
            div {
                ..merged,
                {content}
            }
        }
    };

    rsx! {
        {root_el}
        if let Some(input) = hidden_input {
            {input}
        }
    }
}

pub use RatingGroupRoot as RatingGroup;

use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
    shared::{ScopeHandle, use_direction_context, use_stable_id},
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
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("rating-group").child(stable_id.clone()));
    let rels = use_hook(|| {
        crate::rating_group::relationships::RatingGroupRelationships::new(scope.clone())
    });
    let dom_root_id = props.id.clone().unwrap_or_else(|| rels.root_id().to_string());

    let def = RatingGroupDefinition::new(scope)
        .with_min(props.min)
        .with_max(props.max)
        .with_allow_half(props.allow_half)
        .with_hover_preview(props.hover_preview)
        .with_disabled(props.disabled)
        .with_read_only(props.read_only)
        .with_required(props.required)
        .with_orientation(props.orientation)
        .with_direction(resolved_dir);

    let initial_value_for_reset = use_hook(|| {
        clamp_rating_value(
            props.value.or(props.default_value).unwrap_or(0.0),
            def.min,
            def.max,
        )
    });

    let runtime = use_rating_group_runtime(
        def,
        rels.clone(),
        dom_root_id.clone(),
        props.value,
        props.default_value.unwrap_or(0.0),
        props.on_value_change,
        props.aria_label.clone(),
        props.aria_labelledby.clone(),
        props.aria_valuetext.clone(),
    );

    let mut reset_runtime = runtime.clone();
    let (should_render_hidden, reset_epoch) = use_form_control_sync_with_epoch(
        &dom_root_id,
        props.name.as_deref(),
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
                name: props.name.clone(),
                value: format_rating_value(runtime.current_value()),
                disabled: props.disabled,
                required: props.required,
                read_only: props.read_only,
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
                on_mounted: props.on_mounted,
                attributes: props.attributes,
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
        props.attributes,
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
            "aria-label": root_attrs.aria_label,
            "aria-labelledby": root_attrs.aria_labelledby,
            "data-orientation": root_attrs.data_orientation,
            "data-disabled": root_attrs.data_disabled,
            "data-readonly": root_attrs.data_readonly,
            "data-rating-group-root": root_attrs.data_rating_group_root,
            onkeydown: move |evt| on_key_down.call(evt),
            onpointerleave: move |evt| on_pointer_leave.call(evt),
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

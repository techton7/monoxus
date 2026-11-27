use std::borrow::Cow;

use dioxus::core::{DynamicNode, VNode, VNodeChild};
use dioxus::prelude::*;
use monoxus_core::label::LABEL_INTERACTIVE_TARGET_SELECTOR;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::use_stable_id,
    state::sync_signal,
};

use super::{
    attrs::{
        FieldControlAttrs, FieldDescriptionAttrs, FieldErrorAttrs, FieldLabelAttrs,
        FieldLegendAttrs, FieldRootAttrs, FieldSetAttrs,
    },
    relationships::FieldRelationships,
    runtime::{
        FieldContext, FieldControlContext, RegistrationToken, try_use_field_context,
        try_use_field_control_context, use_field_control,
    },
    state::{FieldErrorRegistration, dedupe_error_messages},
    types::{FieldErrorLiveMode, FieldLegendVariant, FieldOrientation},
};

/// Render props passed to `render_child` when [`FieldRoot`] uses `as_child: true`.
#[derive(Clone)]
pub struct FieldRootRenderChildProps {
    pub attrs: FieldRootAttrs,
    pub children: Element,
}

/// Render props passed to `render_child` when [`FieldSet`] uses `as_child: true`.
#[derive(Clone)]
pub struct FieldSetRenderChildProps {
    pub attrs: FieldSetAttrs,
    pub children: Element,
}

/// Render props passed to `render_child` when [`FieldLegend`] uses `as_child: true`.
#[derive(Clone)]
pub struct FieldLegendRenderChildProps {
    pub attrs: FieldLegendAttrs,
    pub children: Element,
}

/// Render props passed to `render_child` when [`FieldLabel`] uses `as_child: true`.
#[derive(Clone)]
pub struct FieldLabelRenderChildProps {
    pub attrs: FieldLabelAttrs,
    pub children: Element,
}

/// Render props passed to `render_child` when [`FieldDescription`] uses `as_child: true`.
#[derive(Clone)]
pub struct FieldDescriptionRenderChildProps {
    pub attrs: FieldDescriptionAttrs,
    pub children: Element,
}

/// Render props passed to `render_child` when [`FieldError`] uses `as_child: true`.
#[derive(Clone)]
pub struct FieldErrorRenderChildProps {
    pub attrs: FieldErrorAttrs,
    pub errors: Vec<String>,
    pub active: bool,
    pub children: Element,
}

/// Plain-fn helper initializing and syncing a [`FieldContext`] (zero internal `#[component]`
/// taking `Option<EventHandler>` per `Dioxus #5894`).
#[allow(clippy::too_many_arguments)]
fn use_field_provider(
    explicit_id: Option<String>,
    name: Option<String>,
    invalid: bool,
    disabled: bool,
    required: bool,
    read_only: bool,
    orientation: FieldOrientation,
    is_fieldset: bool,
) -> FieldContext {
    let stable_id = use_stable_id(None);
    let build_rels = |id_override: Option<String>| {
        let base = FieldRelationships::new(stable_id.clone());
        match (is_fieldset, id_override) {
            (false, Some(id)) => base.with_root_id(id),
            (true, Some(id)) => base.with_fieldset_id(id),
            (_, None) => base,
        }
    };

    let rels_now = build_rels(explicit_id);
    let mut relationships = use_hook(|| CopyValue::new(rels_now.clone()));
    if relationships
        .try_read()
        .is_ok_and(|cur| *cur != rels_now)
    {
        relationships.set(rels_now);
    }

    let mut name_sig = use_signal(|| name.clone());
    sync_signal(&mut name_sig, name);
    let mut invalid_sig = use_signal(|| invalid);
    sync_signal(&mut invalid_sig, invalid);
    let mut disabled_sig = use_signal(|| disabled);
    sync_signal(&mut disabled_sig, disabled);
    let mut required_sig = use_signal(|| required);
    sync_signal(&mut required_sig, required);
    let mut read_only_sig = use_signal(|| read_only);
    sync_signal(&mut read_only_sig, read_only);
    let mut orientation_sig = use_signal(|| orientation);
    sync_signal(&mut orientation_sig, orientation);
    let mut is_fieldset_sig = use_signal(|| is_fieldset);
    sync_signal(&mut is_fieldset_sig, is_fieldset);

    let primary_control_id = use_signal(|| None::<String>);
    let mounted_labels = use_signal(Vec::<(RegistrationToken, String)>::new);
    let mounted_legend = use_signal(Vec::<(RegistrationToken, String)>::new);
    let mounted_descriptions = use_signal(Vec::<(RegistrationToken, String)>::new);
    let mounted_errors = use_signal(Vec::<(RegistrationToken, FieldErrorRegistration)>::new);
    let next_control_index = use_hook(|| CopyValue::new(0_usize));

    let ctx = FieldContext {
        relationships,
        name: name_sig,
        invalid: invalid_sig,
        disabled: disabled_sig,
        required: required_sig,
        read_only: read_only_sig,
        orientation: orientation_sig,
        is_fieldset: is_fieldset_sig,
        primary_control_id,
        mounted_labels,
        mounted_legend,
        mounted_descriptions,
        mounted_errors,
        next_control_index,
    };

    use_context_provider(|| ctx);
    ctx
}

/// Plain-fn helper handling double-click text-selection suppression on `<label>` while
/// preserving interaction on nested interactive targets (`LABEL_INTERACTIVE_TARGET_SELECTOR`).
fn handle_field_label_mouse_down(
    evt: MouseEvent,
    on_mouse_down_slot: CopyValue<Option<EventHandler<MouseEvent>>>,
) {
    if let Some(target) = oxidase::dom::mouse_event_target(&evt.data())
        && target.closest(LABEL_INTERACTIVE_TARGET_SELECTOR).is_some()
    {
        return;
    }

    if let Some(handler) = on_mouse_down_slot.try_read().ok().and_then(|slot| *slot) {
        handler.call(evt.clone());
    }

    if evt.default_action_enabled() && oxidase::dom::mouse_event_detail(&evt.data()) > 1 {
        evt.prevent_default();
    }
}

fn element_has_content(children: &Element) -> bool {
    let Ok(vnode) = children else {
        return false;
    };
    vnode_has_content(vnode)
}

fn vnode_has_content(vnode: &VNode) -> bool {
    vnode.children().any(child_has_content)
}

fn child_has_content(child: VNodeChild<'_>) -> bool {
    match child {
        VNodeChild::Text(static_text) => !static_text.text().trim().is_empty(),
        VNodeChild::Element(_) => true,
        VNodeChild::Dynamic(anchor) => anchor.nodes().any(|slot| match &*slot {
            DynamicNode::Text(vtext) => !vtext.value.trim().is_empty(),
            DynamicNode::Fragment(nodes) => nodes.iter().any(vnode_has_content),
            DynamicNode::Component(_) => true,
        }),
    }
}

/// Properties for [`FieldRoot`] (aliased as [`Field`]).
#[derive(Props, Clone, PartialEq)]
pub struct FieldRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub name: Option<String>,
    #[props(default = false)]
    pub invalid: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub orientation: FieldOrientation,
    #[props(default = Some("group"))]
    pub role: Option<&'static str>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<FieldRootRenderChildProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless field container (`<div role="group">`) coordinating deterministic IDs, mounted-only
/// `aria-describedby` wiring, and `invalid`/`disabled`/`required`/`read_only`/`orientation` state.
#[component]
pub fn FieldRoot(props: FieldRootProps) -> Element {
    let ctx = use_field_provider(
        props.id.clone(),
        props.name.clone(),
        props.invalid,
        props.disabled,
        props.required,
        props.read_only,
        props.orientation,
        false,
    );

    let mut on_mounted_slot = use_hook(|| CopyValue::new(props.on_mounted));
    on_mounted_slot.set(props.on_mounted);

    let def = ctx.definition();
    let attrs = FieldRootAttrs::from_definition(
        &def,
        props.role,
        ctx.aria_labelledby(),
        ctx.aria_describedby(None),
    );

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(FieldRootRenderChildProps {
            attrs,
            children: props.children,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: attrs.id.as_str(),
            role: attrs.role,
            "aria-labelledby": attrs.aria_labelledby.as_deref(),
            "aria-describedby": attrs.aria_describedby.as_deref(),
            "data-orientation": attrs.data_orientation,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            "data-readonly": attrs.data_readonly,
            "data-monoxus-field-root": attrs.data_monoxus_field_root,
            class: merged.class,
            style: merged.style,
            onmounted: move |evt| {
                if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

pub use FieldRoot as Field;

/// Properties for [`FieldSet`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldSetProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub name: Option<String>,
    #[props(default = false)]
    pub invalid: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub orientation: FieldOrientation,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<FieldSetRenderChildProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless `<fieldset>` container supporting 1:N multi-control groups via [`FieldControl`],
/// [`FieldLegend`], shared [`FieldDescription`], and [`FieldError`].
#[component]
pub fn FieldSet(props: FieldSetProps) -> Element {
    let parent_ctx = try_use_field_context();
    let effective_invalid = props.invalid || parent_ctx.is_some_and(|p| p.is_invalid());
    let effective_disabled = props.disabled || parent_ctx.is_some_and(|p| p.is_disabled());
    let effective_required = props.required || parent_ctx.is_some_and(|p| p.is_required());
    let effective_read_only = props.read_only || parent_ctx.is_some_and(|p| p.is_read_only());

    let ctx = use_field_provider(
        props.id.clone(),
        props.name.clone(),
        effective_invalid,
        effective_disabled,
        effective_required,
        effective_read_only,
        props.orientation,
        true,
    );

    let mut on_mounted_slot = use_hook(|| CopyValue::new(props.on_mounted));
    on_mounted_slot.set(props.on_mounted);

    let def = ctx.definition();
    let attrs = FieldSetAttrs::from_definition(&def, ctx.aria_describedby(None));

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(FieldSetRenderChildProps {
            attrs,
            children: props.children,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        fieldset {
            id: attrs.id.as_str(),
            disabled: attrs.disabled,
            "aria-invalid": attrs.aria_invalid,
            "aria-describedby": attrs.aria_describedby.as_deref(),
            "data-orientation": attrs.data_orientation,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            "data-readonly": attrs.data_readonly,
            "data-monoxus-field-set": attrs.data_monoxus_field_set,
            class: merged.class,
            style: merged.style,
            onmounted: move |evt| {
                if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

/// Properties for [`FieldLegend`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldLegendProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub variant: FieldLegendVariant,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<FieldLegendRenderChildProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless `<legend>` caption for [`FieldSet`].
#[component]
pub fn FieldLegend(props: FieldLegendProps) -> Element {
    let field_ctx = try_use_field_context();
    let fallback_id = use_stable_id(None);
    let token = use_hook(RegistrationToken::next);

    let legend_id = props
        .id
        .clone()
        .filter(|id| !id.trim().is_empty())
        .or_else(|| field_ctx.map(|ctx| ctx.relationships().legend_id().to_owned()))
        .unwrap_or_else(|| fallback_id.into_owned());

    if let Some(ctx) = field_ctx {
        ctx.register_legend(token, legend_id.clone());
    }

    use_drop(move || {
        if let Some(ctx) = field_ctx {
            ctx.unregister_legend(token);
        }
    });

    let mut on_mounted_slot = use_hook(|| CopyValue::new(props.on_mounted));
    on_mounted_slot.set(props.on_mounted);

    let attrs = FieldLegendAttrs::new(
        legend_id,
        props.variant,
        field_ctx.is_some_and(|c| c.is_invalid()),
        field_ctx.is_some_and(|c| c.is_disabled()),
        field_ctx.is_some_and(|c| c.is_required()),
        field_ctx.is_some_and(|c| c.is_read_only()),
    );

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(FieldLegendRenderChildProps {
            attrs,
            children: props.children,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        legend {
            id: attrs.id.as_str(),
            "data-variant": attrs.data_variant,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            "data-readonly": attrs.data_readonly,
            "data-monoxus-field-legend": attrs.data_monoxus_field_legend,
            class: merged.class,
            style: merged.style,
            onmounted: move |evt| {
                if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

/// Properties for [`FieldLabel`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldLabelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub r#for: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<FieldLabelRenderChildProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mouse_down: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless `<label>` that auto-pairs `r#for` with its enclosing [`FieldControl`] or [`FieldRoot`]
/// (omitting `r#for` directly inside [`FieldSet`] without a [`FieldControl`]).
#[component]
pub fn FieldLabel(props: FieldLabelProps) -> Element {
    let field_ctx = try_use_field_context();
    let control_ctx = try_use_field_control_context();
    let fallback_id = use_stable_id(None);
    let token = use_hook(RegistrationToken::next);

    let label_id = props
        .id
        .clone()
        .filter(|id| !id.trim().is_empty())
        .or_else(|| control_ctx.map(|c| c.default_label_id()))
        .or_else(|| field_ctx.map(|c| c.relationships().label_id().to_owned()))
        .unwrap_or_else(|| fallback_id.into_owned());

    let html_for = props
        .r#for
        .clone()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| control_ctx.map(|c| c.control_id()))
        .or_else(|| {
            field_ctx.and_then(|c| (!c.is_fieldset()).then(|| c.resolved_control_id()))
        });

    if let Some(ctrl_ctx) = control_ctx {
        ctrl_ctx.register_label(token, label_id.clone());
    } else if let Some(f_ctx) = field_ctx {
        f_ctx.register_label(token, label_id.clone());
    }

    use_drop(move || {
        if let Some(ctrl_ctx) = control_ctx {
            ctrl_ctx.unregister_label(token);
        } else if let Some(f_ctx) = field_ctx {
            f_ctx.unregister_label(token);
        }
    });

    let mut on_click_slot = use_hook(|| CopyValue::new(props.on_click));
    on_click_slot.set(props.on_click);
    let mut on_mouse_down_slot = use_hook(|| CopyValue::new(props.on_mouse_down));
    on_mouse_down_slot.set(props.on_mouse_down);
    let mut on_mounted_slot = use_hook(|| CopyValue::new(props.on_mounted));
    on_mounted_slot.set(props.on_mounted);

    let attrs = FieldLabelAttrs::new(
        label_id,
        html_for,
        field_ctx.is_some_and(|c| c.is_invalid()),
        field_ctx.is_some_and(|c| c.is_disabled()),
        field_ctx.is_some_and(|c| c.is_required()),
        field_ctx.is_some_and(|c| c.is_read_only()),
    );

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(FieldLabelRenderChildProps {
            attrs,
            children: props.children,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        label {
            id: attrs.id.as_str(),
            r#for: attrs.r#for.as_deref(),
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            "data-readonly": attrs.data_readonly,
            "data-monoxus-field-label": attrs.data_monoxus_field_label,
            class: merged.class,
            style: merged.style,
            onclick: move |evt| {
                if let Some(handler) = on_click_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            onmousedown: move |evt| {
                handle_field_label_mouse_down(evt, on_mouse_down_slot);
            },
            onmounted: move |evt| {
                if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

/// Properties for [`FieldControl`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldControlProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<FieldControlAttrs, Element>>,
    #[props(default)]
    pub children: Element,
}

/// Renderless control-scoping context provider enabling 1:N control/label pairing inside
/// [`FieldSet`] or [`FieldRoot`], with optional `render_child: Callback<FieldControlAttrs, Element>`.
#[component]
pub fn FieldControl(props: FieldControlProps) -> Element {
    let field_ctx = try_use_field_context();
    let fallback_id = use_stable_id(props.id.clone().map(Cow::Owned));

    let slot_index = use_hook(|| {
        field_ctx.map_or(0_usize, |ctx| ctx.allocate_control_index())
    });

    let explicit_clean = props.id.clone().filter(|id| !id.trim().is_empty());
    let default_control_id = field_ctx
        .map(|ctx| ctx.relationships().control_id_indexed(slot_index))
        .unwrap_or_else(|| fallback_id.into_owned());
    let default_label_id = field_ctx
        .map(|ctx| ctx.relationships().label_id_indexed(slot_index))
        .unwrap_or_else(|| format!("{default_control_id}-label"));

    let resolved_control_id = explicit_clean
        .clone()
        .unwrap_or_else(|| default_control_id.clone());

    let mut control_id_sig = use_signal(|| resolved_control_id.clone());
    if explicit_clean.is_some() {
        sync_signal(&mut control_id_sig, resolved_control_id.clone());
    }

    let mut explicit_id_locked = use_hook(|| CopyValue::new(explicit_clean.is_some()));
    explicit_id_locked.set(explicit_clean.is_some());

    let default_label_slot = use_hook(|| CopyValue::new(default_label_id));
    let mounted_labels = use_signal(Vec::<(RegistrationToken, String)>::new);

    let control_ctx = FieldControlContext {
        control_id: control_id_sig,
        default_label_id: default_label_slot,
        mounted_labels,
        explicit_id_locked,
    };
    use_context_provider(|| control_ctx);

    if slot_index == 0
        && let Some(f_ctx) = field_ctx
        && !f_ctx.is_fieldset()
    {
        f_ctx.set_primary_control_id(Some(control_ctx.control_id()));
    }

    if let Some(render_child) = props.render_child.as_ref() {
        let attrs = use_field_control()
            .map(|handle| handle.into_attrs())
            .unwrap_or_else(|| {
                FieldControlAttrs::new(
                    control_ctx.control_id(),
                    None,
                    control_ctx.mounted_label_id(),
                    None,
                    false,
                    false,
                    false,
                    false,
                )
            });
        return render_child.call(attrs);
    }

    props.children
}

/// Properties for [`FieldDescription`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldDescriptionProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<FieldDescriptionRenderChildProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless helper description (`<p>`) that dynamically registers its ID into the enclosing
/// field's `aria-describedby` while mounted and cleans up safely via [`RegistrationToken`].
#[component]
pub fn FieldDescription(props: FieldDescriptionProps) -> Element {
    let field_ctx = try_use_field_context();
    let fallback_id = use_stable_id(None);
    let token = use_hook(RegistrationToken::next);

    let desc_id = props
        .id
        .clone()
        .filter(|id| !id.trim().is_empty())
        .or_else(|| field_ctx.map(|c| c.relationships().description_id().to_owned()))
        .unwrap_or_else(|| fallback_id.into_owned());

    if let Some(ctx) = field_ctx {
        ctx.register_description(token, desc_id.clone());
    }

    use_drop(move || {
        if let Some(ctx) = field_ctx {
            ctx.unregister_description(token);
        }
    });

    let mut on_mounted_slot = use_hook(|| CopyValue::new(props.on_mounted));
    on_mounted_slot.set(props.on_mounted);

    let attrs = FieldDescriptionAttrs::new(
        desc_id,
        field_ctx.is_some_and(|c| c.is_invalid()),
        field_ctx.is_some_and(|c| c.is_disabled()),
    );

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(FieldDescriptionRenderChildProps {
            attrs,
            children: props.children,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        p {
            id: attrs.id.as_str(),
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-monoxus-field-description": attrs.data_monoxus_field_description,
            class: merged.class,
            style: merged.style,
            onmounted: move |evt| {
                if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

/// Properties for [`FieldError`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldErrorProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub errors: Vec<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub live_mode: FieldErrorLiveMode,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<FieldErrorRenderChildProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless error announcement container (`<div role="alert">`) that registers its ID into
/// `aria-describedby` strictly when mounted and active, renders single errors as text or
/// multiple deduplicated errors as `<ul><li>`, and omits DOM output when inactive unless
/// `force_mount: true`.
#[component]
pub fn FieldError(props: FieldErrorProps) -> Element {
    let field_ctx = try_use_field_context();
    let fallback_id = use_stable_id(None);
    let token = use_hook(RegistrationToken::next);

    let error_id = props
        .id
        .clone()
        .filter(|id| !id.trim().is_empty())
        .or_else(|| field_ctx.map(|c| c.relationships().error_id().to_owned()))
        .unwrap_or_else(|| fallback_id.into_owned());

    let unique_errors = dedupe_error_messages(&props.errors);
    let has_children = element_has_content(&props.children);
    let has_errors = !unique_errors.is_empty();
    let field_invalid = field_ctx.is_some_and(|c| c.is_invalid());
    let active = if field_ctx.is_some() {
        has_errors || (has_children && field_invalid)
    } else {
        has_errors || has_children
    };

    let should_render = active || props.force_mount;

    if let Some(ctx) = field_ctx {
        if should_render {
            ctx.register_error(token, error_id.clone(), active);
        } else {
            ctx.unregister_error(token);
        }
    }

    use_drop(move || {
        if let Some(ctx) = field_ctx {
            ctx.unregister_error(token);
        }
    });

    let mut on_mounted_slot = use_hook(|| CopyValue::new(props.on_mounted));
    on_mounted_slot.set(props.on_mounted);

    if !should_render {
        return rsx! {};
    }

    let attrs = FieldErrorAttrs::new(
        error_id,
        active,
        props.live_mode,
        field_invalid,
        field_ctx.is_some_and(|c| c.is_disabled()),
    );

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(FieldErrorRenderChildProps {
            attrs,
            errors: unique_errors,
            active,
            children: props.children,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: attrs.id.as_str(),
            role: attrs.role,
            "aria-live": attrs.aria_live,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            "data-monoxus-field-error": attrs.data_monoxus_field_error,
            class: merged.class,
            style: merged.style,
            onmounted: move |evt| {
                if let Some(handler) = on_mounted_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            if has_children {
                {props.children}
            } else if unique_errors.len() == 1 {
                "{unique_errors[0]}"
            } else if unique_errors.len() > 1 {
                ul {
                    for err in unique_errors {
                        li { "{err}" }
                    }
                }
            }
        }
    }
}

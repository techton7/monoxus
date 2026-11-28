use std::{borrow::Cow, rc::Rc};

use dioxus::core::{DynamicNode, VNode, VNodeChild};
use dioxus::prelude::*;
use ui_sdk::label::LABEL_INTERACTIVE_TARGET_SELECTOR;

use crate::{
    agent::{
        AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult, SnapshotValue,
        SurfaceMetaDefaults, use_agent_surface,
    },
    foundation::{
        compose::{ChildProps, collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
        shared::use_stable_id,
        state::sync_signal,
    },
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

/// Runtime state exposed alongside merged DOM `props` when [`FieldError`] uses `child`.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldErrorChildState {
    pub errors: Vec<String>,
    pub active: bool,
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
    let mounted_error_messages = use_signal(Vec::<(RegistrationToken, Vec<String>)>::new);
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
        mounted_error_messages,
        next_control_index,
    };

    use_context_provider(|| ctx);
    ctx
}

/// Plain-fn helper handling double-click text-selection suppression on `<label>` while
/// preserving interaction on nested interactive targets (`LABEL_INTERACTIVE_TARGET_SELECTOR`).
fn handle_field_label_mouse_down(evt: MouseEvent, on_mouse_down: Option<EventHandler<MouseEvent>>) {
    if let Some(target) = oxidase::dom::mouse_event_target(&evt.data())
        && target.closest(LABEL_INTERACTIVE_TARGET_SELECTOR).is_some()
    {
        return;
    }

    if let Some(handler) = on_mouse_down {
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
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
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
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
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

    let field_invalid = ctx.is_invalid();
    let field_errors = ctx.active_error_messages();
    let has_active_error = field_invalid || !field_errors.is_empty();
    let effective_agent_hints = if !has_active_error {
        let mut h = props.agent.clone().unwrap_or_default();
        h.hidden = true;
        Some(h)
    } else {
        props.agent.clone()
    };

    let build_field_snapshot = |invalid: bool, errors: &[String]| -> ComponentSnapshot {
        let mut snap = ComponentSnapshot::new()
            .with_field("invalid", SnapshotValue::Bool(invalid))
            .with_field(
                "errors",
                SnapshotValue::List(
                    errors
                        .iter()
                        .cloned()
                        .map(SnapshotValue::Text)
                        .collect(),
                ),
            );
        if !errors.is_empty() {
            snap = snap.with_error(errors.join("; "));
        } else if invalid {
            snap = snap.with_error("Invalid field");
        }
        snap
    };

    use_agent_surface(
        effective_agent_hints.as_ref(),
        SurfaceMetaDefaults::default(),
        || {
            let label = props
                .name
                .clone()
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| String::from("Field"));
            AgentSurface::new("field", label)
                .with_disabled(props.disabled)
                .with_snapshot(build_field_snapshot(field_invalid, &field_errors))
        },
        || Rc::new(|_| DispatchExecResult::ok().with_host_error("field is read-only")),
        || {
            let ctx_reader = ctx;
            Rc::new(move || {
                let inv = ctx_reader
                    .invalid
                    .try_peek()
                    .map(|g| *g)
                    .unwrap_or(false);
                let errs = ctx_reader.peek_active_error_messages();
                let mut snap = ComponentSnapshot::new()
                    .with_field("invalid", SnapshotValue::Bool(inv))
                    .with_field(
                        "errors",
                        SnapshotValue::List(errs.iter().cloned().map(SnapshotValue::Text).collect()),
                    );
                if !errs.is_empty() {
                    snap = snap.with_error(errs.join("; "));
                } else if inv {
                    snap = snap.with_error("Invalid field");
                }
                snap
            })
        },
    );

    let def = ctx.definition();
    let attrs = FieldRootAttrs::from_definition(
        &def,
        props.role,
        ctx.aria_labelledby(),
        ctx.aria_describedby(None),
    );

    let mut base = Vec::with_capacity(10);
    base.push(Attribute::new("id", attrs.id, None, false));
    push_attr_opt(&mut base, "role", attrs.role);
    push_attr_opt(&mut base, "aria-labelledby", attrs.aria_labelledby);
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby);
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation,
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    base.push(Attribute::new(
        "data-monoxus-field-root",
        attrs.data_monoxus_field_root,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

pub use FieldRoot as Field;

/// Properties for [`FieldSet`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldSetProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
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
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
    // Capture any enclosing parent FieldContext once on mount before `use_field_provider`
    // registers this FieldSet's own FieldContext into the current scope (otherwise re-renders
    // would read self-context and permanently latch `invalid`/`disabled`/`required`/`read_only`).
    let parent_ctx = use_hook(try_use_field_context);
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

    let def = ctx.definition();
    let attrs = FieldSetAttrs::from_definition(&def, ctx.aria_describedby(None));

    let mut base = Vec::with_capacity(10);
    base.push(Attribute::new("id", attrs.id, None, false));
    if attrs.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_attr_opt(&mut base, "aria-invalid", attrs.aria_invalid);
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby);
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation,
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    base.push(Attribute::new(
        "data-monoxus-field-set",
        attrs.data_monoxus_field_set,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        fieldset {
            ..merged,
            {props.children}
        }
    }
}

/// Properties for [`FieldLegend`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldLegendProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub variant: FieldLegendVariant,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let attrs = FieldLegendAttrs::new(
        legend_id,
        props.variant,
        field_ctx.is_some_and(|c| c.is_invalid()),
        field_ctx.is_some_and(|c| c.is_disabled()),
        field_ctx.is_some_and(|c| c.is_required()),
        field_ctx.is_some_and(|c| c.is_read_only()),
    );

    let mut base = Vec::with_capacity(8);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new(
        "data-variant",
        attrs.data_variant,
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    base.push(Attribute::new(
        "data-monoxus-field-legend",
        attrs.data_monoxus_field_legend,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        legend {
            ..merged,
            {props.children}
        }
    }
}

/// Properties for [`FieldLabel`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldLabelProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub r#for: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let attrs = FieldLabelAttrs::new(
        label_id,
        html_for,
        field_ctx.is_some_and(|c| c.is_invalid()),
        field_ctx.is_some_and(|c| c.is_disabled()),
        field_ctx.is_some_and(|c| c.is_required()),
        field_ctx.is_some_and(|c| c.is_read_only()),
    );

    let mut base = Vec::with_capacity(9);
    base.push(Attribute::new("id", attrs.id, None, false));
    push_attr_opt(&mut base, "for", attrs.r#for);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    base.push(Attribute::new(
        "data-monoxus-field-label",
        attrs.data_monoxus_field_label,
        None,
        false,
    ));
    {
        let on_mouse_down = props.on_mouse_down;
        push_event_handler(
            &mut base,
            "onmousedown",
            Some(EventHandler::new(move |evt: MouseEvent| {
                handle_field_label_mouse_down(evt, on_mouse_down);
            })),
        );
    }

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onclick", props.on_click);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        label {
            ..merged,
            {props.children}
        }
    }
}

/// Properties for [`FieldControl`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldControlProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub child: Option<Callback<ChildProps<FieldControlAttrs>, Element>>,
    #[props(default)]
    pub children: Element,
}

/// Renderless control-scoping context provider enabling 1:N control/label pairing inside
/// [`FieldSet`] or [`FieldRoot`], with optional `child: Callback<ChildProps<FieldControlAttrs>, Element>`.
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

    if let Some(child) = props.child {
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
        let mut merged = Vec::with_capacity(16);
        merged.push(Attribute::new("id", attrs.id.clone(), None, false));
        push_attr_opt(&mut merged, "name", attrs.name.clone());
        push_attr_opt(&mut merged, "aria-labelledby", attrs.aria_labelledby.clone());
        push_attr_opt(&mut merged, "aria-describedby", attrs.aria_describedby.clone());
        push_attr_opt(&mut merged, "aria-invalid", attrs.aria_invalid);
        push_attr_opt(&mut merged, "aria-required", attrs.aria_required);
        push_attr_opt(&mut merged, "aria-disabled", attrs.aria_disabled);
        push_attr_opt(&mut merged, "aria-readonly", attrs.aria_readonly);
        if attrs.disabled {
            merged.push(Attribute::new("disabled", true, None, false));
        }
        if attrs.required {
            merged.push(Attribute::new("required", true, None, false));
        }
        if attrs.read_only {
            merged.push(Attribute::new("readonly", true, None, false));
        }
        push_attr_opt(&mut merged, "data-invalid", attrs.data_invalid);
        push_attr_opt(&mut merged, "data-disabled", attrs.data_disabled);
        push_attr_opt(&mut merged, "data-required", attrs.data_required);
        push_attr_opt(&mut merged, "data-readonly", attrs.data_readonly);
        merged.push(Attribute::new(
            "data-monoxus-field-control",
            attrs.data_monoxus_field_control,
            None,
            false,
        ));
        return child.call(ChildProps::new(merged, attrs));
    }

    props.children
}

/// Properties for [`FieldDescription`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldDescriptionProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let attrs = FieldDescriptionAttrs::new(
        desc_id,
        field_ctx.is_some_and(|c| c.is_invalid()),
        field_ctx.is_some_and(|c| c.is_disabled()),
    );

    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("id", attrs.id, None, false));
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    base.push(Attribute::new(
        "data-monoxus-field-description",
        attrs.data_monoxus_field_description,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        p {
            ..merged,
            {props.children}
        }
    }
}

/// Properties for [`FieldError`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldErrorProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub errors: Vec<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub live_mode: FieldErrorLiveMode,
    #[props(default)]
    pub child: Option<Callback<ChildProps<FieldErrorChildState>, Element>>,
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
            if active && !unique_errors.is_empty() {
                ctx.register_error_messages(token, unique_errors.clone());
            } else {
                ctx.unregister_error_messages(token);
            }
        } else {
            ctx.unregister_error(token);
            ctx.unregister_error_messages(token);
        }
    }

    use_drop(move || {
        if let Some(ctx) = field_ctx {
            ctx.unregister_error(token);
            ctx.unregister_error_messages(token);
        }
    });

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

    let mut base = Vec::with_capacity(7);
    base.push(Attribute::new("id", attrs.id, None, false));
    push_attr_opt(&mut base, "role", attrs.role);
    push_attr_opt(&mut base, "aria-live", attrs.aria_live);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    base.push(Attribute::new(
        "data-monoxus-field-error",
        attrs.data_monoxus_field_error,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(ChildProps::new(
            merged,
            FieldErrorChildState {
                errors: unique_errors,
                active,
            },
        ));
    }

    rsx! {
        div {
            ..merged,
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

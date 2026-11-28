use std::rc::Rc;

use dioxus::core::{
    AttributeValue, DynamicNode, Event, IntoAttributeValue, ListenerCallback, VNode, VNodeChild,
};
use dioxus::html::PlatformEventData;
use dioxus::prelude::{Attribute, Element, EventHandler, MountedData};

pub use ui_sdk::foundation::compose::*;

/// Lifecycle events whose chained listeners always execute unconditionally,
/// regardless of `prevent_default()`.
const LIFECYCLE_EVENTS: &[&str] = &["onmounted", "onresize", "onvisible"];

/// Configuration controlling invariant attributes and non-cancelable bookkeeping
/// events in [`merge_props_with_config`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MergePropsConfig<'a> {
    /// Un-namespaced attribute names where `base` wins over `user`
    /// (e.g., `["aria-expanded", "aria-controls", "data-state"]`).
    pub invariant_attrs: &'a [&'a str],
    /// Event attribute names (e.g., `"onpointermove"`) where `base` listeners
    /// still execute even if `user` called `evt.prevent_default()`.
    ///
    /// Lifecycle events (`"onmounted"`, `"onresize"`, `"onvisible"`) always run
    /// all listeners unconditionally without needing to be listed here.
    pub bookkeeping_events: &'a [&'a str],
}

fn should_check_default_prevented(name: &str, config: &MergePropsConfig<'_>) -> bool {
    !LIFECYCLE_EVENTS.contains(&name) && !config.bookkeeping_events.contains(&name)
}

/// Chains two erased Dioxus event listeners so `first` executes before `second`.
///
/// When `check_default_prevented` is `true`, `second` is skipped if `first` called
/// `evt.prevent_default()`. When `false`, `second` runs unconditionally.
pub fn chain_event_listeners(
    first: ListenerCallback,
    second: ListenerCallback,
    check_default_prevented: bool,
) -> ListenerCallback {
    ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
        first.call(evt.clone().into_any());
        if !check_default_prevented || evt.default_action_enabled() {
            second.call(evt.into_any());
        }
    })
    .erase()
}

fn fold_listeners(
    listeners: Vec<ListenerCallback>,
    check_default_prevented: bool,
) -> Option<ListenerCallback> {
    listeners
        .into_iter()
        .reduce(|acc, next| chain_event_listeners(acc, next, check_default_prevented))
}

enum MergedSlot {
    Class,
    Style,
    AriaDescribedBy,
    Listener {
        name: &'static str,
        namespace: Option<&'static str>,
        volatile: bool,
        base: Vec<ListenerCallback>,
        user: Vec<ListenerCallback>,
    },
    Scalar {
        attr: Attribute,
        from_base: bool,
    },
}

/// Merges primitive-owned `base` attributes with consumer `user` attributes using
/// default [`MergePropsConfig`].
pub fn merge_props(base: Vec<Attribute>, user: Vec<Attribute>) -> Vec<Attribute> {
    merge_props_with_config(base, user, MergePropsConfig::default())
}

/// Canonical Dioxus 0.8 `Vec<Attribute>` composition layer merging primitive-owned
/// `base` attributes with consumer `user` attributes:
///
/// - Un-namespaced `"class"`: space-joined via [`join_classes`] (`base` first, then `user`).
/// - Un-namespaced `"style"`: semicolon-joined via [`join_styles`] (`base` first, then `user`);
///   preserves `namespace == Some("style")` CSS shorthands.
/// - Un-namespaced `"aria-describedby"`: deduplicated token union via [`merge_aria_describedby`]
///   (`base` first, then `user`, preserving first-seen order).
/// - `AttributeValue::Listener`: duplicate `(name, namespace)` listeners are chained via
///   [`chain_event_listeners`] with `user` running first and `base` running second (preserving
///   relative order within both `user` and `base`).
/// - Scalar attributes (`yieldToAuthor`): `user` overrides `base` unless the un-namespaced name
///   is listed in `config.invariant_attrs` (in which case `base` wins).
/// - `AttributeValue::None` entries are dropped so `user` never strips `base` attributes.
pub fn merge_props_with_config(
    base: Vec<Attribute>,
    user: Vec<Attribute>,
    config: MergePropsConfig<'_>,
) -> Vec<Attribute> {
    if base.is_empty() && user.is_empty() {
        return Vec::new();
    }

    let mut slots: Vec<MergedSlot> = Vec::with_capacity(base.len() + user.len());
    let mut class_parts: Vec<String> = Vec::new();
    let mut style_parts: Vec<String> = Vec::new();
    let mut describedby_parts: Vec<String> = Vec::new();
    let mut has_class_slot = false;
    let mut has_style_slot = false;
    let mut has_describedby_slot = false;

    for attr in base {
        if matches!(attr.value, AttributeValue::None) {
            continue;
        }

        if attr.namespace.is_none() {
            match (attr.name, attr.value) {
                ("class", AttributeValue::Text(text)) => {
                    class_parts.push(text);
                    if !has_class_slot {
                        slots.push(MergedSlot::Class);
                        has_class_slot = true;
                    }
                    continue;
                }
                ("style", AttributeValue::Text(text)) => {
                    style_parts.push(text);
                    if !has_style_slot {
                        slots.push(MergedSlot::Style);
                        has_style_slot = true;
                    }
                    continue;
                }
                ("aria-describedby", AttributeValue::Text(text)) => {
                    describedby_parts.push(text);
                    if !has_describedby_slot {
                        slots.push(MergedSlot::AriaDescribedBy);
                        has_describedby_slot = true;
                    }
                    continue;
                }
                (name, value) => {
                    push_base_slot(
                        &mut slots,
                        Attribute {
                            name,
                            value,
                            namespace: None,
                            volatile: attr.volatile,
                        },
                    );
                    continue;
                }
            }
        }

        push_base_slot(&mut slots, attr);
    }

    for attr in user {
        if matches!(attr.value, AttributeValue::None) {
            continue;
        }

        if attr.namespace.is_none() {
            match (attr.name, attr.value) {
                ("class", AttributeValue::Text(text)) => {
                    class_parts.push(text);
                    if !has_class_slot {
                        slots.push(MergedSlot::Class);
                        has_class_slot = true;
                    }
                    continue;
                }
                ("style", AttributeValue::Text(text)) => {
                    style_parts.push(text);
                    if !has_style_slot {
                        slots.push(MergedSlot::Style);
                        has_style_slot = true;
                    }
                    continue;
                }
                ("aria-describedby", AttributeValue::Text(text)) => {
                    describedby_parts.push(text);
                    if !has_describedby_slot {
                        slots.push(MergedSlot::AriaDescribedBy);
                        has_describedby_slot = true;
                    }
                    continue;
                }
                (name, value) => {
                    push_user_slot(
                        &mut slots,
                        Attribute {
                            name,
                            value,
                            namespace: None,
                            volatile: attr.volatile,
                        },
                        &config,
                    );
                    continue;
                }
            }
        }

        push_user_slot(&mut slots, attr, &config);
    }

    let mut out = Vec::with_capacity(slots.len());
    for slot in slots {
        match slot {
            MergedSlot::Class => {
                if let Some(merged) = join_classes(class_parts.iter().map(String::as_str)) {
                    out.push(Attribute::new("class", merged, None, false));
                }
            }
            MergedSlot::Style => {
                if let Some(merged) = join_styles(style_parts.iter().map(String::as_str)) {
                    out.push(Attribute::new("style", merged, None, false));
                }
            }
            MergedSlot::AriaDescribedBy => {
                if let Some(merged) =
                    merge_aria_describedby(describedby_parts.iter().map(|s| Some(s.as_str())))
                {
                    out.push(Attribute::new("aria-describedby", merged, None, false));
                }
            }
            MergedSlot::Listener {
                name,
                namespace,
                volatile,
                base,
                user,
            } => {
                let check_dp = should_check_default_prevented(name, &config);
                let user_cb = fold_listeners(user, check_dp);
                let base_cb = fold_listeners(base, check_dp);
                let combined = match (user_cb, base_cb) {
                    (Some(u), Some(b)) => Some(chain_event_listeners(u, b, check_dp)),
                    (Some(u), None) => Some(u),
                    (None, Some(b)) => Some(b),
                    (None, None) => None,
                };
                if let Some(listener) = combined {
                    out.push(Attribute {
                        name,
                        value: AttributeValue::Listener(listener),
                        namespace,
                        volatile,
                    });
                }
            }
            MergedSlot::Scalar { attr, .. } => {
                out.push(attr);
            }
        }
    }

    out
}

fn push_base_slot(slots: &mut Vec<MergedSlot>, attr: Attribute) {
    match attr.value {
        AttributeValue::Listener(listener) => {
            if let Some(MergedSlot::Listener { base, volatile, .. }) = slots.iter_mut().find(|s| {
                matches!(
                    s,
                    MergedSlot::Listener {
                        name,
                        namespace,
                        ..
                    } if *name == attr.name && *namespace == attr.namespace
                )
            }) {
                base.push(listener);
                *volatile |= attr.volatile;
            } else {
                slots.push(MergedSlot::Listener {
                    name: attr.name,
                    namespace: attr.namespace,
                    volatile: attr.volatile,
                    base: vec![listener],
                    user: Vec::new(),
                });
            }
        }
        _ => {
            if let Some(MergedSlot::Scalar {
                attr: existing,
                from_base,
            }) = slots.iter_mut().find(|s| {
                matches!(
                    s,
                    MergedSlot::Scalar { attr: existing, .. }
                        if existing.name == attr.name && existing.namespace == attr.namespace
                )
            }) {
                *existing = attr;
                *from_base = true;
            } else {
                slots.push(MergedSlot::Scalar {
                    attr,
                    from_base: true,
                });
            }
        }
    }
}

fn push_user_slot(
    slots: &mut Vec<MergedSlot>,
    attr: Attribute,
    config: &MergePropsConfig<'_>,
) {
    match attr.value {
        AttributeValue::Listener(listener) => {
            if let Some(MergedSlot::Listener { user, volatile, .. }) = slots.iter_mut().find(|s| {
                matches!(
                    s,
                    MergedSlot::Listener {
                        name,
                        namespace,
                        ..
                    } if *name == attr.name && *namespace == attr.namespace
                )
            }) {
                user.push(listener);
                *volatile |= attr.volatile;
            } else {
                slots.push(MergedSlot::Listener {
                    name: attr.name,
                    namespace: attr.namespace,
                    volatile: attr.volatile,
                    base: Vec::new(),
                    user: vec![listener],
                });
            }
        }
        _ => {
            let is_invariant =
                attr.namespace.is_none() && config.invariant_attrs.contains(&attr.name);
            if let Some(MergedSlot::Scalar {
                attr: existing,
                from_base,
            }) = slots.iter_mut().find(|s| {
                matches!(
                    s,
                    MergedSlot::Scalar { attr: existing, .. }
                        if existing.name == attr.name && existing.namespace == attr.namespace
                )
            }) {
                if is_invariant && *from_base {
                    return;
                }
                *existing = attr;
                *from_base = false;
            } else {
                slots.push(MergedSlot::Scalar {
                    attr,
                    from_base: false,
                });
            }
        }
    }
}

/// Attaches an optional typed `EventHandler<Event<Data>>` to an attribute list as an
/// erased `ListenerCallback<PlatformEventData>`, avoiding Dioxus 0.8 `E0282` closure
/// inference friction while composing cleanly through [`merge_props`].
pub fn push_event_handler<Data>(
    attrs: &mut Vec<Attribute>,
    name: &'static str,
    handler: Option<EventHandler<Event<Data>>>,
) where
    Data: for<'a> From<&'a PlatformEventData> + 'static,
{
    if let Some(handler) = handler {
        attrs.push(Attribute::new(
            name,
            AttributeValue::Listener(
                ListenerCallback::<PlatformEventData>::new(move |evt| {
                    handler.call(evt.map(|d| Data::from(d)));
                })
                .erase(),
            ),
            None,
            false,
        ));
    }
}

/// Pushes an optional un-namespaced attribute when `Some` and non-`AttributeValue::None`.
pub fn push_attr_opt<V: IntoAttributeValue<M>, M>(
    attrs: &mut Vec<Attribute>,
    name: &'static str,
    value: Option<V>,
) {
    if let Some(value) = value {
        let attr_value = value.into_value();
        if !matches!(attr_value, AttributeValue::None) {
            attrs.push(Attribute {
                name,
                value: attr_value,
                namespace: None,
                volatile: false,
            });
        }
    }
}

/// Generic payload passed to `child: Option<Callback<ChildProps<S>, Element>>` for
/// stateful primitives that expose runtime state alongside merged DOM `props`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChildProps<S = ()> {
    pub props: Vec<Attribute>,
    pub state: S,
}

impl<S> ChildProps<S> {
    #[inline]
    pub fn new(props: Vec<Attribute>, state: S) -> Self {
        Self { props, state }
    }
}

/// Payload passed to `child: Option<Callback<FloatingContentChildProps, Element>>` on
/// two-level floating `*Content` parts (outer positioner wrapper + inner popup surface),
/// preventing floating `translate` vs CSS animation `transform` collisions (`DIV-01`).
#[derive(Clone, Debug, PartialEq)]
pub struct FloatingContentChildProps {
    pub wrapper_props: Vec<Attribute>,
    pub props: Vec<Attribute>,
}

impl FloatingContentChildProps {
    #[inline]
    pub fn new(wrapper_props: Vec<Attribute>, props: Vec<Attribute>) -> Self {
        Self {
            wrapper_props,
            props,
        }
    }
}

/// Combines optional explicit `class`/`style` props with `#[props(extends = GlobalAttributes)]`
/// into a single consumer attribute list ready for [`merge_props`].
pub fn collect_user_attrs(
    class: Option<String>,
    style: Option<String>,
    mut attributes: Vec<Attribute>,
) -> Vec<Attribute> {
    if class.is_none() && style.is_none() {
        return attributes;
    }
    let mut user = Vec::with_capacity(attributes.len() + 2);
    push_attr_opt(&mut user, "class", class);
    push_attr_opt(&mut user, "style", style);
    user.append(&mut attributes);
    user
}

/// Pushes a boolean `data-*` attribute (`""`) when `present` is `true`, omitting it otherwise.
pub fn push_bool_data_attr(attrs: &mut Vec<Attribute>, name: &'static str, present: bool) {
    if present {
        attrs.push(Attribute::new(name, "", None, false));
    }
}

pub type MountedHandle = Rc<MountedData>;

/// Result of splitting and merging consumer `GlobalAttributes` (`class` / `style`) with
/// primitive-owned base classes and inline styles (`Dioxus #3717`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MergedAttributes {
    pub class: Option<String>,
    pub style: Option<String>,
    pub rest: Vec<Attribute>,
}

/// Zero-overhead merge of primitive-owned `base_class`/`base_style` with forwarded
/// `#[props(extends = GlobalAttributes)] attributes: Vec<Attribute>` (`Dioxus #3717`).
///
/// - Fast path: when `attrs` contains no un-namespaced `"class"` or `"style"` entries, moves
///   `attrs` directly into `rest` with zero reallocation.
/// - Slow path: extracts un-namespaced `"class"` and `"style"` `AttributeValue::Text` entries and
///   merges them via [`join_classes`] / [`join_styles`]; drops `AttributeValue::None` entries so
///   spread never strips primitive-owned styles; preserves `namespace == Some("style")` CSS
///   shorthands and all other attributes in `rest`.
pub fn merge_global_attributes(
    base_class: Option<&str>,
    base_style: Option<&str>,
    attrs: Vec<Attribute>,
) -> MergedAttributes {
    let has_class_or_style = attrs
        .iter()
        .any(|attr| attr.namespace.is_none() && matches!(attr.name, "class" | "style"));

    if !has_class_or_style {
        return MergedAttributes {
            class: base_class.and_then(|c| join_classes([c])),
            style: base_style.and_then(|s| join_styles([s])),
            rest: attrs,
        };
    }

    let mut class_parts: Vec<String> = Vec::new();
    if let Some(base) = base_class {
        class_parts.push(base.to_owned());
    }
    let mut style_parts: Vec<String> = Vec::new();
    if let Some(base) = base_style {
        style_parts.push(base.to_owned());
    }

    let mut rest = Vec::with_capacity(attrs.len());
    for attr in attrs {
        if attr.namespace.is_none() && attr.name == "class" {
            match attr.value {
                AttributeValue::Text(text) => class_parts.push(text),
                AttributeValue::None => {}
                other => rest.push(Attribute {
                    value: other,
                    ..attr
                }),
            }
        } else if attr.namespace.is_none() && attr.name == "style" {
            match attr.value {
                AttributeValue::Text(text) => style_parts.push(text),
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

    MergedAttributes {
        class: join_classes(class_parts.iter().map(String::as_str)),
        style: join_styles(style_parts.iter().map(String::as_str)),
        rest,
    }
}

/// Prepends canonical `data-monoxus-<variant>-<part>=""` attribute ahead of forwarded user attributes.
pub fn merge_attributes_with_part(
    custom: Vec<Attribute>,
    part_attr: &'static str,
) -> Vec<Attribute> {
    let mut merged = Vec::with_capacity(custom.len() + 1);
    merged.push(Attribute::new(part_attr, "", None, false));
    merged.extend(custom);
    merged
}

/// Extracted `aria-invalid` and `aria-describedby` values from forwarded `GlobalAttributes`,
/// with the remaining non-extracted attributes preserved in `rest`.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub(crate) struct ExtractedAriaAttrs {
    pub aria_invalid: Option<bool>,
    pub aria_describedby: Option<String>,
    pub rest: Vec<Attribute>,
}

/// Merges multiple optional whitespace-separated `aria-describedby` strings into a single
/// space-separated deduplicated token string preserving first-seen order.
/// Returns `None` when no non-empty tokens are present.
pub fn merge_aria_describedby<'a>(
    parts: impl IntoIterator<Item = Option<&'a str>>,
) -> Option<String> {
    let mut tokens: Vec<&'a str> = Vec::new();
    for part in parts.into_iter().flatten() {
        for token in part.split_ascii_whitespace() {
            if !token.is_empty() && !tokens.contains(&token) {
                tokens.push(token);
            }
        }
    }
    (!tokens.is_empty()).then(|| tokens.join(" "))
}

/// Extracts un-namespaced `aria-invalid` and `aria-describedby` entries from forwarded
/// `GlobalAttributes` so primitives can apply unified precedence (`explicit prop > GlobalAttributes
/// > FieldControlContext/FieldContext > control default`) and merge `aria-describedby` tokens
/// without emitting duplicate HTML attributes.
#[allow(dead_code)]
pub(crate) fn extract_aria_invalid_and_describedby(attrs: Vec<Attribute>) -> ExtractedAriaAttrs {
    let has_target = attrs
        .iter()
        .any(|attr| attr.namespace.is_none() && matches!(attr.name, "aria-invalid" | "aria-describedby"));

    if !has_target {
        return ExtractedAriaAttrs {
            aria_invalid: None,
            aria_describedby: None,
            rest: attrs,
        };
    }

    let mut aria_invalid: Option<bool> = None;
    let mut describedby_parts: Vec<String> = Vec::new();
    let mut rest = Vec::with_capacity(attrs.len());

    for attr in attrs {
        if attr.namespace.is_none() && attr.name == "aria-invalid" {
            match attr.value {
                AttributeValue::Bool(flag) => aria_invalid = Some(flag),
                AttributeValue::Text(text) => {
                    let trimmed = text.trim();
                    if trimmed.eq_ignore_ascii_case("false") {
                        aria_invalid = Some(false);
                    } else if !trimmed.is_empty() {
                        aria_invalid = Some(true);
                    }
                }
                AttributeValue::None => {}
                other => rest.push(Attribute {
                    value: other,
                    ..attr
                }),
            }
        } else if attr.namespace.is_none() && attr.name == "aria-describedby" {
            match attr.value {
                AttributeValue::Text(text) => describedby_parts.push(text),
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

    let aria_describedby = merge_aria_describedby(
        describedby_parts
            .iter()
            .map(|part| Some(part.as_str())),
    );

    ExtractedAriaAttrs {
        aria_invalid,
        aria_describedby,
        rest,
    }
}

/// Extracts un-namespaced `aria-label` and `aria-labelledby` entries from forwarded
/// `GlobalAttributes` so widgets with computed label precedence (e.g., `SliderThumb`,
/// `RatingGroupRoot`) can honor author-supplied labels without declaring duplicate props.
#[allow(dead_code)]
pub(crate) fn extract_aria_label_and_labelledby(
    attrs: Vec<Attribute>,
) -> (Option<String>, Option<String>, Vec<Attribute>) {
    let has_target = attrs.iter().any(|attr| {
        attr.namespace.is_none() && matches!(attr.name, "aria-label" | "aria-labelledby")
    });
    if !has_target {
        return (None, None, attrs);
    }

    let mut aria_label: Option<String> = None;
    let mut aria_labelledby: Option<String> = None;
    let mut rest = Vec::with_capacity(attrs.len());

    for attr in attrs {
        if attr.namespace.is_none() && attr.name == "aria-label" {
            match attr.value {
                AttributeValue::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        aria_label = Some(trimmed.to_owned());
                    }
                }
                AttributeValue::None => {}
                other => rest.push(Attribute {
                    value: other,
                    ..attr
                }),
            }
        } else if attr.namespace.is_none() && attr.name == "aria-labelledby" {
            match attr.value {
                AttributeValue::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        aria_labelledby = Some(trimmed.to_owned());
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

    (aria_label, aria_labelledby, rest)
}

/// Applies the unified form-control ARIA precedence:
/// - `invalid`: `explicit_invalid > attr_invalid > field_invalid > default_invalid`
/// - `aria-describedby`: deduplicated space-separated union of `[field_describedby, attr_describedby, explicit_describedby]`
#[allow(dead_code)]
pub(crate) fn resolve_invalid_and_describedby(
    explicit_invalid: Option<bool>,
    attr_invalid: Option<bool>,
    field_invalid: Option<bool>,
    default_invalid: bool,
    explicit_describedby: Option<&str>,
    attr_describedby: Option<&str>,
    field_describedby: Option<&str>,
) -> (bool, Option<String>) {
    let invalid = explicit_invalid
        .or(attr_invalid)
        .or(field_invalid)
        .unwrap_or(default_invalid);
    let describedby = merge_aria_describedby([
        field_describedby,
        attr_describedby,
        explicit_describedby,
    ]);
    (invalid, describedby)
}

/// Synchronous fallback text extractor walking static/dynamic VNode children when `text_value` is omitted.
pub fn extract_vnode_text(children: &Element) -> String {
    let Ok(vnode) = children else {
        return String::new();
    };
    let mut buffer = String::new();
    collect_vnode_text(vnode, &mut buffer);
    buffer.trim().to_owned()
}

fn collect_vnode_text(vnode: &VNode, out: &mut String) {
    for child in vnode.children() {
        collect_child_text(child, out);
    }
}

fn collect_child_text(child: VNodeChild<'_>, out: &mut String) {
    match child {
        VNodeChild::Text(static_text) => {
            out.push_str(static_text.text());
        }
        VNodeChild::Element(static_el) => {
            for grand in static_el.children() {
                collect_child_text(grand, out);
            }
        }
        VNodeChild::Dynamic(anchor) => {
            for slot in anchor.nodes() {
                match &*slot {
                    DynamicNode::Text(vtext) => {
                        out.push_str(&vtext.value);
                    }
                    DynamicNode::Fragment(nodes) => {
                        for nested in nodes {
                            collect_vnode_text(nested, out);
                        }
                    }
                    DynamicNode::Component(_) => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use dioxus::core::AttributeValue;
    use dioxus::prelude::Attribute;

    use super::{merge_attributes_with_part, merge_global_attributes};

    #[test]
    fn merge_global_attributes_fast_path_preserves_rest_and_normalizes_base() {
        let attrs = vec![
            Attribute::new("data-testid", "trigger", None, false),
            Attribute::new("color", "blue", Some("style"), false),
        ];
        let ptr_before = attrs.as_ptr();
        let merged = merge_global_attributes(Some("  base-cls  "), Some(" --x: 1px; "), attrs);
        assert_eq!(merged.class.as_deref(), Some("base-cls"));
        assert_eq!(merged.style.as_deref(), Some("--x: 1px;"));
        assert_eq!(merged.rest.len(), 2);
        assert_eq!(merged.rest.as_ptr(), ptr_before);
    }

    #[test]
    fn merge_global_attributes_merges_class_style_drops_none_and_keeps_style_shorthands() {
        let attrs = vec![
            Attribute::new("class", "a", None, false),
            Attribute::new("style", "color: red", None, false),
            Attribute {
                name: "style",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
            Attribute {
                name: "class",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
            Attribute::new("background", "green", Some("style"), false),
            Attribute::new("data-extra", "yes", None, false),
        ];
        let merged =
            merge_global_attributes(Some("b"), Some("--monoxus-x: 4px; margin: 0;"), attrs);

        assert_eq!(merged.class.as_deref(), Some("b a"));
        assert_eq!(
            merged.style.as_deref(),
            Some("--monoxus-x: 4px; margin: 0; color: red")
        );
        assert!(!merged.style.as_deref().unwrap().contains(";;"));
        assert_eq!(merged.rest.len(), 2);
        assert_eq!(merged.rest[0].name, "background");
        assert_eq!(merged.rest[0].namespace, Some("style"));
        assert_eq!(merged.rest[1].name, "data-extra");
    }

    #[test]
    fn merge_attributes_with_part_prepends_part_marker() {
        let attrs = vec![Attribute::new("data-user", "1", None, false)];
        let merged = merge_attributes_with_part(attrs, "data-monoxus-menu-item");
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].name, "data-monoxus-menu-item");
        assert_eq!(merged[1].name, "data-user");
    }

    #[test]
    fn extract_aria_invalid_and_describedby_fast_path_moves_without_reallocation() {
        use super::extract_aria_invalid_and_describedby;

        let attrs = vec![Attribute::new("data-testid", "inp", None, false)];
        let ptr_before = attrs.as_ptr();
        let extracted = extract_aria_invalid_and_describedby(attrs);
        assert_eq!(extracted.aria_invalid, None);
        assert_eq!(extracted.aria_describedby, None);
        assert_eq!(extracted.rest.len(), 1);
        assert_eq!(extracted.rest.as_ptr(), ptr_before);
    }

    #[test]
    fn extract_aria_invalid_and_describedby_parses_merges_and_dedupes() {
        use super::{
            extract_aria_invalid_and_describedby, merge_aria_describedby,
            resolve_invalid_and_describedby,
        };

        let attrs = vec![
            Attribute::new("aria-invalid", "true", None, false),
            Attribute::new("aria-describedby", "desc-a  desc-b", None, false),
            Attribute::new("aria-describedby", "desc-b desc-c", None, false),
            Attribute {
                name: "aria-describedby",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
            Attribute::new("data-keep", "1", None, false),
        ];

        let extracted = extract_aria_invalid_and_describedby(attrs);
        assert_eq!(extracted.aria_invalid, Some(true));
        assert_eq!(
            extracted.aria_describedby.as_deref(),
            Some("desc-a desc-b desc-c")
        );
        assert_eq!(extracted.rest.len(), 1);
        assert_eq!(extracted.rest[0].name, "data-keep");

        // False text & bool values
        let false_attrs = vec![
            Attribute::new("aria-invalid", "false", None, false),
            Attribute::new("aria-describedby", "   ", None, false),
        ];
        let extracted_false = extract_aria_invalid_and_describedby(false_attrs);
        assert_eq!(extracted_false.aria_invalid, Some(false));
        assert_eq!(extracted_false.aria_describedby, None);

        let bool_attrs = vec![Attribute::new("aria-invalid", true, None, false)];
        assert_eq!(
            extract_aria_invalid_and_describedby(bool_attrs).aria_invalid,
            Some(true)
        );

        // Precedence & deduped merge
        assert_eq!(
            merge_aria_describedby([Some("f1 f2"), None, Some("f2 a1"), Some("a1 e1")]).as_deref(),
            Some("f1 f2 a1 e1")
        );
        assert_eq!(merge_aria_describedby([None, Some("   ")]), None);

        let (inv, desc) = resolve_invalid_and_describedby(
            Some(false),
            Some(true),
            Some(true),
            true,
            Some("ext-1 desc-1"),
            Some("attr-1"),
            Some("desc-1 err-1"),
        );
        assert!(!inv);
        assert_eq!(desc.as_deref(), Some("desc-1 err-1 attr-1 ext-1"));

        let (inv2, _) = resolve_invalid_and_describedby(
            None,
            None,
            Some(true),
            false,
            None,
            None,
            None,
        );
        assert!(inv2);
    }

    fn find_attr<'a>(
        attrs: &'a [Attribute],
        name: &str,
        namespace: Option<&str>,
    ) -> Option<&'a Attribute> {
        attrs
            .iter()
            .find(|attr| attr.name == name && attr.namespace == namespace)
    }

    fn attr_text<'a>(
        attrs: &'a [Attribute],
        name: &str,
        namespace: Option<&str>,
    ) -> Option<&'a str> {
        match &find_attr(attrs, name, namespace)?.value {
            AttributeValue::Text(text) => Some(text.as_str()),
            _ => None,
        }
    }

    #[test]
    fn merge_props_merges_class_style_drops_none_and_preserves_style_shorthands() {
        use super::merge_props;

        let base = vec![
            Attribute::new("class", "  base-cls ", None, false),
            Attribute::new("style", "--monoxus-x: 4px; margin: 0;", None, false),
            Attribute::new("background", "blue", Some("style"), false),
            Attribute {
                name: "data-drop-base",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
        ];
        let user = vec![
            Attribute::new("class", "user-cls", None, false),
            Attribute {
                name: "class",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
            Attribute::new("style", "color: red", None, false),
            Attribute {
                name: "style",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
            Attribute::new("background", "green", Some("style"), false),
            Attribute::new("padding", "8px", Some("style"), false),
        ];

        let merged = merge_props(base, user);

        assert_eq!(attr_text(&merged, "class", None), Some("base-cls user-cls"));
        let style = attr_text(&merged, "style", None).unwrap();
        assert_eq!(style, "--monoxus-x: 4px; margin: 0; color: red");
        assert!(!style.contains(";;"));
        assert_eq!(
            attr_text(&merged, "background", Some("style")),
            Some("green")
        );
        assert_eq!(attr_text(&merged, "padding", Some("style")), Some("8px"));
        assert!(find_attr(&merged, "data-drop-base", None).is_none());

        // Whitespace-only class/style/aria-describedby do not emit empty attributes.
        let blank = merge_props(
            vec![
                Attribute::new("class", "   ", None, false),
                Attribute::new("style", " ; ", None, false),
                Attribute::new("aria-describedby", "  \t ", None, false),
            ],
            Vec::new(),
        );
        assert!(blank.is_empty());
    }

    #[test]
    fn merge_props_merges_aria_describedby_deduplicated_in_order() {
        use super::merge_props;

        let base = vec![
            Attribute::new("aria-describedby", "field-desc  tooltip-id", None, false),
        ];
        let user = vec![
            Attribute::new(
                "aria-describedby",
                "tooltip-id user-desc field-desc",
                None,
                false,
            ),
            Attribute {
                name: "aria-describedby",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
        ];

        let merged = merge_props(base, user);
        assert_eq!(
            merged
                .iter()
                .filter(|a| a.namespace.is_none() && a.name == "aria-describedby")
                .count(),
            1
        );
        assert_eq!(
            attr_text(&merged, "aria-describedby", None),
            Some("field-desc tooltip-id user-desc")
        );
    }

    #[test]
    fn merge_props_yield_to_author_and_invariant_attrs_protection() {
        use super::{MergePropsConfig, merge_props, merge_props_with_config};

        let base = vec![
            Attribute::new("id", "base-id", None, false),
            Attribute::new("role", "button", None, false),
            Attribute::new("aria-expanded", "true", None, false),
            Attribute::new("data-state", "open", None, false),
        ];
        let user = vec![
            Attribute::new("id", "user-id", None, false),
            Attribute {
                name: "role",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
            Attribute::new("aria-expanded", "false", None, false),
            Attribute::new("data-state", "closed", None, false),
            Attribute::new("aria-controls", "user-controls", None, false),
            Attribute::new("data-extra", "yes", None, false),
        ];

        let protected = merge_props_with_config(
            base.clone(),
            user.clone(),
            MergePropsConfig {
                invariant_attrs: &["aria-expanded", "aria-controls", "data-state"],
                bookkeeping_events: &[],
            },
        );
        assert_eq!(attr_text(&protected, "id", None), Some("user-id"));
        assert_eq!(attr_text(&protected, "role", None), Some("button"));
        assert_eq!(attr_text(&protected, "aria-expanded", None), Some("true"));
        assert_eq!(attr_text(&protected, "data-state", None), Some("open"));
        assert_eq!(
            attr_text(&protected, "aria-controls", None),
            Some("user-controls")
        );
        assert_eq!(attr_text(&protected, "data-extra", None), Some("yes"));

        // Default config uses yieldToAuthor for all scalar attributes.
        let yielded = merge_props(base, user);
        assert_eq!(attr_text(&yielded, "aria-expanded", None), Some("false"));
        assert_eq!(attr_text(&yielded, "data-state", None), Some("closed"));
        assert_eq!(attr_text(&yielded, "role", None), Some("button"));
    }

    #[test]
    fn merge_props_handles_empty_base_and_empty_user() {
        use super::merge_props;

        assert!(merge_props(Vec::new(), Vec::new()).is_empty());

        let only_user = merge_props(
            Vec::new(),
            vec![
                Attribute::new("class", "  a  ", None, false),
                Attribute {
                    name: "title",
                    value: AttributeValue::None,
                    namespace: None,
                    volatile: false,
                },
                Attribute::new("data-u", "1", None, false),
            ],
        );
        assert_eq!(only_user.len(), 2);
        assert_eq!(attr_text(&only_user, "class", None), Some("a"));
        assert_eq!(attr_text(&only_user, "data-u", None), Some("1"));

        let only_base = merge_props(
            vec![
                Attribute::new("style", "color: red;", None, false),
                Attribute {
                    name: "id",
                    value: AttributeValue::None,
                    namespace: None,
                    volatile: false,
                },
                Attribute::new("data-b", "2", None, false),
            ],
            Vec::new(),
        );
        assert_eq!(only_base.len(), 2);
        assert_eq!(attr_text(&only_base, "style", None), Some("color: red;"));
        assert_eq!(attr_text(&only_base, "data-b", None), Some("2"));
    }

    #[test]
    fn push_attr_opt_and_push_bool_data_attr_emit_expected_attributes() {
        use super::{push_attr_opt, push_bool_data_attr};

        let mut attrs = Vec::new();
        push_attr_opt(&mut attrs, "id", Some("trigger-1"));
        push_attr_opt(&mut attrs, "title", Option::<&str>::None);
        push_attr_opt(&mut attrs, "data-none", Some(AttributeValue::None));
        push_bool_data_attr(&mut attrs, "data-disabled", true);
        push_bool_data_attr(&mut attrs, "data-readonly", false);

        assert_eq!(attrs.len(), 2);
        assert_eq!(attr_text(&attrs, "id", None), Some("trigger-1"));
        assert_eq!(attr_text(&attrs, "data-disabled", None), Some(""));
        assert!(find_attr(&attrs, "title", None).is_none());
        assert!(find_attr(&attrs, "data-none", None).is_none());
        assert!(find_attr(&attrs, "data-readonly", None).is_none());
    }

    #[test]
    fn should_check_default_prevented_respects_lifecycle_and_bookkeeping_events() {
        use super::{MergePropsConfig, should_check_default_prevented};

        let default_cfg = MergePropsConfig::default();
        assert!(!should_check_default_prevented("onmounted", &default_cfg));
        assert!(!should_check_default_prevented("onresize", &default_cfg));
        assert!(!should_check_default_prevented("onvisible", &default_cfg));
        assert!(should_check_default_prevented("onclick", &default_cfg));
        assert!(should_check_default_prevented("onpointermove", &default_cfg));

        let custom_cfg = MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &["onpointermove"],
        };
        assert!(!should_check_default_prevented("onpointermove", &custom_cfg));
        assert!(should_check_default_prevented("onclick", &custom_cfg));
    }
}

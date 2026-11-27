use std::rc::Rc;

use dioxus::core::{DynamicNode, VNode, VNodeChild};
use dioxus::core::AttributeValue;
use dioxus::prelude::{Attribute, Element, MountedData};

pub use monoxus_core::foundation::compose::*;

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
#[allow(dead_code)]
pub(crate) fn merge_aria_describedby<'a>(
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
}

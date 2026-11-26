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
}

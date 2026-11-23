use std::cell::Cell;
use std::rc::Rc;

use dioxus::core::{DynamicNode, VNode, VNodeChild};
use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

pub use monoxus_core::menu::types::*;

/// Select event passed to `Item`, `CheckboxItem`, `RadioItem`, and `SubTrigger` `on_select` handlers (`M-10`, `3B D-6`).
#[derive(Clone, Debug)]
pub struct MenuSelectEvent {
    default_prevented: Rc<Cell<bool>>,
}

impl Default for MenuSelectEvent {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuSelectEvent {
    pub fn new() -> Self {
        Self {
            default_prevented: Rc::new(Cell::new(false)),
        }
    }

    pub fn prevent_default(&self) {
        self.default_prevented.set(true);
    }

    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented.get()
    }
}

/// Preventable outside-interaction event passed to `on_pointer_down_outside` / `on_focus_outside` / `on_interact_outside`.
#[derive(Clone, Debug)]
pub struct MenuOutsideEvent {
    default_prevented: Rc<Cell<bool>>,
    target_id: Option<String>,
}

impl MenuOutsideEvent {
    pub fn new(target_id: Option<String>) -> Self {
        Self {
            default_prevented: Rc::new(Cell::new(false)),
            target_id,
        }
    }

    pub fn target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }

    pub fn prevent_default(&self) {
        self.default_prevented.set(true);
    }

    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented.get()
    }
}

/// Preventable event passed to `on_escape_keydown`, `on_open_auto_focus`, and `on_close_auto_focus`.
#[derive(Clone, Debug, Default)]
pub struct MenuPreventableEvent {
    default_prevented: Rc<Cell<bool>>,
}

impl MenuPreventableEvent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prevent_default(&self) {
        self.default_prevented.set(true);
    }

    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented.get()
    }
}

/// Prepend canonical `data-monoxus-<variant>-<part>=""` attribute ahead of forwarded user attributes.
pub fn merge_attributes_with_part(
    custom: Vec<Attribute>,
    part_attr: &'static str,
) -> Vec<Attribute> {
    let mut merged = Vec::with_capacity(custom.len() + 1);
    merged.push(Attribute::new(part_attr, "", None, false));
    merged.extend(custom);
    merged
}

/// Translate Dioxus `keyboard_types::Key` into host-neutral [`MenuKey`].
pub fn translate_menu_key(key: &Key) -> Option<MenuKey> {
    match key {
        Key::Named(NamedKey::ArrowDown) => Some(MenuKey::ArrowDown),
        Key::Named(NamedKey::ArrowUp) => Some(MenuKey::ArrowUp),
        Key::Named(NamedKey::ArrowRight) => Some(MenuKey::ArrowRight),
        Key::Named(NamedKey::ArrowLeft) => Some(MenuKey::ArrowLeft),
        Key::Named(NamedKey::Home) => Some(MenuKey::Home),
        Key::Named(NamedKey::End) => Some(MenuKey::End),
        Key::Named(NamedKey::PageUp) => Some(MenuKey::PageUp),
        Key::Named(NamedKey::PageDown) => Some(MenuKey::PageDown),
        Key::Named(NamedKey::Enter) => Some(MenuKey::Enter),
        Key::Named(NamedKey::Escape) => Some(MenuKey::Escape),
        Key::Named(NamedKey::Tab) => Some(MenuKey::Tab),
        Key::Named(NamedKey::Backspace) => Some(MenuKey::Backspace),
        Key::Character(s) => {
            if s == " " {
                Some(MenuKey::Space)
            } else {
                let mut chars = s.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) if !c.is_control() => Some(MenuKey::Char(c)),
                    _ => None,
                }
            }
        }
        _ => None,
    }
}

/// Synchronous fallback text extractor walking static/dynamic VNode children when `text_value` is omitted (`M-13`).
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

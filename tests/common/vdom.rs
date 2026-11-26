//! Shared headless Dioxus VirtualDom test helpers.

#![allow(dead_code)]

use dioxus::prelude::{Element, VirtualDom};

/// Mounts a zero-prop Dioxus component into a headless `VirtualDom`, performs the initial
/// build, and flushes immediate mutations.
pub fn mount_vdom(app: fn() -> Element) -> VirtualDom {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom
}

/// Mounts a Dioxus component with explicit props into a headless `VirtualDom`, performs the
/// initial build, and flushes immediate mutations.
pub fn mount_with_props<P>(app: fn(P) -> Element, props: P) -> VirtualDom
where
    P: Clone + 'static,
{
    let mut dom = VirtualDom::new_with_props(app, props);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom
}

/// Flushes pending immediate renders on an existing `VirtualDom`.
pub fn flush_vdom(dom: &mut VirtualDom) {
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
}

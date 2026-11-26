pub(crate) mod arrow;
pub mod dismiss;
pub mod floating;
pub mod focus;
pub mod grace;
pub mod portal;
pub mod presence;
pub(crate) mod watcher;

#[allow(unused_imports)]
pub(crate) use self::arrow::*;
pub use self::dismiss::*;
pub use self::floating::*;
pub use self::focus::*;
pub use self::grace::*;
pub use self::portal::*;
pub use self::presence::*;
#[allow(unused_imports)]
pub(crate) use self::watcher::*;

#[doc(hidden)]
pub use crate::foundation::browser::{
    PortalOp, start_portal_op_recording, stop_portal_op_recording, take_portal_ops,
};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

thread_local! {
    static ACTIVE_OVERLAY_LAYERS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static OVERLAY_ESCAPE_CONSUMED_GENERATION: Cell<u64> = const { Cell::new(0) };
    static OVERLAY_ESCAPE_CONSUMED: Cell<bool> = const { Cell::new(false) };
}

pub fn mark_overlay_escape_consumed() {
    let generation = OVERLAY_ESCAPE_CONSUMED_GENERATION.with(|g| {
        let next = g.get().wrapping_add(1);
        g.set(next);
        next
    });
    OVERLAY_ESCAPE_CONSUMED.with(|flag| flag.set(true));
    crate::foundation::browser::spawn_root_task(async move {
        let _ = oxidase::frame::next_frame().await;
        OVERLAY_ESCAPE_CONSUMED_GENERATION.with(|g| {
            if g.get() == generation {
                OVERLAY_ESCAPE_CONSUMED.with(|flag| flag.set(false));
            }
        });
    });
}

pub fn is_overlay_escape_consumed() -> bool {
    OVERLAY_ESCAPE_CONSUMED.with(|flag| flag.get())
}

pub fn register_active_overlay_layer(layer_id: &str) {
    if layer_id.is_empty() {
        return;
    }
    ACTIVE_OVERLAY_LAYERS.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.retain(|existing| existing != layer_id);
        stack.push(layer_id.to_owned());
    });
}

/// Ensures `layer_id` is registered on `ACTIVE_OVERLAY_LAYERS` without reordering if already present (`bits#2143`).
pub fn ensure_active_overlay_layer(layer_id: &str) {
    if layer_id.is_empty() {
        return;
    }
    ACTIVE_OVERLAY_LAYERS.with(|stack| {
        let mut stack = stack.borrow_mut();
        if !stack.iter().any(|existing| existing == layer_id) {
            stack.push(layer_id.to_owned());
        }
    });
}

/// Replaces `old_id` with `new_id` in place on `ACTIVE_OVERLAY_LAYERS` without disturbing LIFO
/// stack order. If `old_id` is not present, pushes `new_id` unless already registered.
pub fn replace_active_overlay_layer(old_id: &str, new_id: &str) {
    if new_id.is_empty() {
        if !old_id.is_empty() {
            unregister_active_overlay_layer(old_id);
        }
        return;
    }
    if old_id == new_id {
        ensure_active_overlay_layer(new_id);
        return;
    }
    ACTIVE_OVERLAY_LAYERS.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(pos) = stack.iter().position(|existing| existing == old_id) {
            if stack.iter().any(|existing| existing == new_id) {
                stack.remove(pos);
            } else {
                stack[pos] = new_id.to_owned();
            }
        } else if !stack.iter().any(|existing| existing == new_id) {
            stack.push(new_id.to_owned());
        }
    });
}

pub fn unregister_active_overlay_layer(layer_id: &str) {
    ACTIVE_OVERLAY_LAYERS.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.retain(|existing| existing != layer_id);
    });
}

pub fn is_topmost_overlay_layer(layer_id: &str) -> bool {
    ACTIVE_OVERLAY_LAYERS.with(|stack| {
        let stack = stack.borrow();
        match stack.last() {
            None => true,
            Some(top) => top == layer_id,
        }
    })
}

pub fn active_overlay_layers_snapshot() -> Vec<String> {
    ACTIVE_OVERLAY_LAYERS.with(|stack| stack.borrow().clone())
}

pub type OverlayEscapeGuardFn = Rc<RefCell<dyn FnMut() -> bool>>;

#[derive(Clone)]
pub struct OverlayEscapeGuardContext {
    register: Rc<RefCell<dyn FnMut(String, OverlayEscapeGuardFn)>>,
    unregister: Rc<RefCell<dyn FnMut(String)>>,
}

impl PartialEq for OverlayEscapeGuardContext {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.register, &other.register)
            && Rc::ptr_eq(&self.unregister, &other.unregister)
    }
}

impl OverlayEscapeGuardContext {
    pub fn new(
        register_guard: impl FnMut(String, OverlayEscapeGuardFn) + 'static,
        unregister_guard: impl FnMut(String) + 'static,
    ) -> Self {
        Self {
            register: Rc::new(RefCell::new(register_guard)),
            unregister: Rc::new(RefCell::new(unregister_guard)),
        }
    }

    pub fn register_guard(&self, id: impl Into<String>, guard: impl FnMut() -> bool + 'static) {
        if let Ok(mut cb) = self.register.try_borrow_mut() {
            cb(id.into(), Rc::new(RefCell::new(guard)));
        }
    }

    pub fn unregister_guard(&self, id: impl Into<String>) {
        if let Ok(mut cb) = self.unregister.try_borrow_mut() {
            cb(id.into());
        }
    }
}

#[derive(Clone)]
pub struct OverlayBranchContext {
    register: Rc<RefCell<dyn FnMut(String)>>,
    unregister: Rc<RefCell<dyn FnMut(String)>>,
}

impl PartialEq for OverlayBranchContext {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.register, &other.register)
            && Rc::ptr_eq(&self.unregister, &other.unregister)
    }
}

impl OverlayBranchContext {
    pub fn new(
        register_branch: impl FnMut(String) + 'static,
        unregister_branch: impl FnMut(String) + 'static,
    ) -> Self {
        Self {
            register: Rc::new(RefCell::new(register_branch)),
            unregister: Rc::new(RefCell::new(unregister_branch)),
        }
    }

    pub fn register_branch(&self, branch: impl Into<String>) {
        if let Ok(mut cb) = self.register.try_borrow_mut() {
            cb(branch.into());
        }
    }

    pub fn unregister_branch(&self, branch: impl Into<String>) {
        if let Ok(mut cb) = self.unregister.try_borrow_mut() {
            cb(branch.into());
        }
    }
}

/// Shared portal teleportation & drop cleanup hook for all `*Portal` components (`PRT-1`..`PRT-7`).
pub(crate) fn use_portal_teleport(
    pid: String,
    should_render: bool,
    disabled: bool,
    resolved_host: &PortalHost,
) {
    use dioxus::prelude::*;

    let state: Rc<RefCell<(String, bool)>> =
        use_hook(|| Rc::new(RefCell::new((pid.clone(), false))));

    {
        let state_effect = Rc::clone(&state);
        use_effect(use_reactive(
            (&should_render, &disabled, &pid, resolved_host),
            move |(render, is_disabled, cur_pid, host)| {
                let mut tracked = state_effect.borrow_mut();
                let should_teleport = render && !is_disabled && !host.is_inline();
                if should_teleport {
                    if tracked.1 && tracked.0 != cur_pid {
                        crate::foundation::browser::remove_element_by_id(&tracked.0);
                    }
                    let host_id = match &host {
                        PortalHost::Default => None,
                        PortalHost::Named(name) => Some(name.as_ref()),
                        PortalHost::Inline => None,
                    };
                    crate::foundation::browser::teleport_element_to_host(&cur_pid, host_id);
                    tracked.0 = cur_pid;
                    tracked.1 = true;
                } else {
                    if tracked.1 && tracked.0 != cur_pid {
                        crate::foundation::browser::remove_element_by_id(&tracked.0);
                    }
                    tracked.0 = cur_pid;
                    tracked.1 = false;
                }
            },
        ));
    }

    {
        let state_drop = Rc::clone(&state);
        use_drop(move || {
            let (cur_pid, was_teleported) = state_drop.borrow().clone();
            if was_teleported {
                crate::foundation::browser::remove_element_by_id(&cur_pid);
            }
        });
    }
}

/// Registers a set of DOM element IDs (e.g. `[content_id, wrapper_id, portal_id]`) into an
/// enclosing [`OverlayBranchContext`] while `active` is `true`, cleaning up stale IDs on
/// ID change, close (`active == false`), and unmount (`use_drop`).
pub(crate) fn use_register_parent_overlay_branches(
    parent_branch: Option<OverlayBranchContext>,
    active: bool,
    ids: Vec<String>,
) {
    use dioxus::prelude::*;

    let tracked: Rc<RefCell<Vec<String>>> = use_hook(|| Rc::new(RefCell::new(Vec::new())));

    if let Some(ref bctx) = parent_branch {
        let mut prev = tracked.borrow_mut();
        if !active {
            for old in prev.drain(..) {
                bctx.unregister_branch(old);
            }
        } else {
            let stale: Vec<String> = prev
                .iter()
                .filter(|old| !ids.iter().any(|n| n == *old))
                .cloned()
                .collect();
            for old in &stale {
                bctx.unregister_branch(old.clone());
            }
            prev.retain(|old| !stale.contains(old));

            for id in &ids {
                if !id.is_empty() && !prev.contains(id) {
                    bctx.register_branch(id.clone());
                    prev.push(id.clone());
                }
            }
        }
    } else {
        tracked.borrow_mut().clear();
    }

    let drop_tracked = Rc::clone(&tracked);
    let drop_branch = parent_branch;
    use_drop(move || {
        if let Some(ref bctx) = drop_branch {
            for old in drop_tracked.borrow_mut().drain(..) {
                bctx.unregister_branch(old);
            }
        }
    });
}

#[cfg(test)]
mod tests;

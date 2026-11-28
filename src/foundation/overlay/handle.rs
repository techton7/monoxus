use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt,
    marker::PhantomData,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

static NEXT_PAYLOAD_ID: AtomicU64 = AtomicU64::new(1);

/// Marker trait describing an overlay family for [`OverlayHandle`] and [`OverlayHandleProp`].
pub trait OverlayHandleKind: Copy + 'static {
    /// Human-readable overlay component family name used in dev diagnostics.
    const KIND_NAME: &'static str;
    /// Whether `open(None)` / `open_with_payload(...)` without an anchor trigger is valid.
    const ALLOWS_TRIGGERLESS_OPEN: bool;
}

/// Marker kind for `Dialog` (and `AlertDialog`) handles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DialogKind;

impl OverlayHandleKind for DialogKind {
    const KIND_NAME: &'static str = "Dialog";
    const ALLOWS_TRIGGERLESS_OPEN: bool = true;
}

/// Marker kind for `AlertDialog` handles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AlertDialogKind;

impl OverlayHandleKind for AlertDialogKind {
    const KIND_NAME: &'static str = "AlertDialog";
    const ALLOWS_TRIGGERLESS_OPEN: bool = true;
}

/// Marker kind for `Popover` handles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PopoverKind;

impl OverlayHandleKind for PopoverKind {
    const KIND_NAME: &'static str = "Popover";
    const ALLOWS_TRIGGERLESS_OPEN: bool = false;
}

/// Marker kind for `Tooltip` handles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TooltipKind;

impl OverlayHandleKind for TooltipKind {
    const KIND_NAME: &'static str = "Tooltip";
    const ALLOWS_TRIGGERLESS_OPEN: bool = false;
}

/// Marker kind for `HoverCard` handles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct HoverCardKind;

impl OverlayHandleKind for HoverCardKind {
    const KIND_NAME: &'static str = "HoverCard";
    const ALLOWS_TRIGGERLESS_OPEN: bool = false;
}

/// Type-erased payload carried by detached overlay triggers and [`OverlayHandle`].
///
/// Construct with [`OverlayHandle::trigger_payload`] to enforce compile-time type matching
/// with a typed handle, or [`OverlayPayload::new`] when constructing dynamically.
#[derive(Clone)]
pub struct OverlayPayload {
    id: u64,
    value: Rc<dyn Any>,
    eq_fn: Option<fn(&dyn Any, &dyn Any) -> bool>,
}

impl OverlayPayload {
    /// Wraps `value` in a type-erased [`OverlayPayload`] with a unique identity nonce.
    pub fn new<P: 'static>(value: P) -> Self {
        Self {
            id: NEXT_PAYLOAD_ID.fetch_add(1, Ordering::Relaxed),
            value: Rc::new(value),
            eq_fn: None,
        }
    }

    /// Wraps a `PartialEq` `value` in a type-erased [`OverlayPayload`] that compares equal
    /// by value across re-renders (preventing spurious version bumps when triggers recreate
    /// payloads during render).
    pub fn new_eq<P: PartialEq + 'static>(value: P) -> Self {
        fn cmp<P: PartialEq + 'static>(a: &dyn Any, b: &dyn Any) -> bool {
            match (a.downcast_ref::<P>(), b.downcast_ref::<P>()) {
                (Some(lhs), Some(rhs)) => lhs == rhs,
                _ => false,
            }
        }
        Self {
            id: NEXT_PAYLOAD_ID.fetch_add(1, Ordering::Relaxed),
            value: Rc::new(value),
            eq_fn: Some(cmp::<P>),
        }
    }

    /// Returns the monotonic identity nonce for this payload instance.
    #[inline]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the [`TypeId`] of the stored concrete payload value.
    #[inline]
    pub fn type_id(&self) -> TypeId {
        (*self.value).type_id()
    }

    /// Attempts to borrow the stored payload as `&P`.
    #[inline]
    pub fn downcast_ref<P: 'static>(&self) -> Option<&P> {
        self.value.downcast_ref::<P>()
    }

    /// Attempts to clone the stored payload as `P`.
    #[inline]
    pub fn downcast_cloned<P: Clone + 'static>(&self) -> Option<P> {
        self.value.downcast_ref::<P>().cloned()
    }
}

impl PartialEq for OverlayPayload {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id && Rc::ptr_eq(&self.value, &other.value) {
            return true;
        }
        if let Some(eq_fn) = self.eq_fn
            && self.type_id() == other.type_id()
        {
            return eq_fn(&*self.value, &*other.value);
        }
        false
    }
}

impl fmt::Debug for OverlayPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OverlayPayload")
            .field("id", &self.id)
            .field("type_id", &self.type_id())
            .finish()
    }
}

/// Metadata registered by a detached `*Trigger` component on an overlay handle.
#[derive(Clone)]
pub struct RegisteredOverlayTrigger {
    pub id: String,
    pub mounted: Option<Rc<MountedData>>,
    pub payload: Option<OverlayPayload>,
}

impl fmt::Debug for RegisteredOverlayTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisteredOverlayTrigger")
            .field("id", &self.id)
            .field("has_mounted", &self.mounted.is_some())
            .field("has_payload", &self.payload.is_some())
            .finish()
    }
}

struct AttachedRootEntry {
    root_id: String,
    is_open: Rc<dyn Fn() -> bool>,
    set_open: Rc<dyn Fn(bool)>,
}

struct OverlayHandleCore {
    kind_name: &'static str,
    allows_triggerless_open: bool,
    attached_roots: Vec<AttachedRootEntry>,
    has_ever_attached: bool,
    warned_detached_verbs: HashSet<&'static str>,
    triggers: HashMap<String, RegisteredOverlayTrigger>,
    active_trigger_id: Option<String>,
    payload: Option<OverlayPayload>,
    opened_without_trigger: bool,
    open_state: bool,
    version: u64,
    next_subscriber_id: u64,
    subscribers: Vec<(u64, Signal<u64>)>,
    runtime_signals: Vec<(Weak<dioxus::core::Runtime>, Signal<u64>)>,
}

impl OverlayHandleCore {
    fn new(kind_name: &'static str, allows_triggerless_open: bool) -> Self {
        Self {
            kind_name,
            allows_triggerless_open,
            attached_roots: Vec::new(),
            has_ever_attached: false,
            warned_detached_verbs: HashSet::new(),
            triggers: HashMap::new(),
            active_trigger_id: None,
            payload: None,
            opened_without_trigger: false,
            open_state: false,
            version: 0,
            next_subscriber_id: 1,
            subscribers: Vec::new(),
            runtime_signals: Vec::new(),
        }
    }

    fn bump_version_and_collect_signals(&mut self) -> (u64, Vec<Signal<u64>>) {
        self.version = self.version.wrapping_add(1);
        let next_version = self.version;

        let mut live_signals = Vec::with_capacity(self.subscribers.len() + self.runtime_signals.len());

        self.subscribers.retain(|(_, sig)| {
            if sig.try_peek().is_ok() {
                live_signals.push(*sig);
                true
            } else {
                false
            }
        });

        self.runtime_signals.retain(|(weak_rt, sig)| {
            if weak_rt.upgrade().is_some() && sig.try_peek().is_ok() {
                live_signals.push(*sig);
                true
            } else {
                false
            }
        });

        (next_version, live_signals)
    }

    fn ensure_runtime_signal(&mut self) -> Option<Signal<u64>> {
        let cur_rt = dioxus::core::Runtime::try_current()?;
        self.runtime_signals
            .retain(|(weak_rt, sig)| weak_rt.upgrade().is_some() && sig.try_peek().is_ok());

        if let Some((_, existing)) = self.runtime_signals.iter().find(|(weak_rt, _)| {
            weak_rt
                .upgrade()
                .is_some_and(|rt| Rc::ptr_eq(&rt, &cur_rt))
        }) {
            return Some(*existing);
        }

        let version = self.version;
        let sig = cur_rt.in_scope(dioxus::core::ScopeId::ROOT, || Signal::new(version));
        self.runtime_signals.push((Rc::downgrade(&cur_rt), sig));
        Some(sig)
    }

    fn warn_if_never_attached(&mut self, verb: &'static str) {
        if !self.has_ever_attached && self.warned_detached_verbs.insert(verb) {
            crate::foundation::diagnostics::warn_detached_handle_action(self.kind_name, verb);
        }
    }

    fn active_root_setter(&self) -> Option<Rc<dyn Fn(bool)>> {
        self.attached_roots.last().map(|r| Rc::clone(&r.set_open))
    }

    fn active_root_is_open(&self) -> Option<Rc<dyn Fn() -> bool>> {
        self.attached_roots.last().map(|r| Rc::clone(&r.is_open))
    }
}

fn write_signals(next_version: u64, signals: Vec<Signal<u64>>) {
    for mut sig in signals {
        if let Ok(mut w) = sig.try_write() {
            *w = next_version;
        }
    }
}

/// Type-erased (payload-independent) overlay handle for a specific overlay family `K`.
///
/// Used internally by `*Root` and `*Trigger` components so their prop structs stay
/// completely non-generic.
pub struct AnyOverlayHandle<K> {
    core: Rc<RefCell<OverlayHandleCore>>,
    _kind: PhantomData<fn() -> K>,
}

impl<K> Clone for AnyOverlayHandle<K> {
    fn clone(&self) -> Self {
        Self {
            core: Rc::clone(&self.core),
            _kind: PhantomData,
        }
    }
}

impl<K> PartialEq for AnyOverlayHandle<K> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.core, &other.core)
    }
}

impl<K> fmt::Debug for AnyOverlayHandle<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let core = self.core.borrow();
        f.debug_struct("AnyOverlayHandle")
            .field("kind", &core.kind_name)
            .field("attached_roots", &core.attached_roots.len())
            .field("triggers", &core.triggers.len())
            .field("active_trigger_id", &core.active_trigger_id)
            .field("open_state", &core.open_state)
            .field("version", &core.version)
            .finish()
    }
}

impl<K> AnyOverlayHandle<K> {
    fn notify_subscribers(&self) {
        let (next_version, signals) = self.core.borrow_mut().bump_version_and_collect_signals();
        write_signals(next_version, signals);
    }

    fn track_reactive_read(&self) {
        let sig = self.core.borrow_mut().ensure_runtime_signal();
        if let Some(sig) = sig {
            let _ = sig.try_read();
        }
    }

    /// Returns the current reactive version counter, subscribing the active Dioxus reactive
    /// scope when called inside a component.
    pub fn subscribe(&self) -> u64 {
        self.track_reactive_read();
        self.core.borrow().version
    }

    /// Registers an explicit subscriber signal and returns its subscription ID.
    pub fn register_subscriber(&self, sig: Signal<u64>) -> u64 {
        let mut core = self.core.borrow_mut();
        let id = core.next_subscriber_id;
        core.next_subscriber_id = core.next_subscriber_id.saturating_add(1);
        core.subscribers.push((id, sig));
        id
    }

    /// Removes a previously registered subscriber signal by ID.
    pub fn unregister_subscriber(&self, subscriber_id: u64) {
        self.core
            .borrow_mut()
            .subscribers
            .retain(|(id, _)| *id != subscriber_id);
    }

    /// Returns `true` if at least one `*Root` is currently attached to this handle.
    pub fn is_attached(&self) -> bool {
        !self.core.borrow().attached_roots.is_empty()
    }

    /// Returns the current open state, subscribing the active reactive scope.
    /// Never emits detached diagnostics.
    pub fn is_open(&self) -> bool {
        self.track_reactive_read();
        let reader = self.core.borrow().active_root_is_open();
        if let Some(reader) = reader {
            reader()
        } else {
            self.core.borrow().open_state
        }
    }

    /// Returns the currently active trigger ID, subscribing the active reactive scope.
    /// Never emits detached diagnostics.
    pub fn active_trigger_id(&self) -> Option<String> {
        self.track_reactive_read();
        self.core.borrow().active_trigger_id.clone()
    }

    /// Returns `true` if the overlay was opened programmatically without a trigger.
    pub fn opened_without_trigger(&self) -> bool {
        self.track_reactive_read();
        self.core.borrow().opened_without_trigger
    }

    /// Returns the raw [`OverlayPayload`] currently associated with this handle, subscribing
    /// the active reactive scope.
    pub fn raw_payload(&self) -> Option<OverlayPayload> {
        self.track_reactive_read();
        self.core.borrow().payload.clone()
    }

    /// Attaches a `*Root` onto the LIFO attach stack (preserving previous roots during
    /// route-transition overlap). Emits a dev overlap warning if multiple roots are attached
    /// simultaneously.
    pub fn attach_root(
        &self,
        root_id: impl Into<String>,
        is_open: Rc<dyn Fn() -> bool>,
        set_open: Rc<dyn Fn(bool)>,
    ) {
        let root_id = root_id.into();
        let (should_warn_overlap, kind_name, next_version, signals) = {
            let mut core = self.core.borrow_mut();
            core.has_ever_attached = true;
            if let Some(existing) = core
                .attached_roots
                .iter_mut()
                .find(|entry| entry.root_id == root_id)
            {
                existing.is_open = is_open;
                existing.set_open = set_open;
            } else {
                core.attached_roots.push(AttachedRootEntry {
                    root_id,
                    is_open,
                    set_open,
                });
            }
            let overlap = core.attached_roots.len() > 1;
            let kind_name = core.kind_name;
            let (v, s) = core.bump_version_and_collect_signals();
            (overlap, kind_name, v, s)
        };
        if should_warn_overlap {
            crate::foundation::diagnostics::warn_handle_overlap(kind_name);
        }
        write_signals(next_version, signals);
    }

    /// Detaches `root_id` from the LIFO attach stack, restoring any previous attached root.
    pub fn detach_root(&self, root_id: &str) {
        let (next_version, signals) = {
            let mut core = self.core.borrow_mut();
            let before = core.attached_roots.len();
            core.attached_roots.retain(|entry| entry.root_id != root_id);
            if core.attached_roots.len() == before {
                return;
            }
            if core.attached_roots.is_empty() {
                core.open_state = false;
            }
            core.bump_version_and_collect_signals()
        };
        write_signals(next_version, signals);
    }

    /// Registers (or updates) a detached `*Trigger` on this handle.
    pub fn register_trigger(
        &self,
        trigger_id: impl Into<String>,
        mounted: Option<Rc<MountedData>>,
        payload: Option<OverlayPayload>,
    ) {
        let id = trigger_id.into();
        if id.is_empty() {
            return;
        }
        let (next_version, signals) = {
            let mut core = self.core.borrow_mut();
            let is_active = core.active_trigger_id.as_deref() == Some(id.as_str());
            let existed = core.triggers.contains_key(&id);
            let entry = core
                .triggers
                .entry(id.clone())
                .or_insert_with(|| RegisteredOverlayTrigger {
                    id: id.clone(),
                    mounted: None,
                    payload: None,
                });
            if mounted.is_some() {
                entry.mounted = mounted;
            }
            let payload_changed = entry.payload != payload;
            entry.payload = payload.clone();
            if is_active && payload_changed {
                core.payload = payload;
                core.bump_version_and_collect_signals()
            } else if !existed {
                // Notify attached roots once so `dismiss_boundaries` includes the new trigger ID.
                core.bump_version_and_collect_signals()
            } else {
                return;
            }
        };
        write_signals(next_version, signals);
    }

    /// Returns all currently registered detached trigger IDs on this handle.
    pub fn registered_trigger_ids(&self) -> Vec<String> {
        self.core.borrow().triggers.keys().cloned().collect()
    }

    /// Updates the [`MountedData`] for a registered trigger.
    pub fn update_trigger_mounted(&self, trigger_id: &str, mounted: Option<Rc<MountedData>>) {
        if trigger_id.is_empty() {
            return;
        }
        let (should_notify, next_version, signals) = {
            let mut core = self.core.borrow_mut();
            let is_active = core.active_trigger_id.as_deref() == Some(trigger_id);
            if let Some(entry) = core.triggers.get_mut(trigger_id) {
                entry.mounted = mounted;
            } else {
                core.triggers.insert(
                    trigger_id.to_owned(),
                    RegisteredOverlayTrigger {
                        id: trigger_id.to_owned(),
                        mounted,
                        payload: None,
                    },
                );
            }
            if is_active {
                let (v, s) = core.bump_version_and_collect_signals();
                (true, v, s)
            } else {
                (false, 0, Vec::new())
            }
        };
        if should_notify {
            write_signals(next_version, signals);
        }
    }

    /// Updates the payload for a registered trigger, propagating immediately if it is active.
    pub fn update_trigger_payload(&self, trigger_id: &str, payload: Option<OverlayPayload>) {
        if trigger_id.is_empty() {
            return;
        }
        let (should_notify, next_version, signals) = {
            let mut core = self.core.borrow_mut();
            let is_active = core.active_trigger_id.as_deref() == Some(trigger_id);
            let changed = if let Some(entry) = core.triggers.get_mut(trigger_id) {
                let diff = entry.payload != payload;
                entry.payload = payload.clone();
                diff
            } else {
                core.triggers.insert(
                    trigger_id.to_owned(),
                    RegisteredOverlayTrigger {
                        id: trigger_id.to_owned(),
                        mounted: None,
                        payload: payload.clone(),
                    },
                );
                true
            };
            if is_active && changed {
                core.payload = payload;
                let (v, s) = core.bump_version_and_collect_signals();
                (true, v, s)
            } else {
                (false, 0, Vec::new())
            }
        };
        if should_notify {
            write_signals(next_version, signals);
        }
    }

    /// Unregisters a trigger on unmount.
    pub fn unregister_trigger(&self, trigger_id: &str) {
        if trigger_id.is_empty() {
            return;
        }
        let (should_notify, next_version, signals) = {
            let mut core = self.core.borrow_mut();
            let removed = core.triggers.remove(trigger_id).is_some();
            if !removed {
                return;
            }
            let was_active = core.active_trigger_id.as_deref() == Some(trigger_id);
            if was_active && !core.open_state {
                core.active_trigger_id = None;
            }
            // Notify attached roots so stale trigger IDs are removed from `dismiss_boundaries`.
            let (v, s) = core.bump_version_and_collect_signals();
            (true, v, s)
        };
        if should_notify {
            write_signals(next_version, signals);
        }
    }

    /// Activates `trigger_id` (updating `active_trigger_id`, `payload`, and clearing
    /// `opened_without_trigger`). When `open_if_closed` is `true` and an attached root is
    /// currently closed, requests the active root to open; when already open on another
    /// trigger, switches triggers cleanly without closing.
    pub fn activate_trigger(&self, trigger_id: &str, open_if_closed: bool) -> bool {
        if trigger_id.is_empty() {
            return false;
        }
        let (setter, reader, was_open, next_version, signals) = {
            let mut core = self.core.borrow_mut();
            let trigger_payload = core
                .triggers
                .get(trigger_id)
                .and_then(|entry| entry.payload.clone());
            core.active_trigger_id = Some(trigger_id.to_owned());
            core.payload = trigger_payload;
            core.opened_without_trigger = false;
            let setter = core.active_root_setter();
            let reader = core.active_root_is_open();
            let was_open = core.open_state;
            let (v, s) = core.bump_version_and_collect_signals();
            (setter, reader, was_open, v, s)
        };
        write_signals(next_version, signals);

        let is_currently_open = reader.as_ref().map(|r| r()).unwrap_or(was_open);
        if open_if_closed && !is_currently_open {
            if let Some(setter) = setter {
                setter(true);
                return true;
            }
            return false;
        }
        true
    }

    /// Synchronizes the handle's cached open state from the attached `*Root`.
    ///
    /// When transitioning to closed, preserves `active_trigger_id` so focus restoration
    /// targets the last active trigger until the next open or explicit clear.
    pub fn sync_open_state(&self, open: bool) {
        let (changed, next_version, signals) = {
            let mut core = self.core.borrow_mut();
            if core.open_state == open {
                (false, 0, Vec::new())
            } else {
                core.open_state = open;
                let (v, s) = core.bump_version_and_collect_signals();
                (true, v, s)
            }
        };
        if changed {
            write_signals(next_version, signals);
        }
    }

    /// Synchronizes the handle's cached open state ONLY when `root_id` is the active
    /// LIFO top root in `attached_roots`, preventing overlapping route-transition roots
    /// from ping-ponging `open_state` across shared subscribers.
    pub fn sync_open_state_for_root(&self, root_id: &str, open: bool) {
        let (changed, next_version, signals) = {
            let mut core = self.core.borrow_mut();
            let is_top = core
                .attached_roots
                .last()
                .is_some_and(|top| top.root_id == root_id);
            if !is_top || core.open_state == open {
                (false, 0, Vec::new())
            } else {
                core.open_state = open;
                let (v, s) = core.bump_version_and_collect_signals();
                (true, v, s)
            }
        };
        if changed {
            write_signals(next_version, signals);
        }
    }

    /// Returns the [`MountedData`] of the currently active trigger, if registered and mounted.
    pub fn active_trigger_mounted(&self) -> Option<Rc<MountedData>> {
        self.track_reactive_read();
        let core = self.core.borrow();
        let active_id = core.active_trigger_id.as_deref()?;
        core.triggers
            .get(active_id)
            .and_then(|entry| entry.mounted.clone())
    }

    /// Returns `true` iff the overlay is currently open AND `trigger_id` is the active trigger
    /// (and was not opened via `open_with_payload` / triggerless open).
    pub fn is_opened_by_trigger(&self, trigger_id: &str) -> bool {
        if trigger_id.is_empty() {
            return false;
        }
        let open = self.is_open();
        if !open {
            return false;
        }
        let core = self.core.borrow();
        !core.opened_without_trigger && core.active_trigger_id.as_deref() == Some(trigger_id)
    }

    /// Opens the overlay, optionally targeting a specific registered `trigger_id`.
    ///
    /// - For `Dialog`/`AlertDialog`, `None` opens without a trigger (`opened_without_trigger = true`).
    /// - For anchored overlays (`Popover`/`Tooltip`/`HoverCard`), `None` or an unregistered
    ///   `trigger_id` returns `false` without panicking.
    /// - When detached and never attached, emits a dev-only warn-once `DetachedHandleAction`.
    pub fn open(&self, trigger_id: Option<&str>) -> bool {
        let (setter, next_version, signals) = {
            let mut core = self.core.borrow_mut();
            let Some(setter) = core.active_root_setter() else {
                core.warn_if_never_attached("open");
                return false;
            };
            match trigger_id {
                Some(id) if !id.is_empty() => {
                    let Some(trigger_payload) =
                        core.triggers.get(id).map(|trigger| trigger.payload.clone())
                    else {
                        if !core.allows_triggerless_open {
                            return false;
                        }
                        core.active_trigger_id = Some(id.to_owned());
                        core.payload = None;
                        core.opened_without_trigger = false;
                        let (v, s) = core.bump_version_and_collect_signals();
                        return {
                            drop(core);
                            write_signals(v, s);
                            setter(true);
                            true
                        };
                    };
                    core.active_trigger_id = Some(id.to_owned());
                    core.payload = trigger_payload;
                    core.opened_without_trigger = false;
                }
                _ => {
                    if !core.allows_triggerless_open {
                        return false;
                    }
                    core.active_trigger_id = None;
                    core.opened_without_trigger = true;
                }
            }
            let (v, s) = core.bump_version_and_collect_signals();
            (setter, v, s)
        };
        write_signals(next_version, signals);
        setter(true);
        true
    }

    /// Opens the overlay programmatically with `payload` and no associated trigger
    /// (`opened_without_trigger = true`, `active_trigger_id = None`).
    pub fn open_with_raw_payload(&self, payload: OverlayPayload) -> bool {
        let (setter, next_version, signals) = {
            let mut core = self.core.borrow_mut();
            let Some(setter) = core.active_root_setter() else {
                core.warn_if_never_attached("open_with_payload");
                return false;
            };
            core.active_trigger_id = None;
            core.opened_without_trigger = true;
            core.payload = Some(payload);
            let (v, s) = core.bump_version_and_collect_signals();
            (setter, v, s)
        };
        write_signals(next_version, signals);
        setter(true);
        true
    }

    /// Closes the attached overlay root.
    ///
    /// When detached and never attached, emits a dev-only warn-once `DetachedHandleAction`.
    pub fn close(&self) -> bool {
        let setter = {
            let mut core = self.core.borrow_mut();
            let Some(setter) = core.active_root_setter() else {
                core.warn_if_never_attached("close");
                return false;
            };
            setter
        };
        setter(false);
        self.notify_subscribers();
        true
    }
}

/// Typed detached-trigger and programmatic control handle for overlay families.
///
/// Pass directly to non-generic `*Root { handle: my_handle }` and `*Trigger { handle: my_handle }`
/// props via [`OverlayHandleProp`].
pub struct OverlayHandle<K, P = ()> {
    any: AnyOverlayHandle<K>,
    _payload: PhantomData<fn() -> P>,
}

impl<K, P> Clone for OverlayHandle<K, P> {
    fn clone(&self) -> Self {
        Self {
            any: self.any.clone(),
            _payload: PhantomData,
        }
    }
}

impl<K, P> PartialEq for OverlayHandle<K, P> {
    fn eq(&self, other: &Self) -> bool {
        self.any == other.any
    }
}

impl<K, P> fmt::Debug for OverlayHandle<K, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OverlayHandle").field(&self.any).finish()
    }
}

impl<K: OverlayHandleKind, P> Default for OverlayHandle<K, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: OverlayHandleKind, P> OverlayHandle<K, P> {
    /// Creates a new detached [`OverlayHandle`]. Works both inside Dioxus components and
    /// standalone (e.g. in unit tests or external state stores).
    pub fn new() -> Self {
        Self {
            any: AnyOverlayHandle {
                core: Rc::new(RefCell::new(OverlayHandleCore::new(
                    K::KIND_NAME,
                    K::ALLOWS_TRIGGERLESS_OPEN,
                ))),
                _kind: PhantomData,
            },
            _payload: PhantomData,
        }
    }
}

impl<K, P> OverlayHandle<K, P> {
    /// Returns the payload-erased [`AnyOverlayHandle`] view of this handle.
    #[inline]
    pub fn as_any(&self) -> &AnyOverlayHandle<K> {
        &self.any
    }

    /// Consumes this typed handle and returns its payload-erased [`AnyOverlayHandle`].
    #[inline]
    pub fn into_any(self) -> AnyOverlayHandle<K> {
        self.any
    }

    /// Opens the overlay, optionally targeting a registered `trigger_id`.
    #[inline]
    pub fn open(&self, trigger_id: Option<&str>) -> bool {
        self.any.open(trigger_id)
    }

    /// Closes the attached overlay root.
    #[inline]
    pub fn close(&self) -> bool {
        self.any.close()
    }

    /// Returns whether the overlay is currently open (reactive inside components; never warns).
    #[inline]
    pub fn is_open(&self) -> bool {
        self.any.is_open()
    }

    /// Returns whether at least one `*Root` is currently attached (never warns).
    #[inline]
    pub fn is_attached(&self) -> bool {
        self.any.is_attached()
    }

    /// Returns the currently active trigger ID (reactive inside components; never warns).
    #[inline]
    pub fn active_trigger_id(&self) -> Option<String> {
        self.any.active_trigger_id()
    }

    /// Returns `true` if opened programmatically without an associated trigger.
    #[inline]
    pub fn opened_without_trigger(&self) -> bool {
        self.any.opened_without_trigger()
    }
}

impl<K, P: PartialEq + 'static> OverlayHandle<K, P> {
    /// Wraps `value` into an [`OverlayPayload`] while statically checking that `P` matches
    /// this handle's payload type and preserving value equality across trigger re-renders.
    #[inline]
    pub fn trigger_payload(&self, value: P) -> OverlayPayload {
        OverlayPayload::new_eq(value)
    }
}

impl<K, P: 'static> OverlayHandle<K, P> {
    /// Opens the overlay programmatically with `payload` and no associated trigger.
    #[inline]
    pub fn open_with_payload(&self, payload: P) -> bool {
        self.any.open_with_raw_payload(OverlayPayload::new(payload))
    }
}

impl<K, P: Clone + 'static> OverlayHandle<K, P> {
    /// Returns the currently active typed payload (reactive inside components; never warns).
    #[inline]
    pub fn payload(&self) -> Option<P> {
        self.any
            .raw_payload()
            .and_then(|payload| payload.downcast_cloned::<P>())
    }
}

/// Non-generic prop wrapper accepted by `*RootProps` and `*TriggerProps` via
/// `#[props(default, into)] pub handle: OverlayHandleProp<K>`.
pub struct OverlayHandleProp<K>(pub Option<AnyOverlayHandle<K>>);

impl<K> Clone for OverlayHandleProp<K> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K> Default for OverlayHandleProp<K> {
    fn default() -> Self {
        Self(None)
    }
}

impl<K> PartialEq for OverlayHandleProp<K> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K> fmt::Debug for OverlayHandleProp<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OverlayHandleProp").field(&self.0).finish()
    }
}

impl<K> OverlayHandleProp<K> {
    #[inline]
    pub fn as_any(&self) -> Option<&AnyOverlayHandle<K>> {
        self.0.as_ref()
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl<K, P: 'static> From<OverlayHandle<K, P>> for OverlayHandleProp<K> {
    fn from(handle: OverlayHandle<K, P>) -> Self {
        Self(Some(handle.into_any()))
    }
}

impl<K, P: 'static> From<Option<OverlayHandle<K, P>>> for OverlayHandleProp<K> {
    fn from(handle: Option<OverlayHandle<K, P>>) -> Self {
        Self(handle.map(OverlayHandle::into_any))
    }
}

impl<K> From<AnyOverlayHandle<K>> for OverlayHandleProp<K> {
    fn from(handle: AnyOverlayHandle<K>) -> Self {
        Self(Some(handle))
    }
}

impl<K> From<Option<AnyOverlayHandle<K>>> for OverlayHandleProp<K> {
    fn from(handle: Option<AnyOverlayHandle<K>>) -> Self {
        Self(handle)
    }
}

/// Typed detached handle for `Dialog`.
pub type DialogHandle<P = ()> = OverlayHandle<DialogKind, P>;
/// Typed detached handle for `AlertDialog` (aliased to [`DialogHandle`] so `DialogTrigger`
/// interoperates seamlessly with `AlertDialogRoot`).
pub type AlertDialogHandle<P = ()> = DialogHandle<P>;
/// Typed detached handle for `Popover`.
pub type PopoverHandle<P = ()> = OverlayHandle<PopoverKind, P>;
/// Typed detached handle for `Tooltip`.
pub type TooltipHandle<P = ()> = OverlayHandle<TooltipKind, P>;
/// Typed detached handle for `HoverCard`.
pub type HoverCardHandle<P = ()> = OverlayHandle<HoverCardKind, P>;

/// Non-generic prop wrapper for `DialogRoot` / `DialogTrigger` `handle` props.
pub type DialogHandleProp = OverlayHandleProp<DialogKind>;
/// Non-generic prop wrapper for `AlertDialogRoot` / `AlertDialogTrigger` `handle` props.
pub type AlertDialogHandleProp = DialogHandleProp;
/// Non-generic prop wrapper for `PopoverRoot` / `PopoverTrigger` `handle` props.
pub type PopoverHandleProp = OverlayHandleProp<PopoverKind>;
/// Non-generic prop wrapper for `TooltipRoot` / `TooltipTrigger` `handle` props.
pub type TooltipHandleProp = OverlayHandleProp<TooltipKind>;
/// Non-generic prop wrapper for `HoverCardRoot` / `HoverCardTrigger` `handle` props.
pub type HoverCardHandleProp = OverlayHandleProp<HoverCardKind>;

/// Payload-erased handle aliases.
pub type AnyDialogHandle = AnyOverlayHandle<DialogKind>;
pub type AnyAlertDialogHandle = AnyDialogHandle;
pub type AnyPopoverHandle = AnyOverlayHandle<PopoverKind>;
pub type AnyTooltipHandle = AnyOverlayHandle<TooltipKind>;
pub type AnyHoverCardHandle = AnyOverlayHandle<HoverCardKind>;

/// Creates and memoizes a typed [`OverlayHandle`] for the lifetime of the current component.
pub fn use_overlay_handle<K: OverlayHandleKind, P: 'static>() -> OverlayHandle<K, P> {
    use_hook(OverlayHandle::<K, P>::new)
}

/// Creates and memoizes a [`DialogHandle`] for the lifetime of the current component.
pub fn use_dialog_handle<P: 'static>() -> DialogHandle<P> {
    use_overlay_handle::<DialogKind, P>()
}

/// Creates and memoizes an [`AlertDialogHandle`] for the lifetime of the current component.
pub fn use_alert_dialog_handle<P: 'static>() -> AlertDialogHandle<P> {
    use_dialog_handle::<P>()
}

/// Creates and memoizes a [`PopoverHandle`] for the lifetime of the current component.
pub fn use_popover_handle<P: 'static>() -> PopoverHandle<P> {
    use_overlay_handle::<PopoverKind, P>()
}

/// Creates and memoizes a [`TooltipHandle`] for the lifetime of the current component.
pub fn use_tooltip_handle<P: 'static>() -> TooltipHandle<P> {
    use_overlay_handle::<TooltipKind, P>()
}

/// Creates and memoizes a [`HoverCardHandle`] for the lifetime of the current component.
pub fn use_hover_card_handle<P: 'static>() -> HoverCardHandle<P> {
    use_overlay_handle::<HoverCardKind, P>()
}

/// Subscribes the current component to reactive version updates on an optional
/// [`AnyOverlayHandle`], cleaning up automatically on handle change or unmount.
pub fn use_overlay_handle_version<K: 'static>(handle: Option<&AnyOverlayHandle<K>>) -> u64 {
    let version = use_signal(|| handle.map(|h| h.core.borrow().version).unwrap_or(0));
    let sub: Rc<RefCell<Option<(AnyOverlayHandle<K>, u64)>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    {
        let mut slot = sub.borrow_mut();
        let needs_rebind = match (&*slot, handle) {
            (Some((prev, _)), Some(next)) => prev != next,
            (Some(_), None) | (None, Some(_)) => true,
            (None, None) => false,
        };
        if needs_rebind {
            if let Some((prev, id)) = slot.take() {
                prev.unregister_subscriber(id);
            }
            if let Some(next) = handle {
                let id = next.register_subscriber(version);
                *slot = Some((next.clone(), id));
            }
        }
    }

    let drop_sub = Rc::clone(&sub);
    use_drop(move || {
        if let Some((prev, id)) = drop_sub.borrow_mut().take() {
            prev.unregister_subscriber(id);
        }
    });

    version()
}

/// Subscribes the current component to reactive version updates on a typed [`OverlayHandle`].
pub fn use_handle_version<K: 'static, P>(handle: &OverlayHandle<K, P>) -> u64 {
    use_overlay_handle_version(Some(handle.as_any()))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ItemPayload {
        id: u32,
        name: String,
    }

    #[test]
    fn handle_prop_accepts_typed_and_option_conversions() {
        let handle = DialogHandle::<ItemPayload>::new();
        let prop_direct: DialogHandleProp = handle.clone().into();
        let prop_some: DialogHandleProp = Some(handle.clone()).into();
        let prop_none: DialogHandleProp = Option::<DialogHandle<ItemPayload>>::None.into();

        assert!(prop_direct.is_some());
        assert!(prop_some.is_some());
        assert_eq!(prop_direct, prop_some);
        assert!(prop_none.is_none());

        // AlertDialogHandle aliases DialogHandle seamlessly.
        let alert_handle: AlertDialogHandle<ItemPayload> = handle;
        let alert_prop: AlertDialogHandleProp = alert_handle.into();
        assert_eq!(alert_prop, prop_direct);
    }

    #[test]
    fn detached_verbs_warn_once_before_first_attach_and_never_after_unmount() {
        let guard = crate::foundation::diagnostics::capture_diagnostics();
        let handle = DialogHandle::<String>::new();

        // Reads never warn.
        assert!(!handle.is_attached());
        assert!(!handle.is_open());
        assert_eq!(handle.active_trigger_id(), None);
        assert_eq!(handle.payload(), None);
        assert!(guard.diagnostics().is_empty());

        // Detached verbs warn once per verb before first attach.
        assert!(!handle.open(None));
        assert!(!handle.open(None));
        assert!(!handle.close());
        assert!(!handle.open_with_payload("hello".to_owned()));
        let first_batch = guard.take();
        assert_eq!(first_batch.len(), 3);
        assert!(
            first_batch
                .iter()
                .all(|d| d.kind == crate::foundation::diagnostics::DiagnosticKind::DetachedHandleAction)
        );

        // Attach and detach (simulating unmount) -> has_ever_attached == true -> no more warnings.
        let open_flag = Rc::new(Cell::new(false));
        let reader_flag = Rc::clone(&open_flag);
        let setter_flag = Rc::clone(&open_flag);
        handle.as_any().attach_root(
            "root-1",
            Rc::new(move || reader_flag.get()),
            Rc::new(move |next| setter_flag.set(next)),
        );
        assert!(handle.is_attached());
        handle.as_any().detach_root("root-1");
        assert!(!handle.is_attached());

        assert!(!handle.open(None));
        assert!(!handle.close());
        assert!(guard.diagnostics().is_empty());
    }

    #[test]
    fn multi_trigger_switching_and_open_with_payload_lifecycle() {
        let handle = DialogHandle::<ItemPayload>::new();
        let open_flag = Rc::new(Cell::new(false));
        let open_calls = Rc::new(Cell::new(0usize));

        {
            let reader = Rc::clone(&open_flag);
            let setter = Rc::clone(&open_flag);
            let calls = Rc::clone(&open_calls);
            let any = handle.as_any().clone();
            handle.as_any().attach_root(
                "dialog-root",
                Rc::new(move || reader.get()),
                Rc::new(move |next| {
                    calls.set(calls.get() + 1);
                    setter.set(next);
                    any.sync_open_state(next);
                }),
            );
        }

        let payload_a = ItemPayload {
            id: 1,
            name: "Alpha".to_owned(),
        };
        let payload_b = ItemPayload {
            id: 2,
            name: "Beta".to_owned(),
        };

        handle.as_any().register_trigger(
            "trigger-a",
            None,
            Some(handle.trigger_payload(payload_a.clone())),
        );
        handle.as_any().register_trigger(
            "trigger-b",
            None,
            Some(handle.trigger_payload(payload_b.clone())),
        );

        // Clicking Trigger A opens with Payload A.
        assert!(handle.as_any().activate_trigger("trigger-a", true));
        assert!(handle.is_open());
        assert_eq!(handle.active_trigger_id().as_deref(), Some("trigger-a"));
        assert_eq!(handle.payload(), Some(payload_a));
        assert!(handle.as_any().is_opened_by_trigger("trigger-a"));
        assert!(!handle.as_any().is_opened_by_trigger("trigger-b"));
        assert_eq!(open_calls.get(), 1);

        // Switching to Trigger B while open updates payload & active trigger without re-opening/closing.
        assert!(handle.as_any().activate_trigger("trigger-b", true));
        assert!(handle.is_open());
        assert_eq!(handle.active_trigger_id().as_deref(), Some("trigger-b"));
        assert_eq!(handle.payload(), Some(payload_b));
        assert!(!handle.as_any().is_opened_by_trigger("trigger-a"));
        assert!(handle.as_any().is_opened_by_trigger("trigger-b"));
        assert_eq!(open_calls.get(), 1);

        // Closing preserves active_trigger_id for focus restoration.
        assert!(handle.close());
        assert!(!handle.is_open());
        assert_eq!(handle.active_trigger_id().as_deref(), Some("trigger-b"));

        // Programmatic open_with_payload sets opened_without_trigger and clears active_trigger_id.
        let payload_c = ItemPayload {
            id: 3,
            name: "Gamma".to_owned(),
        };
        assert!(handle.open_with_payload(payload_c.clone()));
        assert!(handle.is_open());
        assert!(handle.opened_without_trigger());
        assert_eq!(handle.active_trigger_id(), None);
        assert_eq!(handle.payload(), Some(payload_c));
        assert!(!handle.as_any().is_opened_by_trigger("trigger-a"));
        assert!(!handle.as_any().is_opened_by_trigger("trigger-b"));
    }

    #[test]
    fn anchored_popover_rejects_triggerless_or_missing_trigger_open() {
        let handle = PopoverHandle::<String>::new();
        let open_flag = Rc::new(Cell::new(false));
        {
            let reader = Rc::clone(&open_flag);
            let setter = Rc::clone(&open_flag);
            handle.as_any().attach_root(
                "popover-root",
                Rc::new(move || reader.get()),
                Rc::new(move |next| setter.set(next)),
            );
        }

        assert!(!handle.open(None));
        assert!(!handle.open(Some("missing-trigger")));
        assert!(!handle.is_open());

        handle.as_any().register_trigger(
            "anchor-1",
            None,
            Some(handle.trigger_payload("ctx".to_owned())),
        );
        assert!(handle.open(Some("anchor-1")));
        assert!(handle.is_open());
        assert_eq!(handle.payload().as_deref(), Some("ctx"));
    }

    #[test]
    fn lifo_attach_stack_survives_route_transition_overlap_and_warns() {
        let guard = crate::foundation::diagnostics::capture_diagnostics();
        let handle = DialogHandle::<()>::new();

        let root_a_open = Rc::new(Cell::new(false));
        let root_b_open = Rc::new(Cell::new(false));

        {
            let ra = Rc::clone(&root_a_open);
            let sa = Rc::clone(&root_a_open);
            handle.as_any().attach_root(
                "route-a",
                Rc::new(move || ra.get()),
                Rc::new(move |v| sa.set(v)),
            );
        }
        assert!(guard.diagnostics().is_empty());

        // Overlapping mount of route-b emits HandleOverlap warning and becomes active top.
        {
            let rb = Rc::clone(&root_b_open);
            let sb = Rc::clone(&root_b_open);
            handle.as_any().attach_root(
                "route-b",
                Rc::new(move || rb.get()),
                Rc::new(move |v| sb.set(v)),
            );
        }
        assert_eq!(guard.diagnostics().len(), 1);
        assert_eq!(
            guard.diagnostics()[0].kind,
            crate::foundation::diagnostics::DiagnosticKind::HandleOverlap
        );

        assert!(handle.open(None));
        assert!(root_b_open.get());
        assert!(!root_a_open.get());

        // Detaching route-a leaves route-b intact; detaching route-b restores nothing.
        handle.as_any().detach_root("route-a");
        assert!(handle.is_attached());
        assert!(handle.close());
        assert!(!root_b_open.get());

        handle.as_any().detach_root("route-b");
        assert!(!handle.is_attached());
    }
}

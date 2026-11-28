//! Dev-only diagnostic reporting, capture guards, and controlled-refusal
//! tracking (`S2-LD-05`).
//!
//! All public types and functions remain available in release builds so
//! callsites require no `#[cfg(debug_assertions)]` gating, while internal
//! storage and emission compile down to inline no-ops when `debug_assertions`
//! are disabled.

use std::time::Duration;

#[cfg(debug_assertions)]
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

/// Default grace window before an unacknowledged controlled transition emits
/// [`DiagnosticKind::ControlledRefusal`].
pub const DEFAULT_CONTROLLED_REFUSAL_GRACE: Duration = Duration::from_millis(1000);

/// Categories of development-time diagnostics emitted by `monoxus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticKind {
    ControlledRefusal,
    ControlledModeSwitch,
    ControlledAndDefaultProvided,
    DetachedHandleAction,
    TwoWayAndReadOnlyConflict,
    HandleOverlap,
    InvalidProp,
}

/// Structured development diagnostic payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevDiagnostic {
    pub kind: DiagnosticKind,
    pub component: &'static str,
    pub instance_id: String,
    pub prop: &'static str,
    pub message: String,
}

#[cfg(debug_assertions)]
type SinkBuf = Rc<RefCell<Vec<DevDiagnostic>>>;

#[cfg(debug_assertions)]
type PendingCheckMap = HashMap<u64, (u64, Box<dyn FnOnce()>)>;

#[cfg(debug_assertions)]
static NEXT_DIAG_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

#[cfg(debug_assertions)]
thread_local! {
    static CAPTURE_SINK: RefCell<Option<SinkBuf>> = const { RefCell::new(None) };
    static STRICT_MODE: Cell<bool> = const { Cell::new(false) };
    static REFUSAL_GRACE: Cell<Duration> = const { Cell::new(DEFAULT_CONTROLLED_REFUSAL_GRACE) };
    static PENDING_CHECKS: RefCell<PendingCheckMap> = RefCell::new(HashMap::new());
}

/// RAII guard that captures emitted [`DevDiagnostic`] values on the current
/// thread and restores the previous sink on drop.
pub struct DiagnosticCaptureGuard {
    #[cfg(debug_assertions)]
    buf: SinkBuf,
    #[cfg(debug_assertions)]
    previous: Option<SinkBuf>,
}

impl DiagnosticCaptureGuard {
    /// Returns a cloned snapshot of all diagnostics captured so far.
    #[inline]
    pub fn diagnostics(&self) -> Vec<DevDiagnostic> {
        #[cfg(debug_assertions)]
        {
            self.buf.borrow().clone()
        }
        #[cfg(not(debug_assertions))]
        {
            Vec::new()
        }
    }

    /// Drains and returns all diagnostics captured so far.
    #[inline]
    pub fn take(&self) -> Vec<DevDiagnostic> {
        #[cfg(debug_assertions)]
        {
            std::mem::take(&mut *self.buf.borrow_mut())
        }
        #[cfg(not(debug_assertions))]
        {
            Vec::new()
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for DiagnosticCaptureGuard {
    fn drop(&mut self) {
        let prev = self.previous.take();
        let _ = CAPTURE_SINK.try_with(|slot| {
            *slot.borrow_mut() = prev;
        });
    }
}

/// Installs a thread-local diagnostic capture sink and returns an RAII guard
/// that restores the previous sink on drop.
#[inline]
pub fn capture_diagnostics() -> DiagnosticCaptureGuard {
    #[cfg(debug_assertions)]
    {
        let buf = Rc::new(RefCell::new(Vec::new()));
        let previous = CAPTURE_SINK.with(|slot| slot.borrow_mut().replace(Rc::clone(&buf)));
        DiagnosticCaptureGuard { buf, previous }
    }
    #[cfg(not(debug_assertions))]
    {
        DiagnosticCaptureGuard {}
    }
}

/// Enables or disables thread-local strict mode. When enabled in debug builds,
/// [`emit_diagnostic`] panics after recording/logging the diagnostic.
#[inline]
pub fn set_strict_mode(strict: bool) {
    #[cfg(debug_assertions)]
    {
        STRICT_MODE.with(|flag| flag.set(strict));
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = strict;
    }
}

/// Returns whether thread-local strict mode is currently enabled.
#[inline]
pub fn is_strict_mode() -> bool {
    #[cfg(debug_assertions)]
    {
        STRICT_MODE.with(Cell::get)
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

/// Sets the thread-local controlled-refusal grace duration and returns the
/// previous duration.
#[inline]
pub fn with_controlled_refusal_grace(duration: Duration) -> Duration {
    #[cfg(debug_assertions)]
    {
        REFUSAL_GRACE.with(|cell| {
            let prev = cell.get();
            cell.set(duration);
            prev
        })
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = duration;
        DEFAULT_CONTROLLED_REFUSAL_GRACE
    }
}

/// Returns the current thread-local controlled-refusal grace duration.
#[inline]
pub fn controlled_refusal_grace() -> Duration {
    #[cfg(debug_assertions)]
    {
        REFUSAL_GRACE.with(Cell::get)
    }
    #[cfg(not(debug_assertions))]
    {
        DEFAULT_CONTROLLED_REFUSAL_GRACE
    }
}

/// Synchronously evaluates and drains all pending controlled-refusal checks on
/// the current thread. Intended for deterministic headless `VirtualDom` tests.
#[inline]
pub fn flush_pending_controlled_checks() {
    #[cfg(debug_assertions)]
    {
        let checks: Vec<Box<dyn FnOnce()>> = PENDING_CHECKS.with(|map| {
            let mut map = map.borrow_mut();
            map.drain().map(|(_, (_, check))| check).collect()
        });
        for check in checks {
            check();
        }
    }
}

/// Emits a development diagnostic to the active thread-local capture sink (if
/// installed) or logs it to stderr on non-WASM hosts. Panics when strict mode
/// is active. Compiles to an empty inline no-op in release builds.
#[inline]
pub fn emit_diagnostic(diag: DevDiagnostic) {
    #[cfg(debug_assertions)]
    {
        let captured = CAPTURE_SINK.with(|slot| {
            if let Some(buf) = slot.borrow().as_ref() {
                buf.borrow_mut().push(diag.clone());
                true
            } else {
                false
            }
        });

        if !captured {
            #[cfg(not(target_arch = "wasm32"))]
            {
                eprintln!(
                    "[monoxus {:?}] <{}#{}> ({}): {}",
                    diag.kind, diag.component, diag.instance_id, diag.prop, diag.message
                );
            }
        }

        if STRICT_MODE.with(Cell::get) {
            panic!(
                "[monoxus strict {:?}] <{}#{}> ({}): {}",
                diag.kind, diag.component, diag.instance_id, diag.prop, diag.message
            );
        }
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = diag;
    }
}

/// Emits [`DiagnosticKind::DetachedHandleAction`] when a handle verb is called
/// before the handle has ever attached to a mounted root.
#[inline]
pub fn warn_detached_handle_action(component: &'static str, verb: &'static str) {
    #[cfg(debug_assertions)]
    {
        emit_diagnostic(DevDiagnostic {
            kind: DiagnosticKind::DetachedHandleAction,
            component,
            instance_id: String::new(),
            prop: verb,
            message: format!(
                "{component} handle.{verb}() was called before attaching to a mounted root."
            ),
        });
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = (component, verb);
    }
}

/// Emits [`DiagnosticKind::HandleOverlap`] when multiple roots attach to the
/// same overlay handle simultaneously.
#[inline]
pub fn warn_handle_overlap(component: &'static str) {
    #[cfg(debug_assertions)]
    {
        emit_diagnostic(DevDiagnostic {
            kind: DiagnosticKind::HandleOverlap,
            component,
            instance_id: String::new(),
            prop: "handle",
            message: format!(
                "Multiple {component} roots are attached to the same handle simultaneously."
            ),
        });
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = component;
    }
}

/// Emits [`DiagnosticKind::InvalidProp`] for invalid or out-of-range component
/// prop values.
#[inline]
pub fn warn_invalid_prop(component: &'static str, prop: &'static str, message: impl Into<String>) {
    #[cfg(debug_assertions)]
    {
        emit_diagnostic(DevDiagnostic {
            kind: DiagnosticKind::InvalidProp,
            component,
            instance_id: String::new(),
            prop,
            message: message.into(),
        });
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = (component, prop, message);
    }
}

#[cfg(debug_assertions)]
pub(crate) fn next_diagnostic_instance_id() -> u64 {
    NEXT_DIAG_INSTANCE_ID.fetch_add(1, Ordering::Relaxed)
}

#[cfg(debug_assertions)]
pub(crate) fn register_pending_controlled_check(key: u64, epoch: u64, check: Box<dyn FnOnce()>) {
    PENDING_CHECKS.with(|map| {
        map.borrow_mut().insert(key, (epoch, check));
    });

    if dioxus::core::Runtime::try_current().is_some() {
        let grace = REFUSAL_GRACE.with(Cell::get);
        crate::foundation::browser::spawn_root_task(async move {
            futures_timer::Delay::new(grace).await;
            run_pending_controlled_check(key, epoch);
        });
    }
}

#[cfg(debug_assertions)]
pub(crate) fn cancel_pending_controlled_check(key: u64) {
    let _ = PENDING_CHECKS.try_with(|map| {
        map.borrow_mut().remove(&key);
    });
}

#[cfg(debug_assertions)]
fn run_pending_controlled_check(key: u64, expected_epoch: u64) {
    let check = PENDING_CHECKS
        .try_with(|map| {
            let mut map = map.borrow_mut();
            let matches = map
                .get(&key)
                .is_some_and(|(epoch, _)| *epoch == expected_epoch);
            if matches {
                map.remove(&key).map(|(_, check)| check)
            } else {
                None
            }
        })
        .ok()
        .flatten();

    if let Some(check) = check {
        check();
    }
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::Duration;

    use super::{
        DEFAULT_CONTROLLED_REFUSAL_GRACE, DevDiagnostic, DiagnosticKind,
        cancel_pending_controlled_check, capture_diagnostics, controlled_refusal_grace,
        emit_diagnostic, flush_pending_controlled_checks, is_strict_mode,
        next_diagnostic_instance_id, register_pending_controlled_check, set_strict_mode,
        warn_detached_handle_action, warn_handle_overlap, warn_invalid_prop,
        with_controlled_refusal_grace,
    };

    #[test]
    fn capture_guard_nests_restores_and_takes_diagnostics() {
        let outer = capture_diagnostics();
        warn_invalid_prop("Progress", "min", "min must be finite");
        assert_eq!(outer.diagnostics().len(), 1);

        {
            let inner = capture_diagnostics();
            warn_detached_handle_action("Dialog", "open");
            warn_handle_overlap("Dialog");
            let drained = inner.take();
            assert_eq!(drained.len(), 2);
            assert_eq!(drained[0].kind, DiagnosticKind::DetachedHandleAction);
            assert_eq!(drained[0].prop, "open");
            assert_eq!(drained[1].kind, DiagnosticKind::HandleOverlap);
            assert!(inner.diagnostics().is_empty());
        }

        // Outer sink is restored after inner guard drops.
        emit_diagnostic(DevDiagnostic {
            kind: DiagnosticKind::ControlledRefusal,
            component: "Dialog",
            instance_id: String::from("Dialog-1"),
            prop: "open",
            message: String::from("refused"),
        });
        let outer_diags = outer.take();
        assert_eq!(outer_diags.len(), 2);
        assert_eq!(outer_diags[0].kind, DiagnosticKind::InvalidProp);
        assert_eq!(outer_diags[1].kind, DiagnosticKind::ControlledRefusal);
    }

    #[test]
    fn pending_checks_flush_and_cancel_deterministically() {
        let fired = Rc::new(Cell::new(0_usize));
        let key_a = next_diagnostic_instance_id();
        let key_b = next_diagnostic_instance_id();

        let fired_a = Rc::clone(&fired);
        register_pending_controlled_check(
            key_a,
            1,
            Box::new(move || fired_a.set(fired_a.get() + 1)),
        );
        let fired_b = Rc::clone(&fired);
        register_pending_controlled_check(
            key_b,
            1,
            Box::new(move || fired_b.set(fired_b.get() + 10)),
        );

        cancel_pending_controlled_check(key_b);
        flush_pending_controlled_checks();
        assert_eq!(fired.get(), 1);

        // Flushing again is a no-op.
        flush_pending_controlled_checks();
        assert_eq!(fired.get(), 1);
    }

    #[test]
    fn grace_duration_override_and_strict_mode_toggle() {
        let prev = with_controlled_refusal_grace(Duration::from_millis(25));
        assert_eq!(controlled_refusal_grace(), Duration::from_millis(25));
        with_controlled_refusal_grace(prev);
        assert_eq!(controlled_refusal_grace(), DEFAULT_CONTROLLED_REFUSAL_GRACE);

        assert!(!is_strict_mode());
        let guard = capture_diagnostics();
        set_strict_mode(true);
        let res = std::panic::catch_unwind(|| {
            warn_handle_overlap("Popover");
        });
        set_strict_mode(false);
        assert!(res.is_err());
        assert_eq!(guard.diagnostics().len(), 1);
    }
}

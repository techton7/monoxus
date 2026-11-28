//! Zero-bloat Dioxus agent runtime, borrow-free 3-phase action execution,
//! unmount abort safety, semantic-diff `meta_epoch` guard, and `<AgentProvider>`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::agent::{
    AbortResult, ActionDescriptor, ActionDispatch, AgentError, AgentHints, AgentMetadata,
    AgentRegistry, AgentReply, AgentRequest, AgentSurface, ApprovalDecision, ApprovalRequest,
    ComponentId, ComponentSnapshot, DispatchCompletion, RegistrySnapshot, SnapshotValue,
    StepOutcome, TurnState,
};
use ui_sdk::kernel::ChangeContext;

use super::boundary::{normalize_non_empty, use_agent_boundary_scope};

const MAX_STASHED_ABORTS: usize = 64;

/// Primitive-level metadata defaults layered onto resolved [`AgentMetadata`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SurfaceMetaDefaults {
    pub secret: bool,
    pub reveals: Vec<String>,
}

impl SurfaceMetaDefaults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_secret(mut self, secret: bool) -> Self {
        self.secret = secret;
        self
    }

    pub fn with_reveals(mut self, reveals: Vec<String>) -> Self {
        self.reveals = reveals;
        self
    }
}

/// Host execution outcome returned by an [`ActionDispatcher`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DispatchExecResult {
    pub canceled: bool,
    pub host_error: Option<String>,
    pub expected_field: Option<(String, SnapshotValue)>,
}

impl DispatchExecResult {
    pub fn ok() -> Self {
        Self::default()
    }

    pub fn with_canceled(mut self, canceled: bool) -> Self {
        self.canceled = canceled;
        self
    }

    pub fn with_host_error(mut self, msg: impl Into<String>) -> Self {
        self.host_error = Some(msg.into());
        self
    }

    pub fn with_expected(mut self, key: impl Into<String>, value: SnapshotValue) -> Self {
        self.expected_field = Some((key.into(), value));
        self
    }
}

/// Host callback that executes a policy-cleared [`ActionDispatch`] on the UI thread.
pub type ActionDispatcher = Rc<dyn Fn(&ActionDispatch) -> DispatchExecResult>;

/// Host callback that reads a component's live [`ComponentSnapshot`].
pub type SnapshotReader = Rc<dyn Fn() -> ComponentSnapshot>;

/// Optional async human confirmation hook for WebMCP or custom UI hosts.
pub type ApprovalHandler =
    Rc<dyn Fn(ApprovalRequest) -> Pin<Box<dyn Future<Output = ApprovalDecision>>>>;

#[derive(Clone, Copy, Debug)]
struct ActiveAgentDispatchScope {
    ctx: ChangeContext,
    canceled: bool,
}

thread_local! {
    static ACTIVE_AGENT_DISPATCH: RefCell<Option<ActiveAgentDispatchScope>> =
        const { RefCell::new(None) };
}

/// Returns the active [`ChangeContext`] while an agent action dispatcher is running.
pub fn current_change_context() -> Option<ChangeContext> {
    ACTIVE_AGENT_DISPATCH.with(|cell| cell.borrow().map(|scope| scope.ctx))
}

/// Marks the currently executing agent action as canceled by the host or transition gate.
pub fn cancel_current_agent_action() {
    ACTIVE_AGENT_DISPATCH.with(|cell| {
        if let Some(scope) = cell.borrow_mut().as_mut() {
            scope.canceled = true;
            scope.ctx.cancel();
        }
    });
}

/// Returns `true` if the currently executing agent action has been marked canceled.
pub fn is_current_agent_action_canceled() -> bool {
    ACTIVE_AGENT_DISPATCH.with(|cell| {
        cell.borrow()
            .is_some_and(|scope| scope.canceled || scope.ctx.is_canceled())
    })
}

struct AgentDispatchScopeGuard {
    previous: Option<ActiveAgentDispatchScope>,
}

impl AgentDispatchScopeGuard {
    fn enter(ctx: ChangeContext) -> Self {
        let previous = ACTIVE_AGENT_DISPATCH.with(|cell| {
            cell.borrow_mut()
                .replace(ActiveAgentDispatchScope { ctx, canceled: false })
        });
        Self { previous }
    }
}

impl Drop for AgentDispatchScopeGuard {
    fn drop(&mut self) {
        let _ = ACTIVE_AGENT_DISPATCH.try_with(|cell| {
            if let Ok(mut slot) = cell.try_borrow_mut() {
                *slot = self.previous;
            }
        });
    }
}

struct LiveComponentHost {
    dispatcher: ActionDispatcher,
    reader: SnapshotReader,
}

struct AgentBridgeInner {
    registry: AgentRegistry,
    hosts: BTreeMap<ComponentId, LiveComponentHost>,
    executing_target: Option<(u64, ComponentId)>,
    aborted_turns: BTreeMap<u64, AbortResult>,
    approval_handler: Option<ApprovalHandler>,
}

impl AgentBridgeInner {
    fn new() -> Self {
        Self {
            registry: AgentRegistry::new(),
            hosts: BTreeMap::new(),
            executing_target: None,
            aborted_turns: BTreeMap::new(),
            approval_handler: None,
        }
    }

    fn stash_abort(&mut self, result: AbortResult) {
        self.aborted_turns.insert(result.turn_id, result);
        while self.aborted_turns.len() > MAX_STASHED_ABORTS {
            let _ = self.aborted_turns.pop_first();
        }
    }

    fn take_aborted_by_approval(&mut self, approval_id: u64) -> Option<AbortResult> {
        let turn_id = self
            .aborted_turns
            .iter()
            .find_map(|(&tid, res)| (res.approval_id == Some(approval_id)).then_some(tid))?;
        self.aborted_turns.remove(&turn_id)
    }
}

fn metadata_semantically_equal(a: &AgentMetadata, b: &AgentMetadata) -> bool {
    a.kind == b.kind
        && a.label == b.label
        && a.row == b.row
        && a.sections == b.sections
        && a.hidden == b.hidden
        && a.secret == b.secret
        && a.confirm == b.confirm
        && a.inert == b.inert
        && a.disabled == b.disabled
        && a.reveals == b.reveals
}

enum RequestPhase1 {
    Immediate(AgentReply),
    Dispatch {
        turn_id: u64,
        dispatch: ActionDispatch,
        pre_snapshot: ComponentSnapshot,
        dispatcher: ActionDispatcher,
        reader: SnapshotReader,
    },
}

struct ActiveTurnAbortGuard<'a> {
    bridge: &'a AgentBridgeHandle,
    turn_id: u64,
    armed: bool,
}

impl<'a> ActiveTurnAbortGuard<'a> {
    fn new(bridge: &'a AgentBridgeHandle, turn_id: u64) -> Self {
        Self {
            bridge,
            turn_id,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for ActiveTurnAbortGuard<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Ok(mut inner) = self.bridge.inner.try_borrow_mut() {
            if inner
                .executing_target
                .is_some_and(|(tid, _)| tid == self.turn_id)
            {
                inner.executing_target = None;
            }
            if let Ok(abort_res) = inner
                .registry
                .abort_turn(self.turn_id, "dispatch interrupted before settlement")
            {
                inner.stash_abort(abort_res);
            }
        }
    }
}

/// Cloneable host bridge coordinating live Dioxus primitives with [`AgentRegistry`].
#[derive(Clone)]
pub struct AgentBridgeHandle {
    inner: Rc<RefCell<AgentBridgeInner>>,
}

impl Default for AgentBridgeHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for AgentBridgeHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for AgentBridgeHandle {}

impl fmt::Debug for AgentBridgeHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("AgentBridgeHandle");
        match self.inner.try_borrow() {
            Ok(inner) => s
                .field("components", &inner.registry.len())
                .field("turn_state", inner.registry.turn_state())
                .finish(),
            Err(_) => s.field("state", &"<borrowed>").finish(),
        }
    }
}

impl AgentBridgeHandle {
    /// Creates an empty [`AgentBridgeHandle`] backed by a fresh [`AgentRegistry`].
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(AgentBridgeInner::new())),
        }
    }

    /// Returns a redacted snapshot of visible components, off-page history, and turn state.
    pub fn registry_snapshot(&self) -> RegistrySnapshot {
        self.inner.borrow().registry.snapshot_full()
    }

    /// Executes an [`AgentRequest`] synchronously using borrow-free 3-phase dispatch.
    pub fn handle_request(&self, request: AgentRequest) -> AgentReply {
        let (turn_id, dispatch, pre_snapshot, dispatcher, reader) = match self.begin_request(request)
        {
            RequestPhase1::Immediate(reply) => return reply,
            RequestPhase1::Dispatch {
                turn_id,
                dispatch,
                pre_snapshot,
                dispatcher,
                reader,
            } => (turn_id, dispatch, pre_snapshot, dispatcher, reader),
        };

        let mut turn_guard = ActiveTurnAbortGuard::new(self, turn_id);

        // Phase 2: execute host dispatcher with scoped ChangeContext and ZERO RefCell borrows held.
        let (exec_result, scope_canceled) = Self::run_dispatcher(&dispatch, &dispatcher);

        turn_guard.disarm();

        // Phase 3: settle post-snapshot and complete dispatch.
        self.finish_dispatch(turn_id, pre_snapshot, exec_result, scope_canceled, &reader)
    }

    /// Executes an [`AgentRequest`] with an async frame settle (`oxidase::frame::next_frame()`)
    /// between Phase 2 dispatch and Phase 3 snapshot verification so controlled parents can
    /// re-render and synchronize state.
    pub async fn handle_request_settled(&self, request: AgentRequest) -> AgentReply {
        let (turn_id, dispatch, pre_snapshot, dispatcher, reader) = match self.begin_request(request)
        {
            RequestPhase1::Immediate(reply) => return reply,
            RequestPhase1::Dispatch {
                turn_id,
                dispatch,
                pre_snapshot,
                dispatcher,
                reader,
            } => (turn_id, dispatch, pre_snapshot, dispatcher, reader),
        };

        let mut turn_guard = ActiveTurnAbortGuard::new(self, turn_id);

        // Phase 2: execute host dispatcher with scoped ChangeContext and ZERO RefCell borrows held.
        let (exec_result, scope_canceled) = Self::run_dispatcher(&dispatch, &dispatcher);

        // Yield one host frame so controlled parents can re-render and sync snapshots.
        let _ = oxidase::frame::next_frame().await;

        turn_guard.disarm();

        // Phase 3: settle post-snapshot (or return AgentReply::Aborted if target unmounted).
        self.finish_dispatch(turn_id, pre_snapshot, exec_result, scope_canceled, &reader)
    }

    /// Decodes a JSON [`AgentRequest`], executes it synchronously, and encodes the [`AgentReply`].
    pub fn handle_json(&self, json: &str) -> String {
        let request: AgentRequest = match oxidase::serde_json::from_str(json) {
            Ok(req) => req,
            Err(err) => return invalid_json_reply(&err.to_string()),
        };
        let reply = self.handle_request(request);
        serialize_reply(&reply)
    }

    /// Decodes a JSON [`AgentRequest`], executes it with frame settlement, and encodes the [`AgentReply`].
    pub async fn handle_json_settled(&self, json: &str) -> String {
        let request: AgentRequest = match oxidase::serde_json::from_str(json) {
            Ok(req) => req,
            Err(err) => return invalid_json_reply(&err.to_string()),
        };
        let reply = self.handle_request_settled(request).await;
        serialize_reply(&reply)
    }

    /// Submits a human approval decision to resume an approval-gated turn.
    pub fn resume_approval(&self, approval_id: u64, decision: ApprovalDecision) -> AgentReply {
        self.handle_request(AgentRequest::ResumeApproval {
            approval_id,
            decision,
        })
    }

    /// Aborts whichever turn is currently active, returning the resulting [`AbortResult`] if any.
    pub fn abort_active_turn(&self, reason: impl Into<String>) -> Option<AbortResult> {
        let mut inner = self.inner.borrow_mut();
        let result = inner.registry.abort_active_turn(reason);
        if let Some(ref abort_res) = result {
            if inner
                .executing_target
                .is_some_and(|(tid, _)| tid == abort_res.turn_id)
            {
                inner.executing_target = None;
            }
            inner.stash_abort(abort_res.clone());
        }
        result
    }

    /// Installs or clears the optional async human confirmation hook.
    pub fn set_approval_handler(&self, handler: Option<ApprovalHandler>) {
        self.inner.borrow_mut().approval_handler = handler;
    }

    /// Returns a clone of the currently installed [`ApprovalHandler`], if any.
    pub fn approval_handler(&self) -> Option<ApprovalHandler> {
        self.inner.borrow().approval_handler.clone()
    }

    pub(crate) fn register_component(
        &self,
        meta: AgentMetadata,
        snapshot: ComponentSnapshot,
        actions: Vec<ActionDescriptor>,
        dispatcher: ActionDispatcher,
        reader: SnapshotReader,
    ) -> ComponentId {
        let mut inner = self.inner.borrow_mut();
        let id = inner.registry.register(meta, snapshot, actions);
        inner
            .hosts
            .insert(id, LiveComponentHost { dispatcher, reader });
        id
    }

    pub(crate) fn sync_component(
        &self,
        id: ComponentId,
        meta: AgentMetadata,
        snapshot: ComponentSnapshot,
        actions: Vec<ActionDescriptor>,
        dispatcher: ActionDispatcher,
        reader: SnapshotReader,
    ) -> bool {
        let mut inner = self.inner.borrow_mut();
        let (meta_changed, snapshot_changed, actions_changed) = {
            let Some(existing) = inner.registry.entry(id) else {
                return false;
            };
            (
                !metadata_semantically_equal(&existing.meta, &meta),
                existing.snapshot != snapshot,
                existing.actions != actions,
            )
        };

        inner
            .hosts
            .insert(id, LiveComponentHost { dispatcher, reader });

        if meta_changed {
            inner.registry.update_metadata(id, meta);
        }
        if snapshot_changed {
            inner.registry.update_snapshot(id, snapshot);
        }
        if actions_changed {
            inner.registry.update_actions(id, actions);
        }
        true
    }

    pub(crate) fn unregister(&self, id: ComponentId) {
        let mut inner = self.inner.borrow_mut();

        if let Some((turn_id, target_id)) = inner.executing_target
            && target_id == id
        {
            inner.executing_target = None;
            if let Ok(abort_res) = inner
                .registry
                .abort_turn(turn_id, "target component unmounted")
            {
                inner.stash_abort(abort_res);
            }
        } else if let TurnState::AwaitingApproval {
            turn_id, request, ..
        } = inner.registry.turn_state()
            && request.target_id == id
        {
            let turn_id = *turn_id;
            if let Ok(abort_res) = inner
                .registry
                .abort_turn(turn_id, "target component unmounted")
            {
                inner.stash_abort(abort_res);
            }
        }

        inner.hosts.remove(&id);
        inner.registry.unregister(id);
    }

    fn begin_request(&self, request: AgentRequest) -> RequestPhase1 {
        let mut inner = self.inner.borrow_mut();

        if let AgentRequest::ResumeApproval { approval_id, .. } = &request
            && !matches!(inner.registry.turn_state(), TurnState::AwaitingApproval { .. })
            && let Some(aborted) = inner.take_aborted_by_approval(*approval_id)
        {
            return RequestPhase1::Immediate(AgentReply::Aborted(aborted));
        }

        let reply = inner.registry.handle_request(request);

        match reply {
            AgentReply::Step(StepOutcome::Dispatch { turn_id, dispatch }) => {
                let Some(host) = inner.hosts.get(&dispatch.target_id) else {
                    if let Ok(abort_res) = inner
                        .registry
                        .abort_turn(turn_id, "target component host missing")
                    {
                        return RequestPhase1::Immediate(AgentReply::Aborted(abort_res));
                    }
                    return RequestPhase1::Immediate(AgentReply::Error(AgentError::NotFound));
                };

                let dispatcher = Rc::clone(&host.dispatcher);
                let reader = Rc::clone(&host.reader);
                let pre_snapshot = inner
                    .registry
                    .entry(dispatch.target_id)
                    .map(|entry| entry.snapshot.clone())
                    .unwrap_or_default();
                inner.executing_target = Some((turn_id, dispatch.target_id));

                RequestPhase1::Dispatch {
                    turn_id,
                    dispatch,
                    pre_snapshot,
                    dispatcher,
                    reader,
                }
            }
            AgentReply::Aborted(abort_res) => {
                if inner
                    .executing_target
                    .is_some_and(|(tid, _)| tid == abort_res.turn_id)
                {
                    inner.executing_target = None;
                    inner.stash_abort(abort_res.clone());
                }
                RequestPhase1::Immediate(AgentReply::Aborted(abort_res))
            }
            other => RequestPhase1::Immediate(other),
        }
    }

    fn run_dispatcher(
        dispatch: &ActionDispatch,
        dispatcher: &ActionDispatcher,
    ) -> (DispatchExecResult, bool) {
        let _guard = AgentDispatchScopeGuard::enter(dispatch.ctx);
        let result = (dispatcher)(dispatch);
        let scope_canceled = is_current_agent_action_canceled();
        (result, scope_canceled)
    }

    fn finish_dispatch(
        &self,
        turn_id: u64,
        pre_snapshot: ComponentSnapshot,
        exec_result: DispatchExecResult,
        scope_canceled: bool,
        reader: &SnapshotReader,
    ) -> AgentReply {
        // Short Phase 3a borrow: check if the turn was aborted during dispatch or frame wait.
        {
            let mut inner = self.inner.borrow_mut();
            if inner.executing_target.is_some_and(|(tid, _)| tid == turn_id) {
                inner.executing_target = None;
            }
            if let Some(abort_res) = inner.aborted_turns.remove(&turn_id) {
                return AgentReply::Aborted(abort_res);
            }
        }

        // Read post-snapshot with ZERO RefCell borrows held.
        let post_snapshot = (reader)();

        // Short Phase 3b borrow: settle completion in registry.
        let mut inner = self.inner.borrow_mut();
        if let Some(abort_res) = inner.aborted_turns.remove(&turn_id) {
            return AgentReply::Aborted(abort_res);
        }

        let mut completion = DispatchCompletion::new(pre_snapshot, post_snapshot);
        if let Some(host_error) = exec_result.host_error {
            completion = completion.with_host_error(host_error);
        }
        if exec_result.canceled || scope_canceled {
            completion = completion.with_canceled(true);
        }
        if let Some((key, value)) = exec_result.expected_field {
            completion = completion.with_expected(key, value);
        }

        match inner.registry.complete_dispatch_with(turn_id, completion) {
            Ok(outcome) => AgentReply::Step(outcome),
            Err(err) => AgentReply::Error(err),
        }
    }
}

fn serialize_reply(reply: &AgentReply) -> String {
    oxidase::serde_json::to_string(reply)
        .unwrap_or_else(|err| invalid_json_reply(&err.to_string()))
}

fn invalid_json_reply(message: &str) -> String {
    let encoded =
        oxidase::serde_json::to_string(message).unwrap_or_else(|_| String::from("\"invalid json\""));
    format!(r#"{{"InvalidRequest":{encoded}}}"#)
}

/// Dioxus context exposing the active [`AgentBridgeHandle`] to descendant primitives.
#[derive(Clone, Copy, PartialEq)]
pub struct AgentRuntimeContext(pub CopyValue<AgentBridgeHandle>);

/// Props for [`AgentProvider`].
#[derive(Clone, PartialEq, Props)]
pub struct AgentProviderProps {
    /// Optional caller-owned bridge handle. When omitted, a fresh handle is created.
    #[props(default)]
    pub handle: Option<AgentBridgeHandle>,
    /// Enables the `wasm32` WebMCP bridge (`navigator.modelContext` / `document.modelContext`).
    #[props(default = false)]
    pub webmcp: bool,
    /// Invoked once on initial mount with the active [`AgentBridgeHandle`].
    #[props(default)]
    pub on_ready: Option<EventHandler<AgentBridgeHandle>>,
    pub children: Element,
}

/// Root provider enabling agent registration for descendant `monoxus` primitives.
///
/// Renders zero wrapper DOM elements and aborts any active turn on unmount.
#[component]
pub fn AgentProvider(props: AgentProviderProps) -> Element {
    let initial = props.handle.clone().unwrap_or_else(AgentBridgeHandle::new);
    let mut handle_slot = use_hook(|| CopyValue::new(initial));

    if let Some(external) = props.handle {
        let should_swap = handle_slot.try_read().is_ok_and(|cur| *cur != external);
        if should_swap {
            handle_slot.set(external);
        }
    }

    let handle = handle_slot
        .try_read()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| AgentBridgeHandle::new());

    use_context_provider(|| AgentRuntimeContext(handle_slot));

    let mut ready_fired = use_hook(|| CopyValue::new(false));
    if !ready_fired.try_read().is_ok_and(|fired| *fired) {
        ready_fired.set(true);
        if let Some(on_ready) = props.on_ready {
            on_ready.call(handle.clone());
        }
    }

    super::webmcp::use_webmcp_bridge(handle, props.webmcp);

    use_drop(move || {
        if let Ok(bridge) = handle_slot.try_read() {
            bridge.abort_active_turn("agent provider unmounted");
        }
    });

    props.children
}

/// Returns the active [`AgentBridgeHandle`] if called inside an [`AgentProvider`].
pub fn use_agent_bridge() -> Option<AgentBridgeHandle> {
    try_consume_context::<AgentRuntimeContext>()
        .and_then(|ctx| ctx.0.try_read().ok().map(|guard| guard.clone()))
}

fn merge_reveals(defaults: &[String], hints_reveals: &[String]) -> Vec<String> {
    let mut merged = Vec::with_capacity(defaults.len() + hints_reveals.len());
    for item in defaults.iter().chain(hints_reveals.iter()) {
        if let Some(clean) = normalize_non_empty(Some(item.as_str()))
            && !merged.iter().any(|existing| existing == &clean)
        {
            merged.push(clean);
        }
    }
    merged
}

/// Registers and synchronizes a primitive's [`AgentSurface`] with the enclosing
/// [`AgentProvider`].
///
/// Hook call order is invariant across renders. When no [`AgentProvider`] is present,
/// none of `build_surface`, `build_dispatcher`, or `build_reader` are executed
/// (zero-allocation fast path).
pub fn use_agent_surface(
    hints: Option<&AgentHints>,
    defaults: SurfaceMetaDefaults,
    build_surface: impl FnOnce() -> AgentSurface,
    build_dispatcher: impl FnOnce() -> ActionDispatcher,
    build_reader: impl FnOnce() -> SnapshotReader,
) {
    let runtime_ctx = try_consume_context::<AgentRuntimeContext>();
    let mut registration: CopyValue<Option<(AgentBridgeHandle, ComponentId)>> =
        use_hook(|| CopyValue::new(None));

    use_drop(move || {
        let prev = registration.try_write().ok().and_then(|mut slot| slot.take());
        if let Some((bridge, id)) = prev {
            bridge.unregister(id);
        }
    });

    let Some(runtime_ctx) = runtime_ctx else {
        let prev = registration.try_write().ok().and_then(|mut slot| slot.take());
        if let Some((bridge, id)) = prev {
            bridge.unregister(id);
        }
        return;
    };

    let Ok(bridge) = runtime_ctx.0.try_read().map(|guard| guard.clone()) else {
        return;
    };

    let scope = use_agent_boundary_scope();
    let surface = build_surface();
    let dispatcher = build_dispatcher();
    let reader = build_reader();

    let mut meta = surface.resolve_metadata(scope.as_ref(), hints);
    meta.secret |= defaults.secret;
    meta.reveals = merge_reveals(&defaults.reveals, &meta.reveals);
    meta.confirm = normalize_non_empty(hints.and_then(|h| h.confirm.as_deref()))
        .or_else(|| normalize_non_empty(scope.as_ref().and_then(|s| s.confirm.as_deref())));

    let current_reg = registration.try_read().ok().and_then(|slot| slot.clone());
    match current_reg {
        Some((existing_bridge, existing_id)) if existing_bridge == bridge => {
            if !bridge.sync_component(
                existing_id,
                meta.clone(),
                surface.snapshot.clone(),
                surface.actions.clone(),
                Rc::clone(&dispatcher),
                Rc::clone(&reader),
            ) {
                let new_id = bridge.register_component(
                    meta,
                    surface.snapshot,
                    surface.actions,
                    dispatcher,
                    reader,
                );
                registration.set(Some((bridge, new_id)));
            }
        }
        Some((old_bridge, old_id)) => {
            old_bridge.unregister(old_id);
            let new_id = bridge.register_component(
                meta,
                surface.snapshot,
                surface.actions,
                dispatcher,
                reader,
            );
            registration.set(Some((bridge, new_id)));
        }
        None => {
            let new_id = bridge.register_component(
                meta,
                surface.snapshot,
                surface.actions,
                dispatcher,
                reader,
            );
            registration.set(Some((bridge, new_id)));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use dioxus::prelude::*;
    use ui_sdk::agent::{
        ActionCall, ActionDescriptor, AgentHints, AgentReply, AgentRequest, AgentSurface,
        ApprovalDecision, ComponentSnapshot, SnapshotValue, StepOutcome, TargetQuery, TurnState,
    };
    use ui_sdk::kernel::{ChangeContext, ChangeReason, ChangeSource};

    use super::{
        AgentBridgeHandle, AgentDispatchScopeGuard, AgentProvider, DispatchExecResult,
        SurfaceMetaDefaults, cancel_current_agent_action, current_change_context,
        is_current_agent_action_canceled, use_agent_bridge, use_agent_surface,
    };
    use crate::agent::boundary::AgentBoundary;

    #[test]
    fn zero_bloat_fast_path_skips_all_closures_without_provider() {
        let surface_calls = Rc::new(Cell::new(0_usize));
        let dispatcher_calls = Rc::new(Cell::new(0_usize));
        let reader_calls = Rc::new(Cell::new(0_usize));

        #[component]
        fn Leaf(
            surface_calls: Rc<Cell<usize>>,
            dispatcher_calls: Rc<Cell<usize>>,
            reader_calls: Rc<Cell<usize>>,
        ) -> Element {
            assert!(use_agent_bridge().is_none());
            use_agent_surface(
                None,
                SurfaceMetaDefaults::default(),
                || {
                    surface_calls.set(surface_calls.get() + 1);
                    AgentSurface::new("button", "Save")
                },
                || {
                    dispatcher_calls.set(dispatcher_calls.get() + 1);
                    Rc::new(|_| DispatchExecResult::ok())
                },
                || {
                    reader_calls.set(reader_calls.get() + 1);
                    Rc::new(ComponentSnapshot::new)
                },
            );
            rsx! { button { "Save" } }
        }

        let mut dom = VirtualDom::new_with_props(
            Leaf,
            LeafProps {
                surface_calls: Rc::clone(&surface_calls),
                dispatcher_calls: Rc::clone(&dispatcher_calls),
                reader_calls: Rc::clone(&reader_calls),
            },
        );
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert_eq!(surface_calls.get(), 0);
        assert_eq!(dispatcher_calls.get(), 0);
        assert_eq!(reader_calls.get(), 0);
    }

    #[test]
    fn reentrant_dispatch_can_read_snapshot_and_mount_siblings_without_borrow_panic() {
        let bridge = AgentBridgeHandle::new();
        let value = Rc::new(Cell::new(false));
        let bridge_for_dispatch = bridge.clone();
        let value_for_dispatch = Rc::clone(&value);
        let value_for_reader = Rc::clone(&value);

        let seen_source = Rc::new(Cell::new(None::<ChangeSource>));
        let seen_source_for_dispatch = Rc::clone(&seen_source);

        let surface = AgentSurface::new("switch", "Auto renew")
            .with_snapshot(
                ComponentSnapshot::new().with_field("checked", SnapshotValue::Bool(false)),
            )
            .with_action(ActionDescriptor::new("toggle", "Toggle switch", false));

        let meta = surface.resolve_metadata(None, None);
        bridge.register_component(
            meta,
            surface.snapshot,
            surface.actions,
            Rc::new(move |_dispatch| {
                // Verify scoped ChangeContext is live inside Phase 2.
                seen_source_for_dispatch.set(current_change_context().map(|ctx| ctx.source));

                // Re-entrantly inspect registry snapshot AND register a sibling while executing.
                let snap = bridge_for_dispatch.registry_snapshot();
                assert_eq!(snap.components.len(), 1);
                let sibling_surface = AgentSurface::new("button", "Sibling");
                let sibling_meta = sibling_surface.resolve_metadata(None, None);
                bridge_for_dispatch.register_component(
                    sibling_meta,
                    ComponentSnapshot::new(),
                    Vec::new(),
                    Rc::new(|_| DispatchExecResult::ok()),
                    Rc::new(ComponentSnapshot::new),
                );

                value_for_dispatch.set(true);
                DispatchExecResult::ok().with_expected("checked", SnapshotValue::Bool(true))
            }),
            Rc::new(move || {
                ComponentSnapshot::new()
                    .with_field("checked", SnapshotValue::Bool(value_for_reader.get()))
            }),
        );

        let reply = bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("Auto renew"),
            call: ActionCall::new("toggle"),
        });

        assert_eq!(seen_source.get(), Some(ChangeSource::Agent));
        assert!(current_change_context().is_none());
        assert!(matches!(reply, AgentReply::Step(StepOutcome::Applied { .. })));
        let final_snap = bridge.registry_snapshot();
        assert_eq!(final_snap.components.len(), 2);
        assert_eq!(final_snap.turn_state, TurnState::Idle);
    }

    #[test]
    fn unmount_during_executing_and_awaiting_approval_aborts_cleanly() {
        // Case 1: unmount while Executing -> returns AgentReply::Aborted and Idle state.
        let bridge = AgentBridgeHandle::new();
        let id_cell: Rc<Cell<Option<ui_sdk::agent::ComponentId>>> = Rc::new(Cell::new(None));
        let bridge_for_dispatch = bridge.clone();
        let id_for_dispatch = Rc::clone(&id_cell);

        let surface = AgentSurface::new("button", "Self destruct")
            .with_action(ActionDescriptor::new("press", "Press button", false));
        let id = bridge.register_component(
            surface.resolve_metadata(None, None),
            surface.snapshot,
            surface.actions,
            Rc::new(move |_| {
                if let Some(cid) = id_for_dispatch.get() {
                    bridge_for_dispatch.unregister(cid);
                }
                DispatchExecResult::ok()
            }),
            Rc::new(ComponentSnapshot::new),
        );
        id_cell.set(Some(id));

        let reply = bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("Self destruct"),
            call: ActionCall::new("press"),
        });
        assert!(matches!(reply, AgentReply::Aborted(_)));
        assert_eq!(bridge.registry_snapshot().turn_state, TurnState::Idle);

        // Case 2: unmount while AwaitingApproval -> aborts turn and resume returns Aborted.
        let confirm_surface = AgentSurface::new("button", "Delete account")
            .with_action(ActionDescriptor::new("press", "Delete", false));
        let confirm_hints = AgentHints::new().with_confirm("Delete permanently?");
        let cid2 = bridge.register_component(
            confirm_surface.resolve_metadata(None, Some(&confirm_hints)),
            confirm_surface.snapshot,
            confirm_surface.actions,
            Rc::new(|_| DispatchExecResult::ok()),
            Rc::new(ComponentSnapshot::new),
        );

        let approval_id = match bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("Delete account"),
            call: ActionCall::new("press"),
        }) {
            AgentReply::Step(StepOutcome::NeedsApproval { approval_id, .. }) => approval_id,
            other => panic!("expected NeedsApproval, got {other:?}"),
        };
        assert!(matches!(
            bridge.registry_snapshot().turn_state,
            TurnState::AwaitingApproval { .. }
        ));

        bridge.unregister(cid2);
        assert_eq!(bridge.registry_snapshot().turn_state, TurnState::Idle);

        let resumed = bridge.resume_approval(approval_id, ApprovalDecision::Allow);
        assert!(matches!(resumed, AgentReply::Aborted(_)));
    }

    #[test]
    fn semantic_diff_guard_preserves_meta_epoch_across_unchanged_rerenders() {
        let bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>> = Rc::new(RefCell::new(None));
        let render_tick_holder: Rc<RefCell<Option<Signal<u64>>>> = Rc::new(RefCell::new(None));

        #[component]
        fn ConfirmButton(tick: u64) -> Element {
            let _ = tick;
            let hints = AgentHints::new().with_confirm("Proceed?");
            use_agent_surface(
                Some(&hints),
                SurfaceMetaDefaults::new().with_reveals(vec![String::from("panel-1")]),
                || {
                    AgentSurface::new("button", "Submit")
                        .with_action(ActionDescriptor::new("press", "Submit", false))
                },
                || Rc::new(|_| DispatchExecResult::ok()),
                || Rc::new(ComponentSnapshot::new),
            );
            rsx! { button { "Submit" } }
        }

        #[component]
        fn App(
            bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>>,
            render_tick_holder: Rc<RefCell<Option<Signal<u64>>>>,
        ) -> Element {
            let tick = use_signal(|| 0_u64);
            *render_tick_holder.borrow_mut() = Some(tick);
            let slot = Rc::clone(&bridge_slot);
            rsx! {
                AgentProvider {
                    on_ready: move |h| {
                        *slot.borrow_mut() = Some(h);
                    },
                    AgentBoundary {
                        section: "Checkout",
                        ConfirmButton { tick: tick() }
                    }
                }
            }
        }

        let mut dom = VirtualDom::new_with_props(
            App,
            AppProps {
                bridge_slot: Rc::clone(&bridge_slot),
                render_tick_holder: Rc::clone(&render_tick_holder),
            },
        );
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        let bridge = bridge_slot.borrow().clone().expect("bridge ready");
        let initial_epoch = bridge.registry_snapshot().components[0].meta.meta_epoch;
        assert_eq!(initial_epoch, 0);

        // Trigger approval wait.
        let approval_id = match bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("Checkout > Submit"),
            call: ActionCall::new("press"),
        }) {
            AgentReply::Step(StepOutcome::NeedsApproval { approval_id, .. }) => approval_id,
            other => panic!("expected NeedsApproval, got {other:?}"),
        };

        // Re-render ConfirmButton with unchanged semantic metadata (e.g., approval dialog opens).
        let mut tick_sig = render_tick_holder.borrow().expect("tick signal");
        tick_sig.set(1);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        tick_sig.set(2);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert_eq!(
            bridge.registry_snapshot().components[0].meta.meta_epoch,
            initial_epoch
        );

        // Approval fingerprint remains valid and resumes cleanly.
        let resumed = bridge.resume_approval(approval_id, ApprovalDecision::Allow);
        assert!(matches!(
            resumed,
            AgentReply::Step(StepOutcome::Applied { .. })
        ));
    }

    #[test]
    fn scoped_cancel_and_panic_guard_cleanup_thread_local() {
        let ctx = ChangeContext::agent(ChangeReason::Toggle);
        let result = std::panic::catch_unwind(|| {
            let _guard = AgentDispatchScopeGuard::enter(ctx);
            assert_eq!(current_change_context().map(|c| c.source), Some(ChangeSource::Agent));
            assert!(!is_current_agent_action_canceled());
            cancel_current_agent_action();
            assert!(is_current_agent_action_canceled());
            panic!("simulated callback panic");
        });
        assert!(result.is_err());
        assert!(current_change_context().is_none());
        assert!(!is_current_agent_action_canceled());
    }

    #[test]
    fn handle_json_roundtrips_and_rejects_malformed_json_without_panicking() {
        let bridge = AgentBridgeHandle::new();
        let snap_req =
            oxidase::serde_json::to_string(&AgentRequest::Snapshot { include_off_page: false })
                .expect("serialize request");
        let reply_json = bridge.handle_json(&snap_req);
        let reply: AgentReply =
            oxidase::serde_json::from_str(&reply_json).expect("deserialize reply");
        assert!(matches!(reply, AgentReply::Snapshot(_)));

        let invalid = bridge.handle_json("{not-json");
        assert!(invalid.contains("InvalidRequest"));
    }
}

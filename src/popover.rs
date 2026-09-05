use std::{collections::HashMap, rc::Rc, time::Duration};

use dioxus::{document, document::Eval, prelude::*};
use futures_timer::Delay;

pub use crate::foundation::compose::{
    MountedHandle as PopoverMountedHandle, compose_part_event_handlers, compose_part_refs,
    project_as_child,
};

use crate::foundation::{
    browser::{
        DocumentDismissEvent, FloatingAutoUpdateEvent, acquire_scroll_lock, focus_element_by_id,
        focus_first_focusable, focus_mounted_handle, recv_document_dismiss_event,
        recv_floating_auto_update_event, release_scroll_lock, restore_focus_element_by_id,
        start_document_dismiss_monitor, start_floating_auto_update_monitor,
        stop_document_dismiss_monitor, stop_floating_auto_update_monitor,
    },
    overlay::{
        DismissLayer, FloatingLayer, FloatingPlacement, FocusGuards, FocusScope, GeometryVars,
        PlacementSide, PortalHost, Presence, Rect, Size,
    },
    shared::ScopeHandle,
    state::DataState,
};

pub const POPOVER_GEOMETRY_NAMESPACE: &str = "popover";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PopoverOpenFocusPolicy {
    FirstFocusable,
    Target(String),
    Suppress,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PopoverCloseFocusPolicy {
    Trigger,
    Target(String),
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopoverOutsideDismissBehavior {
    Dismiss,
    Ignore,
}

impl PopoverOutsideDismissBehavior {
    pub const fn dismisses(self) -> bool {
        matches!(self, Self::Dismiss)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopoverOutsideInteractionPolicy {
    pointer_down_outside: PopoverOutsideDismissBehavior,
    focus_outside: PopoverOutsideDismissBehavior,
}

impl PopoverOutsideInteractionPolicy {
    pub const fn new(
        pointer_down_outside: PopoverOutsideDismissBehavior,
        focus_outside: PopoverOutsideDismissBehavior,
    ) -> Self {
        Self {
            pointer_down_outside,
            focus_outside,
        }
    }

    pub const fn modal_default() -> Self {
        Self::new(
            PopoverOutsideDismissBehavior::Dismiss,
            PopoverOutsideDismissBehavior::Ignore,
        )
    }

    pub const fn non_modal_default() -> Self {
        Self::new(
            PopoverOutsideDismissBehavior::Dismiss,
            PopoverOutsideDismissBehavior::Dismiss,
        )
    }

    pub const fn pointer_down_outside(&self) -> PopoverOutsideDismissBehavior {
        self.pointer_down_outside
    }

    pub const fn focus_outside(&self) -> PopoverOutsideDismissBehavior {
        self.focus_outside
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopoverScrollLockPolicy {
    enabled: bool,
    restore_delay: Option<u64>,
}

impl PopoverScrollLockPolicy {
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled,
            restore_delay: None,
        }
    }

    pub const fn enabled() -> Self {
        Self::new(true)
    }

    pub const fn disabled() -> Self {
        Self::new(false)
    }

    pub const fn with_restore_delay(mut self, restore_delay: Option<u64>) -> Self {
        self.restore_delay = restore_delay;
        self
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub const fn restore_delay(&self) -> Option<u64> {
        self.restore_delay
    }
}

pub const POPOVER_PARTS: [PopoverPart; 7] = [
    PopoverPart::Root,
    PopoverPart::Trigger,
    PopoverPart::Portal,
    PopoverPart::Content,
    PopoverPart::Arrow,
    PopoverPart::Anchor,
    PopoverPart::Close,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PopoverPart {
    Root,
    Trigger,
    Portal,
    Content,
    Arrow,
    Anchor,
    Close,
}

impl PopoverPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Portal => "portal",
            Self::Content => "content",
            Self::Arrow => "arrow",
            Self::Anchor => "anchor",
            Self::Close => "close",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PopoverStateRequest {
    Open,
    Close,
    Toggle,
}

impl PopoverStateRequest {
    pub const fn next_open(self, current_open: bool) -> bool {
        match self {
            Self::Open => true,
            Self::Close => false,
            Self::Toggle => !current_open,
        }
    }

    pub fn data_state(self, current_open: bool) -> DataState {
        if self.next_open(current_open) {
            DataState::Open
        } else {
            DataState::Closed
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    anchor_id: String,
    content_id: String,
    arrow_id: String,
    close_id: String,
}

impl PopoverRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            anchor_id: scope.qualify("anchor"),
            content_id: scope.qualify("content"),
            arrow_id: scope.qualify("arrow"),
            close_id: scope.qualify("close"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn trigger_id(&self) -> &str {
        &self.trigger_id
    }

    pub fn anchor_id(&self) -> &str {
        &self.anchor_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn arrow_id(&self) -> &str {
        &self.arrow_id
    }

    pub fn close_id(&self) -> &str {
        &self.close_id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PopoverLifecycle {
    portal_host: PortalHost,
    presence: Presence,
    focus_scope: FocusScope<String>,
    focus_guards: FocusGuards<String>,
    dismiss_layer: DismissLayer<String>,
    floating: FloatingLayer,
    modal: bool,
    open_focus_policy: PopoverOpenFocusPolicy,
    close_focus_policy: PopoverCloseFocusPolicy,
    default_restore_focus_target: String,
    scroll_lock_policy: PopoverScrollLockPolicy,
    outside_interaction_policy: PopoverOutsideInteractionPolicy,
}

impl PopoverLifecycle {
    pub fn new(relationships: &PopoverRelationships, open: bool) -> Self {
        Self::new_with_modal(relationships, open, false)
    }

    pub fn new_modal(relationships: &PopoverRelationships, open: bool) -> Self {
        Self::new_with_modal(relationships, open, true)
    }

    fn new_with_modal(relationships: &PopoverRelationships, open: bool, modal: bool) -> Self {
        let outside_interaction_policy = Self::default_outside_interaction_policy(modal);
        let scroll_lock_policy = Self::default_scroll_lock_policy(modal);
        let default_restore_focus_target = relationships.trigger_id().to_owned();
        let mut focus_scope = FocusScope::new(relationships.content_id().to_owned())
            .with_trap_focus(modal)
            .with_loop_focus(modal);
        focus_scope.capture_restore_target(Some(default_restore_focus_target.clone()));
        focus_scope.set_autofocus_enabled(true);

        let mut dismiss_layer =
            DismissLayer::new(relationships.content_id().to_owned()).with_modal(modal);
        dismiss_layer.set_pointer_down_outside_dismiss(
            outside_interaction_policy
                .pointer_down_outside()
                .dismisses(),
        );
        dismiss_layer
            .set_focus_outside_dismiss(outside_interaction_policy.focus_outside().dismisses());

        Self {
            portal_host: PortalHost::default_host(),
            presence: Presence::new(open).with_retained_mount(true),
            focus_scope,
            focus_guards: FocusGuards::new(
                relationships.scope().qualify("focus-guard-before"),
                relationships.scope().qualify("focus-guard-after"),
            ),
            dismiss_layer,
            floating: FloatingLayer::new(PlacementSide::Bottom)
                .with_side_offset(4.0)
                .with_namespace(POPOVER_GEOMETRY_NAMESPACE),
            modal,
            open_focus_policy: PopoverOpenFocusPolicy::FirstFocusable,
            close_focus_policy: PopoverCloseFocusPolicy::Trigger,
            default_restore_focus_target,
            scroll_lock_policy,
            outside_interaction_policy,
        }
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.portal_host = portal_host;
        self
    }

    pub fn with_floating(mut self, floating: FloatingLayer) -> Self {
        self.floating = floating.with_namespace(POPOVER_GEOMETRY_NAMESPACE);
        self
    }

    pub fn portal_host(&self) -> &PortalHost {
        &self.portal_host
    }

    pub fn presence(&self) -> &Presence {
        &self.presence
    }

    pub fn presence_mut(&mut self) -> &mut Presence {
        &mut self.presence
    }

    pub fn focus_scope(&self) -> &FocusScope<String> {
        &self.focus_scope
    }

    pub fn focus_scope_mut(&mut self) -> &mut FocusScope<String> {
        &mut self.focus_scope
    }

    pub fn focus_guards(&self) -> &FocusGuards<String> {
        &self.focus_guards
    }

    pub fn focus_guards_mut(&mut self) -> &mut FocusGuards<String> {
        &mut self.focus_guards
    }

    pub fn dismiss_layer(&self) -> &DismissLayer<String> {
        &self.dismiss_layer
    }

    pub fn dismiss_layer_mut(&mut self) -> &mut DismissLayer<String> {
        &mut self.dismiss_layer
    }

    pub fn floating(&self) -> &FloatingLayer {
        &self.floating
    }

    pub const fn is_modal(&self) -> bool {
        self.modal
    }

    pub fn open_focus_policy(&self) -> &PopoverOpenFocusPolicy {
        &self.open_focus_policy
    }

    pub fn close_focus_policy(&self) -> &PopoverCloseFocusPolicy {
        &self.close_focus_policy
    }

    pub const fn scroll_lock_policy(&self) -> &PopoverScrollLockPolicy {
        &self.scroll_lock_policy
    }

    pub const fn outside_interaction_policy(&self) -> &PopoverOutsideInteractionPolicy {
        &self.outside_interaction_policy
    }

    pub fn set_autofocus_target(&mut self, target: Option<String>) {
        match target {
            Some(target) => self.set_open_focus_policy(PopoverOpenFocusPolicy::Target(target)),
            None => self.set_open_focus_policy(PopoverOpenFocusPolicy::FirstFocusable),
        }
    }

    pub fn capture_restore_target(&mut self, target: Option<String>) {
        match target {
            Some(target) => self.set_close_focus_policy(PopoverCloseFocusPolicy::Target(target)),
            None => self.set_close_focus_policy(PopoverCloseFocusPolicy::None),
        }
    }

    pub fn register_branch(&mut self, branch: impl Into<String>) -> bool {
        let branch = branch.into();
        let focus_registered = self.focus_scope.register_branch(branch.clone());
        let dismiss_registered = self.dismiss_layer.register_branch(branch);
        debug_assert_eq!(focus_registered, dismiss_registered);
        focus_registered && dismiss_registered
    }

    pub(crate) fn set_modal(&mut self, modal: bool) {
        let previous_modal = self.modal;
        self.modal = modal;
        self.focus_scope.set_trap_focus(modal);
        self.focus_scope.set_loop_focus(modal);
        self.dismiss_layer.set_modal(modal);

        if self.outside_interaction_policy
            == Self::default_outside_interaction_policy(previous_modal)
        {
            self.set_outside_interaction_policy(Self::default_outside_interaction_policy(modal));
        }

        if self.scroll_lock_policy == Self::default_scroll_lock_policy(previous_modal) {
            self.scroll_lock_policy = Self::default_scroll_lock_policy(modal);
        }
    }

    pub(crate) fn set_open_focus_policy(&mut self, policy: PopoverOpenFocusPolicy) {
        match &policy {
            PopoverOpenFocusPolicy::FirstFocusable => {
                self.focus_scope.set_autofocus_enabled(true);
                self.focus_scope.set_autofocus_target(None);
            }
            PopoverOpenFocusPolicy::Target(target) => {
                self.focus_scope.set_autofocus_enabled(true);
                self.focus_scope.set_autofocus_target(Some(target.clone()));
            }
            PopoverOpenFocusPolicy::Suppress => {
                self.focus_scope.set_autofocus_enabled(false);
                self.focus_scope.set_autofocus_target(None);
            }
        }

        self.open_focus_policy = policy;
    }

    pub(crate) fn set_close_focus_policy(&mut self, policy: PopoverCloseFocusPolicy) {
        match &policy {
            PopoverCloseFocusPolicy::Trigger => self
                .focus_scope
                .capture_restore_target(Some(self.default_restore_focus_target.clone())),
            PopoverCloseFocusPolicy::Target(target) => self
                .focus_scope
                .capture_restore_target(Some(target.clone())),
            PopoverCloseFocusPolicy::None => self.focus_scope.capture_restore_target(None),
        }

        self.close_focus_policy = policy;
    }

    pub(crate) fn set_scroll_lock_policy(&mut self, policy: PopoverScrollLockPolicy) {
        self.scroll_lock_policy = policy;
    }

    pub(crate) fn set_outside_interaction_policy(
        &mut self,
        policy: PopoverOutsideInteractionPolicy,
    ) {
        self.dismiss_layer
            .set_pointer_down_outside_dismiss(policy.pointer_down_outside().dismisses());
        self.dismiss_layer
            .set_focus_outside_dismiss(policy.focus_outside().dismisses());
        self.outside_interaction_policy = policy;
    }

    fn default_outside_interaction_policy(modal: bool) -> PopoverOutsideInteractionPolicy {
        if modal {
            PopoverOutsideInteractionPolicy::modal_default()
        } else {
            PopoverOutsideInteractionPolicy::non_modal_default()
        }
    }

    fn default_scroll_lock_policy(modal: bool) -> PopoverScrollLockPolicy {
        if modal {
            PopoverScrollLockPolicy::enabled()
        } else {
            PopoverScrollLockPolicy::disabled()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Popover {
    relationships: PopoverRelationships,
    lifecycle: PopoverLifecycle,
    open: bool,
}

impl Popover {
    pub fn new(scope: ScopeHandle, open: bool) -> Self {
        Self::new_with_modal(scope, open, false)
    }

    pub fn new_modal(scope: ScopeHandle, open: bool) -> Self {
        Self::new_with_modal(scope, open, true)
    }

    fn new_with_modal(scope: ScopeHandle, open: bool, modal: bool) -> Self {
        let relationships = PopoverRelationships::new(scope);
        let lifecycle = if modal {
            PopoverLifecycle::new_modal(&relationships, open)
        } else {
            PopoverLifecycle::new(&relationships, open)
        };

        Self {
            relationships,
            lifecycle,
            open,
        }
    }

    pub const fn parts() -> &'static [PopoverPart] {
        &POPOVER_PARTS
    }

    pub const fn is_open(&self) -> bool {
        self.open
    }

    pub fn data_state(&self) -> DataState {
        if self.open {
            DataState::Open
        } else {
            DataState::Closed
        }
    }

    pub fn relationships(&self) -> &PopoverRelationships {
        &self.relationships
    }

    pub fn lifecycle(&self) -> &PopoverLifecycle {
        &self.lifecycle
    }

    pub fn lifecycle_mut(&mut self) -> &mut PopoverLifecycle {
        &mut self.lifecycle
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.lifecycle = self.lifecycle.with_portal_host(portal_host);
        self
    }

    pub fn with_floating(mut self, floating: FloatingLayer) -> Self {
        self.lifecycle = self.lifecycle.with_floating(floating);
        self
    }

    pub fn with_modal(mut self, modal: bool) -> Self {
        self.lifecycle.set_modal(modal);
        self
    }

    pub fn with_open_focus_policy(mut self, policy: PopoverOpenFocusPolicy) -> Self {
        self.lifecycle.set_open_focus_policy(policy);
        self
    }

    pub fn with_close_focus_policy(mut self, policy: PopoverCloseFocusPolicy) -> Self {
        self.lifecycle.set_close_focus_policy(policy);
        self
    }

    pub fn with_scroll_lock_policy(mut self, policy: PopoverScrollLockPolicy) -> Self {
        self.lifecycle.set_scroll_lock_policy(policy);
        self
    }

    pub fn with_outside_interaction_policy(
        mut self,
        policy: PopoverOutsideInteractionPolicy,
    ) -> Self {
        self.lifecycle.set_outside_interaction_policy(policy);
        self
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.lifecycle.floating().geometry_vars(anchor, content)
    }

    pub fn root(&self) -> PopoverRootAttributes {
        PopoverRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_state: self.data_state(),
        }
    }

    pub fn trigger(&self) -> PopoverTriggerAttributes {
        PopoverTriggerAttributes {
            id: self.relationships.trigger_id().to_owned(),
            aria_controls: self.relationships.content_id().to_owned(),
            aria_expanded: self.is_open(),
            data_state: self.data_state(),
            open_request: PopoverStateRequest::Toggle,
        }
    }

    pub fn anchor(&self) -> PopoverAnchorAttributes {
        PopoverAnchorAttributes {
            id: self.relationships.anchor_id().to_owned(),
        }
    }

    pub fn portal(&self) -> PopoverPortalAttributes {
        PopoverPortalAttributes {
            host: self.lifecycle.portal_host().clone(),
        }
    }

    pub fn content(&self) -> PopoverContentAttributes {
        PopoverContentAttributes {
            id: self.relationships.content_id().to_owned(),
            role: "dialog",
            aria_modal: self.lifecycle.dismiss_layer().is_modal(),
            data_state: self.data_state(),
            data_side: self.lifecycle.floating().data_side(),
            data_align: self.lifecycle.floating().data_align(),
        }
    }

    pub fn arrow(&self) -> PopoverArrowAttributes {
        PopoverArrowAttributes {
            id: self.relationships.arrow_id().to_owned(),
            data_state: self.data_state(),
            data_side: self.lifecycle.floating().data_side(),
            data_align: self.lifecycle.floating().data_align(),
        }
    }

    pub fn close(&self) -> PopoverCloseAttributes {
        PopoverCloseAttributes {
            id: self.relationships.close_id().to_owned(),
            data_state: self.data_state(),
            close_request: PopoverStateRequest::Close,
        }
    }
}

type PopoverOpenChangeHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Copy)]
struct PopoverRuntimeState {
    trigger_handle: Signal<Option<PopoverMountedHandle>>,
    anchor_handle: Signal<Option<PopoverMountedHandle>>,
    content_handle: Signal<Option<PopoverMountedHandle>>,
    placement: Signal<Option<FloatingPlacement>>,
    focus_targets: Signal<HashMap<String, PopoverMountedHandle>>,
    position_loop_token: Signal<u64>,
    position_monitor: Signal<Option<Eval>>,
    dismiss_loop_token: Signal<u64>,
    dismiss_monitor: Signal<Option<Eval>>,
    last_open: Signal<bool>,
    pending_open_focus: Signal<bool>,
    scroll_lock_held: Signal<bool>,
}

#[derive(Clone)]
pub struct PopoverRuntime {
    popover: Popover,
    on_open_change: PopoverOpenChangeHandler,
    state: PopoverRuntimeState,
}

pub fn use_popover_runtime<F>(popover: Popover, on_open_change: F) -> PopoverRuntime
where
    F: Fn(bool) + 'static,
{
    let synced_open_change: PopoverOpenChangeHandler = Rc::new(on_open_change);
    let state = PopoverRuntimeState {
        trigger_handle: use_signal(|| None),
        anchor_handle: use_signal(|| None),
        content_handle: use_signal(|| None),
        placement: use_signal(|| None),
        focus_targets: use_signal(HashMap::new),
        position_loop_token: use_signal(|| 0),
        position_monitor: use_signal(|| Option::<Eval>::None),
        dismiss_loop_token: use_signal(|| 0),
        dismiss_monitor: use_signal(|| Option::<Eval>::None),
        last_open: use_signal(|| popover.is_open()),
        pending_open_focus: use_signal(|| popover.is_open()),
        scroll_lock_held: use_signal(|| false),
    };
    let effect_state = state;
    let cleanup_state = state;
    let cleanup_key = popover.relationships().root_id().to_owned();
    let effect_open_change = Rc::clone(&synced_open_change);

    use_effect(use_reactive((&popover,), move |(popover,)| {
        sync_popover_runtime(&popover, effect_state, Rc::clone(&effect_open_change));
    }));

    dioxus::core::use_drop(move || {
        advance_popover_token(cleanup_state.position_loop_token);
        stop_popover_position_monitor(cleanup_state);
        advance_popover_token(cleanup_state.dismiss_loop_token);
        stop_popover_dismiss_monitor(cleanup_state);
        if *cleanup_state.scroll_lock_held.peek() {
            release_scroll_lock(&cleanup_key, None);
        }
    });

    PopoverRuntime {
        popover,
        on_open_change: synced_open_change,
        state,
    }
}

impl PopoverRuntime {
    pub fn popover(&self) -> &Popover {
        &self.popover
    }

    pub const fn is_open(&self) -> bool {
        self.popover.is_open()
    }

    pub fn data_state(&self) -> DataState {
        self.popover.data_state()
    }

    pub fn relationships(&self) -> &PopoverRelationships {
        self.popover.relationships()
    }

    pub fn lifecycle(&self) -> &PopoverLifecycle {
        self.popover.lifecycle()
    }

    pub fn root(&self) -> PopoverRootAttributes {
        self.popover.root()
    }

    pub fn trigger(&self) -> PopoverTriggerAttributes {
        self.popover.trigger()
    }

    pub fn anchor(&self) -> PopoverAnchorAttributes {
        self.popover.anchor()
    }

    pub fn portal(&self) -> PopoverPortalAttributes {
        self.popover.portal()
    }

    pub fn content(&self) -> PopoverContentAttributes {
        let mut content = self.popover.content();
        if let Some(placement) = self.placement() {
            content.data_side = placement.side().as_str();
            content.data_align = placement.align().as_str();
        }
        content
    }

    pub fn arrow(&self) -> PopoverArrowAttributes {
        let mut arrow = self.popover.arrow();
        if let Some(placement) = self.placement() {
            arrow.data_side = placement.side().as_str();
            arrow.data_align = placement.align().as_str();
        }
        arrow
    }

    pub fn close(&self) -> PopoverCloseAttributes {
        self.popover.close()
    }

    pub fn geometry_vars(&self, anchor: Rect, content: Size) -> GeometryVars {
        self.popover.geometry_vars(anchor, content)
    }

    pub fn trigger_handle(&self) -> Option<PopoverMountedHandle> {
        self.state.trigger_handle.cloned()
    }

    pub fn anchor_handle(&self) -> Option<PopoverMountedHandle> {
        self.state.anchor_handle.cloned()
    }

    pub fn content_handle(&self) -> Option<PopoverMountedHandle> {
        self.state.content_handle.cloned()
    }

    pub fn placement(&self) -> Option<FloatingPlacement> {
        self.state.placement.cloned()
    }

    pub fn mounted_focus_target(&self, id: &str) -> Option<PopoverMountedHandle> {
        self.state
            .focus_targets
            .with_peek(|targets| targets.get(id).cloned())
    }

    pub fn request_state(&self, request: PopoverStateRequest) -> bool {
        let next_open = request.next_open(self.is_open());
        if next_open != self.is_open() {
            (self.on_open_change)(next_open);
        }
        next_open
    }

    pub fn open(&self) {
        self.request_state(PopoverStateRequest::Open);
    }

    pub fn close_now(&self) {
        self.request_state(PopoverStateRequest::Close);
    }

    pub fn toggle(&self) {
        self.request_state(PopoverStateRequest::Toggle);
    }

    pub fn capture_trigger(&self, mounted: PopoverMountedHandle) {
        let mut trigger_handle = self.state.trigger_handle;
        trigger_handle.set(Some(mounted.clone()));
        self.capture_focus_target(self.relationships().trigger_id().to_owned(), mounted);
        self.refresh_live_placement();
    }

    pub fn capture_anchor(&self, mounted: PopoverMountedHandle) {
        let mut anchor_handle = self.state.anchor_handle;
        anchor_handle.set(Some(mounted));
        self.refresh_live_placement();
    }

    pub fn capture_content(&self, mounted: PopoverMountedHandle) {
        let mut content_handle = self.state.content_handle;
        content_handle.set(Some(mounted.clone()));
        self.capture_focus_target(self.relationships().content_id().to_owned(), mounted);
        self.refresh_live_placement();
        self.try_apply_pending_open_focus();
    }

    pub fn capture_close(&self, mounted: PopoverMountedHandle) {
        self.capture_focus_target(self.relationships().close_id().to_owned(), mounted);
    }

    pub fn capture_focus_target(&self, id: impl Into<String>, mounted: PopoverMountedHandle) {
        let id = id.into();
        let mut focus_targets = self.state.focus_targets;
        focus_targets.with_mut(|targets| {
            targets.insert(id, mounted);
        });
        self.try_apply_pending_open_focus();
    }

    pub fn mount_trigger(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_trigger(event.data())
    }

    pub fn mount_anchor(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_anchor(event.data())
    }

    pub fn mount_content(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_content(event.data())
    }

    pub fn mount_close(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_close(event.data())
    }

    pub fn mount_focus_target(&self, id: impl Into<String>) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        let id = id.into();
        move |event| runtime.capture_focus_target(id.clone(), event.data())
    }

    pub fn inside_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        move |event| event.stop_propagation()
    }

    pub fn inside_focus_in(&self) -> impl FnMut(Event<FocusData>) + 'static {
        move |event| event.stop_propagation()
    }

    pub fn outside_pointer_down(&self) -> impl FnMut(Event<PointerData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if runtime.dismiss_outside_pointer(None) {
                event.stop_propagation();
            }
        }
    }

    pub fn outside_focus_in(&self) -> impl FnMut(Event<FocusData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if runtime.dismiss_outside_focus(None) {
                event.stop_propagation();
            }
        }
    }

    pub fn escape_keydown(&self) -> impl FnMut(Event<KeyboardData>) + 'static {
        let runtime = self.clone();
        move |event| {
            if event.key().to_string() == "Escape" && runtime.dismiss_escape() {
                event.prevent_default();
                event.stop_propagation();
            }
        }
    }

    pub fn dismiss_escape(&self) -> bool {
        let stack = self.dismiss_stack();
        if self
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_escape(&stack)
        {
            self.close_now();
            return true;
        }

        false
    }

    pub fn dismiss_outside_pointer(&self, target: Option<&str>) -> bool {
        let target = target.map(|target| target.to_owned());
        let stack = self.dismiss_stack();
        if self
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_outside_pointer(target.as_ref(), &stack)
        {
            self.close_now();
            return true;
        }

        false
    }

    pub fn dismiss_outside_focus(&self, target: Option<&str>) -> bool {
        let target = target.map(|target| target.to_owned());
        let stack = self.dismiss_stack();
        if self
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_outside_focus(target.as_ref(), &stack)
        {
            self.close_now();
            return true;
        }

        false
    }

    pub fn trigger_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| runtime.toggle()
    }

    pub fn close_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| runtime.close_now()
    }

    fn refresh_live_placement(&self) {
        if !self.is_open() {
            return;
        }

        let runtime = self.clone();
        spawn(async move {
            if let Err(error) = measure_popover_placement(runtime.popover(), runtime.state).await {
                eprintln!(
                    "monoxus popover runtime could not refresh placement for {}: {error}",
                    runtime.relationships().root_id(),
                );
            }
        });
    }

    fn dismiss_stack(&self) -> Vec<String> {
        vec![self.relationships().content_id().to_owned()]
    }

    fn try_apply_pending_open_focus(&self) {
        schedule_pending_open_focus(self.popover.clone(), self.state);
    }
}

fn schedule_pending_open_focus(popover: Popover, state: PopoverRuntimeState) {
    if !popover.is_open() || !*state.pending_open_focus.peek() {
        return;
    }

    spawn(async move {
        loop {
            if !popover.is_open() || !*state.pending_open_focus.peek() {
                return;
            }

            if apply_popover_open_focus(&popover, state).await {
                let mut pending_open_focus = state.pending_open_focus;
                pending_open_focus.set(false);
                return;
            }

            Delay::new(Duration::from_millis(16)).await;
        }
    });
}

fn sync_popover_runtime(
    popover: &Popover,
    state: PopoverRuntimeState,
    on_open_change: PopoverOpenChangeHandler,
) {
    let is_open = popover.is_open();
    let was_open = *state.last_open.peek();
    let should_hold_scroll_lock = is_open
        && popover.lifecycle().is_modal()
        && popover.lifecycle().scroll_lock_policy().is_enabled();
    let is_holding_scroll_lock = *state.scroll_lock_held.peek();

    if is_open && !was_open && !*state.pending_open_focus.peek() {
        let mut pending_open_focus = state.pending_open_focus;
        pending_open_focus.set(true);
        schedule_pending_open_focus(popover.clone(), state);
    }

    sync_popover_document_dismissal(popover, state, on_open_change);
    sync_popover_positioning(popover, state);

    if should_hold_scroll_lock && !is_holding_scroll_lock {
        acquire_scroll_lock(popover.relationships().root_id());
        let mut scroll_lock_held = state.scroll_lock_held;
        scroll_lock_held.set(true);
    } else if !should_hold_scroll_lock && is_holding_scroll_lock {
        let restore_delay = if was_open && !is_open {
            popover.lifecycle().scroll_lock_policy().restore_delay()
        } else {
            None
        };
        release_scroll_lock(popover.relationships().root_id(), restore_delay);
        let mut scroll_lock_held = state.scroll_lock_held;
        scroll_lock_held.set(false);
    }

    if !is_open {
        if was_open {
            restore_popover_close_focus(popover, state);
        }

        if *state.pending_open_focus.peek() {
            let mut pending_open_focus = state.pending_open_focus;
            pending_open_focus.set(false);
        }
    }

    if was_open != is_open {
        let mut last_open = state.last_open;
        last_open.set(is_open);
    }
}

fn sync_popover_document_dismissal(
    popover: &Popover,
    state: PopoverRuntimeState,
    on_open_change: PopoverOpenChangeHandler,
) {
    stop_popover_dismiss_monitor(state);

    if !popover.is_open() {
        advance_popover_token(state.dismiss_loop_token);
        return;
    }

    let dismiss_loop_token = advance_popover_token(state.dismiss_loop_token);
    let popover = popover.clone();
    let monitor = start_document_dismiss_monitor();
    let mut dismiss_monitor = state.dismiss_monitor;
    dismiss_monitor.set(Some(monitor));

    spawn(async move {
        let mut monitor = monitor;

        loop {
            if *state.dismiss_loop_token.peek() != dismiss_loop_token {
                break;
            }

            match recv_document_dismiss_event(&mut monitor).await {
                Ok(DocumentDismissEvent::Stopped) => break,
                Ok(event) => {
                    if should_dismiss_popover_from_document_event(&popover, &event) {
                        (on_open_change)(false);
                        break;
                    }
                }
                Err(error) => {
                    if *state.dismiss_loop_token.peek() == dismiss_loop_token {
                        eprintln!(
                            "monoxus popover runtime could not read dismissal events for {}: {error}",
                            popover.relationships().root_id(),
                        );
                    }
                    break;
                }
            }
        }
    });
}

fn should_dismiss_popover_from_document_event(
    popover: &Popover,
    event: &DocumentDismissEvent,
) -> bool {
    let dismiss_stack = popover_dismiss_stack(popover);
    match event {
        DocumentDismissEvent::PointerDown { path_ids } => {
            popover_document_path_is_outside(popover, path_ids)
                && popover
                    .lifecycle()
                    .dismiss_layer()
                    .should_dismiss_outside_pointer(None, &dismiss_stack)
        }
        DocumentDismissEvent::FocusIn { path_ids } => {
            popover_document_path_is_outside(popover, path_ids)
                && popover
                    .lifecycle()
                    .dismiss_layer()
                    .should_dismiss_outside_focus(None, &dismiss_stack)
        }
        DocumentDismissEvent::Escape => popover
            .lifecycle()
            .dismiss_layer()
            .should_dismiss_escape(&dismiss_stack),
        DocumentDismissEvent::Stopped => false,
    }
}

fn popover_dismiss_stack(popover: &Popover) -> Vec<String> {
    vec![popover.relationships().content_id().to_owned()]
}

fn popover_document_path_is_outside(popover: &Popover, path_ids: &[String]) -> bool {
    !popover_document_path_is_inside(popover, path_ids)
}

fn popover_document_path_is_inside(popover: &Popover, path_ids: &[String]) -> bool {
    let relationships = popover.relationships();
    let focus_guards = popover.lifecycle().focus_guards();
    let dismiss_layer = popover.lifecycle().dismiss_layer();

    path_ids.iter().any(|id| {
        id == relationships.content_id()
            || id == relationships.trigger_id()
            || id == relationships.anchor_id()
            || id == focus_guards.before()
            || id == focus_guards.after()
            || dismiss_layer.branches().iter().any(|branch| branch == id)
    })
}

async fn apply_popover_open_focus(popover: &Popover, state: PopoverRuntimeState) -> bool {
    if !popover.lifecycle().focus_scope().autofocus_enabled() {
        return true;
    }

    match popover.lifecycle().open_focus_policy() {
        PopoverOpenFocusPolicy::FirstFocusable => {
            if state.content_handle.with_peek(|handle| handle.is_none()) {
                return false;
            }

            focus_first_focusable(popover.relationships().content_id(), None);
            true
        }
        PopoverOpenFocusPolicy::Target(target) => {
            focus_registered_target(state, target);
            focus_element_by_id(target);
            active_element_matches_id(target).await
        }
        PopoverOpenFocusPolicy::Suppress => true,
    }
}

fn restore_popover_close_focus(popover: &Popover, _state: PopoverRuntimeState) {
    match popover.lifecycle().close_focus_policy() {
        PopoverCloseFocusPolicy::Trigger => {
            restore_focus_element_by_id(popover.relationships().trigger_id());
        }
        PopoverCloseFocusPolicy::Target(target) => {
            restore_focus_element_by_id(target);
        }
        PopoverCloseFocusPolicy::None => {}
    }
}

fn focus_registered_target(state: PopoverRuntimeState, target: &str) -> bool {
    focus_mounted_handle(
        state
            .focus_targets
            .with_peek(|targets| targets.get(target).cloned()),
    )
}

async fn active_element_matches_id(target_id: &str) -> bool {
    let Ok(is_active) = document::eval(&format!(
        r#"(function() {{
    const target = document.getElementById({target_id:?});
    return target instanceof HTMLElement && document.activeElement === target;
}})();"#,
    ))
    .join()
    .await
    else {
        return false;
    };

    is_active
}

fn sync_popover_positioning(popover: &Popover, state: PopoverRuntimeState) {
    stop_popover_position_monitor(state);

    if !popover.is_open() {
        advance_popover_token(state.position_loop_token);
        clear_popover_content_handle(state);
        clear_popover_placement(state);
        return;
    }

    let position_loop_token = advance_popover_token(state.position_loop_token);
    let popover = popover.clone();
    let monitor = start_floating_auto_update_monitor(
        &[
            popover.relationships().anchor_id(),
            popover.relationships().trigger_id(),
        ],
        popover.relationships().content_id(),
    );
    let mut position_monitor = state.position_monitor;
    position_monitor.set(Some(monitor));

    spawn(async move {
        let mut monitor = monitor;

        if let Err(error) = measure_popover_placement(&popover, state).await {
            eprintln!(
                "monoxus popover runtime could not measure placement for {}: {error}",
                popover.relationships().root_id(),
            );
        }

        loop {
            if *state.position_loop_token.peek() != position_loop_token {
                break;
            }

            match recv_floating_auto_update_event(&mut monitor).await {
                Ok(FloatingAutoUpdateEvent::Scroll) => {
                    match sync_hidden_popover_placement(&popover, state).await {
                        Ok(true) => continue,
                        Ok(false) => {}
                        Err(error) => {
                            eprintln!(
                                "monoxus popover runtime could not evaluate detached reference state for {}: {error}",
                                popover.relationships().root_id(),
                            );
                        }
                    }
                }
                Ok(FloatingAutoUpdateEvent::Update) => {}
                Ok(FloatingAutoUpdateEvent::Stopped) => break,
                Err(error) => {
                    if *state.position_loop_token.peek() == position_loop_token {
                        eprintln!(
                            "monoxus popover runtime auto-update monitor failed for {}: {error}",
                            popover.relationships().root_id(),
                        );
                    }
                    break;
                }
            }

            if *state.position_loop_token.peek() != position_loop_token {
                break;
            }

            if let Err(error) = measure_popover_placement(&popover, state).await {
                eprintln!(
                    "monoxus popover runtime could not measure placement for {}: {error}",
                    popover.relationships().root_id(),
                );
            }
        }
    });
}

fn stop_popover_position_monitor(state: PopoverRuntimeState) {
    let Some(monitor) = state.position_monitor.with_peek(|monitor| *monitor) else {
        return;
    };

    let mut position_monitor = state.position_monitor;
    position_monitor.set(None);

    if let Err(error) = stop_floating_auto_update_monitor(monitor) {
        eprintln!("monoxus popover runtime could not stop auto-update monitor: {error}");
    }
}

fn stop_popover_dismiss_monitor(state: PopoverRuntimeState) {
    let Some(monitor) = state.dismiss_monitor.with_peek(|monitor| *monitor) else {
        return;
    };

    let mut dismiss_monitor = state.dismiss_monitor;
    dismiss_monitor.set(None);

    if let Err(error) = stop_document_dismiss_monitor(monitor) {
        eprintln!("monoxus popover runtime could not stop dismiss monitor: {error}");
    }
}

async fn sync_hidden_popover_placement(
    popover: &Popover,
    state: PopoverRuntimeState,
) -> Result<bool, String> {
    if !popover.lifecycle().floating().hide_when_detached() {
        return Ok(false);
    }

    if !popover_reference_is_hidden(popover).await? {
        return Ok(false);
    }

    let Some(current_placement) = state.placement.with_peek(|current| current.clone()) else {
        return Ok(false);
    };
    if current_placement.reference_hidden() {
        return Ok(true);
    }

    let mut placement = state.placement;
    placement.set(Some(current_placement.hide_reference()));
    Ok(true)
}

async fn popover_reference_is_hidden(popover: &Popover) -> Result<bool, String> {
    let anchor_ids = format!(
        "[{:?}, {:?}]",
        popover.relationships().anchor_id(),
        popover.relationships().trigger_id(),
    );
    let hidden: Option<bool> = document::eval(&format!(
        r#"(() => {{
    const anchorIds = {anchor_ids};
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    for (const id of anchorIds) {{
        const element = document.getElementById(id);
        if (!(element instanceof HTMLElement)) {{
            continue;
        }}

        const rect = element.getBoundingClientRect();
        return rect.width <= 0 ||
            rect.height <= 0 ||
            rect.right <= 0 ||
            rect.bottom <= 0 ||
            rect.left >= viewportWidth ||
            rect.top >= viewportHeight;
    }}

    return null;
}})();"#,
    ))
    .join()
    .await
    .map_err(|error| format!("reference hidden query failed: {error}"))?;

    Ok(hidden.unwrap_or(false))
}

async fn measure_popover_placement(
    popover: &Popover,
    state: PopoverRuntimeState,
) -> Result<(), String> {
    let Some(anchor_handle) = state
        .anchor_handle
        .with_peek(|handle| handle.clone())
        .or_else(|| state.trigger_handle.with_peek(|handle| handle.clone()))
    else {
        return Ok(());
    };
    let Some(content_handle) = state.content_handle.with_peek(|handle| handle.clone()) else {
        return Ok(());
    };

    let anchor_rect = read_client_rect(anchor_handle, "anchor").await?;
    let content_rect = read_client_rect(content_handle, "content").await?;
    let content_size = Size::new(content_rect.width(), content_rect.height());
    let viewport_size = read_viewport_size().await?;
    let placement = popover.lifecycle().floating().position_with_available_size(
        anchor_rect,
        content_size,
        viewport_size,
    );

    let should_update = state
        .placement
        .with_peek(|current| current.as_ref() != Some(&placement));
    if should_update {
        let mut current = state.placement;
        current.set(Some(placement));
    }

    Ok(())
}

async fn read_client_rect(mounted: PopoverMountedHandle, label: &str) -> Result<Rect, String> {
    let rect = mounted
        .get_client_rect()
        .await
        .map_err(|error| format!("{label} get_client_rect failed: {error}"))?;

    Ok(Rect::new(
        rect.origin.x as f32,
        rect.origin.y as f32,
        rect.width() as f32,
        rect.height() as f32,
    ))
}

async fn read_viewport_size() -> Result<Size, String> {
    let viewport: [f64; 2] = document::eval("return [window.innerWidth, window.innerHeight];")
        .join()
        .await
        .map_err(|error| format!("viewport query failed: {error}"))?;

    Ok(Size::new(viewport[0] as f32, viewport[1] as f32))
}

fn clear_popover_content_handle(state: PopoverRuntimeState) {
    if state.content_handle.with_peek(|handle| handle.is_some()) {
        let mut content_handle = state.content_handle;
        content_handle.set(None);
    }
}

fn clear_popover_placement(state: PopoverRuntimeState) {
    if state.placement.with_peek(|placement| placement.is_some()) {
        let mut placement = state.placement;
        placement.set(None);
    }
}

fn advance_popover_token(signal: Signal<u64>) -> u64 {
    let next = signal.with_peek(|value| value.saturating_add(1));
    let mut signal = signal;
    signal.set(next);
    next
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverRootAttributes {
    id: String,
    data_state: DataState,
}

impl PopoverRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverTriggerAttributes {
    id: String,
    aria_controls: String,
    aria_expanded: bool,
    data_state: DataState,
    open_request: PopoverStateRequest,
}

impl PopoverTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn aria_controls(&self) -> &str {
        &self.aria_controls
    }

    pub const fn aria_expanded(&self) -> bool {
        self.aria_expanded
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn open_request(&self) -> PopoverStateRequest {
        self.open_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverAnchorAttributes {
    id: String,
}

impl PopoverAnchorAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverPortalAttributes {
    host: PortalHost,
}

impl PopoverPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverContentAttributes {
    id: String,
    role: &'static str,
    aria_modal: bool,
    data_state: DataState,
    data_side: &'static str,
    data_align: &'static str,
}

impl PopoverContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub const fn aria_modal(&self) -> bool {
        self.aria_modal
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_side(&self) -> &'static str {
        self.data_side
    }

    pub const fn data_align(&self) -> &'static str {
        self.data_align
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverArrowAttributes {
    id: String,
    data_state: DataState,
    data_side: &'static str,
    data_align: &'static str,
}

impl PopoverArrowAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_side(&self) -> &'static str {
        self.data_side
    }

    pub const fn data_align(&self) -> &'static str {
        self.data_align
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopoverCloseAttributes {
    id: String,
    data_state: DataState,
    close_request: PopoverStateRequest,
}

impl PopoverCloseAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn close_request(&self) -> PopoverStateRequest {
        self.close_request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popover_document_path_detection_treats_runtime_surfaces_as_inside() {
        let scope = ScopeHandle::root("playground").child("popover-test");
        let mut popover = Popover::new(scope.clone(), true).with_modal(true);
        let branch_id = scope.qualify("branch");
        let focus_before = popover.lifecycle().focus_guards().before().clone();

        assert!(popover.lifecycle_mut().register_branch(branch_id.clone()));

        assert!(popover_document_path_is_inside(
            &popover,
            &[branch_id.clone()]
        ));
        assert!(popover_document_path_is_inside(
            &popover,
            &[popover.relationships().trigger_id().to_owned()]
        ));
        assert!(popover_document_path_is_inside(
            &popover,
            &[popover.relationships().anchor_id().to_owned()]
        ));
        assert!(popover_document_path_is_inside(
            &popover,
            &[popover.relationships().content_id().to_owned()]
        ));
        assert!(popover_document_path_is_inside(&popover, &[focus_before]));
        assert!(popover_document_path_is_outside(
            &popover,
            &[String::from("outside-target")]
        ));
    }
}

use std::{collections::HashMap, rc::Rc};

use dioxus::{document, prelude::*};

use crate::foundation::{
    compose::{
        AsChildSlot, EventHandlerOptions, RefHandler, Slottable, compose_event_handlers,
        compose_refs,
    },
    overlay::{DismissLayer, FocusGuards, FocusScope, PortalHost, Presence},
    shared::ScopeHandle,
    state::DataState,
};

pub type DialogMountedHandle = Rc<MountedData>;

const DIALOG_FOCUSABLE_SELECTOR: &str = concat!(
    "[data-monoxus-autofocus],",
    "[autofocus],",
    "button:not([disabled]),",
    "[href],",
    "input:not([disabled]):not([type=\"hidden\"]),",
    "select:not([disabled]),",
    "textarea:not([disabled]),",
    "[tabindex]:not([tabindex=\"-1\"])"
);

pub const DIALOG_PARTS: [DialogPart; 8] = [
    DialogPart::Root,
    DialogPart::Trigger,
    DialogPart::Portal,
    DialogPart::Overlay,
    DialogPart::Content,
    DialogPart::Title,
    DialogPart::Description,
    DialogPart::Close,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DialogPart {
    Root,
    Trigger,
    Portal,
    Overlay,
    Content,
    Title,
    Description,
    Close,
}

impl DialogPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Portal => "portal",
            Self::Overlay => "overlay",
            Self::Content => "content",
            Self::Title => "title",
            Self::Description => "description",
            Self::Close => "close",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DialogStateRequest {
    Open,
    Close,
}

impl DialogStateRequest {
    pub const fn next_open(self) -> bool {
        matches!(self, Self::Open)
    }

    pub fn data_state(self) -> DataState {
        if self.next_open() {
            DataState::Open
        } else {
            DataState::Closed
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DialogMode {
    Modal,
    NonModal,
}

impl DialogMode {
    pub const fn is_modal(self) -> bool {
        matches!(self, Self::Modal)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogOpenFocusPolicy {
    FirstFocusable,
    Target(String),
    Suppress,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogCloseFocusPolicy {
    Trigger,
    Target(String),
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogScrollLockPolicy {
    enabled: bool,
    restore_delay: Option<u64>,
}

impl DialogScrollLockPolicy {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogOutsideDismissBehavior {
    Dismiss,
    Ignore,
}

impl DialogOutsideDismissBehavior {
    pub const fn dismisses(self) -> bool {
        matches!(self, Self::Dismiss)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogOutsideInteractionPolicy {
    pointer_down_outside: DialogOutsideDismissBehavior,
    focus_outside: DialogOutsideDismissBehavior,
}

impl DialogOutsideInteractionPolicy {
    pub const fn new(
        pointer_down_outside: DialogOutsideDismissBehavior,
        focus_outside: DialogOutsideDismissBehavior,
    ) -> Self {
        Self {
            pointer_down_outside,
            focus_outside,
        }
    }

    pub const fn modal_default() -> Self {
        Self::new(
            DialogOutsideDismissBehavior::Dismiss,
            DialogOutsideDismissBehavior::Ignore,
        )
    }

    pub const fn non_modal_default() -> Self {
        Self::new(
            DialogOutsideDismissBehavior::Dismiss,
            DialogOutsideDismissBehavior::Dismiss,
        )
    }

    pub const fn alert_default() -> Self {
        Self::new(
            DialogOutsideDismissBehavior::Ignore,
            DialogOutsideDismissBehavior::Ignore,
        )
    }

    pub const fn pointer_down_outside(&self) -> DialogOutsideDismissBehavior {
        self.pointer_down_outside
    }

    pub const fn focus_outside(&self) -> DialogOutsideDismissBehavior {
        self.focus_outside
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    overlay_id: String,
    content_id: String,
    title_id: String,
    description_id: String,
    close_id: String,
    focus_guard_before_id: String,
    focus_guard_after_id: String,
}

impl DialogRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            overlay_id: scope.qualify("overlay"),
            content_id: scope.qualify("content"),
            title_id: scope.qualify("title"),
            description_id: scope.qualify("description"),
            close_id: scope.qualify("close"),
            focus_guard_before_id: scope.qualify("focus-guard-before"),
            focus_guard_after_id: scope.qualify("focus-guard-after"),
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

    pub fn overlay_id(&self) -> &str {
        &self.overlay_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn title_id(&self) -> &str {
        &self.title_id
    }

    pub fn description_id(&self) -> &str {
        &self.description_id
    }

    pub fn close_id(&self) -> &str {
        &self.close_id
    }

    pub fn focus_guard_before_id(&self) -> &str {
        &self.focus_guard_before_id
    }

    pub fn focus_guard_after_id(&self) -> &str {
        &self.focus_guard_after_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogLifecycle {
    portal_host: PortalHost,
    presence: Presence,
    focus_scope: FocusScope<String>,
    focus_guards: FocusGuards<String>,
    dismiss_layer: DismissLayer<String>,
    mode: DialogMode,
    open_focus_policy: DialogOpenFocusPolicy,
    close_focus_policy: DialogCloseFocusPolicy,
    default_restore_focus_target: String,
    scroll_lock_policy: DialogScrollLockPolicy,
    outside_interaction_policy: DialogOutsideInteractionPolicy,
}

impl DialogLifecycle {
    pub fn new(relationships: &DialogRelationships, open: bool) -> Self {
        Self::new_with_mode(relationships, open, DialogMode::Modal)
    }

    fn new_with_mode(relationships: &DialogRelationships, open: bool, mode: DialogMode) -> Self {
        let outside_interaction_policy = match mode {
            DialogMode::Modal => DialogOutsideInteractionPolicy::modal_default(),
            DialogMode::NonModal => DialogOutsideInteractionPolicy::non_modal_default(),
        };
        let scroll_lock_policy = match mode {
            DialogMode::Modal => DialogScrollLockPolicy::enabled(),
            DialogMode::NonModal => DialogScrollLockPolicy::disabled(),
        };
        let default_restore_focus_target = relationships.trigger_id().to_owned();
        let mut focus_scope = FocusScope::new(relationships.content_id().to_owned())
            .with_trap_focus(mode.is_modal())
            .with_loop_focus(true);
        focus_scope.capture_restore_target(Some(default_restore_focus_target.clone()));
        focus_scope.set_autofocus_enabled(true);
        let mut dismiss_layer =
            DismissLayer::new(relationships.content_id().to_owned()).with_modal(mode.is_modal());
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
                relationships.focus_guard_before_id().to_owned(),
                relationships.focus_guard_after_id().to_owned(),
            ),
            dismiss_layer,
            mode,
            open_focus_policy: DialogOpenFocusPolicy::FirstFocusable,
            close_focus_policy: DialogCloseFocusPolicy::Trigger,
            default_restore_focus_target,
            scroll_lock_policy,
            outside_interaction_policy,
        }
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.portal_host = portal_host;
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

    pub const fn mode(&self) -> DialogMode {
        self.mode
    }

    pub fn open_focus_policy(&self) -> &DialogOpenFocusPolicy {
        &self.open_focus_policy
    }

    pub fn close_focus_policy(&self) -> &DialogCloseFocusPolicy {
        &self.close_focus_policy
    }

    pub const fn scroll_lock_policy(&self) -> &DialogScrollLockPolicy {
        &self.scroll_lock_policy
    }

    pub const fn outside_interaction_policy(&self) -> &DialogOutsideInteractionPolicy {
        &self.outside_interaction_policy
    }

    pub fn set_autofocus_target(&mut self, target: Option<String>) {
        match target {
            Some(target) => self.set_open_focus_policy(DialogOpenFocusPolicy::Target(target)),
            None => self.set_open_focus_policy(DialogOpenFocusPolicy::FirstFocusable),
        }
    }

    pub fn capture_restore_target(&mut self, target: Option<String>) {
        match target {
            Some(target) => self.set_close_focus_policy(DialogCloseFocusPolicy::Target(target)),
            None => self.set_close_focus_policy(DialogCloseFocusPolicy::None),
        }
    }

    pub fn register_branch(&mut self, branch: impl Into<String>) -> bool {
        let branch = branch.into();
        let focus_registered = self.focus_scope.register_branch(branch.clone());
        let dismiss_registered = self.dismiss_layer.register_branch(branch);
        debug_assert_eq!(focus_registered, dismiss_registered);
        focus_registered && dismiss_registered
    }

    pub(crate) fn set_mode(&mut self, mode: DialogMode) {
        let previous_mode = self.mode;
        self.mode = mode;
        self.focus_scope.set_trap_focus(mode.is_modal());
        self.focus_scope.set_loop_focus(true);
        self.dismiss_layer.set_modal(mode.is_modal());

        if self.outside_interaction_policy
            == Self::default_outside_interaction_policy(previous_mode)
        {
            self.set_outside_interaction_policy(Self::default_outside_interaction_policy(mode));
        }

        if self.scroll_lock_policy == Self::default_scroll_lock_policy(previous_mode) {
            self.scroll_lock_policy = Self::default_scroll_lock_policy(mode);
        }
    }

    pub(crate) fn set_open_focus_policy(&mut self, policy: DialogOpenFocusPolicy) {
        match &policy {
            DialogOpenFocusPolicy::FirstFocusable => {
                self.focus_scope.set_autofocus_enabled(true);
                self.focus_scope.set_autofocus_target(None);
            }
            DialogOpenFocusPolicy::Target(target) => {
                self.focus_scope.set_autofocus_enabled(true);
                self.focus_scope.set_autofocus_target(Some(target.clone()));
            }
            DialogOpenFocusPolicy::Suppress => {
                self.focus_scope.set_autofocus_enabled(false);
                self.focus_scope.set_autofocus_target(None);
            }
        }

        self.open_focus_policy = policy;
    }

    pub(crate) fn set_close_focus_policy(&mut self, policy: DialogCloseFocusPolicy) {
        match &policy {
            DialogCloseFocusPolicy::Trigger => self
                .focus_scope
                .capture_restore_target(Some(self.default_restore_focus_target.clone())),
            DialogCloseFocusPolicy::Target(target) => self
                .focus_scope
                .capture_restore_target(Some(target.clone())),
            DialogCloseFocusPolicy::None => self.focus_scope.capture_restore_target(None),
        }

        self.close_focus_policy = policy;
    }

    pub(crate) fn set_scroll_lock_policy(&mut self, policy: DialogScrollLockPolicy) {
        self.scroll_lock_policy = policy;
    }

    pub(crate) fn set_outside_interaction_policy(
        &mut self,
        policy: DialogOutsideInteractionPolicy,
    ) {
        self.dismiss_layer
            .set_pointer_down_outside_dismiss(policy.pointer_down_outside().dismisses());
        self.dismiss_layer
            .set_focus_outside_dismiss(policy.focus_outside().dismisses());
        self.outside_interaction_policy = policy;
    }

    fn default_outside_interaction_policy(mode: DialogMode) -> DialogOutsideInteractionPolicy {
        match mode {
            DialogMode::Modal => DialogOutsideInteractionPolicy::modal_default(),
            DialogMode::NonModal => DialogOutsideInteractionPolicy::non_modal_default(),
        }
    }

    fn default_scroll_lock_policy(mode: DialogMode) -> DialogScrollLockPolicy {
        match mode {
            DialogMode::Modal => DialogScrollLockPolicy::enabled(),
            DialogMode::NonModal => DialogScrollLockPolicy::disabled(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dialog {
    relationships: DialogRelationships,
    lifecycle: DialogLifecycle,
    open: bool,
}

impl Dialog {
    pub fn new(scope: ScopeHandle, open: bool) -> Self {
        Self::new_with_mode(scope, open, DialogMode::Modal)
    }

    pub fn new_non_modal(scope: ScopeHandle, open: bool) -> Self {
        Self::new_with_mode(scope, open, DialogMode::NonModal)
    }

    fn new_with_mode(scope: ScopeHandle, open: bool, mode: DialogMode) -> Self {
        let relationships = DialogRelationships::new(scope);
        let lifecycle = DialogLifecycle::new_with_mode(&relationships, open, mode);

        Self {
            relationships,
            lifecycle,
            open,
        }
    }

    pub const fn parts() -> &'static [DialogPart] {
        &DIALOG_PARTS
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

    pub fn relationships(&self) -> &DialogRelationships {
        &self.relationships
    }

    pub fn lifecycle(&self) -> &DialogLifecycle {
        &self.lifecycle
    }

    pub fn lifecycle_mut(&mut self) -> &mut DialogLifecycle {
        &mut self.lifecycle
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.lifecycle = self.lifecycle.with_portal_host(portal_host);
        self
    }

    pub fn with_mode(mut self, mode: DialogMode) -> Self {
        self.lifecycle.set_mode(mode);
        self
    }

    pub fn with_open_focus_policy(mut self, policy: DialogOpenFocusPolicy) -> Self {
        self.lifecycle.set_open_focus_policy(policy);
        self
    }

    pub fn with_close_focus_policy(mut self, policy: DialogCloseFocusPolicy) -> Self {
        self.lifecycle.set_close_focus_policy(policy);
        self
    }

    pub fn with_scroll_lock_policy(mut self, policy: DialogScrollLockPolicy) -> Self {
        self.lifecycle.set_scroll_lock_policy(policy);
        self
    }

    pub fn with_outside_interaction_policy(
        mut self,
        policy: DialogOutsideInteractionPolicy,
    ) -> Self {
        self.lifecycle.set_outside_interaction_policy(policy);
        self
    }

    pub fn root(&self) -> DialogRootAttributes {
        DialogRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_state: self.data_state(),
        }
    }

    pub fn trigger(&self) -> DialogTriggerAttributes {
        DialogTriggerAttributes {
            id: self.relationships.trigger_id().to_owned(),
            aria_controls: self.relationships.content_id().to_owned(),
            aria_expanded: self.is_open(),
            data_state: self.data_state(),
            open_request: DialogStateRequest::Open,
        }
    }

    pub fn portal(&self) -> DialogPortalAttributes {
        DialogPortalAttributes {
            host: self.lifecycle.portal_host().clone(),
        }
    }

    pub fn overlay(&self) -> DialogOverlayAttributes {
        DialogOverlayAttributes {
            id: self.relationships.overlay_id().to_owned(),
            data_state: self.data_state(),
        }
    }

    pub fn content(&self) -> DialogContentAttributes {
        self.content_with_role("dialog")
    }

    pub(crate) fn content_with_role(&self, role: &'static str) -> DialogContentAttributes {
        DialogContentAttributes {
            id: self.relationships.content_id().to_owned(),
            role,
            aria_modal: self.lifecycle.dismiss_layer().is_modal(),
            aria_labelledby: self.relationships.title_id().to_owned(),
            aria_describedby: self.relationships.description_id().to_owned(),
            data_state: self.data_state(),
        }
    }

    pub fn title(&self) -> DialogTitleAttributes {
        DialogTitleAttributes {
            id: self.relationships.title_id().to_owned(),
        }
    }

    pub fn description(&self) -> DialogDescriptionAttributes {
        DialogDescriptionAttributes {
            id: self.relationships.description_id().to_owned(),
        }
    }

    pub fn close(&self) -> DialogCloseAttributes {
        DialogCloseAttributes {
            id: self.relationships.close_id().to_owned(),
            data_state: self.data_state(),
            close_request: DialogStateRequest::Close,
        }
    }
}

#[derive(Clone, Copy)]
struct DialogRuntimeState {
    trigger_handle: Signal<Option<DialogMountedHandle>>,
    content_handle: Signal<Option<DialogMountedHandle>>,
    focus_targets: Signal<HashMap<String, DialogMountedHandle>>,
    last_open: Signal<bool>,
    pending_open_focus: Signal<bool>,
    scroll_lock_held: Signal<bool>,
}

#[derive(Clone)]
pub struct DialogRuntime {
    dialog: Dialog,
    state: DialogRuntimeState,
}

pub fn use_dialog_runtime(dialog: Dialog) -> DialogRuntime {
    let state = DialogRuntimeState {
        trigger_handle: use_signal(|| None),
        content_handle: use_signal(|| None),
        focus_targets: use_signal(HashMap::new),
        last_open: use_signal(|| dialog.is_open()),
        pending_open_focus: use_signal(|| dialog.is_open()),
        scroll_lock_held: use_signal(|| false),
    };
    let effect_state = state;
    let cleanup_state = state;
    let cleanup_key = dialog.relationships().root_id().to_owned();

    use_effect(use_reactive((&dialog,), move |(dialog,)| {
        sync_dialog_runtime(&dialog, effect_state);
    }));

    dioxus::core::use_drop(move || {
        if *cleanup_state.scroll_lock_held.peek() {
            release_scroll_lock(&cleanup_key, None);
        }
    });

    DialogRuntime { dialog, state }
}

impl DialogRuntime {
    pub fn dialog(&self) -> &Dialog {
        &self.dialog
    }

    pub const fn is_open(&self) -> bool {
        self.dialog.is_open()
    }

    pub fn data_state(&self) -> DataState {
        self.dialog.data_state()
    }

    pub fn relationships(&self) -> &DialogRelationships {
        self.dialog.relationships()
    }

    pub fn lifecycle(&self) -> &DialogLifecycle {
        self.dialog.lifecycle()
    }

    pub fn root(&self) -> DialogRootAttributes {
        self.dialog.root()
    }

    pub fn trigger(&self) -> DialogTriggerAttributes {
        self.dialog.trigger()
    }

    pub fn portal(&self) -> DialogPortalAttributes {
        self.dialog.portal()
    }

    pub fn overlay(&self) -> DialogOverlayAttributes {
        self.dialog.overlay()
    }

    pub fn content(&self) -> DialogContentAttributes {
        self.dialog.content()
    }

    pub fn title(&self) -> DialogTitleAttributes {
        self.dialog.title()
    }

    pub fn description(&self) -> DialogDescriptionAttributes {
        self.dialog.description()
    }

    pub fn close(&self) -> DialogCloseAttributes {
        self.dialog.close()
    }

    pub fn trigger_handle(&self) -> Option<DialogMountedHandle> {
        self.state.trigger_handle.cloned()
    }

    pub fn content_handle(&self) -> Option<DialogMountedHandle> {
        self.state.content_handle.cloned()
    }

    pub fn mounted_focus_target(&self, id: &str) -> Option<DialogMountedHandle> {
        self.state
            .focus_targets
            .with_peek(|targets| targets.get(id).cloned())
    }

    pub fn capture_trigger(&self, mounted: DialogMountedHandle) {
        let mut trigger_handle = self.state.trigger_handle;
        trigger_handle.set(Some(mounted.clone()));
        self.capture_focus_target(self.relationships().trigger_id().to_owned(), mounted);
    }

    pub fn capture_content(&self, mounted: DialogMountedHandle) {
        let mut content_handle = self.state.content_handle;
        content_handle.set(Some(mounted.clone()));
        self.capture_focus_target(self.relationships().content_id().to_owned(), mounted);
    }

    pub fn capture_close(&self, mounted: DialogMountedHandle) {
        self.capture_focus_target(self.relationships().close_id().to_owned(), mounted);
    }

    pub fn capture_focus_target(&self, id: impl Into<String>, mounted: DialogMountedHandle) {
        let id = id.into();
        let mut focus_targets = self.state.focus_targets;
        focus_targets.with_mut(|targets| {
            targets.insert(id, mounted);
        });
    }

    pub fn mount_trigger(&self) -> impl FnMut(MountedEvent) + 'static {
        let runtime = self.clone();
        move |event| runtime.capture_trigger(event.data())
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
}

fn sync_dialog_runtime(dialog: &Dialog, state: DialogRuntimeState) {
    let is_open = dialog.is_open();
    let was_open = *state.last_open.peek();
    let should_hold_scroll_lock = is_open
        && dialog.lifecycle().mode().is_modal()
        && dialog.lifecycle().scroll_lock_policy().is_enabled();
    let is_holding_scroll_lock = *state.scroll_lock_held.peek();

    if is_open && !was_open && !*state.pending_open_focus.peek() {
        let mut pending_open_focus = state.pending_open_focus;
        pending_open_focus.set(true);
    }

    if should_hold_scroll_lock && !is_holding_scroll_lock {
        acquire_scroll_lock(dialog.relationships().root_id());
        let mut scroll_lock_held = state.scroll_lock_held;
        scroll_lock_held.set(true);
    } else if !should_hold_scroll_lock && is_holding_scroll_lock {
        let restore_delay = if was_open && !is_open {
            dialog.lifecycle().scroll_lock_policy().restore_delay()
        } else {
            None
        };
        release_scroll_lock(dialog.relationships().root_id(), restore_delay);
        let mut scroll_lock_held = state.scroll_lock_held;
        scroll_lock_held.set(false);
    }

    if is_open {
        if *state.pending_open_focus.peek() && apply_open_focus(dialog) {
            let mut pending_open_focus = state.pending_open_focus;
            pending_open_focus.set(false);
        }
    } else {
        if was_open {
            restore_close_focus(dialog, state);
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

fn apply_open_focus(dialog: &Dialog) -> bool {
    if !dialog.lifecycle().focus_scope().autofocus_enabled() {
        return true;
    }

    match dialog.lifecycle().open_focus_policy() {
        DialogOpenFocusPolicy::FirstFocusable => {
            focus_first_focusable(dialog.relationships().content_id());
            true
        }
        DialogOpenFocusPolicy::Target(target) => {
            focus_element_by_id(target);
            true
        }
        DialogOpenFocusPolicy::Suppress => true,
    }
}

fn restore_close_focus(dialog: &Dialog, state: DialogRuntimeState) {
    match dialog.lifecycle().close_focus_policy() {
        DialogCloseFocusPolicy::Trigger => {
            if !focus_mounted_handle(state.trigger_handle.cloned()) {
                focus_element_by_id(dialog.relationships().trigger_id());
            }
        }
        DialogCloseFocusPolicy::Target(target) => {
            if !focus_registered_target(state, target) {
                focus_element_by_id(target);
            }
        }
        DialogCloseFocusPolicy::None => {}
    }
}

fn focus_registered_target(state: DialogRuntimeState, target: &str) -> bool {
    focus_mounted_handle(
        state
            .focus_targets
            .with_peek(|targets| targets.get(target).cloned()),
    )
}

fn focus_mounted_handle(handle: Option<DialogMountedHandle>) -> bool {
    let Some(handle) = handle else {
        return false;
    };

    spawn(async move {
        let _ = handle.set_focus(true).await;
    });
    true
}

fn focus_element_by_id(target_id: &str) {
    document::eval(&format!(
        r#"(function() {{
    const target = document.getElementById({target_id:?});
    if (!(target instanceof HTMLElement)) {{
        return;
    }}

    target.focus();
}})();"#,
    ));
}

fn focus_first_focusable(content_id: &str) {
    document::eval(&format!(
        r#"(function() {{
    const root = document.getElementById({content_id:?});
    if (!(root instanceof HTMLElement)) {{
        return;
    }}

    const selector = {DIALOG_FOCUSABLE_SELECTOR:?};
    const candidate =
        (root.matches(selector) ? root : root.querySelector(selector));

    if (candidate instanceof HTMLElement) {{
        if (candidate === root && !root.hasAttribute("tabindex")) {{
            root.setAttribute("tabindex", "-1");
        }}

        candidate.focus();
        return;
    }}

    if (!root.hasAttribute("tabindex")) {{
        root.setAttribute("tabindex", "-1");
    }}

    root.focus();
}})();"#,
    ));
}

fn acquire_scroll_lock(lock_id: &str) {
    document::eval(&format!(
        r#"(function() {{
    const lockId = {lock_id:?};
    const state = window.__monoxusDialogRuntime ??= {{
        scrollLocks: new Set(),
        previousBodyOverflow: null,
        previousBodyPaddingRight: null,
        previousDocumentOverflow: null,
        restoreToken: 0,
    }};

    state.restoreToken += 1;
    if (state.scrollLocks.has(lockId)) {{
        return;
    }}

    if (state.scrollLocks.size === 0) {{
        const body = document.body;
        const documentElement = document.documentElement;
        const computedBodyStyle = window.getComputedStyle(body);
        const bodyPaddingRight = Number.parseFloat(computedBodyStyle.paddingRight || "0") || 0;
        const scrollbarWidth = Math.max(0, window.innerWidth - documentElement.clientWidth);

        state.previousBodyOverflow = body.style.overflow;
        state.previousBodyPaddingRight = body.style.paddingRight;
        state.previousDocumentOverflow = documentElement.style.overflow;

        body.style.overflow = "hidden";
        documentElement.style.overflow = "hidden";

        if (scrollbarWidth > 0) {{
            body.style.paddingRight = `${{bodyPaddingRight + scrollbarWidth}}px`;
        }}
    }}

    state.scrollLocks.add(lockId);
}})();"#,
    ));
}

fn release_scroll_lock(lock_id: &str, restore_delay: Option<u64>) {
    let delay_ms = restore_delay.unwrap_or_default();

    document::eval(&format!(
        r#"(function() {{
    const lockId = {lock_id:?};
    const delayMs = {delay_ms};
    const state = window.__monoxusDialogRuntime;
    if (!state || !(state.scrollLocks instanceof Set)) {{
        return;
    }}

    state.scrollLocks.delete(lockId);
    const restoreToken = ++state.restoreToken;
    if (state.scrollLocks.size > 0) {{
        return;
    }}

    const restore = () => {{
        const currentState = window.__monoxusDialogRuntime;
        if (!currentState || currentState.restoreToken !== restoreToken) {{
            return;
        }}

        if (currentState.scrollLocks instanceof Set && currentState.scrollLocks.size > 0) {{
            return;
        }}

        document.body.style.overflow = currentState.previousBodyOverflow ?? "";
        document.body.style.paddingRight = currentState.previousBodyPaddingRight ?? "";
        document.documentElement.style.overflow = currentState.previousDocumentOverflow ?? "";
        currentState.previousBodyOverflow = null;
        currentState.previousBodyPaddingRight = null;
        currentState.previousDocumentOverflow = null;
    }};

    if (delayMs > 0) {{
        window.setTimeout(restore, delayMs);
        return;
    }}

    restore();
}})();"#,
    ));
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogRootAttributes {
    id: String,
    data_state: DataState,
}

impl DialogRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogTriggerAttributes {
    id: String,
    aria_controls: String,
    aria_expanded: bool,
    data_state: DataState,
    open_request: DialogStateRequest,
}

impl DialogTriggerAttributes {
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

    pub const fn open_request(&self) -> DialogStateRequest {
        self.open_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogPortalAttributes {
    host: PortalHost,
}

impl DialogPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogOverlayAttributes {
    id: String,
    data_state: DataState,
}

impl DialogOverlayAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogContentAttributes {
    id: String,
    role: &'static str,
    aria_modal: bool,
    aria_labelledby: String,
    aria_describedby: String,
    data_state: DataState,
}

impl DialogContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub const fn aria_modal(&self) -> bool {
        self.aria_modal
    }

    pub fn aria_labelledby(&self) -> &str {
        &self.aria_labelledby
    }

    pub fn aria_describedby(&self) -> &str {
        &self.aria_describedby
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogTitleAttributes {
    id: String,
}

impl DialogTitleAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogDescriptionAttributes {
    id: String,
}

impl DialogDescriptionAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogCloseAttributes {
    id: String,
    data_state: DataState,
    close_request: DialogStateRequest,
}

impl DialogCloseAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn close_request(&self) -> DialogStateRequest {
        self.close_request
    }
}

pub fn project_as_child<T, C>(target: T, content: Slottable<C>) -> (T, C) {
    AsChildSlot::new(target).with_slottable(content)
}

pub fn compose_part_event_handlers<E, C, I>(
    consumer: Option<C>,
    internal: Option<I>,
    options: EventHandlerOptions<E>,
) -> impl FnMut(&mut E)
where
    C: FnMut(&mut E),
    I: FnMut(&mut E),
{
    compose_event_handlers(consumer, internal, options)
}

pub fn compose_part_refs<T, I>(refs: I) -> impl FnMut(T)
where
    T: Clone + 'static,
    I: IntoIterator<Item = Option<RefHandler<T>>>,
{
    compose_refs(refs)
}

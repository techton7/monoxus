use crate::foundation::{
    overlay::{DismissLayer, FocusGuards, FocusScope, PortalHost, Presence},
    shared::ScopeHandle,
    state::DataState,
};

use super::{
    attrs::{
        DialogCloseAttributes, DialogContentAttributes, DialogDescriptionAttributes,
        DialogOverlayAttributes, DialogPortalAttributes, DialogRootAttributes,
        DialogTitleAttributes, DialogTriggerAttributes,
    },
    relationships::DialogRelationships,
    types::{
        DIALOG_PARTS, DialogCloseFocusPolicy, DialogMode, DialogOpenFocusPolicy,
        DialogOutsideInteractionPolicy, DialogPart, DialogScrollLockPolicy, DialogStateRequest,
    },
};

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

    pub fn new_with_mode(relationships: &DialogRelationships, open: bool, mode: DialogMode) -> Self {
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

    pub fn content_with_role(&self, role: &'static str) -> DialogContentAttributes {
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

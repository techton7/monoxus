use crate::foundation::{
    overlay::{
        DismissLayer, FloatingLayer, FocusGuards, FocusScope, GeometryVars, PlacementSide,
        PortalHost, Presence, Rect, Size,
    },
    shared::ScopeHandle,
    state::DataState,
};

use super::{
    attrs::{
        PopoverAnchorAttributes, PopoverArrowAttributes, PopoverCloseAttributes,
        PopoverContentAttributes, PopoverPortalAttributes, PopoverRootAttributes,
        PopoverTriggerAttributes,
    },
    relationships::PopoverRelationships,
    types::{
        POPOVER_GEOMETRY_NAMESPACE, POPOVER_PARTS, PopoverCloseFocusPolicy, PopoverOpenFocusPolicy,
        PopoverOutsideInteractionPolicy, PopoverPart, PopoverScrollLockPolicy, PopoverStateRequest,
    },
};

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

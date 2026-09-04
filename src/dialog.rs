use crate::foundation::{
    compose::{
        AsChildSlot, EventHandlerOptions, RefHandler, Slottable, compose_event_handlers,
        compose_refs,
    },
    overlay::{DismissLayer, FocusGuards, FocusScope, PortalHost, Presence},
    shared::ScopeHandle,
    state::DataState,
};

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
}

impl DialogLifecycle {
    pub fn new(relationships: &DialogRelationships, open: bool) -> Self {
        Self {
            portal_host: PortalHost::default_host(),
            presence: Presence::new(open).with_retained_mount(true),
            focus_scope: FocusScope::new(relationships.content_id().to_owned())
                .with_trap_focus(true)
                .with_loop_focus(true),
            focus_guards: FocusGuards::new(
                relationships.focus_guard_before_id().to_owned(),
                relationships.focus_guard_after_id().to_owned(),
            ),
            dismiss_layer: DismissLayer::new(relationships.content_id().to_owned())
                .with_modal(true),
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

    pub fn set_autofocus_target(&mut self, target: Option<String>) {
        self.focus_scope.set_autofocus_target(target);
    }

    pub fn capture_restore_target(&mut self, target: Option<String>) {
        self.focus_scope.capture_restore_target(target);
    }

    pub fn register_branch(&mut self, branch: impl Into<String>) -> bool {
        let branch = branch.into();
        let focus_registered = self.focus_scope.register_branch(branch.clone());
        let dismiss_registered = self.dismiss_layer.register_branch(branch);
        debug_assert_eq!(focus_registered, dismiss_registered);
        focus_registered && dismiss_registered
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
        let relationships = DialogRelationships::new(scope);
        let lifecycle = DialogLifecycle::new(&relationships, open);

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

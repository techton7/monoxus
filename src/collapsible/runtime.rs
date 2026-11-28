use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeReason, ChangeSource};

use crate::foundation::state::{ChangeOutcome, ControllableSignal};

use super::{
    attrs::{CollapsibleContentAttributes, CollapsibleRootAttributes, CollapsibleTriggerAttributes},
    relationships::CollapsibleRelationships,
    state::Collapsible,
    types::{CollapsibleOpenChangeHandler, CollapsibleStateRequest},
};

#[derive(Clone, PartialEq)]
pub struct CollapsibleRuntimeState {
    pub open: Signal<bool>,
}

#[derive(Clone, PartialEq)]
pub struct CollapsibleRuntime {
    collapsible: Collapsible,
    open_ctl: Option<ControllableSignal<bool>>,
    on_open_change: CopyValue<Option<CollapsibleOpenChangeHandler>>,
    state: CollapsibleRuntimeState,
}

pub fn use_collapsible_runtime<F>(
    collapsible: Collapsible,
    on_open_change: Option<F>,
) -> CollapsibleRuntime
where
    F: Fn(bool) + 'static,
{
    let synced_change: Option<CollapsibleOpenChangeHandler> =
        on_open_change.map(|f| Rc::new(f) as CollapsibleOpenChangeHandler);
    let mut on_open_change_slot: CopyValue<Option<CollapsibleOpenChangeHandler>> =
        use_hook(|| CopyValue::new(None));
    on_open_change_slot.set(synced_change);

    let initial_open = collapsible.is_open();

    let state = CollapsibleRuntimeState {
        open: use_signal(|| initial_open),
    };

    let effect_state = state.clone();

    use_effect(use_reactive((&collapsible,), move |(collapsible,)| {
        let mut open_sig = effect_state.open;
        crate::foundation::state::sync_signal(&mut open_sig, collapsible.is_open());
    }));

    CollapsibleRuntime {
        collapsible,
        open_ctl: None,
        on_open_change: on_open_change_slot,
        state,
    }
}

pub(crate) fn use_collapsible_runtime_controllable(
    collapsible: Collapsible,
    open_ctl: ControllableSignal<bool>,
) -> CollapsibleRuntime {
    let on_open_change_slot: CopyValue<Option<CollapsibleOpenChangeHandler>> =
        use_hook(|| CopyValue::new(None));
    let state = CollapsibleRuntimeState {
        open: open_ctl.signal(),
    };

    CollapsibleRuntime {
        collapsible,
        open_ctl: Some(open_ctl),
        on_open_change: on_open_change_slot,
        state,
    }
}

impl CollapsibleRuntime {
    pub fn collapsible(&self) -> Collapsible {
        let mut updated = self.collapsible.clone();
        let _ = updated.set_open(*self.state.open.read());
        updated
    }

    pub fn relationships(&self) -> &CollapsibleRelationships {
        self.collapsible.relationships()
    }

    pub fn is_open(&self) -> bool {
        *self.state.open.read()
    }

    pub fn is_disabled(&self) -> bool {
        self.collapsible.is_disabled()
    }

    pub fn root(&self) -> CollapsibleRootAttributes {
        self.collapsible().root()
    }

    pub fn trigger(&self) -> CollapsibleTriggerAttributes {
        self.collapsible().trigger()
    }

    pub fn content(&self) -> CollapsibleContentAttributes {
        self.collapsible().content()
    }

    pub fn set_open(&self, open: bool) {
        let _ = self.set_open_with_context(
            open,
            ChangeContext::new(ChangeReason::Programmatic, ChangeSource::Programmatic),
        );
    }

    pub fn set_open_with_context(&self, open: bool, ctx: ChangeContext) -> ChangeOutcome {
        if self.is_disabled() {
            return ChangeOutcome::Unchanged;
        }
        if let Some(ctl) = self.open_ctl {
            return ctl.set_with_context(open, ctx);
        }
        let changed = *self.state.open.peek() != open;
        if changed {
            let mut open_sig = self.state.open;
            open_sig.set(open);
            let callback = self
                .on_open_change
                .try_read()
                .ok()
                .and_then(|slot| slot.as_ref().cloned());
            if let Some(callback) = callback {
                callback(open);
            }
            ChangeOutcome::Committed(ctx)
        } else {
            ChangeOutcome::Unchanged
        }
    }

    pub fn toggle(&self) {
        let current = self.is_open();
        self.set_open(!current);
    }

    pub fn toggle_with_context(&self, ctx: ChangeContext) -> ChangeOutcome {
        let current = *self.state.open.peek();
        self.set_open_with_context(!current, ctx)
    }

    pub fn request_state(&self, request: CollapsibleStateRequest) {
        let next = request.next_open(self.is_open());
        self.set_open(next);
    }

    pub fn trigger_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let runtime = self.clone();
        move |_| runtime.toggle()
    }
}

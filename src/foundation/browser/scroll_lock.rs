use oxidase::dom::{ModalShieldGuard, ModalShieldMode, ScrollLockOptions};

#[allow(dead_code)]
pub(crate) fn shield_external_dom(keep_ids: &[&str], mode: ModalShieldMode) -> ModalShieldGuard {
    oxidase::dom::shield_others_with_mode(keep_ids, mode)
}

#[allow(dead_code)]
pub(crate) fn acquire_scroll_lock(lock_id: &str) {
    oxidase::dom::acquire_scroll_lock(lock_id);
}

pub(crate) fn acquire_scroll_lock_with_options(lock_id: &str, prevent_body_pointer_events: bool) {
    oxidase::dom::acquire_scroll_lock_with_options(
        lock_id,
        ScrollLockOptions {
            prevent_body_pointer_events,
        },
    );
}

pub(crate) fn is_body_pointer_events_locked() -> bool {
    oxidase::dom::is_body_pointer_events_locked()
}

pub(crate) fn release_scroll_lock(lock_id: &str, restore_delay: Option<u64>) {
    oxidase::dom::release_scroll_lock(lock_id, restore_delay);
}

pub(crate) fn set_body_user_select_none() {
    let doc = oxidase::dom::document();
    doc.set_body_user_select_none();
}

pub(crate) fn restore_body_user_select() {
    let doc = oxidase::dom::document();
    doc.restore_body_user_select();
}

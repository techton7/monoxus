use monoxus::foundation::overlay::{Presence, PresenceController, PresenceState};

#[test]
fn shared_presence_controller_exposes_close_cycle_handoff() {
    let mut controller = PresenceController::new(true).with_retained_mount(true);

    assert!(controller.should_render());
    assert_eq!(controller.state(), PresenceState::Mounted);

    let close = controller.sync(false);
    let first_cycle = close.started_close_cycle().unwrap();
    assert_eq!(close.state(), PresenceState::Suspended);
    assert!(close.should_render());
    assert_eq!(controller.active_close_cycle(), Some(first_cycle));

    let reopen = controller.sync(true);
    assert_eq!(reopen.invalidated_close_cycle(), Some(first_cycle));
    assert_eq!(controller.state(), PresenceState::Mounted);
    assert!(controller.should_render());

    let second_cycle = controller.sync(false).started_close_cycle().unwrap();
    assert_ne!(first_cycle, second_cycle);
    assert!(!controller.complete_close_cycle(first_cycle));
    assert_eq!(controller.state(), PresenceState::Suspended);
    assert!(controller.complete_close_cycle(second_cycle));
    assert_eq!(controller.state(), PresenceState::Unmounted);
    assert!(!controller.should_render());
}

#[test]
fn existing_presence_api_stays_compatible() {
    let mut presence = Presence::new(true).with_retained_mount(true);

    assert_eq!(presence.sync(false), PresenceState::Suspended);
    assert!(presence.complete_unmount());
    assert_eq!(presence.state(), PresenceState::Unmounted);
}

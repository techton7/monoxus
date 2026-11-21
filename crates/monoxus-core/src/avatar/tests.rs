use super::*;

#[test]
fn status_machine_preserves_idle_and_generation_order() {
    let mut state = AvatarStateMachine::new();
    assert_eq!(state.status(), AvatarImageLoadingStatus::Idle);
    let first = state.begin_generation();
    assert_eq!(
        state.transition(first, AvatarImageLoadingStatus::Loading),
        Some(AvatarImageLoadingStatus::Loading)
    );
    let second = state.begin_generation();
    assert_eq!(
        state.transition(first, AvatarImageLoadingStatus::Loaded),
        None
    );
    assert_eq!(
        state.transition(second, AvatarImageLoadingStatus::Loaded),
        Some(AvatarImageLoadingStatus::Loaded)
    );
}

#[test]
fn transitions_are_idempotent_and_errors_publish_once() {
    let mut state = AvatarStateMachine::new();
    let generation = state.begin_generation();
    assert_eq!(
        state.transition(generation, AvatarImageLoadingStatus::Error),
        Some(AvatarImageLoadingStatus::Error)
    );
    assert_eq!(
        state.transition(generation, AvatarImageLoadingStatus::Error),
        None
    );
}

#[test]
fn fallback_delay_cancels_stale_generations_and_loaded_hides_it() {
    let mut state = AvatarFallbackState::new();
    let stale = state.begin(false);
    let current = state.begin(false);
    assert!(!state.finish(stale));
    assert!(state.finish(current));
    assert!(state.is_visible(AvatarImageLoadingStatus::Error));
    assert!(!state.is_visible(AvatarImageLoadingStatus::Loaded));
}

#[test]
fn attributes_publish_status_and_part_hooks() {
    assert_eq!(
        AvatarRootAttributes::new(AvatarImageLoadingStatus::Loading).data_status,
        "loading"
    );
    assert_eq!(
        AvatarImageAttributes::new(AvatarImageLoadingStatus::Loaded).data_avatar_image,
        ""
    );
    assert_eq!(
        AvatarFallbackAttributes::new(AvatarImageLoadingStatus::Error).data_avatar_fallback,
        ""
    );
}

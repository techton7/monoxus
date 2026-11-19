use std::{cell::RefCell, rc::Rc};
use std::{future::Future, pin::pin, task::Context, time::Duration};

use dioxus::prelude::*;
use monoxus::avatar::{
    AVATAR_PARTS, Avatar, AvatarFallback, AvatarImage, AvatarImageLoadingStatus, AvatarPart,
    AvatarRoot, AvatarStateMachine, Fallback, Image, project_avatar_fallback_as_child,
    project_avatar_image_as_child, project_avatar_root_as_child,
};
use monoxus::foundation::compose::Slottable;

fn poll_once<F: Future>(future: F) {
    struct ThreadWaker(std::thread::Thread);
    impl std::task::Wake for ThreadWaker {
        fn wake(self: std::sync::Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = std::task::Waker::from(std::sync::Arc::new(ThreadWaker(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    let deadline = std::time::Instant::now() + Duration::from_millis(100);
    loop {
        if future.as_mut().poll(&mut context).is_ready() {
            break;
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        std::thread::park_timeout(remaining);
    }
}

#[test]
fn avatar_public_parts_and_aliases_are_stable() {
    assert_eq!(
        AVATAR_PARTS,
        [AvatarPart::Root, AvatarPart::Image, AvatarPart::Fallback]
    );
    assert_eq!(AVATAR_PARTS[0].as_str(), "root");
    assert_eq!(AVATAR_PARTS[1].as_str(), "image");
    assert_eq!(AVATAR_PARTS[2].as_str(), "fallback");
}

#[test]
fn avatar_as_child_projection_preserves_one_target_and_attributes() {
    let status = AvatarImageLoadingStatus::Error;
    let root = monoxus::avatar::AvatarRootAttributes::new(status);
    let (target, attrs, content) =
        project_avatar_root_as_child(&root, "section", Slottable::new("fallback"));
    assert_eq!(target, "section");
    assert_eq!(content, "fallback");
    assert_eq!(attrs.data_status, "error");

    let (_, image, _) = project_avatar_image_as_child(
        &monoxus::avatar::AvatarImageAttributes::new(AvatarImageLoadingStatus::Loaded),
        "img",
        Slottable::new(()),
    );
    assert_eq!(image.data_status, "loaded");

    let (_, fallback, _) = project_avatar_fallback_as_child(
        &monoxus::avatar::AvatarFallbackAttributes::new(status),
        "span",
        Slottable::new(()),
    );
    assert_eq!(fallback.data_status, "error");
}

#[test]
fn stale_source_results_cannot_publish_after_replacement() {
    let mut machine = AvatarStateMachine::new();
    let old = machine.begin_generation();
    assert!(
        machine
            .transition(old, AvatarImageLoadingStatus::Loading)
            .is_some()
    );
    let new = machine.begin_generation();
    assert_eq!(
        machine.transition(old, AvatarImageLoadingStatus::Loaded),
        None
    );
    assert_eq!(
        machine.transition(new, AvatarImageLoadingStatus::Error),
        Some(AvatarImageLoadingStatus::Error)
    );
}

#[component]
fn AvatarMountScenario(statuses: Rc<RefCell<Vec<AvatarImageLoadingStatus>>>) -> Element {
    rsx! {
        AvatarRoot {
            on_loading_status_change: move |status| statuses.borrow_mut().push(status),
            AvatarImage { src: "".to_string() }
            AvatarFallback { delay_ms: 0, "fallback" }
        }
    }
}

#[test]
fn empty_source_mounts_and_publishes_error() {
    let statuses = Rc::new(RefCell::new(Vec::new()));
    let mut dom = VirtualDom::new_with_props(
        AvatarMountScenario,
        AvatarMountScenarioProps {
            statuses: statuses.clone(),
        },
    );
    dom.rebuild_in_place();
    poll_once(dom.wait_for_work());
    dom.render_immediate(&mut dioxus_core::NoOpMutations);
    assert_eq!(&*statuses.borrow(), &[AvatarImageLoadingStatus::Error]);
}

#[component]
fn ControlledAvatarScenario() -> Element {
    rsx! {
        Avatar {
            loading_status: AvatarImageLoadingStatus::Loaded,
            Image { src: "portrait.png".to_string() }
            Fallback { "fallback" }
        }
    }
}

#[test]
fn controlled_loaded_avatar_mounts_image_and_aliases() {
    let mut dom = VirtualDom::new(ControlledAvatarScenario);
    dom.rebuild_in_place();
}

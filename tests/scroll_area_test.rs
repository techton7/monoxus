use monoxus::foundation::compose::project_attrs_as_child;
use dioxus::prelude::*;
use monoxus::foundation::compose::Slottable;
use monoxus::foundation::shared::{Direction, Orientation};
use monoxus::scroll_area::*;

#[component]
fn BasicScrollAreaScenario() -> Element {
    rsx! {
        ScrollAreaRoot {
            id: "contract-scroll-root",
            r#type: ScrollAreaType::Hover,
            dir: Direction::Ltr,
            scroll_hide_delay: 600,
            ScrollAreaViewport {
                id: "contract-scroll-viewport",
                nonce: "contract-nonce-abc".to_string(),
                div {
                    style: "height: 5000px; width: 3000px;",
                    "Scrollable massive content"
                }
            }
            ScrollAreaScrollbar {
                id: "contract-scrollbar-y",
                orientation: Orientation::Vertical,
                force_mount: true,
                ScrollAreaThumb {
                    id: "contract-thumb-y",
                    force_mount: true,
                }
            }
            ScrollAreaScrollbar {
                id: "contract-scrollbar-x",
                orientation: Orientation::Horizontal,
                force_mount: true,
                ScrollAreaThumb {
                    id: "contract-thumb-x",
                    force_mount: true,
                }
            }
            ScrollAreaCorner {
                id: "contract-corner",
            }
        }
    }
}

#[component]
fn RtlScrollAreaScenario() -> Element {
    rsx! {
        ScrollArea {
            id: "rtl-scroll-root",
            r#type: ScrollAreaType::Always,
            dir: Direction::Rtl,
            ScrollAreaViewport {
                id: "rtl-scroll-viewport",
                div {
                    style: "height: 2000px; width: 2000px;",
                    "RTL content"
                }
            }
            ScrollAreaScrollbar {
                orientation: Orientation::Horizontal,
                ScrollAreaThumb {}
            }
            ScrollAreaScrollbar {
                orientation: Orientation::Vertical,
                ScrollAreaThumb {}
            }
            ScrollAreaCorner {}
        }
    }
}

#[component]
fn ScrollModeScenario() -> Element {
    rsx! {
        ScrollAreaRoot {
            id: "scroll-mode-root",
            r#type: ScrollAreaType::Scroll,
            scroll_hide_delay: 400,
            ScrollAreaViewport {
                id: "scroll-mode-viewport",
                div { style: "height: 1000px;", "Content" }
            }
            ScrollAreaScrollbar {
                orientation: Orientation::Vertical,
                ScrollAreaThumb {}
            }
        }
    }
}

#[component]
fn AutoModeScenario() -> Element {
    rsx! {
        ScrollAreaRoot {
            id: "auto-mode-root",
            r#type: ScrollAreaType::Auto,
            ScrollAreaViewport {
                id: "auto-mode-viewport",
                div { style: "height: 800px;", "Content" }
            }
            ScrollAreaScrollbar {
                orientation: Orientation::Vertical,
                ScrollAreaThumb {}
            }
        }
    }
}

#[test]
fn test_scroll_area_public_contract_renders_basic_scenario() {
    let mut dom = VirtualDom::new(BasicScrollAreaScenario);
    dom.rebuild_in_place();
}

#[test]
fn test_scroll_area_rtl_and_always_mode_renders() {
    let mut dom = VirtualDom::new(RtlScrollAreaScenario);
    dom.rebuild_in_place();
}

#[test]
fn test_scroll_area_scroll_and_auto_modes_render() {
    let mut dom_scroll = VirtualDom::new(ScrollModeScenario);
    dom_scroll.rebuild_in_place();

    let mut dom_auto = VirtualDom::new(AutoModeScenario);
    dom_auto.rebuild_in_place();
}

#[test]
fn test_scroll_area_as_child_projections() {
    let root_attrs = ScrollAreaRootAttributes::new("p-root").with_dir(Some(Direction::Ltr));
    let (target, attrs, content) = project_attrs_as_child(
        &root_attrs,
        "div-target",
        Slottable::new("inner-content"),
    );
    assert_eq!(target, "div-target");
    assert_eq!(attrs.id(), "p-root");
    assert_eq!(attrs.dir_str(), Some("ltr"));
    assert_eq!(content, "inner-content");

    let vp_attrs = ScrollAreaViewportAttributes::new("p-viewport");
    let (target, attrs, content) =
        project_attrs_as_child(&vp_attrs, "main-target", Slottable::new(42_u32));
    assert_eq!(target, "main-target");
    assert_eq!(attrs.id(), "p-viewport");
    assert_eq!(attrs.tabindex(), 0);
    assert_eq!(content, 42_u32);

    let cnt_attrs = ScrollAreaContentAttributes::new("p-content");
    let (target, attrs, content) =
        project_attrs_as_child(&cnt_attrs, "section-target", Slottable::new("body"));
    assert_eq!(target, "section-target");
    assert_eq!(attrs.id(), "p-content");
    assert_eq!(attrs.role(), "presentation");
    assert_eq!(content, "body");

    let sb_attrs =
        ScrollAreaScrollbarAttributes::new("p-scrollbar", Orientation::Vertical, "p-viewport")
            .with_visible(true);
    let (target, attrs, content) =
        project_attrs_as_child(&sb_attrs, "aside-target", Slottable::new("sb"));
    assert_eq!(target, "aside-target");
    assert_eq!(attrs.role(), "scrollbar");
    assert_eq!(attrs.data_orientation(), "vertical");
    assert_eq!(attrs.data_state(), "visible");
    assert_eq!(content, "sb");

    let th_attrs = ScrollAreaThumbAttributes::new("p-thumb").with_visible(true);
    let (target, attrs, content) =
        project_attrs_as_child(&th_attrs, "span-target", Slottable::new("th"));
    assert_eq!(target, "span-target");
    assert_eq!(attrs.data_state(), "visible");
    assert_eq!(content, "th");

    let cn_attrs = ScrollAreaCornerAttributes::new("p-corner");
    let (target, attrs, content) =
        project_attrs_as_child(&cn_attrs, "div-corner", Slottable::new("cn"));
    assert_eq!(target, "div-corner");
    assert_eq!(attrs.data_state(), "visible");
    assert_eq!(content, "cn");
}

#[test]
fn test_scroll_area_flexbox_rules_contract() {
    let sizes = ScrollAreaSizes::new(1000.0, 200.0, 200.0, ScrollbarPadding::default());
    assert_eq!(get_thumb_size(sizes), 40.0);
    assert_eq!(SCROLL_AREA_MIN_THUMB_SIZE, 18.0);

    let small_sizes = ScrollAreaSizes::new(10000.0, 50.0, 50.0, ScrollbarPadding::default());
    assert_eq!(get_thumb_size(small_sizes), 18.0);
}

#[test]
fn test_scroll_area_corner_force_mount_and_reset_to_zero() {
    use std::cell::RefCell;
    use std::rc::Rc;

    #[component]
    fn CornerProbe(
        runtime_holder: Rc<RefCell<Option<ScrollAreaRuntime>>>,
        show_corner: Signal<bool>,
    ) -> Element {
        let ctx = use_context::<ScrollAreaContext>();
        *runtime_holder.borrow_mut() = Some(ctx.runtime);
        rsx! {
            if show_corner() {
                ScrollAreaCorner {
                    id: "fm-col-3-corner",
                    force_mount: true,
                }
            }
        }
    }

    #[component]
    fn CornerResetHost(
        runtime_holder: Rc<RefCell<Option<ScrollAreaRuntime>>>,
        toggle_holder: Rc<RefCell<Option<Signal<bool>>>>,
    ) -> Element {
        let show_corner = use_signal(|| true);
        *toggle_holder.borrow_mut() = Some(show_corner);
        rsx! {
            ScrollAreaRoot {
                id: "fm-col-3-root",
                r#type: ScrollAreaType::Auto,
                ScrollAreaViewport {
                    id: "fm-col-3-vp",
                    div { "content" }
                }
                CornerProbe {
                    runtime_holder,
                    show_corner,
                }
            }
        }
    }

    let rt_holder = Rc::new(RefCell::new(None));
    let toggle_holder = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new_with_props(
        CornerResetHost,
        CornerResetHostProps {
            runtime_holder: rt_holder.clone(),
            toggle_holder: toggle_holder.clone(),
        },
    );
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let mut rt = rt_holder.borrow().expect("ScrollAreaRuntime captured");
    let mut show_corner = toggle_holder.borrow().expect("toggle signal captured");

    // FM-COL-3 / radix#2383/#4009: unmounting ScrollAreaCorner must reset corner dimensions to 0.0 via try_write().
    dom.in_scope(ScopeId::APP, || {
        rt.corner_width.set(16.0);
        rt.corner_height.set(12.0);
        assert_eq!(rt.corner_width(), 16.0);
        assert_eq!(rt.corner_height(), 12.0);
        show_corner.set(false);
    });
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    dom.in_scope(ScopeId::APP, || {
        assert_eq!(rt.corner_width(), 0.0);
        assert_eq!(rt.corner_height(), 0.0);
    });
}

#[cfg(feature = "native")]
#[test]
fn test_native_scroll_area_corner_force_mount_renders_explicitly() {
    use crate::common::native_fixture::{NATIVE_TEST_LOCK, mount_native_scenario};

    #[component]
    fn ForceMountCornerScenario() -> Element {
        rsx! {
            ScrollAreaRoot {
                id: "fm-native-sa-root",
                r#type: ScrollAreaType::Auto,
                ScrollAreaViewport {
                    id: "fm-native-sa-vp",
                    div { "Small non-overflowing content" }
                }
                ScrollAreaCorner {
                    id: "fm-native-sa-corner",
                    force_mount: true,
                }
            }
        }
    }

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let harness = mount_native_scenario(ForceMountCornerScenario);
    assert!(
        harness.query("#fm-native-sa-corner").is_some(),
        "ScrollAreaCorner with force_mount: true must mount even when scrollbars are not visible"
    );
}

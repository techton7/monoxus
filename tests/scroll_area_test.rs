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

#[cfg(feature = "native")]
mod native_w2_scroll_tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use blitz_traits::events::{
        BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, PointerCoords,
        PointerDetails, UiEvent,
    };

    use crate::common::native_fixture::*;

    fn make_mouse_pointer_event(
        x: f32,
        y: f32,
        button: MouseEventButton,
        buttons: MouseEventButtons,
    ) -> BlitzPointerEvent {
        BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                screen_x: x,
                screen_y: y,
                client_x: x,
                client_y: y,
            },
            button,
            buttons,
            mods: Default::default(),
            details: PointerDetails::default(),
            element: Default::default(),
            active_pointers: Default::default(),
        }
    }

    #[component]
    fn ScrollMetricsProbe(id: String) -> Element {
        let ctx = use_context::<ScrollAreaContext>();
        let scroll_top = ctx
            .runtime
            .viewport_metrics()
            .map(|m| m.scroll_top)
            .unwrap_or(0.0);
        let corner_w = ctx.runtime.corner_width();
        let corner_h = ctx.runtime.corner_height();
        rsx! {
            div {
                id: "{id}",
                "data-scroll-top": "{scroll_top}",
                "data-corner-w": "{corner_w}",
                "data-corner-h": "{corner_h}",
            }
        }
    }

    #[component]
    fn ReactiveScrollAreaScenario() -> Element {
        let mut scroll_type = use_signal(|| ScrollAreaType::Hover);
        let mut dir = use_signal(|| Direction::Ltr);

        rsx! {
            div {
                button {
                    id: "scr-set-always",
                    r#type: "button",
                    onclick: move |_| scroll_type.set(ScrollAreaType::Always),
                    "Always"
                }
                button {
                    id: "scr-set-rtl",
                    r#type: "button",
                    onclick: move |_| dir.set(Direction::Rtl),
                    "RTL"
                }
                button {
                    id: "scr-set-scroll",
                    r#type: "button",
                    onclick: move |_| scroll_type.set(ScrollAreaType::Scroll),
                    "Scroll"
                }
                ScrollAreaRoot {
                    id: "scr-react-root",
                    r#type: scroll_type(),
                    dir: dir(),
                    style: "width: 200px; height: 200px;",
                    ScrollAreaViewport {
                        id: "scr-react-vp",
                        style: "width: 200px; height: 200px;",
                        div {
                            style: "width: 600px; height: 600px;",
                            "Overflowing content"
                        }
                    }
                    ScrollAreaScrollbar {
                        id: "scr-react-sb-y",
                        orientation: Orientation::Vertical,
                        style: "width: 12px;",
                        ScrollAreaThumb { id: "scr-react-th-y" }
                    }
                    ScrollAreaScrollbar {
                        id: "scr-react-sb-x",
                        orientation: Orientation::Horizontal,
                        style: "height: 12px;",
                        ScrollAreaThumb { id: "scr-react-th-x" }
                    }
                    ScrollAreaCorner { id: "scr-react-corner" }
                }
            }
        }
    }

    #[test]
    fn test_native_scroll_area_root_reactive_type_and_dir_propagation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(ReactiveScrollAreaScenario);
        advance_frame_and_pump(&mut harness, 0.02);

        // Initial: Hover (not hovered) + LTR -> scrollbar not rendered
        assert_eq!(
            harness.attr("#scr-react-root", "dir").as_deref(),
            Some("ltr")
        );
        assert!(
            harness.query("#scr-react-sb-y").is_none(),
            "FM-W2-SCR: Hover mode scrollbar must start hidden before pointer enter"
        );

        // Post-mount r#type: Hover -> Always propagates to scrollbars & corner
        click_and_pump(&mut harness, "#scr-set-always");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#scr-react-sb-y").is_some(),
            "FM-W2-SCR: post-mount ScrollAreaType::Always must mount vertical scrollbar"
        );
        assert!(
            harness.query("#scr-react-corner").is_some(),
            "FM-W2-SCR: post-mount ScrollAreaType::Always with both scrollbars must mount corner"
        );
        let sb_style_ltr = harness.attr("#scr-react-sb-y", "style").unwrap_or_default();
        assert!(
            sb_style_ltr.contains("right: 0;") && sb_style_ltr.contains("left: auto;"),
            "FM-W2-SCR: LTR vertical scrollbar must align right: 0; left: auto;, got {sb_style_ltr}"
        );
        let corner_style_ltr = harness
            .attr("#scr-react-corner", "style")
            .unwrap_or_default();
        assert!(
            corner_style_ltr.contains("right: 0;") && corner_style_ltr.contains("left: auto;"),
            "FM-W2-SCR: LTR corner must align right: 0; left: auto;, got {corner_style_ltr}"
        );

        // Post-mount dir: Ltr -> Rtl propagates to scrollbar & corner styles
        click_and_pump(&mut harness, "#scr-set-rtl");
        assert_eq!(
            harness.attr("#scr-react-root", "dir").as_deref(),
            Some("rtl")
        );
        let sb_style_rtl = harness.attr("#scr-react-sb-y", "style").unwrap_or_default();
        assert!(
            sb_style_rtl.contains("right: auto;") && sb_style_rtl.contains("left: 0;"),
            "FM-W2-SCR: RTL vertical scrollbar must flip to right: auto; left: 0;, got {sb_style_rtl}"
        );
        let corner_style_rtl = harness
            .attr("#scr-react-corner", "style")
            .unwrap_or_default();
        assert!(
            corner_style_rtl.contains("right: auto;") && corner_style_rtl.contains("left: 0;"),
            "FM-W2-SCR: RTL corner must flip to right: auto; left: 0;, got {corner_style_rtl}"
        );

        // Post-mount r#type: Always -> Scroll hides corner (has_corner returns false for Scroll)
        click_and_pump(&mut harness, "#scr-set-scroll");
        assert!(
            harness.query("#scr-react-corner").is_none(),
            "FM-W2-SCR: post-mount ScrollAreaType::Scroll must hide corner"
        );
    }

    #[component]
    fn ThumbRightClickScenario() -> Element {
        rsx! {
            div {
                style: "padding: 20px;",
                ScrollAreaRoot {
                    id: "scr-rc-root",
                    r#type: ScrollAreaType::Always,
                    style: "width: 200px; height: 200px;",
                    ScrollMetricsProbe { id: "scr-rc-probe".to_string() }
                    ScrollAreaViewport {
                        id: "scr-rc-vp",
                        style: "width: 200px; height: 200px;",
                        div {
                            style: "width: 180px; height: 1000px;",
                            "Tall scrollable content"
                        }
                    }
                    ScrollAreaScrollbar {
                        id: "scr-rc-sb-y",
                        orientation: Orientation::Vertical,
                        style: "width: 16px; height: 200px;",
                        ScrollAreaThumb {
                            id: "scr-rc-th-y",
                            style: "background-color: #888;",
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_scroll_area_thumb_right_click_does_not_swallow_primary_track_click() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(ThumbRightClickScenario);
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);

        let sb_rect = harness.layout_rect("#scr-rc-sb-y");
        let (thumb_x, thumb_y) = harness.center_of("#scr-rc-th-y");

        // 1. Right-click (Secondary mouse button) on the thumb
        harness.dispatch(UiEvent::PointerDown(make_mouse_pointer_event(
            thumb_x,
            thumb_y,
            MouseEventButton::Secondary,
            MouseEventButtons::Secondary,
        )));
        drain_polls_and_resolve(&mut harness);
        harness.dispatch(UiEvent::PointerUp(make_mouse_pointer_event(
            thumb_x,
            thumb_y,
            MouseEventButton::Secondary,
            MouseEventButtons::empty(),
        )));
        drain_polls_and_resolve(&mut harness);

        let scroll_after_right_click: f64 = harness
            .attr("#scr-rc-probe", "data-scroll-top")
            .unwrap_or_default()
            .parse()
            .unwrap_or(-1.0);
        assert_eq!(
            scroll_after_right_click, 0.0,
            "FM-W2-SCR: right-clicking thumb must not scroll viewport"
        );

        // 2. Next primary click on lower scrollbar track must scroll viewport (must NOT be swallowed)
        let track_x = sb_rect.x + sb_rect.width * 0.5;
        let track_y = sb_rect.y + sb_rect.height * 0.80;
        click_at_and_pump(&mut harness, track_x, track_y);

        let scroll_after_track_click: f64 = harness
            .attr("#scr-rc-probe", "data-scroll-top")
            .unwrap_or_default()
            .parse()
            .unwrap_or(0.0);
        assert!(
            scroll_after_track_click > 100.0,
            "FM-W2-SCR: primary track click after thumb right-click must scroll viewport, got scroll_top={scroll_after_track_click}"
        );
    }

    #[derive(Clone, PartialEq, Props)]
    struct StaleCornerScenarioProps {
        controls: Rc<RefCell<Option<(Signal<ScrollAreaType>, Signal<bool>)>>>,
    }

    #[component]
    fn StaleCornerScenario(props: StaleCornerScenarioProps) -> Element {
        let scroll_type = use_signal(|| ScrollAreaType::Always);
        let mount_sb = use_signal(|| true);
        *props.controls.borrow_mut() = Some((scroll_type, mount_sb));

        rsx! {
            ScrollAreaRoot {
                id: "scr-stale-root",
                r#type: scroll_type(),
                style: "width: 200px; height: 200px;",
                ScrollMetricsProbe { id: "scr-stale-probe".to_string() }
                ScrollAreaViewport {
                    id: "scr-stale-vp",
                    style: "width: 200px; height: 200px;",
                    div {
                        style: "width: 180px; height: 1000px;",
                        "Tall content"
                    }
                }
                if mount_sb() {
                    ScrollAreaScrollbar {
                        id: "scr-stale-sb-y",
                        orientation: Orientation::Vertical,
                        style: "width: 14px; height: 200px;",
                        ScrollAreaThumb { id: "scr-stale-th-y" }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_scroll_area_hiding_or_unmounting_scrollbar_cancels_stale_corner_write() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // Case 1: Hide scrollbar (Always -> Hover) before next_frame resolves
        {
            let controls = Rc::new(RefCell::new(None));
            let mut harness = mount_native_scenario_with_props(
                StaleCornerScenario,
                StaleCornerScenarioProps {
                    controls: Rc::clone(&controls),
                },
            );
            let (mut scroll_type, _) = controls.borrow().expect("controls captured");
            harness.doc.vdom.in_scope(ScopeId::APP, || {
                scroll_type.set(ScrollAreaType::Hover);
            });
            drain_polls_and_resolve(&mut harness);
            assert_eq!(
                harness.attr("#scr-stale-sb-y", "data-state").as_deref(),
                Some("hidden"),
                "FM-W2-SCR: switching Always -> Hover must transition scrollbar to hidden state"
            );

            // Flush presence exit frame + pending next_frame().await spawned during mount
            advance_frame_and_pump(&mut harness, 0.02);
            advance_frame_and_pump(&mut harness, 0.02);
            assert!(
                harness.query("#scr-stale-sb-y").is_none(),
                "FM-W2-SCR: hidden scrollbar without exit animation must unmount after presence frame"
            );
            let corner_w: f64 = harness
                .attr("#scr-stale-probe", "data-corner-w")
                .unwrap_or_default()
                .parse()
                .unwrap_or(-1.0);
            assert_eq!(
                corner_w, 0.0,
                "FM-W2-SCR: hiding scrollbar before next_frame must keep corner_width at 0.0"
            );
        }

        // Case 2: Unmount scrollbar before next_frame resolves (must not panic or write corner_width)
        {
            let controls = Rc::new(RefCell::new(None));
            let mut harness = mount_native_scenario_with_props(
                StaleCornerScenario,
                StaleCornerScenarioProps {
                    controls: Rc::clone(&controls),
                },
            );
            let (_, mut mount_sb) = controls.borrow().expect("controls captured");
            harness.doc.vdom.in_scope(ScopeId::APP, || {
                mount_sb.set(false);
            });
            drain_polls_and_resolve(&mut harness);
            assert!(harness.query("#scr-stale-sb-y").is_none());

            // Flush pending next_frame().await -> must not panic or write stale corner width
            advance_frame_and_pump(&mut harness, 0.02);
            let corner_w: f64 = harness
                .attr("#scr-stale-probe", "data-corner-w")
                .unwrap_or_default()
                .parse()
                .unwrap_or(-1.0);
            assert_eq!(
                corner_w, 0.0,
                "FM-W2-SCR: unmounting scrollbar before next_frame must cancel stale corner write without panic"
            );
        }
    }
}

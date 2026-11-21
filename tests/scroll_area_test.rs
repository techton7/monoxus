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
    let (target, attrs, content) = project_scroll_area_root_as_child(
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
        project_scroll_area_viewport_as_child(&vp_attrs, "main-target", Slottable::new(42_u32));
    assert_eq!(target, "main-target");
    assert_eq!(attrs.id(), "p-viewport");
    assert_eq!(attrs.tabindex(), 0);
    assert_eq!(content, 42_u32);

    let cnt_attrs = ScrollAreaContentAttributes::new("p-content");
    let (target, attrs, content) =
        project_scroll_area_content_as_child(&cnt_attrs, "section-target", Slottable::new("body"));
    assert_eq!(target, "section-target");
    assert_eq!(attrs.id(), "p-content");
    assert_eq!(attrs.role(), "presentation");
    assert_eq!(content, "body");

    let sb_attrs =
        ScrollAreaScrollbarAttributes::new("p-scrollbar", Orientation::Vertical, "p-viewport")
            .with_visible(true);
    let (target, attrs, content) =
        project_scroll_area_scrollbar_as_child(&sb_attrs, "aside-target", Slottable::new("sb"));
    assert_eq!(target, "aside-target");
    assert_eq!(attrs.role(), "scrollbar");
    assert_eq!(attrs.data_orientation(), "vertical");
    assert_eq!(attrs.data_state(), "visible");
    assert_eq!(content, "sb");

    let th_attrs = ScrollAreaThumbAttributes::new("p-thumb").with_visible(true);
    let (target, attrs, content) =
        project_scroll_area_thumb_as_child(&th_attrs, "span-target", Slottable::new("th"));
    assert_eq!(target, "span-target");
    assert_eq!(attrs.data_state(), "visible");
    assert_eq!(content, "th");

    let cn_attrs = ScrollAreaCornerAttributes::new("p-corner");
    let (target, attrs, content) =
        project_scroll_area_corner_as_child(&cn_attrs, "div-corner", Slottable::new("cn"));
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

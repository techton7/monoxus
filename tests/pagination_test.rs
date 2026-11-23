use dioxus::prelude::*;
use monoxus::{
    foundation::{
        compose::Slottable,
        roving_focus::RovingModifiers,
        shared::{CollectionRegistry, Direction, Orientation, ScopeHandle},
    },
    pagination::{
        NEXT_ROVING_KEY, PAGINATION_PARTS, PREV_ROVING_KEY, PageData, PageItem, PageRange,
        PaginationDefinition, PaginationItemRegistration, PaginationNextButton, PaginationPage,
        PaginationPart, PaginationPrevButton, PaginationRelationships, PaginationRoot,
        PaginationRootRenderChildProps, PaginationSnapshot, clamp_page, compute_page_items,
        compute_range, compute_total_pages, page_roving_key, project_pagination_next_button_as_child,
        project_pagination_page_as_child, project_pagination_prev_button_as_child,
        project_pagination_root_as_child, use_pagination_context,
    },
};

#[cfg(feature = "native")]
use crate::common;

#[test]
fn test_pagination_parts_and_relationships() {
    assert_eq!(
        PAGINATION_PARTS
            .iter()
            .map(PaginationPart::as_str)
            .collect::<Vec<_>>(),
        ["root", "prev-button", "next-button", "page"]
    );

    let scope = ScopeHandle::root("pagination").child("contract");
    let rel = PaginationRelationships::new(scope.clone());
    assert_eq!(rel.root_id(), scope.token());
    assert_eq!(rel.prev_id(), scope.qualify("prev"));
    assert_eq!(rel.next_id(), scope.qualify("next"));
    assert_eq!(rel.page_id(1), scope.qualify("page-1"));
    assert_eq!(PREV_ROVING_KEY, "prev");
    assert_eq!(NEXT_ROVING_KEY, "next");
    assert_eq!(page_roving_key(3), "page-3");
}

#[test]
fn test_pagination_arithmetic_zero_guards_and_range() {
    assert_eq!(compute_total_pages(0, 10), 1);
    assert_eq!(compute_total_pages(0, 0), 1);
    assert_eq!(compute_total_pages(25, 0), 25);
    assert_eq!(compute_total_pages(25, 10), 3);
    assert_eq!(clamp_page(0, 3), 1);
    assert_eq!(clamp_page(99, 3), 3);

    assert_eq!(compute_range(1, 10, 0), PageRange { start: 0, end: 0 });
    assert_eq!(compute_range(1, 10, 100), PageRange { start: 1, end: 10 });
    assert_eq!(compute_range(5, 10, 100), PageRange { start: 41, end: 50 });
    assert_eq!(compute_range(10, 10, 95), PageRange { start: 91, end: 95 });
}

#[test]
fn test_pagination_page_items_and_deterministic_keys() {
    let p1 = compute_page_items(1, 10, 1);
    assert_eq!(
        p1.iter().map(PageItem::key).collect::<Vec<_>>(),
        ["page-1", "page-2", "page-3", "page-4", "ellipsis-0", "page-10"]
    );

    let p5 = compute_page_items(5, 10, 1);
    assert_eq!(
        p5.iter().map(PageItem::key).collect::<Vec<_>>(),
        [
            "page-1",
            "ellipsis-0",
            "page-4",
            "page-5",
            "page-6",
            "ellipsis-1",
            "page-10"
        ]
    );

    // Exhaustive check: zero duplicates, zero contiguous ellipses, no usize underflow.
    for total_pages in 1..=20 {
        for sibling_count in 0..=2 {
            for page in 1..=total_pages {
                let items = compute_page_items(page, total_pages, sibling_count);
                let mut prev_ellipsis = false;
                let mut prev_page = 0usize;
                for item in &items {
                    match item {
                        PageItem::Ellipsis { .. } => {
                            assert!(!prev_ellipsis);
                            prev_ellipsis = true;
                        }
                        PageItem::Page { value, .. } => {
                            assert!(*value > prev_page);
                            prev_page = *value;
                            prev_ellipsis = false;
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_pagination_attributes_and_roving_contract() {
    let scope = ScopeHandle::root("pagination").child("attrs");
    let def = PaginationDefinition::new(scope, 30)
        .with_per_page(10)
        .with_page(1)
        .with_direction(Direction::Rtl)
        .with_orientation(Orientation::Horizontal)
        .with_loop_focus(true)
        .with_aria_label("Search results");

    let root = def.root_attributes(None);
    assert_eq!(root.role(), "navigation");
    assert_eq!(root.aria_label(), "Search results");
    assert_eq!(root.dir(), "rtl");
    assert_eq!(root.data_orientation(), "horizontal");
    assert_eq!(root.data_monoxus_pagination_root(), "");

    let active_page = def.page_attributes(1, None, false);
    assert_eq!(active_page.aria_current(), Some("page"));
    assert_eq!(active_page.data_selected(), Some(""));
    assert_eq!(active_page.data_disabled(), None);
    assert_eq!(active_page.aria_disabled(), None);

    let inactive_page = def.page_attributes(2, None, false);
    assert_eq!(inactive_page.aria_current(), None);
    assert_eq!(inactive_page.data_selected(), None);
    assert_eq!(inactive_page.data_disabled(), None);

    let prev = def.prev_button_attributes(None, false);
    assert!(prev.is_disabled());
    assert_eq!(prev.aria_disabled(), Some("true"));
    assert_eq!(prev.data_disabled(), Some(""));

    let next = def.next_button_attributes(None, false);
    assert!(!next.is_disabled());
    assert_eq!(next.aria_disabled(), None);
    assert_eq!(next.data_disabled(), None);

    let (_, proj_root, _) =
        project_pagination_root_as_child(&root, "nav-slot", Slottable::new("body"));
    assert_eq!(proj_root, root);
    let (_, proj_page, _) =
        project_pagination_page_as_child(&active_page, "btn-slot", Slottable::new("1"));
    assert_eq!(proj_page, active_page);
    let (_, proj_prev, _) =
        project_pagination_prev_button_as_child(&prev, "prev-slot", Slottable::new("Prev"));
    assert_eq!(proj_prev, prev);
    let (_, proj_next, _) =
        project_pagination_next_button_as_child(&next, "next-slot", Slottable::new("Next"));
    assert_eq!(proj_next, next);

    let mut reg = CollectionRegistry::new();
    reg.register(
        PREV_ROVING_KEY.to_string(),
        PaginationItemRegistration {
            key: PREV_ROVING_KEY.to_string(),
            id: "prev".to_string(),
            disabled: true,
        },
    );
    reg.register(
        "page-1".to_string(),
        PaginationItemRegistration {
            key: "page-1".to_string(),
            id: "page-1".to_string(),
            disabled: false,
        },
    );
    reg.register(
        NEXT_ROVING_KEY.to_string(),
        PaginationItemRegistration {
            key: NEXT_ROVING_KEY.to_string(),
            id: "next".to_string(),
            disabled: false,
        },
    );

    assert_eq!(
        def.navigate_key(&reg, Some("page-1"), "ArrowLeft", RovingModifiers::none()),
        Some("next".to_string())
    );
    assert_eq!(
        def.navigate_key(&reg, Some("page-1"), "ArrowRight", RovingModifiers::none()),
        Some("next".to_string())
    );
}

#[component]
fn PaginationContextProbe() -> Element {
    let ctx = use_pagination_context();
    let snap = ctx.snapshot();
    rsx! {
        div {
            id: "ctx-probe",
            "data-current": "{snap.current_page}",
            "data-total": "{snap.total_pages}",
            PaginationPrevButton { id: "ctx-prev", "Prev" }
            for item in snap.pages.iter() {
                match item {
                    PageItem::Page { value, key } => rsx! {
                        PaginationPage {
                            key: "{key}",
                            id: "ctx-page-{value}",
                            page: PageData::new(*value),
                        }
                    },
                    PageItem::Ellipsis { key } => rsx! {
                        span { key: "{key}", "..." }
                    },
                }
            }
            PaginationNextButton { id: "ctx-next", "Next" }
        }
    }
}

#[component]
fn PaginationVdomContextApp() -> Element {
    rsx! {
        PaginationRoot {
            id: "vdom-ctx-root",
            count: 50,
            per_page: 10,
            PaginationContextProbe {}
        }
    }
}

#[test]
fn test_pagination_vdom_mounts_with_context_and_render_children() {
    let mut dom = VirtualDom::new(PaginationVdomContextApp);
    dom.rebuild_in_place();

    #[component]
    fn RenderChildrenApp() -> Element {
        rsx! {
            PaginationRoot {
                id: "vdom-render-root",
                count: 100,
                per_page: 10,
                render_children: move |snap: PaginationSnapshot| rsx! {
                    PaginationPrevButton { "Prev" }
                    for item in snap.pages.iter() {
                        if let Some(page) = item.as_page_data() {
                            PaginationPage {
                                key: "{item.key()}",
                                page,
                            }
                        }
                    }
                    PaginationNextButton { "Next" }
                }
            }
        }
    }

    let mut render_dom = VirtualDom::new(RenderChildrenApp);
    render_dom.rebuild_in_place();

    #[component]
    fn AsChildRootApp() -> Element {
        rsx! {
            PaginationRoot {
                id: "vdom-as-child-root",
                count: 20,
                per_page: 10,
                as_child: Some(true),
                render_child: move |rc: PaginationRootRenderChildProps| rsx! {
                    section {
                        id: "{rc.attrs.id}",
                        role: "{rc.attrs.role}",
                        aria_label: "{rc.attrs.aria_label}",
                        "data-total-pages": "{rc.snapshot.total_pages}",
                        {rc.children}
                    }
                },
                PaginationPrevButton { "Prev" }
                PaginationNextButton { "Next" }
            }
        }
    }

    let mut as_child_dom = VirtualDom::new(AsChildRootApp);
    as_child_dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    #[component]
    fn NativeBasicPaginationApp() -> Element {
        rsx! {
            PaginationRoot {
                id: "pg-root",
                count: 100,
                per_page: 10,
                sibling_count: 1,
                default_page: 1,
                render_children: move |snap: PaginationSnapshot| rsx! {
                    div {
                        id: "pg-status",
                        "data-page": "{snap.current_page}",
                        "data-total": "{snap.total_pages}",
                        "data-range": "{snap.range.start}-{snap.range.end}",
                    }
                    div { style: "display: flex; gap: 4px;",
                        PaginationPrevButton { id: "pg-prev", "Prev" }
                        for item in snap.pages.iter() {
                            match item {
                                PageItem::Page { value, key } => rsx! {
                                    PaginationPage {
                                        key: "{key}",
                                        id: "pg-page-{value}",
                                        page: PageData::new(*value),
                                    }
                                },
                                PageItem::Ellipsis { key } => rsx! {
                                    span { key: "{key}", id: "pg-{key}", "..." }
                                },
                            }
                        }
                        PaginationNextButton { id: "pg-next", "Next" }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_pagination_initial_render_selection_and_boundaries() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeBasicPaginationApp);

        assert_eq!(harness.attr("#pg-root", "role").as_deref(), Some("navigation"));
        assert_eq!(
            harness.attr("#pg-root", "aria-label").as_deref(),
            Some("pagination")
        );
        assert_eq!(
            harness.attr("#pg-root", "data-orientation").as_deref(),
            Some("horizontal")
        );
        assert_eq!(harness.attr("#pg-status", "data-page").as_deref(), Some("1"));
        assert_eq!(
            harness.attr("#pg-status", "data-range").as_deref(),
            Some("1-10")
        );

        // Page 1 is selected; Page 2 is not.
        assert_eq!(
            harness.attr("#pg-page-1", "aria-current").as_deref(),
            Some("page")
        );
        assert_eq!(
            harness.attr("#pg-page-1", "data-selected").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#pg-page-1", "aria-label").as_deref(), Some("Page 1"));
        assert_eq!(harness.attr("#pg-page-2", "aria-current"), None);
        assert_eq!(harness.attr("#pg-page-2", "data-selected"), None);

        // PrevButton is disabled at boundary 1; NextButton is enabled.
        assert!(harness.attr("#pg-prev", "disabled").is_some());
        assert_eq!(
            harness.attr("#pg-prev", "aria-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#pg-prev", "data-disabled").as_deref(),
            Some("")
        );
        assert!(harness.attr("#pg-next", "disabled").is_none());
        assert_eq!(harness.attr("#pg-next", "aria-disabled"), None);
        assert_eq!(harness.attr("#pg-next", "data-disabled"), None);
        assert!(harness.query("#pg-ellipsis-0").is_some());
        assert!(harness.query("#pg-ellipsis-1").is_none());

        // Forced click on disabled PrevButton keeps page at 1.
        click_and_pump(&mut harness, "#pg-prev");
        assert_eq!(harness.attr("#pg-status", "data-page").as_deref(), Some("1"));

        // Click NextButton -> page 2, PrevButton becomes enabled.
        click_and_pump(&mut harness, "#pg-next");
        assert_eq!(harness.attr("#pg-status", "data-page").as_deref(), Some("2"));
        assert_eq!(
            harness.attr("#pg-status", "data-range").as_deref(),
            Some("11-20")
        );
        assert!(harness.attr("#pg-prev", "disabled").is_none());
        assert_eq!(harness.attr("#pg-prev", "data-disabled"), None);

        // Click Page 10 -> boundary end, NextButton disables and both ellipses collapse to leading ellipsis.
        click_and_pump(&mut harness, "#pg-page-10");
        assert_eq!(harness.attr("#pg-status", "data-page").as_deref(), Some("10"));
        assert_eq!(
            harness.attr("#pg-status", "data-range").as_deref(),
            Some("91-100")
        );
        assert_eq!(
            harness.attr("#pg-page-10", "aria-current").as_deref(),
            Some("page")
        );
        assert!(harness.attr("#pg-next", "disabled").is_some());
        assert_eq!(
            harness.attr("#pg-next", "aria-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#pg-next", "data-disabled").as_deref(),
            Some("")
        );

        // Forced click on disabled NextButton keeps page at 10.
        click_and_pump(&mut harness, "#pg-next");
        assert_eq!(harness.attr("#pg-status", "data-page").as_deref(), Some("10"));

        // Jump to Page 7 -> shows middle window and both ellipses.
        click_and_pump(&mut harness, "#pg-page-7");
        assert_eq!(harness.attr("#pg-status", "data-page").as_deref(), Some("7"));
        assert!(harness.query("#pg-ellipsis-0").is_some());
        assert!(harness.query("#pg-ellipsis-1").is_some());
    }

    #[component]
    fn NativeControlledPaginationApp() -> Element {
        let mut page = use_signal(|| 1usize);
        let mut change_calls = use_signal(|| 0usize);

        rsx! {
            PaginationRoot {
                id: "pg-ctrl-root",
                count: 30,
                per_page: 10,
                page: page(),
                aria_label: "Results pagination".to_string(),
                on_page_change: move |next| {
                    change_calls += 1;
                    page.set(next);
                },
                render_children: move |snap: PaginationSnapshot| rsx! {
                    div {
                        id: "pg-ctrl-probe",
                        "data-page": "{snap.current_page}",
                        "data-calls": "{change_calls}",
                    }
                    PaginationPrevButton { id: "pg-ctrl-prev", "Prev" }
                    for item in snap.pages.iter() {
                        if let Some(p) = item.as_page_data() {
                            PaginationPage {
                                key: "{item.key()}",
                                id: "pg-ctrl-page-{p.value}",
                                page: p,
                            }
                        }
                    }
                    PaginationNextButton { id: "pg-ctrl-next", "Next" }
                }
            }
        }
    }

    #[test]
    fn test_native_pagination_controlled_mode_and_custom_aria_label() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeControlledPaginationApp);

        assert_eq!(
            harness.attr("#pg-ctrl-root", "aria-label").as_deref(),
            Some("Results pagination")
        );
        assert_eq!(
            harness.attr("#pg-ctrl-probe", "data-calls").as_deref(),
            Some("0")
        );

        // Disabled prev click must not fire on_page_change.
        click_and_pump(&mut harness, "#pg-ctrl-prev");
        assert_eq!(
            harness.attr("#pg-ctrl-probe", "data-calls").as_deref(),
            Some("0")
        );

        // Next -> 2 -> Prev -> 1 works across repeated controlled updates.
        click_and_pump(&mut harness, "#pg-ctrl-next");
        assert_eq!(
            harness.attr("#pg-ctrl-probe", "data-page").as_deref(),
            Some("2")
        );
        assert_eq!(
            harness.attr("#pg-ctrl-probe", "data-calls").as_deref(),
            Some("1")
        );

        click_and_pump(&mut harness, "#pg-ctrl-prev");
        assert_eq!(
            harness.attr("#pg-ctrl-probe", "data-page").as_deref(),
            Some("1")
        );
        assert_eq!(
            harness.attr("#pg-ctrl-probe", "data-calls").as_deref(),
            Some("2")
        );
    }

    #[component]
    fn NativeRtlLoopPaginationApp() -> Element {
        rsx! {
            PaginationRoot {
                id: "pg-rtl-root",
                count: 30,
                per_page: 10,
                dir: Direction::Rtl,
                loop_focus: true,
                render_children: move |snap: PaginationSnapshot| rsx! {
                    div { id: "pg-rtl-probe", "data-page": "{snap.current_page}" }
                    PaginationPrevButton { id: "pg-rtl-prev", "Prev" }
                    for item in snap.pages.iter() {
                        if let Some(p) = item.as_page_data() {
                            PaginationPage {
                                key: "{item.key()}",
                                id: "pg-rtl-page-{p.value}",
                                page: p,
                            }
                        }
                    }
                    PaginationNextButton { id: "pg-rtl-next", "Next" }
                }
            }
        }
    }

    #[test]
    fn test_native_pagination_keyboard_rtl_loop_and_enter_activation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeRtlLoopPaginationApp);

        assert_eq!(harness.attr("#pg-rtl-root", "dir").as_deref(), Some("rtl"));

        // Focus Page 1, ArrowLeft in RTL advances to Page 2.
        focus_node(&mut harness, "#pg-rtl-page-1");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "pg-rtl-page-2");

        // Enter activates Page 2.
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(
            harness.attr("#pg-rtl-probe", "data-page").as_deref(),
            Some("2")
        );

        // End moves to NextButton; ArrowLeft in RTL wraps around to PrevButton (now enabled on page 2).
        send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
        assert_native_focus_id(&harness, "pg-rtl-next");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "pg-rtl-prev");

        // Home moves to first focusable item (PrevButton on page 2).
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "pg-rtl-page-1");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
        assert_native_focus_id(&harness, "pg-rtl-prev");
    }

    #[component]
    fn NativeVerticalDisabledPageApp() -> Element {
        rsx! {
            PaginationRoot {
                id: "pg-vert-root",
                count: 30,
                per_page: 10,
                orientation: Orientation::Vertical,
                loop_focus: false,
                render_children: move |snap: PaginationSnapshot| rsx! {
                    div { id: "pg-vert-probe", "data-page": "{snap.current_page}" }
                    PaginationPrevButton { id: "pg-vert-prev", "Prev" }
                    for item in snap.pages.iter() {
                        if let Some(p) = item.as_page_data() {
                            PaginationPage {
                                key: "{item.key()}",
                                id: "pg-vert-page-{p.value}",
                                page: p,
                                disabled: p.value == 2,
                            }
                        }
                    }
                    PaginationNextButton { id: "pg-vert-next", "Next" }
                }
            }
        }
    }

    #[test]
    fn test_native_pagination_vertical_skips_disabled_page_and_ignores_horizontal_arrows() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeVerticalDisabledPageApp);

        assert_eq!(
            harness.attr("#pg-vert-root", "data-orientation").as_deref(),
            Some("vertical")
        );
        assert!(harness.attr("#pg-vert-page-2", "disabled").is_some());
        assert_eq!(
            harness.attr("#pg-vert-page-2", "aria-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#pg-vert-page-2", "data-disabled").as_deref(),
            Some("")
        );

        // Disabled page click must be guarded.
        click_and_pump(&mut harness, "#pg-vert-page-2");
        assert_eq!(
            harness.attr("#pg-vert-probe", "data-page").as_deref(),
            Some("1")
        );

        // ArrowDown from Page 1 skips disabled Page 2 and focuses Page 3.
        focus_node(&mut harness, "#pg-vert-page-1");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "pg-vert-page-3");

        // Orthogonal ArrowRight is ignored in Vertical orientation.
        send_key_and_pump(
            &mut harness,
            Key::Named(NamedKey::ArrowRight),
            Code::ArrowRight,
        );
        assert_native_focus_id(&harness, "pg-vert-page-3");
    }
}

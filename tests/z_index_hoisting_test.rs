//! Floating Wrapper Z-Index Hoisting Failure-Mode Test Suite (`FM-ZIX-1`..`FM-ZIX-6`).

use monoxus::foundation::overlay::{
    FloatingLayer, FloatingReadiness, PlacementSide, Rect, Size, normalize_hoisted_z_index,
};

/// FM-ZIX-2 (Pure / Core): `auto` normalizes to `None`, while `0` and `-1` are preserved.
#[test]
fn fm_zix_2_normalize_hoisted_z_index_and_wrapper_style() {
    assert_eq!(normalize_hoisted_z_index(None), None);
    assert_eq!(normalize_hoisted_z_index(Some("")), None);
    assert_eq!(normalize_hoisted_z_index(Some("auto")), None);
    assert_eq!(normalize_hoisted_z_index(Some("AUTO")), None);
    assert_eq!(normalize_hoisted_z_index(Some("0")), Some("0".to_string()));
    assert_eq!(
        normalize_hoisted_z_index(Some("-1")),
        Some("-1".to_string())
    );
    assert_eq!(
        normalize_hoisted_z_index(Some("50")),
        Some("50".to_string())
    );

    let placement = FloatingLayer::new(PlacementSide::Bottom)
        .position(Rect::new(10.0, 20.0, 100.0, 32.0), Size::new(160.0, 80.0));
    assert!(
        !placement
            .wrapper_style(FloatingReadiness::Ready, None)
            .contains("z-index:")
    );
    assert!(
        placement
            .wrapper_style(FloatingReadiness::Ready, Some("0"))
            .contains("z-index: 0;")
    );
    assert!(
        placement
            .wrapper_style(FloatingReadiness::Ready, Some("-1"))
            .contains("z-index: -1;")
    );
    assert!(!FloatingReadiness::wrapper_measuring_style(None).contains("z-index:"));
    assert!(FloatingReadiness::wrapper_measuring_style(Some("50")).contains("z-index: 50;"));
}

#[cfg(feature = "native")]
mod native_z_index_tests {
    use dioxus::prelude::*;
    use monoxus::{
        combobox::{ComboboxContent, ComboboxInput, ComboboxItem, ComboboxRoot},
        date_picker::{DatePickerContent, DatePickerRoot, DatePickerTrigger},
        date_range_picker::{DateRangePickerContent, DateRangePickerRoot, DateRangePickerTrigger},
        dropdown_menu::{
            DropdownMenuContent, DropdownMenuContentRenderProps, DropdownMenuItem,
            DropdownMenuRoot, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger,
            DropdownMenuTrigger,
        },
        hover_card::{HoverCardContent, HoverCardRoot, HoverCardTrigger},
        popover::{PopoverContent, PopoverRoot, PopoverTrigger},
        select::{SelectContent, SelectItem, SelectRoot, SelectTrigger},
        tooltip::{TooltipContent, TooltipRoot, TooltipTrigger},
    };

    use crate::common::native_fixture::*;

    fn pump_z_index(harness: &mut blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>) {
        drain_polls_and_resolve(harness);
        advance_frame_and_pump(harness, 0.02);
        drain_polls_and_resolve(harness);
        advance_frame_and_pump(harness, 0.02);
        drain_polls_and_resolve(harness);
    }

    fn wrapper_style_of(
        harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        wrapper_selector: &str,
    ) -> String {
        harness
            .attr(wrapper_selector, "style")
            .unwrap_or_default()
            .to_string()
    }

    fn node_style(
        harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        node_id: blitz_dom::NodeId,
    ) -> String {
        harness
            .base()
            .get_node(node_id)
            .and_then(|n| n.element_data())
            .and_then(|e| e.attr(blitz_dom::local_name!("style")))
            .unwrap_or("")
            .to_string()
    }

    /// FM-ZIX-1: `position: static` + stylesheet `.z-50` and inline `z-index: 50` hoist onto
    /// `[data-monoxus-floating-content-wrapper]` across all 8 floating overlays.
    #[test]
    fn fm_zix_1_native_position_static_hoisting_across_all_8_overlays() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn app() -> Element {
            rsx! {
                style {
                    ".z-50 {{ z-index: 50; }} .z-40 {{ z-index: 40; }} .z-60 {{ z-index: 60; }} .box {{ width: 120px; height: 32px; display: block; }}"
                }
                div {
                    style: "padding: 40px; display: flex; flex-direction: column; gap: 16px;",

                    // 1. Tooltip (stylesheet .z-50, position: static)
                    TooltipRoot {
                        id: Some("zix1-tt".to_string()),
                        default_open: true,
                        TooltipTrigger { class: "box", "TT" }
                        TooltipContent {
                            id: Some("zix1-tt-content".to_string()),
                            class: "z-50 box",
                            "Tooltip body"
                        }
                    }

                    // 2. Popover (stylesheet .z-50, position: static)
                    PopoverRoot {
                        id: Some("zix1-pop".to_string()),
                        default_open: true,
                        PopoverTrigger { class: "box", "Pop" }
                        PopoverContent {
                            id: Some("zix1-pop-content".to_string()),
                            class: "z-50 box",
                            "Popover body"
                        }
                    }

                    // 3. DatePicker (delegates to PopoverRuntime)
                    DatePickerRoot {
                        id: Some("zix1-dp".to_string()),
                        default_open: true,
                        DatePickerTrigger { class: "box", "DP" }
                        DatePickerContent {
                            id: Some("zix1-dp-content".to_string()),
                            class: "z-50 box",
                            "DatePicker body"
                        }
                    }

                    // 4. DateRangePicker (delegates to PopoverRuntime)
                    DateRangePickerRoot {
                        id: Some("zix1-drp".to_string()),
                        default_open: true,
                        DateRangePickerTrigger { class: "box", "DRP" }
                        DateRangePickerContent {
                            id: Some("zix1-drp-content".to_string()),
                            class: "z-50 box",
                            "DateRangePicker body"
                        }
                    }

                    // 5. HoverCard (stylesheet .z-50, position: static)
                    HoverCardRoot {
                        id: Some("zix1-hc".to_string()),
                        default_open: true,
                        HoverCardTrigger { class: "box", "HC" }
                        HoverCardContent {
                            id: Some("zix1-hc-content".to_string()),
                            class: "z-50 box",
                            "HoverCard body"
                        }
                    }

                    // 6. Select (inline z-index: 50, position: static)
                    SelectRoot {
                        id: Some("zix1-sel".to_string()),
                        open: true,
                        SelectTrigger { class: Some("box".to_string()), "Sel" }
                        SelectContent {
                            style: Some("z-index: 50; width: 120px; height: 40px;".to_string()),
                            SelectItem { value: "a".to_string(), "Alpha" }
                        }
                    }

                    // 7. Combobox (stylesheet .z-50, position: static)
                    ComboboxRoot {
                        id: Some("zix1-cbx".to_string()),
                        default_open: true,
                        ComboboxInput { class: Some("box".to_string()) }
                        ComboboxContent {
                            id: Some("zix1-cbx-content".to_string()),
                            class: Some("z-50 box".to_string()),
                            ComboboxItem { value: "a".to_string(), "Alpha" }
                        }
                    }

                    // 8. Menu + MenuSub (stylesheet .z-40 root + .z-60 sub)
                    DropdownMenuRoot {
                        open: Some(true),
                        modal: false,
                        DropdownMenuTrigger { class: Some("box".to_string()), "Menu" }
                        DropdownMenuContent {
                            id: Some("zix1-menu-content".to_string()),
                            class: Some("z-40 box".to_string()),
                            DropdownMenuItem { "Item 1" }
                            DropdownMenuSub {
                                open: Some(true),
                                DropdownMenuSubTrigger { class: Some("box".to_string()), "Sub" }
                                DropdownMenuSubContent {
                                    id: Some("zix1-sub-content".to_string()),
                                    class: Some("z-60 box".to_string()),
                                    DropdownMenuItem { "Sub Item 1" }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(app);
        pump_z_index(&mut harness);

        let doc = oxidase::dom::document();
        assert_eq!(
            doc.computed_z_index_by_id("zix1-tt-content").as_deref(),
            Some("50")
        );
        assert_eq!(
            doc.computed_z_index_by_id("zix1-pop-content").as_deref(),
            Some("50")
        );
        assert_eq!(
            doc.computed_z_index_by_id("zix1-dp-content").as_deref(),
            Some("50")
        );
        assert_eq!(
            doc.computed_z_index_by_id("zix1-drp-content").as_deref(),
            Some("50")
        );
        assert_eq!(
            doc.computed_z_index_by_id("zix1-hc-content").as_deref(),
            Some("50")
        );
        assert_eq!(
            doc.computed_z_index_by_id("zix1-cbx-content").as_deref(),
            Some("50")
        );
        assert_eq!(
            doc.computed_z_index_by_id("zix1-menu-content").as_deref(),
            Some("40")
        );
        assert_eq!(
            doc.computed_z_index_by_id("zix1-sub-content").as_deref(),
            Some("60")
        );

        let wrappers = harness.query_all("[data-monoxus-floating-content-wrapper]");
        assert_eq!(
            wrappers.len(),
            9,
            "expected 9 floating wrappers (7 overlays + root menu + submenu)"
        );

        let styles: Vec<String> = wrappers
            .iter()
            .map(|id| node_style(&harness, *id))
            .collect();

        assert_eq!(
            styles.iter().filter(|s| s.contains("z-index: 50;")).count(),
            7,
            "expected 7 wrappers with hoisted z-index: 50;, got styles: {styles:#?}"
        );
        assert!(
            styles.iter().any(|s| s.contains("z-index: 40;")),
            "expected root menu wrapper with z-index: 40;, got styles: {styles:#?}"
        );
        assert!(
            styles.iter().any(|s| s.contains("z-index: 60;")),
            "expected submenu wrapper with z-index: 60;, got styles: {styles:#?}"
        );
    }

    /// FM-ZIX-2 (Native): `z-index: auto` omits `z-index:`, while `0` and `-1` are hoisted.
    #[test]
    fn fm_zix_2_native_auto_vs_zero_vs_negative() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn app() -> Element {
            rsx! {
                PopoverRoot {
                    id: Some("zix2-auto".to_string()),
                    default_open: true,
                    PopoverTrigger { style: "width: 80px; height: 24px;", "Auto" }
                    PopoverContent {
                        id: Some("zix2-auto-content".to_string()),
                        style: "width: 100px; height: 40px; z-index: auto;",
                        "Auto"
                    }
                }
                PopoverRoot {
                    id: Some("zix2-zero".to_string()),
                    default_open: true,
                    PopoverTrigger { style: "width: 80px; height: 24px;", "Zero" }
                    PopoverContent {
                        id: Some("zix2-zero-content".to_string()),
                        style: "width: 100px; height: 40px; z-index: 0;",
                        "Zero"
                    }
                }
                PopoverRoot {
                    id: Some("zix2-neg".to_string()),
                    default_open: true,
                    PopoverTrigger { style: "width: 80px; height: 24px;", "Neg" }
                    PopoverContent {
                        id: Some("zix2-neg-content".to_string()),
                        style: "width: 100px; height: 40px; z-index: -1;",
                        "Neg"
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(app);
        pump_z_index(&mut harness);

        let doc = oxidase::dom::document();
        assert_eq!(
            doc.computed_z_index_by_id("zix2-auto-content").as_deref(),
            Some("auto")
        );
        assert_eq!(
            doc.computed_z_index_by_id("zix2-zero-content").as_deref(),
            Some("0")
        );
        assert_eq!(
            doc.computed_z_index_by_id("zix2-neg-content").as_deref(),
            Some("-1")
        );

        let auto_style = wrapper_style_of(&harness, "#zix2-auto-content-wrapper");
        let zero_style = wrapper_style_of(&harness, "#zix2-zero-content-wrapper");
        let neg_style = wrapper_style_of(&harness, "#zix2-neg-content-wrapper");

        assert!(
            !auto_style.contains("z-index:"),
            "auto must not emit z-index on wrapper: {auto_style}"
        );
        assert!(
            zero_style.contains("z-index: 0;"),
            "0 must emit z-index: 0; on wrapper: {zero_style}"
        );
        assert!(
            neg_style.contains("z-index: -1;"),
            "-1 must emit z-index: -1; on wrapper: {neg_style}"
        );
    }

    /// FM-ZIX-3 (Native): Select & Combobox omit z-index when unset and hoist `z-index: 60;` when set.
    #[test]
    fn fm_zix_3_native_select_and_combobox_no_hardcoded_50_and_hoist_60() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn app() -> Element {
            let mut apply_z60 = use_signal(|| false);
            let mut open_both = use_signal(|| true);
            let extra_style = if apply_z60() {
                "width: 120px; height: 40px; z-index: 60;"
            } else {
                "width: 120px; height: 40px;"
            };
            rsx! {
                button {
                    id: "zix3-toggle-z60",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| {
                        apply_z60.set(true);
                        open_both.set(false);
                    },
                    "Set Z60 & Close"
                }
                button {
                    id: "zix3-reopen",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| open_both.set(true),
                    "Reopen"
                }
                SelectRoot {
                    id: Some("zix3-sel".to_string()),
                    open: open_both(),
                    on_open_change: move |v| open_both.set(v),
                    SelectTrigger { style: Some("width: 100px; height: 28px;".to_string()), "Sel" }
                    SelectContent {
                        style: Some(extra_style.to_string()),
                        SelectItem { value: "a".to_string(), "Alpha" }
                    }
                }
                ComboboxRoot {
                    id: Some("zix3-cbx".to_string()),
                    open: Some(open_both),
                    ComboboxInput { style: Some("width: 100px; height: 28px;".to_string()) }
                    ComboboxContent {
                        id: Some("zix3-cbx-content".to_string()),
                        style: Some(extra_style.to_string()),
                        ComboboxItem { value: "a".to_string(), "Alpha" }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(app);
        pump_z_index(&mut harness);

        let initial_wrappers = harness.query_all("[data-monoxus-floating-content-wrapper]");
        assert_eq!(initial_wrappers.len(), 2);
        for id in &initial_wrappers {
            let style = node_style(&harness, *id);
            assert!(
                !style.contains("z-index: 50;") && !style.contains("z-index:"),
                "unset Select/Combobox must not emit hardcoded z-index: {style}"
            );
        }

        click_and_pump(&mut harness, "#zix3-toggle-z60");
        pump_z_index(&mut harness);
        click_and_pump(&mut harness, "#zix3-reopen");
        pump_z_index(&mut harness);

        let reopened_wrappers = harness.query_all("[data-monoxus-floating-content-wrapper]");
        assert_eq!(reopened_wrappers.len(), 2);
        for id in &reopened_wrappers {
            let style = node_style(&harness, *id);
            assert!(
                style.contains("z-index: 60;"),
                "Select/Combobox with z-index: 60 must hoist z-index: 60; onto wrapper: {style}"
            );
        }
    }

    /// FM-ZIX-4 (Native): Cold-mount `default_open` + `force_mount` stylesheet resolution syncs
    /// hoisted `z-index: 50;` alongside `visibility: visible;` (`FloatingReadiness::Ready`).
    #[test]
    fn fm_zix_4_native_cold_mount_default_open_and_force_mount_post_resolve_sync() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn app() -> Element {
            rsx! {
                style { ".z-50 {{ z-index: 50; }}" }
                TooltipRoot {
                    id: Some("zix4-tt".to_string()),
                    default_open: true,
                    TooltipTrigger { style: "width: 100px; height: 28px;", "TT" }
                    TooltipContent {
                        id: Some("zix4-tt-content".to_string()),
                        class: "z-50",
                        style: "width: 120px; height: 32px;",
                        force_mount: true,
                        "Cold Tooltip"
                    }
                }
                PopoverRoot {
                    id: Some("zix4-pop".to_string()),
                    default_open: true,
                    PopoverTrigger { style: "width: 100px; height: 28px;", "Pop" }
                    PopoverContent {
                        id: Some("zix4-pop-content".to_string()),
                        class: "z-50",
                        style: "width: 120px; height: 40px;",
                        force_mount: true,
                        "Cold Popover"
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(app);
        pump_z_index(&mut harness);

        let wrappers = harness.query_all("[data-monoxus-floating-content-wrapper]");
        assert_eq!(wrappers.len(), 2);
        for id in wrappers {
            let style = node_style(&harness, id);
            assert!(
                style.contains("visibility: visible;") && style.contains("z-index: 50;"),
                "cold-mount wrapper must reach Ready with z-index: 50; hoisted: {style}"
            );
        }
    }

    /// FM-ZIX-5 (Native): Close -> reopen resets stale `content_z_index` across Popover and Select.
    #[test]
    fn fm_zix_5_native_close_reopen_resets_stale_z_index() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn app() -> Element {
            let mut open = use_signal(|| true);
            let mut with_z70 = use_signal(|| true);
            let style_val = if with_z70() {
                "width: 120px; height: 40px; z-index: 70;"
            } else {
                "width: 120px; height: 40px;"
            };
            rsx! {
                button {
                    id: "zix5-close",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| {
                        open.set(false);
                        with_z70.set(false);
                    },
                    "Close"
                }
                button {
                    id: "zix5-open",
                    style: "width: 80px; height: 24px;",
                    onclick: move |_| open.set(true),
                    "Open"
                }
                PopoverRoot {
                    id: Some("zix5-pop".to_string()),
                    open: Some(open),
                    on_open_change: move |v| open.set(v),
                    PopoverTrigger { style: "width: 80px; height: 24px;", "Trigger" }
                    PopoverContent {
                        id: Some("zix5-pop-content".to_string()),
                        style: "{style_val}",
                        "Body"
                    }
                }
                SelectRoot {
                    id: Some("zix5-sel".to_string()),
                    open: open(),
                    on_open_change: move |v| open.set(v),
                    SelectTrigger { style: Some("width: 80px; height: 24px;".to_string()), "Sel" }
                    SelectContent {
                        style: Some(style_val.to_string()),
                        SelectItem { value: "a".to_string(), "Alpha" }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(app);
        pump_z_index(&mut harness);

        let open1_wrappers = harness.query_all("[data-monoxus-floating-content-wrapper]");
        assert_eq!(open1_wrappers.len(), 2);
        for id in &open1_wrappers {
            let style = node_style(&harness, *id);
            assert!(
                style.contains("z-index: 70;"),
                "first open must hoist z-index: 70;: {style}"
            );
        }

        click_and_pump(&mut harness, "#zix5-close");
        advance_frame_and_pump(&mut harness, 0.35);
        pump_z_index(&mut harness);
        assert!(
            harness
                .query_all("[data-monoxus-floating-content-wrapper]")
                .is_empty(),
            "wrappers should unmount on close"
        );

        click_and_pump(&mut harness, "#zix5-open");
        pump_z_index(&mut harness);

        let open2_wrappers = harness.query_all("[data-monoxus-floating-content-wrapper]");
        assert_eq!(open2_wrappers.len(), 2);
        for id in &open2_wrappers {
            let style = node_style(&harness, *id);
            assert!(
                !style.contains("z-index: 70;") && !style.contains("z-index:"),
                "reopened wrapper must not retain stale z-index: 70;: {style}"
            );
        }
    }

    /// FM-ZIX-6 (Native): MenuContent (`z-40`) + MenuSubContent (`z-60`) emit
    /// `data-monoxus-floating-content-wrapper=""` and hoist independent z-index values
    /// in BOTH default and `as_child` render branches.
    #[test]
    fn fm_zix_6_native_menu_and_submenu_independent_z_index_default_and_as_child() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn app() -> Element {
            rsx! {
                style { ".z-40 {{ z-index: 40; }} .z-60 {{ z-index: 60; }}" }
                DropdownMenuRoot {
                    open: Some(true),
                    modal: false,
                    DropdownMenuTrigger { style: Some("width: 100px; height: 28px;".to_string()), "Menu" }
                    DropdownMenuContent {
                        id: Some("zix6-root-default".to_string()),
                        class: Some("z-40".to_string()),
                        style: Some("width: 120px; height: 60px;".to_string()),
                        DropdownMenuItem { "Item 1" }
                        DropdownMenuSub {
                            open: Some(true),
                            DropdownMenuSubTrigger { style: Some("width: 100px; height: 24px;".to_string()), "Sub" }
                            DropdownMenuSubContent {
                                id: Some("zix6-sub-default".to_string()),
                                class: Some("z-60".to_string()),
                                style: Some("width: 120px; height: 60px;".to_string()),
                                DropdownMenuItem { "Sub Item 1" }
                            }
                        }
                    }
                }
                DropdownMenuRoot {
                    open: Some(true),
                    modal: false,
                    DropdownMenuTrigger { style: Some("width: 100px; height: 28px;".to_string()), "Menu2" }
                    DropdownMenuContent {
                        id: Some("zix6-root-as-child".to_string()),
                        class: Some("z-40".to_string()),
                        style: Some("width: 120px; height: 60px;".to_string()),
                        as_child: true,
                        render_child: Callback::new(|rp: DropdownMenuContentRenderProps| {
                            rsx! {
                                div {
                                    id: "{rp.attrs.id}",
                                    class: rp.class.as_deref().unwrap_or_default(),
                                    style: "{rp.style}",
                                    role: "{rp.attrs.role}",
                                    {rp.children}
                                }
                            }
                        }),
                        DropdownMenuItem { "Item 1" }
                        DropdownMenuSub {
                            open: Some(true),
                            DropdownMenuSubTrigger { style: Some("width: 100px; height: 24px;".to_string()), "Sub2" }
                            DropdownMenuSubContent {
                                id: Some("zix6-sub-as-child".to_string()),
                                class: Some("z-60".to_string()),
                                style: Some("width: 120px; height: 60px;".to_string()),
                                as_child: true,
                                render_child: Callback::new(|rp: DropdownMenuContentRenderProps| {
                                    rsx! {
                                        div {
                                            id: "{rp.attrs.id}",
                                            class: rp.class.as_deref().unwrap_or_default(),
                                            style: "{rp.style}",
                                            role: "{rp.attrs.role}",
                                            {rp.children}
                                        }
                                    }
                                }),
                                DropdownMenuItem { "Sub Item 1" }
                            }
                        }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(app);
        pump_z_index(&mut harness);

        let wrappers = harness.query_all("[data-monoxus-floating-content-wrapper]");
        assert_eq!(
            wrappers.len(),
            4,
            "expected 4 floating wrappers (default + as_child for root & sub)"
        );

        let styles: Vec<String> = wrappers
            .iter()
            .map(|id| node_style(&harness, *id))
            .collect();

        assert_eq!(
            styles.iter().filter(|s| s.contains("z-index: 40;")).count(),
            2,
            "expected 2 root menu wrappers with z-index: 40;, got {styles:#?}"
        );
        assert_eq!(
            styles.iter().filter(|s| s.contains("z-index: 60;")).count(),
            2,
            "expected 2 submenu wrappers with z-index: 60;, got {styles:#?}"
        );
    }
}

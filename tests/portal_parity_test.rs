//! Portal Unification, Static Anchor Wrapper & OverlayBranchContext Parity Tests
//! (`FM-PRT-1`, `FM-PRT-3`, `FM-PRT-4`, `FM-PRT-5`, `FM-PRT-6`).

use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Waker},
};

use dioxus::prelude::*;
use monoxus::{
    combobox::{ComboboxContent, ComboboxInput, ComboboxItem, ComboboxPortal, ComboboxRoot},
    date_picker::{DatePickerContent, DatePickerPortal, DatePickerRoot, DatePickerTrigger},
    date_range_picker::{
        DateRangePickerContent, DateRangePickerPortal, DateRangePickerRoot, DateRangePickerTrigger,
    },
    dialog::{
        DialogContent, DialogContext, DialogMode, DialogOutsideInteractionPolicy, DialogOverlay,
        DialogPortal, DialogRoot, DialogRuntime, DialogTrigger,
    },
    dropdown_menu::{
        DropdownMenuContent, DropdownMenuItem, DropdownMenuPortal, DropdownMenuRoot,
        DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
    },
    foundation::{
        overlay::{
            PortalHost, PortalOp, active_overlay_layers_snapshot, start_portal_op_recording,
            stop_portal_op_recording, take_portal_ops, unregister_active_overlay_layer,
        },
        shared::ScopeHandle,
    },
    hover_card::{HoverCardContent, HoverCardPortal, HoverCardRoot, HoverCardTrigger},
    menu::MenuRootContext,
    popover::{
        PopoverContent, PopoverContext, PopoverPortal, PopoverRoot, PopoverRuntime, PopoverTrigger,
    },
    select::{SelectContent, SelectItem, SelectPortal, SelectRoot, SelectTrigger},
    tooltip::{TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger},
};

fn clear_active_overlay_stack() {
    for layer_id in active_overlay_layers_snapshot() {
        unregister_active_overlay_layer(&layer_id);
    }
}

fn poll_once<F: Future>(fut: F) {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut fut = std::pin::pin!(fut);
    let _ = Pin::as_mut(&mut fut).poll(&mut cx);
}

fn flush_vdom(dom: &mut VirtualDom) {
    for _ in 0..32 {
        poll_once(dom.wait_for_work());
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
}

struct PortalRecorderGuard;

impl PortalRecorderGuard {
    fn start() -> Self {
        start_portal_op_recording();
        Self
    }
}

impl Drop for PortalRecorderGuard {
    fn drop(&mut self) {
        stop_portal_op_recording();
    }
}

#[derive(Clone)]
struct Slot<T>(Rc<RefCell<Option<T>>>);

impl<T> PartialEq for Slot<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Slot<T> {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    fn set(&self, val: T) {
        *self.0.borrow_mut() = Some(val);
    }

    fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.0.borrow().clone()
    }
}

#[component]
fn CaptureDialogRuntime(slot: Slot<DialogRuntime>) -> Element {
    let ctx = use_context::<DialogContext>();
    slot.set(ctx.runtime.read().clone());
    rsx! {}
}

#[component]
fn CapturePopoverRuntime(slot: Slot<PopoverRuntime>) -> Element {
    let ctx = use_context::<PopoverContext>();
    slot.set(ctx.runtime.read().clone());
    rsx! {}
}

#[component]
fn CaptureMenuRoot(slot: Slot<MenuRootContext>) -> Element {
    let ctx = use_context::<MenuRootContext>();
    slot.set(ctx);
    rsx! {}
}

// =============================================================================
// FM-PRT-1: All 7 *Portal components unify PortalHost::Default -> #main (None)
//           and permanently render [data-portal-anchor]
// =============================================================================

#[component]
fn AllSevenPortalsHarness(open: bool, host: Option<PortalHost>) -> Element {
    rsx! {
        div { id: "parent-p",
            TooltipRoot {
                id: Some("prt1-tt".to_string()),
                default_open: open,
                TooltipTrigger { "TT" }
                TooltipPortal {
                    host: host.clone(),
                    TooltipContent { id: Some("prt1-tt-content".to_string()), "Tooltip" }
                }
            }
            PopoverRoot {
                id: Some("prt1-pop".to_string()),
                default_open: open,
                PopoverTrigger { "Pop" }
                PopoverPortal {
                    host: host.clone(),
                    PopoverContent { id: Some("prt1-pop-content".to_string()), "Popover" }
                }
            }
            HoverCardRoot {
                id: Some("prt1-hc".to_string()),
                default_open: open,
                HoverCardTrigger { "HC" }
                HoverCardPortal {
                    host: host.clone(),
                    HoverCardContent { id: Some("prt1-hc-content".to_string()), "HoverCard" }
                }
            }
            DialogRoot {
                id: Some("prt1-dlg".to_string()),
                default_open: open,
                DialogTrigger { "Dlg" }
                DialogPortal {
                    host: host.clone(),
                    DialogOverlay { id: Some("prt1-dlg-overlay".to_string()) }
                    DialogContent { id: Some("prt1-dlg-content".to_string()), "Dialog" }
                }
            }
            SelectRoot {
                id: Some("prt1-sel".to_string()),
                open: open,
                SelectTrigger { "Sel" }
                SelectPortal {
                    host: host.clone(),
                    SelectContent {
                        SelectItem { value: "a".to_string(), "Alpha" }
                    }
                }
            }
            DropdownMenuRoot {
                open: Some(open),
                modal: false,
                DropdownMenuTrigger { "Menu" }
                DropdownMenuPortal {
                    host: host.clone(),
                    DropdownMenuContent {
                        id: Some("prt1-menu-content".to_string()),
                        DropdownMenuItem { "One" }
                    }
                }
            }
            ComboboxRoot {
                id: Some("prt1-cbx".to_string()),
                default_open: open,
                ComboboxInput {}
                ComboboxPortal {
                    host: host.clone(),
                    ComboboxContent {
                        id: Some("prt1-cbx-content".to_string()),
                        ComboboxItem { value: "a".to_string(), "Alpha" }
                    }
                }
            }
        }
    }
}

#[test]
fn fm_prt_1_all_seven_portals_teleport_default_and_named_and_guard_drop() {
    clear_active_overlay_stack();
    let _rec = PortalRecorderGuard::start();

    // 1. Closed: zero teleport & zero remove on drop.
    let mut closed_dom = VirtualDom::new_with_props(
        AllSevenPortalsHarness,
        AllSevenPortalsHarnessProps {
            open: false,
            host: None,
        },
    );
    closed_dom.rebuild_in_place();
    flush_vdom(&mut closed_dom);
    assert!(
        take_portal_ops().is_empty(),
        "closed portals must not invoke teleport_element_to_host"
    );
    drop(closed_dom);
    assert!(
        take_portal_ops().is_empty(),
        "dropping closed (never-teleported) portals must not call remove_element_by_id"
    );

    // 2. Open with PortalHost::Default: all 7 teleport with host_id = None (#main-first),
    //    then clean up all 7 on drop.
    let mut open_dom = VirtualDom::new_with_props(
        AllSevenPortalsHarness,
        AllSevenPortalsHarnessProps {
            open: true,
            host: None,
        },
    );
    open_dom.rebuild_in_place();
    flush_vdom(&mut open_dom);

    let ops = take_portal_ops();
    let teleported_default: Vec<String> = ops
        .iter()
        .filter_map(|op| match op {
            PortalOp::Teleport {
                element_id,
                host_id: None,
            } => Some(element_id.clone()),
            _ => None,
        })
        .collect();
    assert!(
        teleported_default.len() >= 7,
        "expected >= 7 default-host teleports (all 7 portals), got {ops:?}"
    );
    let expected_ids = [
        ScopeHandle::root("tooltip").child("prt1-tt").qualify("portal"),
        ScopeHandle::root("popover").child("prt1-pop").qualify("portal"),
        ScopeHandle::root("hover-card").child("prt1-hc").qualify("portal"),
        ScopeHandle::root("dialog").child("prt1-dlg").qualify("portal"),
        format!(
            "{}-portal-root",
            ScopeHandle::root("select").child("prt1-sel").qualify("content")
        ),
        "prt1-cbx-content-portal-root".to_string(),
    ];
    for expected in &expected_ids {
        assert!(
            teleported_default.contains(expected),
            "missing default-host teleport for `{expected}` in {teleported_default:?}"
        );
    }
    assert!(
        teleported_default
            .iter()
            .any(|id| id.contains("dropdown-menu") && id.ends_with("-portal")),
        "missing MenuPortal default-host teleport in {teleported_default:?}"
    );

    drop(open_dom);
    let drop_ops = take_portal_ops();
    let removed_ids: Vec<String> = drop_ops
        .iter()
        .filter_map(|op| match op {
            PortalOp::Remove { element_id } => Some(element_id.clone()),
            _ => None,
        })
        .collect();
    assert!(
        removed_ids.len() >= 7,
        "dropping 7 open teleported portals must clean up all 7, got {drop_ops:?}"
    );

    // 3. Open with PortalHost::Named("custom-host"): all 7 teleport to Some("custom-host").
    let mut named_dom = VirtualDom::new_with_props(
        AllSevenPortalsHarness,
        AllSevenPortalsHarnessProps {
            open: true,
            host: Some(PortalHost::named("custom-host")),
        },
    );
    named_dom.rebuild_in_place();
    flush_vdom(&mut named_dom);
    let named_ops = take_portal_ops();
    let named_teleports = named_ops
        .iter()
        .filter(|op| {
            matches!(
                op,
                PortalOp::Teleport {
                    host_id: Some(h),
                    ..
                } if h == "custom-host"
            )
        })
        .count();
    assert!(
        named_teleports >= 7,
        "all 7 portals must teleport to Named(\"custom-host\"), got {named_ops:?}"
    );
    drop(named_dom);
    let _ = take_portal_ops();
    clear_active_overlay_stack();
}

// =============================================================================
// FM-PRT-5: SelectPortal & ComboboxPortal with Inline or disabled NEVER teleport
// =============================================================================

#[component]
fn SelectAndComboboxInlineDisabledHarness(
    host: Option<PortalHost>,
    disabled: bool,
) -> Element {
    rsx! {
        div { id: "inline-parent",
            SelectRoot {
                id: Some("prt5-sel".to_string()),
                open: true,
                SelectTrigger { "Sel" }
                SelectPortal {
                    host: host.clone(),
                    disabled: disabled,
                    SelectContent {
                        SelectItem { value: "a".to_string(), "Alpha" }
                    }
                }
            }
            ComboboxRoot {
                id: Some("prt5-cbx".to_string()),
                default_open: true,
                ComboboxInput {}
                ComboboxPortal {
                    host: host.clone(),
                    disabled: disabled,
                    ComboboxContent {
                        id: Some("prt5-cbx-content".to_string()),
                        ComboboxItem { value: "a".to_string(), "Alpha" }
                    }
                }
            }
        }
    }
}

#[test]
fn fm_prt_5_select_and_combobox_portal_inline_and_disabled_never_teleport_or_remove() {
    clear_active_overlay_stack();
    let _rec = PortalRecorderGuard::start();

    // Case A: PortalHost::Inline
    let mut inline_dom = VirtualDom::new_with_props(
        SelectAndComboboxInlineDisabledHarness,
        SelectAndComboboxInlineDisabledHarnessProps {
            host: Some(PortalHost::inline()),
            disabled: false,
        },
    );
    inline_dom.rebuild_in_place();
    flush_vdom(&mut inline_dom);
    assert!(
        take_portal_ops().is_empty(),
        "PortalHost::Inline must NEVER invoke teleport_element_to_host"
    );
    drop(inline_dom);
    assert!(
        take_portal_ops().is_empty(),
        "Dropping inline portals must NEVER invoke remove_element_by_id"
    );

    // Case B: disabled: true even when host is PortalHost::Default
    let mut disabled_dom = VirtualDom::new_with_props(
        SelectAndComboboxInlineDisabledHarness,
        SelectAndComboboxInlineDisabledHarnessProps {
            host: Some(PortalHost::default_host()),
            disabled: true,
        },
    );
    disabled_dom.rebuild_in_place();
    flush_vdom(&mut disabled_dom);
    assert!(
        take_portal_ops().is_empty(),
        "disabled: true portal must NEVER invoke teleport_element_to_host even with Default host"
    );
    drop(disabled_dom);
    assert!(
        take_portal_ops().is_empty(),
        "Dropping disabled portals must NEVER invoke remove_element_by_id"
    );

    clear_active_overlay_stack();
}

// =============================================================================
// FM-PRT-6 (VDOM): Exhaustive OverlayBranchContext registration & upward
//                  propagation into outer Dialog
// =============================================================================

#[component]
fn DialogWithAllNestedOverlaysHarness(
    outer_dialog_rt: Slot<DialogRuntime>,
    menu_root_ctx: Slot<MenuRootContext>,
    popover_rt: Slot<PopoverRuntime>,
    nested_open: bool,
) -> Element {
    rsx! {
        DialogRoot {
            id: Some("outer-dlg".to_string()),
            default_open: true,
            DialogPortal {
                DialogOverlay { id: Some("outer-dlg-overlay".to_string()) }
                DialogContent {
                    id: Some("outer-dlg-content".to_string()),

                    // 1. Popover -> Select (3-level forwarding!)
                    PopoverRoot {
                        id: Some("inner-pop".to_string()),
                        default_open: nested_open,
                        PopoverTrigger { "Open Pop" }
                        PopoverPortal {
                            PopoverContent {
                                id: Some("inner-pop-content".to_string()),
                                SelectRoot {
                                    id: Some("grandchild-sel".to_string()),
                                    open: nested_open,
                                    SelectTrigger { "Sel" }
                                    SelectPortal {
                                        SelectContent {
                                            SelectItem { value: "x".to_string(), "X" }
                                        }
                                    }
                                }
                            }
                        }
                        CapturePopoverRuntime { slot: popover_rt }
                    }

                    // 2. DatePicker
                    DatePickerRoot {
                        id: Some("inner-dp".to_string()),
                        default_open: nested_open,
                        DatePickerTrigger { "DP" }
                        DatePickerPortal {
                            DatePickerContent {
                                id: Some("inner-dp-content".to_string()),
                                "DP Body"
                            }
                        }
                    }

                    // 3. DateRangePicker
                    DateRangePickerRoot {
                        id: Some("inner-drp".to_string()),
                        default_open: nested_open,
                        DateRangePickerTrigger { "DRP" }
                        DateRangePickerPortal {
                            DateRangePickerContent {
                                id: Some("inner-drp-content".to_string()),
                                "DRP Body"
                            }
                        }
                    }

                    // 4. DropdownMenu + SubMenu
                    DropdownMenuRoot {
                        open: Some(nested_open),
                        modal: false,
                        DropdownMenuTrigger { "Menu" }
                        DropdownMenuPortal {
                            DropdownMenuContent {
                                id: Some("inner-menu-content".to_string()),
                                DropdownMenuItem { "Root Item" }
                                DropdownMenuSub {
                                    open: Some(nested_open),
                                    DropdownMenuSubTrigger { "More" }
                                    DropdownMenuPortal {
                                        DropdownMenuSubContent {
                                            id: Some("inner-sub-content".to_string()),
                                            DropdownMenuItem { "Sub Item" }
                                        }
                                    }
                                }
                            }
                        }
                        CaptureMenuRoot { slot: menu_root_ctx }
                    }

                    // 5. Combobox
                    ComboboxRoot {
                        id: Some("inner-cbx".to_string()),
                        default_open: nested_open,
                        ComboboxInput {}
                        ComboboxPortal {
                            ComboboxContent {
                                id: Some("inner-cbx-content".to_string()),
                                ComboboxItem { value: "k".to_string(), "K" }
                            }
                        }
                    }

                    // 6. Tooltip
                    TooltipRoot {
                        id: Some("inner-tt".to_string()),
                        default_open: nested_open,
                        TooltipTrigger { "TT" }
                        TooltipPortal {
                            TooltipContent {
                                id: Some("inner-tt-content".to_string()),
                                "TT Body"
                            }
                        }
                    }

                    // 7. HoverCard
                    HoverCardRoot {
                        id: Some("inner-hc".to_string()),
                        default_open: nested_open,
                        HoverCardTrigger { "HC" }
                        HoverCardPortal {
                            HoverCardContent {
                                id: Some("inner-hc-content".to_string()),
                                "HC Body"
                            }
                        }
                    }

                    // 8. Nested Dialog
                    DialogRoot {
                        id: Some("inner-dlg".to_string()),
                        default_open: nested_open,
                        DialogPortal {
                            DialogOverlay { id: Some("inner-dlg-overlay".to_string()) }
                            DialogContent {
                                id: Some("inner-dlg-content".to_string()),
                                "Nested Dialog"
                            }
                        }
                    }
                }
            }
            CaptureDialogRuntime { slot: outer_dialog_rt }
        }
    }
}

#[test]
fn fm_prt_6_all_nested_overlays_register_full_branch_triples_and_propagate_upward() {
    clear_active_overlay_stack();
    let outer_dlg_slot = Slot::new();
    let menu_slot = Slot::new();
    let popover_slot = Slot::new();

    let mut dom = VirtualDom::new_with_props(
        DialogWithAllNestedOverlaysHarness,
        DialogWithAllNestedOverlaysHarnessProps {
            outer_dialog_rt: outer_dlg_slot.clone(),
            menu_root_ctx: menu_slot.clone(),
            popover_rt: popover_slot.clone(),
            nested_open: true,
        },
    );
    dom.rebuild_in_place();
    flush_vdom(&mut dom);

    let outer_dlg = outer_dlg_slot.get().expect("outer dialog runtime");
    let menu_root = menu_slot.get().expect("menu root context");
    let pop_rt = popover_slot.get().expect("popover runtime");

    let sel_content = ScopeHandle::root("select")
        .child("grandchild-sel")
        .qualify("content");
    let expected_in_outer = vec![
        // Popover [content, wrapper, portal]
        "inner-pop-content".to_string(),
        "inner-pop-content-wrapper".to_string(),
        ScopeHandle::root("popover").child("inner-pop").qualify("portal"),
        // Grandchild Select forwarded upward through Popover!
        sel_content.clone(),
        format!("{sel_content}-wrapper"),
        format!("{sel_content}-portal-root"),
        // DatePicker [content, wrapper, portal]
        "inner-dp-content".to_string(),
        "inner-dp-content-wrapper".to_string(),
        ScopeHandle::root("popover")
            .child(
                ScopeHandle::root("date-picker")
                    .child("inner-dp")
                    .qualify("popover"),
            )
            .qualify("portal"),
        // DateRangePicker [content, wrapper, portal]
        "inner-drp-content".to_string(),
        "inner-drp-content-wrapper".to_string(),
        ScopeHandle::root("popover")
            .child(
                ScopeHandle::root("date-range-picker")
                    .child("inner-drp")
                    .qualify("popover"),
            )
            .qualify("portal"),
        // Menu + SubMenu content
        "inner-menu-content".to_string(),
        "inner-sub-content".to_string(),
        // Combobox [content, wrapper, portal]
        "inner-cbx-content".to_string(),
        "inner-cbx-content-floating-wrapper".to_string(),
        "inner-cbx-content-portal-root".to_string(),
        // Tooltip [content, wrapper, portal]
        "inner-tt-content".to_string(),
        ScopeHandle::root("tooltip").child("inner-tt").qualify("wrapper"),
        ScopeHandle::root("tooltip").child("inner-tt").qualify("portal"),
        // HoverCard [content, wrapper, portal]
        "inner-hc-content".to_string(),
        ScopeHandle::root("hover-card")
            .child("inner-hc")
            .qualify("wrapper"),
        ScopeHandle::root("hover-card")
            .child("inner-hc")
            .qualify("portal"),
        // Nested Dialog [content, overlay, portal]
        "inner-dlg-content".to_string(),
        "inner-dlg-overlay".to_string(),
        ScopeHandle::root("dialog").child("inner-dlg").qualify("portal"),
    ];

    dom.in_scope(dioxus::core::ScopeId::ROOT, || {
        let dlg_branches = outer_dlg.branches();

        for expected in &expected_in_outer {
            assert!(
                dlg_branches.contains(expected),
                "expected `{expected}` in outer Dialog branches: {dlg_branches:?}"
            );
        }

        // Menu's own wrapper + portal AND submenu's wrapper + portal must also be present
        let menu_wrapper = menu_root.root_surface.current_wrapper_id();
        let menu_portal = format!("{}-portal", menu_root.root_surface.surface_key);
        assert!(
            dlg_branches.contains(&menu_wrapper),
            "expected menu wrapper `{menu_wrapper}` in outer Dialog branches: {dlg_branches:?}"
        );
        assert!(
            dlg_branches.contains(&menu_portal),
            "expected menu portal `{menu_portal}` in outer Dialog branches: {dlg_branches:?}"
        );

        // MenuSurfaceRuntime itself must have submenu content registered in its `branches`!
        let menu_branches = menu_root.root_surface.branches();
        assert!(
            menu_branches.iter().any(|b| b == "inner-sub-content"),
            "expected submenu content in MenuSurfaceRuntime.branches(): {menu_branches:?}"
        );

        // Popover itself must have grandchild Select branches too
        let pop_branches = pop_rt.branches();
        assert!(
            pop_branches.contains(&format!("{sel_content}-portal-root")),
            "expected grandchild select portal in Popover branches: {pop_branches:?}"
        );
    });

    drop(dom);
    assert!(
        active_overlay_layers_snapshot().is_empty(),
        "dropping nested overlays must clean up ACTIVE_OVERLAY_LAYERS, left: {:?}",
        active_overlay_layers_snapshot()
    );
}

// =============================================================================
// Native Blitz DOM Proofs (`FM-PRT-1`, `FM-PRT-3`, `FM-PRT-4`, `FM-PRT-5`, `FM-PRT-6`)
// =============================================================================

#[cfg(feature = "native")]
mod native_portal_parity_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    fn parent_element_id_of(
        harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        selector: &str,
    ) -> Option<String> {
        let node_id = harness.query(selector)?;
        let parent_id = harness.base().get_node(node_id)?.parent?;
        harness
            .base()
            .get_node(parent_id)?
            .element_data()?
            .attr(blitz_dom::local_name!("id"))
            .map(str::to_string)
    }

    fn find_node_by_exact_id(
        harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        target_id: &str,
    ) -> Option<blitz_dom::NodeId> {
        harness
            .query_all("[id]")
            .into_iter()
            .find(|&nid| {
                harness
                    .base()
                    .get_node(nid)
                    .and_then(|n| n.element_data())
                    .and_then(|el| el.attr(blitz_dom::local_name!("id")))
                    == Some(target_id)
            })
    }

    fn attr_of_node(
        harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        node_id: blitz_dom::NodeId,
        attr_name: &str,
    ) -> Option<String> {
        harness
            .base()
            .get_node(node_id)?
            .element_data()?
            .attrs()
            .iter()
            .find(|a| &*a.name.local == attr_name)
            .map(|a| a.value.clone())
    }

    fn has_attr_by_id(
        harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        target_id: &str,
        attr_name: &str,
    ) -> bool {
        let Some(node_id) = find_node_by_exact_id(harness, target_id) else {
            return false;
        };
        attr_of_node(harness, node_id, attr_name).is_some()
    }

    /// FM-PRT-1 & FM-PRT-5 (Native Blitz DOM): Verify all 7 `*Portal` components emit
    /// permanent `[data-portal-anchor]` outer wrappers (both closed and open), inner
    /// `#{pid}[data-portal-host="default"]`, and that `SelectPortal`/`ComboboxPortal`
    /// with `PortalHost::Inline` or `disabled: true` emit `data-portal-teleported="false"`.
    #[test]
    fn fm_prt_1_and_5_native_dom_anchor_and_inline_disabled_attributes() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();

        fn app() -> Element {
            let mut open = use_signal(|| false);
            rsx! {
                div { id: "prt1-native-root",
                    button {
                        id: "toggle-all",
                        style: "width: 100px; height: 30px;",
                        onclick: move |_| open.toggle(),
                        "Toggle"
                    }
                    if open() {
                        div { AllSevenPortalsHarness { open: true, host: None } }
                    } else {
                        section { AllSevenPortalsHarness { open: false, host: None } }
                    }

                    // FM-PRT-5 inline & disabled SelectPortal + ComboboxPortal
                    SelectRoot {
                        id: Some("n5-sel-inline".to_string()),
                        open: true,
                        SelectTrigger { "SelInline" }
                        SelectPortal {
                            host: Some(PortalHost::inline()),
                            SelectContent {
                                SelectItem { value: "a".to_string(), "A" }
                            }
                        }
                    }
                    SelectRoot {
                        id: Some("n5-sel-disabled".to_string()),
                        open: true,
                        SelectTrigger { "SelDis" }
                        SelectPortal {
                            host: Some(PortalHost::default_host()),
                            disabled: true,
                            SelectContent {
                                SelectItem { value: "a".to_string(), "A" }
                            }
                        }
                    }
                    ComboboxRoot {
                        id: Some("n5-cbx-disabled".to_string()),
                        default_open: true,
                        ComboboxInput {}
                        ComboboxPortal {
                            host: Some(PortalHost::default_host()),
                            disabled: true,
                            ComboboxContent {
                                id: Some("n5-cbx-dis-content".to_string()),
                                ComboboxItem { value: "a".to_string(), "A" }
                            }
                        }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(app);
        drain_polls_and_resolve(&mut harness);

        // Closed AllSevenPortalsHarness still renders all 7 [data-portal-anchor] elements!
        let closed_anchors = harness.query_all("#parent-p [data-portal-anchor]");
        assert_eq!(
            closed_anchors.len(),
            7,
            "all 7 closed portals must render permanent [data-portal-anchor] inside #parent-p"
        );
        assert!(
            harness
                .query_all("#parent-p [data-portal-host]")
                .is_empty(),
            "closed portals must not mount inner [data-portal-host]"
        );

        // Verify FM-PRT-5 DOM attributes on inline & disabled SelectPortal + ComboboxPortal.
        let sel_inline_pid = format!(
            "{}-portal-root",
            ScopeHandle::root("select")
                .child("n5-sel-inline")
                .qualify("content")
        );
        let sel_inline_node = find_node_by_exact_id(&harness, &sel_inline_pid)
            .expect("inline SelectPortal root node must exist");
        assert_eq!(
            attr_of_node(&harness, sel_inline_node, "data-portal-host").as_deref(),
            Some("inline")
        );
        assert_eq!(
            attr_of_node(&harness, sel_inline_node, "data-portal-disabled").as_deref(),
            Some("false")
        );
        assert_eq!(
            attr_of_node(&harness, sel_inline_node, "data-portal-teleported").as_deref(),
            Some("false")
        );

        let sel_dis_pid = format!(
            "{}-portal-root",
            ScopeHandle::root("select")
                .child("n5-sel-disabled")
                .qualify("content")
        );
        let sel_dis_node = find_node_by_exact_id(&harness, &sel_dis_pid)
            .expect("disabled SelectPortal root node must exist");
        assert_eq!(
            attr_of_node(&harness, sel_dis_node, "data-portal-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(
            attr_of_node(&harness, sel_dis_node, "data-portal-teleported").as_deref(),
            Some("false")
        );

        assert_eq!(
            harness
                .attr("#n5-cbx-dis-content-portal-root", "data-portal-disabled")
                .as_deref(),
            Some("true")
        );
        assert_eq!(
            harness
                .attr("#n5-cbx-dis-content-portal-root", "data-portal-teleported")
                .as_deref(),
            Some("false")
        );

        // Toggle open -> all 7 inner [data-portal-host="default"] mount under [data-portal-anchor].
        click_and_pump(&mut harness, "#toggle-all");
        let open_hosts = harness.query_all("#parent-p [data-portal-host=\"default\"]");
        assert_eq!(
            open_hosts.len(),
            7,
            "all 7 open portals must emit inner [data-portal-host=\"default\"]"
        );

        drop(harness);
        clear_active_overlay_stack();
    }

    /// FM-PRT-3 & FM-PRT-4: Adjacent sibling conditional mount stays inside parent `P`,
    /// and branch-swap unmount (`if cond { Portal } else { Fallback }`) mounts `#fallback`
    /// cleanly inside `P` with zero panic and proper teleport/remove lifecycle.
    #[test]
    fn fm_prt_3_and_4_native_adjacent_sibling_and_branch_swap_unmount() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();
        let _rec = PortalRecorderGuard::start();

        let expected_portal_id = ScopeHandle::root("dialog")
            .child("swap-dlg")
            .qualify("portal");

        fn app() -> Element {
            let mut show_sibling = use_signal(|| false);
            let mut show_portal = use_signal(|| true);

            rsx! {
                div { id: "app-root",
                    button {
                        id: "toggle-sibling",
                        style: "width: 100px; height: 30px;",
                        onclick: move |_| show_sibling.toggle(),
                        "Toggle Sibling"
                    }
                    button {
                        id: "toggle-portal",
                        style: "width: 100px; height: 30px;",
                        onclick: move |_| show_portal.toggle(),
                        "Toggle Portal"
                    }
                    div { id: "portal-parent-p",
                        if show_portal() {
                            DialogRoot {
                                id: Some("swap-dlg".to_string()),
                                default_open: true,
                                mode: Some(DialogMode::NonModal),
                                DialogPortal {
                                    DialogContent {
                                        id: Some("swap-dlg-content".to_string()),
                                        outside_interaction: Some(DialogOutsideInteractionPolicy::alert_default()),
                                        style: Some("width: 160px; height: 60px;".to_string()),
                                        "Swappable Dialog"
                                    }
                                }
                            }
                        } else {
                            div {
                                id: "fallback-node",
                                style: "width: 120px; height: 30px;",
                                "Fallback"
                            }
                        }
                        if show_sibling() {
                            span { id: "adj-sibling", "Adjacent Sibling" }
                        }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(app);
        drain_polls_and_resolve(&mut harness);

        assert!(harness.query("#swap-dlg-content").is_some());
        assert!(harness.query("#adj-sibling").is_none());
        let initial_ops = take_portal_ops();
        assert!(
            initial_ops.iter().any(|op| matches!(
                op,
                PortalOp::Teleport {
                    element_id,
                    host_id: None
                } if element_id == &expected_portal_id
            )),
            "expected initial default teleport for {expected_portal_id}, got {initial_ops:?}"
        );

        // FM-PRT-3: Mount adjacent sibling while portal is OPEN; must stay in #portal-parent-p.
        click_and_pump(&mut harness, "#toggle-sibling");
        assert!(harness.query("#adj-sibling").is_some());
        assert_eq!(
            parent_element_id_of(&harness, "#adj-sibling").as_deref(),
            Some("portal-parent-p"),
            "adjacent sibling must mount inside parent P"
        );

        // FM-PRT-4: Branch-swap true -> false while portal is OPEN.
        click_and_pump(&mut harness, "#toggle-portal");
        assert!(
            harness.query("#swap-dlg-content").is_none(),
            "old portal content must be unmounted after branch swap"
        );
        assert!(
            harness.query("#fallback-node").is_some(),
            "fallback node must mount after branch swap"
        );
        assert_eq!(
            parent_element_id_of(&harness, "#fallback-node").as_deref(),
            Some("portal-parent-p"),
            "fallback node must mount inside parent P"
        );
        let swap_out_ops = take_portal_ops();
        assert!(
            swap_out_ops.iter().any(|op| matches!(
                op,
                PortalOp::Remove { element_id } if element_id == &expected_portal_id
            )),
            "branch-swap unmount must remove teleported portal node, got {swap_out_ops:?}"
        );

        // Branch-swap false -> true: re-mounts and re-teleports cleanly.
        click_and_pump(&mut harness, "#toggle-portal");
        assert!(harness.query("#swap-dlg-content").is_some());
        assert!(harness.query("#fallback-node").is_none());
        let swap_back_ops = take_portal_ops();
        assert!(
            swap_back_ops.iter().any(|op| matches!(
                op,
                PortalOp::Teleport {
                    element_id,
                    host_id: None
                } if element_id == &expected_portal_id
            )),
            "branch-swap back to true must re-teleport portal node, got {swap_back_ops:?}"
        );

        drop(harness);
        clear_active_overlay_stack();
    }

    /// FM-PRT-6 (Native Blitz): Modal `Dialog` containing nested `Popover`, `DropdownMenu` (+ submenu),
    /// and nested `Dialog`:
    /// - Background sibling outside modal dialog gets `inert` + `aria-hidden="true"`.
    /// - Nested overlay portal/wrapper/content nodes stay EXEMPT from `inert`/`aria-hidden="true"`.
    /// - Interacting inside nested overlay keeps outer `Dialog` open.
    /// - `Escape` dismisses ONLY the innermost overlay while outer `Dialog` remains open,
    ///   and a subsequent `Escape` dismisses outer `Dialog`.
    #[test]
    fn fm_prt_6_native_modal_dialog_shield_exemption_and_innermost_escape() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();

        fn app() -> Element {
            let mut outer_open = use_signal(|| true);
            let mut pop_open = use_signal(|| false);
            let mut menu_open = use_signal(|| false);
            let mut sub_open = use_signal(|| false);
            let mut inner_dlg_open = use_signal(|| false);
            let mut inner_clicks = use_signal(|| 0usize);

            rsx! {
                div { id: "page-root", style: "padding: 20px;",
                    div { id: "bg-sibling", style: "width: 100px; height: 30px;", "Background" }

                    DialogRoot {
                        id: Some("n6-outer".to_string()),
                        open: outer_open(),
                        on_open_change: move |v| outer_open.set(v),
                        DialogTrigger { id: Some("n6-outer-trigger".to_string()), "Open Outer" }
                        DialogPortal {
                            DialogOverlay { id: Some("n6-outer-overlay".to_string()) }
                            DialogContent {
                                id: Some("n6-outer-content".to_string()),
                                style: Some("width: 400px; height: 300px; display: block;".to_string()),

                                span {
                                    id: "click-count",
                                    "data-clicks": "{inner_clicks}",
                                    "{inner_clicks}"
                                }

                                PopoverRoot {
                                    id: Some("n6-pop".to_string()),
                                    open: pop_open(),
                                    on_open_change: move |v| pop_open.set(v),
                                    PopoverTrigger {
                                        id: Some("n6-pop-trigger".to_string()),
                                        style: Some("width: 100px; height: 28px; display: block;".to_string()),
                                        "PopTrig"
                                    }
                                    PopoverPortal {
                                        PopoverContent {
                                            id: Some("n6-pop-content".to_string()),
                                            style: Some("width: 160px; height: 60px; display: block;".to_string()),
                                            button {
                                                id: "n6-pop-btn",
                                                style: "width: 80px; height: 28px; display: block;",
                                                onclick: move |_| inner_clicks += 1,
                                                "Pop Action"
                                            }
                                        }
                                    }
                                }

                                DropdownMenuRoot {
                                    open: Some(menu_open()),
                                    on_open_change: move |v| {
                                        menu_open.set(v);
                                        sub_open.set(v);
                                    },
                                    modal: false,
                                    DropdownMenuTrigger {
                                        id: Some("n6-menu-trigger".to_string()),
                                        style: Some("width: 100px; height: 28px; display: block;".to_string()),
                                        "MenuTrig"
                                    }
                                    DropdownMenuPortal {
                                        DropdownMenuContent {
                                            id: Some("n6-menu-content".to_string()),
                                            style: Some("width: 160px; height: 80px; display: block;".to_string()),
                                            DropdownMenuItem {
                                                id: Some("n6-menu-item".to_string()),
                                                close_on_select: false,
                                                on_select: move |_| inner_clicks += 1,
                                                "Menu Action"
                                            }
                                            DropdownMenuSub {
                                                open: Some(sub_open()),
                                                on_open_change: move |v| sub_open.set(v),
                                                DropdownMenuSubTrigger { "More" }
                                                DropdownMenuPortal {
                                                    DropdownMenuSubContent {
                                                        id: Some("n6-sub-content".to_string()),
                                                        style: Some("width: 140px; height: 60px; display: block;".to_string()),
                                                        DropdownMenuItem {
                                                            id: Some("n6-sub-item".to_string()),
                                                            "Sub Action"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                DialogRoot {
                                    id: Some("n6-inner-dlg".to_string()),
                                    open: inner_dlg_open(),
                                    on_open_change: move |v| inner_dlg_open.set(v),
                                    DialogTrigger {
                                        id: Some("n6-inner-dlg-trigger".to_string()),
                                        style: Some("width: 100px; height: 28px; display: block;".to_string()),
                                        "Open Inner Dlg"
                                    }
                                    DialogPortal {
                                        DialogOverlay { id: Some("n6-inner-dlg-overlay".to_string()) }
                                        DialogContent {
                                            id: Some("n6-inner-dlg-content".to_string()),
                                            style: Some("width: 220px; height: 100px; display: block;".to_string()),
                                            button {
                                                id: "n6-inner-dlg-btn",
                                                style: "width: 90px; height: 28px; display: block;",
                                                onclick: move |_| inner_clicks += 1,
                                                "Inner Dlg Action"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(app);
        drain_polls_and_resolve(&mut harness);
        advance_frame_and_pump(&mut harness, 0.02);

        // Background sibling outside modal dialog MUST be shielded.
        assert!(
            has_attr_by_id(&harness, "bg-sibling", "inert")
                || harness.attr("#bg-sibling", "aria-hidden").as_deref() == Some("true"),
            "background sibling outside modal dialog must be shielded"
        );

        let pop_portal_id = ScopeHandle::root("popover").child("n6-pop").qualify("portal");
        let inner_dlg_portal_id = ScopeHandle::root("dialog")
            .child("n6-inner-dlg")
            .qualify("portal");

        // --- 1. Nested Popover inside Modal Dialog ---
        click_and_pump(&mut harness, "#n6-pop-trigger");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#n6-pop-content").is_some());
        for id in [
            pop_portal_id.as_str(),
            "n6-pop-content-wrapper",
            "n6-pop-content",
        ] {
            let nid = find_node_by_exact_id(&harness, id)
                .unwrap_or_else(|| panic!("expected node `{id}` to exist"));
            assert!(
                attr_of_node(&harness, nid, "inert").is_none(),
                "{id} must NOT be inert under outer modal Dialog"
            );
            assert_ne!(
                attr_of_node(&harness, nid, "aria-hidden").as_deref(),
                Some("true"),
                "{id} must NOT be aria-hidden under outer modal Dialog"
            );
        }
        // Clicking inside nested Popover must NOT dismiss outer Dialog.
        click_and_pump(&mut harness, "#n6-pop-btn");
        assert!(harness.query("#n6-outer-content").is_some());
        assert_eq!(
            harness.attr("#click-count", "data-clicks").as_deref(),
            Some("1")
        );

        // Escape dismisses ONLY nested Popover; outer Dialog stays OPEN.
        focus_node(&mut harness, "#n6-pop-content");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        for _ in 0..6 {
            advance_frame_and_pump(&mut harness, 0.02);
        }
        assert!(
            harness.query("#n6-pop-content").is_none(),
            "Escape must dismiss nested Popover"
        );
        assert!(
            harness.query("#n6-outer-content").is_some(),
            "outer Dialog must remain OPEN after nested Popover handles Escape"
        );

        // --- 2. Nested DropdownMenu + SubMenu inside Modal Dialog ---
        click_and_pump(&mut harness, "#n6-menu-trigger");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#n6-menu-content").is_some());
        assert!(harness.query("#n6-sub-content").is_some());
        for id in ["n6-menu-content", "n6-sub-content"] {
            let nid = find_node_by_exact_id(&harness, id)
                .unwrap_or_else(|| panic!("expected node `{id}` to exist"));
            assert!(
                attr_of_node(&harness, nid, "inert").is_none(),
                "{id} must NOT be inert under outer modal Dialog"
            );
            assert_ne!(
                attr_of_node(&harness, nid, "aria-hidden").as_deref(),
                Some("true"),
                "{id} must NOT be aria-hidden under outer modal Dialog"
            );
        }
        // Escape dismisses ONLY nested DropdownMenu tree; outer Dialog stays OPEN.
        focus_node(&mut harness, "#n6-menu-content");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        for _ in 0..6 {
            advance_frame_and_pump(&mut harness, 0.02);
        }
        assert!(
            harness.query("#n6-menu-content").is_none(),
            "Escape must dismiss nested DropdownMenu"
        );
        assert!(
            harness.query("#n6-outer-content").is_some(),
            "outer Dialog must remain OPEN after nested DropdownMenu handles Escape"
        );

        // --- 3. Nested Dialog inside Modal Dialog ---
        click_and_pump(&mut harness, "#n6-inner-dlg-trigger");
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(harness.query("#n6-inner-dlg-content").is_some());
        for id in [inner_dlg_portal_id.as_str(), "n6-inner-dlg-content"] {
            let nid = find_node_by_exact_id(&harness, id)
                .unwrap_or_else(|| panic!("expected node `{id}` to exist"));
            assert!(
                attr_of_node(&harness, nid, "inert").is_none(),
                "{id} must NOT be inert under outer modal Dialog"
            );
            assert_ne!(
                attr_of_node(&harness, nid, "aria-hidden").as_deref(),
                Some("true"),
                "{id} must NOT be aria-hidden under outer modal Dialog"
            );
        }
        // Interacting inside nested Dialog keeps outer Dialog open.
        click_and_pump(&mut harness, "#n6-inner-dlg-btn");
        assert!(harness.query("#n6-outer-content").is_some());
        assert_eq!(
            harness.attr("#click-count", "data-clicks").as_deref(),
            Some("2")
        );

        // Escape dismisses ONLY nested Dialog; outer Dialog stays OPEN.
        focus_node(&mut harness, "#n6-inner-dlg-content");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.05);
        assert!(
            harness.query("#n6-inner-dlg-content").is_none(),
            "Escape must dismiss nested Dialog"
        );
        assert!(
            harness.query("#n6-outer-content").is_some(),
            "outer Dialog must remain OPEN after nested Dialog handles Escape"
        );

        // Second Escape now dismisses outer Dialog.
        focus_node(&mut harness, "#n6-outer-content");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.05);
        assert!(
            harness.query("#n6-outer-content").is_none(),
            "second Escape must dismiss outer Dialog"
        );

        drop(harness);
        clear_active_overlay_stack();
    }
}

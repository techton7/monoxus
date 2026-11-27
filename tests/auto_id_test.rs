//! Auto-ID parity failure-mode verification (`FM-AID-1`, `FM-AID-2`, `FM-AID-3`).
//!
//! Verifies:
//! - `FM-AID-1`: Dual-instance DOM uniqueness and interaction isolation across all 9 migrated
//!   primitives (10 sites: `AccordionRoot`, `AlertDialogRoot`, `CollapsibleRoot`, `DialogRoot`,
//!   `ScrollAreaRoot`, `SelectRoot`, `TabsRoot`, `ToggleGroupRoot`, `TooltipRoot`, `TooltipProvider`).
//! - `FM-AID-2`: Explicit custom `id` precedence and reactive prop updates.
//! - `FM-AID-3`: Dual repeatable `DialogClose` and `PopoverClose` DOM uniqueness, explicit `id`
//!   override, and close + trigger focus restoration.

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use dioxus::prelude::*;
use monoxus::{
    accordion::{
        AccordionContent, AccordionContext, AccordionHeader, AccordionItem, AccordionRoot,
        AccordionTrigger,
    },
    alert_dialog::AlertDialogRoot,
    collapsible::{
        CollapsibleContent, CollapsibleContext, CollapsibleRoot, CollapsibleTrigger,
    },
    dialog::{
        DialogClose, DialogContent, DialogContext, DialogDescription, DialogOverlay, DialogPortal,
        DialogRoot, DialogTitle, DialogTrigger,
    },
    popover::{
        PopoverClose, PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger,
    },
    scroll_area::{
        ScrollAreaContext, ScrollAreaRoot, ScrollAreaScrollbar, ScrollAreaThumb,
        ScrollAreaViewport,
    },
    select::{
        SelectContent, SelectContext, SelectItem, SelectItemData, SelectPortal, SelectRoot,
        SelectTrigger, SelectValue,
    },
    tabs::{TabsContent, TabsContext, TabsList, TabsRoot, TabsTrigger},
    toggle_group::{ToggleGroupContext, ToggleGroupItem, ToggleGroupRoot},
    tooltip::{
        TooltipContent, TooltipPortal, TooltipProvider, TooltipProviderContext, TooltipRoot,
        TooltipRootContext, TooltipTrigger,
    },
};

#[derive(Clone, Default, PartialEq)]
struct CollectedIds {
    ids: Rc<RefCell<Vec<(String, String)>>>,
}

impl CollectedIds {
    fn record(&self, label: &str, id: impl Into<String>) {
        self.ids.borrow_mut().push((label.to_string(), id.into()));
    }

    fn entries(&self) -> Vec<(String, String)> {
        self.ids.borrow().clone()
    }
}

#[component]
fn AccordionIdProbe(label: String, sink: CollectedIds) -> Element {
    let ctx = use_context::<AccordionContext>();
    sink.record(&label, ctx.runtime.root().id().to_string());
    rsx! {}
}

#[component]
fn DialogIdProbe(label: String, sink: CollectedIds) -> Element {
    let ctx = use_context::<DialogContext>();
    sink.record(&label, ctx.runtime.read().root().id().to_string());
    rsx! {}
}

#[component]
fn CollapsibleIdProbe(label: String, sink: CollectedIds) -> Element {
    let ctx = use_context::<CollapsibleContext>();
    sink.record(&label, ctx.runtime.read().root().id().to_string());
    rsx! {}
}

#[component]
fn ScrollAreaIdProbe(label: String, sink: CollectedIds) -> Element {
    let ctx = use_context::<ScrollAreaContext>();
    sink.record(&label, ctx.relationships.read().root_id().to_string());
    rsx! {}
}

#[component]
fn SelectIdProbe(label: String, sink: CollectedIds) -> Element {
    let ctx = use_context::<SelectContext>();
    sink.record(&label, ctx.runtime.root_attributes().id().to_string());
    rsx! {}
}

#[component]
fn TabsIdProbe(label: String, sink: CollectedIds) -> Element {
    let ctx = use_context::<TabsContext>();
    sink.record(&label, ctx.runtime.root().id().to_string());
    rsx! {}
}

#[component]
fn ToggleGroupIdProbe(label: String, sink: CollectedIds) -> Element {
    let ctx = use_context::<ToggleGroupContext>();
    sink.record(
        &label,
        ctx.runtime.root_attributes(None).id().to_string(),
    );
    rsx! {}
}

#[component]
fn TooltipRootIdProbe(label: String, sink: CollectedIds) -> Element {
    let ctx = use_context::<TooltipRootContext>();
    sink.record(&label, ctx.runtime.read().root().id().to_string());
    rsx! {}
}

#[component]
fn TooltipProviderIdProbe(label: String, sink: CollectedIds) -> Element {
    let ctx = use_context::<TooltipProviderContext>();
    sink.record(&label, ctx.runtime.read().provider().id().to_string());
    rsx! {}
}

#[test]
fn fm_aid_1_vdom_dual_instance_unique_ids() {
    let sink = CollectedIds::default();
    let sink_for_app = sink.clone();

    let mut dom = VirtualDom::new_with_props(
        |sink: CollectedIds| {
            rsx! {
                AccordionRoot { AccordionIdProbe { label: "accordion-1".to_string(), sink: sink.clone() } }
                AccordionRoot { AccordionIdProbe { label: "accordion-2".to_string(), sink: sink.clone() } }

                AlertDialogRoot { DialogIdProbe { label: "alert-dialog-1".to_string(), sink: sink.clone() } }
                AlertDialogRoot { DialogIdProbe { label: "alert-dialog-2".to_string(), sink: sink.clone() } }

                CollapsibleRoot { CollapsibleIdProbe { label: "collapsible-1".to_string(), sink: sink.clone() } }
                CollapsibleRoot { CollapsibleIdProbe { label: "collapsible-2".to_string(), sink: sink.clone() } }

                DialogRoot { DialogIdProbe { label: "dialog-1".to_string(), sink: sink.clone() } }
                DialogRoot { DialogIdProbe { label: "dialog-2".to_string(), sink: sink.clone() } }

                ScrollAreaRoot { ScrollAreaIdProbe { label: "scroll-area-1".to_string(), sink: sink.clone() } }
                ScrollAreaRoot { ScrollAreaIdProbe { label: "scroll-area-2".to_string(), sink: sink.clone() } }

                SelectRoot { SelectIdProbe { label: "select-1".to_string(), sink: sink.clone() } }
                SelectRoot { SelectIdProbe { label: "select-2".to_string(), sink: sink.clone() } }

                TabsRoot { TabsIdProbe { label: "tabs-1".to_string(), sink: sink.clone() } }
                TabsRoot { TabsIdProbe { label: "tabs-2".to_string(), sink: sink.clone() } }

                ToggleGroupRoot { ToggleGroupIdProbe { label: "toggle-group-1".to_string(), sink: sink.clone() } }
                ToggleGroupRoot { ToggleGroupIdProbe { label: "toggle-group-2".to_string(), sink: sink.clone() } }

                TooltipRoot { TooltipRootIdProbe { label: "tooltip-root-1".to_string(), sink: sink.clone() } }
                TooltipRoot { TooltipRootIdProbe { label: "tooltip-root-2".to_string(), sink: sink.clone() } }

                TooltipProvider { TooltipProviderIdProbe { label: "tooltip-provider-1".to_string(), sink: sink.clone() } }
                TooltipProvider { TooltipProviderIdProbe { label: "tooltip-provider-2".to_string(), sink: sink.clone() } }
            }
        },
        sink_for_app,
    );
    dom.rebuild_in_place();

    let entries = sink.entries();
    assert_eq!(entries.len(), 20, "expected 20 probed IDs across 10 dual sites");

    let mut seen = HashSet::new();
    for (label, id) in &entries {
        assert!(!id.is_empty(), "{label} produced an empty ID");
        assert!(
            id.contains("monoxus-"),
            "{label} expected auto-generated monoxus-* ID, got '{id}'"
        );
        assert!(
            seen.insert(id.clone()),
            "duplicate auto-generated ID '{id}' found at {label}"
        );
    }
}

#[test]
fn fm_aid_2_vdom_custom_id_precedence() {
    let sink = CollectedIds::default();
    let sink_for_app = sink.clone();

    let mut dom = VirtualDom::new_with_props(
        |sink: CollectedIds| {
            let custom = Some("custom-scope".to_string());
            rsx! {
                AccordionRoot { id: custom.clone(), AccordionIdProbe { label: "accordion".to_string(), sink: sink.clone() } }
                AlertDialogRoot { id: custom.clone(), DialogIdProbe { label: "alert-dialog".to_string(), sink: sink.clone() } }
                CollapsibleRoot { id: custom.clone(), CollapsibleIdProbe { label: "collapsible".to_string(), sink: sink.clone() } }
                DialogRoot { id: custom.clone(), DialogIdProbe { label: "dialog".to_string(), sink: sink.clone() } }
                ScrollAreaRoot { id: custom.clone(), ScrollAreaIdProbe { label: "scroll-area".to_string(), sink: sink.clone() } }
                SelectRoot { id: custom.clone(), SelectIdProbe { label: "select".to_string(), sink: sink.clone() } }
                TabsRoot { id: custom.clone(), TabsIdProbe { label: "tabs".to_string(), sink: sink.clone() } }
                ToggleGroupRoot { id: custom.clone(), ToggleGroupIdProbe { label: "toggle-group".to_string(), sink: sink.clone() } }
                TooltipRoot { id: custom.clone(), TooltipRootIdProbe { label: "tooltip-root".to_string(), sink: sink.clone() } }
                TooltipProvider { id: custom.clone(), TooltipProviderIdProbe { label: "tooltip-provider".to_string(), sink: sink.clone() } }
            }
        },
        sink_for_app,
    );
    dom.rebuild_in_place();

    let entries = sink.entries();
    assert_eq!(entries.len(), 10, "expected 10 probed custom IDs");
    for (label, id) in &entries {
        assert!(
            id.contains("custom-scope"),
            "{label} did not derive from explicit custom-scope id: got '{id}'"
        );
        assert!(
            !id.contains("monoxus-"),
            "{label} fell back to monoxus-* despite explicit id: got '{id}'"
        );
    }
}

#[cfg(feature = "native")]
mod native_auto_id_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use std::collections::BTreeMap;

    fn collect_all_dom_ids(
        harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
    ) -> Vec<String> {
        let node_ids = harness.query_all("[id]");
        let base = harness.base();
        let mut ids = Vec::new();
        for node_id in node_ids {
            if let Some(node) = base.get_node(node_id) {
                if let Some(id_val) = node.attr(blitz_dom::local_name!("id")) {
                    if !id_val.is_empty() {
                        ids.push(id_val.to_string());
                    }
                }
            }
        }
        ids
    }

    fn assert_zero_duplicate_dom_ids(
        harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        context: &str,
    ) {
        let ids = collect_all_dom_ids(harness);
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for id in ids {
            *counts.entry(id).or_insert(0) += 1;
        }
        let duplicates: Vec<_> = counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .collect();
        assert!(
            duplicates.is_empty(),
            "{context}: found duplicate DOM id attributes: {duplicates:?}"
        );
    }

    fn attr_of(
        harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
        selector: &str,
        attr_name: &str,
    ) -> Option<String> {
        let node_id = harness.query(selector)?;
        let base = harness.base();
        let node = base.get_node(node_id)?;
        let local = blitz_dom::LocalName::from(attr_name);
        node.attr(local).map(ToString::to_string)
    }

    #[test]
    fn fm_aid_1_native_dual_instance_dom_uniqueness_and_isolation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn dual_instances_app() -> Element {
            let items = vec![
                SelectItemData::new("a", "Alpha", false),
                SelectItemData::new("b", "Beta", false),
            ];

            rsx! {
                div {
                    style: "display: flex; flex-direction: column; gap: 8px; padding: 12px;",

                    // 1. AccordionRoot x2
                    div {
                        class: "acc-1",
                        AccordionRoot {
                            AccordionItem {
                                value: "item-1".to_string(),
                                AccordionHeader {
                                    AccordionTrigger { class: "acc-1-trigger", "Acc 1" }
                                }
                                AccordionContent { class: "acc-1-content", "Acc 1 Body" }
                            }
                        }
                    }
                    div {
                        class: "acc-2",
                        AccordionRoot {
                            AccordionItem {
                                value: "item-1".to_string(),
                                AccordionHeader {
                                    AccordionTrigger { class: "acc-2-trigger", "Acc 2" }
                                }
                                AccordionContent { class: "acc-2-content", "Acc 2 Body" }
                            }
                        }
                    }

                    // 2. AlertDialogRoot x2
                    div {
                        class: "ad-1",
                        AlertDialogRoot {
                            default_open: false,
                            DialogTrigger { class: "ad-1-trigger", "AD 1" }
                            DialogPortal {
                                DialogOverlay {}
                                DialogContent {
                                    class: "ad-1-content",
                                    DialogTitle { "AD 1 Title" }
                                    DialogDescription { "AD 1 Desc" }
                                }
                            }
                        }
                    }
                    div {
                        class: "ad-2",
                        AlertDialogRoot {
                            default_open: false,
                            DialogTrigger { class: "ad-2-trigger", "AD 2" }
                            DialogPortal {
                                DialogOverlay {}
                                DialogContent {
                                    class: "ad-2-content",
                                    DialogTitle { "AD 2 Title" }
                                    DialogDescription { "AD 2 Desc" }
                                }
                            }
                        }
                    }

                    // 3. CollapsibleRoot x2
                    div {
                        class: "col-1",
                        CollapsibleRoot {
                            CollapsibleTrigger { class: "col-1-trigger", "Col 1" }
                            CollapsibleContent { class: "col-1-content", "Col 1 Body" }
                        }
                    }
                    div {
                        class: "col-2",
                        CollapsibleRoot {
                            CollapsibleTrigger { class: "col-2-trigger", "Col 2" }
                            CollapsibleContent { class: "col-2-content", "Col 2 Body" }
                        }
                    }

                    // 4. DialogRoot x2
                    div {
                        class: "dlg-1",
                        DialogRoot {
                            DialogTrigger { class: "dlg-1-trigger", "Dlg 1" }
                            DialogPortal {
                                DialogOverlay {}
                                DialogContent {
                                    class: "dlg-1-content",
                                    DialogTitle { "Dlg 1 Title" }
                                    DialogDescription { "Dlg 1 Desc" }
                                }
                            }
                        }
                    }
                    div {
                        class: "dlg-2",
                        DialogRoot {
                            DialogTrigger { class: "dlg-2-trigger", "Dlg 2" }
                            DialogPortal {
                                DialogOverlay {}
                                DialogContent {
                                    class: "dlg-2-content",
                                    DialogTitle { "Dlg 2 Title" }
                                    DialogDescription { "Dlg 2 Desc" }
                                }
                            }
                        }
                    }

                    // 5. ScrollAreaRoot x2
                    div {
                        class: "sa-1",
                        ScrollAreaRoot {
                            ScrollAreaViewport { class: "sa-1-vp", "SA 1" }
                            ScrollAreaScrollbar {
                                orientation: monoxus::foundation::shared::Orientation::Vertical,
                                ScrollAreaThumb {}
                            }
                        }
                    }
                    div {
                        class: "sa-2",
                        ScrollAreaRoot {
                            ScrollAreaViewport { class: "sa-2-vp", "SA 2" }
                            ScrollAreaScrollbar {
                                orientation: monoxus::foundation::shared::Orientation::Vertical,
                                ScrollAreaThumb {}
                            }
                        }
                    }

                    // 6. SelectRoot x2
                    div {
                        class: "sel-1",
                        SelectRoot {
                            items: Some(items.clone()),
                            SelectTrigger { class: "sel-1-trigger", SelectValue { placeholder: "Pick 1" } }
                            SelectPortal {
                                SelectContent {
                                    SelectItem { value: "a".to_string(), "Alpha" }
                                }
                            }
                        }
                    }
                    div {
                        class: "sel-2",
                        SelectRoot {
                            items: Some(items),
                            SelectTrigger { class: "sel-2-trigger", SelectValue { placeholder: "Pick 2" } }
                            SelectPortal {
                                SelectContent {
                                    SelectItem { value: "a".to_string(), "Alpha" }
                                }
                            }
                        }
                    }

                    // 7. TabsRoot x2
                    div {
                        class: "tabs-1",
                        TabsRoot {
                            default_value: Some("a".to_string()),
                            TabsList {
                                TabsTrigger { class: "tabs-1-a", value: "a".to_string(), "A" }
                                TabsTrigger { class: "tabs-1-b", value: "b".to_string(), "B" }
                            }
                            TabsContent { value: "a".to_string(), "Panel A" }
                            TabsContent { value: "b".to_string(), "Panel B" }
                        }
                    }
                    div {
                        class: "tabs-2",
                        TabsRoot {
                            default_value: Some("a".to_string()),
                            TabsList {
                                TabsTrigger { class: "tabs-2-a", value: "a".to_string(), "A" }
                                TabsTrigger { class: "tabs-2-b", value: "b".to_string(), "B" }
                            }
                            TabsContent { value: "a".to_string(), "Panel A" }
                            TabsContent { value: "b".to_string(), "Panel B" }
                        }
                    }

                    // 8. ToggleGroupRoot x2
                    div {
                        class: "tg-1",
                        ToggleGroupRoot {
                            ToggleGroupItem { class: "tg-1-item", value: "x".to_string(), "X1" }
                        }
                    }
                    div {
                        class: "tg-2",
                        ToggleGroupRoot {
                            ToggleGroupItem { class: "tg-2-item", value: "x".to_string(), "X2" }
                        }
                    }

                    // 9 & 10. TooltipProvider x2 + TooltipRoot x2
                    TooltipProvider {
                        TooltipRoot {
                            default_open: true,
                            TooltipTrigger { class: "tt-1-trigger", "TT 1" }
                            TooltipPortal { TooltipContent { class: "tt-1-content", "TT 1 Tip" } }
                        }
                    }
                    TooltipProvider {
                        TooltipRoot {
                            default_open: true,
                            TooltipTrigger { class: "tt-2-trigger", "TT 2" }
                            TooltipPortal { TooltipContent { class: "tt-2-content", "TT 2 Tip" } }
                        }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(dual_instances_app);
        assert_zero_duplicate_dom_ids(&harness, "initial mount of all 10 dual primitive sites");

        // Distinct DOM IDs across sibling instances
        for (a, b, label) in [
            (".acc-1-trigger", ".acc-2-trigger", "AccordionTrigger"),
            (".ad-1-trigger", ".ad-2-trigger", "AlertDialogTrigger"),
            (".col-1-trigger", ".col-2-trigger", "CollapsibleTrigger"),
            (".dlg-1-trigger", ".dlg-2-trigger", "DialogTrigger"),
            (".sa-1-vp", ".sa-2-vp", "ScrollAreaViewport"),
            (".sel-1-trigger", ".sel-2-trigger", "SelectTrigger"),
            (".tabs-1-a", ".tabs-2-a", "TabsTrigger"),
            (".tg-1-item", ".tg-2-item", "ToggleGroupItem"),
            (".tt-1-trigger", ".tt-2-trigger", "TooltipTrigger"),
        ] {
            let id_a = attr_of(&harness, a, "id")
                .unwrap_or_else(|| panic!("missing id on {a} ({label})"));
            let id_b = attr_of(&harness, b, "id")
                .unwrap_or_else(|| panic!("missing id on {b} ({label})"));
            assert_ne!(id_a, id_b, "sibling {label} DOM ids must differ");
        }

        // Distinct ARIA targets on mounted sibling panels/tooltips
        let tabs_1_controls =
            attr_of(&harness, ".tabs-1-a", "aria-controls").expect("tabs-1-a aria-controls");
        let tabs_2_controls =
            attr_of(&harness, ".tabs-2-a", "aria-controls").expect("tabs-2-a aria-controls");
        assert_ne!(
            tabs_1_controls, tabs_2_controls,
            "sibling TabsTrigger aria-controls must differ"
        );

        let tt_1_desc =
            attr_of(&harness, ".tt-1-trigger", "aria-describedby").expect("tt-1 aria-describedby");
        let tt_2_desc =
            attr_of(&harness, ".tt-2-trigger", "aria-describedby").expect("tt-2 aria-describedby");
        assert_ne!(
            tt_1_desc, tt_2_desc,
            "sibling TooltipTrigger aria-describedby must differ"
        );

        // Interaction isolation: opening/activating instance #1 must never mutate instance #2
        click_and_pump(&mut harness, ".col-1-trigger");
        assert_eq!(
            attr_of(&harness, ".col-1-trigger", "data-state").as_deref(),
            Some("open")
        );
        assert_eq!(
            attr_of(&harness, ".col-2-trigger", "data-state").as_deref(),
            Some("closed")
        );
        // Open instance #2 too and verify distinct mounted aria-controls
        click_and_pump(&mut harness, ".col-2-trigger");
        let col_1_controls =
            attr_of(&harness, ".col-1-trigger", "aria-controls").expect("col-1 aria-controls");
        let col_2_controls =
            attr_of(&harness, ".col-2-trigger", "aria-controls").expect("col-2 aria-controls");
        assert_ne!(
            col_1_controls, col_2_controls,
            "sibling CollapsibleTrigger aria-controls must differ"
        );

        click_and_pump(&mut harness, ".acc-1-trigger");
        assert_eq!(
            attr_of(&harness, ".acc-1-trigger", "data-state").as_deref(),
            Some("open")
        );
        assert_eq!(
            attr_of(&harness, ".acc-2-trigger", "data-state").as_deref(),
            Some("closed")
        );
        click_and_pump(&mut harness, ".acc-2-trigger");
        let acc_1_controls =
            attr_of(&harness, ".acc-1-trigger", "aria-controls").expect("acc-1 aria-controls");
        let acc_2_controls =
            attr_of(&harness, ".acc-2-trigger", "aria-controls").expect("acc-2 aria-controls");
        assert_ne!(
            acc_1_controls, acc_2_controls,
            "sibling AccordionTrigger aria-controls must differ"
        );

        click_and_pump(&mut harness, ".tabs-1-b");
        assert_eq!(
            attr_of(&harness, ".tabs-1-b", "data-state").as_deref(),
            Some("active")
        );
        assert_eq!(
            attr_of(&harness, ".tabs-2-b", "data-state").as_deref(),
            Some("inactive")
        );

        click_and_pump(&mut harness, ".tg-1-item");
        assert_eq!(
            attr_of(&harness, ".tg-1-item", "data-state").as_deref(),
            Some("on")
        );
        assert_eq!(
            attr_of(&harness, ".tg-2-item", "data-state").as_deref(),
            Some("off")
        );

        // Open AlertDialog #1 and confirm AlertDialog #2 stays closed and has distinct ARIA targets
        click_and_pump(&mut harness, ".ad-1-trigger");
        assert_eq!(
            attr_of(&harness, ".ad-1-trigger", "data-state").as_deref(),
            Some("open")
        );
        assert_eq!(
            attr_of(&harness, ".ad-2-trigger", "data-state").as_deref(),
            Some("closed")
        );
        assert!(
            attr_of(&harness, ".ad-1-content", "aria-labelledby").is_some(),
            "opened AlertDialog #1 content should have aria-labelledby"
        );
        assert!(
            attr_of(&harness, ".ad-1-content", "aria-describedby").is_some(),
            "opened AlertDialog #1 content should have aria-describedby"
        );

        assert_zero_duplicate_dom_ids(&harness, "post-interaction dual primitive sites");
    }

    #[test]
    fn fm_aid_2_native_custom_id_precedence_and_reactive_update() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn provider_id_probe() -> Element {
            let ctx = use_context::<monoxus::tooltip::TooltipProviderContext>();
            let provider_id = ctx.runtime.read().provider().id().to_string();
            rsx! {
                div {
                    class: "rx-tt-provider",
                    id: "{provider_id}",
                }
            }
        }

        fn reactive_id_app() -> Element {
            let mut scope_name = use_signal(|| "custom-scope".to_string());
            let current = Some(scope_name());

            rsx! {
                div {
                    style: "display: flex; flex-direction: column; gap: 6px; padding: 12px;",
                    button {
                        id: "switch-scope-btn",
                        onclick: move |_| scope_name.set("updated-scope".to_string()),
                        "Switch Scope"
                    }
                    TooltipProvider {
                        id: current.clone(),
                        provider_id_probe {}
                    }
                    AccordionRoot {
                        id: current.clone(),
                        class: "rx-acc-root",
                        AccordionItem {
                            value: "one".to_string(),
                            AccordionHeader { AccordionTrigger { "A" } }
                            AccordionContent { "B" }
                        }
                    }
                    AlertDialogRoot {
                        id: current.clone(),
                        DialogTrigger { class: "rx-ad-trigger", "AD" }
                    }
                    CollapsibleRoot {
                        id: current.clone(),
                        class: "rx-col-root",
                        CollapsibleTrigger { class: "rx-col-trigger", "C" }
                        CollapsibleContent { "Body" }
                    }
                    DialogRoot {
                        id: current.clone(),
                        DialogTrigger { class: "rx-dlg-trigger", "D" }
                    }
                    ScrollAreaRoot {
                        id: current.clone(),
                        class: "rx-sa-root",
                        ScrollAreaViewport { class: "rx-sa-vp", "S" }
                    }
                    SelectRoot {
                        id: current.clone(),
                        SelectTrigger { class: "rx-sel-trigger", SelectValue { placeholder: "Pick" } }
                    }
                    TabsRoot {
                        id: current.clone(),
                        class: "rx-tabs-root",
                        default_value: Some("t1".to_string()),
                        TabsList { TabsTrigger { value: "t1".to_string(), "T1" } }
                        TabsContent { value: "t1".to_string(), "C1" }
                    }
                    ToggleGroupRoot {
                        id: current.clone(),
                        class: "rx-tg-root",
                        ToggleGroupItem { value: "x".to_string(), "X" }
                    }
                    TooltipRoot {
                        id: current,
                        TooltipTrigger { class: "rx-tt-trigger", "TT" }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(reactive_id_app);

        let selectors = [
            ".rx-tt-provider",
            ".rx-acc-root",
            ".rx-ad-trigger",
            ".rx-col-root",
            ".rx-col-trigger",
            ".rx-dlg-trigger",
            ".rx-sa-root",
            ".rx-sa-vp",
            ".rx-sel-trigger",
            ".rx-tabs-root",
            ".rx-tg-root",
            ".rx-tt-trigger",
        ];

        for sel in selectors {
            let id = attr_of(&harness, sel, "id")
                .unwrap_or_else(|| panic!("missing id on {sel} initially"));
            assert!(
                id.contains("custom-scope"),
                "{sel}: expected id containing 'custom-scope', got '{id}'"
            );
            assert!(
                !id.contains("monoxus-"),
                "{sel}: unexpected monoxus-* fallback when explicit id set, got '{id}'"
            );
        }

        // Reactively update the `id` prop across all migrated primitives
        click_and_pump(&mut harness, "#switch-scope-btn");

        for sel in selectors {
            let id = attr_of(&harness, sel, "id")
                .unwrap_or_else(|| panic!("missing id on {sel} after reactive update"));
            assert!(
                id.contains("updated-scope"),
                "{sel}: expected id to reactively update to 'updated-scope', got '{id}'"
            );
        }
    }

    #[test]
    fn fm_aid_3_native_dual_repeatable_dialog_and_popover_close() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn dual_close_app() -> Element {
            rsx! {
                div {
                    style: "display: flex; flex-direction: column; gap: 16px; padding: 16px;",

                    DialogRoot {
                        DialogTrigger {
                            id: "dlg-trigger",
                            style: "width: 120px; height: 32px;",
                            "Open Dialog"
                        }
                        DialogPortal {
                            DialogOverlay {}
                            DialogContent {
                                id: "dlg-content",
                                style: "width: 240px; height: 140px; display: flex; flex-direction: column; gap: 8px;",
                                DialogTitle { "Dialog" }
                                DialogClose {
                                    class: "dlg-close-1",
                                    style: "width: 80px; height: 28px;",
                                    "X"
                                }
                                DialogClose {
                                    class: "dlg-close-2",
                                    style: "width: 80px; height: 28px;",
                                    "Close"
                                }
                                DialogClose {
                                    id: "explicit-dlg-close",
                                    class: "dlg-close-explicit",
                                    style: "width: 80px; height: 28px;",
                                    "Explicit"
                                }
                            }
                        }
                    }

                    PopoverRoot {
                        PopoverTrigger {
                            id: "pop-trigger",
                            style: "width: 120px; height: 32px;",
                            "Open Popover"
                        }
                        PopoverPortal {
                            PopoverContent {
                                id: "pop-content",
                                style: "width: 240px; height: 140px; display: flex; flex-direction: column; gap: 8px;",
                                PopoverClose {
                                    class: "pop-close-1",
                                    style: "width: 80px; height: 28px;",
                                    "X"
                                }
                                PopoverClose {
                                    class: "pop-close-2",
                                    style: "width: 80px; height: 28px;",
                                    "Done"
                                }
                                PopoverClose {
                                    id: "explicit-pop-close",
                                    class: "pop-close-explicit",
                                    style: "width: 80px; height: 28px;",
                                    "Explicit"
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(dual_close_app);

        // --- DialogClose ---
        click_and_pump(&mut harness, "#dlg-trigger");
        assert!(harness.query("#dlg-content").is_some());
        assert_zero_duplicate_dom_ids(&harness, "dialog open with dual DialogClose");

        let dlg_close_1 = attr_of(&harness, ".dlg-close-1", "id").expect("dlg-close-1 id");
        let dlg_close_2 = attr_of(&harness, ".dlg-close-2", "id").expect("dlg-close-2 id");
        let dlg_close_explicit =
            attr_of(&harness, ".dlg-close-explicit", "id").expect("dlg-close-explicit id");
        assert!(!dlg_close_1.is_empty() && !dlg_close_2.is_empty());
        assert_ne!(
            dlg_close_1, dlg_close_2,
            "sibling DialogClose buttons must have distinct DOM IDs"
        );
        assert_eq!(dlg_close_explicit, "explicit-dlg-close");

        // Click second DialogClose -> closes dialog & restores trigger focus
        click_and_pump(&mut harness, ".dlg-close-2");
        advance_frame_and_pump(&mut harness, 0.35);
        assert!(
            harness.query("#dlg-content").is_none(),
            "dialog should close when second DialogClose is clicked"
        );
        assert_native_focus_id(&harness, "#dlg-trigger");

        // Reopen & click first DialogClose -> closes dialog & restores trigger focus
        click_and_pump(&mut harness, "#dlg-trigger");
        assert!(harness.query("#dlg-content").is_some());
        click_and_pump(&mut harness, ".dlg-close-1");
        advance_frame_and_pump(&mut harness, 0.35);
        assert!(
            harness.query("#dlg-content").is_none(),
            "dialog should close when first DialogClose is clicked"
        );
        assert_native_focus_id(&harness, "#dlg-trigger");

        // --- PopoverClose ---
        click_and_pump(&mut harness, "#pop-trigger");
        assert!(harness.query("#pop-content").is_some());
        assert_zero_duplicate_dom_ids(&harness, "popover open with dual PopoverClose");

        let pop_close_1 = attr_of(&harness, ".pop-close-1", "id").expect("pop-close-1 id");
        let pop_close_2 = attr_of(&harness, ".pop-close-2", "id").expect("pop-close-2 id");
        let pop_close_explicit =
            attr_of(&harness, ".pop-close-explicit", "id").expect("pop-close-explicit id");
        assert!(!pop_close_1.is_empty() && !pop_close_2.is_empty());
        assert_ne!(
            pop_close_1, pop_close_2,
            "sibling PopoverClose buttons must have distinct DOM IDs"
        );
        assert_eq!(pop_close_explicit, "explicit-pop-close");

        // Click second PopoverClose -> closes popover & restores trigger focus
        click_and_pump(&mut harness, ".pop-close-2");
        advance_frame_and_pump(&mut harness, 0.35);
        assert!(
            harness.query("#pop-content").is_none(),
            "popover should close when second PopoverClose is clicked"
        );
        assert_native_focus_id(&harness, "#pop-trigger");

        // Reopen & click first PopoverClose -> closes popover & restores trigger focus
        click_and_pump(&mut harness, "#pop-trigger");
        assert!(harness.query("#pop-content").is_some());
        click_and_pump(&mut harness, ".pop-close-1");
        advance_frame_and_pump(&mut harness, 0.35);
        assert!(
            harness.query("#pop-content").is_none(),
            "popover should close when first PopoverClose is clicked"
        );
        assert_native_focus_id(&harness, "#pop-trigger");
    }
}

use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::{
    foundation::{
        date_time::{CalendarDate, DateRange},
        overlay::{
            active_overlay_layers_snapshot, is_topmost_overlay_layer,
            register_active_overlay_layer, unregister_active_overlay_layer,
        },
        shared::ScopeHandle,
    },
    popover::{
        Popover, PopoverContent, PopoverContext, PopoverOpenFocusPolicy, PopoverPortal,
        PopoverRoot, PopoverRuntime, PopoverTrigger,
    },
    range_calendar::{RangeCalendarContext, RangeCalendarRoot, RangeCalendarRuntime},
    select::{
        SelectContent, SelectContext, SelectItem, SelectPortal, SelectRoot, SelectRuntime,
        SelectTrigger,
    },
};

fn clear_active_overlay_stack() {
    for layer_id in active_overlay_layers_snapshot() {
        unregister_active_overlay_layer(&layer_id);
    }
}

#[test]
fn overlay_lifo_stack_tracks_topmost_layer_and_repromotion() {
    clear_active_overlay_stack();

    assert!(active_overlay_layers_snapshot().is_empty());
    assert!(is_topmost_overlay_layer("any-layer"));

    register_active_overlay_layer("popover-content");
    assert_eq!(
        active_overlay_layers_snapshot(),
        vec!["popover-content".to_string()]
    );
    assert!(is_topmost_overlay_layer("popover-content"));
    assert!(!is_topmost_overlay_layer("select-content"));

    register_active_overlay_layer("select-content");
    assert_eq!(
        active_overlay_layers_snapshot(),
        vec!["popover-content".to_string(), "select-content".to_string()]
    );
    assert!(!is_topmost_overlay_layer("popover-content"));
    assert!(is_topmost_overlay_layer("select-content"));

    // Re-registering an existing layer moves it to the top without duplicating it.
    register_active_overlay_layer("popover-content");
    assert_eq!(
        active_overlay_layers_snapshot(),
        vec!["select-content".to_string(), "popover-content".to_string()]
    );
    assert!(is_topmost_overlay_layer("popover-content"));
    assert!(!is_topmost_overlay_layer("select-content"));

    unregister_active_overlay_layer("popover-content");
    assert_eq!(
        active_overlay_layers_snapshot(),
        vec!["select-content".to_string()]
    );
    assert!(is_topmost_overlay_layer("select-content"));

    unregister_active_overlay_layer("select-content");
    assert!(active_overlay_layers_snapshot().is_empty());
    assert!(is_topmost_overlay_layer("popover-content"));
}

#[derive(Clone, Default, PartialEq)]
struct PopoverIdProbeResult {
    root_id: String,
    trigger_id: String,
    content_id: String,
}

#[component]
fn PopoverIdProbe(out: Rc<RefCell<PopoverIdProbeResult>>) -> Element {
    let ctx = use_context::<PopoverContext>();
    let rels = ctx.relationships;
    *out.borrow_mut() = PopoverIdProbeResult {
        root_id: rels.root_id().to_string(),
        trigger_id: rels.trigger_id().to_string(),
        content_id: rels.content_id().to_string(),
    };
    rsx! {}
}

#[component]
fn TwoAutoIdPopoversHarness(
    first: Rc<RefCell<PopoverIdProbeResult>>,
    second: Rc<RefCell<PopoverIdProbeResult>>,
    explicit: Rc<RefCell<PopoverIdProbeResult>>,
) -> Element {
    rsx! {
        PopoverRoot {
            PopoverTrigger { "First" }
            PopoverIdProbe { out: first }
        }
        PopoverRoot {
            PopoverTrigger { "Second" }
            PopoverIdProbe { out: second }
        }
        PopoverRoot {
            id: "custom-picker".to_string(),
            PopoverTrigger { "Explicit" }
            PopoverIdProbe { out: explicit }
        }
    }
}

#[test]
fn popover_root_generates_unique_stable_ids_when_id_is_omitted() {
    clear_active_overlay_stack();
    let first = Rc::new(RefCell::new(PopoverIdProbeResult::default()));
    let second = Rc::new(RefCell::new(PopoverIdProbeResult::default()));
    let explicit = Rc::new(RefCell::new(PopoverIdProbeResult::default()));

    let mut dom = VirtualDom::new_with_props(
        TwoAutoIdPopoversHarness,
        TwoAutoIdPopoversHarnessProps {
            first: first.clone(),
            second: second.clone(),
            explicit: explicit.clone(),
        },
    );
    dom.rebuild_in_place();

    let first_ids = first.borrow().clone();
    let second_ids = second.borrow().clone();
    let explicit_ids = explicit.borrow().clone();

    assert_ne!(first_ids.root_id, second_ids.root_id);
    assert_ne!(first_ids.trigger_id, second_ids.trigger_id);
    assert_ne!(first_ids.content_id, second_ids.content_id);
    assert!(!first_ids.root_id.contains("popover-popover"));
    assert!(!second_ids.root_id.contains("popover-popover"));
    let expected_explicit =
        monoxus::popover::PopoverRelationships::new(ScopeHandle::root("popover").child("custom-picker"));
    assert_eq!(explicit_ids.root_id, expected_explicit.root_id());
    assert_eq!(explicit_ids.content_id, expected_explicit.content_id());

    drop(dom);
    clear_active_overlay_stack();
}

#[test]
fn popover_open_focus_policy_selector_enables_autofocus_and_preserves_selector() {
    let scope = ScopeHandle::root("popover").child("picker-focus");
    let selector = "[data-calendar-day][data-focused]".to_string();
    let popover = Popover::new(scope, true)
        .with_open_focus_policy(PopoverOpenFocusPolicy::Selector(selector.clone()));

    assert_eq!(
        popover.lifecycle().open_focus_policy(),
        &PopoverOpenFocusPolicy::Selector(selector)
    );
    assert!(popover.lifecycle().focus_scope().autofocus_enabled());
    assert_eq!(popover.lifecycle().focus_scope().autofocus_target(), None);
}

struct RuntimeSlot<T>(Rc<RefCell<Option<T>>>);

impl<T> Clone for RuntimeSlot<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for RuntimeSlot<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> RuntimeSlot<T> {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    fn set(&self, value: T) {
        *self.0.borrow_mut() = Some(value);
    }

    fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.0.borrow().clone()
    }
}

#[component]
fn CapturePopoverRuntime(slot: RuntimeSlot<PopoverRuntime>) -> Element {
    let ctx = use_context::<PopoverContext>();
    slot.set(ctx.runtime.read().clone());
    rsx! {}
}

#[component]
fn CaptureSelectRuntime(slot: RuntimeSlot<SelectRuntime>) -> Element {
    let ctx = use_context::<SelectContext>();
    slot.set(ctx.runtime.clone());
    rsx! {}
}

#[component]
fn PopoverWithNestedSelectHarness(
    popover_rt: RuntimeSlot<PopoverRuntime>,
    select_rt: RuntimeSlot<SelectRuntime>,
) -> Element {
    rsx! {
        PopoverRoot {
            id: "nested-select-popover".to_string(),
            default_open: true,
            PopoverTrigger { "Open Picker" }
            PopoverPortal {
                PopoverContent {
                    SelectRoot {
                        id: "month-select".to_string(),
                        open: true,
                        SelectTrigger { "Month" }
                        SelectPortal {
                            SelectContent {
                                SelectItem { value: "jan".to_string(), "January" }
                                SelectItem { value: "feb".to_string(), "February" }
                            }
                        }
                        CaptureSelectRuntime { slot: select_rt }
                    }
                }
            }
            CapturePopoverRuntime { slot: popover_rt }
        }
    }
}

#[test]
fn nested_select_content_registers_branches_and_lifo_layer_inside_popover() {
    clear_active_overlay_stack();
    let popover_rt_slot = RuntimeSlot::new();
    let select_rt_slot = RuntimeSlot::new();

    let mut dom = VirtualDom::new_with_props(
        PopoverWithNestedSelectHarness,
        PopoverWithNestedSelectHarnessProps {
            popover_rt: popover_rt_slot.clone(),
            select_rt: select_rt_slot.clone(),
        },
    );
    dom.rebuild_in_place();

    let popover_rt = popover_rt_slot.get().expect("popover runtime captured");
    let select_rt = select_rt_slot.get().expect("select runtime captured");

    let select_content_id = select_rt.relationships().content_id().to_string();
    let select_wrapper_id = format!("{select_content_id}-wrapper");
    let select_portal_id = format!("{select_content_id}-portal-root");
    let popover_content_id = popover_rt.relationships().content_id().to_string();

    dom.in_scope(dioxus::core::ScopeId::ROOT, || {
        let branches = popover_rt.branches();
        assert!(
            branches.contains(&select_content_id),
            "expected select content_id in popover branches: {branches:?}"
        );
        assert!(
            branches.contains(&select_wrapper_id),
            "expected select wrapper_id in popover branches: {branches:?}"
        );
        assert!(
            branches.contains(&select_portal_id),
            "expected select portal_id in popover branches: {branches:?}"
        );

        // Select opened inside Popover sits on top of the LIFO overlay stack.
        assert!(is_topmost_overlay_layer(&select_content_id));
        assert!(!is_topmost_overlay_layer(&popover_content_id));

        // Escape on the parent Popover while Select is topmost must be ignored.
        assert!(!popover_rt.dismiss_escape());
        assert!(popover_rt.is_open());

        // Closing Select pops it from the LIFO stack so Popover becomes topmost.
        select_rt.close_dropdown();
    });
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    dom.in_scope(dioxus::core::ScopeId::ROOT, || {
        let branches = popover_rt.branches();
        assert!(!branches.contains(&select_content_id));
        assert!(!branches.contains(&select_wrapper_id));
        assert!(!branches.contains(&select_portal_id));
        assert!(is_topmost_overlay_layer(&popover_content_id));
    });

    drop(dom);
    assert!(active_overlay_layers_snapshot().is_empty());
}

#[component]
fn CaptureRangeCalendarRuntime(slot: RuntimeSlot<RangeCalendarRuntime>) -> Element {
    let ctx = use_context::<RangeCalendarContext>();
    slot.set(ctx.runtime.clone());
    rsx! {}
}

#[component]
fn PopoverWithRangeCalendarHarness(
    popover_rt: RuntimeSlot<PopoverRuntime>,
    range_rt: RuntimeSlot<RangeCalendarRuntime>,
    open_out: Rc<RefCell<bool>>,
) -> Element {
    let mut open_sig = use_signal(|| true);
    let open_out_cb = open_out.clone();
    let partial_start = CalendarDate::new(2026, 4, 10).unwrap();

    rsx! {
        PopoverRoot {
            id: "range-picker-popover".to_string(),
            open: open_sig,
            open_focus: PopoverOpenFocusPolicy::Selector("[data-range-calendar-day][data-focused]".to_string()),
            on_open_change: move |next| {
                open_sig.set(next);
                *open_out_cb.borrow_mut() = next;
            },
            PopoverTrigger { "Open Range Picker" }
            PopoverPortal {
                PopoverContent {
                    RangeCalendarRoot {
                        id: "picker-range-cal".to_string(),
                        default_value: DateRange {
                            start: Some(partial_start),
                            end: None,
                        },
                        CaptureRangeCalendarRuntime { slot: range_rt }
                    }
                }
            }
            CapturePopoverRuntime { slot: popover_rt }
        }
    }
}

#[test]
fn range_calendar_partial_selection_intercepts_first_escape_before_popover_dismisses() {
    clear_active_overlay_stack();
    let popover_rt_slot = RuntimeSlot::new();
    let range_rt_slot = RuntimeSlot::new();
    let open_out = Rc::new(RefCell::new(true));

    let mut dom = VirtualDom::new_with_props(
        PopoverWithRangeCalendarHarness,
        PopoverWithRangeCalendarHarnessProps {
            popover_rt: popover_rt_slot.clone(),
            range_rt: range_rt_slot.clone(),
            open_out: open_out.clone(),
        },
    );
    dom.rebuild_in_place();

    let popover_rt = popover_rt_slot.get().expect("popover runtime captured");
    let range_rt = range_rt_slot.get().expect("range calendar runtime captured");

    dom.in_scope(dioxus::core::ScopeId::ROOT, || {
        assert!(range_rt.start_value.read().is_some());
        assert!(range_rt.end_value.read().is_none());

        // First Escape clears partial range via OverlayEscapeGuardContext and keeps Popover open.
        assert!(!popover_rt.dismiss_escape());
        assert!(range_rt.start_value.read().is_none());
        assert!(range_rt.end_value.read().is_none());
        assert!(*open_out.borrow());

        // Second Escape finds no active partial range and closes Popover.
        assert!(popover_rt.dismiss_escape());
        assert!(!*open_out.borrow());
    });

    drop(dom);
    clear_active_overlay_stack();
}

#[test]
fn select_content_unmount_while_open_does_not_leak_active_overlay_layers() {
    clear_active_overlay_stack();
    let popover_rt_slot = RuntimeSlot::new();
    let select_rt_slot = RuntimeSlot::new();

    let mut dom = VirtualDom::new_with_props(
        PopoverWithNestedSelectHarness,
        PopoverWithNestedSelectHarnessProps {
            popover_rt: popover_rt_slot.clone(),
            select_rt: select_rt_slot.clone(),
        },
    );
    dom.rebuild_in_place();

    let select_rt = select_rt_slot.get().expect("select runtime captured");
    let select_content_id = select_rt.relationships().content_id().to_string();
    assert!(is_topmost_overlay_layer(&select_content_id));

    // Drop the entire VDOM while BOTH Popover and nested Select are still open (DEF-5C-05).
    drop(dom);
    assert!(
        active_overlay_layers_snapshot().is_empty(),
        "unmounting open SelectContent/PopoverContent must clean up ACTIVE_OVERLAY_LAYERS, left: {:?}",
        active_overlay_layers_snapshot()
    );
}

#[cfg(feature = "native")]
#[test]
fn native_floating_overlay_clamps_tall_flipped_content_inside_800x600_viewport() {
    use blitz_dom::{Attribute, BaseDocument, DocumentConfig, LocalName, QualName, ns};
    use monoxus::foundation::overlay::{
        OverlayAnchor, PlacementAlign, PlacementSide, VirtualOverlayPositionConfig,
        compute_virtual_overlay_position,
    };
    use oxidase::dom::Document;

    let qname = |tag: &'static str| QualName::new(None, ns!(html), LocalName::from(tag));
    let attr = |name: &'static str, val: &str| Attribute {
        name: QualName::new(None, ns!(), LocalName::from(name)),
        value: val.to_string(),
    };

    let mut base_doc_raw = BaseDocument::new(DocumentConfig::default());
    {
        let mut vp = base_doc_raw.viewport_mut();
        vp.window_size = (800, 600);
        vp.hidpi_scale = 1.0;
    }
    let base_doc = Rc::new(RefCell::new(base_doc_raw));
    let doc = Document::from_base(base_doc.clone());

    {
        let mut base = base_doc.borrow_mut();
        let root_id = base.root_node().id;
        let mut mutator = base.mutate();
        let html = mutator.create_element(qname("html"), vec![]);
        let body = mutator.create_element(
            qname("body"),
            vec![attr("style", "margin: 0; padding: 0;")],
        );
        let content = mutator.create_element(
            qname("div"),
            vec![
                attr("id", "tall-picker-content"),
                attr(
                    "style",
                    "position: fixed; width: 280px; height: 350px; display: block;",
                ),
            ],
        );
        mutator.append_children(body, &[content]);
        mutator.append_children(html, &[body]);
        mutator.append_children(root_id, &[html]);
    }
    doc.resolve(0.0);

    Document::with_current(doc, || {
        // Trigger at y = 280..318 on an 800x600 viewport with a 350px-tall popover (ENG D-2 / DEF-5C-04).
        let placement = compute_virtual_overlay_position(&VirtualOverlayPositionConfig {
            anchor: OverlayAnchor::VirtualPoint {
                x: 100.0,
                y: 280.0,
                width: 180.0,
                height: 38.0,
                context_element_id: None,
            },
            content_id: "tall-picker-content",
            arrow_id: None,
            boundary_id: None,
            side: PlacementSide::Bottom,
            align: PlacementAlign::Start,
            side_offset: 4.0,
            align_offset: 0.0,
            avoid_collisions: true,
            collision_padding: 8.0,
            arrow_padding: 0.0,
            hide_when_detached: false,
            sticky: false,
            namespace: Some("popover"),
        })
        .expect("floating placement must succeed");

        let geom = placement.geometry();
        assert!(
            geom.y() >= 8.0,
            "flipped tall popover must clamp y >= 8.0, got {}",
            geom.y()
        );
        assert!(
            geom.y() + geom.content_height() <= 592.0,
            "clamped popover bottom {} must stay within 592.0",
            geom.y() + geom.content_height()
        );
        assert!(
            geom.available_height() <= 584.0 && geom.available_height() < 600.0,
            "available_height must reflect remaining side space (<= 584.0), got {}",
            geom.available_height()
        );
    });
}

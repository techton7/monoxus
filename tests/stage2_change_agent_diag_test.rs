//! Stage 2 Failure-Mode Integration Tests — Part 1 (Ledger Section 4 Items 1, 2, and 3):
//! 1. `ChangeEvent<T>`, `TransitionGate` re-entry, bitwise NaN equality, and overlay `allow_propagation()`.
//! 2. Agent runtime, `<AgentBoundary>`, 33-primitive coverage (plus 10 non-registered display primitives), and WebMCP.
//! 3. Dev-only diagnostics (`ControlledRefusal`, `ControlledAndDefaultProvided`, `DetachedHandleAction`).

#![allow(non_snake_case)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::Duration;

use dioxus::prelude::*;
use monoxus::accessible_icon::AccessibleIcon;
use monoxus::accordion::{
    AccordionItem, AccordionRoot, AccordionTrigger,
};
use monoxus::agent::{
    ActionCall, AgentBoundary, AgentBridgeHandle, AgentError, AgentHints, AgentProvider,
    AgentReply, AgentRequest, ApprovalDecision, REDACTED_PLACEHOLDER, SnapshotValue, StepOutcome,
    TargetQuery, TurnState, WEBMCP_TOOL_NAMES, agent_tool_descriptors, use_agent_bridge,
    webmcp_tool_descriptors,
};
use monoxus::alert_dialog::{AlertDialogAction, AlertDialogCancel, AlertDialogRoot};
use monoxus::aspect_ratio::AspectRatio;
use monoxus::avatar::AvatarRoot;
use monoxus::button::Button;
use monoxus::calendar::CalendarRoot;
use monoxus::checkbox::{CheckboxGroup, CheckboxRoot};
use monoxus::collapsible::CollapsibleRoot;
use monoxus::combobox::ComboboxRoot;
use monoxus::command::CommandRoot;
use monoxus::context_menu::ContextMenuRoot;
use monoxus::date_field::DateFieldRoot;
use monoxus::date_picker::DatePickerRoot;
use monoxus::date_range_field::DateRangeFieldRoot;
use monoxus::date_range_picker::DateRangePickerRoot;
use monoxus::dialog::{DialogContent, DialogPortal, DialogRoot};
use monoxus::dropdown_menu::DropdownMenuRoot;
use monoxus::field::FieldRoot;
#[cfg(debug_assertions)]
use monoxus::foundation::diagnostics::DiagnosticKind;
use monoxus::foundation::diagnostics::{
    capture_diagnostics, flush_pending_controlled_checks, with_controlled_refusal_grace,
};
use monoxus::foundation::overlay::{
    DialogHandle, DismissCallbacks, DismissConfig, DismissEvent, DismissLayerHandle,
    SyntheticEscapeKey, active_overlay_layers_snapshot, clear_outside_press_propagating,
    is_overlay_escape_consumed, is_topmost_overlay_layer_for_outside_press,
    mark_outside_press_propagating, unregister_active_overlay_layer,
    use_dismissable_layer_with_details,
};
use monoxus::foundation::state::{
    ChangeOutcome, ControllableConfig, ControllableSignal, sync_f64_signal,
    use_controllable_signal_config,
};
use monoxus::hover_card::HoverCardRoot;
use monoxus::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};
use monoxus::label::Label;
use monoxus::menubar::MenubarRoot;
use monoxus::meter::MeterRoot;
use monoxus::navigation_menu::{
    NavigationMenuItem, NavigationMenuList, NavigationMenuRoot, NavigationMenuTrigger,
};
use monoxus::pagination::PaginationRoot;
use monoxus::pin_input::PinInputRoot;
use monoxus::popover::PopoverRoot;
use monoxus::progress::ProgressRoot;
use monoxus::radio_group::RadioGroupRoot;
use monoxus::range_calendar::RangeCalendarRoot;
use monoxus::rating_group::RatingGroupRoot;
use monoxus::scroll_area::ScrollAreaRoot;
use monoxus::select::{SelectItem, SelectItemData, SelectRoot};
use monoxus::separator::Separator;
use monoxus::slider::{SliderRoot, SliderValue};
use monoxus::switch::SwitchRoot;
use monoxus::tabs::{TabsContext, TabsList, TabsRoot, TabsTrigger};
use monoxus::time_field::TimeFieldRoot;
use monoxus::time_range_field::TimeRangeFieldRoot;
use monoxus::toast::{ToastId, ToastRoot};
use monoxus::toggle::Toggle;
use monoxus::toggle_group::ToggleGroupRoot;
use monoxus::toolbar::ToolbarRoot;
use monoxus::tooltip::{TooltipProvider, TooltipRoot};
use monoxus::visually_hidden::VisuallyHidden;

use crate::common::vdom::{flush_vdom, mount_vdom, mount_with_props};

fn clear_active_overlay_stack() {
    clear_outside_press_propagating();
    for layer_id in active_overlay_layers_snapshot() {
        unregister_active_overlay_layer(&layer_id);
    }
}

fn noop_waker() -> Waker {
    unsafe fn clone_fn(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    unsafe fn wake_fn(_: *const ()) {}
    unsafe fn wake_by_ref_fn(_: *const ()) {}
    unsafe fn drop_fn(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_fn, wake_fn, wake_by_ref_fn, drop_fn);
    // SAFETY: No-op vtable with null data pointer; all functions are safe no-ops.
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

fn args_one(key: &str, value: SnapshotValue) -> BTreeMap<String, SnapshotValue> {
    let mut map = BTreeMap::new();
    map.insert(key.to_owned(), value);
    map
}

fn label_hint(label: &str) -> AgentHints {
    AgentHints {
        label: Some(label.to_owned()),
        ..Default::default()
    }
}

// ============================================================================
// 1. ChangeEvent<T>, TransitionGate Re-Entry, NaN Equality & allow_propagation
// ============================================================================

#[derive(Clone)]
struct CancelAuditCounters {
    bridge: Rc<RefCell<Option<AgentBridgeHandle>>>,
    accordion_legacy: Rc<Cell<usize>>,
    slider_legacy: Rc<Cell<usize>>,
    slider_commit: Rc<Cell<usize>>,
    select_open_legacy: Rc<Cell<usize>>,
    select_val_legacy: Rc<Cell<usize>>,
    tabs_legacy: Rc<Cell<usize>>,
    tabs_stop: Rc<RefCell<String>>,
    checkbox_group_legacy: Rc<Cell<usize>>,
    dialog_legacy: Rc<Cell<usize>>,
}

impl PartialEq for CancelAuditCounters {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.bridge, &other.bridge)
            && Rc::ptr_eq(&self.accordion_legacy, &other.accordion_legacy)
            && Rc::ptr_eq(&self.slider_legacy, &other.slider_legacy)
            && Rc::ptr_eq(&self.slider_commit, &other.slider_commit)
            && Rc::ptr_eq(&self.select_open_legacy, &other.select_open_legacy)
            && Rc::ptr_eq(&self.select_val_legacy, &other.select_val_legacy)
            && Rc::ptr_eq(&self.tabs_legacy, &other.tabs_legacy)
            && Rc::ptr_eq(&self.tabs_stop, &other.tabs_stop)
            && Rc::ptr_eq(&self.checkbox_group_legacy, &other.checkbox_group_legacy)
            && Rc::ptr_eq(&self.dialog_legacy, &other.dialog_legacy)
    }
}

impl CancelAuditCounters {
    fn new() -> Self {
        Self {
            bridge: Rc::new(RefCell::new(None)),
            accordion_legacy: Rc::new(Cell::new(0)),
            slider_legacy: Rc::new(Cell::new(0)),
            slider_commit: Rc::new(Cell::new(0)),
            select_open_legacy: Rc::new(Cell::new(0)),
            select_val_legacy: Rc::new(Cell::new(0)),
            tabs_legacy: Rc::new(Cell::new(0)),
            tabs_stop: Rc::new(RefCell::new(String::new())),
            checkbox_group_legacy: Rc::new(Cell::new(0)),
            dialog_legacy: Rc::new(Cell::new(0)),
        }
    }
}

#[component]
fn TabsStopProbe(slot: Rc<RefCell<String>>) -> Element {
    let ctx = use_context::<TabsContext>();
    *slot.borrow_mut() = ctx.runtime.current_tab_stop();
    rsx! {}
}

#[component]
fn SixPrimitiveCancelHarness(
    controlled: bool,
    cancel_all: bool,
    counters: CancelAuditCounters,
) -> Element {
    let bridge_slot = Rc::clone(&counters.bridge);
    let acc_leg = Rc::clone(&counters.accordion_legacy);
    let sld_leg = Rc::clone(&counters.slider_legacy);
    let sld_com = Rc::clone(&counters.slider_commit);
    let sel_open_leg = Rc::clone(&counters.select_open_legacy);
    let sel_val_leg = Rc::clone(&counters.select_val_legacy);
    let tabs_leg = Rc::clone(&counters.tabs_legacy);
    let tabs_stop = Rc::clone(&counters.tabs_stop);
    let cbg_leg = Rc::clone(&counters.checkbox_group_legacy);
    let dlg_leg = Rc::clone(&counters.dialog_legacy);

    rsx! {
        AgentProvider {
            on_ready: move |h| {
                *bridge_slot.borrow_mut() = Some(h);
            },

            AccordionRoot {
                value: if controlled { Some(String::from("a")) } else { None },
                default_value: if controlled { None } else { Some(String::from("a")) },
                agent: Some(label_hint("cancel-accordion")),
                on_value_change: move |_| acc_leg.set(acc_leg.get() + 1),
                on_value_change_details: move |ev: ChangeEvent<String>| {
                    if cancel_all {
                        ev.cancel();
                    }
                },
                AccordionItem {
                    value: "a",
                    AccordionTrigger { "Item A" }
                }
                AccordionItem {
                    value: "b",
                    AccordionTrigger { "Item B" }
                }
            }

            SliderRoot {
                value: if controlled { Some(SliderValue::Single(20.0)) } else { None },
                default_value: if controlled { None } else { Some(SliderValue::Single(20.0)) },
                min: 0.0,
                max: 100.0,
                step: 1.0,
                agent: Some(label_hint("cancel-slider")),
                on_value_change: move |_| sld_leg.set(sld_leg.get() + 1),
                on_value_change_details: move |ev: ChangeEvent<SliderValue>| {
                    if cancel_all {
                        ev.cancel();
                    }
                },
                on_value_commit: move |_| sld_com.set(sld_com.get() + 1),
                div {}
            }

            SelectRoot {
                open: if controlled { Some(false) } else { None },
                value: if controlled { Some(String::from("opt1")) } else { None },
                default_value: if controlled { None } else { Some(String::from("opt1")) },
                items: Some(vec![
                    SelectItemData::new("opt1", "Option 1", false),
                    SelectItemData::new("opt2", "Option 2", false),
                ]),
                agent: Some(label_hint("cancel-select")),
                on_open_change: move |_| sel_open_leg.set(sel_open_leg.get() + 1),
                on_open_change_details: move |ev: ChangeEvent<bool>| {
                    if cancel_all {
                        ev.cancel();
                    }
                },
                on_value_change: move |_| sel_val_leg.set(sel_val_leg.get() + 1),
                on_value_change_details: move |ev: ChangeEvent<Option<String>>| {
                    if cancel_all {
                        ev.cancel();
                    }
                },
                SelectItem { value: "opt1", "Option 1" }
                SelectItem { value: "opt2", "Option 2" }
            }

            TabsRoot {
                value: if controlled { Some(String::from("t1")) } else { None },
                default_value: if controlled { None } else { Some(String::from("t1")) },
                agent: Some(label_hint("cancel-tabs")),
                on_value_change: move |_| tabs_leg.set(tabs_leg.get() + 1),
                on_value_change_details: move |ev: ChangeEvent<String>| {
                    if cancel_all {
                        ev.cancel();
                    }
                },
                TabsList {
                    TabsTrigger { value: "t1", "Tab 1" }
                    TabsTrigger { value: "t2", "Tab 2" }
                }
                TabsStopProbe { slot: tabs_stop }
            }

            CheckboxGroup {
                values: if controlled { Some(vec![String::from("c1")]) } else { None },
                default_values: if controlled { Vec::new() } else { vec![String::from("c1")] },
                agent: Some(label_hint("cancel-checkbox-group")),
                on_values_change: move |_| cbg_leg.set(cbg_leg.get() + 1),
                on_values_change_details: move |ev: ChangeEvent<Vec<String>>| {
                    if cancel_all {
                        ev.cancel();
                    }
                },
                CheckboxRoot { value: "c1", "C1" }
                CheckboxRoot { value: "c2", "C2" }
            }

            DialogRoot {
                open: if controlled { Some(false) } else { None },
                agent: Some(label_hint("cancel-dialog")),
                on_open_change: move |_| dlg_leg.set(dlg_leg.get() + 1),
                on_open_change_details: move |ev: ChangeEvent<bool>| {
                    if cancel_all {
                        ev.cancel();
                    }
                },
                div {}
            }
        }
    }
}

#[test]
fn change_event_cancel_uncontrolled_and_controlled_across_six_primitives() {
    for controlled in [false, true] {
        let counters = CancelAuditCounters::new();
        let mut dom = mount_with_props(
            SixPrimitiveCancelHarness,
            SixPrimitiveCancelHarnessProps {
                controlled,
                cancel_all: true,
                counters: counters.clone(),
            },
        );
        let bridge = counters.bridge.borrow().clone().expect("bridge ready");

        // 1. Accordion: cancel toggle -> state unchanged, legacy on_value_change skipped.
        let r = bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("cancel-accordion"),
            call: ActionCall::with_args("toggle", args_one("value", SnapshotValue::text("b"))),
        });
        assert!(matches!(r, AgentReply::Step(StepOutcome::Canceled { .. })));
        flush_vdom(&mut dom);
        assert_eq!(counters.accordion_legacy.get(), 0);
        let acc = match bridge.handle_request(AgentRequest::Query {
            target: TargetQuery::parse("cancel-accordion"),
        }) {
            AgentReply::Component(c) => c,
            other => panic!("expected Component, got {other:?}"),
        };
        assert_eq!(
            acc.snapshot.fields.get("value"),
            Some(&SnapshotValue::text("a"))
        );

        // 2. Slider: cancel set -> state unchanged, legacy on_value_change & on_value_commit skipped.
        let r = bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("cancel-slider"),
            call: ActionCall::with_args("set", args_one("value", SnapshotValue::Float(75.0))),
        });
        assert!(matches!(r, AgentReply::Step(StepOutcome::Canceled { .. })));
        flush_vdom(&mut dom);
        assert_eq!(counters.slider_legacy.get(), 0);
        assert_eq!(counters.slider_commit.get(), 0);
        let sld = match bridge.handle_request(AgentRequest::Query {
            target: TargetQuery::parse("cancel-slider"),
        }) {
            AgentReply::Component(c) => c,
            other => panic!("expected Component, got {other:?}"),
        };
        assert_eq!(
            sld.snapshot.fields.get("value"),
            Some(&SnapshotValue::Float(20.0))
        );

        // 3. Select: cancel open AND cancel select -> open/value unchanged, legacy callbacks skipped.
        let r_open = bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("cancel-select"),
            call: ActionCall::new("open"),
        });
        assert!(matches!(
            r_open,
            AgentReply::Step(StepOutcome::Canceled { .. })
        ));
        let r_sel = bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("cancel-select"),
            call: ActionCall::with_args("set", args_one("value", SnapshotValue::text("opt2"))),
        });
        assert!(matches!(
            r_sel,
            AgentReply::Step(StepOutcome::Canceled { .. })
        ));
        flush_vdom(&mut dom);
        assert_eq!(counters.select_open_legacy.get(), 0);
        assert_eq!(counters.select_val_legacy.get(), 0);
        let sel = match bridge.handle_request(AgentRequest::Query {
            target: TargetQuery::parse("cancel-select"),
        }) {
            AgentReply::Component(c) => c,
            other => panic!("expected Component, got {other:?}"),
        };
        assert_eq!(
            sel.snapshot.fields.get("open"),
            Some(&SnapshotValue::Bool(false))
        );
        assert_eq!(
            sel.snapshot.fields.get("value"),
            Some(&SnapshotValue::text("opt1"))
        );

        // 4. Tabs: cancel select -> value & roving tab stop unchanged, legacy skipped.
        let r = bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("cancel-tabs"),
            call: ActionCall::with_args("select", args_one("value", SnapshotValue::text("t2"))),
        });
        assert!(matches!(r, AgentReply::Step(StepOutcome::Canceled { .. })));
        flush_vdom(&mut dom);
        assert_eq!(counters.tabs_legacy.get(), 0);
        assert_eq!(counters.tabs_stop.borrow().as_str(), "t1");
        let tabs = match bridge.handle_request(AgentRequest::Query {
            target: TargetQuery::parse("cancel-tabs"),
        }) {
            AgentReply::Component(c) => c,
            other => panic!("expected Component, got {other:?}"),
        };
        assert_eq!(
            tabs.snapshot.fields.get("value"),
            Some(&SnapshotValue::text("t1"))
        );

        // 5. CheckboxGroup: cancel toggle -> values unchanged, legacy skipped.
        let r = bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("cancel-checkbox-group"),
            call: ActionCall::with_args("toggle", args_one("value", SnapshotValue::text("c2"))),
        });
        assert!(matches!(r, AgentReply::Step(StepOutcome::Canceled { .. })));
        flush_vdom(&mut dom);
        assert_eq!(counters.checkbox_group_legacy.get(), 0);
        let cbg = match bridge.handle_request(AgentRequest::Query {
            target: TargetQuery::parse("cancel-checkbox-group"),
        }) {
            AgentReply::Component(c) => c,
            other => panic!("expected Component, got {other:?}"),
        };
        assert_eq!(
            cbg.snapshot.fields.get("values"),
            Some(&SnapshotValue::List(vec![SnapshotValue::text("c1")]))
        );

        // 6. Dialog: cancel open -> open unchanged, legacy skipped.
        let r = bridge.handle_request(AgentRequest::Act {
            target: TargetQuery::parse("cancel-dialog"),
            call: ActionCall::new("open"),
        });
        assert!(matches!(r, AgentReply::Step(StepOutcome::Canceled { .. })));
        flush_vdom(&mut dom);
        assert_eq!(counters.dialog_legacy.get(), 0);
        let dlg = match bridge.handle_request(AgentRequest::Query {
            target: TargetQuery::parse("cancel-dialog"),
        }) {
            AgentReply::Component(c) => c,
            other => panic!("expected Component, got {other:?}"),
        };
        assert_eq!(
            dlg.snapshot.fields.get("open"),
            Some(&SnapshotValue::Bool(false))
        );
    }

    // Positive control: uncontrolled without cancel commits & fires legacy + commit side effects.
    let counters = CancelAuditCounters::new();
    let mut dom = mount_with_props(
        SixPrimitiveCancelHarness,
        SixPrimitiveCancelHarnessProps {
            controlled: false,
            cancel_all: false,
            counters: counters.clone(),
        },
    );
    let bridge = counters.bridge.borrow().clone().expect("bridge ready");
    let r = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("cancel-slider"),
        call: ActionCall::with_args("set", args_one("value", SnapshotValue::Float(60.0))),
    });
    assert!(matches!(r, AgentReply::Step(StepOutcome::Applied { .. })));
    flush_vdom(&mut dom);
    assert_eq!(counters.slider_legacy.get(), 1);
    assert_eq!(counters.slider_commit.get(), 1);
}

#[test]
fn transition_gate_reentrant_clamp_back_supersedes_outer_transition() {
    let ctl_slot: Rc<RefCell<Option<ControllableSignal<i32>>>> = Rc::new(RefCell::new(None));
    let legacy_seen: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));

    #[component]
    fn ClampHarness(
        ctl_slot: Rc<RefCell<Option<ControllableSignal<i32>>>>,
        legacy_seen: Rc<RefCell<Vec<i32>>>,
    ) -> Element {
        let ctl_for_details: Rc<RefCell<Option<ControllableSignal<i32>>>> =
            use_hook(|| Rc::new(RefCell::new(None)));
        let ctl_for_cb = Rc::clone(&ctl_for_details);
        let seen_cb = Rc::clone(&legacy_seen);

        let ctl = use_controllable_signal_config(
            ControllableConfig::new(None, 10_i32)
                .with_on_change(Some(EventHandler::new(move |val: i32| {
                    seen_cb.borrow_mut().push(val);
                })))
                .with_on_change_details(Some(EventHandler::new(move |ev: ChangeEvent<i32>| {
                    if *ev.value() == 50
                        && let Some(inner_ctl) = *ctl_for_cb.borrow()
                    {
                        // Re-entrant clamp-back: 10 -> 50 -> 10 while outer 50 is in flight.
                        // `TransitionGate::begin_ask_by` bumps `ask_epoch` (superseding outer 50)
                        // and returns `Unchanged` because owned value is already 10.
                        let inner_outcome = inner_ctl.set_with_context(
                            10,
                            ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Programmatic),
                        );
                        assert_eq!(inner_outcome, ChangeOutcome::Unchanged);
                    }
                })))
                .with_component("ClampHarness")
                .with_prop("value"),
        );

        *ctl_for_details.borrow_mut() = Some(ctl);
        *ctl_slot.borrow_mut() = Some(ctl);
        let _ = ctl.get();
        rsx! {}
    }

    let mut dom = mount_with_props(
        ClampHarness,
        ClampHarnessProps {
            ctl_slot: Rc::clone(&ctl_slot),
            legacy_seen: Rc::clone(&legacy_seen),
        },
    );

    let ctl = ctl_slot.borrow().expect("controllable signal captured");
    assert_eq!(ctl.peek(), 10);

    let outcome = ctl.set_with_context(
        50,
        ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Programmatic),
    );
    flush_vdom(&mut dom);

    assert!(
        matches!(outcome, ChangeOutcome::Superseded(_)),
        "outer 50 proposal must be Superseded by re-entrant clamp-back to 10, got {outcome:?}"
    );
    assert_eq!(ctl.peek(), 10);
    assert!(
        !legacy_seen.borrow().contains(&50),
        "legacy on_change must never observe superseded value 50: {:?}",
        *legacy_seen.borrow()
    );
}

#[test]
fn slider_and_rating_group_bitwise_nan_equality_no_infinite_sync_loop() {
    let render_count = Rc::new(Cell::new(0_usize));
    let nan_sync_changed = Rc::new(Cell::new(true));

    #[component]
    fn NanHarness(render_count: Rc<Cell<usize>>, nan_sync_changed: Rc<Cell<bool>>) -> Element {
        render_count.set(render_count.get() + 1);
        let mut raw_sig = use_signal(|| f64::NAN);
        let changed = sync_f64_signal(&mut raw_sig, f64::NAN);
        nan_sync_changed.set(changed);

        rsx! {
            SliderRoot {
                value: Some(SliderValue::Single(f64::NAN)),
                min: 0.0,
                max: 100.0,
                div {}
            }
            RatingGroupRoot {
                value: Some(f64::NAN),
                max: 5,
            }
        }
    }

    let mut dom = mount_with_props(
        NanHarness,
        NanHarnessProps {
            render_count: Rc::clone(&render_count),
            nan_sync_changed: Rc::clone(&nan_sync_changed),
        },
    );
    flush_vdom(&mut dom);
    flush_vdom(&mut dom);

    assert!(!nan_sync_changed.get(), "sync_f64_signal must treat NaN == NaN bitwise");
    assert!(
        render_count.get() <= 3,
        "NaN controlled props must not cause infinite sync loops (renders={})",
        render_count.get()
    );
}

#[derive(Clone, Props)]
struct LayerPairProps {
    allow_propagation: bool,
    parent_closed: Rc<Cell<usize>>,
    child_closed: Rc<Cell<usize>>,
    parent_handle: Rc<RefCell<Option<DismissLayerHandle>>>,
    child_handle: Rc<RefCell<Option<DismissLayerHandle>>>,
}

impl PartialEq for LayerPairProps {
    fn eq(&self, other: &Self) -> bool {
        self.allow_propagation == other.allow_propagation
            && Rc::ptr_eq(&self.parent_closed, &other.parent_closed)
            && Rc::ptr_eq(&self.child_closed, &other.child_closed)
            && Rc::ptr_eq(&self.parent_handle, &other.parent_handle)
            && Rc::ptr_eq(&self.child_handle, &other.child_handle)
    }
}

#[component]
fn LayerPair(props: LayerPairProps) -> Element {
    let mut parent_open = use_signal(|| true);
    let mut child_open = use_signal(|| true);
    let p_closed = Rc::clone(&props.parent_closed);
    let c_closed = Rc::clone(&props.child_closed);
    let allow_prop = props.allow_propagation;

    let parent = use_dismissable_layer_with_details(
        parent_open(),
        DismissConfig::new("s25-parent-layer").with_modal(true),
        DismissCallbacks::default(),
        Some(EventHandler::new(
            move |(_evt, change): (DismissEvent, ChangeEvent<bool>)| {
                if !change.is_canceled() {
                    p_closed.set(p_closed.get() + 1);
                    parent_open.set(false);
                }
            },
        )),
    );
    let child = use_dismissable_layer_with_details(
        child_open(),
        DismissConfig::new("s25-child-layer"),
        DismissCallbacks::default(),
        Some(EventHandler::new(
            move |(_evt, change): (DismissEvent, ChangeEvent<bool>)| {
                if allow_prop {
                    change.allow_propagation();
                }
                if !change.is_canceled() {
                    c_closed.set(c_closed.get() + 1);
                    child_open.set(false);
                }
            },
        )),
    );
    *props.parent_handle.borrow_mut() = Some(parent);
    *props.child_handle.borrow_mut() = Some(child);
    rsx! {}
}

#[test]
fn nested_overlay_escape_without_allow_propagation_keeps_parent_open_headless() {
    clear_active_overlay_stack();
    let parent_closed = Rc::new(Cell::new(0));
    let child_closed = Rc::new(Cell::new(0));
    let parent_h = Rc::new(RefCell::new(None));
    let child_h = Rc::new(RefCell::new(None));
    let mut dom = mount_with_props(
        LayerPair,
        LayerPairProps {
            allow_propagation: false,
            parent_closed: Rc::clone(&parent_closed),
            child_closed: Rc::clone(&child_closed),
            parent_handle: Rc::clone(&parent_h),
            child_handle: Rc::clone(&child_h),
        },
    );
    let esc_evt: KeyboardEvent =
        dioxus::core::Event::new(Rc::new(KeyboardData::new(SyntheticEscapeKey)), true);
    let child = child_h.borrow().expect("child handle");
    let parent = parent_h.borrow().expect("parent handle");

    assert!(child.handle_keydown(&esc_evt));
    assert!(is_overlay_escape_consumed());
    flush_vdom(&mut dom);
    // Parent must NOT handle Escape on the same tick because child did not allow propagation.
    assert!(!parent.handle_keydown(&esc_evt));
    assert_eq!(child_closed.get(), 1);
    assert_eq!(parent_closed.get(), 0);
    drop(dom);
    clear_active_overlay_stack();
}

#[test]
fn nested_overlay_escape_and_outside_press_allow_propagation_headless() {
    // 1. Escape WITH allow_propagation(): child closes AND leaves Escape unconsumed so parent closes too.
    {
        clear_active_overlay_stack();
        let parent_closed = Rc::new(Cell::new(0));
        let child_closed = Rc::new(Cell::new(0));
        let parent_h = Rc::new(RefCell::new(None));
        let child_h = Rc::new(RefCell::new(None));
        let mut dom = mount_with_props(
            LayerPair,
            LayerPairProps {
                allow_propagation: true,
                parent_closed: Rc::clone(&parent_closed),
                child_closed: Rc::clone(&child_closed),
                parent_handle: Rc::clone(&parent_h),
                child_handle: Rc::clone(&child_h),
            },
        );
        let esc_evt: KeyboardEvent =
            dioxus::core::Event::new(Rc::new(KeyboardData::new(SyntheticEscapeKey)), true);
        let child = child_h.borrow().expect("child handle");
        let parent = parent_h.borrow().expect("parent handle");

        assert!(child.handle_keydown(&esc_evt));
        assert!(
            !is_overlay_escape_consumed(),
            "child calling allow_propagation() must not mark Escape consumed"
        );
        flush_vdom(&mut dom);
        assert!(parent.handle_keydown(&esc_evt));
        flush_vdom(&mut dom);
        assert_eq!(child_closed.get(), 1);
        assert_eq!(parent_closed.get(), 1);
        drop(dom);
        clear_active_overlay_stack();
    }

    // 3. Outside-press allow_propagation(): child marks propagating -> parent becomes topmost on same tick.
    {
        clear_active_overlay_stack();
        let parent_closed = Rc::new(Cell::new(0));
        let child_closed = Rc::new(Cell::new(0));
        let parent_h = Rc::new(RefCell::new(None));
        let child_h = Rc::new(RefCell::new(None));
        let dom = mount_with_props(
            LayerPair,
            LayerPairProps {
                allow_propagation: true,
                parent_closed,
                child_closed,
                parent_handle: Rc::clone(&parent_h),
                child_handle: Rc::clone(&child_h),
            },
        );
        let parent = parent_h.borrow().expect("parent handle");
        let child = child_h.borrow().expect("child handle");

        assert!(child.is_topmost_for_outside_press());
        assert!(!parent.is_topmost_for_outside_press());
        assert!(!is_topmost_overlay_layer_for_outside_press("s25-parent-layer"));

        mark_outside_press_propagating("s25-child-layer");
        assert!(parent.is_topmost_for_outside_press());
        assert!(is_topmost_overlay_layer_for_outside_press("s25-parent-layer"));

        clear_outside_press_propagating();
        assert!(!parent.is_topmost_for_outside_press());
        drop(dom);
        clear_active_overlay_stack();
    }
}

// ============================================================================
// 2. Agent Runtime, <AgentBoundary>, 33-Primitive Coverage & WebMCP
// ============================================================================

#[test]
fn agent_reentrant_callback_mounts_unmounts_siblings_and_reads_snapshot_without_panic() {
    let bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>> = Rc::new(RefCell::new(None));
    let snapshot_len_during_cb = Rc::new(Cell::new(0_usize));

    #[component]
    fn ReentrantApp(
        bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>>,
        snapshot_len_during_cb: Rc<Cell<usize>>,
    ) -> Element {
        let mut show_sibling = use_signal(|| false);
        let bridge_for_ready = Rc::clone(&bridge_slot);
        let bridge_for_cb = Rc::clone(&bridge_slot);
        let snap_len_cb = Rc::clone(&snapshot_len_during_cb);

        rsx! {
            AgentProvider {
                on_ready: move |h| {
                    *bridge_for_ready.borrow_mut() = Some(h);
                },
                SwitchRoot {
                    agent: Some(label_hint("reentrant-switch")),
                    on_checked_change: move |checked| {
                        // Re-entrantly read snapshot during Phase 2 dispatch (zero RefCell borrows held).
                        if let Some(b) = bridge_for_cb.borrow().as_ref() {
                            snap_len_cb.set(b.registry_snapshot().components.len());
                        }
                        show_sibling.set(checked);
                    },
                }
                if show_sibling() {
                    Button {
                        agent: Some(label_hint("dynamic-sibling")),
                        "Sibling"
                    }
                }
            }
        }
    }

    let mut dom = mount_with_props(
        ReentrantApp,
        ReentrantAppProps {
            bridge_slot: Rc::clone(&bridge_slot),
            snapshot_len_during_cb: Rc::clone(&snapshot_len_during_cb),
        },
    );
    let bridge = bridge_slot.borrow().clone().expect("bridge ready");
    assert_eq!(bridge.registry_snapshot().components.len(), 1);

    // Act -> mounts sibling via signal + reads snapshot re-entrantly without panicking.
    let reply = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("reentrant-switch"),
        call: ActionCall::new("toggle"),
    });
    assert!(matches!(reply, AgentReply::Step(StepOutcome::Applied { .. })));
    assert_eq!(snapshot_len_during_cb.get(), 1);
    assert_eq!(bridge.registry_snapshot().turn_state, TurnState::Idle);

    flush_vdom(&mut dom);
    assert_eq!(bridge.registry_snapshot().components.len(), 2);

    // Act again -> unmounts sibling via signal + reads snapshot re-entrantly.
    let reply2 = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("reentrant-switch"),
        call: ActionCall::new("toggle"),
    });
    assert!(matches!(reply2, AgentReply::Step(StepOutcome::Applied { .. })));
    assert_eq!(snapshot_len_during_cb.get(), 2);

    flush_vdom(&mut dom);
    assert_eq!(bridge.registry_snapshot().components.len(), 1);
}

#[test]
fn agent_unmount_during_executing_and_awaiting_approval_aborts_turn() {
    let bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>> = Rc::new(RefCell::new(None));
    let show_exec_sig: Rc<RefCell<Option<Signal<bool>>>> = Rc::new(RefCell::new(None));
    let show_approve_sig: Rc<RefCell<Option<Signal<bool>>>> = Rc::new(RefCell::new(None));

    #[component]
    fn UnmountTurnApp(
        bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>>,
        show_exec_sig: Rc<RefCell<Option<Signal<bool>>>>,
        show_approve_sig: Rc<RefCell<Option<Signal<bool>>>>,
    ) -> Element {
        let show_exec = use_signal(|| true);
        let show_approve = use_signal(|| true);
        *show_exec_sig.borrow_mut() = Some(show_exec);
        *show_approve_sig.borrow_mut() = Some(show_approve);

        let slot = Rc::clone(&bridge_slot);
        rsx! {
            AgentProvider {
                on_ready: move |h| {
                    *slot.borrow_mut() = Some(h);
                },
                if show_exec() {
                    Button {
                        agent: Some(label_hint("exec-target")),
                        "Exec"
                    }
                }
                if show_approve() {
                    Button {
                        agent: Some(AgentHints {
                            label: Some(String::from("approve-target")),
                            confirm: Some(String::from("Confirm destructive action?")),
                            ..Default::default()
                        }),
                        "Approve"
                    }
                }
            }
        }
    }

    let mut dom = mount_with_props(
        UnmountTurnApp,
        UnmountTurnAppProps {
            bridge_slot: Rc::clone(&bridge_slot),
            show_exec_sig: Rc::clone(&show_exec_sig),
            show_approve_sig: Rc::clone(&show_approve_sig),
        },
    );
    let bridge = bridge_slot.borrow().clone().expect("bridge ready");

    // Case 1: Unmount while TurnState::Executing (mid-settlement in handle_request_settled).
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    let mut fut = Box::pin(bridge.handle_request_settled(AgentRequest::Act {
        target: TargetQuery::parse("exec-target"),
        call: ActionCall::new("press"),
    }));
    // First poll runs Phase 1 + Phase 2 and suspends on next_frame().
    assert!(matches!(Pin::as_mut(&mut fut).poll(&mut cx), Poll::Pending));
    assert!(matches!(
        bridge.registry_snapshot().turn_state,
        TurnState::Executing { .. }
    ));

    // Unmount the executing target while still in TurnState::Executing.
    let mut exec_sig = show_exec_sig.borrow().expect("exec signal");
    exec_sig.set(false);
    flush_vdom(&mut dom);
    assert_eq!(bridge.registry_snapshot().turn_state, TurnState::Idle);

    // Tick frame so handle_request_settled resumes Phase 3 and returns AgentReply::Aborted.
    oxidase::frame::tick(Duration::from_millis(16));
    let exec_reply = match Pin::as_mut(&mut fut).poll(&mut cx) {
        Poll::Ready(reply) => reply,
        Poll::Pending => panic!("expected handle_request_settled to complete after frame tick"),
    };
    assert!(
        matches!(exec_reply, AgentReply::Aborted(_)),
        "unmounting during Executing must return AgentReply::Aborted, got {exec_reply:?}"
    );
    assert_eq!(bridge.registry_snapshot().turn_state, TurnState::Idle);

    // Case 2: Unmount while TurnState::AwaitingApproval.
    let approval_id = match bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("approve-target"),
        call: ActionCall::new("press"),
    }) {
        AgentReply::Step(StepOutcome::NeedsApproval { approval_id, .. }) => approval_id,
        other => panic!("expected NeedsApproval, got {other:?}"),
    };
    assert!(matches!(
        bridge.registry_snapshot().turn_state,
        TurnState::AwaitingApproval { .. }
    ));

    let mut approve_sig = show_approve_sig.borrow().expect("approve signal");
    approve_sig.set(false);
    flush_vdom(&mut dom);
    assert_eq!(bridge.registry_snapshot().turn_state, TurnState::Idle);

    let resumed = bridge.resume_approval(approval_id, ApprovalDecision::Allow);
    assert!(
        matches!(resumed, AgentReply::Aborted(_)),
        "resuming after AwaitingApproval unmount must return AgentReply::Aborted, got {resumed:?}"
    );
}

#[test]
fn agent_unchanged_rerender_preserves_meta_epoch_and_approval_fingerprint() {
    let bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>> = Rc::new(RefCell::new(None));
    let tick_slot: Rc<RefCell<Option<Signal<u64>>>> = Rc::new(RefCell::new(None));
    let pressed = Rc::new(Cell::new(0_usize));

    #[component]
    fn EpochApp(
        bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>>,
        tick_slot: Rc<RefCell<Option<Signal<u64>>>>,
        pressed: Rc<Cell<usize>>,
    ) -> Element {
        let tick = use_signal(|| 0_u64);
        *tick_slot.borrow_mut() = Some(tick);
        let slot = Rc::clone(&bridge_slot);
        let p = Rc::clone(&pressed);
        let current_tick = tick();

        rsx! {
            AgentProvider {
                on_ready: move |h| {
                    *slot.borrow_mut() = Some(h);
                },
                AgentBoundary {
                    section: "Checkout",
                    Button {
                        id: "pay-now-btn",
                        agent: Some(AgentHints {
                            label: Some(String::from("Pay now")),
                            confirm: Some(String::from("Charge card?")),
                            ..Default::default()
                        }),
                        on_press: move |_| p.set(p.get() + 1),
                        "Pay now ({current_tick})"
                    }
                }
            }
        }
    }

    let mut dom = mount_with_props(
        EpochApp,
        EpochAppProps {
            bridge_slot: Rc::clone(&bridge_slot),
            tick_slot: Rc::clone(&tick_slot),
            pressed: Rc::clone(&pressed),
        },
    );
    let bridge = bridge_slot.borrow().clone().expect("bridge ready");
    let initial_epoch = bridge.registry_snapshot().components[0].meta.meta_epoch;
    assert_eq!(initial_epoch, 0);

    let approval_id = match bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("Checkout > Pay now"),
        call: ActionCall::new("press"),
    }) {
        AgentReply::Step(StepOutcome::NeedsApproval { approval_id, .. }) => approval_id,
        other => panic!("expected NeedsApproval, got {other:?}"),
    };

    // Re-render multiple times with identical semantic metadata.
    let mut tick_sig = tick_slot.borrow().expect("tick signal");
    tick_sig.set(1);
    flush_vdom(&mut dom);
    tick_sig.set(2);
    flush_vdom(&mut dom);

    assert_eq!(
        bridge.registry_snapshot().components[0].meta.meta_epoch,
        initial_epoch,
        "unchanged semantic metadata must not increment meta_epoch"
    );

    let resumed = bridge.resume_approval(approval_id, ApprovalDecision::Allow);
    assert!(matches!(
        resumed,
        AgentReply::Step(StepOutcome::Applied { .. })
    ));
    assert_eq!(pressed.get(), 1);
}

#[test]
fn agent_boundary_nested_inheritance_propagates_monotonically() {
    let bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>> = Rc::new(RefCell::new(None));

    #[component]
    fn BoundaryApp(bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>>) -> Element {
        let slot = Rc::clone(&bridge_slot);
        rsx! {
            AgentProvider {
                on_ready: move |h| {
                    *slot.borrow_mut() = Some(h);
                },
                AgentBoundary {
                    section: "Account",
                    sections: vec![String::from("Security")],
                    secret: true,
                    confirm: "Confirm security change?",
                    // Child drops whitespace-only confirm, preserves parent confirm,
                    // monotonically preserves secret: true even when child sets secret: false.
                    AgentBoundary {
                        section: "Billing",
                        secret: false,
                        confirm: "   \t\n  ",
                        Button {
                            agent: Some(label_hint("billing-btn")),
                            "Update card"
                        }
                    }
                    // Child sets hidden: true -> hidden from snapshot and NotFound on Act.
                    AgentBoundary {
                        section: "HiddenZone",
                        hidden: true,
                        Button {
                            agent: Some(label_hint("hidden-btn")),
                            "Hidden"
                        }
                    }
                    // Child sets inert: true -> visible but BlockedInert on Act.
                    AgentBoundary {
                        section: "ReadOnlyZone",
                        inert: true,
                        Button {
                            agent: Some(label_hint("inert-btn")),
                            "Inert"
                        }
                    }
                }
            }
        }
    }

    let _dom = mount_with_props(
        BoundaryApp,
        BoundaryAppProps {
            bridge_slot: Rc::clone(&bridge_slot),
        },
    );
    let bridge = bridge_slot.borrow().clone().expect("bridge ready");
    let snap = bridge.registry_snapshot();

    // Hidden child is omitted from visible snapshot.
    assert!(
        snap.components
            .iter()
            .all(|c| c.meta.label != "hidden-btn"),
        "hidden boundary child must not appear in snapshot"
    );

    let billing = snap
        .components
        .iter()
        .find(|c| c.meta.label == "billing-btn")
        .expect("billing-btn registered");
    assert_eq!(
        billing.meta.sections,
        vec![
            String::from("Account"),
            String::from("Security"),
            String::from("Billing"),
        ]
    );
    assert!(billing.meta.secret, "secret must propagate monotonically");
    assert_eq!(
        billing.meta.confirm.as_deref(),
        Some("Confirm security change?"),
        "whitespace-only child confirm must preserve parent confirm"
    );

    // Hidden target returns NotFound.
    let hidden_res = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("hidden-btn"),
        call: ActionCall::new("press"),
    });
    assert_eq!(hidden_res, AgentReply::Error(AgentError::NotFound));

    // Inert target returns BlockedInert.
    let inert_res = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("inert-btn"),
        call: ActionCall::new("press"),
    });
    assert_eq!(inert_res, AgentReply::Error(AgentError::BlockedInert));
}

#[test]
fn agent_concrete_primitive_behaviors_pin_reveals_alert_button_slider_readonly() {
    let bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>> = Rc::new(RefCell::new(None));
    let btn_pressed = Rc::new(Cell::new(0_usize));
    let slider_committed = Rc::new(RefCell::new(Vec::<SliderValue>::new()));

    #[component]
    fn ConcreteSurfaceApp(
        bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>>,
        btn_pressed: Rc<Cell<usize>>,
        slider_committed: Rc<RefCell<Vec<SliderValue>>>,
    ) -> Element {
        let slot = Rc::clone(&bridge_slot);
        let bp = Rc::clone(&btn_pressed);
        let sc = Rc::clone(&slider_committed);

        rsx! {
            AgentProvider {
                on_ready: move |h| {
                    *slot.borrow_mut() = Some(h);
                },

                // PinInput defaults to secret: true
                PinInputRoot {
                    default_value: "1234",
                    max_length: 4,
                    agent: Some(label_hint("otp-pin")),
                }

                // Reveals primitives: Tabs, Accordion, Collapsible, Pagination, NavigationMenu
                TabsRoot {
                    default_value: "tab-a",
                    agent: Some(label_hint("demo-tabs")),
                    TabsList {
                        TabsTrigger { value: "tab-a", "A" }
                        TabsTrigger { value: "tab-b", "B" }
                    }
                }
                AccordionRoot {
                    default_value: "acc-a",
                    agent: Some(label_hint("demo-accordion")),
                    AccordionItem {
                        value: "acc-a",
                        AccordionTrigger { "A" }
                    }
                    AccordionItem {
                        value: "acc-b",
                        AccordionTrigger { "B" }
                    }
                }
                CollapsibleRoot {
                    default_open: false,
                    agent: Some(label_hint("demo-collapsible")),
                }
                PaginationRoot {
                    count: 30,
                    per_page: 10,
                    agent: Some(label_hint("demo-pagination")),
                }
                NavigationMenuRoot {
                    agent: Some(label_hint("demo-nav")),
                    NavigationMenuList {
                        NavigationMenuItem {
                            value: "docs",
                            NavigationMenuTrigger { "Docs" }
                        }
                    }
                }

                // AlertDialogAction carries confirm gate without blocking AlertDialogCancel
                AlertDialogRoot {
                    default_open: true,
                    agent: Some(label_hint("demo-alert")),
                    DialogPortal {
                        DialogContent {
                            AlertDialogAction {
                                agent: Some(AgentHints {
                                    label: Some(String::from("confirm-delete")),
                                    confirm: Some(String::from("Delete permanently?")),
                                    ..Default::default()
                                }),
                                "Delete"
                            }
                            AlertDialogCancel {
                                agent: Some(label_hint("cancel-delete")),
                                "Cancel"
                            }
                        }
                    }
                }

                // Button press invokes on_press in headless VirtualDom
                Button {
                    agent: Some(label_hint("headless-btn")),
                    on_press: move |_| bp.set(bp.get() + 1),
                    "Press me"
                }

                // Slider set fires on_value_commit
                SliderRoot {
                    default_value: Some(SliderValue::Single(10.0)),
                    min: 0.0,
                    max: 100.0,
                    step: 5.0,
                    agent: Some(label_hint("commit-slider")),
                    on_value_commit: move |v| sc.borrow_mut().push(v),
                    div {}
                }

                // Read-only controls omit mutating actions -> UnknownAction
                FieldRoot {
                    read_only: true,
                    SelectRoot {
                        default_value: "x",
                        agent: Some(label_hint("ro-select")),
                        SelectItem { value: "x", "X" }
                    }
                }
                CalendarRoot {
                    read_only: true,
                    agent: Some(label_hint("ro-calendar")),
                }
                RatingGroupRoot {
                    read_only: true,
                    agent: Some(label_hint("ro-rating")),
                }
            }
        }
    }

    let mut dom = mount_with_props(
        ConcreteSurfaceApp,
        ConcreteSurfaceAppProps {
            bridge_slot: Rc::clone(&bridge_slot),
            btn_pressed: Rc::clone(&btn_pressed),
            slider_committed: Rc::clone(&slider_committed),
        },
    );
    // One extra flush so roving items/triggers settle into reveals.
    flush_vdom(&mut dom);

    let bridge = bridge_slot.borrow().clone().expect("bridge ready");
    let snap = bridge.registry_snapshot();

    // 1. PinInput defaults to secret: true and is redacted in Snapshot, Query, and Act diff.
    let pin = snap
        .components
        .iter()
        .find(|c| c.meta.label == "otp-pin")
        .expect("otp-pin registered");
    assert!(pin.meta.secret, "PinInput must default secret: true");
    assert_eq!(
        pin.snapshot.fields.get("value"),
        Some(&SnapshotValue::Text(String::from(REDACTED_PLACEHOLDER)))
    );

    let pin_query = match bridge.handle_request(AgentRequest::Query {
        target: TargetQuery::parse("otp-pin"),
    }) {
        AgentReply::Component(c) => c,
        other => panic!("expected Component for otp-pin, got {other:?}"),
    };
    assert_eq!(
        pin_query.snapshot.fields.get("value"),
        Some(&SnapshotValue::Text(String::from(REDACTED_PLACEHOLDER)))
    );

    let pin_act = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("otp-pin"),
        call: ActionCall::with_args("set", args_one("value", SnapshotValue::text("9876"))),
    });
    match pin_act {
        AgentReply::Step(StepOutcome::Applied { diff, .. }) => {
            let serialized = oxidase::serde_json::to_string(&diff).expect("serialize diff");
            assert!(
                !serialized.contains("1234") && !serialized.contains("9876"),
                "secret PinInput values must never leak in Act diff: {serialized}"
            );
        }
        other => panic!("expected Applied for otp-pin set, got {other:?}"),
    }

    // 2. Tabs, Accordion, Collapsible, Pagination, NavigationMenu populate `reveals`.
    for label in [
        "demo-tabs",
        "demo-accordion",
        "demo-collapsible",
        "demo-pagination",
        "demo-nav",
    ] {
        let entry = snap
            .components
            .iter()
            .find(|c| c.meta.label == label)
            .unwrap_or_else(|| panic!("{label} registered"));
        assert!(
            !entry.meta.reveals.is_empty(),
            "{label} must populate non-empty reveals"
        );
    }

    // 3. AlertDialogAction carries confirm gate without blocking AlertDialogCancel.
    let action_approval_id = match bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("confirm-delete"),
        call: ActionCall::new("press"),
    }) {
        AgentReply::Step(StepOutcome::NeedsApproval { approval_id, .. }) => approval_id,
        other => panic!("expected NeedsApproval for AlertDialogAction, got {other:?}"),
    };
    let denied = bridge.resume_approval(action_approval_id, ApprovalDecision::Deny);
    assert!(matches!(denied, AgentReply::Step(StepOutcome::Denied { .. })));

    let cancel_res = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("cancel-delete"),
        call: ActionCall::new("press"),
    });
    assert!(
        matches!(cancel_res, AgentReply::Step(StepOutcome::Applied { .. })),
        "AlertDialogCancel must not be blocked by AlertDialogAction confirm gate: {cancel_res:?}"
    );

    // 4. Button `press` invokes `on_press` in headless VirtualDom.
    let btn_res = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("headless-btn"),
        call: ActionCall::new("press"),
    });
    assert!(matches!(
        btn_res,
        AgentReply::Step(StepOutcome::Applied { .. })
    ));
    assert_eq!(btn_pressed.get(), 1);

    // 5. Slider `set` fires `on_value_commit`.
    let slider_res = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("commit-slider"),
        call: ActionCall::with_args("set", args_one("value", SnapshotValue::Float(45.0))),
    });
    assert!(matches!(
        slider_res,
        AgentReply::Step(StepOutcome::Applied { .. })
    ));
    assert_eq!(*slider_committed.borrow(), vec![SliderValue::Single(45.0)]);

    // 6. Read-only controls omit mutating actions (`AgentError::UnknownAction`).
    let ro_select_res = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("ro-select"),
        call: ActionCall::with_args("set", args_one("value", SnapshotValue::text("y"))),
    });
    assert_eq!(ro_select_res, AgentReply::Error(AgentError::UnknownAction));

    let ro_cal_res = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("ro-calendar"),
        call: ActionCall::with_args("set", args_one("value", SnapshotValue::text("2026-05-01"))),
    });
    assert_eq!(ro_cal_res, AgentReply::Error(AgentError::UnknownAction));

    let ro_rating_res = bridge.handle_request(AgentRequest::Act {
        target: TargetQuery::parse("ro-rating"),
        call: ActionCall::with_args("set", args_one("value", SnapshotValue::Float(4.0))),
    });
    assert_eq!(ro_rating_res, AgentReply::Error(AgentError::UnknownAction));
}

#[test]
fn agent_33_interactive_primitives_register_and_10_display_primitives_register_zero() {
    // Part A: Mount all interactive primitives inside <AgentProvider> and verify registration.
    let interactive_bridge: Rc<RefCell<Option<AgentBridgeHandle>>> = Rc::new(RefCell::new(None));

    #[component]
    fn AllInteractiveApp(bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>>) -> Element {
        let slot = Rc::clone(&bridge_slot);
        rsx! {
            AgentProvider {
                on_ready: move |h| {
                    *slot.borrow_mut() = Some(h);
                },
                AccordionRoot { agent: Some(label_hint("p-accordion")), div {} }
                AlertDialogRoot { agent: Some(label_hint("p-alert-dialog")), div {} }
                Button { agent: Some(label_hint("p-button")), "Btn" }
                CalendarRoot { agent: Some(label_hint("p-calendar")) }
                CheckboxRoot { agent: Some(label_hint("p-checkbox")), "Chk" }
                CollapsibleRoot { agent: Some(label_hint("p-collapsible")) }
                ComboboxRoot { agent: Some(label_hint("p-combobox")), div {} }
                CommandRoot { agent: Some(label_hint("p-command")) }
                ContextMenuRoot { agent: Some(label_hint("p-context-menu")), div {} }
                DateFieldRoot { agent: Some(label_hint("p-date-field")) }
                DatePickerRoot { agent: Some(label_hint("p-date-picker")) }
                DateRangeFieldRoot { agent: Some(label_hint("p-date-range-field")) }
                DateRangePickerRoot { agent: Some(label_hint("p-date-range-picker")) }
                DialogRoot { agent: Some(label_hint("p-dialog")), div {} }
                DropdownMenuRoot { agent: Some(label_hint("p-dropdown-menu")), div {} }
                FieldRoot { invalid: true, agent: Some(label_hint("p-field")), div {} }
                HoverCardRoot { agent: Some(label_hint("p-hover-card")), div {} }
                MenubarRoot { agent: Some(label_hint("p-menubar")), div {} }
                NavigationMenuRoot { agent: Some(label_hint("p-navigation-menu")), div {} }
                PaginationRoot { count: 20, per_page: 10, agent: Some(label_hint("p-pagination")) }
                PinInputRoot { agent: Some(label_hint("p-pin-input")) }
                PopoverRoot { agent: Some(label_hint("p-popover")), div {} }
                RadioGroupRoot { agent: Some(label_hint("p-radio-group")), div {} }
                RangeCalendarRoot { agent: Some(label_hint("p-range-calendar")) }
                RatingGroupRoot { agent: Some(label_hint("p-rating-group")) }
                SelectRoot { agent: Some(label_hint("p-select")), div {} }
                SliderRoot { agent: Some(label_hint("p-slider")), div {} }
                SwitchRoot { agent: Some(label_hint("p-switch")) }
                TabsRoot { agent: Some(label_hint("p-tabs")), div {} }
                TimeFieldRoot { agent: Some(label_hint("p-time-field")) }
                TimeRangeFieldRoot { agent: Some(label_hint("p-time-range-field")) }
                ToastRoot { id: ToastId(1), agent: Some(label_hint("p-toast")) }
                Toggle { agent: Some(label_hint("p-toggle")), "T" }
                ToggleGroupRoot { agent: Some(label_hint("p-toggle-group")), div {} }
                TooltipProvider {
                    TooltipRoot { agent: Some(label_hint("p-tooltip")), div {} }
                }
            }
        }
    }

    let _dom_interactive = mount_with_props(
        AllInteractiveApp,
        AllInteractiveAppProps {
            bridge_slot: Rc::clone(&interactive_bridge),
        },
    );
    let bridge = interactive_bridge.borrow().clone().expect("bridge ready");
    let snap = bridge.registry_snapshot();
    let registered_labels: BTreeSet<String> = snap
        .components
        .iter()
        .map(|c| c.meta.label.clone())
        .collect();

    let expected_interactive = [
        "p-accordion",
        "p-alert-dialog",
        "p-button",
        "p-calendar",
        "p-checkbox",
        "p-collapsible",
        "p-combobox",
        "p-command",
        "p-context-menu",
        "p-date-field",
        "p-date-picker",
        "p-date-range-field",
        "p-date-range-picker",
        "p-dialog",
        "p-dropdown-menu",
        "p-field",
        "p-hover-card",
        "p-menubar",
        "p-navigation-menu",
        "p-pagination",
        "p-pin-input",
        "p-popover",
        "p-radio-group",
        "p-range-calendar",
        "p-rating-group",
        "p-select",
        "p-slider",
        "p-switch",
        "p-tabs",
        "p-time-field",
        "p-time-range-field",
        "p-toast",
        "p-toggle",
        "p-toggle-group",
        "p-tooltip",
    ];
    for label in expected_interactive {
        assert!(
            registered_labels.contains(label),
            "interactive primitive `{label}` must register an AgentSurface"
        );
    }
    assert!(
        snap.components.len() >= 33,
        "expected at least 33 registered interactive primitives, got {}",
        snap.components.len()
    );

    // Part B: Mount all 10 display/structural primitives inside <AgentProvider> -> 0 surfaces (S2-DIV-04).
    let display_bridge: Rc<RefCell<Option<AgentBridgeHandle>>> = Rc::new(RefCell::new(None));

    #[component]
    fn AllDisplayApp(bridge_slot: Rc<RefCell<Option<AgentBridgeHandle>>>) -> Element {
        let slot = Rc::clone(&bridge_slot);
        rsx! {
            AgentProvider {
                on_ready: move |h| {
                    *slot.borrow_mut() = Some(h);
                },
                AccessibleIcon { label: "Icon", span { "x" } }
                AspectRatio { ratio: 16.0 / 9.0, div {} }
                AvatarRoot { span { "AB" } }
                Label { "Field label" }
                Separator {}
                VisuallyHidden { "SR only" }
                ScrollAreaRoot { div {} }
                MeterRoot { value: 40.0 }
                ProgressRoot { value: Some(50.0) }
                ToolbarRoot { div {} }
            }
        }
    }

    let _dom_display = mount_with_props(
        AllDisplayApp,
        AllDisplayAppProps {
            bridge_slot: Rc::clone(&display_bridge),
        },
    );
    let disp_bridge = display_bridge.borrow().clone().expect("display bridge ready");
    assert!(
        disp_bridge.registry_snapshot().components.is_empty(),
        "10 display/structural primitives must register 0 agent surfaces (S2-DIV-04), got {:?}",
        disp_bridge.registry_snapshot().components
    );
}

#[test]
fn agent_webmcp_tool_list_strictly_exposes_three_tools_and_zero_bloat_fast_path() {
    // 1. WebMCP allowlist strictly exposes ["snapshot", "query", "act"] and excludes "resume_approval".
    assert_eq!(WEBMCP_TOOL_NAMES, ["snapshot", "query", "act"]);
    let webmcp_names: Vec<&str> = webmcp_tool_descriptors().iter().map(|d| d.name).collect();
    assert_eq!(webmcp_names, vec!["snapshot", "query", "act"]);
    assert!(!webmcp_names.contains(&"resume_approval"));

    // Trusted host descriptors include resume_approval, confirming explicit filtering.
    let all_names: Vec<&str> = agent_tool_descriptors().iter().map(|d| d.name).collect();
    assert!(all_names.contains(&"resume_approval"));

    // Mounting <AgentProvider webmcp: true> on host/native must not panic or fail with NoOpDocument.
    fn WebMcpEnabledApp() -> Element {
        rsx! {
            AgentProvider {
                webmcp: true,
                Button { "OK" }
            }
        }
    }
    let _dom_webmcp = mount_vdom(WebMcpEnabledApp);

    // 2. Zero-bloat fast path: mounting interactive primitives outside <AgentProvider> succeeds cleanly.
    let saw_bridge = Rc::new(Cell::new(true));

    #[component]
    fn NoProviderApp(saw_bridge: Rc<Cell<bool>>) -> Element {
        saw_bridge.set(use_agent_bridge().is_some());
        rsx! {
            Button { "No provider" }
            DialogRoot { div {} }
            SwitchRoot {}
            SliderRoot { div {} }
            TabsRoot { div {} }
            AccordionRoot { div {} }
            SelectRoot { div {} }
            PinInputRoot {}
            PopoverRoot { div {} }
            TooltipProvider {
                TooltipRoot { div {} }
            }
        }
    }

    let _dom_no_provider = mount_with_props(
        NoProviderApp,
        NoProviderAppProps {
            saw_bridge: Rc::clone(&saw_bridge),
        },
    );
    assert!(!saw_bridge.get(), "use_agent_bridge() must be None outside AgentProvider");
}

// ============================================================================
// 3. Dev-Only Diagnostics
// ============================================================================

#[test]
fn diagnostics_controlled_refusal_emits_once_and_zero_false_positives() {
    let prev_grace = with_controlled_refusal_grace(Duration::from_secs(60));
    // Clear any stale pending checks from earlier tests on this thread.
    flush_pending_controlled_checks();

    // Positive case: controlled prop ignored after non-canceled interaction emits ControlledRefusal once.
    {
        let guard = capture_diagnostics();
        let handle = DialogHandle::<()>::new();

        #[component]
        fn IgnoredControlledDialog(handle: DialogHandle<()>) -> Element {
            rsx! {
                DialogRoot {
                    open: Some(false),
                    handle,
                    div {}
                }
            }
        }

        let mut dom = mount_with_props(
            IgnoredControlledDialog,
            IgnoredControlledDialogProps {
                handle: handle.clone(),
            },
        );
        assert!(handle.open(None));
        flush_vdom(&mut dom);
        flush_pending_controlled_checks();

        #[cfg(debug_assertions)]
        {
            let refusals: Vec<_> = guard
                .diagnostics()
                .into_iter()
                .filter(|d| d.kind == DiagnosticKind::ControlledRefusal)
                .collect();
            assert_eq!(
                refusals.len(),
                1,
                "expected exactly 1 ControlledRefusal, got {refusals:?}"
            );
        }

        // Second ignored attempt on the same instance must NOT emit a duplicate ControlledRefusal.
        let _ = guard.take();
        assert!(handle.open(None));
        flush_vdom(&mut dom);
        flush_pending_controlled_checks();

        #[cfg(debug_assertions)]
        {
            let second_refusals: Vec<_> = guard
                .diagnostics()
                .into_iter()
                .filter(|d| d.kind == DiagnosticKind::ControlledRefusal)
                .collect();
            assert!(
                second_refusals.is_empty(),
                "ControlledRefusal must be deduplicated per instance, got {second_refusals:?}"
            );
        }
        let _ = guard;
    }

    // Zero false positive (a): intentional ev.cancel() inside on_open_change_details.
    {
        let guard = capture_diagnostics();
        let handle = DialogHandle::<()>::new();

        #[component]
        fn CanceledControlledDialog(handle: DialogHandle<()>) -> Element {
            rsx! {
                DialogRoot {
                    open: Some(false),
                    handle,
                    on_open_change_details: move |ev: ChangeEvent<bool>| {
                        ev.cancel();
                    },
                    div {}
                }
            }
        }

        let mut dom = mount_with_props(
            CanceledControlledDialog,
            CanceledControlledDialogProps {
                handle: handle.clone(),
            },
        );
        assert!(handle.open(None));
        flush_vdom(&mut dom);
        flush_pending_controlled_checks();

        #[cfg(debug_assertions)]
        assert!(
            guard
                .diagnostics()
                .iter()
                .all(|d| d.kind != DiagnosticKind::ControlledRefusal),
            "intentional ev.cancel() must not emit ControlledRefusal"
        );
        let _ = guard;
    }

    // Zero false positive (b): parent updates controlled prop before grace expires.
    {
        let guard = capture_diagnostics();
        let handle = DialogHandle::<()>::new();

        #[component]
        fn UpdatingControlledDialog(handle: DialogHandle<()>) -> Element {
            let mut open = use_signal(|| false);
            rsx! {
                DialogRoot {
                    open: Some(open()),
                    handle,
                    on_open_change: move |next| open.set(next),
                    div {}
                }
            }
        }

        let mut dom = mount_with_props(
            UpdatingControlledDialog,
            UpdatingControlledDialogProps {
                handle: handle.clone(),
            },
        );
        assert!(handle.open(None));
        flush_vdom(&mut dom);
        flush_pending_controlled_checks();

        #[cfg(debug_assertions)]
        assert!(
            guard
                .diagnostics()
                .iter()
                .all(|d| d.kind != DiagnosticKind::ControlledRefusal),
            "updating controlled prop before grace expires must not emit ControlledRefusal"
        );
        let _ = guard;
    }

    // Zero false positive (c): rapid false -> true -> false round-trip.
    {
        let guard = capture_diagnostics();
        let handle = DialogHandle::<()>::new();
        let open_slot: Rc<RefCell<Option<Signal<bool>>>> = Rc::new(RefCell::new(None));

        #[component]
        fn RoundTripDialog(
            handle: DialogHandle<()>,
            open_slot: Rc<RefCell<Option<Signal<bool>>>>,
        ) -> Element {
            let mut open = use_signal(|| false);
            *open_slot.borrow_mut() = Some(open);
            rsx! {
                DialogRoot {
                    open: Some(open()),
                    handle,
                    on_open_change: move |next| open.set(next),
                    div {}
                }
            }
        }

        let mut dom = mount_with_props(
            RoundTripDialog,
            RoundTripDialogProps {
                handle: handle.clone(),
                open_slot: Rc::clone(&open_slot),
            },
        );
        assert!(handle.open(None));
        flush_vdom(&mut dom);
        assert!(handle.close());
        flush_vdom(&mut dom);
        flush_pending_controlled_checks();

        #[cfg(debug_assertions)]
        assert!(
            guard
                .diagnostics()
                .iter()
                .all(|d| d.kind != DiagnosticKind::ControlledRefusal),
            "rapid false -> true -> false round-trip must not emit ControlledRefusal"
        );
        let _ = guard;
    }

    // Zero false positive (d): same-value echo (proposing false when already false).
    {
        let guard = capture_diagnostics();
        let handle = DialogHandle::<()>::new();

        #[component]
        fn SameValueDialog(handle: DialogHandle<()>) -> Element {
            rsx! {
                DialogRoot {
                    open: Some(false),
                    handle,
                    div {}
                }
            }
        }

        let mut dom = mount_with_props(
            SameValueDialog,
            SameValueDialogProps {
                handle: handle.clone(),
            },
        );
        assert!(handle.close());
        flush_vdom(&mut dom);
        flush_pending_controlled_checks();

        #[cfg(debug_assertions)]
        assert!(
            guard
                .diagnostics()
                .iter()
                .all(|d| d.kind != DiagnosticKind::ControlledRefusal),
            "same-value echo must not emit ControlledRefusal"
        );
        let _ = guard;
    }

    // Zero false positive (e): component unmounts before grace expires.
    {
        let guard = capture_diagnostics();
        let handle = DialogHandle::<()>::new();
        let show_slot: Rc<RefCell<Option<Signal<bool>>>> = Rc::new(RefCell::new(None));

        #[component]
        fn UnmountingDialog(
            handle: DialogHandle<()>,
            show_slot: Rc<RefCell<Option<Signal<bool>>>>,
        ) -> Element {
            let show = use_signal(|| true);
            *show_slot.borrow_mut() = Some(show);
            rsx! {
                if show() {
                    DialogRoot {
                        open: Some(false),
                        handle,
                        div {}
                    }
                }
            }
        }

        let mut dom = mount_with_props(
            UnmountingDialog,
            UnmountingDialogProps {
                handle: handle.clone(),
                show_slot: Rc::clone(&show_slot),
            },
        );
        assert!(handle.open(None));
        flush_vdom(&mut dom);
        let mut show = show_slot.borrow().expect("show signal");
        show.set(false);
        flush_vdom(&mut dom);
        flush_pending_controlled_checks();

        #[cfg(debug_assertions)]
        assert!(
            guard
                .diagnostics()
                .iter()
                .all(|d| d.kind != DiagnosticKind::ControlledRefusal),
            "unmounting before grace expires must not emit ControlledRefusal"
        );
        let _ = guard;
    }

    with_controlled_refusal_grace(prev_grace);
}

#[test]
fn diagnostics_controlled_and_default_provided_guard() {
    // 1. Implicit default_open = false on DialogRoot { open: Some(false) } must NOT warn.
    {
        let guard = capture_diagnostics();
        fn ImplicitDefaultDialog() -> Element {
            rsx! {
                DialogRoot {
                    open: Some(false),
                    div {}
                }
            }
        }
        let _dom = mount_vdom(ImplicitDefaultDialog);

        #[cfg(debug_assertions)]
        assert!(
            guard
                .diagnostics()
                .iter()
                .all(|d| d.kind != DiagnosticKind::ControlledAndDefaultProvided),
            "DialogRoot {{ open: Some(false) }} with implicit default must not warn"
        );
        let _ = guard;
    }

    // 2. Explicit default_open: true alongside open: Some(false) MUST warn.
    {
        let guard = capture_diagnostics();
        fn ExplicitDefaultDialog() -> Element {
            rsx! {
                DialogRoot {
                    open: Some(false),
                    default_open: true,
                    div {}
                }
            }
        }
        let _dom = mount_vdom(ExplicitDefaultDialog);

        #[cfg(debug_assertions)]
        assert!(
            guard
                .diagnostics()
                .iter()
                .any(|d| d.kind == DiagnosticKind::ControlledAndDefaultProvided),
            "DialogRoot {{ open: Some(false), default_open: true }} must emit ControlledAndDefaultProvided"
        );
        let _ = guard;
    }

    // 3. TabsRoot { value: Some("a"), default_value: Some("b") } MUST warn.
    {
        let guard = capture_diagnostics();
        fn ExplicitDefaultTabs() -> Element {
            rsx! {
                TabsRoot {
                    value: Some(String::from("a")),
                    default_value: Some(String::from("b")),
                    div {}
                }
            }
        }
        let _dom = mount_vdom(ExplicitDefaultTabs);

        #[cfg(debug_assertions)]
        assert!(
            guard
                .diagnostics()
                .iter()
                .any(|d| d.kind == DiagnosticKind::ControlledAndDefaultProvided),
            "TabsRoot {{ value: Some(\"a\"), default_value: Some(\"b\") }} must emit ControlledAndDefaultProvided"
        );
        let _ = guard;
    }
}

#[test]
fn diagnostics_detached_handle_action_once_per_verb_and_never_on_reads_or_after_unmount() {
    let guard = capture_diagnostics();
    let handle = DialogHandle::<String>::new();

    // Reads on a detached handle must NEVER emit DetachedHandleAction.
    assert!(!handle.is_open());
    assert_eq!(handle.payload(), None);
    assert_eq!(handle.active_trigger_id(), None);
    assert!(!handle.is_attached());
    assert!(!handle.opened_without_trigger());

    #[cfg(debug_assertions)]
    assert!(
        guard
            .diagnostics()
            .iter()
            .all(|d| d.kind != DiagnosticKind::DetachedHandleAction),
        "reads on detached handle must never emit DetachedHandleAction"
    );

    // Verbs before first attach warn once per verb.
    assert!(!handle.open(None));
    assert!(!handle.open(None));
    assert!(!handle.close());
    assert!(!handle.close());
    assert!(!handle.open_with_payload(String::from("payload-1")));
    assert!(!handle.open_with_payload(String::from("payload-2")));

    #[cfg(debug_assertions)]
    {
        let detached: Vec<_> = guard
            .take()
            .into_iter()
            .filter(|d| d.kind == DiagnosticKind::DetachedHandleAction)
            .collect();
        let verbs: Vec<&str> = detached.iter().map(|d| d.prop).collect();
        assert_eq!(
            verbs,
            vec!["open", "close", "open_with_payload"],
            "DetachedHandleAction must fire exactly once per verb before first attach"
        );
    }

    // Attach to a mounted DialogRoot, then unmount (has_ever_attached == true).
    let show_slot: Rc<RefCell<Option<Signal<bool>>>> = Rc::new(RefCell::new(None));

    #[component]
    fn MountThenUnmountDialog(
        handle: DialogHandle<String>,
        show_slot: Rc<RefCell<Option<Signal<bool>>>>,
    ) -> Element {
        let show = use_signal(|| true);
        *show_slot.borrow_mut() = Some(show);
        rsx! {
            if show() {
                DialogRoot {
                    handle,
                    div {}
                }
            }
        }
    }

    let mut dom = mount_with_props(
        MountThenUnmountDialog,
        MountThenUnmountDialogProps {
            handle: handle.clone(),
            show_slot: Rc::clone(&show_slot),
        },
    );
    assert!(handle.is_attached());

    let mut show = show_slot.borrow().expect("show signal");
    show.set(false);
    flush_vdom(&mut dom);
    assert!(!handle.is_attached());

    // After unmount, calling verbs again must NOT emit DetachedHandleAction.
    let _ = guard.take();
    assert!(!handle.open(None));
    assert!(!handle.close());
    assert!(!handle.open_with_payload(String::from("after-unmount")));

    #[cfg(debug_assertions)]
    assert!(
        guard
            .diagnostics()
            .iter()
            .all(|d| d.kind != DiagnosticKind::DetachedHandleAction),
        "DetachedHandleAction must never fire after unmount (has_ever_attached == true)"
    );
    let _ = guard;
}

// ============================================================================
// Native DOM Verification (Blitz)
// ============================================================================

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use dioxus::prelude::keyboard_types::{Code, Key, NamedKey};
    use monoxus::popover::{PopoverContent, PopoverPortal};
    use monoxus::tooltip::{TooltipContent, TooltipPortal};
    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, mount_native_scenario, mount_native_scenario_with_props,
        send_key_and_pump,
    };

    #[test]
    fn native_nested_escape_popover_inside_dialog_respects_allow_propagation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        #[component]
        fn PopoverInDialog(
            allow_propagation: bool,
            dialog_open_out: Rc<Cell<bool>>,
            popover_open_out: Rc<Cell<bool>>,
        ) -> Element {
            let mut dialog_open = use_signal(|| true);
            let mut popover_open = use_signal(|| true);
            dialog_open_out.set(dialog_open());
            popover_open_out.set(popover_open());

            rsx! {
                DialogRoot {
                    id: "esc-parent-dialog",
                    open: Some(dialog_open()),
                    on_open_change: move |next| dialog_open.set(next),
                    DialogPortal {
                        DialogContent {
                            PopoverRoot {
                                id: "esc-child-popover",
                                open: Some(popover_open()),
                                on_open_change: move |next| popover_open.set(next),
                                on_open_change_details: move |ev: ChangeEvent<bool>| {
                                    if allow_propagation {
                                        ev.allow_propagation();
                                    }
                                },
                                PopoverPortal {
                                    PopoverContent { "Child Popover" }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Case 1: Without allow_propagation() -> Escape closes Popover, parent Dialog stays open.
        clear_active_overlay_stack();
        let d_open = Rc::new(Cell::new(true));
        let p_open = Rc::new(Cell::new(true));
        let mut harness = mount_native_scenario_with_props(
            PopoverInDialog,
            PopoverInDialogProps {
                allow_propagation: false,
                dialog_open_out: Rc::clone(&d_open),
                popover_open_out: Rc::clone(&p_open),
            },
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert!(!p_open.get(), "child Popover must close on Escape");
        assert!(
            d_open.get(),
            "parent Dialog must stay open when child does not call allow_propagation()"
        );
        drop(harness);

        // Case 2: With allow_propagation() -> Escape closes Popover AND parent Dialog.
        clear_active_overlay_stack();
        let d_open2 = Rc::new(Cell::new(true));
        let p_open2 = Rc::new(Cell::new(true));
        let mut harness2 = mount_native_scenario_with_props(
            PopoverInDialog,
            PopoverInDialogProps {
                allow_propagation: true,
                dialog_open_out: Rc::clone(&d_open2),
                popover_open_out: Rc::clone(&p_open2),
            },
        );
        send_key_and_pump(&mut harness2, Key::Named(NamedKey::Escape), Code::Escape);
        assert!(!p_open2.get(), "child Popover must close on Escape");
        assert!(
            !d_open2.get(),
            "parent Dialog must also close when child calls allow_propagation()"
        );
        drop(harness2);
        clear_active_overlay_stack();
    }

    #[test]
    fn native_nested_escape_tooltip_inside_dialog_respects_allow_propagation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        #[component]
        fn TooltipInDialog(
            allow_propagation: bool,
            dialog_open_out: Rc<Cell<bool>>,
            tooltip_open_out: Rc<Cell<bool>>,
        ) -> Element {
            let mut dialog_open = use_signal(|| true);
            let mut tooltip_open = use_signal(|| true);
            dialog_open_out.set(dialog_open());
            tooltip_open_out.set(tooltip_open());

            rsx! {
                TooltipProvider {
                    DialogRoot {
                        id: "esc-tt-parent-dialog",
                        open: Some(dialog_open()),
                        on_open_change: move |next| dialog_open.set(next),
                        DialogPortal {
                            DialogContent {
                                TooltipRoot {
                                    id: "esc-child-tooltip",
                                    open: Some(tooltip_open()),
                                    on_open_change: move |next| tooltip_open.set(next),
                                    on_open_change_details: move |ev: ChangeEvent<bool>| {
                                        if allow_propagation {
                                            ev.allow_propagation();
                                        }
                                    },
                                    TooltipPortal {
                                        TooltipContent { "Child Tooltip" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        clear_active_overlay_stack();
        let d_open = Rc::new(Cell::new(true));
        let t_open = Rc::new(Cell::new(true));
        let mut harness = mount_native_scenario_with_props(
            TooltipInDialog,
            TooltipInDialogProps {
                allow_propagation: false,
                dialog_open_out: Rc::clone(&d_open),
                tooltip_open_out: Rc::clone(&t_open),
            },
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        assert!(!t_open.get(), "child Tooltip must close on Escape");
        assert!(
            d_open.get(),
            "parent Dialog must stay open when Tooltip does not call allow_propagation()"
        );
        drop(harness);

        clear_active_overlay_stack();
        let d_open2 = Rc::new(Cell::new(true));
        let t_open2 = Rc::new(Cell::new(true));
        let mut harness2 = mount_native_scenario_with_props(
            TooltipInDialog,
            TooltipInDialogProps {
                allow_propagation: true,
                dialog_open_out: Rc::clone(&d_open2),
                tooltip_open_out: Rc::clone(&t_open2),
            },
        );
        send_key_and_pump(&mut harness2, Key::Named(NamedKey::Escape), Code::Escape);
        assert!(!t_open2.get(), "child Tooltip must close on Escape");
        assert!(
            !d_open2.get(),
            "parent Dialog must close when Tooltip calls allow_propagation()"
        );
        drop(harness2);
        clear_active_overlay_stack();
    }

    #[test]
    fn native_agent_boundary_renders_zero_wrapper_dom_elements() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        fn BoundaryDomApp() -> Element {
            rsx! {
                AgentProvider {
                    div {
                        id: "boundary-parent",
                        AgentBoundary {
                            section: "Outer",
                            AgentBoundary {
                                section: "Inner",
                                button {
                                    id: "boundary-leaf",
                                    "Leaf"
                                }
                            }
                        }
                    }
                }
            }
        }

        let harness = mount_native_scenario(BoundaryDomApp);
        let parent_id = harness.node("#boundary-parent");
        let leaf_id = harness.node("#boundary-leaf");
        let leaf_parent = harness
            .base()
            .get_node(leaf_id)
            .and_then(|node| node.parent);
        assert_eq!(
            leaf_parent,
            Some(parent_id),
            "<AgentBoundary> must render zero wrapper DOM elements"
        );
    }
}

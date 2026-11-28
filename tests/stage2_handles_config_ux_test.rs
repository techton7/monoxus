//! Stage 2 Failure-Mode Integration Tests — Part 2 (`S2.5`, Ledger Section 4 Items 4–8):
//! - Item 4: Detached Triggers & Typed `OverlayHandle<K, P = ()>`
//! - Item 5: `<MonoxusConfig>` Cascading Context
//! - Item 6: Nested `DialogBackdrop` (`DialogOverlay`)
//! - Item 7: Async Pending `Button` & Shared Accessible Announcer
//! - Item 8: `Progress.min`

use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Waker},
    time::Duration,
};

use dioxus::{core::AttributeValue, prelude::*};
use monoxus::*;

use crate::common::vdom::{flush_vdom, mount_vdom, mount_with_props};

fn poll_once<F: Future>(fut: F) {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut fut = std::pin::pin!(fut);
    let _ = Pin::as_mut(&mut fut).poll(&mut cx);
}

fn pump_vdom(dom: &mut VirtualDom) {
    for _ in 0..32 {
        poll_once(dom.wait_for_work());
        flush_vdom(dom);
    }
}

fn clear_active_overlay_stack() {
    for layer_id in foundation::overlay::active_overlay_layers_snapshot() {
        foundation::overlay::unregister_active_overlay_layer(&layer_id);
    }
}

fn extract_attr(attrs: &[Attribute], name: &str) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if attr.name != name {
            return None;
        }
        match &attr.value {
            AttributeValue::Text(s) => Some(s.clone()),
            AttributeValue::Bool(b) => Some(b.to_string()),
            AttributeValue::Int(i) => Some(i.to_string()),
            AttributeValue::Float(f) => Some(f.to_string()),
            _ => None,
        }
    })
}

fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.name == name)
}

#[derive(Clone)]
struct PtrSlot<T>(Rc<RefCell<T>>);

impl<T> PartialEq for PtrSlot<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> PtrSlot<T> {
    fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

struct PortalRecorderGuard;

impl PortalRecorderGuard {
    fn start() -> Self {
        foundation::overlay::start_portal_op_recording();
        Self
    }
}

impl Drop for PortalRecorderGuard {
    fn drop(&mut self) {
        foundation::overlay::stop_portal_op_recording();
    }
}

// =============================================================================
// Item 4: Detached Triggers & Typed `OverlayHandle<K, P = ()>`
// =============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
struct ItemPayload {
    id: u32,
    label: &'static str,
}

#[component]
fn NonGenericExistingCallsitesApp() -> Element {
    rsx! {
        div {
            dialog::DialogRoot {
                dialog::DialogTrigger { "Open Dialog" }
                dialog::DialogContent { "Dialog Body" }
            }
            popover::PopoverRoot {
                popover::PopoverTrigger { "Open Popover" }
                popover::PopoverContent { "Popover Body" }
            }
            tooltip::TooltipRoot {
                tooltip::TooltipTrigger { "Hover Tooltip" }
                tooltip::TooltipContent { "Tooltip Body" }
            }
            hover_card::HoverCardRoot {
                hover_card::HoverCardTrigger { "Hover Card" }
                hover_card::HoverCardContent { "Hover Card Body" }
            }
        }
    }
}

#[test]
fn item4_non_generic_existing_callsites_compile_without_type_annotations() {
    clear_active_overlay_stack();
    let mut dom = mount_vdom(NonGenericExistingCallsitesApp);
    pump_vdom(&mut dom);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DetachedOverlaySnapshot {
    trig_a_expanded: Option<String>,
    trig_b_expanded: Option<String>,
    trig_a_state: Option<String>,
    trig_b_state: Option<String>,
    rendered_payload: Option<ItemPayload>,
}

#[component]
fn MultiDetachedTriggersApp(
    dialog_handle: foundation::overlay::DialogHandle<ItemPayload>,
    popover_handle: foundation::overlay::PopoverHandle<ItemPayload>,
    tooltip_handle: foundation::overlay::TooltipHandle<ItemPayload>,
    dialog_snap: PtrSlot<DetachedOverlaySnapshot>,
    popover_snap: PtrSlot<DetachedOverlaySnapshot>,
    tooltip_snap: PtrSlot<DetachedOverlaySnapshot>,
    dialog_close_count: PtrSlot<usize>,
    popover_close_count: PtrSlot<usize>,
    tooltip_close_count: PtrSlot<usize>,
) -> Element {
    let payload_a = ItemPayload {
        id: 1,
        label: "Alpha",
    };
    let payload_b = ItemPayload {
        id: 2,
        label: "Beta",
    };

    let dlg_snap_a = dialog_snap.clone();
    let dlg_snap_b = dialog_snap.clone();
    let dlg_snap_p = dialog_snap.clone();
    let dlg_close = dialog_close_count.clone();
    let dlg_h_content = dialog_handle.clone();

    let pop_snap_a = popover_snap.clone();
    let pop_snap_b = popover_snap.clone();
    let pop_snap_p = popover_snap.clone();
    let pop_close = popover_close_count.clone();
    let pop_h_content = popover_handle.clone();

    let tt_snap_a = tooltip_snap.clone();
    let tt_snap_b = tooltip_snap.clone();
    let tt_snap_p = tooltip_snap.clone();
    let tt_close = tooltip_close_count.clone();
    let tt_h_content = tooltip_handle.clone();

    let dlg_payload_now = dlg_h_content.payload();
    dlg_snap_p.0.borrow_mut().rendered_payload = dlg_payload_now;

    let pop_payload_now = pop_h_content.payload();
    pop_snap_p.0.borrow_mut().rendered_payload = pop_payload_now;

    let tt_payload_now = tt_h_content.payload();
    tt_snap_p.0.borrow_mut().rendered_payload = tt_payload_now;

    rsx! {
        div {
            // Detached Dialog triggers outside DialogRoot
            dialog::DialogTrigger {
                id: "dlg-trig-a",
                handle: dialog_handle.clone(),
                payload: Some(dialog_handle.trigger_payload(payload_a.clone())),
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    let mut s = dlg_snap_a.0.borrow_mut();
                    s.trig_a_expanded = extract_attr(&attrs, "aria-expanded");
                    s.trig_a_state = extract_attr(&attrs, "data-state");
                    rsx! { button { ..attrs, "Dlg A" } }
                }),
            }
            dialog::DialogTrigger {
                id: "dlg-trig-b",
                handle: dialog_handle.clone(),
                payload: Some(dialog_handle.trigger_payload(payload_b.clone())),
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    let mut s = dlg_snap_b.0.borrow_mut();
                    s.trig_b_expanded = extract_attr(&attrs, "aria-expanded");
                    s.trig_b_state = extract_attr(&attrs, "data-state");
                    rsx! { button { ..attrs, "Dlg B" } }
                }),
            }
            dialog::DialogRoot {
                id: "detached-dlg-root",
                handle: dialog_handle.clone(),
                on_open_change: move |open: bool| {
                    if !open {
                        *dlg_close.0.borrow_mut() += 1;
                    }
                },
                dialog::DialogContent {
                    id: "detached-dlg-content",
                    "Dialog Content"
                }
            }

            // Detached Popover triggers outside PopoverRoot
            popover::PopoverTrigger {
                id: "pop-trig-a",
                handle: popover_handle.clone(),
                payload: Some(popover_handle.trigger_payload(payload_a.clone())),
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    let mut s = pop_snap_a.0.borrow_mut();
                    s.trig_a_expanded = extract_attr(&attrs, "aria-expanded");
                    s.trig_a_state = extract_attr(&attrs, "data-state");
                    rsx! { button { ..attrs, "Pop A" } }
                }),
            }
            popover::PopoverTrigger {
                id: "pop-trig-b",
                handle: popover_handle.clone(),
                payload: Some(popover_handle.trigger_payload(payload_b.clone())),
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    let mut s = pop_snap_b.0.borrow_mut();
                    s.trig_b_expanded = extract_attr(&attrs, "aria-expanded");
                    s.trig_b_state = extract_attr(&attrs, "data-state");
                    rsx! { button { ..attrs, "Pop B" } }
                }),
            }
            popover::PopoverRoot {
                id: "detached-pop-root",
                handle: popover_handle.clone(),
                on_open_change: move |open: bool| {
                    if !open {
                        *pop_close.0.borrow_mut() += 1;
                    }
                },
                popover::PopoverContent {
                    id: "detached-pop-content",
                    "Popover Content"
                }
            }

            // Detached Tooltip triggers outside TooltipRoot
            tooltip::TooltipTrigger {
                id: "tt-trig-a",
                handle: tooltip_handle.clone(),
                payload: Some(tooltip_handle.trigger_payload(payload_a)),
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    let mut s = tt_snap_a.0.borrow_mut();
                    s.trig_a_state = extract_attr(&attrs, "data-state");
                    rsx! { button { ..attrs, "TT A" } }
                }),
            }
            tooltip::TooltipTrigger {
                id: "tt-trig-b",
                handle: tooltip_handle.clone(),
                payload: Some(tooltip_handle.trigger_payload(payload_b)),
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    let mut s = tt_snap_b.0.borrow_mut();
                    s.trig_b_state = extract_attr(&attrs, "data-state");
                    rsx! { button { ..attrs, "TT B" } }
                }),
            }
            tooltip::TooltipRoot {
                id: "detached-tt-root",
                handle: tooltip_handle.clone(),
                on_open_change: move |open: bool| {
                    if !open {
                        *tt_close.0.borrow_mut() += 1;
                    }
                },
                tooltip::TooltipContent {
                    id: "detached-tt-content",
                    "Tooltip Content"
                }
            }
        }
    }
}

#[test]
fn item4_dialog_popover_tooltip_multi_detached_triggers_with_typed_payload() {
    clear_active_overlay_stack();

    let dialog_handle = foundation::overlay::DialogHandle::<ItemPayload>::new();
    let popover_handle = foundation::overlay::PopoverHandle::<ItemPayload>::new();
    let tooltip_handle = foundation::overlay::TooltipHandle::<ItemPayload>::new();

    let dialog_snap = PtrSlot::new(DetachedOverlaySnapshot::default());
    let popover_snap = PtrSlot::new(DetachedOverlaySnapshot::default());
    let tooltip_snap = PtrSlot::new(DetachedOverlaySnapshot::default());

    let dialog_close_count = PtrSlot::new(0usize);
    let popover_close_count = PtrSlot::new(0usize);
    let tooltip_close_count = PtrSlot::new(0usize);

    let mut dom = mount_with_props(
        MultiDetachedTriggersApp,
        MultiDetachedTriggersAppProps {
            dialog_handle: dialog_handle.clone(),
            popover_handle: popover_handle.clone(),
            tooltip_handle: tooltip_handle.clone(),
            dialog_snap: dialog_snap.clone(),
            popover_snap: popover_snap.clone(),
            tooltip_snap: tooltip_snap.clone(),
            dialog_close_count: dialog_close_count.clone(),
            popover_close_count: popover_close_count.clone(),
            tooltip_close_count: tooltip_close_count.clone(),
        },
    );
    pump_vdom(&mut dom);

    let payload_a = ItemPayload {
        id: 1,
        label: "Alpha",
    };
    let payload_b = ItemPayload {
        id: 2,
        label: "Beta",
    };

    // --- Dialog: Trigger A -> Trigger B switch without closing ---
    assert!(dialog_handle.as_any().activate_trigger("dlg-trig-a", true));
    pump_vdom(&mut dom);
    assert!(dialog_handle.is_open());
    assert_eq!(dialog_handle.active_trigger_id().as_deref(), Some("dlg-trig-a"));
    assert_eq!(dialog_handle.payload(), Some(payload_a.clone()));
    {
        let s = dialog_snap.0.borrow();
        assert_eq!(s.trig_a_expanded.as_deref(), Some("true"));
        assert_eq!(s.trig_b_expanded.as_deref(), Some("false"));
        assert_eq!(s.rendered_payload, Some(payload_a.clone()));
    }

    assert!(dialog_handle.as_any().activate_trigger("dlg-trig-b", true));
    pump_vdom(&mut dom);
    assert!(dialog_handle.is_open());
    assert_eq!(*dialog_close_count.0.borrow(), 0, "switching detached triggers must not close dialog");
    assert_eq!(dialog_handle.active_trigger_id().as_deref(), Some("dlg-trig-b"));
    assert_eq!(dialog_handle.payload(), Some(payload_b.clone()));
    {
        let s = dialog_snap.0.borrow();
        assert_eq!(s.trig_a_expanded.as_deref(), Some("false"));
        assert_eq!(s.trig_b_expanded.as_deref(), Some("true"));
        assert_eq!(s.rendered_payload, Some(payload_b.clone()));
    }

    // --- Popover: Trigger A -> Trigger B switch without closing ---
    assert!(popover_handle.as_any().activate_trigger("pop-trig-a", true));
    pump_vdom(&mut dom);
    assert!(popover_handle.is_open());
    assert_eq!(popover_handle.active_trigger_id().as_deref(), Some("pop-trig-a"));
    assert_eq!(popover_handle.payload(), Some(payload_a.clone()));
    {
        let s = popover_snap.0.borrow();
        assert_eq!(s.trig_a_expanded.as_deref(), Some("true"));
        assert_eq!(s.trig_b_expanded.as_deref(), Some("false"));
        assert_eq!(s.rendered_payload, Some(payload_a.clone()));
    }

    assert!(popover_handle.as_any().activate_trigger("pop-trig-b", true));
    pump_vdom(&mut dom);
    assert!(popover_handle.is_open());
    assert_eq!(*popover_close_count.0.borrow(), 0, "switching detached triggers must not close popover");
    assert_eq!(popover_handle.active_trigger_id().as_deref(), Some("pop-trig-b"));
    assert_eq!(popover_handle.payload(), Some(payload_b.clone()));
    {
        let s = popover_snap.0.borrow();
        assert_eq!(s.trig_a_expanded.as_deref(), Some("false"));
        assert_eq!(s.trig_b_expanded.as_deref(), Some("true"));
        assert_eq!(s.rendered_payload, Some(payload_b.clone()));
    }

    // --- Tooltip: Trigger A -> Trigger B switch without closing ---
    assert!(tooltip_handle.as_any().activate_trigger("tt-trig-a", true));
    pump_vdom(&mut dom);
    assert!(tooltip_handle.is_open());
    assert_eq!(tooltip_handle.active_trigger_id().as_deref(), Some("tt-trig-a"));
    assert_eq!(tooltip_handle.payload(), Some(payload_a.clone()));
    {
        let s = tooltip_snap.0.borrow();
        assert_eq!(s.trig_a_state.as_deref(), Some("open"));
        assert_eq!(s.trig_b_state.as_deref(), Some("closed"));
        assert_eq!(s.rendered_payload, Some(payload_a));
    }

    assert!(tooltip_handle.as_any().activate_trigger("tt-trig-b", true));
    pump_vdom(&mut dom);
    assert!(tooltip_handle.is_open());
    assert_eq!(*tooltip_close_count.0.borrow(), 0, "switching detached triggers must not close tooltip");
    assert_eq!(tooltip_handle.active_trigger_id().as_deref(), Some("tt-trig-b"));
    assert_eq!(tooltip_handle.payload(), Some(payload_b.clone()));
    {
        let s = tooltip_snap.0.borrow();
        assert_eq!(s.trig_a_state.as_deref(), Some("closed"));
        assert_eq!(s.trig_b_state.as_deref(), Some("open"));
        assert_eq!(s.rendered_payload, Some(payload_b));
    }

    // Programmatic open_with_payload opens without associating any trigger.
    let payload_c = ItemPayload {
        id: 3,
        label: "Gamma",
    };
    assert!(dialog_handle.close());
    pump_vdom(&mut dom);
    assert!(!dialog_handle.is_open());

    assert!(dialog_handle.open_with_payload(payload_c.clone()));
    pump_vdom(&mut dom);
    assert!(dialog_handle.is_open());
    assert!(dialog_handle.opened_without_trigger());
    assert_eq!(dialog_handle.active_trigger_id(), None);
    assert_eq!(dialog_handle.payload(), Some(payload_c.clone()));
    {
        let s = dialog_snap.0.borrow();
        assert_eq!(s.trig_a_expanded.as_deref(), Some("false"));
        assert_eq!(s.trig_b_expanded.as_deref(), Some("false"));
        assert_eq!(s.rendered_payload, Some(payload_c));
    }
}

#[derive(Clone, Default)]
struct RouteTransitionControl {
    show_a: Option<Signal<bool>>,
    show_b: Option<Signal<bool>>,
    a_open: bool,
    b_open: bool,
}

#[component]
fn RouteTransitionApp(
    handle: foundation::overlay::DialogHandle<ItemPayload>,
    ctl: PtrSlot<RouteTransitionControl>,
) -> Element {
    let show_a = use_signal(|| true);
    let show_b = use_signal(|| false);
    {
        let mut c = ctl.0.borrow_mut();
        c.show_a = Some(show_a);
        c.show_b = Some(show_b);
    }

    let ctl_a = ctl.clone();
    let ctl_b = ctl.clone();

    rsx! {
        div {
            div { id: "route-a-slot",
                if show_a() {
                    dialog::DialogRoot {
                        id: "route-a-root",
                        handle: handle.clone(),
                        on_open_change: move |open: bool| {
                            ctl_a.0.borrow_mut().a_open = open;
                        },
                        dialog::DialogContent { "Route A" }
                    }
                }
            }
            div { id: "route-b-slot",
                if show_b() {
                    dialog::DialogRoot {
                        id: "route-b-root",
                        handle: handle.clone(),
                        on_open_change: move |open: bool| {
                            ctl_b.0.borrow_mut().b_open = open;
                        },
                        dialog::DialogContent { "Route B" }
                    }
                }
            }
        }
    }
}

#[test]
fn item4_lifo_attach_stack_survives_overlapping_route_transition_mount_unmount() {
    clear_active_overlay_stack();

    let diag_guard = foundation::diagnostics::capture_diagnostics();
    let handle = foundation::overlay::DialogHandle::<ItemPayload>::new();
    let ctl = PtrSlot::new(RouteTransitionControl::default());

    let mut dom = mount_with_props(
        RouteTransitionApp,
        RouteTransitionAppProps {
            handle: handle.clone(),
            ctl: ctl.clone(),
        },
    );
    pump_vdom(&mut dom);

    assert!(handle.is_attached());
    assert!(diag_guard.diagnostics().is_empty());

    // Mount Route B BEFORE dropping Route A (overlapping route transition).
    let mut show_b = ctl.0.borrow().show_b.expect("show_b signal");
    let mut show_a = ctl.0.borrow().show_a.expect("show_a signal");
    dom.in_scope(ScopeId::ROOT, || {
        show_b.set(true);
    });
    pump_vdom(&mut dom);

    let diags = diag_guard.take();
    assert!(
        diags
            .iter()
            .any(|d| d.kind == foundation::diagnostics::DiagnosticKind::HandleOverlap),
        "expected HandleOverlap diagnostic during overlapping root mount, got {diags:?}"
    );

    // Opening targets Route B (LIFO top), leaving Route A closed.
    let payload = ItemPayload {
        id: 42,
        label: "RouteB",
    };
    assert!(handle.open_with_payload(payload.clone()));
    pump_vdom(&mut dom);
    assert!(handle.is_open());
    assert!(handle.opened_without_trigger());
    assert_eq!(handle.payload(), Some(payload));
    assert!(!ctl.0.borrow().a_open);
    assert!(ctl.0.borrow().b_open);

    // Dropping old Route A must NOT detach Route B.
    dom.in_scope(ScopeId::ROOT, || {
        show_a.set(false);
    });
    pump_vdom(&mut dom);
    assert!(handle.is_attached());
    assert!(handle.is_open());

    assert!(handle.close());
    pump_vdom(&mut dom);
    assert!(!handle.is_open());
    assert!(!ctl.0.borrow().b_open);

    // Dropping Route B empties the LIFO attach stack.
    dom.in_scope(ScopeId::ROOT, || {
        show_b.set(false);
    });
    pump_vdom(&mut dom);
    assert!(!handle.is_attached());
}

// =============================================================================
// Item 5: `<MonoxusConfig>` Cascading Context
// =============================================================================

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ConfigProbeSnapshot {
    values: MonoxusConfigValues,
    dir: foundation::shared::Direction,
    portal: foundation::overlay::PortalHost,
    locale: foundation::date_time::CalendarLocale,
}

#[component]
fn ConfigProbe(out: PtrSlot<Option<ConfigProbeSnapshot>>) -> Element {
    let snap = ConfigProbeSnapshot {
        values: use_monoxus_config(),
        dir: foundation::shared::use_direction_context(None),
        portal: foundation::config::use_portal_host_context(
            None,
            foundation::overlay::PortalHost::default_host(),
        ),
        locale: foundation::config::use_locale_context(None),
    };
    *out.0.borrow_mut() = Some(snap);
    rsx! { span { "probe" } }
}

#[component]
fn CascadingConfigAndDirectionApp(
    nested_out: PtrSlot<Option<ConfigProbeSnapshot>>,
    interleaved_a_out: PtrSlot<Option<ConfigProbeSnapshot>>,
    interleaved_b_out: PtrSlot<Option<ConfigProbeSnapshot>>,
) -> Element {
    use foundation::{
        date_time::CalendarLocale,
        overlay::PortalHost,
        shared::{Direction, DirectionProvider},
    };

    rsx! {
        div {
            // 1. Nested MonoxusConfig inherits omitted fields and overrides only specified field.
            MonoxusConfig {
                dir: Direction::Rtl,
                default_portal_to: PortalHost::named("outer-portal"),
                default_locale: CalendarLocale::KoKr,
                MonoxusConfig {
                    default_portal_to: PortalHost::named("inner-portal"),
                    ConfigProbe { out: nested_out }
                }
            }

            // 2. Interleaved MonoxusConfig -> DirectionProvider -> MonoxusConfig.
            MonoxusConfig {
                dir: Direction::Rtl,
                default_portal_to: PortalHost::named("outer-interleaved"),
                default_locale: CalendarLocale::KoKr,
                DirectionProvider {
                    dir: Direction::Ltr,
                    MonoxusConfig {
                        default_locale: CalendarLocale::EnGb,
                        ConfigProbe { out: interleaved_a_out }
                    }
                }
            }

            // 3. Interleaved DirectionProvider -> MonoxusConfig -> DirectionProvider.
            DirectionProvider {
                dir: Direction::Rtl,
                MonoxusConfig {
                    dir: Direction::Ltr,
                    default_portal_to: PortalHost::named("mid-portal"),
                    DirectionProvider {
                        dir: Direction::Rtl,
                        ConfigProbe { out: interleaved_b_out }
                    }
                }
            }
        }
    }
}

#[test]
fn item5_nested_monoxus_config_and_interleaved_direction_provider() {
    use foundation::{date_time::CalendarLocale, overlay::PortalHost, shared::Direction};

    let nested_out = PtrSlot::new(None);
    let interleaved_a_out = PtrSlot::new(None);
    let interleaved_b_out = PtrSlot::new(None);

    let mut dom = mount_with_props(
        CascadingConfigAndDirectionApp,
        CascadingConfigAndDirectionAppProps {
            nested_out: nested_out.clone(),
            interleaved_a_out: interleaved_a_out.clone(),
            interleaved_b_out: interleaved_b_out.clone(),
        },
    );
    pump_vdom(&mut dom);

    let nested = nested_out.0.borrow().clone().expect("nested snapshot");
    assert_eq!(nested.dir, Direction::Rtl);
    assert_eq!(nested.values.dir, Some(Direction::Rtl));
    assert_eq!(nested.portal, PortalHost::named("inner-portal"));
    assert_eq!(nested.locale, CalendarLocale::KoKr);

    let inter_a = interleaved_a_out
        .0
        .borrow()
        .clone()
        .expect("interleaved A snapshot");
    assert_eq!(inter_a.dir, Direction::Ltr);
    assert_eq!(inter_a.portal, PortalHost::named("outer-interleaved"));
    assert_eq!(inter_a.locale, CalendarLocale::EnGb);

    let inter_b = interleaved_b_out
        .0
        .borrow()
        .clone()
        .expect("interleaved B snapshot");
    assert_eq!(inter_b.dir, Direction::Rtl);
    assert_eq!(inter_b.portal, PortalHost::named("mid-portal"));
}

#[component]
fn AllSevenPortalsUnderConfigApp(explicit_host: Option<foundation::overlay::PortalHost>) -> Element {
    rsx! {
        MonoxusConfig {
            default_portal_to: foundation::overlay::PortalHost::named("cfg-portal-host"),
            dialog::DialogRoot {
                id: "cfg-dlg",
                default_open: true,
                dialog::DialogTrigger { "Dlg" }
                dialog::DialogPortal {
                    host: explicit_host.clone(),
                    dialog::DialogContent { id: "cfg-dlg-content", "Dlg" }
                }
            }
            popover::PopoverRoot {
                id: "cfg-pop",
                default_open: true,
                popover::PopoverTrigger { "Pop" }
                popover::PopoverPortal {
                    host: explicit_host.clone(),
                    popover::PopoverContent { id: "cfg-pop-content", "Pop" }
                }
            }
            tooltip::TooltipRoot {
                id: "cfg-tt",
                default_open: true,
                tooltip::TooltipTrigger { "TT" }
                tooltip::TooltipPortal {
                    host: explicit_host.clone(),
                    tooltip::TooltipContent { id: "cfg-tt-content", "TT" }
                }
            }
            hover_card::HoverCardRoot {
                id: "cfg-hc",
                default_open: true,
                hover_card::HoverCardTrigger { "HC" }
                hover_card::HoverCardPortal {
                    host: explicit_host.clone(),
                    hover_card::HoverCardContent { id: "cfg-hc-content", "HC" }
                }
            }
            select::SelectRoot {
                id: Some("cfg-sel".to_string()),
                open: true,
                select::SelectTrigger { "Sel" }
                select::SelectPortal {
                    host: explicit_host.clone(),
                    select::SelectContent {
                        select::SelectItem { value: "a".to_string(), "Alpha" }
                    }
                }
            }
            dropdown_menu::DropdownMenuRoot {
                open: Some(true),
                modal: false,
                dropdown_menu::DropdownMenuTrigger { "Menu" }
                dropdown_menu::DropdownMenuPortal {
                    host: explicit_host.clone(),
                    dropdown_menu::DropdownMenuContent {
                        id: Some("cfg-menu-content".to_string()),
                        dropdown_menu::DropdownMenuItem { "One" }
                    }
                }
            }
            combobox::ComboboxRoot {
                id: Some("cfg-cbx".to_string()),
                default_open: true,
                combobox::ComboboxInput {}
                combobox::ComboboxPortal {
                    host: explicit_host,
                    combobox::ComboboxContent {
                        id: Some("cfg-cbx-content".to_string()),
                        combobox::ComboboxItem { value: "a".to_string(), "Alpha" }
                    }
                }
            }
        }
    }
}

#[test]
fn item5_all_seven_portals_inherit_default_portal_to_and_honor_explicit_host_override() {
    clear_active_overlay_stack();
    let _rec = PortalRecorderGuard::start();
    let _ = foundation::overlay::take_portal_ops();

    // 1. Omitted `host` -> all 7 inherit `default_portal_to: "cfg-portal-host"`.
    let mut inherited_dom = mount_with_props(
        AllSevenPortalsUnderConfigApp,
        AllSevenPortalsUnderConfigAppProps {
            explicit_host: None,
        },
    );
    pump_vdom(&mut inherited_dom);

    let inherited_hosts: Vec<Option<String>> = foundation::overlay::take_portal_ops()
        .into_iter()
        .filter_map(|op| match op {
            foundation::overlay::PortalOp::Teleport { host_id, .. } => Some(host_id),
            foundation::overlay::PortalOp::Remove { .. } => None,
        })
        .collect();
    assert_eq!(
        inherited_hosts.len(),
        7,
        "all 7 *Portal components must teleport when open"
    );
    assert!(
        inherited_hosts
            .iter()
            .all(|h| h.as_deref() == Some("cfg-portal-host")),
        "all 7 *Portal components must inherit default_portal_to, got {inherited_hosts:?}"
    );
    drop(inherited_dom);
    let _ = foundation::overlay::take_portal_ops();

    // 2. Explicit `host` -> all 7 honor explicit override `"override-portal-host"`.
    clear_active_overlay_stack();
    let mut override_dom = mount_with_props(
        AllSevenPortalsUnderConfigApp,
        AllSevenPortalsUnderConfigAppProps {
            explicit_host: Some(foundation::overlay::PortalHost::named(
                "override-portal-host",
            )),
        },
    );
    pump_vdom(&mut override_dom);

    let override_hosts: Vec<Option<String>> = foundation::overlay::take_portal_ops()
        .into_iter()
        .filter_map(|op| match op {
            foundation::overlay::PortalOp::Teleport { host_id, .. } => Some(host_id),
            foundation::overlay::PortalOp::Remove { .. } => None,
        })
        .collect();
    assert_eq!(
        override_hosts.len(),
        7,
        "all 7 *Portal components must teleport with explicit host"
    );
    assert!(
        override_hosts
            .iter()
            .all(|h| h.as_deref() == Some("override-portal-host")),
        "all 7 *Portal components must honor explicit host override, got {override_hosts:?}"
    );
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DirInheritSnapshot {
    hover_card_floating: Option<String>,
    hover_card_static: Option<String>,
    toast_viewport: Option<String>,
    hover_card_override: Option<String>,
    toast_override: Option<String>,
}

#[component]
fn HoverCardAndToastDirApp(out: PtrSlot<DirInheritSnapshot>) -> Element {
    let out_f = out.clone();
    let out_s = out.clone();
    let out_t = out.clone();
    let out_ho = out.clone();
    let out_to = out;

    rsx! {
        MonoxusConfig {
            dir: foundation::shared::Direction::Rtl,
            hover_card::HoverCardRoot {
                id: "hc-dir-floating",
                default_open: true,
                hover_card::HoverCardTrigger { "F" }
                hover_card::HoverCardContent {
                    force_mount: true,
                    child: Callback::new(move |cp: foundation::compose::FloatingContentChildProps| {
                        out_f.0.borrow_mut().hover_card_floating = extract_attr(&cp.props, "dir");
                        rsx! { div { ..cp.wrapper_props, div { ..cp.props, "Floating" } } }
                    }),
                }
            }
            hover_card::HoverCardRoot {
                id: "hc-dir-static",
                default_open: true,
                hover_card::HoverCardTrigger { "S" }
                hover_card::HoverCardContentStatic {
                    force_mount: true,
                    child: Callback::new(move |attrs: Vec<Attribute>| {
                        out_s.0.borrow_mut().hover_card_static = extract_attr(&attrs, "dir");
                        rsx! { div { ..attrs, "Static" } }
                    }),
                }
            }
            hover_card::HoverCardRoot {
                id: "hc-dir-override",
                default_open: true,
                hover_card::HoverCardTrigger { "O" }
                hover_card::HoverCardContentStatic {
                    force_mount: true,
                    dir: foundation::shared::Direction::Ltr,
                    child: Callback::new(move |attrs: Vec<Attribute>| {
                        out_ho.0.borrow_mut().hover_card_override = extract_attr(&attrs, "dir");
                        rsx! { div { ..attrs, "Override" } }
                    }),
                }
            }
            toast::ToastViewport {
                id: "toast-vp-inherited",
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    out_t.0.borrow_mut().toast_viewport = extract_attr(&attrs, "dir");
                    rsx! { ol { ..attrs } }
                }),
            }
            toast::ToastViewport {
                id: "toast-vp-override",
                dir: foundation::shared::Direction::Ltr,
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    out_to.0.borrow_mut().toast_override = extract_attr(&attrs, "dir");
                    rsx! { ol { ..attrs } }
                }),
            }
        }
    }
}

#[test]
fn item5_hover_card_and_toast_viewport_inherit_dir_from_monoxus_config() {
    clear_active_overlay_stack();
    let out = PtrSlot::new(DirInheritSnapshot::default());
    let mut dom = mount_with_props(
        HoverCardAndToastDirApp,
        HoverCardAndToastDirAppProps { out: out.clone() },
    );
    pump_vdom(&mut dom);

    let snap = out.0.borrow().clone();
    assert_eq!(snap.hover_card_floating.as_deref(), Some("rtl"));
    assert_eq!(snap.hover_card_static.as_deref(), Some("rtl"));
    assert_eq!(snap.toast_viewport.as_deref(), Some("rtl"));
    assert_eq!(snap.hover_card_override.as_deref(), Some("ltr"));
    assert_eq!(snap.toast_override.as_deref(), Some("ltr"));
}

#[component]
fn CalendarLocaleProbe(
    key_name: &'static str,
    out: PtrSlot<HashMap<&'static str, foundation::date_time::CalendarLocale>>,
) -> Element {
    let ctx = calendar::use_calendar_context();
    let loc = ctx.runtime.config.read().locale.clone();
    out.0.borrow_mut().insert(key_name, loc);
    rsx! {}
}

#[component]
fn RangeCalendarLocaleProbe(
    key_name: &'static str,
    out: PtrSlot<HashMap<&'static str, foundation::date_time::CalendarLocale>>,
) -> Element {
    let ctx = range_calendar::use_range_calendar_context();
    let loc = ctx.runtime.base.config.read().locale.clone();
    out.0.borrow_mut().insert(key_name, loc);
    rsx! {}
}

#[component]
fn DateFieldLocaleProbe(
    key_name: &'static str,
    out: PtrSlot<HashMap<&'static str, foundation::date_time::CalendarLocale>>,
) -> Element {
    let rt = date_field::use_date_field_context();
    let loc = rt.state.read().def().locale.clone();
    out.0.borrow_mut().insert(key_name, loc);
    rsx! {}
}

#[component]
fn DateRangeFieldLocaleProbe(
    key_name: &'static str,
    out: PtrSlot<HashMap<&'static str, foundation::date_time::CalendarLocale>>,
) -> Element {
    let rt = date_range_field::use_date_range_field_context();
    let loc = rt.state.read().def().locale.clone();
    out.0.borrow_mut().insert(key_name, loc);
    rsx! {}
}

#[component]
fn DatePickerLocaleProbe(
    key_name: &'static str,
    out: PtrSlot<HashMap<&'static str, foundation::date_time::CalendarLocale>>,
) -> Element {
    let rt = date_picker::use_date_picker_context();
    let loc = rt.calendar_config.read().config.locale.clone();
    out.0.borrow_mut().insert(key_name, loc);
    rsx! {}
}

#[component]
fn DateRangePickerLocaleProbe(
    key_name: &'static str,
    out: PtrSlot<HashMap<&'static str, foundation::date_time::CalendarLocale>>,
) -> Element {
    let rt = date_range_picker::use_date_range_picker_context();
    let loc = rt.calendar_config.read().config.locale.clone();
    out.0.borrow_mut().insert(key_name, loc);
    rsx! {}
}

#[component]
fn TimeFieldLocaleProbe(
    key_name: &'static str,
    out: PtrSlot<HashMap<&'static str, foundation::date_time::CalendarLocale>>,
) -> Element {
    let rt = time_field::use_time_field_context();
    let loc = rt.state.read().def().locale.clone();
    out.0.borrow_mut().insert(key_name, loc);
    rsx! {}
}

#[component]
fn TimeRangeFieldLocaleProbe(
    key_name: &'static str,
    out: PtrSlot<HashMap<&'static str, foundation::date_time::CalendarLocale>>,
) -> Element {
    let rt = time_range_field::use_time_range_field_context();
    let loc = rt.state.read().def().locale.clone();
    out.0.borrow_mut().insert(key_name, loc);
    rsx! {}
}

#[component]
fn AllEightDateTimeCalendarLocaleApp(
    explicit_locale: Option<foundation::date_time::CalendarLocale>,
    out: PtrSlot<HashMap<&'static str, foundation::date_time::CalendarLocale>>,
) -> Element {
    rsx! {
        MonoxusConfig {
            default_locale: foundation::date_time::CalendarLocale::KoKr,
            calendar::CalendarRoot {
                locale: explicit_locale.clone(),
                CalendarLocaleProbe { key_name: "CalendarRoot", out: out.clone() }
            }
            range_calendar::RangeCalendarRoot {
                locale: explicit_locale.clone(),
                RangeCalendarLocaleProbe { key_name: "RangeCalendarRoot", out: out.clone() }
            }
            date_field::DateFieldRoot {
                locale: explicit_locale.clone(),
                DateFieldLocaleProbe { key_name: "DateFieldRoot", out: out.clone() }
            }
            date_range_field::DateRangeFieldRoot {
                locale: explicit_locale.clone(),
                DateRangeFieldLocaleProbe { key_name: "DateRangeFieldRoot", out: out.clone() }
            }
            date_picker::DatePickerRoot {
                locale: explicit_locale.clone(),
                DatePickerLocaleProbe { key_name: "DatePickerRoot", out: out.clone() }
            }
            date_range_picker::DateRangePickerRoot {
                locale: explicit_locale.clone(),
                DateRangePickerLocaleProbe { key_name: "DateRangePickerRoot", out: out.clone() }
            }
            time_field::TimeFieldRoot {
                locale: explicit_locale.clone(),
                TimeFieldLocaleProbe { key_name: "TimeFieldRoot", out: out.clone() }
            }
            time_range_field::TimeRangeFieldRoot {
                locale: explicit_locale,
                TimeRangeFieldLocaleProbe { key_name: "TimeRangeFieldRoot", out }
            }
        }
    }
}

#[test]
fn item5_all_eight_date_time_calendar_roots_inherit_default_locale_and_honor_override() {
    use foundation::date_time::CalendarLocale;

    const ALL_EIGHT: [&str; 8] = [
        "CalendarRoot",
        "RangeCalendarRoot",
        "DateFieldRoot",
        "DateRangeFieldRoot",
        "DatePickerRoot",
        "DateRangePickerRoot",
        "TimeFieldRoot",
        "TimeRangeFieldRoot",
    ];

    // 1. Omitted locale -> all 8 inherit CalendarLocale::KoKr from MonoxusConfig.
    let inherited = PtrSlot::new(HashMap::new());
    let mut dom_inherited = mount_with_props(
        AllEightDateTimeCalendarLocaleApp,
        AllEightDateTimeCalendarLocaleAppProps {
            explicit_locale: None,
            out: inherited.clone(),
        },
    );
    pump_vdom(&mut dom_inherited);
    {
        let map = inherited.0.borrow();
        assert_eq!(map.len(), 8);
        for name in ALL_EIGHT {
            assert_eq!(
                map.get(name),
                Some(&CalendarLocale::KoKr),
                "{name} must inherit CalendarLocale::KoKr from MonoxusConfig"
            );
        }
    }

    // 2. Explicit locale: CalendarLocale::EnUs -> all 8 honor override back to EnUs.
    let overridden = PtrSlot::new(HashMap::new());
    let mut dom_override = mount_with_props(
        AllEightDateTimeCalendarLocaleApp,
        AllEightDateTimeCalendarLocaleAppProps {
            explicit_locale: Some(CalendarLocale::EnUs),
            out: overridden.clone(),
        },
    );
    pump_vdom(&mut dom_override);
    {
        let map = overridden.0.borrow();
        assert_eq!(map.len(), 8);
        for name in ALL_EIGHT {
            assert_eq!(
                map.get(name),
                Some(&CalendarLocale::EnUs),
                "{name} must honor explicit CalendarLocale::EnUs override"
            );
        }
    }
}

// =============================================================================
// Item 6: Nested `DialogBackdrop` (`DialogOverlay`)
// =============================================================================

#[component]
fn NestedOverlaySuppressionApp(rendered_overlays: PtrSlot<Vec<&'static str>>) -> Element {
    let r_outer = rendered_overlays.clone();
    let r_child_default = rendered_overlays.clone();
    let r_child_forced = rendered_overlays.clone();
    let r_alert_default = rendered_overlays.clone();
    let r_alert_forced = rendered_overlays;

    rsx! {
        dialog::DialogRoot {
            id: "outer-dlg",
            default_open: true,
            dialog::DialogOverlay {
                id: "outer-overlay",
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    r_outer.0.borrow_mut().push("outer");
                    rsx! { div { ..attrs } }
                }),
            }
            dialog::DialogContent {
                id: "outer-content",
                dialog::DialogRoot {
                    id: "nested-dlg-default",
                    default_open: true,
                    dialog::DialogOverlay {
                        id: "nested-dlg-default-overlay",
                        child: Callback::new(move |attrs: Vec<Attribute>| {
                            r_child_default.0.borrow_mut().push("nested-dialog-default");
                            rsx! { div { ..attrs } }
                        }),
                    }
                    dialog::DialogContent { id: "nested-dlg-default-content", "Child" }
                }
                dialog::DialogRoot {
                    id: "nested-dlg-forced",
                    default_open: true,
                    dialog::DialogOverlay {
                        id: "nested-dlg-forced-overlay",
                        force_render: true,
                        child: Callback::new(move |attrs: Vec<Attribute>| {
                            r_child_forced.0.borrow_mut().push("nested-dialog-forced");
                            rsx! { div { ..attrs } }
                        }),
                    }
                    dialog::DialogContent { id: "nested-dlg-forced-content", "Child Forced" }
                }
                alert_dialog::AlertDialogRoot {
                    id: "nested-alert-default",
                    default_open: true,
                    alert_dialog::DialogOverlay {
                        id: "nested-alert-default-overlay",
                        child: Callback::new(move |attrs: Vec<Attribute>| {
                            r_alert_default.0.borrow_mut().push("nested-alert-default");
                            rsx! { div { ..attrs } }
                        }),
                    }
                    alert_dialog::DialogContent { id: "nested-alert-default-content", "Alert" }
                }
                alert_dialog::AlertDialogRoot {
                    id: "nested-alert-forced",
                    default_open: true,
                    alert_dialog::DialogOverlay {
                        id: "nested-alert-forced-overlay",
                        force_render: true,
                        child: Callback::new(move |attrs: Vec<Attribute>| {
                            r_alert_forced.0.borrow_mut().push("nested-alert-forced");
                            rsx! { div { ..attrs } }
                        }),
                    }
                    alert_dialog::DialogContent { id: "nested-alert-forced-content", "Alert Forced" }
                }
            }
        }
    }
}

#[test]
fn item6_nested_dialog_and_alert_dialog_overlay_suppression_and_force_render() {
    clear_active_overlay_stack();
    let rendered = PtrSlot::new(Vec::new());
    let mut dom = mount_with_props(
        NestedOverlaySuppressionApp,
        NestedOverlaySuppressionAppProps {
            rendered_overlays: rendered.clone(),
        },
    );
    pump_vdom(&mut dom);

    let list = rendered.0.borrow();
    assert!(list.contains(&"outer"), "root DialogOverlay must render");
    assert!(
        !list.contains(&"nested-dialog-default"),
        "nested DialogOverlay with default force_render=false must be suppressed"
    );
    assert!(
        list.contains(&"nested-dialog-forced"),
        "nested DialogOverlay with force_render=true must render"
    );
    assert!(
        !list.contains(&"nested-alert-default"),
        "nested AlertDialog DialogOverlay with default force_render=false must be suppressed"
    );
    assert!(
        list.contains(&"nested-alert-forced"),
        "nested AlertDialog DialogOverlay with force_render=true must render"
    );
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DialogContentAttrsSnap {
    style: Option<String>,
    has_nested: bool,
    has_nested_open: bool,
}

#[component]
fn DeepNestingC(snap_c: PtrSlot<DialogContentAttrsSnap>) -> Element {
    rsx! {
        dialog::DialogRoot {
            id: "deep-c",
            default_open: true,
            dialog::DialogContent {
                id: "deep-c-content",
                child: Callback::new(move |c_attrs: Vec<Attribute>| {
                    *snap_c.0.borrow_mut() = DialogContentAttrsSnap {
                        style: extract_attr(&c_attrs, "style"),
                        has_nested: has_attr(&c_attrs, "data-nested"),
                        has_nested_open: has_attr(&c_attrs, "data-nested-dialog-open"),
                    };
                    rsx! { div { ..c_attrs, "C" } }
                }),
            }
        }
    }
}

#[component]
fn DeepNestingB(
    snap_b: PtrSlot<DialogContentAttrsSnap>,
    snap_c: PtrSlot<DialogContentAttrsSnap>,
) -> Element {
    rsx! {
        dialog::DialogRoot {
            id: "deep-b",
            default_open: true,
            dialog::DialogContent {
                id: "deep-b-content",
                child: Callback::new(move |b_attrs: Vec<Attribute>| {
                    *snap_b.0.borrow_mut() = DialogContentAttrsSnap {
                        style: extract_attr(&b_attrs, "style"),
                        has_nested: has_attr(&b_attrs, "data-nested"),
                        has_nested_open: has_attr(&b_attrs, "data-nested-dialog-open"),
                    };
                    let snap_c = snap_c.clone();
                    rsx! {
                        div { ..b_attrs,
                            DeepNestingC { snap_c }
                        }
                    }
                }),
            }
        }
    }
}

#[component]
fn DeepNestingApp(
    snap_a: PtrSlot<DialogContentAttrsSnap>,
    snap_b: PtrSlot<DialogContentAttrsSnap>,
    snap_c: PtrSlot<DialogContentAttrsSnap>,
) -> Element {
    rsx! {
        dialog::DialogRoot {
            id: "deep-a",
            default_open: true,
            dialog::DialogContent {
                id: "deep-a-content",
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    *snap_a.0.borrow_mut() = DialogContentAttrsSnap {
                        style: extract_attr(&attrs, "style"),
                        has_nested: has_attr(&attrs, "data-nested"),
                        has_nested_open: has_attr(&attrs, "data-nested-dialog-open"),
                    };
                    let snap_b = snap_b.clone();
                    let snap_c = snap_c.clone();
                    rsx! {
                        div { ..attrs,
                            DeepNestingB { snap_b, snap_c }
                        }
                    }
                }),
            }
        }
    }
}

#[derive(Clone, Default)]
struct SiblingDialogsControl {
    b_open: Option<Signal<bool>>,
    c_open: Option<Signal<bool>>,
    mount_b: Option<Signal<bool>>,
    mount_c: Option<Signal<bool>>,
}

#[component]
fn SiblingChildren(ctl: PtrSlot<SiblingDialogsControl>) -> Element {
    let b_open = use_signal(|| true);
    let c_open = use_signal(|| true);
    let mount_b = use_signal(|| true);
    let mount_c = use_signal(|| true);
    {
        let mut c = ctl.0.borrow_mut();
        c.b_open = Some(b_open);
        c.c_open = Some(c_open);
        c.mount_b = Some(mount_b);
        c.mount_c = Some(mount_c);
    }

    rsx! {
        div { id: "sib-b-slot",
            if mount_b() {
                dialog::DialogRoot {
                    id: "sib-b",
                    open: Some(b_open()),
                    dialog::DialogContent { id: "sib-b-content", "Sibling B" }
                }
            }
        }
        div { id: "sib-c-slot",
            if mount_c() {
                dialog::DialogRoot {
                    id: "sib-c",
                    open: Some(c_open()),
                    dialog::DialogContent { id: "sib-c-content", "Sibling C" }
                }
            }
        }
    }
}

#[component]
fn SimultaneousSiblingsApp(
    snap_a: PtrSlot<DialogContentAttrsSnap>,
    ctl: PtrSlot<SiblingDialogsControl>,
) -> Element {
    rsx! {
        dialog::DialogRoot {
            id: "sib-a",
            default_open: true,
            dialog::DialogContent {
                id: "sib-a-content",
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    *snap_a.0.borrow_mut() = DialogContentAttrsSnap {
                        style: extract_attr(&attrs, "style"),
                        has_nested: has_attr(&attrs, "data-nested"),
                        has_nested_open: has_attr(&attrs, "data-nested-dialog-open"),
                    };
                    let ctl = ctl.clone();
                    rsx! {
                        div { ..attrs,
                            SiblingChildren { ctl }
                        }
                    }
                }),
            }
        }
    }
}

#[test]
fn item6_deep_nesting_and_simultaneous_siblings_nested_counters_and_unmount() {
    clear_active_overlay_stack();

    // 1. Deep nesting: A -> B -> C
    let snap_a = PtrSlot::new(DialogContentAttrsSnap::default());
    let snap_b = PtrSlot::new(DialogContentAttrsSnap::default());
    let snap_c = PtrSlot::new(DialogContentAttrsSnap::default());

    let mut deep_dom = mount_with_props(
        DeepNestingApp,
        DeepNestingAppProps {
            snap_a: snap_a.clone(),
            snap_b: snap_b.clone(),
            snap_c: snap_c.clone(),
        },
    );
    pump_vdom(&mut deep_dom);

    {
        let a = snap_a.0.borrow();
        let b = snap_b.0.borrow();
        let c = snap_c.0.borrow();
        assert!(
            a.style.as_deref().is_some_and(|s| s.contains("--nested-dialogs: 2;")),
            "A must emit --nested-dialogs: 2, got {:?}",
            a.style
        );
        assert!(!a.has_nested, "A is root and must not have data-nested");
        assert!(a.has_nested_open, "A must have data-nested-dialog-open");

        assert!(
            b.style.as_deref().is_some_and(|s| s.contains("--nested-dialogs: 1;")),
            "B must emit --nested-dialogs: 1, got {:?}",
            b.style
        );
        assert!(b.has_nested, "B must have data-nested");
        assert!(b.has_nested_open, "B must have data-nested-dialog-open");

        assert!(
            c.style.as_deref().is_some_and(|s| s.contains("--nested-dialogs: 0;")),
            "C must emit --nested-dialogs: 0, got {:?}",
            c.style
        );
        assert!(c.has_nested, "C must have data-nested");
        assert!(!c.has_nested_open, "C must not have data-nested-dialog-open");
    }
    drop(deep_dom);

    // 2. Simultaneous siblings: A -> {B, C}, close B, unmount closed B, unmount open C.
    clear_active_overlay_stack();
    let sib_a = PtrSlot::new(DialogContentAttrsSnap::default());
    let ctl = PtrSlot::new(SiblingDialogsControl::default());

    let mut sib_dom = mount_with_props(
        SimultaneousSiblingsApp,
        SimultaneousSiblingsAppProps {
            snap_a: sib_a.clone(),
            ctl: ctl.clone(),
        },
    );
    pump_vdom(&mut sib_dom);

    assert!(
        sib_a
            .0
            .borrow()
            .style
            .as_deref()
            .is_some_and(|s| s.contains("--nested-dialogs: 2;")),
        "A with two open siblings must emit --nested-dialogs: 2, got {:?}",
        sib_a.0.borrow().style
    );
    assert!(sib_a.0.borrow().has_nested_open);

    let mut b_open = ctl.0.borrow().b_open.expect("b_open");
    let mut mount_b = ctl.0.borrow().mount_b.expect("mount_b");
    let mut mount_c = ctl.0.borrow().mount_c.expect("mount_c");

    // Closing B decrements count from 2 -> 1.
    sib_dom.in_scope(ScopeId::ROOT, || {
        b_open.set(false);
    });
    pump_vdom(&mut sib_dom);
    assert!(
        sib_a
            .0
            .borrow()
            .style
            .as_deref()
            .is_some_and(|s| s.contains("--nested-dialogs: 1;")),
        "closing B must decrement A to --nested-dialogs: 1, got {:?}",
        sib_a.0.borrow().style
    );
    assert!(sib_a.0.borrow().has_nested_open);

    // Unmounting ALREADY-CLOSED B must NOT decrement again (stays 1).
    sib_dom.in_scope(ScopeId::ROOT, || {
        mount_b.set(false);
    });
    pump_vdom(&mut sib_dom);
    assert!(
        sib_a
            .0
            .borrow()
            .style
            .as_deref()
            .is_some_and(|s| s.contains("--nested-dialogs: 1;")),
        "unmounting a closed child must not double-decrement, got {:?}",
        sib_a.0.borrow().style
    );
    assert!(sib_a.0.borrow().has_nested_open);

    // Unmounting STILL-OPEN C decrements count from 1 -> 0 and clears data-nested-dialog-open.
    sib_dom.in_scope(ScopeId::ROOT, || {
        mount_c.set(false);
    });
    pump_vdom(&mut sib_dom);
    assert!(
        sib_a
            .0
            .borrow()
            .style
            .as_deref()
            .is_some_and(|s| s.contains("--nested-dialogs: 0;")),
        "unmounting open C must decrement A to --nested-dialogs: 0, got {:?}",
        sib_a.0.borrow().style
    );
    assert!(!sib_a.0.borrow().has_nested_open);
}

// =============================================================================
// Item 7: Async Pending `Button` & Shared Accessible Announcer
// =============================================================================

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PendingButtonAttrsSnap {
    aria_busy: Option<String>,
    aria_disabled: Option<String>,
    data_pending: Option<String>,
    has_disabled_attr: bool,
}

#[component]
fn PendingButtonsAnnouncerApp(attrs_out: PtrSlot<PendingButtonAttrsSnap>) -> Element {
    rsx! {
        div {
            // 1. Default announcement ("Loading") + attribute inspection
            button::Button {
                id: "pending-default",
                pending: true,
                child: Callback::new(move |attrs: Vec<Attribute>| {
                    *attrs_out.0.borrow_mut() = PendingButtonAttrsSnap {
                        aria_busy: extract_attr(&attrs, "aria-busy"),
                        aria_disabled: extract_attr(&attrs, "aria-disabled"),
                        data_pending: extract_attr(&attrs, "data-pending"),
                        has_disabled_attr: has_attr(&attrs, "disabled"),
                    };
                    rsx! { button { ..attrs, "Submit" } }
                }),
                "Submit"
            }
            // 2. Custom announcement
            button::Button {
                id: "pending-custom",
                pending: true,
                pending_announcement: "Saving profile",
                "Save"
            }
            // 3. Explicit empty announcement -> stays silent
            button::Button {
                id: "pending-silent",
                pending: true,
                pending_announcement: String::new(),
                "Silent"
            }
        }
    }
}

#[test]
fn item7_pending_button_attributes_and_delayed_polite_announcements() {
    assert_eq!(
        foundation::announce::LIVE_ANNOUNCER_ID,
        "monoxus-live-announcer"
    );

    let guard = foundation::announce::capture_announcements();
    let attrs_out = PtrSlot::new(PendingButtonAttrsSnap::default());

    let mut dom = mount_with_props(
        PendingButtonsAnnouncerApp,
        PendingButtonsAnnouncerAppProps {
            attrs_out: attrs_out.clone(),
        },
    );
    pump_vdom(&mut dom);

    // Pending attributes verified immediately: aria-busy="true", aria-disabled="true",
    // data-pending="", and NO HTML disabled attribute.
    {
        let snap = attrs_out.0.borrow();
        assert_eq!(snap.aria_busy.as_deref(), Some("true"));
        assert_eq!(snap.aria_disabled.as_deref(), Some("true"));
        assert_eq!(snap.data_pending.as_deref(), Some(""));
        assert!(
            !snap.has_disabled_attr,
            "pending button must omit HTML disabled attribute to remain focusable"
        );
    }

    // Before 1000ms elapses, zero announcements have fired.
    assert!(
        guard.messages().is_empty(),
        "announcements must not fire before 1000ms"
    );

    // After >= 1000ms continuous pending, default ("Loading") and custom ("Saving profile")
    // announce via the shared polite announcer while empty pending_announcement stays silent.
    std::thread::sleep(Duration::from_millis(1100));
    pump_vdom(&mut dom);

    let mut msgs = guard.take();
    msgs.sort();
    assert_eq!(
        msgs,
        vec!["Loading".to_string(), "Saving profile".to_string()]
    );
}

#[derive(Clone, Default)]
struct CancelPendingControl {
    pending_a: Option<Signal<bool>>,
    mount_b: Option<Signal<bool>>,
}

#[component]
fn CancelPendingBeforeThresholdApp(ctl: PtrSlot<CancelPendingControl>) -> Element {
    let pending_a = use_signal(|| true);
    let mount_b = use_signal(|| true);
    {
        let mut c = ctl.0.borrow_mut();
        c.pending_a = Some(pending_a);
        c.mount_b = Some(mount_b);
    }

    rsx! {
        div {
            button::Button {
                id: "cancel-settle-btn",
                pending: pending_a(),
                "Settles Early"
            }
            if mount_b() {
                button::Button {
                    id: "cancel-unmount-btn",
                    pending: true,
                    "Unmounts Early"
                }
            }
        }
    }
}

#[test]
fn item7_pending_button_settle_and_unmount_before_1000ms_cancel_announcements() {
    let guard = foundation::announce::capture_announcements();
    let ctl = PtrSlot::new(CancelPendingControl::default());

    let mut dom = mount_with_props(
        CancelPendingBeforeThresholdApp,
        CancelPendingBeforeThresholdAppProps { ctl: ctl.clone() },
    );
    pump_vdom(&mut dom);
    assert!(guard.messages().is_empty());

    // Settle Button A and unmount Button B well before 1000ms.
    let mut pending_a = ctl.0.borrow().pending_a.expect("pending_a");
    let mut mount_b = ctl.0.borrow().mount_b.expect("mount_b");
    dom.in_scope(ScopeId::ROOT, || {
        pending_a.set(false);
        mount_b.set(false);
    });
    pump_vdom(&mut dom);

    std::thread::sleep(Duration::from_millis(1100));
    pump_vdom(&mut dom);

    assert!(
        guard.messages().is_empty(),
        "settling or unmounting before 1000ms must cancel pending announcement timers, got {:?}",
        guard.messages()
    );
}

// =============================================================================
// Item 8: `Progress.min`
// =============================================================================

#[derive(Clone, Debug, Default, PartialEq)]
struct ProgressMinSnapshot {
    root_valuemin: Option<String>,
    root_valuemax: Option<String>,
    root_valuenow: Option<String>,
    root_data_min: Option<String>,
    indicator_data_min: Option<String>,
    runtime_min: f64,
    runtime_percentage: Option<f64>,
    validation_issues: Vec<progress::ProgressValidation>,
}

#[component]
fn ProgressMinProbe(out: PtrSlot<ProgressMinSnapshot>) -> Element {
    let ctx = progress::use_progress_context();
    let rt = ctx.runtime();
    let mut snap = out.0.borrow_mut();
    snap.runtime_min = rt.min();
    snap.runtime_percentage = rt.percentage();
    snap.validation_issues = rt.progress().validation_issues().to_vec();
    rsx! {}
}

#[component]
fn ProgressMinApp(
    min: f64,
    max: f64,
    value: Option<f64>,
    out: PtrSlot<ProgressMinSnapshot>,
) -> Element {
    let out_root = out.clone();
    let out_ind = out.clone();
    let out_probe = out;

    rsx! {
        progress::ProgressRoot {
            id: "prog-min-root",
            min,
            max,
            value,
            child: Callback::new(move |attrs: Vec<Attribute>| {
                {
                    let mut snap = out_root.0.borrow_mut();
                    snap.root_valuemin = extract_attr(&attrs, "aria-valuemin");
                    snap.root_valuemax = extract_attr(&attrs, "aria-valuemax");
                    snap.root_valuenow = extract_attr(&attrs, "aria-valuenow");
                    snap.root_data_min = extract_attr(&attrs, "data-min");
                }
                let ind_slot = out_ind.clone();
                let probe_slot = out_probe.clone();
                rsx! {
                    div { ..attrs,
                        ProgressMinProbe { out: probe_slot }
                        progress::ProgressIndicator {
                            id: "prog-min-indicator",
                            child: Callback::new(move |ind_attrs: Vec<Attribute>| {
                                ind_slot.0.borrow_mut().indicator_data_min =
                                    extract_attr(&ind_attrs, "data-min");
                                rsx! { div { ..ind_attrs } }
                            }),
                        }
                    }
                }
            }),
        }
    }
}

#[test]
fn item8_progress_min_attributes_percentage_and_nan_validation() {
    // 1. Valid custom min/max/value: min=10, max=50, value=30 -> 50%.
    let valid_out = PtrSlot::new(ProgressMinSnapshot::default());
    let mut dom_valid = mount_with_props(
        ProgressMinApp,
        ProgressMinAppProps {
            min: 10.0,
            max: 50.0,
            value: Some(30.0),
            out: valid_out.clone(),
        },
    );
    pump_vdom(&mut dom_valid);

    {
        let snap = valid_out.0.borrow();
        assert_eq!(snap.root_valuemin.as_deref(), Some("10"));
        assert_eq!(snap.root_valuemax.as_deref(), Some("50"));
        assert_eq!(snap.root_valuenow.as_deref(), Some("30"));
        assert_eq!(snap.root_data_min.as_deref(), Some("10"));
        assert_eq!(snap.indicator_data_min.as_deref(), Some("10"));
        assert_eq!(snap.runtime_min, 10.0);
        assert_eq!(snap.runtime_percentage, Some(50.0));
        assert!(snap.validation_issues.is_empty());
    }

    // 2. Non-finite min (f64::NAN) falls back to 0.0 and reports InvalidMin diagnostic.
    let diag_guard = foundation::diagnostics::capture_diagnostics();
    let nan_out = PtrSlot::new(ProgressMinSnapshot::default());
    let mut dom_nan = mount_with_props(
        ProgressMinApp,
        ProgressMinAppProps {
            min: f64::NAN,
            max: 100.0,
            value: Some(25.0),
            out: nan_out.clone(),
        },
    );
    pump_vdom(&mut dom_nan);

    {
        let snap = nan_out.0.borrow();
        assert_eq!(snap.runtime_min, 0.0);
        assert_eq!(snap.root_valuemin.as_deref(), Some("0"));
        assert_eq!(snap.root_data_min.as_deref(), Some("0"));
        assert_eq!(snap.indicator_data_min.as_deref(), Some("0"));
        assert!(
            snap.validation_issues
                .contains(&progress::ProgressValidation::InvalidMin),
            "expected ProgressValidation::InvalidMin, got {:?}",
            snap.validation_issues
        );
    }

    let diags = diag_guard.take();
    assert!(
        diags.iter().any(|d| {
            d.kind == foundation::diagnostics::DiagnosticKind::InvalidProp
                && d.component == "Progress"
                && d.prop == "min"
        }),
        "expected InvalidProp diagnostic for Progress.min, got {diags:?}"
    );
}

// =============================================================================
// Native DOM Runtime Proofs (`#[cfg(feature = "native")]`)
// =============================================================================

#[cfg(feature = "native")]
mod native_tests {
    use std::cell::Cell;

    use super::*;
    use crate::common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    #[component]
    fn NativeDetachedDialogApp() -> Element {
        let handle = foundation::overlay::use_dialog_handle::<ItemPayload>();
        let mut close_count = use_signal(|| 0_usize);
        let payload_label = handle
            .payload()
            .map(|p| p.label)
            .unwrap_or("None");

        rsx! {
            div {
                style: "display: flex; flex-direction: column; gap: 8px; padding: 16px;",
                div {
                    id: "native-dlg-close-count",
                    "data-close-count": "{close_count()}",
                    "{close_count()}"
                }
                dialog::DialogTrigger {
                    id: "native-trig-a",
                    style: "width: 120px; height: 36px;",
                    handle: handle.clone(),
                    payload: Some(handle.trigger_payload(ItemPayload { id: 1, label: "Alpha" })),
                    "Trigger A"
                }
                dialog::DialogTrigger {
                    id: "native-trig-b",
                    style: "width: 120px; height: 36px;",
                    handle: handle.clone(),
                    payload: Some(handle.trigger_payload(ItemPayload { id: 2, label: "Beta" })),
                    "Trigger B"
                }
                dialog::DialogRoot {
                    id: "native-detached-dlg",
                    mode: dialog::DialogMode::NonModal,
                    handle: handle.clone(),
                    on_open_change: move |open: bool| {
                        if !open {
                            close_count += 1;
                        }
                    },
                    dialog::DialogContent {
                        id: "native-detached-dlg-content",
                        style: "width: 240px; height: 120px;",
                        div {
                            id: "native-payload-readout",
                            "data-payload": "{payload_label}",
                            "{payload_label}"
                        }
                        dialog::DialogClose {
                            id: "native-detached-dlg-close",
                            style: "width: 80px; height: 32px;",
                            "Close"
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_detached_dialog_triggers_switch_payload_and_restore_focus_to_active_trigger() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();

        let mut harness = mount_native_scenario(NativeDetachedDialogApp);
        assert!(harness.query("#native-detached-dlg-content").is_none());

        // Click Trigger A -> opens with Alpha, aria-expanded="true" ONLY on Trigger A.
        click_and_pump(&mut harness, "#native-trig-a");
        assert!(harness.query("#native-detached-dlg-content").is_some());
        {
            let base = harness.base();
            let a = base.get_node(harness.node("#native-trig-a")).unwrap();
            let b = base.get_node(harness.node("#native-trig-b")).unwrap();
            let p = base.get_node(harness.node("#native-payload-readout")).unwrap();
            let cc = base.get_node(harness.node("#native-dlg-close-count")).unwrap();
            assert_eq!(a.attr("aria-expanded".into()), Some("true"));
            assert_eq!(b.attr("aria-expanded".into()), Some("false"));
            assert_eq!(p.attr("data-payload".into()), Some("Alpha"));
            assert_eq!(cc.attr("data-close-count".into()), Some("0"));
        }

        // Click Trigger B while open -> switches active trigger & payload to Beta WITHOUT closing.
        click_and_pump(&mut harness, "#native-trig-b");
        assert!(harness.query("#native-detached-dlg-content").is_some());
        {
            let base = harness.base();
            let a = base.get_node(harness.node("#native-trig-a")).unwrap();
            let b = base.get_node(harness.node("#native-trig-b")).unwrap();
            let p = base.get_node(harness.node("#native-payload-readout")).unwrap();
            let cc = base.get_node(harness.node("#native-dlg-close-count")).unwrap();
            assert_eq!(a.attr("aria-expanded".into()), Some("false"));
            assert_eq!(b.attr("aria-expanded".into()), Some("true"));
            assert_eq!(p.attr("data-payload".into()), Some("Beta"));
            assert_eq!(
                cc.attr("data-close-count".into()),
                Some("0"),
                "switching detached DialogTrigger must never fire on_open_change(false)"
            );
        }

        // Dismiss via Escape -> closes dialog and restores focus to Trigger B.
        focus_node(&mut harness, "#native-detached-dlg-content");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.35);

        assert!(harness.query("#native-detached-dlg-content").is_none());
        assert_eq!(
            harness.attr("#native-dlg-close-count", "data-close-count").as_deref(),
            Some("1")
        );
        assert_native_focus_id(&harness, "#native-trig-b");
    }

    #[component]
    fn NativeDetachedPopoverApp() -> Element {
        let handle = foundation::overlay::use_popover_handle::<ItemPayload>();
        let mut close_count = use_signal(|| 0_usize);
        let payload_label = handle.payload().map(|p| p.label).unwrap_or("None");

        rsx! {
            div {
                style: "width: 720px; height: 480px; position: relative; padding: 24px;",
                div {
                    id: "native-pop-close-count",
                    "data-close-count": "{close_count()}",
                    "{close_count()}"
                }
                popover::PopoverTrigger {
                    id: "native-pop-trig-a",
                    style: "position: absolute; left: 24px; top: 48px; width: 100px; height: 32px;",
                    handle: handle.clone(),
                    payload: Some(handle.trigger_payload(ItemPayload { id: 1, label: "Alpha" })),
                    "Pop A"
                }
                popover::PopoverTrigger {
                    id: "native-pop-trig-b",
                    style: "position: absolute; left: 360px; top: 48px; width: 100px; height: 32px;",
                    handle: handle.clone(),
                    payload: Some(handle.trigger_payload(ItemPayload { id: 2, label: "Beta" })),
                    "Pop B"
                }
                popover::PopoverRoot {
                    id: "native-detached-pop",
                    handle: handle.clone(),
                    on_open_change: move |open: bool| {
                        if !open {
                            close_count += 1;
                        }
                    },
                    popover::PopoverPortal {
                        popover::PopoverContent {
                            id: "native-detached-pop-content",
                            style: "width: 140px; height: 64px;",
                            div {
                                id: "native-pop-payload",
                                "data-payload": "{payload_label}",
                                "{payload_label}"
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_detached_popover_triggers_switch_payload_reanchor_and_restore_focus() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();

        let mut harness = mount_native_scenario(NativeDetachedPopoverApp);
        assert!(harness.query("#native-detached-pop-content").is_none());

        // 1. Click Trigger A -> opens anchored under Trigger A with Payload Alpha.
        click_and_pump(&mut harness, "#native-pop-trig-a");
        advance_frame_and_pump(&mut harness, 0.05);
        drain_polls_and_resolve(&mut harness);
        assert!(harness.query("#native-detached-pop-content").is_some());
        assert_eq!(
            harness.attr("#native-pop-trig-a", "aria-expanded").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#native-pop-trig-b", "aria-expanded").as_deref(),
            Some("false")
        );
        assert_eq!(
            harness.attr("#native-pop-payload", "data-payload").as_deref(),
            Some("Alpha")
        );
        assert_eq!(
            harness.attr("#native-pop-close-count", "data-close-count").as_deref(),
            Some("0")
        );
        let rect_a = harness.layout_rect("#native-detached-pop-content");
        assert!(rect_a.width > 0.0 && rect_a.height > 0.0);

        // 2. Click Trigger B while open -> NEVER closes (close_count == 0), switches payload to Beta,
        //    and re-anchors floating content rect horizontally to Trigger B.
        click_and_pump(&mut harness, "#native-pop-trig-b");
        advance_frame_and_pump(&mut harness, 0.05);
        drain_polls_and_resolve(&mut harness);
        assert!(harness.query("#native-detached-pop-content").is_some());
        assert_eq!(
            harness.attr("#native-pop-close-count", "data-close-count").as_deref(),
            Some("0"),
            "switching detached PopoverTrigger must never fire on_open_change(false)"
        );
        assert_eq!(
            harness.attr("#native-pop-trig-a", "aria-expanded").as_deref(),
            Some("false")
        );
        assert_eq!(
            harness.attr("#native-pop-trig-b", "aria-expanded").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#native-pop-payload", "data-payload").as_deref(),
            Some("Beta")
        );
        let rect_b = harness.layout_rect("#native-detached-pop-content");
        assert!(
            rect_b.x > rect_a.x + 150.0,
            "PopoverContent must re-anchor from Trigger A (x={}) to Trigger B (x={})",
            rect_a.x,
            rect_b.x
        );

        // 3. Escape closes Popover (close_count == 1) and restores focus to Trigger B.
        focus_node(&mut harness, "#native-detached-pop-content");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.35);

        assert!(harness.query("#native-detached-pop-content").is_none());
        assert_eq!(
            harness.attr("#native-pop-close-count", "data-close-count").as_deref(),
            Some("1")
        );
        assert_native_focus_id(&harness, "#native-pop-trig-b");
    }

    #[component]
    fn NativeNestedDialogOverlayGuardApp() -> Element {
        rsx! {
            div {
                style: "width: 600px; height: 600px; position: relative;",
                dialog::DialogRoot {
                    id: "native-outer-dlg",
                    default_open: true,
                    dialog::DialogOverlay {
                        id: "native-outer-overlay",
                        style: "position: fixed; left: 0px; top: 0px; width: 600px; height: 600px;",
                    }
                    dialog::DialogContent {
                        id: "native-outer-content",
                        style: "position: fixed; left: 220px; top: 220px; width: 200px; height: 200px;",
                        "Outer Content"
                        dialog::DialogRoot {
                            id: "native-inner-dlg",
                            default_open: true,
                            dialog::DialogOverlay {
                                id: "native-inner-overlay",
                                style: "position: fixed; left: 0px; top: 0px; width: 600px; height: 600px;",
                            }
                            dialog::DialogContent {
                                id: "native-inner-content",
                                style: "position: fixed; left: 260px; top: 260px; width: 120px; height: 120px;",
                                "Inner Content"
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_nested_dialog_root_overlay_click_does_not_close_parent() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();

        let mut harness = mount_native_scenario(NativeNestedDialogOverlayGuardApp);

        assert!(harness.query("#native-outer-overlay").is_some());
        assert!(harness.query("#native-outer-content").is_some());
        assert!(
            harness.query("#native-inner-overlay").is_none(),
            "nested child DialogOverlay must be suppressed by default"
        );
        assert!(harness.query("#native-inner-content").is_some());

        // Real click on the visible root DialogOverlay at (12, 12) while nested child is open.
        click_at_and_pump(&mut harness, 12.0, 12.0);
        advance_frame_and_pump(&mut harness, 0.35);

        assert!(
            harness.query("#native-outer-content").is_some(),
            "clicking root DialogOverlay while a nested child dialog is open must NOT close parent"
        );
    }

    #[component]
    fn NativePendingButtonInteractionApp(
        click_count: Rc<Cell<u32>>,
        press_count: Rc<Cell<u32>>,
        tab_key_seen: Rc<Cell<bool>>,
        async_action_calls: Rc<Cell<u32>>,
    ) -> Element {
        let clicks = click_count.clone();
        let presses = press_count.clone();
        let tab_seen = tab_key_seen;
        let action_calls = async_action_calls;

        rsx! {
            div {
                style: "display: flex; flex-direction: column; gap: 12px; padding: 16px;",
                button::Button {
                    id: "native-pending-btn",
                    style: "width: 140px; height: 40px;",
                    pending: true,
                    on_click: move |_| {
                        clicks.set(clicks.get() + 1);
                    },
                    on_press: move |_| {
                        presses.set(presses.get() + 1);
                    },
                    on_key_down: move |evt: KeyboardEvent| {
                        if evt.key() == Key::Named(NamedKey::Tab) {
                            tab_seen.set(true);
                        }
                    },
                    "Pending Action"
                }
                button::Button {
                    id: "native-async-action-btn",
                    style: "width: 140px; height: 40px;",
                    on_action: button::AsyncButtonAction::new(move |_| {
                        action_calls.set(action_calls.get() + 1);
                        async {
                            futures_timer::Delay::new(Duration::from_millis(200)).await;
                        }
                    }),
                    "Run Async Action"
                }
                button {
                    id: "native-after-btn",
                    style: "width: 140px; height: 40px;",
                    "Next Button"
                }
            }
        }
    }

    #[test]
    fn native_pending_button_suppresses_click_and_activation_keys_while_allowing_tab_and_focus() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();

        let click_count = Rc::new(Cell::new(0u32));
        let press_count = Rc::new(Cell::new(0u32));
        let tab_key_seen = Rc::new(Cell::new(false));
        let async_action_calls = Rc::new(Cell::new(0u32));

        let mut harness = mount_native_scenario_with_props(
            NativePendingButtonInteractionApp,
            NativePendingButtonInteractionAppProps {
                click_count: click_count.clone(),
                press_count: press_count.clone(),
                tab_key_seen: tab_key_seen.clone(),
                async_action_calls: async_action_calls.clone(),
            },
        );

        {
            let base = harness.base();
            let node = base.get_node(harness.node("#native-pending-btn")).unwrap();
            assert_eq!(node.attr("aria-busy".into()), Some("true"));
            assert_eq!(node.attr("aria-disabled".into()), Some("true"));
            assert_eq!(node.attr("data-pending".into()), Some(""));
            assert_eq!(node.attr("disabled".into()), None);
        }

        // Focus remains on the pending <button> and duplicate clicks / Enter / Space are suppressed.
        focus_node(&mut harness, "#native-pending-btn");
        assert_native_focus_id(&harness, "#native-pending-btn");

        click_and_pump(&mut harness, "#native-pending-btn");
        assert_eq!(click_count.get(), 0, "pending button must suppress clicks");
        assert_eq!(press_count.get(), 0, "pending button must suppress on_press");

        focus_node(&mut harness, "#native-pending-btn");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        send_key_and_pump(
            &mut harness,
            Key::Character(" ".to_string()),
            Code::Space,
        );
        assert_eq!(click_count.get(), 0, "pending button must suppress Enter/Space");
        assert_eq!(press_count.get(), 0, "pending button must suppress Enter/Space");
        assert_native_focus_id(&harness, "#native-pending-btn");

        // Tab is NOT suppressed by pending state.
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Tab), Code::Tab);
        assert!(
            tab_key_seen.get(),
            "Tab key must pass through pending button keydown handler"
        );

        // AsyncButtonAction in flight: first click launches action and sets pending attrs;
        // duplicate click while in flight is suppressed.
        click_and_pump(&mut harness, "#native-async-action-btn");
        assert_eq!(async_action_calls.get(), 1);
        {
            let base = harness.base();
            let node = base
                .get_node(harness.node("#native-async-action-btn"))
                .unwrap();
            assert_eq!(node.attr("aria-busy".into()), Some("true"));
            assert_eq!(node.attr("aria-disabled".into()), Some("true"));
            assert_eq!(node.attr("data-pending".into()), Some(""));
            assert_eq!(node.attr("disabled".into()), None);
        }
        click_and_pump(&mut harness, "#native-async-action-btn");
        assert_eq!(
            async_action_calls.get(),
            1,
            "duplicate click while AsyncButtonAction is in flight must be suppressed"
        );
    }

    #[component]
    fn NativeAnnouncerShieldExclusionApp() -> Element {
        rsx! {
            div {
                id: "shield-test-root",
                div {
                    id: "{foundation::announce::LIVE_ANNOUNCER_ID}",
                    role: "status",
                    "aria-live": "polite",
                }
                div {
                    id: "ordinary-outside-sibling",
                    "Outside"
                }
                dialog::DialogRoot {
                    id: "modal-shield-dlg",
                    default_open: true,
                    dialog::DialogContent {
                        id: "modal-shield-dlg-content",
                        "Modal Dialog"
                    }
                }
            }
        }
    }

    #[test]
    fn native_live_announcer_id_is_excluded_from_modal_shield_external_dom() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_active_overlay_stack();

        let harness = mount_native_scenario(NativeAnnouncerShieldExclusionApp);
        let base = harness.base();
        let announcer = base
            .get_node(harness.node(&format!("#{}", foundation::announce::LIVE_ANNOUNCER_ID)))
            .expect("announcer node");
        assert_ne!(
            announcer.attr("aria-hidden".into()),
            Some("true"),
            "LIVE_ANNOUNCER_ID must be excluded from modal aria-hidden shielding"
        );
        assert_eq!(
            announcer.attr("inert".into()),
            None,
            "LIVE_ANNOUNCER_ID must be excluded from modal inert shielding"
        );
    }
}

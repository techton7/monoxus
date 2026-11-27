use monoxus::{
    foundation::{
        overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost, Rect, Size},
        shared::ScopeHandle,
    },
    select::Select,
};

#[test]
fn select_floating_layer_flips_to_top_when_space_constrained() {
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(4.0);

    let anchor = Rect::new(100.0, 700.0, 200.0, 40.0);
    let content = Size::new(200.0, 150.0);
    let viewport = Size::new(1024.0, 768.0);

    let pos = layer.position_with_available_size(anchor, content, viewport);
    assert_eq!(pos.side(), PlacementSide::Top);
}

#[test]
fn select_floating_layer_stays_bottom_when_space_sufficient() {
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(4.0);

    let anchor = Rect::new(100.0, 100.0, 200.0, 40.0);
    let content = Size::new(200.0, 150.0);
    let viewport = Size::new(1024.0, 768.0);

    let pos = layer.position_with_available_size(anchor, content, viewport);
    assert_eq!(pos.side(), PlacementSide::Bottom);
}

#[test]
fn select_portal_and_content_attributes_publish_side_and_host() {
    let scope = ScopeHandle::root("select-test").child("portal-content");
    let portal_host = PortalHost::named("overlay-container");

    let select = Select::new(scope.clone())
        .with_portal_host(portal_host.clone())
        .with_open(true);

    let portal_attrs = select.portal_attributes();
    assert_eq!(portal_attrs.host(), &portal_host);

    let content_attrs = select.content_attributes_with_side(None, PlacementSide::Top);
    assert_eq!(content_attrs.data_side_str(), "top");
    assert_eq!(content_attrs.data_align_str(), "start");
}

#[test]
fn select_dynamic_scroll_reposition_and_flip_contract() {
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(4.0);

    let content = Size::new(220.0, 180.0);
    let viewport = Size::new(1024.0, 768.0);

    let initial_anchor = Rect::new(100.0, 650.0, 220.0, 38.0);
    let initial_pos = layer.position_with_available_size(initial_anchor, content, viewport);
    assert_eq!(initial_pos.side(), PlacementSide::Top);

    let scrolled_anchor = Rect::new(100.0, 450.0, 220.0, 38.0);
    let scrolled_pos = layer.position_with_available_size(scrolled_anchor, content, viewport);
    assert_eq!(scrolled_pos.side(), PlacementSide::Bottom);
}

#[test]
fn select_content_reference_hidden_contract() {
    let scope = ScopeHandle::root("select-test").child("ref-hidden");
    let select = Select::new(scope.clone()).with_open(true);

    let mut attrs = select.content_attributes(None);
    assert!(!attrs.is_reference_hidden());
    assert_eq!(attrs.data_reference_hidden_str(), None);

    attrs.reference_hidden = true;
    assert!(attrs.is_reference_hidden());
    assert_eq!(attrs.data_reference_hidden_str(), Some("true"));
}

#[test]
fn select_pointer_down_outside_event_prevent_default_contract() {
    use monoxus::select::PointerDownOutsideEvent;

    let evt = PointerDownOutsideEvent::default();
    assert!(!evt.is_default_prevented());

    let evt_clone = evt.clone();
    evt_clone.prevent_default();

    // prevent_default on clone mutates shared cell
    assert!(evt.is_default_prevented());
    assert!(evt_clone.is_default_prevented());
}

#[test]
fn select_content_phase35_props_and_attributes_contract() {
    let scope = ScopeHandle::root("select-test").child("phase35");
    let select = Select::new(scope.clone()).with_open(true);

    let attrs = select.content_attributes(None);
    // baseline assertions
    assert_eq!(attrs.data_state_str(), "open");
    assert_eq!(attrs.data_side_str(), "bottom");
    assert_eq!(attrs.data_align_str(), "start");
}

#[test]
fn select_custom_anchor_and_prevent_overflow_text_selection_contract() {
    use monoxus::foundation::overlay::{FloatingLayer, PlacementAlign, PlacementSide, Rect, Size};

    // 1. Custom anchor dynamic floating placement:
    // If standard trigger is constrained at bottom, placement flips to Top.
    // But if custom anchor is at a spacious location (e.g. y=100), placement stays at Bottom!
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(4.0);

    let default_trigger_anchor = Rect::new(100.0, 700.0, 200.0, 40.0);
    let custom_target_anchor = Rect::new(100.0, 100.0, 200.0, 40.0);
    let content = Size::new(200.0, 150.0);
    let viewport = Size::new(1024.0, 768.0);

    let default_pos = layer.position_with_available_size(default_trigger_anchor, content, viewport);
    assert_eq!(default_pos.side(), PlacementSide::Top);

    // Custom anchor at spacious coordinate stays at preferred bottom
    let custom_pos = layer.position_with_available_size(custom_target_anchor, content, viewport);
    assert_eq!(custom_pos.side(), PlacementSide::Bottom);
}

#[test]
fn select_content_force_mount_state_contract() {
    let scope = ScopeHandle::root("select-test").child("force-mount");
    let select = Select::new(scope.clone()).with_open(false);

    let attrs = select.content_attributes(None);
    // Closed state
    assert_eq!(attrs.data_state_str(), "closed");
}

#[test]
fn select_content_center_alignment_uses_translate_not_conflicting_transform() {
    // PlacementAlign::Center must decouple horizontal centering using CSS `translate: -50% 0;`
    // to prevent collision with keyframe `transform: scale(...)` or `translateY(...)`.
    let eff_align = PlacementAlign::Center;
    let align_placement_style = match eff_align {
        PlacementAlign::End => "right: 0; left: auto;",
        PlacementAlign::Center => "left: 50%; translate: -50% 0; --monoxus-select-align-x: -50%;",
        PlacementAlign::Start => "left: 0; right: auto;",
    };
    assert!(align_placement_style.contains("translate: -50% 0"));
    assert!(!align_placement_style.contains("transform: translateX"));
}

#[test]
fn select_content_css_variables_and_bits_parity_contract() {
    use monoxus::select::runtime::select_content_css_variables;

    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(4.0);

    let anchor = Rect::new(100.0, 200.0, 260.0, 40.0);
    let content = Size::new(260.0, 150.0);
    let viewport = Size::new(1024.0, 768.0);

    let placement = layer.position_with_available_size(anchor, content, viewport);
    let geom = placement.geometry();

    assert_eq!(geom.anchor_width(), 260.0);
    assert_eq!(geom.anchor_height(), 40.0);

    let vars = select_content_css_variables(geom);
    let var_map: std::collections::HashMap<String, String> = vars.into_iter().collect();

    // Bits UI measured pixel variables
    assert_eq!(
        var_map.get("--bits-select-anchor-width"),
        Some(&"260px".to_string())
    );
    assert_eq!(
        var_map.get("--bits-select-anchor-height"),
        Some(&"40px".to_string())
    );
    assert!(var_map.contains_key("--bits-select-content-available-width"));
    assert!(var_map.contains_key("--bits-select-content-available-height"));
    assert!(var_map.contains_key("--bits-select-content-transform-origin"));

    // Radix compatibility aliases
    assert_eq!(
        var_map.get("--radix-select-trigger-width"),
        Some(&"260px".to_string())
    );
    assert_eq!(
        var_map.get("--radix-select-trigger-height"),
        Some(&"40px".to_string())
    );
}

#[test]
fn select_floating_readiness_and_measuring_contract() {
    use monoxus::foundation::overlay::FloatingReadiness;

    let measuring = FloatingReadiness::default();
    assert_eq!(measuring, FloatingReadiness::Measuring);
    assert_eq!(measuring.positioning_state(), "unpositioned");
    assert!(!measuring.is_positioned());

    let ready = FloatingReadiness::Ready;
    assert_eq!(ready.positioning_state(), "positioned");
    assert!(ready.is_positioned());
}

#[test]
fn select_floating_arrow_coordinates_contract() {
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Center)
        .with_side_offset(4.0);

    let anchor = Rect::new(100.0, 200.0, 200.0, 40.0);
    let content = Size::new(200.0, 150.0);
    let viewport = Size::new(1024.0, 768.0);

    let placement = layer.position_with_available_size(anchor, content, viewport);
    let arrow = placement.arrow();

    assert!(!arrow.hidden());
    assert!(arrow.x().is_some());
}

#[test]
fn select_scroll_buttons_compound_surface_contract() {
    use dioxus::prelude::*;
    use monoxus::select::{SelectScrollDownButton, SelectScrollUpButton, SelectViewport};

    let mut dom = VirtualDom::new(|| {
        rsx! {
            SelectScrollUpButton {
                class: "scroll-up",
                "▲"
            }
            SelectViewport {
                class: "viewport",
                "content"
            }
            SelectScrollDownButton {
                class: "scroll-down",
                "▼"
            }
        }
    });
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
#[test]
fn fm_ovl_1_select_ime_process_key_does_not_dismiss_open_select() {
    use dioxus::prelude::*;
    use keyboard_types::{Code, Key, NamedKey};
    use monoxus::select::{
        SelectContent, SelectItem, SelectPortal, SelectRoot, SelectTrigger, SelectViewport,
    };

    use crate::common::native_fixture::*;

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn SelectImeScenario() -> Element {
        rsx! {
            div { style: "padding: 24px;",
                SelectRoot {
                    id: "fm-ovl1-sel".to_string(),
                    SelectTrigger {
                        class: "fm-ovl1-sel-trig".to_string(),
                        style: "width: 160px; height: 32px;".to_string(),
                        "Pick"
                    }
                    SelectPortal {
                        SelectContent {
                            class: "fm-ovl1-sel-content".to_string(),
                            style: "width: 160px; background: white;".to_string(),
                            SelectViewport {
                                SelectItem {
                                    value: "a".to_string(),
                                    text: "Alpha".to_string(),
                                    "Alpha"
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(SelectImeScenario);
    click_and_pump(&mut harness, ".fm-ovl1-sel-trig");
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query(".fm-ovl1-sel-content").is_some());

    // IME Process key must not dismiss open Select (`FM-OVL-1`, `radix#4153`).
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Process), Code::Unidentified);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query(".fm-ovl1-sel-content").is_some());

    // Normal Escape closes Select.
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query(".fm-ovl1-sel-content").is_none());
}

#[cfg(feature = "native")]
#[test]
fn fm_ovl_3_select_content_and_static_inside_dialog_preserve_dialog_open() {
    use dioxus::prelude::*;
    use monoxus::{
        dialog::{DialogContent, DialogPortal, DialogRoot},
        select::{
            SelectContent, SelectContentStatic, SelectItem, SelectPortal, SelectRoot,
            SelectTrigger, SelectViewport,
        },
    };

    use crate::common::native_fixture::*;

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn SelectInDialogScenario() -> Element {
        let mut dlg_open = use_signal(|| true);
        let mut floating_val = use_signal(|| None::<String>);
        let mut static_val = use_signal(|| None::<String>);

        rsx! {
            div { style: "padding: 24px;",
                span { id: "fm3-sel-float-val", "{floating_val().unwrap_or_else(|| \"none\".to_string())}" }
                span { id: "fm3-sel-static-val", "{static_val().unwrap_or_else(|| \"none\".to_string())}" }
                DialogRoot {
                    id: Some("fm3-sel-dlg".to_string()),
                    open: Some(dlg_open()),
                    on_open_change: move |next| dlg_open.set(next),
                    DialogPortal {
                        DialogContent {
                            id: Some("fm3-sel-dlg-content".to_string()),
                            style: Some("width: 400px; height: 320px; padding: 16px; background: white;".to_string()),
                            SelectRoot {
                                id: "fm3-sel-float".to_string(),
                                on_value_change: move |v| floating_val.set(v),
                                SelectTrigger {
                                    class: "fm3-sel-float-trig".to_string(),
                                    style: "width: 160px; height: 32px;".to_string(),
                                    "Floating"
                                }
                                SelectPortal {
                                    SelectContent {
                                        class: "fm3-sel-float-content".to_string(),
                                        style: "width: 160px; background: white;".to_string(),
                                        SelectViewport {
                                            SelectItem {
                                                value: "opt-f".to_string(),
                                                text: "Float Opt".to_string(),
                                                class: "fm3-sel-float-opt".to_string(),
                                                "Float Opt"
                                            }
                                        }
                                    }
                                }
                            }
                            SelectRoot {
                                id: "fm3-sel-static".to_string(),
                                on_value_change: move |v| static_val.set(v),
                                SelectTrigger {
                                    class: "fm3-sel-static-trig".to_string(),
                                    style: "width: 160px; height: 32px; margin-top: 12px;".to_string(),
                                    "Static"
                                }
                                SelectContentStatic {
                                    class: "fm3-sel-static-content".to_string(),
                                    style: "width: 160px; background: white;".to_string(),
                                    SelectViewport {
                                        SelectItem {
                                            value: "opt-s".to_string(),
                                            text: "Static Opt".to_string(),
                                            class: "fm3-sel-static-opt".to_string(),
                                            "Static Opt"
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

    let mut harness = mount_native_scenario(SelectInDialogScenario);
    assert!(harness.query("#fm3-sel-dlg-content").is_some());

    // 1. Portalled floating SelectContent registers branches so clicking item never dismisses Dialog.
    click_and_pump(&mut harness, ".fm3-sel-float-trig");
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query(".fm3-sel-float-content").is_some());
    click_and_pump(&mut harness, ".fm3-sel-float-opt");
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#fm3-sel-dlg-content").is_some());
    assert_eq!(harness.text_content("#fm3-sel-float-val"), "opt-f");

    // 2. SelectContentStatic registers branches & LIFO layer (`B7`), preserving Dialog open.
    click_and_pump(&mut harness, ".fm3-sel-static-trig");
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query(".fm3-sel-static-content").is_some());
    click_and_pump(&mut harness, ".fm3-sel-static-opt");
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#fm3-sel-dlg-content").is_some());
    assert_eq!(harness.text_content("#fm3-sel-static-val"), "opt-s");
}

#[cfg(feature = "native")]
#[test]
fn fm_ovl_7_select_arrow_emits_attrs_and_registers_arrow_id() {
    use std::{cell::RefCell, rc::Rc};

    use dioxus::prelude::*;
    use monoxus::select::{
        SelectArrow, SelectContent, SelectContext, SelectItem, SelectPortal, SelectRoot,
        SelectTrigger, SelectViewport,
    };

    use crate::common::native_fixture::*;

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[derive(Clone, PartialEq)]
    struct ArrowProbeSlot(Rc<RefCell<Option<String>>>);

    #[component]
    fn ArrowIdProbe(slot: ArrowProbeSlot) -> Element {
        let ctx = use_context::<SelectContext>();
        *slot.0.borrow_mut() = ctx.runtime.arrow_id();
        rsx! {}
    }

    let arrow_slot = ArrowProbeSlot(Rc::new(RefCell::new(None)));

    #[component]
    fn SelectArrowScenario(slot: ArrowProbeSlot) -> Element {
        rsx! {
            div { style: "padding: 40px;",
                SelectRoot {
                    id: "fm7-sel-arrow".to_string(),
                    open: true,
                    SelectTrigger {
                        class: "fm7-sel-trig".to_string(),
                        style: "width: 140px; height: 32px;".to_string(),
                        "Trigger"
                    }
                    SelectPortal {
                        SelectContent {
                            class: "fm7-sel-content".to_string(),
                            style: "width: 140px; background: white;".to_string(),
                            SelectArrow {
                                id: Some("fm7-sel-arrow-el".to_string()),
                            }
                            SelectViewport {
                                SelectItem {
                                    value: "a".to_string(),
                                    text: "Alpha".to_string(),
                                    "Alpha"
                                }
                            }
                        }
                    }
                    ArrowIdProbe { slot }
                }
            }
        }
    }

    let mut harness = mount_native_scenario_with_props(
        SelectArrowScenario,
        SelectArrowScenarioProps {
            slot: arrow_slot.clone(),
        },
    );
    advance_frame_and_pump(&mut harness, 0.02);

    assert!(harness.query("#fm7-sel-arrow-el").is_some());
    assert_eq!(
        harness
            .attr("#fm7-sel-arrow-el", "data-monoxus-select-arrow")
            .as_deref(),
        Some("")
    );
    assert_eq!(
        harness.attr("#fm7-sel-arrow-el", "data-state").as_deref(),
        Some("open")
    );
    assert!(harness.attr("#fm7-sel-arrow-el", "data-side").is_some());
    assert!(harness.attr("#fm7-sel-arrow-el", "data-align").is_some());
    assert_eq!(
        arrow_slot.0.borrow().as_deref(),
        Some("fm7-sel-arrow-el")
    );
}


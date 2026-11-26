use super::{
    DocumentDismissEventPayload, DocumentPointerStreamGuard, ElementScrollMetrics, ElementSize,
    FloatingAutoUpdatePayload, FormResetEventPayload, ModalShieldMode, PresenceEventPayload,
    PresenceMonitorFallback, ResizeObserverBox, blur_active_element, get_element_client_rect,
    get_element_offset_rect, get_element_scroll_metrics, get_tabbable_candidates,
    is_focus_within_element, local_today, local_today_ymd, measure_element_without_animation,
    remove_from_tab_order, restore_tab_order, set_element_scroll_position,
    set_element_style_property, shield_external_dom, start_document_pointer_stream,
    start_element_scroll_monitor, start_keyboard_modality_monitor, start_resize_monitor,
    start_window_blur_monitor,
};

#[test]
fn parses_document_dismiss_payload() {
    let json = r#"{"kind":"pointer_down","pathIds":["content","child"]}"#;
    let payload: DocumentDismissEventPayload = serde_json::from_str(json).unwrap();
    assert_eq!(
        payload,
        DocumentDismissEventPayload::PointerDown {
            path_ids: vec!["content".to_string(), "child".to_string()]
        }
    );

    let json = r#"{"kind":"focus_in","pathIds":["content"]}"#;
    let payload: DocumentDismissEventPayload = serde_json::from_str(json).unwrap();
    assert_eq!(
        payload,
        DocumentDismissEventPayload::FocusIn {
            path_ids: vec!["content".to_string()]
        }
    );

    let json = r#"{"kind":"click","pathIds":["outside-btn"]}"#;
    let payload: DocumentDismissEventPayload = serde_json::from_str(json).unwrap();
    assert_eq!(
        payload,
        DocumentDismissEventPayload::Click {
            path_ids: vec!["outside-btn".to_string()]
        }
    );

    let json = r#"{"kind":"escape"}"#;
    let payload: DocumentDismissEventPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload, DocumentDismissEventPayload::Escape);
}

#[test]
fn parses_presence_event_payload() {
    let json = r#"{"kind":"fallback","cycleId":6,"reason":"missing"}"#;
    let payload: PresenceEventPayload = serde_json::from_str(json).unwrap();
    assert_eq!(
        payload,
        PresenceEventPayload::Fallback {
            cycle_id: 6,
            reason: PresenceMonitorFallback::Missing,
        }
    );

    let json = r#"{"kind":"animation_end","cycleId":9,"animationName":"fade-out"}"#;
    let payload: PresenceEventPayload = serde_json::from_str(json).unwrap();
    assert_eq!(
        payload,
        PresenceEventPayload::AnimationEnd {
            cycle_id: 9,
            animation_name: "fade-out".to_string(),
        }
    );

    let json = r#"{"kind":"stopped","cycleId":11}"#;
    let payload: PresenceEventPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload, PresenceEventPayload::Stopped { cycle_id: 11 });
}

#[test]
fn parses_floating_auto_update_payload() {
    let json = r#"{"kind":"scroll"}"#;
    let payload: FloatingAutoUpdatePayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload, FloatingAutoUpdatePayload::Scroll);

    let json = r#"{"kind":"update"}"#;
    let payload: FloatingAutoUpdatePayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload, FloatingAutoUpdatePayload::Update);
}

#[test]
fn parses_form_reset_payload() {
    let json = r#"{"kind":"reset"}"#;
    let payload: FormResetEventPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload, FormResetEventPayload::Reset);
}

#[test]
fn test_document_pointer_stream_and_client_rect_noop() {
    let mut guard = DocumentPointerStreamGuard::noop();
    guard.stop();

    let mut stream_guard = start_document_pointer_stream(|_pt| {});
    stream_guard.stop();

    assert_eq!(get_element_client_rect("nonexistent"), None);
}

#[test]
fn test_element_metrics_and_monitors_noop() {
    assert_eq!(measure_element_without_animation("nonexistent"), None);
    assert_eq!(get_element_scroll_metrics("nonexistent"), None);
    set_element_scroll_position("nonexistent", 10.0, 20.0);
    set_element_style_property("nonexistent", "opacity", "1");

    let scroll_guard = start_element_scroll_monitor("nonexistent", || {});
    scroll_guard.stop();

    let resize_guard = start_resize_monitor("nonexistent", ResizeObserverBox::BorderBox, |_| {});
    resize_guard.stop();

    let size = ElementSize {
        width: 100.0,
        height: 50.0,
    };
    assert_eq!(size.width, 100.0);
    assert_eq!(size.height, 50.0);

    let metrics = ElementScrollMetrics::default();
    assert_eq!(metrics.scroll_left, 0.0);
    assert_eq!(metrics.scroll_top, 0.0);
}

#[test]
fn test_batch2_browser_foundation_mock_helpers() {
    let (year, month, day) = local_today_ymd();
    assert!(year >= 1970);
    assert!((1..=12).contains(&month));
    assert!((1..=31).contains(&day));

    let today = local_today();
    assert_eq!(today.year, year);
    assert_eq!(today.month, month);
    assert_eq!(today.day, day);

    assert!(!is_focus_within_element("nonexistent"));
    assert!(!blur_active_element());

    let shield = shield_external_dom(&["nonexistent"], ModalShieldMode::AriaHiddenOnly);
    assert!(shield.is_released());

    assert_eq!(get_element_offset_rect("nonexistent"), None);
    assert!(get_tabbable_candidates("nonexistent").is_empty());
    remove_from_tab_order("nonexistent");
    restore_tab_order("nonexistent");

    let blur_guard = start_window_blur_monitor(|| {});
    blur_guard.stop();

    let modality_guard = start_keyboard_modality_monitor(|_| {});
    modality_guard.stop();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::super::{
        get_element_offset_rect, get_tabbable_candidates, remove_from_tab_order,
        restore_tab_order, start_keyboard_modality_monitor, start_window_blur_monitor,
    };
    use blitz_dom::{Attribute, BaseDocument, DocumentConfig, LocalName, QualName, ns};
    use blitz_traits::events::{
        BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, KeyState, MouseEventButton,
        MouseEventButtons, PointerCoords, PointerDetails, UiEvent,
    };
    use keyboard_types::{Code, Key, Location, Modifiers};
    use oxidase::dom::Document;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn qname(tag: &'static str) -> QualName {
        QualName::new(None, ns!(html), LocalName::from(tag))
    }

    fn attr(name: &'static str, val: &str) -> Attribute {
        Attribute {
            name: QualName::new(None, ns!(), LocalName::from(name)),
            value: val.to_string(),
        }
    }

    fn mk_pe(x: f32, y: f32, buttons: MouseEventButtons) -> BlitzPointerEvent {
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
            button: MouseEventButton::Main,
            buttons,
            mods: Default::default(),
            details: PointerDetails::default(),
            element: Default::default(),
            active_pointers: Default::default(),
        }
    }

    #[test]
    fn test_native_offset_rect_and_window_blur_monitor() {
        let base_doc = Rc::new(RefCell::new(BaseDocument::new(DocumentConfig::default())));
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
            let parent = mutator.create_element(
                qname("div"),
                vec![attr(
                    "style",
                    "position: relative; margin-left: 80px; padding-left: 12px; padding-top: 6px;",
                )],
            );
            let item = mutator.create_element(
                qname("div"),
                vec![
                    attr("id", "nav-item"),
                    attr(
                        "style",
                        "margin-left: 10px; margin-top: 4px; width: 90px; height: 32px; display: block;",
                    ),
                ],
            );
            mutator.append_children(parent, &[item]);
            mutator.append_children(body, &[parent]);
            mutator.append_children(html, &[body]);
            mutator.append_children(root_id, &[html]);
        }
        doc.resolve(0.0);

        Document::with_current(doc.clone(), || {
            let rect = get_element_offset_rect("nav-item").expect("offset rect must resolve");
            assert!((rect.x() - 22.0).abs() < 1.0, "x={}", rect.x());
            assert!((rect.y() - 10.0).abs() < 1.0, "y={}", rect.y());
            assert!((rect.width() - 90.0).abs() < 1.0, "w={}", rect.width());
            assert!((rect.height() - 32.0).abs() < 1.0, "h={}", rect.height());

            let blur_count = Rc::new(RefCell::new(0usize));
            let blur_clone = blur_count.clone();
            let guard = start_window_blur_monitor(move || {
                *blur_clone.borrow_mut() += 1;
            });
            doc.dispatch_window_blur();
            assert_eq!(*blur_count.borrow(), 1);
            guard.stop();
            doc.dispatch_window_blur();
            assert_eq!(*blur_count.borrow(), 1);
        });
    }

    #[test]
    fn test_native_tabbable_candidates_and_tab_order_suppression() {
        let base_doc = Rc::new(RefCell::new(BaseDocument::new(DocumentConfig::default())));
        let doc = Document::from_base(base_doc.clone());

        {
            let mut base = base_doc.borrow_mut();
            let root_id = base.root_node().id;
            let mut mutator = base.mutate();
            let html = mutator.create_element(qname("html"), vec![]);
            let body = mutator.create_element(qname("body"), vec![]);
            let container = mutator.create_element(qname("div"), vec![attr("id", "tab-scope")]);

            let btn_ok = mutator.create_element(
                qname("button"),
                vec![attr("id", "btn-ok"), attr("tabindex", "0")],
            );
            let btn_no_id = mutator.create_element(qname("button"), vec![]);
            let btn_disabled = mutator.create_element(
                qname("button"),
                vec![attr("id", "btn-disabled"), attr("disabled", "")],
            );
            let btn_hidden = mutator.create_element(
                qname("button"),
                vec![attr("id", "btn-hidden"), attr("hidden", "")],
            );
            let btn_aria_hidden = mutator.create_element(
                qname("button"),
                vec![attr("id", "btn-aria-hidden"), attr("aria-hidden", "true")],
            );
            let input_hidden = mutator.create_element(
                qname("input"),
                vec![attr("id", "input-hidden"), attr("type", "hidden")],
            );
            let div_neg = mutator.create_element(
                qname("div"),
                vec![attr("id", "div-neg"), attr("tabindex", "-1")],
            );
            let link_ok = mutator.create_element(
                qname("a"),
                vec![attr("id", "link-ok"), attr("href", "#top")],
            );

            mutator.append_children(
                container,
                &[
                    btn_ok,
                    btn_no_id,
                    btn_disabled,
                    btn_hidden,
                    btn_aria_hidden,
                    input_hidden,
                    div_neg,
                    link_ok,
                ],
            );
            mutator.append_children(body, &[container]);
            mutator.append_children(html, &[body]);
            mutator.append_children(root_id, &[html]);
        }
        doc.resolve(0.0);

        Document::with_current(doc.clone(), || {
            let candidates = get_tabbable_candidates("tab-scope");
            assert_eq!(candidates.len(), 3);
            assert_eq!(candidates[0], "btn-ok");
            assert!(candidates[1].starts_with("monoxus-tabbable-"));
            assert_eq!(candidates[2], "link-ok");

            remove_from_tab_order("tab-scope");
            remove_from_tab_order("tab-scope");

            assert!(get_tabbable_candidates("tab-scope").is_empty());
            let btn_ok_el = doc.element_by_id("btn-ok").unwrap();
            let link_ok_el = doc.element_by_id("link-ok").unwrap();
            assert_eq!(btn_ok_el.get_attribute("tabindex").as_deref(), Some("-1"));
            assert_eq!(
                btn_ok_el.get_attribute("data-monoxus-tabindex").as_deref(),
                Some("0")
            );
            assert_eq!(link_ok_el.get_attribute("tabindex").as_deref(), Some("-1"));
            assert_eq!(
                link_ok_el.get_attribute("data-monoxus-tabindex").as_deref(),
                Some("__none__")
            );

            restore_tab_order("tab-scope");
            let restored = get_tabbable_candidates("tab-scope");
            assert_eq!(restored, candidates);
            assert_eq!(btn_ok_el.get_attribute("tabindex").as_deref(), Some("0"));
            assert!(!btn_ok_el.has_attribute("data-monoxus-tabindex"));
            assert_eq!(link_ok_el.get_attribute("tabindex"), None);
            assert!(!link_ok_el.has_attribute("data-monoxus-tabindex"));
        });
    }

    #[test]
    fn test_native_keyboard_modality_monitor() {
        let base_doc = Rc::new(RefCell::new(BaseDocument::new(DocumentConfig::default())));
        let doc = Document::from_base(base_doc.clone());

        {
            let mut base = base_doc.borrow_mut();
            let root_id = base.root_node().id;
            let mut mutator = base.mutate();
            let html = mutator.create_element(qname("html"), vec![]);
            let body = mutator.create_element(qname("body"), vec![]);
            let target = mutator.create_element(
                qname("div"),
                vec![
                    attr("id", "modality-target"),
                    attr("style", "width: 200px; height: 200px; display: block;"),
                ],
            );
            mutator.append_children(body, &[target]);
            mutator.append_children(html, &[body]);
            mutator.append_children(root_id, &[html]);
        }
        doc.resolve(0.0);

        let transitions = Rc::new(RefCell::new(Vec::<bool>::new()));
        let trans_clone = transitions.clone();
        let guard = start_keyboard_modality_monitor(move |is_kb| {
            trans_clone.borrow_mut().push(is_kb);
        });

        doc.handle_ui_event(UiEvent::KeyDown(BlitzKeyEvent {
            key: Key::Character("Tab".into()),
            code: Code::Tab,
            modifiers: Modifiers::empty(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state: KeyState::Pressed,
            text: None,
        }));
        doc.handle_ui_event(UiEvent::PointerDown(mk_pe(
            10.0,
            10.0,
            MouseEventButtons::Primary,
        )));
        doc.handle_ui_event(UiEvent::KeyDown(BlitzKeyEvent {
            key: Key::Character("a".into()),
            code: Code::KeyA,
            modifiers: Modifiers::empty(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state: KeyState::Pressed,
            text: None,
        }));
        doc.handle_ui_event(UiEvent::PointerMove(mk_pe(
            20.0,
            20.0,
            MouseEventButtons::None,
        )));

        assert_eq!(*transitions.borrow(), vec![true, false, true, false]);

        guard.stop();
        doc.handle_ui_event(UiEvent::KeyDown(BlitzKeyEvent {
            key: Key::Character("b".into()),
            code: Code::KeyB,
            modifiers: Modifiers::empty(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state: KeyState::Pressed,
            text: None,
        }));
        assert_eq!(transitions.borrow().len(), 4);
    }
}

use dioxus::prelude::*;
use monoxus::field::{
    Field, FieldControl, FieldControlAttrs, FieldDescription, FieldError, FieldErrorLiveMode,
    FieldLabel, FieldLegend, FieldLegendVariant, FieldOrientation, FieldRoot, FieldSet,
    use_field_control,
};

#[cfg(feature = "native")]
use crate::common;

#[component]
fn BoundInput(
    #[props(default)] id_override: Option<String>,
    #[props(default = "text")] r#type: &'static str,
) -> Element {
    let handle = monoxus::field::use_field_control_with_id(id_override);
    let attrs = handle
        .as_ref()
        .map(|h| h.to_global_attributes())
        .unwrap_or_default();
    rsx! {
        input {
            r#type: r#type,
            ..attrs,
        }
    }
}

#[test]
fn m2b_core_builders_and_controlled_select_flags_contract() {
    use monoxus::foundation::shared::ScopeHandle;
    use monoxus::pin_input::{PinInputDefinition, PinInputSelection};
    use monoxus::rating_group::{
        Direction as RatingDirection, Orientation as RatingOrientation, RatingGroupItemAttributes,
        RatingGroupItemState, RatingGroupRootAttributes,
    };
    use monoxus::select::{Select, SelectMode};
    use monoxus::slider::{
        Direction as SliderDirection, Orientation as SliderOrientation, SliderRootAttributes,
        SliderThumbAttributes,
    };

    // 1. Select controlled flags & empty values regression guard
    let sel_scope = ScopeHandle::root("m2b-core").child("select");
    let controlled_empty_multi = Select::new(sel_scope.clone())
        .with_mode(SelectMode::Multiple)
        .with_values_controlled(true)
        .with_values(Vec::new())
        .with_default_values(vec!["a".to_string()]);
    assert!(controlled_empty_multi.is_values_controlled());
    assert!(controlled_empty_multi.values().is_empty());
    assert_eq!(controlled_empty_multi.default_values(), &["a"]);
    assert_eq!(
        controlled_empty_multi
            .trigger_attributes()
            .data_placeholder_str(),
        "true"
    );

    let sel_root_inv = controlled_empty_multi.root_attributes().with_invalid(true);
    assert_eq!(sel_root_inv.data_invalid(), Some(""));
    assert_eq!(
        controlled_empty_multi
            .root_attributes()
            .with_invalid(false)
            .data_invalid(),
        None
    );

    let sel_trig_inv = controlled_empty_multi
        .trigger_attributes()
        .with_invalid(true)
        .with_aria_describedby(Some("desc-1 err-1".to_string()))
        .with_aria_labelledby(Some("lbl-1".to_string()));
    assert_eq!(sel_trig_inv.aria_invalid(), Some("true"));
    assert_eq!(sel_trig_inv.data_invalid(), Some(""));
    assert_eq!(sel_trig_inv.aria_describedby(), Some("desc-1 err-1"));
    assert_eq!(sel_trig_inv.aria_labelledby(), Some("lbl-1"));

    // 2. Slider root & thumb additive builders (zero aria-required on role="slider")
    let slider_root = SliderRootAttributes::new(
        "s-root".to_string(),
        SliderOrientation::Horizontal,
        SliderDirection::Ltr,
        false,
    )
    .with_required(true)
    .with_invalid(true)
    .with_aria_describedby(Some("s-desc".to_string()));
    assert_eq!(slider_root.data_required(), Some(""));
    assert_eq!(slider_root.data_invalid(), Some(""));
    assert_eq!(slider_root.aria_describedby(), Some("s-desc"));

    let slider_thumb = SliderThumbAttributes::new(
        "s-thumb".to_string(),
        25.0,
        0.0,
        100.0,
        SliderOrientation::Horizontal,
        SliderDirection::Ltr,
        false,
        false,
        false,
        25.0,
        None,
        None,
        None,
    )
    .with_required(true)
    .with_invalid(true)
    .with_aria_describedby(Some("s-desc".to_string()));
    assert_eq!(slider_thumb.role(), "slider");
    assert_eq!(slider_thumb.aria_invalid(), Some("true"));
    assert_eq!(slider_thumb.data_invalid(), Some(""));
    assert_eq!(slider_thumb.data_required(), Some(""));
    assert_eq!(slider_thumb.aria_describedby(), Some("s-desc"));

    // 3. PinInput definition & attrs
    let pin_scope = ScopeHandle::root("m2b-core").child("pin");
    let pin_def = PinInputDefinition::new(pin_scope).with_invalid(true);
    assert!(pin_def.is_invalid());
    assert_eq!(pin_def.root_attributes(None).data_invalid(), Some(""));
    assert_eq!(pin_def.cell_attributes(0, None, false).data_invalid(), Some(""));
    let pin_in = pin_def
        .input_attributes(None, "12", PinInputSelection::empty(), None)
        .with_aria_describedby(Some("pin-d".to_string()))
        .with_aria_labelledby(Some("pin-l".to_string()))
        .with_aria_label(Some("OTP".to_string()));
    assert_eq!(pin_in.aria_invalid(), Some("true"));
    assert_eq!(pin_in.data_invalid(), Some(""));
    assert_eq!(pin_in.aria_describedby(), Some("pin-d"));
    assert_eq!(pin_in.aria_labelledby(), Some("pin-l"));
    assert_eq!(pin_in.aria_label(), Some("OTP"));

    // 4. RatingGroup root & item attrs
    let rg_root = RatingGroupRootAttributes::new(
        "rg-root".to_string(),
        3.0,
        0.0,
        5,
        RatingOrientation::Horizontal,
        RatingDirection::Ltr,
        false,
        false,
        true,
        None,
        None,
        None,
    )
    .with_invalid(true)
    .with_aria_describedby(Some("rg-d".to_string()));
    assert_eq!(rg_root.aria_invalid, Some("true"));
    assert_eq!(rg_root.data_invalid, Some(""));
    assert_eq!(rg_root.aria_describedby.as_deref(), Some("rg-d"));

    let rg_item = RatingGroupItemAttributes::new(
        "rg-item-0".to_string(),
        0,
        RatingOrientation::Horizontal,
        false,
        false,
        RatingGroupItemState::Active,
    )
    .with_invalid(true);
    assert_eq!(rg_item.data_invalid, Some(""));
}

#[test]
fn m3a_core_builders_combobox_date_field_and_range_field_contract() {
    use monoxus::combobox::{ComboboxInputAttrs, ComboboxTriggerAttrs};
    use monoxus::date_field::DateFieldInputAttrs;
    use monoxus::date_range_field::DateRangeFieldRootAttrs;
    use monoxus::foundation::date_time::field_policy::FieldRangeRole;
    use monoxus::foundation::shared::Direction;

    let cbx_in = ComboboxInputAttrs::new("cbx-in", Some("cbx-ct".into()), true, None, false)
        .with_invalid(true)
        .with_aria_describedby(Some("d1 e1".into()))
        .with_required(true);
    assert_eq!(cbx_in.aria_invalid(), Some("true"));
    assert_eq!(cbx_in.data_invalid(), Some(""));
    assert_eq!(cbx_in.aria_describedby(), Some("d1 e1"));
    assert_eq!(cbx_in.aria_required(), Some("true"));

    let cbx_in_valid = cbx_in.clone().with_invalid(false).with_required(false);
    assert_eq!(cbx_in_valid.aria_invalid(), None);
    assert_eq!(cbx_in_valid.data_invalid(), None);
    assert_eq!(cbx_in_valid.aria_required(), None);

    let cbx_trig = ComboboxTriggerAttrs::new("cbx-tr", Some("cbx-ct".into()), false, false)
        .with_invalid(true)
        .with_aria_describedby(Some("d1 e1".into()))
        .with_required(true);
    assert_eq!(cbx_trig.aria_invalid(), Some("true"));
    assert_eq!(cbx_trig.data_invalid(), Some(""));
    assert_eq!(cbx_trig.aria_describedby(), Some("d1 e1"));
    assert_eq!(cbx_trig.aria_required(), Some("true"));

    let df_in = DateFieldInputAttrs::new(
        "df-in".into(),
        FieldRangeRole::Standalone,
        Some("df-lbl".into()),
        None,
        Direction::Ltr,
        true,
        false,
    )
    .with_aria_describedby(Some("df-desc df-err".into()));
    assert_eq!(df_in.aria_invalid, Some("true"));
    assert_eq!(df_in.data_invalid, Some(""));
    assert_eq!(df_in.aria_describedby.as_deref(), Some("df-desc df-err"));

    let drf_root = DateRangeFieldRootAttrs::new(
        "drf-root".into(),
        "drf-lbl".into(),
        Direction::Ltr,
        true,
        false,
    )
    .with_aria_describedby(Some("drf-desc drf-err".into()));
    assert_eq!(drf_root.aria_invalid, Some("true"));
    assert_eq!(drf_root.data_invalid, Some(""));
    assert_eq!(
        drf_root.aria_describedby.as_deref(),
        Some("drf-desc drf-err")
    );
}

#[test]
fn field_root_and_field_alias_mount_in_vdom() {
    #[component]
    fn App() -> Element {
        rsx! {
            Field {
                id: "email-field",
                name: "email",
                orientation: FieldOrientation::Vertical,
                invalid: true,
                required: true,
                FieldLabel { "Email" }
                FieldControl {
                    render_child: move |attrs: FieldControlAttrs| rsx! {
                        input {
                            id: attrs.id.as_str(),
                            name: attrs.name.as_deref(),
                            "aria-invalid": attrs.aria_invalid,
                            "aria-describedby": attrs.aria_describedby.as_deref(),
                            "data-invalid": attrs.data_invalid,
                        }
                    }
                }
                FieldDescription { "We will never share your email." }
                FieldError {
                    errors: vec![
                        String::from("Email is required"),
                        String::from("Email is required"),
                        String::from("Must contain @"),
                    ],
                }
            }
        }
    }

    let mut dom = VirtualDom::new(App);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
}

#[test]
fn field_optional_handler_toggle_never_panics_dioxus_5894() {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[derive(Props, Clone)]
    struct ToggleProps {
        attach_slot: Rc<RefCell<Option<Signal<bool>>>>,
        render_count: Rc<Cell<usize>>,
    }
    impl PartialEq for ToggleProps {
        fn eq(&self, _other: &Self) -> bool {
            true
        }
    }

    #[component]
    fn ToggleHarness(props: ToggleProps) -> Element {
        let attach = use_signal(|| true);
        *props.attach_slot.borrow_mut() = Some(attach);
        props.render_count.set(props.render_count.get() + 1);
        rsx! {
            FieldRoot {
                id: "toggle-root",
                on_mounted: if attach() {
                    Some(EventHandler::new(|_| {}))
                } else {
                    None
                },
                FieldLabel {
                    id: "toggle-label",
                    on_click: if attach() {
                        Some(EventHandler::new(|_| {}))
                    } else {
                        None
                    },
                    on_mouse_down: if attach() {
                        Some(EventHandler::new(|_| {}))
                    } else {
                        None
                    },
                    "Toggle"
                }
            }
        }
    }

    let attach_slot = Rc::new(RefCell::new(None::<Signal<bool>>));
    let render_count = Rc::new(Cell::new(0_usize));
    let mut dom = VirtualDom::new_with_props(
        ToggleHarness,
        ToggleProps {
            attach_slot: Rc::clone(&attach_slot),
            render_count: Rc::clone(&render_count),
        },
    );
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(render_count.get(), 1);

    // Flip true -> false -> true across renders to genuinely exercise Dioxus #5894 safety.
    let mut sig = attach_slot.borrow().expect("attach signal initialized");
    dom.in_scope(dioxus::core::ScopeId::ROOT, || sig.set(false));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(render_count.get(), 2);

    dom.in_scope(dioxus::core::ScopeId::ROOT, || sig.set(true));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(render_count.get(), 3);
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use super::common::native_fixture::*;
    use monoxus::checkbox::{CheckboxRoot, CheckedState};

    /// FM-FLD-01: Dynamic mounted-only `aria-describedby` + `use_drop` + keyed swap `RegistrationToken`.
    #[component]
    fn FmFld01Scenario() -> Element {
        let mut show_desc = use_signal(|| false);
        let mut desc_key = use_signal(|| 1_u32);
        let mut is_invalid = use_signal(|| false);
        let mut force_mount_err = use_signal(|| false);

        rsx! {
            div {
                button {
                    id: "btn-toggle-desc",
                    onclick: move |_| show_desc.toggle(),
                    "Toggle Desc"
                }
                button {
                    id: "btn-swap-desc-key",
                    onclick: move |_| desc_key += 1,
                    "Swap Desc Key"
                }
                button {
                    id: "btn-toggle-invalid",
                    onclick: move |_| is_invalid.toggle(),
                    "Toggle Invalid"
                }
                button {
                    id: "btn-toggle-force-err",
                    onclick: move |_| force_mount_err.toggle(),
                    "Toggle Force Err"
                }

                FieldRoot {
                    id: "fm01-root",
                    name: "username",
                    invalid: is_invalid(),
                    required: true,
                    FieldLabel { id: "fm01-label", "Username" }
                    BoundInput { id_override: Some(String::from("fm01-input")) }
                    if show_desc() {
                        FieldDescription {
                            key: "{desc_key()}",
                            id: "fm01-desc",
                            "Helper copy v{desc_key()}"
                        }
                    }
                    FieldError {
                        id: "fm01-err",
                        force_mount: force_mount_err(),
                        "Username is required"
                    }
                }
            }
        }
    }

    #[test]
    fn fm_fld_01_dynamic_mounted_only_describedby_and_keyed_swap_safety() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(FmFld01Scenario);
        drain_polls_and_resolve(&mut harness);

        // 1. Neither description nor error mounted -> no aria-describedby, no data-invalid/aria-invalid
        {
            let base = harness.base();
            let input = base.get_node(harness.node("#fm01-input")).unwrap();
            let root = base.get_node(harness.node("#fm01-root")).unwrap();
            assert_eq!(input.attr("aria-describedby".into()), None);
            assert_eq!(input.attr("aria-invalid".into()), None);
            assert_eq!(input.attr("data-invalid".into()), None);
            assert_eq!(root.attr("aria-describedby".into()), None);
            assert_eq!(root.attr("data-invalid".into()), None);
            assert_eq!(root.attr("role".into()), Some("group"));
            assert!(harness.query("#fm01-err").is_none());
        }

        // 2. Mount FieldDescription -> aria-describedby == "fm01-desc"
        click_and_pump(&mut harness, "#btn-toggle-desc");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let input = base.get_node(harness.node("#fm01-input")).unwrap();
            let root = base.get_node(harness.node("#fm01-root")).unwrap();
            assert_eq!(input.attr("aria-describedby".into()), Some("fm01-desc"));
            assert_eq!(root.attr("aria-describedby".into()), Some("fm01-desc"));
        }

        // 3. Keyed swap of FieldDescription with identical id -> old use_drop must NOT erase replacement
        click_and_pump(&mut harness, "#btn-swap-desc-key");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let input = base.get_node(harness.node("#fm01-input")).unwrap();
            assert_eq!(
                input.attr("aria-describedby".into()),
                Some("fm01-desc"),
                "Keyed swap with identical id must retain aria-describedby via RegistrationToken guard"
            );
        }

        // 4. Toggle invalid: true -> FieldError mounts, aria-describedby == "fm01-desc fm01-err",
        //    aria-invalid="true", data-invalid=""
        click_and_pump(&mut harness, "#btn-toggle-invalid");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let input = base.get_node(harness.node("#fm01-input")).unwrap();
            let root = base.get_node(harness.node("#fm01-root")).unwrap();
            let err = base.get_node(harness.node("#fm01-err")).unwrap();
            assert_eq!(
                input.attr("aria-describedby".into()),
                Some("fm01-desc fm01-err")
            );
            assert_eq!(
                root.attr("aria-describedby".into()),
                Some("fm01-desc fm01-err")
            );
            assert_eq!(input.attr("aria-invalid".into()), Some("true"));
            assert_eq!(input.attr("data-invalid".into()), Some(""));
            assert_eq!(root.attr("data-invalid".into()), Some(""));
            assert_eq!(err.attr("role".into()), Some("alert"));
            assert_eq!(err.attr("aria-live".into()), None);
            assert_eq!(err.attr("data-invalid".into()), Some(""));
        }

        // 5. Toggle invalid: false -> FieldError unmounts, aria-describedby == "fm01-desc" (no dangling ID)
        click_and_pump(&mut harness, "#btn-toggle-invalid");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let input = base.get_node(harness.node("#fm01-input")).unwrap();
            let root = base.get_node(harness.node("#fm01-root")).unwrap();
            assert_eq!(input.attr("aria-describedby".into()), Some("fm01-desc"));
            assert_eq!(input.attr("aria-invalid".into()), None);
            assert_eq!(input.attr("data-invalid".into()), None);
            assert_eq!(root.attr("data-invalid".into()), None);
            assert!(harness.query("#fm01-err").is_none());
        }

        // 6. Force-mount inactive FieldError -> DOM node exists with role=None, error ID omitted from describedby
        click_and_pump(&mut harness, "#btn-toggle-force-err");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let input = base.get_node(harness.node("#fm01-input")).unwrap();
            let err = base.get_node(harness.node("#fm01-err")).unwrap();
            assert_eq!(input.attr("aria-describedby".into()), Some("fm01-desc"));
            assert_eq!(err.attr("role".into()), None);
            assert_eq!(err.attr("aria-live".into()), None);
            assert_eq!(err.attr("data-invalid".into()), None);
        }
    }

    #[component]
    fn FieldBoundCheckbox(
        probe_attr: &'static str,
        checked: Signal<CheckedState>,
        on_toggle: EventHandler<CheckedState>,
    ) -> Element {
        let handle = use_field_control().expect("inside FieldControl");
        let id = handle.id().to_owned();
        let mut attrs = handle.to_global_attributes();
        // Remove duplicate id since CheckboxRoot takes explicit id prop
        attrs.retain(|a| a.name != "id");
        rsx! {
            CheckboxRoot {
                id: id,
                checked: Some(checked()),
                on_checked_change: move |next| on_toggle.call(next),
                "data-probe": probe_attr,
                attributes: attrs,
                span { "box" }
            }
        }
    }

    /// FM-FLD-02: 1:N Multi-Control (`FieldSet` + `FieldControl` + `FieldLabel` + `CheckboxRoot`).
    #[component]
    fn FmFld02Scenario() -> Element {
        let mut first = use_signal(|| CheckedState::Unchecked);
        let mut second = use_signal(|| CheckedState::Unchecked);

        rsx! {
            div {
                div {
                    id: "fm02-state",
                    "data-first": first().aria_checked(),
                    "data-second": second().aria_checked(),
                }
                FieldSet {
                    id: "fm02-fieldset",
                    invalid: true,
                    required: true,
                    FieldLegend {
                        id: "fm02-legend",
                        variant: FieldLegendVariant::Label,
                        "Notifications"
                    }
                    FieldDescription {
                        id: "fm02-desc",
                        "Choose notification channels."
                    }
                    div {
                        style: "display: block; padding: 4px;",
                        FieldControl {
                            FieldLabel {
                                id: "fm02-label-1",
                                style: "display: inline-block; padding: 4px;",
                                "Email alerts"
                            }
                            FieldBoundCheckbox {
                                probe_attr: "cb-1",
                                checked: first,
                                on_toggle: move |next| first.set(next),
                            }
                        }
                    }
                    div {
                        style: "display: block; padding: 4px;",
                        FieldControl {
                            FieldLabel {
                                id: "fm02-label-2",
                                style: "display: inline-block; padding: 4px;",
                                "SMS alerts"
                            }
                            FieldBoundCheckbox {
                                probe_attr: "cb-2",
                                checked: second,
                                on_toggle: move |next| second.set(next),
                            }
                        }
                    }
                    FieldError {
                        id: "fm02-err",
                        live_mode: FieldErrorLiveMode::Polite,
                        errors: vec![
                            String::from("Select at least one channel"),
                            String::from("Select at least one channel"),
                        ],
                    }
                }
            }
        }
    }

    #[test]
    fn fm_fld_02_fieldset_multi_control_pairing_and_label_click_isolation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(FmFld02Scenario);
        drain_polls_and_resolve(&mut harness);

        let (cb1_id, cb2_id) = {
            let base = harness.base();
            let fs = base.get_node(harness.node("#fm02-fieldset")).unwrap();
            let legend = base.get_node(harness.node("#fm02-legend")).unwrap();
            let label1 = base.get_node(harness.node("#fm02-label-1")).unwrap();
            let label2 = base.get_node(harness.node("#fm02-label-2")).unwrap();
            let cb1 = base.get_node(harness.node("[data-probe='cb-1']")).unwrap();
            let cb2 = base.get_node(harness.node("[data-probe='cb-2']")).unwrap();
            let err = base.get_node(harness.node("#fm02-err")).unwrap();

            let id1 = cb1.attr("id".into()).unwrap().to_owned();
            let id2 = cb2.attr("id".into()).unwrap().to_owned();
            assert_ne!(id1, id2, "1:N FieldControl slots must emit distinct deterministic IDs");

            assert_eq!(label1.attr("for".into()), Some(id1.as_str()));
            assert_eq!(label2.attr("for".into()), Some(id2.as_str()));

            assert_eq!(fs.attr("aria-describedby".into()), Some("fm02-desc fm02-err"));
            assert_eq!(cb1.attr("aria-describedby".into()), Some("fm02-desc fm02-err"));
            assert_eq!(cb2.attr("aria-describedby".into()), Some("fm02-desc fm02-err"));
            assert_eq!(cb1.attr("aria-labelledby".into()), Some("fm02-label-1"));
            assert_eq!(cb2.attr("aria-labelledby".into()), Some("fm02-label-2"));

            // FM-FLD-03 data-invalid hygiene on every Field* part
            assert_eq!(fs.attr("data-invalid".into()), Some(""));
            assert_eq!(fs.attr("aria-invalid".into()), Some("true"));
            assert_eq!(legend.attr("data-variant".into()), Some("label"));
            assert_eq!(legend.attr("data-invalid".into()), Some(""));
            assert_eq!(label1.attr("data-invalid".into()), Some(""));
            assert_eq!(label2.attr("data-invalid".into()), Some(""));
            assert_eq!(cb1.attr("data-invalid".into()), Some(""));
            assert_eq!(cb2.attr("data-invalid".into()), Some(""));
            assert_eq!(err.attr("role".into()), Some("alert"));
            assert_eq!(err.attr("aria-live".into()), Some("polite"));
            assert_eq!(err.attr("data-invalid".into()), Some(""));

            (id1, id2)
        };
        assert!(!cb1_id.is_empty() && !cb2_id.is_empty());

        // Click label 1 -> toggles ONLY checkbox 1
        click_and_pump(&mut harness, "#fm02-label-1");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let state = base.get_node(harness.node("#fm02-state")).unwrap();
            assert_eq!(state.attr("data-first".into()), Some("true"));
            assert_eq!(state.attr("data-second".into()), Some("false"));
        }

        // Click label 2 -> toggles ONLY checkbox 2
        click_and_pump(&mut harness, "#fm02-label-2");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let state = base.get_node(harness.node("#fm02-state")).unwrap();
            assert_eq!(state.attr("data-first".into()), Some("true"));
            assert_eq!(state.attr("data-second".into()), Some("true"));
        }
    }

    /// FM-FLD-03: Valid state omits `data-invalid` everywhere (never `"false"`), and multi-error renders `<ul><li>`.
    #[component]
    fn FmFld03Scenario() -> Element {
        let mut invalid = use_signal(|| false);

        rsx! {
            div {
                button {
                    id: "fm03-toggle",
                    onclick: move |_| invalid.toggle(),
                    "Toggle"
                }
                FieldRoot {
                    id: "fm03-root",
                    invalid: invalid(),
                    disabled: false,
                    required: true,
                    read_only: true,
                    FieldLabel { id: "fm03-label", "Bio" }
                    BoundInput { id_override: Some(String::from("fm03-input")) }
                    FieldDescription { id: "fm03-desc", "Short bio." }
                    if invalid() {
                        FieldError {
                            id: "fm03-err",
                            errors: vec![
                                String::from("Too short"),
                                String::from("Too short"),
                                String::from("No emojis"),
                            ],
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_fld_03_data_invalid_hygiene_and_deduped_error_list() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(FmFld03Scenario);
        drain_polls_and_resolve(&mut harness);

        // Valid state: data-invalid omitted everywhere, data-required="" & data-readonly="" present
        {
            let base = harness.base();
            for sel in ["#fm03-root", "#fm03-label", "#fm03-input", "#fm03-desc"] {
                let node = base.get_node(harness.node(sel)).unwrap();
                assert_eq!(
                    node.attr("data-invalid".into()),
                    None,
                    "{sel} must omit data-invalid when valid"
                );
            }
            let input = base.get_node(harness.node("#fm03-input")).unwrap();
            assert_eq!(input.attr("aria-invalid".into()), None);
            assert_eq!(input.attr("data-required".into()), Some(""));
            assert_eq!(input.attr("data-readonly".into()), Some(""));
            assert_eq!(input.attr("data-disabled".into()), None);
        }

        // Invalid state: data-invalid="" everywhere + deduped <ul><li> with 2 items
        click_and_pump(&mut harness, "#fm03-toggle");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            for sel in ["#fm03-root", "#fm03-label", "#fm03-input", "#fm03-desc", "#fm03-err"] {
                let node = base.get_node(harness.node(sel)).unwrap();
                assert_eq!(
                    node.attr("data-invalid".into()),
                    Some(""),
                    "{sel} must emit empty-string data-invalid when invalid"
                );
            }
            assert!(
                harness.query("#fm03-err ul").is_some(),
                "Multiple deduplicated errors must render a <ul> list"
            );
            let li_nodes = harness.query_all("#fm03-err ul li");
            assert_eq!(li_nodes.len(), 2, "Duplicate error messages must be deduplicated");
        }
    }

    /// FM-CTL-04 (M2-A): Controlled CheckboxRoot, CheckboxGroup, and SwitchRoot freeze
    /// mount-time initial values on `<form>` reset (never `default_*` or later prop mutations).
    #[component]
    fn M2aControlledResetScenario() -> Element {
        use monoxus::button::{Button, ButtonType};
        use monoxus::checkbox::{CheckboxGroup, CheckboxIndicator};
        use monoxus::switch::{SwitchRoot, SwitchThumb};

        let mut cb_checked = use_signal(|| CheckedState::Checked);
        let mut sw_checked = use_signal(|| true);
        let mut grp_values = use_signal(|| vec![String::from("a")]);
        let mut last_cb_reset = use_signal(|| String::from("none"));
        let mut last_sw_reset = use_signal(|| String::from("none"));
        let mut last_grp_reset = use_signal(|| String::from("none"));

        rsx! {
            div {
                style: "width: 800px; height: 600px;",
                div {
                    id: "m2a-reset-probe",
                    "data-cb": cb_checked().aria_checked(),
                    "data-sw": if sw_checked() { "true" } else { "false" },
                    "data-grp": grp_values().join(","),
                    "data-last-cb": last_cb_reset(),
                    "data-last-sw": last_sw_reset(),
                    "data-last-grp": last_grp_reset(),
                }
                form {
                    id: "m2a-reset-form",
                    CheckboxRoot {
                        id: "m2a-reset-cb",
                        name: "agree",
                        checked: Some(cb_checked()),
                        default_checked: CheckedState::Unchecked,
                        on_checked_change: move |next: CheckedState| {
                            last_cb_reset.set(next.aria_checked().to_string());
                            cb_checked.set(next);
                        },
                        style: "width: 40px; height: 24px;",
                        CheckboxIndicator { "v" }
                    }
                    SwitchRoot {
                        id: "m2a-reset-sw",
                        name: "alerts",
                        checked: Some(sw_checked()),
                        default_checked: false,
                        on_checked_change: move |next: bool| {
                            last_sw_reset.set(next.to_string());
                            sw_checked.set(next);
                        },
                        style: "width: 40px; height: 24px;",
                        SwitchThumb {}
                    }
                    CheckboxGroup {
                        id: "m2a-reset-grp",
                        name: "tags",
                        values: Some(grp_values()),
                        default_values: vec![],
                        on_values_change: move |next: Vec<String>| {
                            last_grp_reset.set(next.join(","));
                            grp_values.set(next);
                        },
                        CheckboxRoot {
                            id: "m2a-reset-grp-a",
                            value: "a",
                            style: "width: 40px; height: 24px;",
                            "A"
                        }
                        CheckboxRoot {
                            id: "m2a-reset-grp-b",
                            value: "b",
                            style: "width: 40px; height: 24px;",
                            "B"
                        }
                    }
                    Button {
                        id: "m2a-reset-btn".to_string(),
                        r#type: ButtonType::Reset,
                        style: "width: 80px; height: 28px;".to_string(),
                        "Reset"
                    }
                }
            }
        }
    }

    #[test]
    fn m2a_controlled_reset_baselines_freeze_mount_time_values() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M2aControlledResetScenario);
        advance_frame_and_pump(&mut harness, 0.016);

        // Mutate all 3 controlled widgets away from their mount-time initial values
        click_and_pump(&mut harness, "#m2a-reset-cb");
        click_and_pump(&mut harness, "#m2a-reset-sw");
        click_and_pump(&mut harness, "#m2a-reset-grp-a");
        click_and_pump(&mut harness, "#m2a-reset-grp-b");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2a-reset-probe")).unwrap();
            assert_eq!(probe.attr("data-cb".into()), Some("false"));
            assert_eq!(probe.attr("data-sw".into()), Some("false"));
            assert_eq!(probe.attr("data-grp".into()), Some("b"));
        }

        // Trigger `<form>` reset -> must restore mount-time initial values (`true`, `true`, `"a"`)
        click_and_pump(&mut harness, "#m2a-reset-btn");
        advance_frame_and_pump(&mut harness, 0.016);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2a-reset-probe")).unwrap();
            assert_eq!(
                probe.attr("data-cb".into()),
                Some("true"),
                "Controlled CheckboxRoot reset must emit mount-time Checked (not default Unchecked)"
            );
            assert_eq!(
                probe.attr("data-sw".into()),
                Some("true"),
                "Controlled SwitchRoot reset must emit mount-time true (not default false)"
            );
            assert_eq!(
                probe.attr("data-grp".into()),
                Some("a"),
                "Controlled CheckboxGroup reset must emit mount-time ['a'] (not default [])"
            );
        }
    }

    /// FM-CTL-03 (M2-A): Composite `on_focus_out` on RadioGroup / ToggleGroup / CheckboxGroup
    /// never fires on internal item-to-item focus moves, fires once when leaving to an external
    /// button, and toggling `on_blur: Some <-> None` never panics (`Dioxus #5894`).
    #[component]
    fn M2aCompositeFocusOutAndBlurScenario() -> Element {
        use monoxus::checkbox::CheckboxGroup;
        use monoxus::radio_group::{RadioGroupItem, RadioGroupRoot};
        use monoxus::switch::SwitchRoot;
        use monoxus::toggle::Toggle;
        use monoxus::toggle_group::{ToggleGroupItem, ToggleGroupMode, ToggleGroupRoot};

        let mut rg_focus_outs = use_signal(|| 0_u32);
        let mut tg_focus_outs = use_signal(|| 0_u32);
        let mut cg_focus_outs = use_signal(|| 0_u32);
        let mut item_blurs = use_signal(|| 0_u32);
        let mut attach_blur = use_signal(|| true);

        let mk_blur = move || {
            if attach_blur() {
                Some(EventHandler::new(move |_: FocusEvent| {
                    item_blurs += 1;
                }))
            } else {
                None
            }
        };

        rsx! {
            div {
                style: "width: 800px; height: 600px;",
                div {
                    id: "m2a-focus-probe",
                    "data-rg-out": "{rg_focus_outs}",
                    "data-tg-out": "{tg_focus_outs}",
                    "data-cg-out": "{cg_focus_outs}",
                    "data-blurs": "{item_blurs}",
                    "data-attached": if attach_blur() { "true" } else { "false" },
                }
                button {
                    id: "m2a-toggle-blur-handler",
                    r#type: "button",
                    style: "width: 100px; height: 28px;",
                    onclick: move |_| attach_blur.toggle(),
                    "Toggle Blur"
                }
                button {
                    id: "m2a-ext-btn",
                    r#type: "button",
                    style: "width: 100px; height: 28px;",
                    "External"
                }

                RadioGroupRoot {
                    id: "m2a-rg",
                    default_value: Some(String::from("a")),
                    on_focus_out: move |_| rg_focus_outs += 1,
                    RadioGroupItem {
                        id: "m2a-rg-a",
                        value: "a",
                        on_blur: mk_blur(),
                        style: "width: 60px; height: 28px;",
                        "A"
                    }
                    RadioGroupItem {
                        id: "m2a-rg-b",
                        value: "b",
                        on_blur: mk_blur(),
                        style: "width: 60px; height: 28px;",
                        "B"
                    }
                }

                ToggleGroupRoot {
                    id: "m2a-tg",
                    r#type: ToggleGroupMode::Single,
                    on_focus_out: move |_| tg_focus_outs += 1,
                    ToggleGroupItem {
                        id: "m2a-tg-1",
                        value: "1",
                        on_blur: mk_blur(),
                        style: "width: 60px; height: 28px;",
                        "1"
                    }
                    ToggleGroupItem {
                        id: "m2a-tg-2",
                        value: "2",
                        on_blur: mk_blur(),
                        style: "width: 60px; height: 28px;",
                        "2"
                    }
                }

                CheckboxGroup {
                    id: "m2a-cg",
                    on_focus_out: move |_| cg_focus_outs += 1,
                    CheckboxRoot {
                        id: "m2a-cg-1",
                        value: "one",
                        on_blur: mk_blur(),
                        style: "width: 60px; height: 28px;",
                        "One"
                    }
                    CheckboxRoot {
                        id: "m2a-cg-2",
                        value: "two",
                        on_blur: mk_blur(),
                        style: "width: 60px; height: 28px;",
                        "Two"
                    }
                }

                SwitchRoot {
                    id: "m2a-blur-sw",
                    on_blur: mk_blur(),
                    style: "width: 60px; height: 28px;",
                    "Sw"
                }
                Toggle {
                    id: "m2a-blur-toggle",
                    on_blur: mk_blur(),
                    style: "width: 60px; height: 28px;",
                    "Tg"
                }
            }
        }
    }

    #[test]
    fn m2a_composite_focus_out_and_blur_5894_safety() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M2aCompositeFocusOutAndBlurScenario);
        drain_polls_and_resolve(&mut harness);

        // 1. RadioGroup internal focus move A -> B must NOT fire on_focus_out
        click_and_pump(&mut harness, "#m2a-rg-a");
        click_and_pump(&mut harness, "#m2a-rg-b");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2a-focus-probe")).unwrap();
            assert_eq!(
                probe.attr("data-rg-out".into()),
                Some("0"),
                "Internal RadioGroup item move must not fire on_focus_out"
            );
        }

        // Leave RadioGroup to external button -> fires on_focus_out exactly once
        click_and_pump(&mut harness, "#m2a-ext-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2a-focus-probe")).unwrap();
            assert_eq!(
                probe.attr("data-rg-out".into()),
                Some("1"),
                "Leaving RadioGroup to external button must fire on_focus_out once"
            );
        }

        // 2. ToggleGroup internal focus move 1 -> 2 must NOT fire on_focus_out
        click_and_pump(&mut harness, "#m2a-tg-1");
        click_and_pump(&mut harness, "#m2a-tg-2");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2a-focus-probe")).unwrap();
            assert_eq!(
                probe.attr("data-tg-out".into()),
                Some("0"),
                "Internal ToggleGroup item move must not fire on_focus_out"
            );
        }

        // Leave ToggleGroup to external button -> fires on_focus_out exactly once
        click_and_pump(&mut harness, "#m2a-ext-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2a-focus-probe")).unwrap();
            assert_eq!(
                probe.attr("data-tg-out".into()),
                Some("1"),
                "Leaving ToggleGroup to external button must fire on_focus_out once"
            );
        }

        // 3. CheckboxGroup internal focus move 1 -> 2 must NOT fire on_focus_out, leaving fires once
        click_and_pump(&mut harness, "#m2a-cg-1");
        click_and_pump(&mut harness, "#m2a-cg-2");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2a-focus-probe")).unwrap();
            assert_eq!(probe.attr("data-cg-out".into()), Some("0"));
        }
        click_and_pump(&mut harness, "#m2a-ext-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2a-focus-probe")).unwrap();
            assert_eq!(probe.attr("data-cg-out".into()), Some("1"));
        }

        // 4. Toggle on_blur Some <-> None across renders -> zero Dioxus #5894 panic
        click_and_pump(&mut harness, "#m2a-toggle-blur-handler");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m2a-toggle-blur-handler");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2a-focus-probe")).unwrap();
            assert_eq!(probe.attr("data-attached".into()), Some("true"));
        }
    }

    /// FM-FLD-03 + FieldContext/FieldControl wiring + Label auto-`for` across all 6 M2-A controls.
    #[component]
    fn M2aFieldWiringScenario() -> Element {
        use monoxus::checkbox::{CheckboxGroup, CheckboxIndicator};
        use monoxus::label::Label;
        use monoxus::radio_group::{RadioGroupItem, RadioGroupRoot};
        use monoxus::switch::{SwitchRoot, SwitchThumb};
        use monoxus::toggle::Toggle;
        use monoxus::toggle_group::{ToggleGroupItem, ToggleGroupMode, ToggleGroupRoot};

        let mut invalid = use_signal(|| false);

        rsx! {
            div {
                button {
                    id: "m2a-wire-toggle-invalid",
                    onclick: move |_| invalid.toggle(),
                    "Toggle Invalid"
                }

                // 1. Standalone CheckboxRoot + CheckboxIndicator + Label inside FieldRoot
                FieldRoot {
                    id: "m2a-f-cb",
                    invalid: invalid(),
                    required: true,
                    Label { id: "m2a-l-cb", "Accept" }
                    CheckboxRoot {
                        "data-probe": "m2a-cb",
                        default_checked: CheckedState::Checked,
                        CheckboxIndicator {
                            id: "m2a-cb-ind",
                            "v"
                        }
                    }
                    FieldDescription { id: "m2a-d-cb", "Desc" }
                    FieldError { id: "m2a-e-cb", "Err" }
                }

                // 2. SwitchRoot + SwitchThumb + Label inside FieldRoot
                FieldRoot {
                    id: "m2a-f-sw",
                    invalid: invalid(),
                    Label { id: "m2a-l-sw", "Notifications" }
                    SwitchRoot {
                        "data-probe": "m2a-sw",
                        SwitchThumb { id: "m2a-sw-thumb" }
                    }
                    FieldDescription { id: "m2a-d-sw", "Desc" }
                    FieldError { id: "m2a-e-sw", "Err" }
                }

                // 3. RadioGroupRoot + RadioGroupItem inside FieldRoot
                FieldRoot {
                    id: "m2a-f-rg",
                    invalid: invalid(),
                    RadioGroupRoot {
                        id: "m2a-rg-root",
                        RadioGroupItem { id: "m2a-rg-item", value: "x", "X" }
                    }
                    FieldDescription { id: "m2a-d-rg", "Desc" }
                    FieldError { id: "m2a-e-rg", "Err" }
                }

                // 4. Toggle + Label inside FieldRoot
                FieldRoot {
                    id: "m2a-f-tg-single",
                    invalid: invalid(),
                    Label { id: "m2a-l-tg-single", "Bold" }
                    Toggle {
                        "data-probe": "m2a-toggle",
                        "B"
                    }
                    FieldDescription { id: "m2a-d-tg-single", "Desc" }
                    FieldError { id: "m2a-e-tg-single", "Err" }
                }

                // 5. ToggleGroupRoot + ToggleGroupItem inside FieldRoot
                FieldRoot {
                    id: "m2a-f-tg-grp",
                    invalid: invalid(),
                    ToggleGroupRoot {
                        id: "m2a-tg-grp-root",
                        r#type: ToggleGroupMode::Multiple,
                        ToggleGroupItem { id: "m2a-tg-grp-item", value: "i", "I" }
                    }
                    FieldDescription { id: "m2a-d-tg-grp", "Desc" }
                    FieldError { id: "m2a-e-tg-grp", "Err" }
                }

                // 6. FieldSet: Label directly inside omits `for`, Label inside FieldControl binds `for`,
                //    CheckboxGroup suppresses CheckboxRoot auto-binding to FieldControl
                FieldSet {
                    id: "m2a-fs",
                    invalid: invalid(),
                    disabled: true,
                    Label { id: "m2a-fs-direct-label", "Direct Label" }
                    FieldControl {
                        Label { id: "m2a-fs-ctrl-label", "Scoped Label" }
                        CheckboxRoot {
                            "data-probe": "m2a-fs-cb",
                            "C"
                        }
                    }
                    CheckboxGroup {
                        id: "m2a-fs-cbg",
                        CheckboxRoot {
                            id: "m2a-fs-cbg-item",
                            value: "g1",
                            "G1"
                        }
                    }
                    FieldDescription { id: "m2a-d-fs", "FS Desc" }
                    FieldError { id: "m2a-e-fs", "FS Err" }
                }
            }
        }
    }

    #[test]
    fn m2a_field_wiring_data_invalid_hygiene_and_label_auto_for() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M2aFieldWiringScenario);
        drain_polls_and_resolve(&mut harness);

        // 1. Valid state: auto-bound IDs & Label `for`, data-invalid & aria-invalid omitted everywhere
        {
            let base = harness.base();
            let cb = base.get_node(harness.node("[data-probe='m2a-cb']")).unwrap();
            let l_cb = base.get_node(harness.node("#m2a-l-cb")).unwrap();
            let cb_ind = base.get_node(harness.node("#m2a-cb-ind")).unwrap();
            assert_eq!(l_cb.attr("for".into()), cb.attr("id".into()));
            assert_eq!(l_cb.attr("data-required".into()), Some(""));
            assert_eq!(cb.attr("aria-describedby".into()), Some("m2a-d-cb"));
            assert_eq!(cb.attr("aria-invalid".into()), None);
            assert_eq!(cb.attr("data-invalid".into()), None);
            assert_eq!(cb_ind.attr("data-invalid".into()), None);

            let sw = base.get_node(harness.node("[data-probe='m2a-sw']")).unwrap();
            let l_sw = base.get_node(harness.node("#m2a-l-sw")).unwrap();
            let sw_thumb = base.get_node(harness.node("#m2a-sw-thumb")).unwrap();
            assert_eq!(l_sw.attr("for".into()), sw.attr("id".into()));
            assert_eq!(sw.attr("aria-describedby".into()), Some("m2a-d-sw"));
            assert_eq!(sw.attr("aria-invalid".into()), None);
            assert_eq!(sw.attr("data-invalid".into()), None);
            assert_eq!(sw_thumb.attr("data-invalid".into()), None);

            let rg = base.get_node(harness.node("#m2a-rg-root")).unwrap();
            let rg_item = base.get_node(harness.node("#m2a-rg-item")).unwrap();
            assert_eq!(rg.attr("aria-describedby".into()), Some("m2a-d-rg"));
            assert_eq!(rg.attr("aria-invalid".into()), None);
            assert_eq!(rg.attr("data-invalid".into()), None);
            assert_eq!(rg_item.attr("data-invalid".into()), None);

            let tgl = base.get_node(harness.node("[data-probe='m2a-toggle']")).unwrap();
            let l_tgl = base.get_node(harness.node("#m2a-l-tg-single")).unwrap();
            assert_eq!(l_tgl.attr("for".into()), tgl.attr("id".into()));
            assert_eq!(tgl.attr("aria-describedby".into()), Some("m2a-d-tg-single"));
            assert_eq!(tgl.attr("aria-invalid".into()), None);
            assert_eq!(tgl.attr("data-invalid".into()), None);

            let tg_grp = base.get_node(harness.node("#m2a-tg-grp-root")).unwrap();
            let tg_item = base.get_node(harness.node("#m2a-tg-grp-item")).unwrap();
            assert_eq!(tg_grp.attr("aria-describedby".into()), Some("m2a-d-tg-grp"));
            assert_eq!(tg_grp.attr("aria-invalid".into()), None);
            assert_eq!(tg_grp.attr("data-invalid".into()), None);
            assert_eq!(tg_item.attr("data-invalid".into()), None);

            // FieldSet Label auto-for rules + data-disabled inheritance
            let fs_direct_lbl = base.get_node(harness.node("#m2a-fs-direct-label")).unwrap();
            let fs_ctrl_lbl = base.get_node(harness.node("#m2a-fs-ctrl-label")).unwrap();
            let fs_cb = base.get_node(harness.node("[data-probe='m2a-fs-cb']")).unwrap();
            assert_eq!(
                fs_direct_lbl.attr("for".into()),
                None,
                "Label directly inside FieldSet without FieldControl must omit `for`"
            );
            assert_eq!(fs_direct_lbl.attr("data-disabled".into()), Some(""));
            assert_eq!(fs_ctrl_lbl.attr("for".into()), fs_cb.attr("id".into()));
        }

        // 2. Toggle invalid: true -> all 6 controls & subparts emit data-invalid="" & aria-invalid="true" + error ID in aria-describedby
        click_and_pump(&mut harness, "#m2a-wire-toggle-invalid");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            for (sel, expected_desc) in [
                ("[data-probe='m2a-cb']", "m2a-d-cb m2a-e-cb"),
                ("[data-probe='m2a-sw']", "m2a-d-sw m2a-e-sw"),
                ("#m2a-rg-root", "m2a-d-rg m2a-e-rg"),
                ("[data-probe='m2a-toggle']", "m2a-d-tg-single m2a-e-tg-single"),
                ("#m2a-tg-grp-root", "m2a-d-tg-grp m2a-e-tg-grp"),
                ("#m2a-fs-cbg", "m2a-d-fs m2a-e-fs"),
            ] {
                let node = base.get_node(harness.node(sel)).unwrap();
                assert_eq!(
                    node.attr("aria-invalid".into()),
                    Some("true"),
                    "{sel} must emit aria-invalid=true"
                );
                assert_eq!(
                    node.attr("data-invalid".into()),
                    Some(""),
                    "{sel} must emit empty-string data-invalid"
                );
                assert_eq!(
                    node.attr("aria-describedby".into()),
                    Some(expected_desc),
                    "{sel} must include description + error IDs"
                );
            }

            // Sub-parts and Labels must propagate data-invalid=""
            for sel in [
                "#m2a-cb-ind",
                "#m2a-sw-thumb",
                "#m2a-rg-item",
                "#m2a-tg-grp-item",
                "#m2a-l-cb",
                "#m2a-l-sw",
                "#m2a-l-tg-single",
                "#m2a-fs-direct-label",
                "#m2a-fs-ctrl-label",
            ] {
                let node = base.get_node(harness.node(sel)).unwrap();
                assert_eq!(
                    node.attr("data-invalid".into()),
                    Some(""),
                    "{sel} must propagate data-invalid=\"\""
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // Wave M2 Slice M2-B Failure-Mode Gates (FM-CTL-01, FM-CTL-03, FM-CTL-04)
    // -------------------------------------------------------------------------

    /// FM-CTL-01: SelectRoot Pattern C controlled rejection (zero desync),
    /// empty controlled `values: Some(vec![])` regression, synchronous same-frame
    /// value update/reset (zero 1-frame lag), and controlled `<form>` reset baseline.
    #[component]
    fn M2bSelectControlledScenario() -> Element {
        use monoxus::button::{Button, ButtonType};
        use monoxus::select::{
            SelectContentStatic, SelectItem, SelectRoot, SelectTrigger, SelectValue,
        };

        let mut rejected_log = use_signal(Vec::<String>::new);
        let mut accepted_val = use_signal(|| "a".to_string());
        let mut form_val = use_signal(|| "a".to_string());
        let mut form_reset_log = use_signal(Vec::<String>::new);

        rsx! {
            div { style: "padding: 16px; display: flex; flex-direction: column; gap: 12px;",
                div {
                    id: "m2b-sel-probe",
                    "data-rejected": "{rejected_log.read().join(\",\")}",
                    "data-accepted": "{accepted_val()}",
                    "data-form-val": "{form_val()}",
                    "data-form-reset-log": "{form_reset_log.read().join(\",\")}",
                }

                // 1. Rejecting controlled parent: always pinned to Some("a")
                SelectRoot {
                    id: "m2b-sel-reject",
                    name: Some("reject_sel".to_string()),
                    value: Some("a".to_string()),
                    items: Some(vec![
                        monoxus::select::SelectItemData::new("a", "Alpha", false),
                        monoxus::select::SelectItemData::new("b", "Beta", false),
                    ]),
                    on_value_change: move |v: Option<String>| {
                        rejected_log.write().push(v.unwrap_or_else(|| "none".to_string()));
                    },
                    SelectTrigger {
                        id: "m2b-sel-reject-trig",
                        style: "width: 140px; height: 30px;",
                        SelectValue { id: "m2b-sel-reject-val", placeholder: "Pick" }
                    }
                    SelectContentStatic {
                        SelectItem { "data-probe": "m2b-sel-reject-a", value: "a", text: "Alpha", "Alpha" }
                        SelectItem { "data-probe": "m2b-sel-reject-b", value: "b", text: "Beta", "Beta" }
                    }
                }

                // 2. Accepting controlled parent with synchronous reset button
                button {
                    id: "m2b-sel-accept-reset-btn",
                    r#type: "button",
                    style: "width: 100px; height: 24px;",
                    onclick: move |_| accepted_val.set("a".to_string()),
                    "Reset A"
                }
                SelectRoot {
                    id: "m2b-sel-accept",
                    name: Some("accept_sel".to_string()),
                    value: Some(accepted_val()),
                    items: Some(vec![
                        monoxus::select::SelectItemData::new("a", "Alpha", false),
                        monoxus::select::SelectItemData::new("b", "Beta", false),
                    ]),
                    on_value_change: move |v: Option<String>| {
                        if let Some(next) = v {
                            accepted_val.set(next);
                        }
                    },
                    SelectTrigger {
                        id: "m2b-sel-accept-trig",
                        style: "width: 140px; height: 30px;",
                        SelectValue { id: "m2b-sel-accept-val", placeholder: "Pick" }
                    }
                    SelectContentStatic {
                        SelectItem { "data-probe": "m2b-sel-accept-a", value: "a", text: "Alpha", "Alpha" }
                        SelectItem { "data-probe": "m2b-sel-accept-b", value: "b", text: "Beta", "Beta" }
                    }
                }

                // 3. Controlled empty multiple values: Some(vec![]) must NOT fall back to default_values
                SelectRoot {
                    id: "m2b-sel-empty-multi",
                    multiple: true,
                    name: Some("multi_sel".to_string()),
                    values: Some(Vec::new()),
                    default_values: Some(vec!["a".to_string()]),
                    SelectTrigger {
                        id: "m2b-sel-empty-multi-trig",
                        style: "width: 140px; height: 30px;",
                        SelectValue { id: "m2b-sel-empty-multi-val", placeholder: "Empty" }
                    }
                    SelectContentStatic {
                        force_mount: true,
                        SelectItem { "data-probe": "m2b-sel-empty-multi-a", value: "a", text: "Alpha", "Alpha" }
                    }
                }

                // 4. Controlled form reset baseline frozen at mount ("a"), even after parent updates to "b"
                form {
                    id: "m2b-sel-form",
                    button {
                        id: "m2b-sel-form-set-b",
                        r#type: "button",
                        style: "width: 80px; height: 24px;",
                        onclick: move |_| form_val.set("b".to_string()),
                        "Set B"
                    }
                    SelectRoot {
                        id: "m2b-sel-form-root",
                        name: Some("form_sel".to_string()),
                        value: Some(form_val()),
                        default_value: Some("c".to_string()),
                        on_value_change: move |v: Option<String>| {
                            // Deliberately reject reset mutation to verify controlled state stays untouched
                            form_reset_log.write().push(v.unwrap_or_else(|| "none".to_string()));
                        },
                        SelectTrigger {
                            id: "m2b-sel-form-trig",
                            style: "width: 140px; height: 30px;",
                            SelectValue { id: "m2b-sel-form-val", placeholder: "Pick" }
                        }
                        SelectContentStatic {
                            SelectItem { value: "a", text: "Alpha", "Alpha" }
                            SelectItem { value: "b", text: "Beta", "Beta" }
                            SelectItem { value: "c", text: "Gamma", "Gamma" }
                        }
                    }
                    Button {
                        id: "m2b-sel-form-reset".to_string(),
                        r#type: ButtonType::Reset,
                        style: "width: 80px; height: 24px;".to_string(),
                        "Reset"
                    }
                }
            }
        }
    }

    #[test]
    fn m2b_fm_ctl_01_select_controlled_rejection_empty_values_and_zero_lag() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M2bSelectControlledScenario);
        drain_polls_and_resolve(&mut harness);

        // 3. Verify controlled empty multiple initialized empty (not ["a"] from default_values)
        {
            let base = harness.base();
            let trig = base
                .get_node(harness.node("#m2b-sel-empty-multi-trig"))
                .unwrap();
            assert_eq!(trig.attr("data-placeholder".into()), Some("true"));
            let item_a = base
                .get_node(harness.node("[data-probe='m2b-sel-empty-multi-a']"))
                .unwrap();
            assert_eq!(item_a.attr("aria-selected".into()), Some("false"));
            let hidden = base
                .get_node(harness.node("input[name='multi_sel']"))
                .unwrap();
            assert_eq!(hidden.attr("value".into()), Some(""));
        }

        // 1. Rejecting parent: open dropdown, click "b", verify callback fired & zero DOM desync
        click_and_pump(&mut harness, "#m2b-sel-reject-trig");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "[data-probe='m2b-sel-reject-b']");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-sel-probe")).unwrap();
            assert_eq!(probe.attr("data-rejected".into()), Some("b"));
            let val_node = base
                .get_node(harness.node("#m2b-sel-reject-val"))
                .unwrap();
            assert_eq!(val_node.text_content().trim(), "Alpha");
            let hidden = base
                .get_node(harness.node("input[name='reject_sel']"))
                .unwrap();
            assert_eq!(
                hidden.attr("value".into()),
                Some("a"),
                "Rejecting controlled SelectRoot must keep hidden input at 'a'"
            );
            let trig = base
                .get_node(harness.node("#m2b-sel-reject-trig"))
                .unwrap();
            assert_eq!(trig.attr("data-state".into()), Some("closed"));
        }
        // Re-open rejecting select and verify aria-selected is still on 'a', not 'b'
        click_and_pump(&mut harness, "#m2b-sel-reject-trig");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let item_a = base
                .get_node(harness.node("[data-probe='m2b-sel-reject-a']"))
                .unwrap();
            let item_b = base
                .get_node(harness.node("[data-probe='m2b-sel-reject-b']"))
                .unwrap();
            assert_eq!(item_a.attr("aria-selected".into()), Some("true"));
            assert_eq!(item_b.attr("aria-selected".into()), Some("false"));
        }

        // 2. Accepting parent: "a" -> "b" -> "a" synchronously with zero 1-frame reset lag
        click_and_pump(&mut harness, "#m2b-sel-accept-trig");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "[data-probe='m2b-sel-accept-b']");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let val_node = base
                .get_node(harness.node("#m2b-sel-accept-val"))
                .unwrap();
            assert_eq!(val_node.text_content().trim(), "Beta");
            let hidden = base
                .get_node(harness.node("input[name='accept_sel']"))
                .unwrap();
            assert_eq!(hidden.attr("value".into()), Some("b"));
        }
        click_and_pump(&mut harness, "#m2b-sel-accept-reset-btn");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let val_node = base
                .get_node(harness.node("#m2b-sel-accept-val"))
                .unwrap();
            assert_eq!(
                val_node.text_content().trim(),
                "Alpha",
                "Synchronous render-time sync must reflect reset back to 'a' immediately"
            );
            let hidden = base
                .get_node(harness.node("input[name='accept_sel']"))
                .unwrap();
            assert_eq!(hidden.attr("value".into()), Some("a"));
        }

        // 4. Controlled form reset baseline frozen at mount ("a") and controlled signal untouched on reject
        click_and_pump(&mut harness, "#m2b-sel-form-set-b");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m2b-sel-form-reset");
        advance_frame_and_pump(&mut harness, 0.016);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-sel-probe")).unwrap();
            assert_eq!(
                probe.attr("data-form-reset-log".into()),
                Some("a"),
                "Controlled SelectRoot form reset must emit mount-time 'a' (not default_value 'c' or mutated 'b')"
            );
            let hidden = base
                .get_node(harness.node("input[name='form_sel']"))
                .unwrap();
            assert_eq!(
                hidden.attr("value".into()),
                Some("b"),
                "Controlled SelectRoot form reset must not mutate internal signal when parent rejects"
            );
        }
    }

    /// Helper probing `SliderRuntime::focused_thumb` inside consumer `SliderThumb::on_blur`.
    #[component]
    fn SliderBlurOrderThumb(
        index: usize,
        id: String,
        mut blur_saw_none: Signal<Option<bool>>,
        attach_blur: bool,
    ) -> Element {
        use monoxus::slider::{SliderThumb, use_slider_context};
        let slider_ctx = use_slider_context();
        let on_blur = attach_blur.then(|| {
            EventHandler::new(move |_: FocusEvent| {
                let is_none = slider_ctx.0.focused_thumb.peek().is_none();
                blur_saw_none.set(Some(is_none));
            })
        });
        rsx! {
            SliderThumb {
                index,
                id,
                on_blur,
                style: "width: 24px; height: 24px;",
            }
        }
    }

    /// Helper probing `PinInputRuntime::is_focused` inside consumer `PinInputRoot::on_blur`.
    #[component]
    fn PinBlurObserver(
        mut blur_saw_unfocused: Signal<Option<bool>>,
        mut attach_blur: Signal<bool>,
    ) -> Element {
        use monoxus::pin_input::{PinInputCell, PinInputRoot, try_use_pin_input_context};

        // Inner bridge reading PinInputContext into a CopyValue accessible from root's on_blur
        #[component]
        fn CapturePinCtx(
            mut rt_slot: Signal<Option<monoxus::pin_input::PinInputRuntime>>,
        ) -> Element {
            let ctx = try_use_pin_input_context();
            if let Some(c) = ctx
                && rt_slot.peek().is_none()
            {
                rt_slot.set(Some(c.runtime));
            }
            rsx! {}
        }

        let rt_slot = use_signal(|| None::<monoxus::pin_input::PinInputRuntime>);
        let on_blur = attach_blur().then(|| {
            EventHandler::new(move |_: FocusEvent| {
                let unfocused = rt_slot
                    .peek()
                    .as_ref()
                    .is_some_and(|rt| !*rt.is_focused.peek());
                blur_saw_unfocused.set(Some(unfocused));
            })
        });

        rsx! {
            PinInputRoot {
                id: "m2b-pin-blur-root",
                input_id: "m2b-pin-blur-input",
                max_length: 4,
                on_blur,
                style: "width: 160px; height: 32px; display: flex;",
                CapturePinCtx { rt_slot }
                PinInputCell { index: 0, style: "width: 32px; height: 32px;" }
                PinInputCell { index: 1, style: "width: 32px; height: 32px;" }
            }
        }
    }

    /// FM-CTL-03 (M2-B): Internal-first blur ordering on SliderThumb & PinInputRoot,
    /// composite `on_focus_out` on multi-thumb Slider & portaled Select, and `#5894` toggle safety.
    #[component]
    fn M2bBlurAndCompositeFocusScenario() -> Element {
        use monoxus::rating_group::{RatingGroupItem, RatingGroupRoot};
        use monoxus::select::{
            SelectContent, SelectItem, SelectPortal, SelectRoot, SelectTrigger, SelectValue,
        };
        use monoxus::slider::{SliderRange, SliderRoot, SliderTrack, SliderValue};

        let slider_blur_saw_none = use_signal(|| None::<bool>);
        let pin_blur_saw_unfocused = use_signal(|| None::<bool>);
        let mut slider_focus_outs = use_signal(|| 0_u32);
        let mut select_focus_outs = use_signal(|| 0_u32);
        let mut attach_handlers = use_signal(|| true);

        let slider_focus_out = attach_handlers().then(|| {
            EventHandler::new(move |_: FocusEvent| {
                slider_focus_outs += 1;
            })
        });
        let select_focus_out = attach_handlers().then(|| {
            EventHandler::new(move |_: FocusEvent| {
                select_focus_outs += 1;
            })
        });
        let generic_blur = attach_handlers().then(|| EventHandler::new(move |_: FocusEvent| {}));
        let generic_focus = attach_handlers().then(|| EventHandler::new(move |_: FocusEvent| {}));

        rsx! {
            div { style: "padding: 20px; display: flex; flex-direction: column; gap: 16px;",
                div {
                    id: "m2b-blur-probe",
                    "data-slider-blur-none": "{slider_blur_saw_none():?}",
                    "data-pin-blur-unfocused": "{pin_blur_saw_unfocused():?}",
                    "data-slider-focus-out": "{slider_focus_outs()}",
                    "data-select-focus-out": "{select_focus_outs()}",
                    "data-attached": "{attach_handlers()}",
                }
                button {
                    id: "m2b-ext-btn",
                    r#type: "button",
                    style: "width: 90px; height: 28px;",
                    "External"
                }
                button {
                    id: "m2b-toggle-5894",
                    r#type: "button",
                    style: "width: 120px; height: 28px;",
                    onclick: move |_| attach_handlers.toggle(),
                    "Toggle Handlers"
                }

                // Multi-thumb Slider with composite focus-out + internal-first thumb blur
                SliderRoot {
                    id: "m2b-slider-multi",
                    default_value: SliderValue::Multiple(vec![20.0, 80.0]),
                    on_focus_out: slider_focus_out,
                    style: "width: 240px; height: 32px;",
                    SliderTrack {
                        style: "width: 240px; height: 8px;",
                        SliderRange {}
                    }
                    SliderBlurOrderThumb {
                        index: 0,
                        id: "m2b-slider-thumb-0".to_string(),
                        blur_saw_none: slider_blur_saw_none,
                        attach_blur: attach_handlers(),
                    }
                    SliderBlurOrderThumb {
                        index: 1,
                        id: "m2b-slider-thumb-1".to_string(),
                        blur_saw_none: use_signal(|| None::<bool>),
                        attach_blur: attach_handlers(),
                    }
                }

                // PinInput internal-first blur order
                PinBlurObserver {
                    blur_saw_unfocused: pin_blur_saw_unfocused,
                    attach_blur: attach_handlers,
                }

                // Portaled Select composite focus-out
                SelectRoot {
                    id: "m2b-select-comp",
                    on_focus_out: select_focus_out,
                    SelectTrigger {
                        id: "m2b-select-comp-trig",
                        on_blur: generic_blur,
                        on_focus: generic_focus,
                        style: "width: 140px; height: 30px;",
                        SelectValue { placeholder: "Open" }
                    }
                    SelectPortal {
                        SelectContent {
                            SelectItem { id: "m2b-select-comp-a", value: "a", "Alpha" }
                            SelectItem { id: "m2b-select-comp-b", value: "b", "Beta" }
                        }
                    }
                }

                // RatingGroup #5894 toggle target
                RatingGroupRoot {
                    id: "m2b-rg-5894",
                    on_blur: generic_blur,
                    on_focus: generic_focus,
                    RatingGroupItem { index: 0, "★" }
                }
            }
        }
    }

    #[test]
    fn m2b_fm_ctl_03_blur_order_composite_focus_out_and_5894() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M2bBlurAndCompositeFocusScenario);
        drain_polls_and_resolve(&mut harness);

        // 1. Multi-thumb Slider: thumb 0 -> thumb 1 must NOT fire on_focus_out,
        //    and thumb 0 on_blur must observe focused_thumb == None already!
        focus_node(&mut harness, "#m2b-slider-thumb-0");
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m2b-slider-thumb-1");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-blur-probe")).unwrap();
            assert_eq!(
                probe.attr("data-slider-blur-none".into()),
                Some("Some(true)"),
                "SliderThumb consumer on_blur must run AFTER runtime.focused_thumb.set(None)"
            );
            assert_eq!(
                probe.attr("data-slider-focus-out".into()),
                Some("0"),
                "Moving focus between Slider thumbs must not fire on_focus_out"
            );
        }

        // Leave Slider to external button -> on_focus_out fires once
        click_and_pump(&mut harness, "#m2b-ext-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-blur-probe")).unwrap();
            assert_eq!(probe.attr("data-slider-focus-out".into()), Some("1"));
        }

        // 2. PinInputRoot: focus input then blur to external button -> consumer on_blur sees is_focused == false
        focus_node(&mut harness, "#m2b-pin-blur-input");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m2b-ext-btn");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-blur-probe")).unwrap();
            assert_eq!(
                probe.attr("data-pin-blur-unfocused".into()),
                Some("Some(true)"),
                "PinInputRoot consumer on_blur must run AFTER runtime.handle_blur()"
            );
        }

        // 3. Portaled Select: opening listbox moves focus to portaled content -> must NOT fire on_focus_out
        click_and_pump(&mut harness, "#m2b-select-comp-trig");
        advance_frame_and_pump(&mut harness, 0.03);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-blur-probe")).unwrap();
            assert_eq!(
                probe.attr("data-select-focus-out".into()),
                Some("0"),
                "Opening Select listbox must not fire false composite on_focus_out"
            );
        }
        // Moving focus out of the portaled listbox to external button fires on_focus_out once
        focus_node(&mut harness, "#m2b-ext-btn");
        advance_frame_and_pump(&mut harness, 0.03);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-blur-probe")).unwrap();
            assert_eq!(
                probe.attr("data-select-focus-out".into()),
                Some("1"),
                "Leaving Select to external button must fire composite on_focus_out once"
            );
        }

        // 4. Dioxus #5894 toggle Some <-> None across renders on all 4 M2-B controls -> zero panic
        click_and_pump(&mut harness, "#m2b-toggle-5894");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m2b-toggle-5894");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-blur-probe")).unwrap();
            assert_eq!(probe.attr("data-attached".into()), Some("true"));
        }
    }

    /// FM-CTL-04 (M2-B): SelectTrigger custom `id` (placement/dismiss/focus-restore/label),
    /// PinInput `<input>` ARIA forwarding & FieldControl binding, Slider `required` + controlled
    /// reset baseline + FieldControl thumb-0 binding, and RatingGroup Field/ARIA/blur.
    #[component]
    fn M2bFmCtl04Scenario() -> Element {
        use monoxus::button::{Button, ButtonType};
        use monoxus::label::Label;
        use monoxus::pin_input::{PinInputCell, PinInputRoot};
        use monoxus::rating_group::{RatingGroupItem, RatingGroupRoot};
        use monoxus::select::{
            SelectContent, SelectItem, SelectPortal, SelectRoot, SelectTrigger, SelectValue,
        };
        use monoxus::slider::{SliderRange, SliderRoot, SliderThumb, SliderTrack, SliderValue};

        let mut slider_val = use_signal(|| 20.0_f64);
        let mut slider_reset_log = use_signal(Vec::<String>::new);
        let mut rating_focus_events = use_signal(Vec::<String>::new);

        rsx! {
            div { style: "padding: 24px; display: flex; flex-direction: column; gap: 18px;",
                div {
                    id: "m2b-ctl04-probe",
                    "data-slider-val": "{slider_val()}",
                    "data-slider-reset-log": "{slider_reset_log.read().join(\",\")}",
                    "data-rating-events": "{rating_focus_events.read().join(\",\")}",
                }
                button {
                    id: "m2b-ctl04-outside",
                    r#type: "button",
                    style: "width: 90px; height: 28px;",
                    "Outside"
                }

                // 1. SelectTrigger { id: "custom-sel" } + <label for="custom-sel"> + FieldRoot invalid/desc
                FieldRoot {
                    id: "m2b-f-sel",
                    invalid: true,
                    FieldLabel { id: "m2b-l-sel", "Country" }
                    SelectRoot {
                        id: "m2b-sel-custom-root",
                        SelectTrigger {
                            id: "custom-sel",
                            style: "width: 160px; height: 32px;",
                            SelectValue { placeholder: "Choose" }
                        }
                        SelectPortal {
                            SelectContent {
                                SelectItem { value: "kr", "Korea" }
                                SelectItem { value: "us", "USA" }
                            }
                        }
                    }
                    FieldDescription { id: "m2b-d-sel", "Select country" }
                    FieldError { id: "m2b-e-sel", "Country required" }
                }

                // 2a. PinInputRoot with GlobalAttributes ARIA forwarding to focusable <input>
                PinInputRoot {
                    id: "m2b-pin-global-root",
                    input_id: "m2b-pin-global-input",
                    max_length: 4,
                    "aria-invalid": "true",
                    "aria-describedby": "pin-ext-desc",
                    "aria-label": "OTP code",
                    style: "width: 160px; height: 32px; display: flex;",
                    PinInputCell { id: "m2b-pin-global-cell-0", index: 0, style: "width: 32px; height: 32px;" }
                }

                // 2b. PinInputRoot inside FieldRoot + FieldControl (auto-binds input id)
                FieldRoot {
                    id: "m2b-f-pin",
                    invalid: true,
                    FieldControl {
                        FieldLabel { id: "m2b-l-pin", "Passcode" }
                        PinInputRoot {
                            id: "m2b-pin-field-root",
                            max_length: 4,
                            style: "width: 160px; height: 32px; display: flex;",
                            PinInputCell { id: "m2b-pin-field-cell-0", index: 0, style: "width: 32px; height: 32px;" }
                        }
                    }
                    FieldDescription { id: "m2b-d-pin", "4 digits" }
                    FieldError { id: "m2b-e-pin", "Invalid code" }
                }

                // 3. Controlled SliderRoot inside <form> + FieldRoot: required + reset baseline + FieldControl thumb 0
                form {
                    id: "m2b-slider-form",
                    button {
                        id: "m2b-slider-set-75",
                        r#type: "button",
                        style: "width: 90px; height: 24px;",
                        onclick: move |_| slider_val.set(75.0),
                        "Set 75"
                    }
                    FieldRoot {
                        id: "m2b-f-slider",
                        invalid: true,
                        FieldControl {
                            Label { id: "m2b-l-slider", "Volume" }
                            SliderRoot {
                                id: "m2b-slider-root",
                                name: Some("vol".to_string()),
                                required: true,
                                value: SliderValue::Single(slider_val()),
                                default_value: SliderValue::Single(5.0),
                                on_value_change: move |v: SliderValue| {
                                    let first = v.to_vec().first().copied().unwrap_or(0.0);
                                    slider_reset_log.write().push(first.to_string());
                                },
                                style: "width: 200px; height: 32px;",
                                SliderTrack {
                                    style: "width: 200px; height: 8px;",
                                    SliderRange {}
                                }
                                SliderThumb {
                                    index: 0,
                                    "data-probe": "m2b-slider-thumb0",
                                    style: "width: 20px; height: 20px;",
                                }
                            }
                        }
                        FieldDescription { id: "m2b-d-slider", "Adjust volume" }
                        FieldError { id: "m2b-e-slider", "Volume error" }
                    }
                    Button {
                        id: "m2b-slider-reset-btn".to_string(),
                        r#type: ButtonType::Reset,
                        style: "width: 80px; height: 24px;".to_string(),
                        "Reset"
                    }
                }

                // 4. RatingGroup inside FieldRoot: ARIA/data-invalid/FieldLabel + focus/blur
                FieldRoot {
                    id: "m2b-f-rg",
                    invalid: true,
                    FieldLabel { id: "m2b-l-rg", "Score" }
                    RatingGroupRoot {
                        "data-probe": "m2b-rg-root",
                        on_focus: move |_| rating_focus_events.write().push("focus".to_string()),
                        on_blur: move |_| rating_focus_events.write().push("blur".to_string()),
                        style: "width: 140px; height: 28px;",
                        RatingGroupItem { id: "m2b-rg-item-0", index: 0, "★" }
                        RatingGroupItem { id: "m2b-rg-item-1", index: 1, "★" }
                    }
                    FieldDescription { id: "m2b-d-rg", "Rate 1-5" }
                    FieldError { id: "m2b-e-rg", "Score required" }
                }
            }
        }
    }

    #[test]
    fn m2b_fm_ctl_04_select_trigger_id_pin_input_aria_slider_required_reset_and_rating_group() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M2bFmCtl04Scenario);
        advance_frame_and_pump(&mut harness, 0.016);
        drain_polls_and_resolve(&mut harness);

        // 1. SelectTrigger { id: "custom-sel" }: label pairing, Field ARIA, placement, Escape focus restore, outside dismiss
        {
            let base = harness.base();
            let trig = base.get_node(harness.node("#custom-sel")).unwrap();
            let lbl = base.get_node(harness.node("#m2b-l-sel")).unwrap();
            assert_eq!(lbl.attr("for".into()), Some("custom-sel"));
            assert_eq!(trig.attr("aria-invalid".into()), Some("true"));
            assert_eq!(trig.attr("data-invalid".into()), Some(""));
            assert_eq!(
                trig.attr("aria-describedby".into()),
                Some("m2b-d-sel m2b-e-sel")
            );
            assert_eq!(trig.attr("aria-labelledby".into()), Some("m2b-l-sel"));
        }
        // Clicking <label for="custom-sel"> activates/opens the trigger
        click_and_pump(&mut harness, "#m2b-l-sel");
        advance_frame_and_pump(&mut harness, 0.03);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let trig = base.get_node(harness.node("#custom-sel")).unwrap();
            assert_eq!(
                trig.attr("data-state".into()),
                Some("open"),
                "Clicking <label for=\"custom-sel\"> must activate #custom-sel"
            );
            let content = base.get_node(harness.node("[role='listbox']")).unwrap();
            assert_eq!(content.attr("data-state".into()), Some("open"));
            assert_eq!(
                content.attr("data-positioning-state".into()),
                Some("positioned")
            );
        }
        // Escape closes and restores focus to #custom-sel
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        for _ in 0..4 {
            advance_frame_and_pump(&mut harness, 0.016);
        }
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let trig = base.get_node(harness.node("#custom-sel")).unwrap();
            assert_eq!(trig.attr("data-state".into()), Some("closed"));
        }
        assert_native_focus_id(&harness, "custom-sel");

        // Re-open and click outside to verify dismiss boundary recognizes #custom-sel
        click_and_pump(&mut harness, "#custom-sel");
        advance_frame_and_pump(&mut harness, 0.03);
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m2b-ctl04-outside");
        advance_frame_and_pump(&mut harness, 0.03);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let trig = base.get_node(harness.node("#custom-sel")).unwrap();
            assert_eq!(trig.attr("data-state".into()), Some("closed"));
        }

        // 2a. PinInput GlobalAttributes ARIA forwarding to focusable <input> + data-invalid across parts
        focus_node(&mut harness, "#m2b-pin-global-input");
        drain_polls_and_resolve(&mut harness);
        assert_native_focus_id(&harness, "m2b-pin-global-input");
        {
            let base = harness.base();
            let pin_root = base
                .get_node(harness.node("#m2b-pin-global-root"))
                .unwrap();
            let pin_input = base
                .get_node(harness.node("#m2b-pin-global-input"))
                .unwrap();
            let pin_cell = base
                .get_node(harness.node("#m2b-pin-global-cell-0"))
                .unwrap();
            assert_eq!(pin_input.attr("aria-invalid".into()), Some("true"));
            assert_eq!(
                pin_input.attr("aria-describedby".into()),
                Some("pin-ext-desc")
            );
            assert_eq!(pin_input.attr("aria-label".into()), Some("OTP code"));
            assert_eq!(pin_input.attr("data-invalid".into()), Some(""));
            assert_eq!(pin_root.attr("data-invalid".into()), Some(""));
            assert_eq!(pin_cell.attr("data-invalid".into()), Some(""));
            // Must NOT leave aria-invalid/aria-describedby duplicated on outer wrapper div
            assert_eq!(pin_root.attr("aria-invalid".into()), None);
            assert_eq!(pin_root.attr("aria-describedby".into()), None);
        }

        // 2b. PinInput inside FieldControl auto-binds input id to FieldControl::control_id
        {
            let base = harness.base();
            let lbl = base.get_node(harness.node("#m2b-l-pin")).unwrap();
            let pin_input = base
                .get_node(harness.node("#m2b-pin-field-root input"))
                .unwrap();
            let pin_cell = base
                .get_node(harness.node("#m2b-pin-field-cell-0"))
                .unwrap();
            assert_eq!(lbl.attr("for".into()), pin_input.attr("id".into()));
            assert_eq!(pin_input.attr("aria-invalid".into()), Some("true"));
            assert_eq!(
                pin_input.attr("aria-describedby".into()),
                Some("m2b-d-pin m2b-e-pin")
            );
            assert_eq!(pin_input.attr("data-invalid".into()), Some(""));
            assert_eq!(pin_cell.attr("data-invalid".into()), Some(""));
        }

        // 3. Slider required + FieldControl thumb-0 binding + omit aria-required + controlled reset baseline
        {
            let base = harness.base();
            let slider_root = base.get_node(harness.node("#m2b-slider-root")).unwrap();
            let thumb = base
                .get_node(harness.node("[data-probe='m2b-slider-thumb0']"))
                .unwrap();
            let lbl = base.get_node(harness.node("#m2b-l-slider")).unwrap();
            let hidden = base.get_node(harness.node("input[name='vol']")).unwrap();

            assert_eq!(lbl.attr("for".into()), thumb.attr("id".into()));
            assert_eq!(slider_root.attr("data-required".into()), Some(""));
            assert_eq!(slider_root.attr("data-invalid".into()), Some(""));
            assert_eq!(thumb.attr("data-required".into()), Some(""));
            assert_eq!(thumb.attr("data-invalid".into()), Some(""));
            assert_eq!(thumb.attr("aria-invalid".into()), Some("true"));
            assert_eq!(
                thumb.attr("aria-describedby".into()),
                Some("m2b-d-slider m2b-e-slider")
            );
            assert_eq!(
                thumb.attr("aria-required".into()),
                None,
                "role=\"slider\" must omit disallowed aria-required per WAI-ARIA 1.2"
            );
            assert!(
                hidden.attr("required".into()).is_some(),
                "HiddenFormInputs must receive required attribute"
            );
        }
        // Mutate controlled slider to 75.0, then reset form -> must emit mount-time 20.0
        click_and_pump(&mut harness, "#m2b-slider-set-75");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m2b-slider-reset-btn");
        advance_frame_and_pump(&mut harness, 0.016);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-ctl04-probe")).unwrap();
            assert_eq!(
                probe.attr("data-slider-reset-log".into()),
                Some("20"),
                "Controlled SliderRoot form reset must emit mount-time 20 (not default_value 5 or mutated 75)"
            );
        }

        // 4. RatingGroup inside FieldRoot: ARIA/data-invalid/FieldLabel + focus/blur
        {
            let base = harness.base();
            let rg_root = base
                .get_node(harness.node("[data-probe='m2b-rg-root']"))
                .unwrap();
            let rg_item = base.get_node(harness.node("#m2b-rg-item-0")).unwrap();
            let lbl = base.get_node(harness.node("#m2b-l-rg")).unwrap();
            assert_eq!(lbl.attr("for".into()), rg_root.attr("id".into()));
            assert_eq!(rg_root.attr("aria-invalid".into()), Some("true"));
            assert_eq!(rg_root.attr("data-invalid".into()), Some(""));
            assert_eq!(
                rg_root.attr("aria-describedby".into()),
                Some("m2b-d-rg m2b-e-rg")
            );
            assert_eq!(rg_item.attr("data-invalid".into()), Some(""));
        }
        focus_node(&mut harness, "[data-probe='m2b-rg-root']");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m2b-ctl04-outside");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m2b-ctl04-probe")).unwrap();
            assert_eq!(probe.attr("data-rating-events".into()), Some("focus,blur"));
        }
    }

    // -------------------------------------------------------------------------
    // Wave M3 Slice M3-A Failure-Mode Gates (FM-CTL-02, FM-CTL-03, FM-CTL-04)
    // -------------------------------------------------------------------------

    fn m3a_date(y: i32, m: u8, d: u8) -> monoxus::foundation::date_time::DateValue {
        monoxus::foundation::date_time::DateValue::Date(
            monoxus::foundation::date_time::CalendarDate::new(y, m, d).unwrap(),
        )
    }

    fn m3a_time(h: u8, m: u8) -> monoxus::foundation::date_time::TimeValue {
        monoxus::foundation::date_time::TimeValue::Time(
            monoxus::foundation::date_time::CalendarTime::from_hms(h, m, 0).unwrap(),
        )
    }

    /// FM-CTL-02: All 7 Pattern B roots under unidirectional `controlled_value: Option<ReadSignal<T>>`
    /// with rejecting `on_value_change`, `DateField` partial-digit entry preservation, and legacy
    /// two-way `Option<Signal<T>>` compatibility.
    #[component]
    fn M3aFmCtl02Scenario() -> Element {
        use monoxus::combobox::{
            ComboboxContentStatic, ComboboxInput, ComboboxItem, ComboboxItemData, ComboboxRoot,
            ComboboxTrigger,
        };
        use monoxus::date_field::{
            DateFieldInput, DateFieldRoot, DateFieldSegment, DateFieldSegmentItem,
        };
        use monoxus::date_picker::{
            DatePickerInput, DatePickerRoot, DatePickerSegment, DatePickerTrigger,
        };
        use monoxus::date_range_field::{
            DateRangeFieldInput, DateRangeFieldRoot, DateRangeFieldSegment, RangeFieldInputKind,
        };
        use monoxus::date_range_picker::{
            DateRangePickerInput, DateRangePickerRoot, DateRangePickerSegment,
        };
        use monoxus::foundation::date_time::{
            CalendarLocale, DateValue, DateValueRange, TimeGranularity, TimeRange, TimeValue,
        };
        use monoxus::time_field::{
            TimeFieldInput, TimeFieldRoot, TimeFieldSegment, TimeFieldSegmentItem,
        };
        use monoxus::time_range_field::{
            TimeRangeFieldInput, TimeRangeFieldRoot, TimeRangeFieldSegment,
        };

        // 1. Combobox: ReadOnly controlled_value + legacy two-way Signal
        let cbx_ro_src = use_signal(|| Some("a".to_string()));
        let cbx_ro: ReadSignal<Option<String>> = cbx_ro_src.into();
        let mut cbx_ro_proposed = use_signal(Vec::<String>::new);
        let cbx_tw = use_signal(|| Some("a".to_string()));

        // 2. DateField: ReadOnly controlled_value (complete + partial) + legacy two-way
        let df_ro_src = use_signal(|| Some(m3a_date(2025, 5, 10)));
        let df_ro: ReadSignal<Option<DateValue>> = df_ro_src.into();
        let mut df_ro_proposed = use_signal(Vec::<String>::new);
        let df_partial_src = use_signal(|| None::<DateValue>);
        let df_partial_ro: ReadSignal<Option<DateValue>> = df_partial_src.into();
        let mut df_partial_proposed = use_signal(Vec::<String>::new);
        let df_tw = use_signal(|| Some(m3a_date(2025, 5, 10)));

        // 3. TimeField: ReadOnly controlled_value + legacy two-way
        let tf_ro_src = use_signal(|| Some(m3a_time(9, 30)));
        let tf_ro: ReadSignal<Option<TimeValue>> = tf_ro_src.into();
        let mut tf_ro_proposed = use_signal(Vec::<String>::new);
        let tf_tw = use_signal(|| Some(m3a_time(9, 30)));

        // 4. DateRangeField: ReadOnly controlled_value + legacy two-way
        let drf_ro_src = use_signal(|| DateValueRange {
            start: Some(m3a_date(2025, 5, 10)),
            end: Some(m3a_date(2025, 5, 20)),
        });
        let drf_ro: ReadSignal<DateValueRange> = drf_ro_src.into();
        let mut drf_ro_proposed = use_signal(Vec::<String>::new);
        let drf_tw = use_signal(|| DateValueRange {
            start: Some(m3a_date(2025, 5, 10)),
            end: Some(m3a_date(2025, 5, 20)),
        });

        // 5. TimeRangeField: ReadOnly controlled_value + legacy two-way
        let trf_ro_src = use_signal(|| TimeRange {
            start: Some(m3a_time(9, 0)),
            end: Some(m3a_time(17, 0)),
        });
        let trf_ro: ReadSignal<TimeRange> = trf_ro_src.into();
        let mut trf_ro_proposed = use_signal(Vec::<String>::new);
        let trf_tw = use_signal(|| TimeRange {
            start: Some(m3a_time(9, 0)),
            end: Some(m3a_time(17, 0)),
        });

        // 6. DatePicker: ReadOnly controlled_value + legacy two-way
        let dp_ro_src = use_signal(|| Some(m3a_date(2025, 5, 10)));
        let dp_ro: ReadSignal<Option<DateValue>> = dp_ro_src.into();
        let mut dp_ro_proposed = use_signal(Vec::<String>::new);
        let dp_tw = use_signal(|| Some(m3a_date(2025, 5, 10)));

        // 7. DateRangePicker: ReadOnly controlled_value + legacy two-way
        let drp_ro_src = use_signal(|| DateValueRange {
            start: Some(m3a_date(2025, 5, 10)),
            end: Some(m3a_date(2025, 5, 20)),
        });
        let drp_ro: ReadSignal<DateValueRange> = drp_ro_src.into();
        let mut drp_ro_proposed = use_signal(Vec::<String>::new);
        let drp_tw = use_signal(|| DateValueRange {
            start: Some(m3a_date(2025, 5, 10)),
            end: Some(m3a_date(2025, 5, 20)),
        });

        let fmt_opt_date = |v: Option<DateValue>| {
            v.map(|d| d.to_iso_string())
                .unwrap_or_else(|| "none".to_string())
        };
        let fmt_opt_time = |v: Option<TimeValue>| {
            v.map(|t| t.time().to_iso_string())
                .unwrap_or_else(|| "none".to_string())
        };

        rsx! {
            div { style: "padding: 16px; display: flex; flex-direction: column; gap: 10px;",
                div {
                    id: "m3a-ctl02-probe",
                    "data-cbx-ro-sig": "{cbx_ro_src().unwrap_or_default()}",
                    "data-cbx-ro-prop": "{cbx_ro_proposed.read().join(\",\")}",
                    "data-cbx-tw-sig": "{cbx_tw().unwrap_or_default()}",
                    "data-df-ro-sig": "{fmt_opt_date(df_ro_src())}",
                    "data-df-ro-prop": "{df_ro_proposed.read().join(\",\")}",
                    "data-df-partial-sig": "{fmt_opt_date(df_partial_src())}",
                    "data-df-partial-prop": "{df_partial_proposed.read().join(\",\")}",
                    "data-df-tw-sig": "{fmt_opt_date(df_tw())}",
                    "data-tf-ro-sig": "{fmt_opt_time(tf_ro_src())}",
                    "data-tf-ro-prop": "{tf_ro_proposed.read().join(\",\")}",
                    "data-tf-tw-sig": "{fmt_opt_time(tf_tw())}",
                    "data-drf-ro-sig": "{fmt_opt_date(drf_ro_src().start)}..{fmt_opt_date(drf_ro_src().end)}",
                    "data-drf-ro-prop": "{drf_ro_proposed.read().join(\",\")}",
                    "data-drf-tw-sig": "{fmt_opt_date(drf_tw().start)}..{fmt_opt_date(drf_tw().end)}",
                    "data-trf-ro-sig": "{fmt_opt_time(trf_ro_src().start)}..{fmt_opt_time(trf_ro_src().end)}",
                    "data-trf-ro-prop": "{trf_ro_proposed.read().join(\",\")}",
                    "data-trf-tw-sig": "{fmt_opt_time(trf_tw().start)}..{fmt_opt_time(trf_tw().end)}",
                    "data-dp-ro-sig": "{fmt_opt_date(dp_ro_src())}",
                    "data-dp-ro-prop": "{dp_ro_proposed.read().join(\",\")}",
                    "data-dp-tw-sig": "{fmt_opt_date(dp_tw())}",
                    "data-drp-ro-sig": "{fmt_opt_date(drp_ro_src().start)}..{fmt_opt_date(drp_ro_src().end)}",
                    "data-drp-ro-prop": "{drp_ro_proposed.read().join(\",\")}",
                    "data-drp-tw-sig": "{fmt_opt_date(drp_tw().start)}..{fmt_opt_date(drp_tw().end)}",
                }

                // 1a. Combobox ReadOnly rejecting
                ComboboxRoot {
                    id: "m3a-cbx-ro",
                    name: Some("cbx_ro".to_string()),
                    controlled_value: Some(cbx_ro),
                    items: vec![
                        ComboboxItemData::new("a", "Alpha", false),
                        ComboboxItemData::new("b", "Beta", false),
                    ],
                    on_value_change: move |v: Option<String>| {
                        cbx_ro_proposed.write().push(v.unwrap_or_else(|| "none".to_string()));
                    },
                    ComboboxInput { id: "m3a-cbx-ro-in", style: "width: 120px; height: 24px;" }
                    ComboboxTrigger { id: "m3a-cbx-ro-trig", style: "width: 28px; height: 24px;", "v" }
                    ComboboxContentStatic {
                        ComboboxItem { id: "m3a-cbx-ro-a", value: "a", label: "Alpha", "Alpha" }
                        ComboboxItem { id: "m3a-cbx-ro-b", value: "b", label: "Beta", "Beta" }
                    }
                }

                // 1b. Combobox TwoWay legacy
                ComboboxRoot {
                    id: "m3a-cbx-tw",
                    value: Some(cbx_tw),
                    items: vec![
                        ComboboxItemData::new("a", "Alpha", false),
                        ComboboxItemData::new("b", "Beta", false),
                    ],
                    ComboboxInput { id: "m3a-cbx-tw-in", style: "width: 120px; height: 24px;" }
                    ComboboxTrigger { id: "m3a-cbx-tw-trig", style: "width: 28px; height: 24px;", "v" }
                    ComboboxContentStatic {
                        ComboboxItem { id: "m3a-cbx-tw-b", value: "b", label: "Beta", "Beta" }
                    }
                }

                // 2a. DateField ReadOnly rejecting
                DateFieldRoot {
                    id: "m3a-df-ro",
                    locale: CalendarLocale::EnUs,
                    controlled_value: Some(df_ro),
                    on_value_change: move |v: Option<DateValue>| {
                        df_ro_proposed.write().push(fmt_opt_date(v));
                    },
                    DateFieldInput {
                        id: "m3a-df-ro-in",
                        name: Some("df_ro".to_string()),
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-df-ro-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 2b. DateField ReadOnly partial-digit entry preservation
                DateFieldRoot {
                    id: "m3a-df-partial",
                    locale: CalendarLocale::EnUs,
                    controlled_value: Some(df_partial_ro),
                    on_value_change: move |v: Option<DateValue>| {
                        df_partial_proposed.write().push(fmt_opt_date(v));
                    },
                    DateFieldInput {
                        id: "m3a-df-partial-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-df-partial-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 2c. DateField TwoWay legacy
                DateFieldRoot {
                    id: "m3a-df-tw",
                    locale: CalendarLocale::EnUs,
                    value: Some(df_tw),
                    DateFieldInput {
                        id: "m3a-df-tw-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-df-tw-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 3a. TimeField ReadOnly rejecting
                TimeFieldRoot {
                    id: "m3a-tf-ro",
                    locale: CalendarLocale::Iso,
                    granularity: TimeGranularity::Minute,
                    controlled_value: Some(tf_ro),
                    on_value_change: move |v: Option<TimeValue>| {
                        tf_ro_proposed.write().push(fmt_opt_time(v));
                    },
                    TimeFieldInput {
                        id: "m3a-tf-ro-in",
                        render_children: move |segs: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segs {
                                TimeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-tf-ro-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 3b. TimeField TwoWay legacy
                TimeFieldRoot {
                    id: "m3a-tf-tw",
                    locale: CalendarLocale::Iso,
                    granularity: TimeGranularity::Minute,
                    value: Some(tf_tw),
                    TimeFieldInput {
                        id: "m3a-tf-tw-in",
                        render_children: move |segs: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segs {
                                TimeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-tf-tw-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 4a. DateRangeField ReadOnly rejecting
                DateRangeFieldRoot {
                    id: "m3a-drf-ro",
                    locale: CalendarLocale::EnUs,
                    controlled_value: Some(drf_ro),
                    on_value_change: move |v: DateValueRange| {
                        drf_ro_proposed.write().push(format!("{}..{}", fmt_opt_date(v.start), fmt_opt_date(v.end)));
                    },
                    DateRangeFieldInput {
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-drf-ro-start-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    DateRangeFieldInput {
                        kind: RangeFieldInputKind::End,
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-drf-ro-end-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 4b. DateRangeField TwoWay legacy
                DateRangeFieldRoot {
                    id: "m3a-drf-tw",
                    locale: CalendarLocale::EnUs,
                    value: Some(drf_tw),
                    DateRangeFieldInput {
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-drf-tw-start-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 5a. TimeRangeField ReadOnly rejecting
                TimeRangeFieldRoot {
                    id: "m3a-trf-ro",
                    locale: CalendarLocale::Iso,
                    granularity: TimeGranularity::Minute,
                    controlled_value: Some(trf_ro),
                    on_value_change: move |v: TimeRange| {
                        trf_ro_proposed.write().push(format!("{}..{}", fmt_opt_time(v.start), fmt_opt_time(v.end)));
                    },
                    TimeRangeFieldInput {
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segs: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segs {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-trf-ro-start-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 5b. TimeRangeField TwoWay legacy
                TimeRangeFieldRoot {
                    id: "m3a-trf-tw",
                    locale: CalendarLocale::Iso,
                    granularity: TimeGranularity::Minute,
                    value: Some(trf_tw),
                    TimeRangeFieldInput {
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segs: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segs {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-trf-tw-start-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 6a. DatePicker ReadOnly rejecting
                DatePickerRoot {
                    id: "m3a-dp-ro",
                    locale: CalendarLocale::EnUs,
                    controlled_value: Some(dp_ro),
                    on_value_change: move |v: Option<DateValue>| {
                        dp_ro_proposed.write().push(fmt_opt_date(v));
                    },
                    DatePickerInput {
                        id: "m3a-dp-ro-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DatePickerSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-dp-ro-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    DatePickerTrigger { id: "m3a-dp-ro-trig", "Cal" }
                }

                // 6b. DatePicker TwoWay legacy
                DatePickerRoot {
                    id: "m3a-dp-tw",
                    locale: CalendarLocale::EnUs,
                    value: Some(dp_tw),
                    DatePickerInput {
                        id: "m3a-dp-tw-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DatePickerSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-dp-tw-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 7a. DateRangePicker ReadOnly rejecting
                DateRangePickerRoot {
                    id: "m3a-drp-ro",
                    locale: CalendarLocale::EnUs,
                    controlled_value: Some(drp_ro),
                    on_value_change: move |v: DateValueRange| {
                        drp_ro_proposed.write().push(format!("{}..{}", fmt_opt_date(v.start), fmt_opt_date(v.end)));
                    },
                    DateRangePickerInput {
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateRangePickerSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-drp-ro-start-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 7b. DateRangePicker TwoWay legacy
                DateRangePickerRoot {
                    id: "m3a-drp-tw",
                    locale: CalendarLocale::EnUs,
                    value: Some(drp_tw),
                    DateRangePickerInput {
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateRangePickerSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-drp-tw-start-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn m3a_fm_ctl_02_pattern_b_read_only_rejection_partial_digits_and_two_way() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M3aFmCtl02Scenario);
        drain_polls_and_resolve(&mut harness);

        // 1. Combobox: ReadOnly rejects "b" (proposes "b", signal & DOM stay at "a"/"Alpha"); TwoWay mutates
        click_and_pump(&mut harness, "#m3a-cbx-ro-trig");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m3a-cbx-ro-b");
        drain_polls_and_resolve(&mut harness);

        click_and_pump(&mut harness, "#m3a-cbx-tw-trig");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m3a-cbx-tw-b");
        drain_polls_and_resolve(&mut harness);

        // 2a. DateField ReadOnly rejecting ArrowUp on day
        focus_node(&mut harness, "#m3a-df-ro-seg-day");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);

        // 2b. DateField ReadOnly partial digit entry ("0" in month) preserves local draft without firing on_value_change
        focus_node(&mut harness, "#m3a-df-partial-seg-month");
        send_key_and_pump(&mut harness, Key::Character("0".into()), Code::Digit0);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let df_partial_month = base
                .get_node(harness.node("#m3a-df-partial-seg-month"))
                .unwrap();
            assert_eq!(
                df_partial_month.text_content().trim(),
                "0",
                "Partial digit '0' in DateField month must stay intact while focused under ReadOnly"
            );
            let probe = base.get_node(harness.node("#m3a-ctl02-probe")).unwrap();
            assert_eq!(probe.attr("data-df-partial-sig".into()), Some("none"));
            assert_eq!(probe.attr("data-df-partial-prop".into()), Some(""));
        }
        // Complete the month digits ("3" -> "03") while day/year stay placeholder -> still None, draft stays "03"
        send_key_and_pump(&mut harness, Key::Character("3".into()), Code::Digit3);
        drain_polls_and_resolve(&mut harness);

        // 2c. DateField TwoWay ArrowUp mutates caller signal
        focus_node(&mut harness, "#m3a-df-tw-seg-day");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);

        // 3. TimeField ReadOnly rejecting + TwoWay
        focus_node(&mut harness, "#m3a-tf-ro-seg-hour");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-tf-tw-seg-hour");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);

        // 4. DateRangeField ReadOnly rejecting + TwoWay
        focus_node(&mut harness, "#m3a-drf-ro-start-day");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-drf-tw-start-day");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);

        // 5. TimeRangeField ReadOnly rejecting + TwoWay
        focus_node(&mut harness, "#m3a-trf-ro-start-hour");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-trf-tw-start-hour");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);

        // 6. DatePicker ReadOnly rejecting + TwoWay
        focus_node(&mut harness, "#m3a-dp-ro-seg-day");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-dp-tw-seg-day");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);

        // 7. DateRangePicker ReadOnly rejecting + TwoWay
        focus_node(&mut harness, "#m3a-drp-ro-start-day");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-drp-tw-start-day");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        drain_polls_and_resolve(&mut harness);

        let base = harness.base();
        let probe = base.get_node(harness.node("#m3a-ctl02-probe")).unwrap();

        // Verify Combobox
        assert_eq!(probe.attr("data-cbx-ro-sig".into()), Some("a"));
        assert_eq!(probe.attr("data-cbx-ro-prop".into()), Some("b"));
        assert_eq!(probe.attr("data-cbx-tw-sig".into()), Some("b"));
        let cbx_in = base.get_node(harness.node("#m3a-cbx-ro-in")).unwrap();
        assert_eq!(cbx_in.attr("value".into()), Some("Alpha"));
        let cbx_hidden = base.get_node(harness.node("input[name='cbx_ro']")).unwrap();
        assert_eq!(cbx_hidden.attr("value".into()), Some("a"));

        // Verify DateField ReadOnly rejection + partial draft + TwoWay
        assert_eq!(probe.attr("data-df-ro-sig".into()), Some("2025-05-10"));
        assert_eq!(probe.attr("data-df-ro-prop".into()), Some("2025-05-11"));
        assert_eq!(probe.attr("data-df-tw-sig".into()), Some("2025-05-11"));
        let df_day = base.get_node(harness.node("#m3a-df-ro-seg-day")).unwrap();
        assert_eq!(df_day.text_content().trim(), "10");
        assert_eq!(probe.attr("data-df-partial-sig".into()), Some("none"));
        assert_eq!(probe.attr("data-df-partial-prop".into()), Some(""));
        let df_partial_month = base
            .get_node(harness.node("#m3a-df-partial-seg-month"))
            .unwrap();
        assert_eq!(
            df_partial_month.text_content().trim(),
            "03",
            "Completed partial segment '03' in DateField month must stay intact under ReadOnly while full DateValue is None"
        );

        // Verify TimeField
        assert_eq!(probe.attr("data-tf-ro-sig".into()), Some("09:30:00"));
        assert_eq!(probe.attr("data-tf-ro-prop".into()), Some("10:30:00"));
        assert_eq!(probe.attr("data-tf-tw-sig".into()), Some("10:30:00"));
        let tf_hour = base.get_node(harness.node("#m3a-tf-ro-seg-hour")).unwrap();
        assert_eq!(tf_hour.text_content().trim(), "09");

        // Verify DateRangeField
        assert_eq!(
            probe.attr("data-drf-ro-sig".into()),
            Some("2025-05-10..2025-05-20")
        );
        assert_eq!(
            probe.attr("data-drf-ro-prop".into()),
            Some("2025-05-11..2025-05-20")
        );
        assert_eq!(
            probe.attr("data-drf-tw-sig".into()),
            Some("2025-05-11..2025-05-20")
        );
        let drf_day = base
            .get_node(harness.node("#m3a-drf-ro-start-day"))
            .unwrap();
        assert_eq!(drf_day.text_content().trim(), "10");

        // Verify TimeRangeField
        assert_eq!(
            probe.attr("data-trf-ro-sig".into()),
            Some("09:00:00..17:00:00")
        );
        assert_eq!(
            probe.attr("data-trf-ro-prop".into()),
            Some("10:00:00..17:00:00")
        );
        assert_eq!(
            probe.attr("data-trf-tw-sig".into()),
            Some("10:00:00..17:00:00")
        );
        let trf_hour = base
            .get_node(harness.node("#m3a-trf-ro-start-hour"))
            .unwrap();
        assert_eq!(trf_hour.text_content().trim(), "09");

        // Verify DatePicker
        assert_eq!(probe.attr("data-dp-ro-sig".into()), Some("2025-05-10"));
        assert_eq!(probe.attr("data-dp-ro-prop".into()), Some("2025-05-11"));
        assert_eq!(probe.attr("data-dp-tw-sig".into()), Some("2025-05-11"));
        let dp_day = base.get_node(harness.node("#m3a-dp-ro-seg-day")).unwrap();
        assert_eq!(dp_day.text_content().trim(), "10");

        // Verify DateRangePicker
        assert_eq!(
            probe.attr("data-drp-ro-sig".into()),
            Some("2025-05-10..2025-05-20")
        );
        assert_eq!(
            probe.attr("data-drp-ro-prop".into()),
            Some("2025-05-11..2025-05-20")
        );
        assert_eq!(
            probe.attr("data-drp-tw-sig".into()),
            Some("2025-05-11..2025-05-20")
        );
        let drp_day = base
            .get_node(harness.node("#m3a-drp-ro-start-day"))
            .unwrap();
        assert_eq!(drp_day.text_content().trim(), "10");
    }

    /// FM-CTL-03 (M3-A): Composite `on_focus_out` across Combobox (Input -> Trigger -> Content/Item),
    /// DateField (month -> day), DateRangeField (start -> end), DatePicker (segment -> trigger -> calendar),
    /// plus `#5894` toggle safety.
    #[component]
    fn M3aFmCtl03Scenario() -> Element {
        use monoxus::combobox::{
            ComboboxContentStatic, ComboboxInput, ComboboxItem, ComboboxRoot, ComboboxTrigger,
        };
        use monoxus::date_field::{
            DateFieldInput, DateFieldRoot, DateFieldSegment, DateFieldSegmentItem,
        };
        use monoxus::date_picker::{
            DatePickerContentStatic, DatePickerInput, DatePickerRoot, DatePickerSegment,
            DatePickerTrigger,
        };
        use monoxus::date_range_field::{
            DateRangeFieldInput, DateRangeFieldRoot, DateRangeFieldSegment, RangeFieldInputKind,
        };
        use monoxus::foundation::date_time::CalendarLocale;

        let mut cbx_out = use_signal(|| 0_u32);
        let mut cbx_input_blur = use_signal(|| 0_u32);
        let mut df_out = use_signal(|| 0_u32);
        let mut drf_out = use_signal(|| 0_u32);
        let mut dp_out = use_signal(|| 0_u32);
        let mut dp_trig_blur = use_signal(|| 0_u32);
        let mut attach = use_signal(|| true);

        let cbx_fo = attach().then(|| EventHandler::new(move |_: FocusEvent| cbx_out += 1));
        let cbx_blur =
            attach().then(|| EventHandler::new(move |_: FocusEvent| cbx_input_blur += 1));
        let df_fo = attach().then(|| EventHandler::new(move |_: FocusEvent| df_out += 1));
        let drf_fo = attach().then(|| EventHandler::new(move |_: FocusEvent| drf_out += 1));
        let dp_fo = attach().then(|| EventHandler::new(move |_: FocusEvent| dp_out += 1));
        let dp_blur = attach().then(|| EventHandler::new(move |_: FocusEvent| dp_trig_blur += 1));

        rsx! {
            div { style: "padding: 20px; display: flex; flex-direction: column; gap: 14px;",
                div {
                    id: "m3a-ctl03-probe",
                    "data-cbx-out": "{cbx_out()}",
                    "data-cbx-blur": "{cbx_input_blur()}",
                    "data-df-out": "{df_out()}",
                    "data-drf-out": "{drf_out()}",
                    "data-dp-out": "{dp_out()}",
                    "data-dp-trig-blur": "{dp_trig_blur()}",
                    "data-attached": "{attach()}",
                }
                button {
                    id: "m3a-ctl03-ext-btn",
                    r#type: "button",
                    style: "width: 90px; height: 28px;",
                    "External"
                }
                button {
                    id: "m3a-ctl03-toggle-5894",
                    r#type: "button",
                    style: "width: 120px; height: 28px;",
                    onclick: move |_| attach.toggle(),
                    "Toggle"
                }

                // 1. Combobox composite focus-out
                ComboboxRoot {
                    id: "m3a-f-cbx",
                    default_open: true,
                    on_focus_out: cbx_fo,
                    ComboboxInput {
                        id: "m3a-f-cbx-in",
                        on_blur: cbx_blur,
                        style: "width: 120px; height: 28px;",
                    }
                    ComboboxTrigger {
                        id: "m3a-f-cbx-trig",
                        style: "width: 28px; height: 28px;",
                        "v"
                    }
                    ComboboxContentStatic {
                        id: "m3a-f-cbx-content",
                        ComboboxItem {
                            id: "m3a-f-cbx-item-a",
                            value: "a",
                            tabindex: "0",
                            "Alpha"
                        }
                    }
                }

                // 2. DateField composite focus-out (month -> day)
                DateFieldRoot {
                    id: "m3a-f-df",
                    locale: CalendarLocale::EnUs,
                    on_focus_out: df_fo,
                    DateFieldInput {
                        id: "m3a-f-df-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-f-df-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 3. DateRangeField composite focus-out (start -> end)
                DateRangeFieldRoot {
                    id: "m3a-f-drf",
                    locale: CalendarLocale::EnUs,
                    on_focus_out: drf_fo,
                    DateRangeFieldInput {
                        kind: RangeFieldInputKind::Start,
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-f-drf-start-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    DateRangeFieldInput {
                        kind: RangeFieldInputKind::End,
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-f-drf-end-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 4. DatePicker composite focus-out (segment -> trigger -> calendar content)
                DatePickerRoot {
                    id: "m3a-f-dp",
                    locale: CalendarLocale::EnUs,
                    on_focus_out: dp_fo,
                    DatePickerInput {
                        id: "m3a-f-dp-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DatePickerSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-f-dp-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    DatePickerTrigger {
                        id: "m3a-f-dp-trig",
                        on_blur: dp_blur,
                        style: "width: 48px; height: 28px;",
                        "Pick"
                    }
                    DatePickerContentStatic {
                        id: "m3a-f-dp-content",
                        button {
                            id: "m3a-f-dp-cal-cell",
                            r#type: "button",
                            style: "width: 32px; height: 28px;",
                            "15"
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn m3a_fm_ctl_03_composite_focus_out_across_combobox_fields_and_pickers() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M3aFmCtl03Scenario);
        drain_polls_and_resolve(&mut harness);

        // 1. Combobox: Input -> Trigger -> Item inside Content must NOT fire on_focus_out
        focus_node(&mut harness, "#m3a-f-cbx-in");
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-f-cbx-trig");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-f-cbx-item-a");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-ctl03-probe")).unwrap();
            assert_eq!(
                probe.attr("data-cbx-out".into()),
                Some("0"),
                "Combobox internal Input -> Trigger -> Item move must not fire on_focus_out"
            );
            assert_eq!(probe.attr("data-cbx-blur".into()), Some("1"));
        }
        // Leave Combobox to external button -> fires on_focus_out once
        focus_node(&mut harness, "#m3a-ctl03-ext-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-ctl03-probe")).unwrap();
            assert_eq!(
                probe.attr("data-cbx-out".into()),
                Some("1"),
                "Leaving Combobox to external button must fire composite on_focus_out once"
            );
        }

        // 2. DateField: month -> day must NOT fire on_focus_out; leaving fires once
        focus_node(&mut harness, "#m3a-f-df-seg-month");
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-f-df-seg-day");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-ctl03-probe")).unwrap();
            assert_eq!(
                probe.attr("data-df-out".into()),
                Some("0"),
                "DateField month -> day move must not fire composite on_focus_out"
            );
        }
        focus_node(&mut harness, "#m3a-ctl03-ext-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-ctl03-probe")).unwrap();
            assert_eq!(
                probe.attr("data-df-out".into()),
                Some("1"),
                "Leaving DateField to external button must fire composite on_focus_out once"
            );
        }

        // 3. DateRangeField: start -> end must NOT fire on_focus_out; leaving fires once
        focus_node(&mut harness, "#m3a-f-drf-start-month");
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-f-drf-end-month");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-ctl03-probe")).unwrap();
            assert_eq!(
                probe.attr("data-drf-out".into()),
                Some("0"),
                "DateRangeField start -> end move must not fire composite on_focus_out"
            );
        }
        focus_node(&mut harness, "#m3a-ctl03-ext-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-ctl03-probe")).unwrap();
            assert_eq!(
                probe.attr("data-drf-out".into()),
                Some("1"),
                "Leaving DateRangeField to external button must fire composite on_focus_out once"
            );
        }

        // 4. DatePicker: segment -> trigger -> calendar must NOT fire on_focus_out; leaving fires once
        focus_node(&mut harness, "#m3a-f-dp-seg-month");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m3a-f-dp-trig");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-f-dp-cal-cell");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-ctl03-probe")).unwrap();
            assert_eq!(
                probe.attr("data-dp-out".into()),
                Some("0"),
                "DatePicker segment -> trigger -> calendar move must not fire composite on_focus_out"
            );
            assert_eq!(
                probe.attr("data-dp-trig-blur".into()),
                Some("1"),
                "DatePickerTrigger on_blur must fire when focus moves to calendar cell"
            );
        }
        focus_node(&mut harness, "#m3a-ctl03-ext-btn");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-ctl03-probe")).unwrap();
            assert_eq!(
                probe.attr("data-dp-out".into()),
                Some("1"),
                "Leaving DatePicker to external button must fire composite on_focus_out once"
            );
        }

        // 5. Dioxus #5894 toggle safety Some <-> None across renders
        click_and_pump(&mut harness, "#m3a-ctl03-toggle-5894");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m3a-ctl03-toggle-5894");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-ctl03-probe")).unwrap();
            assert_eq!(probe.attr("data-attached".into()), Some("true"));
        }
    }

    /// FM-CTL-04 (M3-A): `DatePickerRoot` & `DateRangePickerRoot` `is_invalid_override: Some(true)`
    /// emitting `data-invalid=""` and `aria-invalid="true"` on rendered segments/inputs, plus
    /// `FieldRoot` / `FieldContext` wiring on `Combobox`, `DateField`, `DateRangeField`, `DatePicker`,
    /// and `DateRangePicker`.
    #[component]
    fn M3aFmCtl04Scenario() -> Element {
        use monoxus::combobox::{ComboboxInput, ComboboxRoot, ComboboxTrigger};
        use monoxus::date_field::{
            DateFieldInput, DateFieldRoot, DateFieldSegment, DateFieldSegmentItem,
        };
        use monoxus::date_picker::{DatePickerInput, DatePickerRoot, DatePickerSegment};
        use monoxus::date_range_field::{
            DateRangeFieldInput, DateRangeFieldRoot, DateRangeFieldSegment, RangeFieldInputKind,
        };
        use monoxus::date_range_picker::{
            DateRangePickerInput, DateRangePickerRoot, DateRangePickerSegment,
        };
        use monoxus::foundation::date_time::CalendarLocale;

        rsx! {
            div { style: "padding: 16px; display: flex; flex-direction: column; gap: 12px;",
                // 1. DatePickerRoot with explicit is_invalid_override: Some(true)
                DatePickerRoot {
                    id: "m3a-inv-dp",
                    locale: CalendarLocale::EnUs,
                    is_invalid_override: Some(true),
                    DatePickerInput {
                        id: "m3a-inv-dp-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DatePickerSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-inv-dp-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 2. DateRangePickerRoot with explicit is_invalid_override: Some(true)
                DateRangePickerRoot {
                    id: "m3a-inv-drp",
                    locale: CalendarLocale::EnUs,
                    is_invalid_override: Some(true),
                    DateRangePickerInput {
                        kind: RangeFieldInputKind::Start,
                        id: "m3a-inv-drp-start-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateRangePickerSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-inv-drp-start-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                    DateRangePickerInput {
                        kind: RangeFieldInputKind::End,
                        id: "m3a-inv-drp-end-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateRangePickerSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-inv-drp-end-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 3. Combobox inside FieldRoot + FieldControl
                FieldRoot {
                    id: "m3a-fld-cbx",
                    invalid: true,
                    required: true,
                    FieldControl {
                        FieldLabel { id: "m3a-lbl-cbx", "Fruit" }
                        ComboboxRoot {
                            id: "m3a-cbx-fld-root",
                            ComboboxInput { "data-probe": "m3a-cbx-fld-in" }
                            ComboboxTrigger { id: "m3a-cbx-fld-trig", "v" }
                        }
                    }
                    FieldDescription { id: "m3a-desc-cbx", "Pick a fruit" }
                    FieldError { id: "m3a-err-cbx", "Fruit required" }
                }

                // 4. DateField inside FieldRoot
                FieldRoot {
                    id: "m3a-fld-df",
                    invalid: true,
                    required: true,
                    DateFieldRoot {
                        id: "m3a-df-fld-root",
                        locale: CalendarLocale::EnUs,
                        DateFieldInput {
                            id: "m3a-df-fld-in",
                            render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segs {
                                    DateFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("m3a-df-fld-seg-{}", seg.kind().as_data_segment()),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                    }
                                }
                            }
                        }
                    }
                    FieldDescription { id: "m3a-desc-df", "Enter DOB" }
                    FieldError { id: "m3a-err-df", "DOB invalid" }
                }

                // 5. DateRangeField inside FieldRoot
                FieldRoot {
                    id: "m3a-fld-drf",
                    invalid: true,
                    DateRangeFieldRoot {
                        id: "m3a-drf-fld-root",
                        locale: CalendarLocale::EnUs,
                        DateRangeFieldInput {
                            kind: RangeFieldInputKind::Start,
                            id: "m3a-drf-fld-start-in",
                            render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segs {
                                    DateRangeFieldSegment {
                                        key: "{seg.index}",
                                        id: format!("m3a-drf-fld-seg-{}", seg.kind().as_data_segment()),
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                    }
                                }
                            }
                        }
                    }
                    FieldDescription { id: "m3a-desc-drf", "Trip dates" }
                    FieldError { id: "m3a-err-drf", "Invalid trip range" }
                }
            }
        }
    }

    #[test]
    fn m3a_fm_ctl_04_picker_invalid_override_and_field_context_wiring() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M3aFmCtl04Scenario);
        drain_polls_and_resolve(&mut harness);

        let base = harness.base();

        // 1. DatePickerRoot is_invalid_override: Some(true)
        for sel in ["#m3a-inv-dp-in", "#m3a-inv-dp-seg-month", "#m3a-inv-dp-seg-day", "#m3a-inv-dp-seg-year"] {
            let node = base.get_node(harness.node(sel)).unwrap();
            assert_eq!(node.attr("data-invalid".into()), Some(""), "{sel} must emit data-invalid=\"\"");
            assert_eq!(node.attr("aria-invalid".into()), Some("true"), "{sel} must emit aria-invalid=\"true\"");
        }

        // 2. DateRangePickerRoot is_invalid_override: Some(true)
        for sel in [
            "#m3a-inv-drp-start-in",
            "#m3a-inv-drp-start-month",
            "#m3a-inv-drp-end-in",
            "#m3a-inv-drp-end-day",
        ] {
            let node = base.get_node(harness.node(sel)).unwrap();
            assert_eq!(node.attr("data-invalid".into()), Some(""), "{sel} must emit data-invalid=\"\"");
            assert_eq!(node.attr("aria-invalid".into()), Some("true"), "{sel} must emit aria-invalid=\"true\"");
        }

        // 3. Combobox inside FieldRoot + FieldControl
        let cbx_lbl = base.get_node(harness.node("#m3a-lbl-cbx")).unwrap();
        let cbx_in = base
            .get_node(harness.node("[data-probe='m3a-cbx-fld-in']"))
            .unwrap();
        let cbx_trig = base.get_node(harness.node("#m3a-cbx-fld-trig")).unwrap();
        assert_eq!(cbx_lbl.attr("for".into()), cbx_in.attr("id".into()));
        for node in [cbx_in, cbx_trig] {
            assert_eq!(node.attr("aria-invalid".into()), Some("true"));
            assert_eq!(node.attr("data-invalid".into()), Some(""));
            assert_eq!(node.attr("aria-required".into()), Some("true"));
            assert_eq!(
                node.attr("aria-describedby".into()),
                Some("m3a-desc-cbx m3a-err-cbx")
            );
        }

        // 4. DateField inside FieldRoot
        let df_in = base.get_node(harness.node("#m3a-df-fld-in")).unwrap();
        let df_seg = base.get_node(harness.node("#m3a-df-fld-seg-month")).unwrap();
        assert_eq!(df_in.attr("aria-invalid".into()), Some("true"));
        assert_eq!(df_in.attr("data-invalid".into()), Some(""));
        assert_eq!(df_in.attr("aria-required".into()), Some("true"));
        assert_eq!(
            df_in.attr("aria-describedby".into()),
            Some("m3a-desc-df m3a-err-df")
        );
        assert_eq!(df_seg.attr("aria-invalid".into()), Some("true"));
        assert_eq!(df_seg.attr("data-invalid".into()), Some(""));

        // 5. DateRangeField inside FieldRoot
        let drf_root = base.get_node(harness.node("#m3a-drf-fld-root")).unwrap();
        let drf_seg = base.get_node(harness.node("#m3a-drf-fld-seg-month")).unwrap();
        assert_eq!(drf_root.attr("aria-invalid".into()), Some("true"));
        assert_eq!(drf_root.attr("data-invalid".into()), Some(""));
        assert_eq!(
            drf_root.attr("aria-describedby".into()),
            Some("m3a-desc-drf m3a-err-drf")
        );
        assert_eq!(drf_seg.attr("aria-invalid".into()), Some("true"));
        assert_eq!(drf_seg.attr("data-invalid".into()), Some(""));
    }

    /// Audit #28 regressions:
    /// 1. `DateFieldRoot` mounted with `on_focus_out: None`, later toggled to `Some(cb)`, must fire
    ///    `cb` on focus exit even when `DateFieldInput` props are unchanged.
    /// 2. `ComboboxRoot` with `controlled_value: Option<ReadSignal<_>>` rejecting a `<form>` reset
    ///    must NOT overwrite uncontrolled `ComboboxInput` text to `default_input`.
    #[component]
    fn M3aAuditRegressionsScenario() -> Element {
        use monoxus::button::{Button, ButtonType};
        use monoxus::combobox::{
            ComboboxContentStatic, ComboboxInput, ComboboxItem, ComboboxItemData, ComboboxRoot,
            ComboboxTrigger,
        };
        use monoxus::date_field::{
            DateFieldInput, DateFieldRoot, DateFieldSegment, DateFieldSegmentItem,
        };
        use monoxus::foundation::date_time::CalendarLocale;

        let mut attach_df_focus_out = use_signal(|| false);
        let mut df_late_out = use_signal(|| 0_u32);
        let df_focus_out = attach_df_focus_out()
            .then(|| EventHandler::new(move |_: FocusEvent| df_late_out += 1));

        let mut cbx_val = use_signal(|| Some("a".to_string()));
        let mut cbx_allow_change = use_signal(|| true);
        let mut cbx_proposed = use_signal(Vec::<String>::new);

        rsx! {
            div { style: "padding: 20px; display: flex; flex-direction: column; gap: 14px;",
                div {
                    id: "m3a-audit-probe",
                    "data-df-late-out": "{df_late_out()}",
                    "data-df-attached": "{attach_df_focus_out()}",
                    "data-cbx-val": "{cbx_val().unwrap_or_else(|| \"none\".to_string())}",
                    "data-cbx-proposed": "{cbx_proposed.read().join(\",\")}",
                }
                button {
                    id: "m3a-audit-ext",
                    r#type: "button",
                    style: "width: 90px; height: 28px;",
                    "External"
                }
                button {
                    id: "m3a-audit-enable-df-fo",
                    r#type: "button",
                    style: "width: 140px; height: 28px;",
                    onclick: move |_| attach_df_focus_out.set(true),
                    "Attach FocusOut"
                }

                // 1. DateFieldRoot starting with on_focus_out: None
                DateFieldRoot {
                    id: "m3a-audit-df",
                    locale: CalendarLocale::EnUs,
                    on_focus_out: df_focus_out,
                    DateFieldInput {
                        id: "m3a-audit-df-in",
                        render_children: move |segs: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segs {
                                DateFieldSegment {
                                    key: "{seg.index}",
                                    id: format!("m3a-audit-df-seg-{}", seg.kind().as_data_segment()),
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                }
                            }
                        }
                    }
                }

                // 2. ComboboxRoot with ReadOnly controlled_value + uncontrolled input_value inside <form>
                form {
                    id: "m3a-audit-cbx-form",
                    button {
                        id: "m3a-audit-cbx-lock",
                        r#type: "button",
                        style: "width: 100px; height: 26px;",
                        onclick: move |_| cbx_allow_change.set(false),
                        "Reject Next"
                    }
                    ComboboxRoot {
                        id: "m3a-audit-cbx",
                        name: Some("audit_cbx".to_string()),
                        controlled_value: Some(cbx_val.into()),
                        items: vec![
                            ComboboxItemData::new("a", "Alpha", false),
                            ComboboxItemData::new("b", "Beta", false),
                        ],
                        on_value_change: move |next: Option<String>| {
                            cbx_proposed
                                .write()
                                .push(next.clone().unwrap_or_else(|| "none".to_string()));
                            if *cbx_allow_change.peek() {
                                cbx_val.set(next);
                            }
                        },
                        ComboboxInput {
                            id: "m3a-audit-cbx-in",
                            style: "width: 140px; height: 28px;",
                        }
                        ComboboxTrigger {
                            id: "m3a-audit-cbx-trig",
                            style: "width: 28px; height: 28px;",
                            "v"
                        }
                        ComboboxContentStatic {
                            ComboboxItem { id: "m3a-audit-cbx-a", value: "a", label: "Alpha", "Alpha" }
                            ComboboxItem { id: "m3a-audit-cbx-b", value: "b", label: "Beta", "Beta" }
                        }
                    }
                    Button {
                        id: "m3a-audit-cbx-reset".to_string(),
                        r#type: ButtonType::Reset,
                        style: "width: 80px; height: 26px;".to_string(),
                        "Reset"
                    }
                }
            }
        }
    }

    #[test]
    fn m3a_audit_regressions_date_field_late_focus_out_and_combobox_readonly_reset() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(M3aAuditRegressionsScenario);
        drain_polls_and_resolve(&mut harness);

        // 1. Toggle DateFieldRoot on_focus_out None -> Some(cb), then focus segment and leave
        click_and_pump(&mut harness, "#m3a-audit-enable-df-fo");
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-audit-df-seg-month");
        drain_polls_and_resolve(&mut harness);
        focus_node(&mut harness, "#m3a-audit-ext");
        advance_frame_and_pump(&mut harness, 0.02);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-audit-probe")).unwrap();
            assert_eq!(probe.attr("data-df-attached".into()), Some("true"));
            assert_eq!(
                probe.attr("data-df-late-out".into()),
                Some("1"),
                "DateFieldRoot toggling on_focus_out None -> Some must fire handler on focus exit"
            );
        }

        // 2. Combobox: accept selection "b" (uncontrolled input becomes "Beta"), then lock & reset form
        click_and_pump(&mut harness, "#m3a-audit-cbx-trig");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m3a-audit-cbx-b");
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let cbx_in = base.get_node(harness.node("#m3a-audit-cbx-in")).unwrap();
            assert_eq!(cbx_in.attr("value".into()), Some("Beta"));
            let probe = base.get_node(harness.node("#m3a-audit-probe")).unwrap();
            assert_eq!(probe.attr("data-cbx-val".into()), Some("b"));
        }

        click_and_pump(&mut harness, "#m3a-audit-cbx-lock");
        drain_polls_and_resolve(&mut harness);
        click_and_pump(&mut harness, "#m3a-audit-cbx-reset");
        advance_frame_and_pump(&mut harness, 0.016);
        drain_polls_and_resolve(&mut harness);
        {
            let base = harness.base();
            let probe = base.get_node(harness.node("#m3a-audit-probe")).unwrap();
            assert_eq!(
                probe.attr("data-cbx-proposed".into()),
                Some("b,a"),
                "Form reset must propose mount-time baseline 'a' via on_value_change"
            );
            assert_eq!(
                probe.attr("data-cbx-val".into()),
                Some("b"),
                "Rejected ReadOnly reset must leave controlled signal at 'b'"
            );
            let hidden = base.get_node(harness.node("input[name='audit_cbx']")).unwrap();
            assert_eq!(hidden.attr("value".into()), Some("b"));
            let cbx_in = base.get_node(harness.node("#m3a-audit-cbx-in")).unwrap();
            assert_eq!(
                cbx_in.attr("value".into()),
                Some("Beta"),
                "Rejected ReadOnly form reset must not overwrite uncontrolled ComboboxInput text to 'Alpha'"
            );
        }
    }
}

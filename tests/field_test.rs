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
    #[component]
    fn ToggleHarness() -> Element {
        let mut attach = use_signal(|| true);
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
                button {
                    onclick: move |_| attach.toggle(),
                    "Flip"
                }
            }
        }
    }

    let mut dom = VirtualDom::new(ToggleHarness);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
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
}

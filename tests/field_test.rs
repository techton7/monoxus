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
}

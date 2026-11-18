use dioxus::prelude::*;
use monoxus::foundation::form::{
    HIDDEN_INPUT_VISUAL_CLAMP_STYLE, HiddenFormInput, HiddenFormInputProps, HiddenFormInputs,
    HiddenInputAttributes, HiddenInputKind, collect_form_data, should_render_hidden_input,
    use_form_control_sync,
};

#[cfg(feature = "native")]
mod common;

#[test]
fn test_hidden_input_pure_contract() {
    let props = HiddenFormInputProps {
        id: Some("hidden-cb".to_string()),
        kind: HiddenInputKind::Checkbox,
        name: Some("newsletter".to_string()),
        value: "weekly".to_string(),
        checked: Some(true),
        disabled: false,
        required: true,
        read_only: false,
        form: Some("prefs-form".to_string()),
        autocomplete: Some("off".to_string()),
        focus_target_id: Some("cb-root".to_string()),
        reset_epoch: 0,
    };

    let attrs = HiddenInputAttributes::from_props(&props);
    assert_eq!(attrs.r#type, "checkbox");
    assert_eq!(attrs.name.as_deref(), Some("newsletter"));
    assert_eq!(attrs.value, "weekly");
    assert_eq!(attrs.checked, Some(true));
    assert!(attrs.required);
    assert!(!attrs.disabled);
    assert_eq!(attrs.form.as_deref(), Some("prefs-form"));
    assert_eq!(attrs.autocomplete.as_deref(), Some("off"));
    assert_eq!(attrs.aria_hidden, "true");
    assert_eq!(attrs.tabindex, -1);
    assert_eq!(attrs.style, HIDDEN_INPUT_VISUAL_CLAMP_STYLE);

    assert!(!should_render_hidden_input(None, None, false));
    assert!(should_render_hidden_input(Some("newsletter"), None, false));
    assert!(should_render_hidden_input(None, Some("prefs-form"), false));
    assert!(should_render_hidden_input(None, None, true));

    assert!(collect_form_data("missing").is_err());
}

#[component]
fn StandaloneVdomApp() -> Element {
    let mut val = use_signal(|| true);
    let should_render = use_form_control_sync("vdom-ctrl", Some("accept"), None, move || {
        val.set(false);
    });

    rsx! {
        div {
            button { id: "vdom-ctrl", r#type: "button", "Control" }
            if should_render {
                HiddenFormInput {
                    id: "vdom-hidden",
                    kind: HiddenInputKind::Checkbox,
                    name: "accept",
                    checked: val(),
                    focus_target_id: "vdom-ctrl",
                }
            }
            HiddenFormInputs {
                name: "multi",
                values: vec!["a".to_string(), "b".to_string()],
            }
        }
    }
}

#[test]
fn test_hidden_form_input_vdom_mounts_cleanly() {
    let mut dom = VirtualDom::new(StandaloneVdomApp);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use monoxus::button::{Button, ButtonType};

    #[component]
    fn SiblingOrderAndFocusApp() -> Element {
        let mut clicks = use_signal(|| 0_u32);
        rsx! {
            div {
                id: "flex-row",
                style: "display: flex; flex-direction: row; gap: 16px; width: 400px; height: 60px;",
                label {
                    id: "implicit-label",
                    style: "display: inline-flex; align-items: center; gap: 8px; width: 160px; height: 36px;",
                    button {
                        id: "ctrl-btn",
                        r#type: "button",
                        style: "width: 24px; height: 24px;",
                        onclick: move |_| clicks += 1,
                        "X"
                    }
                    HiddenFormInput {
                        id: "ctrl-hidden",
                        kind: HiddenInputKind::Checkbox,
                        name: "agree",
                        value: "accepted",
                        checked: true,
                        required: true,
                        focus_target_id: "ctrl-btn",
                    }
                    span { id: "label-caption", style: "width: 80px; height: 20px;", "Agree" }
                }
                div {
                    id: "sibling-after",
                    "data-clicks": "{clicks}",
                    style: "width: 80px; height: 36px;",
                    "Next"
                }
                label {
                    id: "direct-hidden-label",
                    r#for: "ctrl-hidden",
                    style: "width: 80px; height: 36px;",
                    "Focus Hidden"
                }
            }
        }
    }

    #[test]
    fn test_native_hidden_form_input_attributes_label_order_and_focus_forwarding() {
        let mut harness = mount_native_scenario(SiblingOrderAndFocusApp);

        let btn_id = harness.node("#ctrl-btn");
        let hidden_id = harness.node("#ctrl-hidden");
        let label_id = harness.node("#implicit-label");

        {
            let base = harness.base();
            let hidden_node = base.get_node(hidden_id).unwrap();
            assert_eq!(hidden_node.attr("type".into()), Some("checkbox"));
            assert_eq!(hidden_node.attr("name".into()), Some("agree"));
            assert_eq!(hidden_node.attr("value".into()), Some("accepted"));
            assert_eq!(hidden_node.attr("aria-hidden".into()), Some("true"));
            assert_eq!(hidden_node.attr("tabindex".into()), Some("-1"));
            assert_eq!(hidden_node.attr("autocomplete".into()), Some("off"));
            assert_eq!(
                hidden_node.attr("style".into()),
                Some(HIDDEN_INPUT_VISUAL_CLAMP_STYLE)
            );
            assert!(!hidden_node.is_focussable());

            // Verify `<button>` precedes `<input>` in `<label>` child order
            let label_children = &base.get_node(label_id).unwrap().children;
            let btn_pos = label_children.iter().position(|&c| c == btn_id).unwrap();
            let hidden_pos = label_children.iter().position(|&c| c == hidden_id).unwrap();
            assert!(
                btn_pos < hidden_pos,
                "<button> must precede hidden <input> so implicit <label> binds to <button>"
            );
        }

        // Verify `label_bound_input_element` resolves to `#ctrl-btn` (first labelable child) rather than `#ctrl-hidden`,
        // and clicking implicit `<label>` caption focuses `#ctrl-btn`, NOT `#ctrl-hidden`
        assert_eq!(
            harness
                .base()
                .label_bound_input_element(label_id)
                .map(|n| n.id),
            Some(btn_id)
        );
        click_and_pump(&mut harness, "#label-caption");
        assert_native_focus_id(&harness, "#ctrl-btn");
        let after_id = harness.node("#sibling-after");
        assert_eq!(
            harness
                .base()
                .get_node(after_id)
                .unwrap()
                .attr("data-clicks".into()),
            Some("1")
        );

        // Direct click on `#ctrl-btn` is never intercepted by out-of-flow `#ctrl-hidden` (`pointer-events: none`)
        click_and_pump(&mut harness, "#ctrl-btn");
        assert_eq!(
            harness
                .base()
                .get_node(after_id)
                .unwrap()
                .attr("data-clicks".into()),
            Some("2")
        );

        // Blur `#ctrl-btn`, then trigger real native Focus on `#ctrl-hidden` via `<label for="ctrl-hidden">`
        // -> verifies `HiddenFormInput`'s `onfocus` delegates focus immediately to `#ctrl-btn`
        harness.base_mut().clear_focus();
        assert!(harness.base().active_focus_node_id().is_none());
        click_and_pump(&mut harness, "#direct-hidden-label");
        assert_native_focus_id(&harness, "#ctrl-btn");
    }

    #[component]
    fn FormResetAndCollectApp() -> Element {
        let mut enclosed_checked = use_signal(|| false);
        let mut ext_checked = use_signal(|| true);
        let mut anon_checked = use_signal(|| false);

        let render_enclosed = use_form_control_sync("enclosed-btn", Some("opt_a"), None, move || {
            enclosed_checked.set(false);
        });
        let render_ext =
            use_form_control_sync("ext-btn", Some("opt_ext"), Some("named-form"), move || {
                ext_checked.set(true);
            });
        // Nameless control inside anonymous `<form>` (relies on `has_form_ancestor` detection)
        let render_anon = use_form_control_sync("anon-btn", None, None, move || {
            anon_checked.set(false);
        });

        rsx! {
            div {
                style: "width: 800px; height: 600px;",
                form {
                    id: "named-form",
                    style: "width: 600px; height: 120px;",
                    button {
                        id: "enclosed-btn",
                        r#type: "button",
                        "data-checked": "{enclosed_checked}",
                        style: "width: 60px; height: 32px;",
                        onclick: move |_| enclosed_checked.set(!enclosed_checked()),
                        "Toggle Enclosed"
                    }
                    if render_enclosed {
                        HiddenFormInput {
                            id: "enclosed-hidden",
                            kind: HiddenInputKind::Checkbox,
                            name: "opt_a",
                            value: "alpha",
                            checked: enclosed_checked(),
                            focus_target_id: "enclosed-btn",
                        }
                    }
                    HiddenFormInputs {
                        name: "tags",
                        values: vec!["rust".to_string(), "wasm".to_string()],
                    }
                    Button {
                        id: "named-reset-btn".to_string(),
                        r#type: ButtonType::Reset,
                        style: "width: 80px; height: 32px;".to_string(),
                        "Reset Named"
                    }
                }

                div {
                    style: "width: 600px; height: 80px;",
                    button {
                        id: "ext-btn",
                        r#type: "button",
                        form: "named-form",
                        "data-checked": "{ext_checked}",
                        style: "width: 60px; height: 32px;",
                        onclick: move |_| ext_checked.set(!ext_checked()),
                        "Toggle Ext"
                    }
                    if render_ext {
                        HiddenFormInput {
                            id: "ext-hidden",
                            kind: HiddenInputKind::Checkbox,
                            name: "opt_ext",
                            value: "external",
                            checked: ext_checked(),
                            form: "named-form",
                            focus_target_id: "ext-btn",
                        }
                    }
                }

                // Anonymous `<form>` without `id`
                form {
                    style: "width: 600px; height: 120px;",
                    button {
                        id: "anon-btn",
                        r#type: "button",
                        "data-checked": "{anon_checked}",
                        "data-has-input": "{render_anon}",
                        style: "width: 60px; height: 32px;",
                        onclick: move |_| anon_checked.set(!anon_checked()),
                        "Toggle Anon"
                    }
                    if render_anon {
                        HiddenFormInput {
                            id: "anon-hidden",
                            kind: HiddenInputKind::Checkbox,
                            checked: anon_checked(),
                            focus_target_id: "anon-btn",
                        }
                    }
                    Button {
                        id: "anon-reset-btn".to_string(),
                        r#type: ButtonType::Reset,
                        style: "width: 80px; height: 32px;".to_string(),
                        "Reset Anon"
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_form_reset_runtime_guard_button_reset_and_collect_form_data() {
        let mut harness = mount_native_scenario(FormResetAndCollectApp);
        // Pump effects so `use_form_control_sync` registers observers and detects ancestor `<form>`
        advance_frame_and_pump(&mut harness, 0.016);

        let enclosed_id = harness.node("#enclosed-btn");
        let ext_id = harness.node("#ext-btn");
        let anon_id = harness.node("#anon-btn");

        // Anonymous `<form>` ancestor was detected even without `name` or `form` prop
        assert_eq!(
            harness.base().get_node(anon_id).unwrap().attr("data-has-input".into()),
            Some("true")
        );
        assert!(harness.query("#anon-hidden").is_some());

        // Initial serialized entries on `#named-form`: `opt_a` is false (omitted), `tags` has 2, `opt_ext` is true
        assert_eq!(
            collect_form_data("named-form").expect("collect_form_data initial"),
            vec![
                ("tags".to_string(), "rust".to_string()),
                ("tags".to_string(), "wasm".to_string()),
                ("opt_ext".to_string(), "external".to_string()),
            ]
        );

        // Mutate all 3 controls
        click_and_pump(&mut harness, "#enclosed-btn");
        click_and_pump(&mut harness, "#ext-btn");
        click_and_pump(&mut harness, "#anon-btn");

        assert_eq!(
            harness.base().get_node(enclosed_id).unwrap().attr("data-checked".into()),
            Some("true")
        );
        assert_eq!(
            harness.base().get_node(ext_id).unwrap().attr("data-checked".into()),
            Some("false")
        );
        assert_eq!(
            harness.base().get_node(anon_id).unwrap().attr("data-checked".into()),
            Some("true")
        );
        assert_eq!(
            collect_form_data("named-form").expect("collect_form_data mutated"),
            vec![
                ("opt_a".to_string(), "alpha".to_string()),
                ("tags".to_string(), "rust".to_string()),
                ("tags".to_string(), "wasm".to_string()),
            ]
        );

        // 1. Test `start_form_reset_monitor` `RuntimeGuard` safety OUTSIDE any VDOM event turn:
        oxidase::dom::Document::current()
            .expect("active native doc")
            .dispatch_form_reset("named-form");
        advance_frame_and_pump(&mut harness, 0.016);

        assert_eq!(
            harness.base().get_node(enclosed_id).unwrap().attr("data-checked".into()),
            Some("false"),
            "Enclosed control must reset cleanly outside VDOM turn without panic"
        );
        assert_eq!(
            harness.base().get_node(ext_id).unwrap().attr("data-checked".into()),
            Some("true"),
            "External `form=\"named-form\"` control must reset cleanly"
        );
        assert_eq!(
            harness.base().get_node(anon_id).unwrap().attr("data-checked".into()),
            Some("true"),
            "Anonymous form control must remain untouched by `#named-form` reset"
        );

        // 2. Mutate `#named-form` controls again and reset via real `<Button r#type=ButtonType::Reset>` click
        click_and_pump(&mut harness, "#enclosed-btn");
        click_and_pump(&mut harness, "#ext-btn");
        assert_eq!(
            harness.base().get_node(enclosed_id).unwrap().attr("data-checked".into()),
            Some("true")
        );
        click_and_pump(&mut harness, "#named-reset-btn");
        assert_eq!(
            harness.base().get_node(enclosed_id).unwrap().attr("data-checked".into()),
            Some("false")
        );
        assert_eq!(
            harness.base().get_node(ext_id).unwrap().attr("data-checked".into()),
            Some("true")
        );
        assert_eq!(
            collect_form_data("named-form").expect("collect_form_data after button reset"),
            vec![
                ("tags".to_string(), "rust".to_string()),
                ("tags".to_string(), "wasm".to_string()),
                ("opt_ext".to_string(), "external".to_string()),
            ]
        );

        // 3. Click `<Button r#type=ButtonType::Reset>` inside anonymous `<form>` -> resets `#anon-btn`
        click_and_pump(&mut harness, "#anon-reset-btn");
        assert_eq!(
            harness.base().get_node(anon_id).unwrap().attr("data-checked".into()),
            Some("false")
        );
    }

    #[component]
    fn ResetEpochClobberScenarioApp() -> Element {
        use monoxus::button::{Button, ButtonType};
        use monoxus::checkbox::{Checkbox, CheckboxGroup, CheckedState};
        use monoxus::radio_group::{RadioGroupItem, RadioGroupRoot};
        use monoxus::switch::Switch;

        rsx! {
            div {
                style: "width: 800px; height: 600px;",
                form {
                    id: "clobber-form",
                    style: "width: 600px; height: 320px; display: flex; flex-direction: column; gap: 8px;",
                    Checkbox {
                        id: "cb-unmutated",
                        name: "cb_unmutated",
                        value: "yes",
                        default_checked: CheckedState::Checked,
                        style: "width: 24px; height: 24px;",
                        "Unmutated"
                    }
                    Checkbox {
                        id: "cb-mutated",
                        name: "cb_mutated",
                        value: "yes",
                        default_checked: CheckedState::Checked,
                        style: "width: 24px; height: 24px;",
                        "Mutated"
                    }
                    CheckboxGroup {
                        id: "cb-grp",
                        name: "channels",
                        default_values: vec!["email".to_string()],
                        style: "display: flex; flex-direction: row; gap: 8px;",
                        Checkbox { id: "cb-grp-email", value: "email", style: "width: 24px; height: 24px;", "Email" }
                        Checkbox { id: "cb-grp-sms", value: "sms", style: "width: 24px; height: 24px;", "SMS" }
                    }
                    Switch {
                        id: "sw-unmutated",
                        name: "marketing",
                        value: "opt-in",
                        default_checked: true,
                        style: "width: 36px; height: 20px;",
                    }
                    RadioGroupRoot {
                        id: "rg-unmutated",
                        name: "plan",
                        default_value: "pro",
                        style: "display: flex; flex-direction: row; gap: 8px;",
                        RadioGroupItem { id: "rg-starter", value: "starter", style: "width: 60px; height: 28px;", "Starter" }
                        RadioGroupItem { id: "rg-pro", value: "pro", style: "width: 60px; height: 28px;", "Pro" }
                    }
                    Button {
                        id: "clobber-reset-btn".to_string(),
                        r#type: ButtonType::Reset,
                        style: "width: 120px; height: 32px;".to_string(),
                        "Reset"
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_unmutated_and_mutated_default_checked_controls_survive_dom_clobber_on_reset() {
        let mut harness = mount_native_scenario(ResetEpochClobberScenarioApp);
        advance_frame_and_pump(&mut harness, 0.016);

        let expected_initial = vec![
            ("cb_unmutated".to_string(), "yes".to_string()),
            ("cb_mutated".to_string(), "yes".to_string()),
            ("channels".to_string(), "email".to_string()),
            ("marketing".to_string(), "opt-in".to_string()),
            ("plan".to_string(), "pro".to_string()),
        ];
        assert_eq!(
            collect_form_data("clobber-form").expect("initial clobber-form data"),
            expected_initial
        );

        // Mutate only `#cb-mutated` (true -> false) and `#cb-grp-sms` (false -> true),
        // leaving `#cb-unmutated`, `#cb-grp-email`, `#sw-unmutated`, and `#rg-pro` untouched (true -> true).
        click_and_pump(&mut harness, "#cb-mutated");
        click_and_pump(&mut harness, "#cb-grp-sms");
        assert_eq!(
            collect_form_data("clobber-form").expect("mutated clobber-form data"),
            vec![
                ("cb_unmutated".to_string(), "yes".to_string()),
                ("channels".to_string(), "email".to_string()),
                ("channels".to_string(), "sms".to_string()),
                ("marketing".to_string(), "opt-in".to_string()),
                ("plan".to_string(), "pro".to_string()),
            ]
        );

        // Simulate browser native `HTMLFormElement.reset()` setting `input.checked = input.defaultChecked` (false)
        // on every checkbox/radio DOM node outside VDOM diffing right as reset fires.
        let input_ids = harness.query_all("input");
        {
            let mut base = harness.base_mut();
            for nid in input_ids {
                if let Some(el) = base.get_node_mut(nid).and_then(|n| n.element_data_mut()) {
                    if el.checkbox_input_checked().is_some() {
                        el.set_checkbox_input_checked(false);
                    }
                }
            }
        }
        assert!(
            collect_form_data("clobber-form")
                .expect("clobbered form data")
                .is_empty(),
            "Sanity check: simulated native form.reset() cleared DOM input.checked"
        );

        // Click `<button type="reset">` -> `reset_epoch` increments and remounts fresh `<input>` nodes
        // restoring ALL initial entries (both mutated AND unmutated `true -> true` controls).
        click_and_pump(&mut harness, "#clobber-reset-btn");
        assert_eq!(
            collect_form_data("clobber-form").expect("post-reset clobber-form data"),
            expected_initial,
            "All default-checked Checkbox, CheckboxGroup, Switch, and RadioGroup controls must re-sync on form reset"
        );
    }
}

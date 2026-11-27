use monoxus::{foundation::shared::ScopeHandle, select::Select};

#[test]
fn select_hidden_input_form_contract() {
    let scope = ScopeHandle::root("select-test").child("form");
    let select = Select::new(scope)
        .with_value(Some("pro".to_owned()))
        .with_name("tier")
        .with_form("checkout-form")
        .with_autocomplete("off")
        .with_required(true);

    assert_eq!(select.name(), Some("tier"));
    assert_eq!(select.form(), Some("checkout-form"));
    assert_eq!(select.autocomplete(), Some("off"));
    assert!(select.is_required());
    assert_eq!(select.trigger_attributes().aria_required(), Some("true"));
}

#[test]
fn select_disabled_cascades_to_trigger_and_items() {
    let scope = ScopeHandle::root("select-test").child("disabled");
    let select = Select::new(scope).with_disabled(true);

    let trigger_attrs = select.trigger_attributes();
    assert!(trigger_attrs.is_disabled());
    assert_eq!(trigger_attrs.aria_disabled(), Some("true"));
    assert_eq!(trigger_attrs.tabindex(), -1);

    let item_attrs = select.item_attributes("apple", false, false);
    assert!(item_attrs.is_disabled());
    assert_eq!(item_attrs.aria_disabled(), Some("true"));
}

#[test]
fn select_form_reset_listener_contract() {
    let scope = ScopeHandle::root("select-test").child("form-reset");
    let select = Select::new(scope)
        .with_default_value(Some("standard".to_string()))
        .with_value(Some("pro".to_string()))
        .with_name("plan");

    assert_eq!(select.default_value(), Some("standard"));
    assert_eq!(select.value(), Some("pro"));
    assert_eq!(select.name(), Some("plan"));
}

#[cfg(feature = "native")]
#[test]
fn fm_col_4_select_empty_multiple_hidden_input_emits_single_empty_placeholder() {
    use dioxus::prelude::*;
    use monoxus::select::{
        SelectContent, SelectItem, SelectMode, SelectPortal, SelectRoot, SelectTrigger,
        SelectViewport,
    };

    use crate::common::native_fixture::*;

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn EmptyMultipleSelectScenario() -> Element {
        rsx! {
            div { style: "padding: 24px;",
                SelectRoot {
                    id: "fm-col4-multi".to_string(),
                    mode: SelectMode::Multiple,
                    name: "fm_col4_tags".to_string(),
                    SelectTrigger {
                        class: "fm-col4-trig".to_string(),
                        style: "width: 160px; height: 32px;".to_string(),
                        "Tags"
                    }
                    SelectPortal {
                        SelectContent {
                            class: "fm-col4-content".to_string(),
                            style: "width: 160px; background: white;".to_string(),
                            SelectViewport {
                                SelectItem {
                                    value: "rust".to_string(),
                                    text: "Rust".to_string(),
                                    class: "fm-col4-opt-rust".to_string(),
                                    "Rust"
                                }
                                SelectItem {
                                    value: "wasm".to_string(),
                                    text: "Wasm".to_string(),
                                    class: "fm-col4-opt-wasm".to_string(),
                                    "Wasm"
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(EmptyMultipleSelectScenario);

    // Empty `multiple` Select must emit a single empty placeholder `<input type="hidden" name="fm_col4_tags" value="">` (`B12`, `FM-COL-4`).
    assert_eq!(
        harness
            .attr("input[type='hidden'][name='fm_col4_tags']", "value")
            .as_deref(),
        Some("")
    );

    // Selecting items replaces the empty placeholder with per-value hidden inputs.
    click_and_pump(&mut harness, ".fm-col4-trig");
    advance_frame_and_pump(&mut harness, 0.02);
    click_and_pump(&mut harness, ".fm-col4-opt-rust");
    click_and_pump(&mut harness, ".fm-col4-opt-wasm");
    advance_frame_and_pump(&mut harness, 0.02);
    assert_eq!(
        harness
            .attr("input[type='hidden'][name='fm_col4_tags'][value='rust']", "value")
            .as_deref(),
        Some("rust")
    );
    assert_eq!(
        harness
            .attr("input[type='hidden'][name='fm_col4_tags'][value='wasm']", "value")
            .as_deref(),
        Some("wasm")
    );
    assert!(
        harness
            .query("input[type='hidden'][name='fm_col4_tags'][value='']")
            .is_none()
    );
}

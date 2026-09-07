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
    assert_eq!(
        select.trigger_attributes().aria_required(),
        Some("true")
    );
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

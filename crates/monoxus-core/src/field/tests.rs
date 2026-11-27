use crate::foundation::compose::Slottable;

use super::{
    attrs::{
        FieldControlAttrs, FieldDescriptionAttrs, FieldErrorAttrs, FieldLabelAttrs,
        FieldLegendAttrs, FieldRootAttrs, FieldSetAttrs,
    },
    relationships::FieldRelationships,
    state::{
        FieldDefinition, FieldErrorRegistration, dedupe_error_messages, resolve_aria_describedby,
    },
    types::{
        FIELD_PARTS, FieldErrorLiveMode, FieldLegendVariant, FieldOrientation, FieldPart,
    },
};

#[test]
fn field_types_string_and_data_attr_contracts() {
    assert_eq!(FIELD_PARTS.len(), 7);
    assert_eq!(FieldPart::Root.as_str(), "root");
    assert_eq!(FieldPart::Root.data_attr(), "data-monoxus-field-root");
    assert_eq!(FieldPart::FieldSet.data_attr(), "data-monoxus-field-set");
    assert_eq!(FieldPart::Legend.data_attr(), "data-monoxus-field-legend");
    assert_eq!(FieldPart::Label.data_attr(), "data-monoxus-field-label");
    assert_eq!(FieldPart::Control.data_attr(), "data-monoxus-field-control");
    assert_eq!(
        FieldPart::Description.data_attr(),
        "data-monoxus-field-description"
    );
    assert_eq!(FieldPart::Error.data_attr(), "data-monoxus-field-error");

    assert_eq!(FieldOrientation::default(), FieldOrientation::Vertical);
    assert_eq!(FieldOrientation::Vertical.as_str(), "vertical");
    assert_eq!(FieldOrientation::Horizontal.as_str(), "horizontal");
    assert_eq!(FieldOrientation::Responsive.as_str(), "responsive");

    assert_eq!(FieldLegendVariant::default(), FieldLegendVariant::Legend);
    assert_eq!(FieldLegendVariant::Legend.as_str(), "legend");
    assert_eq!(FieldLegendVariant::Label.as_str(), "label");

    assert_eq!(FieldErrorLiveMode::default(), FieldErrorLiveMode::Off);
    assert_eq!(FieldErrorLiveMode::Assertive.as_aria_live(), Some("assertive"));
    assert_eq!(FieldErrorLiveMode::Polite.as_aria_live(), Some("polite"));
    assert_eq!(FieldErrorLiveMode::Off.as_aria_live(), None);
}

#[test]
fn field_relationships_deterministic_ids_and_empty_override_fallback() {
    let rels = FieldRelationships::new("email");
    assert_eq!(rels.root_id(), "5:field|5:email");
    assert_eq!(rels.fieldset_id(), "5:field|5:email|8:fieldset");
    assert_eq!(rels.legend_id(), "5:field|5:email|6:legend");
    assert_eq!(rels.control_id(), "5:field|5:email|7:control");
    assert_eq!(rels.control_id_indexed(0), rels.control_id());
    assert_eq!(rels.control_id_indexed(1), "5:field|5:email|9:control-1");
    assert_eq!(rels.control_id_indexed(2), "5:field|5:email|9:control-2");
    assert_eq!(rels.label_id(), "5:field|5:email|5:label");
    assert_eq!(rels.label_id_indexed(0), rels.label_id());
    assert_eq!(rels.label_id_indexed(1), "5:field|5:email|7:label-1");
    assert_eq!(rels.description_id(), "5:field|5:email|11:description");
    assert_eq!(rels.error_id(), "5:field|5:email|5:error");

    let overridden = rels
        .clone()
        .with_root_id("custom-root")
        .with_fieldset_id("")
        .with_legend_id("   ")
        .with_control_id("custom-control")
        .with_label_id("")
        .with_description_id("custom-desc")
        .with_error_id("custom-err");
    assert_eq!(overridden.root_id(), "custom-root");
    assert_eq!(overridden.fieldset_id(), rels.fieldset_id());
    assert_eq!(overridden.legend_id(), rels.legend_id());
    assert_eq!(overridden.control_id(), "custom-control");
    assert_eq!(overridden.control_id_indexed(0), "custom-control");
    assert_eq!(overridden.label_id(), rels.label_id());
    assert_eq!(overridden.description_id(), "custom-desc");
    assert_eq!(overridden.error_id(), "custom-err");
}

#[test]
fn resolve_aria_describedby_is_mounted_only_and_deduplicated() {
    assert_eq!(resolve_aria_describedby(&[], &[], false, None), None);
    assert_eq!(resolve_aria_describedby(&[], &[], true, Some("   ")), None);

    let descs = vec![String::from("desc-1"), String::from("desc-1")];
    let inactive_err = vec![FieldErrorRegistration::new("err-1", false)];
    assert_eq!(
        resolve_aria_describedby(&descs, &inactive_err, false, None).as_deref(),
        Some("desc-1")
    );
    // field_invalid=true activates mounted error ID
    assert_eq!(
        resolve_aria_describedby(&descs, &inactive_err, true, None).as_deref(),
        Some("desc-1 err-1")
    );

    let active_err = vec![FieldErrorRegistration::new("err-1", true)];
    assert_eq!(
        resolve_aria_describedby(&descs, &active_err, false, Some("err-1 ext-1 ext-2")).as_deref(),
        Some("desc-1 err-1 ext-1 ext-2")
    );
}

#[test]
fn dedupe_error_messages_preserves_order_and_drops_blanks() {
    let input = vec![
        String::from("  "),
        String::from("Required"),
        String::from("Too short"),
        String::from("Required"),
        String::from("  Too short  "),
    ];
    assert_eq!(
        dedupe_error_messages(&input),
        vec![String::from("Required"), String::from("Too short")]
    );
}

#[test]
fn field_attrs_truth_table_and_as_child_projection() {
    let rels = FieldRelationships::new("profile");
    let valid_def = FieldDefinition::new(rels.clone())
        .with_name("email")
        .with_orientation(FieldOrientation::Horizontal);

    let root_valid = FieldRootAttrs::from_definition(&valid_def, Some("group"), None, None);
    assert_eq!(root_valid.role, Some("group"));
    assert_eq!(root_valid.data_orientation, "horizontal");
    assert_eq!(root_valid.data_invalid, None);
    assert_eq!(root_valid.data_disabled, None);
    assert_eq!(root_valid.data_required, None);
    assert_eq!(root_valid.data_readonly, None);
    assert_eq!(root_valid.data_monoxus_field_root, "");

    let invalid_def = valid_def
        .clone()
        .with_invalid(true)
        .with_required(true)
        .with_read_only(true);
    let ctrl = FieldControlAttrs::from_definition(
        &invalid_def,
        None,
        Some(rels.label_id().to_owned()),
        Some(String::from("desc-1 err-1")),
    );
    assert_eq!(ctrl.id, rels.control_id());
    assert_eq!(ctrl.name.as_deref(), Some("email"));
    assert_eq!(ctrl.aria_invalid, Some("true"));
    assert_eq!(ctrl.aria_required, Some("true"));
    assert_eq!(ctrl.aria_disabled, None);
    assert_eq!(ctrl.aria_readonly, Some("true"));
    assert_eq!(ctrl.data_invalid, Some(""));
    assert_eq!(ctrl.data_disabled, None);
    assert_eq!(ctrl.data_required, Some(""));
    assert_eq!(ctrl.data_readonly, Some(""));
    assert_eq!(ctrl.data_monoxus_field_control, "");

    // Disabled suppresses data-readonly & aria-readonly
    let disabled_ro_def = invalid_def.clone().with_disabled(true);
    let ctrl_dis = FieldControlAttrs::from_definition(&disabled_ro_def, None, None, None);
    assert!(ctrl_dis.disabled);
    assert_eq!(ctrl_dis.aria_disabled, Some("true"));
    assert_eq!(ctrl_dis.data_disabled, Some(""));
    assert_eq!(ctrl_dis.aria_readonly, None);
    assert_eq!(ctrl_dis.data_readonly, None);

    // FieldSet, Legend, Label, Description, Error attrs + as_child projection
    let fs = FieldSetAttrs::from_definition(&invalid_def.clone().with_fieldset(true), None);
    assert_eq!(fs.aria_invalid, Some("true"));
    assert_eq!(fs.data_invalid, Some(""));
    assert_eq!(fs.data_monoxus_field_set, "");

    let legend = FieldLegendAttrs::from_definition(&invalid_def, FieldLegendVariant::Label);
    assert_eq!(legend.data_variant, "label");
    assert_eq!(legend.data_invalid, Some(""));

    let label = FieldLabelAttrs::from_definition(&invalid_def, Some(rels.control_id().to_owned()));
    assert_eq!(label.r#for.as_deref(), Some(rels.control_id()));
    assert_eq!(label.data_invalid, Some(""));

    let desc = FieldDescriptionAttrs::from_definition(&invalid_def);
    assert_eq!(desc.data_invalid, Some(""));
    assert_eq!(desc.data_monoxus_field_description, "");

    let active_err = FieldErrorAttrs::from_definition(&invalid_def, true, FieldErrorLiveMode::Off);
    assert_eq!(active_err.role, Some("alert"));
    assert_eq!(active_err.aria_live, None);
    assert_eq!(active_err.data_invalid, Some(""));

    let polite_err =
        FieldErrorAttrs::from_definition(&invalid_def, true, FieldErrorLiveMode::Polite);
    assert_eq!(polite_err.role, Some("alert"));
    assert_eq!(polite_err.aria_live, Some("polite"));

    let inactive_err = FieldErrorAttrs::from_definition(
        &valid_def,
        false,
        FieldErrorLiveMode::Assertive,
    );
    assert_eq!(inactive_err.role, None);
    assert_eq!(inactive_err.aria_live, None);
    assert_eq!(inactive_err.data_invalid, None);

    let (target, projected, content) =
        root_valid.project_as_child("section", Slottable::new("field-body"));
    assert_eq!(target, "section");
    assert_eq!(projected, root_valid);
    assert_eq!(content, "field-body");
}

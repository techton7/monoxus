use dioxus::prelude::*;
use monoxus::{
    calendar::CalendarSnapshot,
    checkbox::{CheckboxIndicator, CheckboxRoot, CheckedState, use_checkbox_context},
    combobox::{
        ComboboxContent, ComboboxInput, ComboboxItem, ComboboxItemData, ComboboxMode,
        ComboboxPortal, ComboboxRoot, ComboboxTrigger, ComboboxViewport,
    },
    date_field::DateFieldSegmentItem,
    date_picker::{
        DatePickerCalendar, DatePickerCell, DatePickerContent, DatePickerDay, DatePickerGrid,
        DatePickerGridBody, DatePickerGridHead, DatePickerGridRow, DatePickerHeadCell,
        DatePickerHeader, DatePickerHeading, DatePickerInput, DatePickerNextButton,
        DatePickerPortal, DatePickerPrevButton, DatePickerRoot, DatePickerSegment,
        DatePickerTrigger,
    },
    field::{
        FieldControl, FieldDescription, FieldError, FieldLabel, FieldLegend, FieldOrientation,
        FieldRoot, FieldSet, use_field_control,
    },
    foundation::date_time::{CalendarDate, CalendarLocale, DateValue},
    pin_input::{
        PinInputCell, PinInputMode, PinInputPattern, PinInputRoot, PinInputSnapshot,
    },
    select::{
        SelectContent, SelectItem, SelectItemData, SelectItemIndicator, SelectItemText,
        SelectPortal, SelectRoot, SelectTrigger, SelectValue, SelectViewport,
    },
    slider::{SliderRange, SliderRoot, SliderThumb, SliderTrack, SliderValue},
};

const FIELD_PLAYGROUND_CSS: &str = r#"
[data-monoxus-field-root] {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
}
[data-monoxus-field-root][data-orientation="horizontal"] {
    flex-direction: row;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem;
}
[data-monoxus-field-label] {
    font-size: 0.8125rem;
    font-weight: 600;
    color: #1e293b;
    cursor: pointer;
    user-select: none;
}
[data-monoxus-field-label][data-invalid] {
    color: #dc2626;
}
[data-monoxus-field-label][data-disabled] {
    color: #94a3b8;
    cursor: not-allowed;
}
[data-monoxus-field-description] {
    margin: 0;
    font-size: 0.75rem;
    color: #64748b;
}
[data-monoxus-field-error] {
    margin: 0;
    font-size: 0.75rem;
    font-weight: 600;
    color: #dc2626;
}
.fld-native-input {
    width: 100%;
    max-width: 320px;
    padding: 0.5rem 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #0f172a;
    font-size: 0.875rem;
    outline: none;
    box-sizing: border-box;
}
.fld-native-input:focus {
    border-color: #7c3aed;
    box-shadow: 0 0 0 2px rgba(124, 58, 237, 0.16);
}
.fld-native-input[data-invalid] {
    border-color: #dc2626;
    background-color: #fef2f2;
}
.fld-native-input[data-disabled] {
    border-color: #e2e8f0;
    background-color: #f1f5f9;
    color: #94a3b8;
    cursor: not-allowed;
}
.fld-select-trigger {
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    min-width: 240px;
    height: 38px;
    padding: 0 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #0f172a;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    outline: none;
}
.fld-select-trigger:focus-visible {
    border-color: #7c3aed;
    box-shadow: 0 0 0 2px rgba(124, 58, 237, 0.2);
}
.fld-select-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.425rem 0.625rem;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    color: #1e293b;
    cursor: pointer;
    user-select: none;
}
.fld-select-item[data-highlighted="true"] {
    background-color: #ede9fe;
    color: #5b21b6;
}
.fld-select-item[data-selected="true"] {
    font-weight: 700;
}
.fld-cbx-item[data-combobox-item] {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.425rem 0.625rem;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: #1e293b;
    cursor: pointer;
    user-select: none;
}
.fld-cbx-item[data-combobox-item][data-highlighted] {
    background-color: #ede9fe;
    color: #5b21b6;
}
.fld-cbx-item[data-combobox-item][data-selected] {
    font-weight: 700;
}
.fld-dp-input[data-date-field-input] {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    min-height: 38px;
    padding: 0 8px 0 12px;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #0f172a;
    font-size: 0.875rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}
.fld-dp-input[data-date-field-input][data-invalid] {
    border-color: #dc2626;
    background-color: #fef2f2;
}
.fld-dp-seg[data-date-field-segment] {
    padding: 2px 3px;
    border-radius: 0.25rem;
    outline: none;
    user-select: none;
}
.fld-dp-seg[data-date-field-segment]:focus {
    background-color: #0f172a;
    color: #ffffff;
}
.fld-dp-trigger[data-segment="trigger"] {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 28px;
    padding: 0 8px;
    margin-left: 6px;
    border-radius: 0.375rem;
    border: 1px solid #cbd5e1;
    background-color: #f8fafc;
    color: #334155;
    font-size: 0.75rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
}
.fld-dp-day[data-calendar-day] {
    width: 34px;
    height: 34px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: #0f172a;
    background-color: transparent;
    cursor: pointer;
    user-select: none;
    border: 1px solid transparent;
}
.fld-dp-day[data-calendar-day]:hover:not([data-disabled]):not([data-selected]) {
    background-color: #f1f5f9;
}
.fld-dp-day[data-calendar-day][data-selected] {
    background-color: #7c3aed;
    color: #ffffff;
    font-weight: 700;
}
.fld-dp-day[data-calendar-day][data-outside-month] {
    color: #94a3b8;
}
.fld-pin-cell[data-monoxus-pin-input-cell] {
    width: 42px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.5rem;
    font-size: 1.125rem;
    font-weight: 700;
    font-family: monospace;
    color: #0f172a;
    box-sizing: border-box;
    border: 1px solid #cbd5e1;
    background-color: #f8fafc;
}
.fld-pin-cell[data-monoxus-pin-input-cell][data-active] {
    border: 2px solid #7c3aed;
    background-color: #ffffff;
}
"#;

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06); display: flex; flex-direction: column; gap: 1rem;";
const STAGE_STYLE: &str = "display: flex; flex-direction: column; gap: 0.875rem; padding: 1.125rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;";
const TOGGLE_ROW_STYLE: &str = "display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;";
const READOUT_ROW_STYLE: &str = "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem; font-size: 0.75rem; font-family: monospace; color: #334155;";

fn toggle_btn_style(active: bool) -> &'static str {
    if active {
        "display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.375rem 0.75rem; border-radius: 0.5rem; border: 1px solid #6d28d9; background-color: #7c3aed; color: #ffffff; font-size: 0.75rem; font-weight: 600; cursor: pointer;"
    } else {
        "display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.375rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;"
    }
}

fn pill_style(highlight: bool) -> &'static str {
    if highlight {
        "display: inline-flex; align-items: center; border-radius: 0.375rem; padding: 0.125rem 0.5rem; background-color: #ede9fe; color: #6d28d9; border: 1px solid #c4b5fd; font-size: 0.75rem; font-family: monospace; font-weight: 700;"
    } else {
        "display: inline-flex; align-items: center; border-radius: 0.375rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; border: 1px solid #e2e8f0; font-size: 0.75rem; font-family: monospace; font-weight: 600;"
    }
}

#[component]
fn FieldCheckboxGlyph() -> Element {
    let ctx = use_checkbox_context();
    match ctx.checked_state() {
        CheckedState::Checked => rsx! { span { "✓" } },
        CheckedState::Indeterminate => rsx! { span { "−" } },
        CheckedState::Unchecked => rsx! {},
    }
}

#[component]
pub fn FieldPlayground() -> Element {
    rsx! {
        div {
            id: "field-playground",
            "data-gui-test-id": "field-playground",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 1.75rem;",
            style { "{FIELD_PLAYGROUND_CSS}" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                    "Field"
                }
                p {
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Headless form-field coordination (FieldRoot, FieldSet, FieldLegend, FieldLabel, FieldControl, FieldDescription, FieldError) with dynamic mounted-only aria-describedby, 1:N fieldset scoping, controlled-value contracts, and composite on_focus_out."
                }
            }

            BasicFieldSection {}
            FieldSetCheckboxSection {}
            SelectControlledFieldSection {}
            PatternBCompositeSection {}
            PinAndSliderFormResetSection {}
        }
    }
}

// ---------------------------------------------------------------------------
// Section 1: Basic FieldRoot + Dynamic FieldDescription / FieldError
// ---------------------------------------------------------------------------
#[component]
fn BasicFieldNativeInput(
    value: Signal<String>,
    blur_count: Signal<usize>,
) -> Element {
    let control = use_field_control();
    let global_attrs = control
        .as_ref()
        .map(|h| h.to_global_attributes())
        .unwrap_or_default();
    let id_readout = control
        .as_ref()
        .map(|h| h.id().to_string())
        .unwrap_or_else(|| "none".to_string());
    let desc_readout = control
        .as_ref()
        .and_then(|h| h.aria_describedby().map(str::to_string))
        .unwrap_or_else(|| "none".to_string());
    let invalid_readout = control
        .as_ref()
        .and_then(|h| h.attrs().aria_invalid)
        .unwrap_or("none");

    let mut value = value;
    let mut blur_count = blur_count;

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 0.5rem;",
            input {
                r#type: "text",
                class: "fld-native-input",
                "data-gui-test-id": "field-s1-input",
                placeholder: "e.g. monoxus_dev",
                value: "{value()}",
                oninput: move |evt| value.set(evt.value()),
                onblur: move |_| blur_count += 1,
                ..global_attrs,
            }
            div {
                style: "{READOUT_ROW_STYLE}",
                span {
                    "id: "
                    strong {
                        id: "field-s1-readout-id",
                        "data-gui-test-id": "field-s1-readout-id",
                        style: "{pill_style(false)}",
                        "{id_readout}"
                    }
                }
                span {
                    "aria-describedby: "
                    strong {
                        id: "field-s1-readout-describedby",
                        "data-gui-test-id": "field-s1-readout-describedby",
                        style: "{pill_style(desc_readout != \"none\")}",
                        "{desc_readout}"
                    }
                }
                span {
                    "aria-invalid: "
                    strong {
                        id: "field-s1-readout-invalid",
                        "data-gui-test-id": "field-s1-readout-invalid",
                        style: "{pill_style(invalid_readout == \"true\")}",
                        "{invalid_readout}"
                    }
                }
                span {
                    "blurs: "
                    strong {
                        id: "field-s1-blur-count",
                        "data-gui-test-id": "field-s1-blur-count",
                        style: "{pill_style(blur_count() > 0)}",
                        "{blur_count()}"
                    }
                }
            }
        }
    }
}

#[component]
fn BasicFieldSection() -> Element {
    let value = use_signal(|| "alice".to_string());
    let blur_count = use_signal(|| 0usize);
    let mut invalid = use_signal(|| false);
    let mut disabled = use_signal(|| false);
    let mut required = use_signal(|| true);
    let mut show_description = use_signal(|| true);
    let mut orientation = use_signal(|| FieldOrientation::Vertical);

    let orient_label = match orientation() {
        FieldOrientation::Vertical => "orientation: vertical",
        FieldOrientation::Horizontal => "orientation: horizontal",
        FieldOrientation::Responsive => "orientation: responsive",
    };

    rsx! {
        section {
            id: "field-section-1",
            "data-gui-test-id": "field-section-1",
            style: "{CARD_STYLE}",

            div {
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 700; color: #0f172a;",
                    "1. Basic FieldRoot + Dynamic Description / Error & State Attributes (FM-FLD-01, FM-FLD-03)"
                }
                p {
                    style: "margin: 0.25rem 0 0; font-size: 0.8125rem; color: #64748b;",
                    "Native <input> wired via use_field_control().to_global_attributes(). Toggle description, invalid, disabled, required, and orientation to observe live aria-describedby and data-* synchronization."
                }
            }

            div {
                style: "{TOGGLE_ROW_STYLE}",
                button {
                    id: "field-s1-toggle-invalid",
                    "data-gui-test-id": "field-s1-toggle-invalid",
                    r#type: "button",
                    style: "{toggle_btn_style(invalid())}",
                    onclick: move |_| invalid.set(!invalid()),
                    if invalid() { "invalid: true" } else { "invalid: false" }
                }
                button {
                    id: "field-s1-toggle-disabled",
                    "data-gui-test-id": "field-s1-toggle-disabled",
                    r#type: "button",
                    style: "{toggle_btn_style(disabled())}",
                    onclick: move |_| disabled.set(!disabled()),
                    if disabled() { "disabled: true" } else { "disabled: false" }
                }
                button {
                    id: "field-s1-toggle-required",
                    "data-gui-test-id": "field-s1-toggle-required",
                    r#type: "button",
                    style: "{toggle_btn_style(required())}",
                    onclick: move |_| required.set(!required()),
                    if required() { "required: true" } else { "required: false" }
                }
                button {
                    id: "field-s1-toggle-description",
                    "data-gui-test-id": "field-s1-toggle-description",
                    r#type: "button",
                    style: "{toggle_btn_style(show_description())}",
                    onclick: move |_| show_description.set(!show_description()),
                    if show_description() { "show_description: true" } else { "show_description: false" }
                }
                button {
                    id: "field-s1-toggle-orientation",
                    "data-gui-test-id": "field-s1-toggle-orientation",
                    r#type: "button",
                    style: "{toggle_btn_style(orientation() == FieldOrientation::Horizontal)}",
                    onclick: move |_| {
                        let next = match orientation() {
                            FieldOrientation::Vertical => FieldOrientation::Horizontal,
                            FieldOrientation::Horizontal | FieldOrientation::Responsive => {
                                FieldOrientation::Vertical
                            }
                        };
                        orientation.set(next);
                    },
                    "{orient_label}"
                }
                button {
                    id: "field-s1-outside-focus",
                    "data-gui-test-id": "field-s1-outside-focus",
                    r#type: "button",
                    style: "{toggle_btn_style(false)}",
                    "Focus Sink"
                }
            }

            div {
                style: "{STAGE_STYLE}",
                FieldRoot {
                    id: Some("field-s1-root".to_string()),
                    "data-gui-test-id": "field-s1-root",
                    name: Some("username".to_string()),
                    invalid: invalid(),
                    disabled: disabled(),
                    required: required(),
                    orientation: orientation(),

                    FieldLabel {
                        id: Some("field-s1-label".to_string()),
                        "data-gui-test-id": "field-s1-label",
                        "Username"
                    }

                    FieldControl {
                        BasicFieldNativeInput {
                            value,
                            blur_count,
                        }
                    }

                    if show_description() {
                        FieldDescription {
                            id: Some("field-s1-desc".to_string()),
                            "data-gui-test-id": "field-s1-desc",
                            "Public handle shown on commits and reviews."
                        }
                    }

                    FieldError {
                        id: Some("field-s1-error".to_string()),
                        "data-gui-test-id": "field-s1-error",
                        errors: if invalid() {
                            vec!["Username is invalid or already taken.".to_string()]
                        } else {
                            vec![]
                        },
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Section 2: 1:N FieldSet + FieldLegend + FieldControl + CheckboxRoot
// ---------------------------------------------------------------------------
#[component]
fn FieldSetCheckboxSection() -> Element {
    let mut alerts_checked = use_signal(|| CheckedState::Checked);
    let mut digest_checked = use_signal(|| CheckedState::Unchecked);
    let mut alerts_toggles = use_signal(|| 0usize);
    let mut digest_toggles = use_signal(|| 0usize);

    let none_selected = !alerts_checked().is_checked() && !digest_checked().is_checked();

    rsx! {
        section {
            id: "field-section-2",
            "data-gui-test-id": "field-section-2",
            style: "{CARD_STYLE}",

            div {
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 700; color: #0f172a;",
                    "2. 1:N FieldSet + FieldLegend + FieldControl + CheckboxRoot (FM-FLD-02)"
                }
                p {
                    style: "margin: 0.25rem 0 0; font-size: 0.8125rem; color: #64748b;",
                    "Each FieldControl scopes its own control/label pairing inside a shared FieldSet. Clicking a FieldLabel toggles only its paired checkbox; unchecking both activates the shared FieldError."
                }
            }

            div {
                style: "{STAGE_STYLE}",
                FieldSet {
                    id: Some("field-s2-fieldset".to_string()),
                    "data-gui-test-id": "field-s2-fieldset",
                    invalid: none_selected,
                    style: Some("border: 1px solid #cbd5e1; border-radius: 0.625rem; padding: 1rem; margin: 0; display: flex; flex-direction: column; gap: 0.625rem; background-color: #ffffff;".to_string()),

                    FieldLegend {
                        id: Some("field-s2-legend".to_string()),
                        "data-gui-test-id": "field-s2-legend",
                        style: Some("padding: 0 0.375rem; font-size: 0.875rem; font-weight: 700; color: #0f172a;".to_string()),
                        "Notification Channels"
                    }

                    FieldDescription {
                        id: Some("field-s2-desc".to_string()),
                        "data-gui-test-id": "field-s2-desc",
                        "Choose at least one channel to receive status updates."
                    }

                    div {
                        style: "display: flex; flex-wrap: wrap; gap: 1.5rem; padding: 0.25rem 0;",

                        FieldControl {
                            id: Some("field-s2-cb-alerts".to_string()),
                            div {
                                style: "display: inline-flex; align-items: center; gap: 0.5rem;",
                                CheckboxRoot {
                                    id: Some("field-s2-cb-alerts".to_string()),
                                    "data-gui-test-id": "field-s2-cb-alerts",
                                    checked: Some(alerts_checked()),
                                    on_checked_change: move |next| {
                                        alerts_checked.set(next);
                                        alerts_toggles += 1;
                                    },
                                    CheckboxIndicator {
                                        id: Some("field-s2-cb-alerts-ind".to_string()),
                                        FieldCheckboxGlyph {}
                                    }
                                }
                                FieldLabel {
                                    id: Some("field-s2-lbl-alerts".to_string()),
                                    "data-gui-test-id": "field-s2-lbl-alerts",
                                    "Alerts"
                                }
                            }
                        }

                        FieldControl {
                            id: Some("field-s2-cb-digest".to_string()),
                            div {
                                style: "display: inline-flex; align-items: center; gap: 0.5rem;",
                                CheckboxRoot {
                                    id: Some("field-s2-cb-digest".to_string()),
                                    "data-gui-test-id": "field-s2-cb-digest",
                                    checked: Some(digest_checked()),
                                    on_checked_change: move |next| {
                                        digest_checked.set(next);
                                        digest_toggles += 1;
                                    },
                                    CheckboxIndicator {
                                        id: Some("field-s2-cb-digest-ind".to_string()),
                                        FieldCheckboxGlyph {}
                                    }
                                }
                                FieldLabel {
                                    id: Some("field-s2-lbl-digest".to_string()),
                                    "data-gui-test-id": "field-s2-lbl-digest",
                                    "Digest"
                                }
                            }
                        }
                    }

                    FieldError {
                        id: Some("field-s2-error".to_string()),
                        "data-gui-test-id": "field-s2-error",
                        errors: if none_selected {
                            vec!["Select at least one notification channel.".to_string()]
                        } else {
                            vec![]
                        },
                    }
                }

                div {
                    style: "{READOUT_ROW_STYLE}",
                    span {
                        "Alerts: "
                        strong {
                            id: "field-s2-alerts-state",
                            "data-gui-test-id": "field-s2-alerts-state",
                            style: "{pill_style(alerts_checked().is_checked())}",
                            "{alerts_checked().data_state_str()} (toggles: {alerts_toggles()})"
                        }
                    }
                    span {
                        "Digest: "
                        strong {
                            id: "field-s2-digest-state",
                            "data-gui-test-id": "field-s2-digest-state",
                            style: "{pill_style(digest_checked().is_checked())}",
                            "{digest_checked().data_state_str()} (toggles: {digest_toggles()})"
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Section 3: Select Controlled Mode (Pattern C) + Custom SelectTrigger.id + on_focus_out
// ---------------------------------------------------------------------------
#[component]
fn SelectControlledFieldSection() -> Element {
    let options = [
        ("us-east", "US East (Virginia)"),
        ("eu-central", "EU Central (Frankfurt)"),
        ("ap-northeast", "AP Northeast (Seoul)"),
    ];
    let items: Vec<SelectItemData> = options
        .iter()
        .map(|(v, l)| SelectItemData::new(*v, *l, false))
        .collect();

    let mut selected = use_signal(|| Some("us-east".to_string()));
    let mut reject_changes = use_signal(|| false);
    let mut attempted_count = use_signal(|| 0usize);
    let mut focus_out_count = use_signal(|| 0usize);

    let current_val = selected().unwrap_or_else(|| "none".to_string());

    rsx! {
        section {
            id: "field-section-3",
            "data-gui-test-id": "field-section-3",
            style: "{CARD_STYLE}",

            div {
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 700; color: #0f172a;",
                    "3. Select Controlled Mode (Pattern C) + Custom SelectTrigger.id + on_focus_out (FM-CTL-01, FM-CTL-04)"
                }
                p {
                    style: "margin: 0.25rem 0 0; font-size: 0.8125rem; color: #64748b;",
                    "Clicking FieldLabel focuses #field-custom-select. Locking controlled mode rejects new selections with zero trigger/item desync, while on_focus_out tracks leaving the select."
                }
            }

            div {
                style: "{TOGGLE_ROW_STYLE}",
                button {
                    id: "field-s3-reject-toggle",
                    "data-gui-test-id": "field-s3-reject-toggle",
                    r#type: "button",
                    style: "{toggle_btn_style(reject_changes())}",
                    onclick: move |_| reject_changes.set(!reject_changes()),
                    if reject_changes() {
                        "Reject changes (controlled lock): ON"
                    } else {
                        "Reject changes (controlled lock): OFF"
                    }
                }
                button {
                    id: "field-s3-reset",
                    "data-gui-test-id": "field-s3-reset",
                    r#type: "button",
                    style: "{toggle_btn_style(false)}",
                    onclick: move |_| {
                        selected.set(Some("us-east".to_string()));
                        reject_changes.set(false);
                        attempted_count.set(0);
                        focus_out_count.set(0);
                    },
                    "Reset"
                }
            }

            div {
                style: "{STAGE_STYLE}",
                FieldRoot {
                    id: Some("field-s3-root".to_string()),
                    "data-gui-test-id": "field-s3-root",
                    name: Some("region".to_string()),

                    FieldLabel {
                        id: Some("field-s3-label".to_string()),
                        "data-gui-test-id": "field-s3-label",
                        "Deployment Region"
                    }

                    SelectRoot {
                        id: Some("field-s3-select".to_string()),
                        items: Some(items),
                        value: selected(),
                        on_value_change: move |next: Option<String>| {
                            attempted_count += 1;
                            if !reject_changes() {
                                selected.set(next);
                            }
                        },
                        on_focus_out: move |_| {
                            focus_out_count += 1;
                        },

                        SelectTrigger {
                            id: Some("field-custom-select".to_string()),
                            "data-gui-test-id": "field-custom-select",
                            class: Some("fld-select-trigger".to_string()),
                            SelectValue {
                                placeholder: Some("Choose a region...".to_string()),
                            }
                            span { style: "font-size: 0.75rem; color: #64748b;", "▾" }
                        }

                        SelectPortal {
                            SelectContent {
                                id: Some("field-s3-content".to_string()),
                                "data-gui-test-id": "field-s3-content",
                                style: Some("z-index: 60; min-width: 240px; padding: 0.25rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; box-shadow: 0 12px 24px rgba(15, 23, 42, 0.14); outline: none;".to_string()),
                                SelectViewport {
                                    for (val, label) in options {
                                        {
                                            let is_selected = selected().as_deref() == Some(val);
                                            rsx! {
                                                SelectItem {
                                                    key: "{val}",
                                                    id: Some(format!("field-s3-item-{val}")),
                                                    "data-gui-test-id": format!("field-s3-item-{val}"),
                                                    value: val.to_string(),
                                                    text: Some(label.to_string()),
                                                    class: Some("fld-select-item".to_string()),
                                                    SelectItemText { "{label}" }
                                                    SelectItemIndicator {
                                                        if is_selected {
                                                            span { style: "color: #7c3aed; font-weight: 700;", "✓" }
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

                    FieldDescription {
                        id: Some("field-s3-desc".to_string()),
                        "data-gui-test-id": "field-s3-desc",
                        "Custom SelectTrigger id (#field-custom-select) is synced to FieldLabel r#for."
                    }
                }

                div {
                    style: "{READOUT_ROW_STYLE}",
                    span {
                        "controlled value: "
                        strong {
                            id: "field-s3-value",
                            "data-gui-test-id": "field-s3-value",
                            style: "{pill_style(true)}",
                            "{current_val}"
                        }
                    }
                    span {
                        "change attempts: "
                        strong {
                            id: "field-s3-attempts",
                            "data-gui-test-id": "field-s3-attempts",
                            style: "{pill_style(attempted_count() > 0)}",
                            "{attempted_count()}"
                        }
                    }
                    span {
                        "on_focus_out: "
                        strong {
                            id: "field-s3-focus-out-count",
                            "data-gui-test-id": "field-s3-focus-out-count",
                            style: "{pill_style(focus_out_count() > 0)}",
                            "{focus_out_count()}"
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Section 4: Pattern B Unidirectional controlled_value (Combobox & DatePicker)
//            + is_invalid_override + Composite on_focus_out
// ---------------------------------------------------------------------------
fn sample_date(year: i32, month: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(year, month, day).expect("valid field playground date"))
}

#[component]
fn PatternBCompositeSection() -> Element {
    // 4A: Combobox Pattern B
    let cbx_options = [
        ("rust", "Rust"),
        ("typescript", "TypeScript"),
        ("zig", "Zig"),
        ("gleam", "Gleam"),
    ];
    let cbx_items: Vec<ComboboxItemData> = cbx_options
        .iter()
        .map(|(v, l)| ComboboxItemData {
            value: (*v).to_string(),
            label: (*l).to_string(),
            disabled: false,
        })
        .collect();

    let mut cbx_selected = use_signal(|| Some("rust".to_string()));
    let mut cbx_reject = use_signal(|| false);
    let mut cbx_attempts = use_signal(|| 0usize);
    let mut cbx_focus_out = use_signal(|| 0usize);

    // 4B: DatePicker Pattern B
    let mut dp_value = use_signal(|| Some(sample_date(2025, 4, 15)));
    let mut dp_invalid = use_signal(|| false);
    let mut dp_reject = use_signal(|| false);
    let mut dp_focus_out = use_signal(|| 0usize);

    let cbx_val_text = cbx_selected().unwrap_or_else(|| "none".to_string());
    let dp_val_text = dp_value().map_or_else(|| "none".to_string(), |d| d.to_iso_string());

    rsx! {
        section {
            id: "field-section-4",
            "data-gui-test-id": "field-section-4",
            style: "{CARD_STYLE}",

            div {
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 700; color: #0f172a;",
                    "4. Pattern B controlled_value (Combobox & DatePicker) + is_invalid_override + Composite on_focus_out (FM-CTL-02, FM-CTL-03, FM-CTL-04)"
                }
                p {
                    style: "margin: 0.25rem 0 0; font-size: 0.8125rem; color: #64748b;",
                    "Unidirectional ReadSignal<T> control without mutating parent signals directly. Composite focus transitions (Input -> Trigger -> Item, Segment -> Segment -> Calendar) keep on_focus_out at 0 until focus truly leaves."
                }
            }

            // 4A: Combobox
            div {
                style: "{STAGE_STYLE}",
                div {
                    style: "{TOGGLE_ROW_STYLE}",
                    button {
                        id: "field-s4-cbx-reject-toggle",
                        "data-gui-test-id": "field-s4-cbx-reject-toggle",
                        r#type: "button",
                        style: "{toggle_btn_style(cbx_reject())}",
                        onclick: move |_| cbx_reject.set(!cbx_reject()),
                        if cbx_reject() { "Combobox Reject updates: ON" } else { "Combobox Reject updates: OFF" }
                    }
                    button {
                        id: "field-s4-outside-focus",
                        "data-gui-test-id": "field-s4-outside-focus",
                        r#type: "button",
                        style: "{toggle_btn_style(false)}",
                        "Outside Focus Target"
                    }
                }

                FieldRoot {
                    id: Some("field-s4-cbx-root".to_string()),
                    "data-gui-test-id": "field-s4-cbx-root",
                    name: Some("language".to_string()),

                    FieldLabel {
                        id: Some("field-s4-cbx-label".to_string()),
                        "data-gui-test-id": "field-s4-cbx-label",
                        "Primary Language (Combobox Pattern B)"
                    }

                    ComboboxRoot {
                        id: Some("field-s4-cbx".to_string()),
                        mode: ComboboxMode::Single,
                        controlled_value: Some(ReadSignal::from(cbx_selected)),
                        default_input_value: Some("Rust".to_string()),
                        items: cbx_items,
                        on_value_change: move |next: Option<String>| {
                            cbx_attempts += 1;
                            if !cbx_reject() {
                                cbx_selected.set(next);
                            }
                        },
                        on_focus_out: move |_| {
                            cbx_focus_out += 1;
                        },

                        div {
                            style: "display: inline-flex; align-items: stretch; width: 100%; max-width: 280px; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; overflow: hidden;",
                            ComboboxInput {
                                id: Some("field-s4-cbx-input".to_string()),
                                "data-gui-test-id": "field-s4-cbx-input",
                                placeholder: Some("Select a language...".to_string()),
                                style: Some("flex: 1; min-width: 0; padding: 0.5rem 0.75rem; border: none; font-size: 0.875rem; color: #0f172a; background-color: transparent; outline: none;".to_string()),
                            }
                            ComboboxTrigger {
                                id: Some("field-s4-cbx-trigger".to_string()),
                                "data-gui-test-id": "field-s4-cbx-trigger",
                                style: Some("display: inline-flex; align-items: center; justify-content: center; padding: 0 0.75rem; border: none; border-left: 1px solid #e2e8f0; background-color: #f8fafc; color: #334155; font-size: 0.75rem; font-weight: 700; cursor: pointer;".to_string()),
                                "▾"
                            }
                        }

                        ComboboxPortal {
                            ComboboxContent {
                                id: Some("field-s4-cbx-content".to_string()),
                                "data-gui-test-id": "field-s4-cbx-content",
                                same_width: true,
                                side_offset: 6.0,
                                style: Some("min-width: 240px; padding: 0.375rem; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; box-shadow: 0 12px 24px rgba(15, 23, 42, 0.14); z-index: 60;".to_string()),
                                ComboboxViewport {
                                    for (val, label) in cbx_options {
                                        ComboboxItem {
                                            key: "{val}",
                                            id: Some(format!("field-s4-cbx-item-{val}")),
                                            "data-gui-test-id": format!("field-s4-cbx-item-{val}"),
                                            class: Some("fld-cbx-item".to_string()),
                                            value: val.to_string(),
                                            label: Some(label.to_string()),
                                            span { "{label}" }
                                            if cbx_selected().as_deref() == Some(val) {
                                                span { style: "color: #7c3aed; font-weight: 700;", "✓" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    FieldDescription {
                        id: Some("field-s4-cbx-desc".to_string()),
                        "data-gui-test-id": "field-s4-cbx-desc",
                        "Input -> Trigger -> Item interactions do not fire on_focus_out until focus leaves the combobox."
                    }
                }

                div {
                    style: "{READOUT_ROW_STYLE}",
                    span {
                        "value: "
                        strong {
                            id: "field-s4-cbx-value",
                            "data-gui-test-id": "field-s4-cbx-value",
                            style: "{pill_style(true)}",
                            "{cbx_val_text}"
                        }
                    }
                    span {
                        "attempts: "
                        strong {
                            id: "field-s4-cbx-attempts",
                            "data-gui-test-id": "field-s4-cbx-attempts",
                            style: "{pill_style(cbx_attempts() > 0)}",
                            "{cbx_attempts()}"
                        }
                    }
                    span {
                        "on_focus_out: "
                        strong {
                            id: "field-s4-cbx-focus-out-count",
                            "data-gui-test-id": "field-s4-cbx-focus-out-count",
                            style: "{pill_style(cbx_focus_out() > 0)}",
                            "{cbx_focus_out()}"
                        }
                    }
                }
            }

            // 4B: DatePicker
            div {
                style: "{STAGE_STYLE}",
                div {
                    style: "{TOGGLE_ROW_STYLE}",
                    button {
                        id: "field-s4-dp-invalid-toggle",
                        "data-gui-test-id": "field-s4-dp-invalid-toggle",
                        r#type: "button",
                        style: "{toggle_btn_style(dp_invalid())}",
                        onclick: move |_| dp_invalid.set(!dp_invalid()),
                        if dp_invalid() { "DatePicker is_invalid_override: true" } else { "DatePicker is_invalid_override: false" }
                    }
                    button {
                        id: "field-s4-dp-reject-toggle",
                        "data-gui-test-id": "field-s4-dp-reject-toggle",
                        r#type: "button",
                        style: "{toggle_btn_style(dp_reject())}",
                        onclick: move |_| dp_reject.set(!dp_reject()),
                        if dp_reject() { "DatePicker Reject updates: ON" } else { "DatePicker Reject updates: OFF" }
                    }
                }

                FieldRoot {
                    id: Some("field-s4-dp-root".to_string()),
                    "data-gui-test-id": "field-s4-dp-root",
                    name: Some("release_date".to_string()),
                    invalid: dp_invalid(),

                    FieldLabel {
                        id: Some("field-s4-dp-label".to_string()),
                        "data-gui-test-id": "field-s4-dp-label",
                        "Release Date (DatePicker Pattern B)"
                    }

                    DatePickerRoot {
                        id: Some("field-s4-dp".to_string()),
                        controlled_value: Some(ReadSignal::from(dp_value)),
                        default_placeholder: Some(sample_date(2025, 4, 15)),
                        is_invalid_override: Some(dp_invalid()),
                        close_on_date_select: true,
                        locale: CalendarLocale::EnUs,
                        on_value_change: move |next: Option<DateValue>| {
                            if !dp_reject() {
                                dp_value.set(next);
                            }
                        },
                        on_focus_out: move |_| {
                            dp_focus_out += 1;
                        },

                        DatePickerInput {
                            id: Some("field-s4-dp-input".to_string()),
                            "data-gui-test-id": "field-s4-dp-input",
                            class: Some("fld-dp-input".to_string()),
                            name: Some("release_date".to_string()),
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DatePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: Some(format!("field-s4-dp-seg-{}", seg.index)),
                                        class: Some("fld-dp-seg".to_string()),
                                    }
                                }
                                DatePickerTrigger {
                                    id: Some("field-s4-dp-trigger".to_string()),
                                    "data-gui-test-id": "field-s4-dp-trigger",
                                    class: Some("fld-dp-trigger".to_string()),
                                    "Calendar"
                                }
                            }
                        }

                        DatePickerPortal {
                            DatePickerContent {
                                id: Some("field-s4-dp-content".to_string()),
                                "data-gui-test-id": "field-s4-dp-content",
                                style: Some("z-index: 60; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; box-shadow: 0 10px 24px rgba(15, 23, 42, 0.14); padding: 0.875rem; outline: none;".to_string()),
                                DatePickerCalendar {
                                    id: Some("field-s4-dp-calendar".to_string()),
                                    render_children: move |snap: CalendarSnapshot| rsx! {
                                        DatePickerHeader {
                                            style: Some("display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; margin-bottom: 0.5rem;".to_string()),
                                            DatePickerPrevButton {
                                                id: Some("field-s4-dp-prev".to_string()),
                                                style: Some("height: 28px; min-width: 28px; border-radius: 0.375rem; border: 1px solid #cbd5e1; background-color: #ffffff; cursor: pointer;".to_string()),
                                                "‹"
                                            }
                                            DatePickerHeading {
                                                id: Some("field-s4-dp-heading".to_string()),
                                                style: Some("font-size: 0.8125rem; font-weight: 700; color: #0f172a;".to_string()),
                                            }
                                            DatePickerNextButton {
                                                id: Some("field-s4-dp-next".to_string()),
                                                style: Some("height: 28px; min-width: 28px; border-radius: 0.375rem; border: 1px solid #cbd5e1; background-color: #ffffff; cursor: pointer;".to_string()),
                                                "›"
                                            }
                                        }
                                        DatePickerGrid {
                                            id: Some("field-s4-dp-grid".to_string()),
                                            style: Some("border-collapse: collapse; table-layout: fixed; width: 238px;".to_string()),
                                            DatePickerGridHead {
                                                DatePickerGridRow {
                                                    for (w_idx, wd) in snap.weekdays.iter().enumerate() {
                                                        DatePickerHeadCell {
                                                            key: "{w_idx}",
                                                            style: Some("width: 34px; height: 24px; text-align: center; font-size: 0.75rem; font-weight: 600; color: #64748b;".to_string()),
                                                            "{wd}"
                                                        }
                                                    }
                                                }
                                            }
                                            DatePickerGridBody {
                                                for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                                    DatePickerGridRow {
                                                        key: "{r_idx}",
                                                        for date in week.iter().copied() {
                                                            DatePickerCell {
                                                                key: "{date}",
                                                                date,
                                                                month: snap.months[0].value,
                                                                style: Some("width: 34px; height: 34px; padding: 1px; text-align: center;".to_string()),
                                                                DatePickerDay {
                                                                    id: Some(format!("field-s4-dp-day-{date}")),
                                                                    class: Some("fld-dp-day".to_string()),
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
                        }
                    }

                    FieldDescription {
                        id: Some("field-s4-dp-desc".to_string()),
                        "data-gui-test-id": "field-s4-dp-desc",
                        "Segment-to-segment and calendar popover focus stays inside the composite DatePicker."
                    }

                    FieldError {
                        id: Some("field-s4-dp-error".to_string()),
                        "data-gui-test-id": "field-s4-dp-error",
                        errors: if dp_invalid() {
                            vec!["Selected release date is blocked by validation.".to_string()]
                        } else {
                            vec![]
                        },
                    }
                }

                div {
                    style: "{READOUT_ROW_STYLE}",
                    span {
                        "date: "
                        strong {
                            id: "field-s4-dp-value",
                            "data-gui-test-id": "field-s4-dp-value",
                            style: "{pill_style(true)}",
                            "{dp_val_text}"
                        }
                    }
                    span {
                        "on_focus_out: "
                        strong {
                            id: "field-s4-dp-focus-out-count",
                            "data-gui-test-id": "field-s4-dp-focus-out-count",
                            style: "{pill_style(dp_focus_out() > 0)}",
                            "{dp_focus_out()}"
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Section 5: PinInput <input> ARIA Forwarding & Slider required + <form> Reset
// ---------------------------------------------------------------------------
#[component]
fn PinAndSliderFormResetSection() -> Element {
    let mut s5_invalid = use_signal(|| false);
    let mut pin_val = use_signal(|| "12".to_string());
    let mut pin_blur_count = use_signal(|| 0usize);
    let mut slider_val = use_signal(|| 40.0_f64);
    let mut slider_blur_count = use_signal(|| 0usize);
    let mut reset_count = use_signal(|| 0usize);

    rsx! {
        section {
            id: "field-section-5",
            "data-gui-test-id": "field-section-5",
            style: "{CARD_STYLE}",

            div {
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 700; color: #0f172a;",
                    "5. PinInput <input> ARIA Forwarding & Slider required + <form> Reset (FM-CTL-03, FM-CTL-04)"
                }
                p {
                    style: "margin: 0.25rem 0 0; font-size: 0.8125rem; color: #64748b;",
                    "Both PinInputRoot and SliderRoot inherit field ARIA attributes (aria-invalid, aria-describedby, aria-required) and reset to their mount-time defaults on native <form> reset."
                }
            }

            div {
                style: "{STAGE_STYLE}",
                form {
                    id: "field-s5-form",
                    "data-gui-test-id": "field-s5-form",
                    style: "display: flex; flex-direction: column; gap: 1.25rem;",
                    onsubmit: move |evt| evt.prevent_default(),

                    FieldRoot {
                        id: Some("field-s5-pin-root".to_string()),
                        "data-gui-test-id": "field-s5-pin-root",
                        name: Some("otp_code".to_string()),
                        invalid: s5_invalid(),
                        required: true,

                        FieldLabel {
                            id: Some("field-s5-pin-label".to_string()),
                            "data-gui-test-id": "field-s5-pin-label",
                            "Verification Code (PinInput)"
                        }

                        PinInputRoot {
                            id: Some("field-s5-pin".to_string()),
                            input_id: Some("field-s5-pin-input".to_string()),
                            "data-gui-test-id": "field-s5-pin",
                            max_length: 4,
                            pattern: PinInputPattern::Digits,
                            input_mode: PinInputMode::Numeric,
                            default_value: Some("12".to_string()),
                            style: Some("display: inline-flex; align-items: center; gap: 8px;".to_string()),
                            on_value_change: move |v: String| pin_val.set(v),
                            on_blur: move |_| {
                                pin_blur_count += 1;
                            },
                            render_children: move |snap: PinInputSnapshot| rsx! {
                                for cell_ref in snap.cells.iter() {
                                    {
                                        let cell = *cell_ref;
                                        let display_char = cell.display_char(snap.mask, '•');
                                        rsx! {
                                            PinInputCell {
                                                key: "{cell.index}",
                                                id: Some(format!("field-s5-pin-cell-{}", cell.index)),
                                                class: Some("fld-pin-cell".to_string()),
                                                index: cell.index,
                                                if let Some(ch) = display_char {
                                                    span { "{ch}" }
                                                } else if cell.has_fake_caret {
                                                    div {
                                                        style: "width: 2px; height: 20px; background-color: #7c3aed; border-radius: 1px;",
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        FieldDescription {
                            id: Some("field-s5-pin-desc".to_string()),
                            "data-gui-test-id": "field-s5-pin-desc",
                            "4-digit OTP. Hidden <input id=\"field-s5-pin-input\"> forwards aria-invalid, aria-required, and aria-describedby."
                        }

                        FieldError {
                            id: Some("field-s5-pin-error".to_string()),
                            "data-gui-test-id": "field-s5-pin-error",
                            errors: if s5_invalid() {
                                vec!["Verification code must match the emailed PIN.".to_string()]
                            } else {
                                vec![]
                            },
                        }
                    }

                    FieldRoot {
                        id: Some("field-s5-slider-root".to_string()),
                        "data-gui-test-id": "field-s5-slider-root",
                        name: Some("priority_level".to_string()),
                        invalid: s5_invalid(),
                        required: true,

                        FieldLabel {
                            id: Some("field-s5-slider-label".to_string()),
                            "data-gui-test-id": "field-s5-slider-label",
                            "Priority Level (Slider required=true)"
                        }

                        div {
                            style: "padding: 0.375rem 0;",
                            SliderRoot {
                                id: Some("field-s5-slider".to_string()),
                                "data-gui-test-id": "field-s5-slider",
                                min: 0.0,
                                max: 100.0,
                                step: 5.0,
                                default_value: Some(SliderValue::Single(40.0)),
                                required: true,
                                style: Some("width: 280px; height: 24px; display: flex; align-items: center;".to_string()),
                                on_value_change: move |v| {
                                    if let SliderValue::Single(n) = v {
                                        slider_val.set(n);
                                    }
                                },
                                on_focus_out: move |_| {
                                    slider_blur_count += 1;
                                },
                                SliderTrack {
                                    id: Some("field-s5-slider-track".to_string()),
                                    class: Some("sw2b-slider-track".to_string()),
                                    style: Some("width: 280px; height: 8px; background: #e2e8f0; border-radius: 9999px; position: relative;".to_string()),
                                    SliderRange {
                                        id: Some("field-s5-slider-range".to_string()),
                                        class: Some("sw2b-slider-range".to_string()),
                                        style: Some("background: #7c3aed; border-radius: 9999px;".to_string()),
                                    }
                                }
                                SliderThumb {
                                    id: Some("field-s5-slider-thumb".to_string()),
                                    "data-gui-test-id": "field-s5-slider-thumb",
                                    index: 0,
                                    class: Some("sw2b-slider-thumb".to_string()),
                                    aria_label: Some("Priority Level".to_string()),
                                }
                            }
                        }

                        FieldDescription {
                            id: Some("field-s5-slider-desc".to_string()),
                            "data-gui-test-id": "field-s5-slider-desc",
                            "Thumb exposes aria-required, aria-invalid, and aria-describedby from FieldRoot."
                        }

                        FieldError {
                            id: Some("field-s5-slider-error".to_string()),
                            "data-gui-test-id": "field-s5-slider-error",
                            errors: if s5_invalid() {
                                vec!["Priority level is outside the approved range.".to_string()]
                            } else {
                                vec![]
                            },
                        }
                    }

                    div {
                        style: "{TOGGLE_ROW_STYLE}",
                        button {
                            id: "field-s5-invalid-toggle",
                            "data-gui-test-id": "field-s5-invalid-toggle",
                            r#type: "button",
                            style: "{toggle_btn_style(s5_invalid())}",
                            onclick: move |_| s5_invalid.set(!s5_invalid()),
                            if s5_invalid() { "Section 5 invalid: true" } else { "Section 5 invalid: false" }
                        }
                        button {
                            id: "field-s5-reset",
                            "data-gui-test-id": "field-s5-reset",
                            r#type: "reset",
                            style: "{toggle_btn_style(false)}",
                            onclick: move |_| {
                                pin_val.set("12".to_string());
                                slider_val.set(40.0);
                                reset_count += 1;
                            },
                            "Reset Form"
                        }
                    }
                }

                div {
                    style: "{READOUT_ROW_STYLE}",
                    span {
                        "pin: "
                        strong {
                            id: "field-s5-pin-value",
                            "data-gui-test-id": "field-s5-pin-value",
                            style: "{pill_style(true)}",
                            "{pin_val()}"
                        }
                    }
                    span {
                        "pin blurs: "
                        strong {
                            id: "field-s5-pin-blur-count",
                            "data-gui-test-id": "field-s5-pin-blur-count",
                            style: "{pill_style(pin_blur_count() > 0)}",
                            "{pin_blur_count()}"
                        }
                    }
                    span {
                        "slider: "
                        strong {
                            id: "field-s5-slider-value",
                            "data-gui-test-id": "field-s5-slider-value",
                            style: "{pill_style(true)}",
                            "{slider_val()}"
                        }
                    }
                    span {
                        "slider blurs: "
                        strong {
                            id: "field-s5-slider-blur-count",
                            "data-gui-test-id": "field-s5-slider-blur-count",
                            style: "{pill_style(slider_blur_count() > 0)}",
                            "{slider_blur_count()}"
                        }
                    }
                    span {
                        "resets: "
                        strong {
                            id: "field-s5-reset-count",
                            "data-gui-test-id": "field-s5-reset-count",
                            style: "{pill_style(reset_count() > 0)}",
                            "{reset_count()}"
                        }
                    }
                }
            }
        }
    }
}

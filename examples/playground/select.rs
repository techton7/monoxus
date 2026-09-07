use dioxus::prelude::*;
use monoxus::{
    foundation::shared::ScopeHandle,
    select::{
        Select, SelectContent, SelectGroup, SelectItem, SelectItemIndicator,
        SelectItemText, SelectLabel, SelectRoot, SelectSeparator, SelectTrigger, SelectValue, SelectViewport,
        use_select_runtime,
    },
};

const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #c084fc; background-color: white; box-shadow: 0 10px 30px rgba(147, 51, 234, 0.08);";
const MUTED_STYLE: &str = "margin: 0; color: #6b21a8;";
const BADGE_STYLE: &str = "display: inline-block; padding: 0.2rem 0.6rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; background-color: #f3e8ff; color: #7e22ce;";

const SELECT_PLAYGROUND_CSS: &str = r#"
.select-item, [role="option"] {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.375rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    cursor: pointer;
    user-select: none;
    transition: background-color 0.12s ease, color 0.12s ease;
}
.select-item[data-highlighted="true"], [role="option"][data-highlighted="true"] {
    background-color: #f3e8ff !important;
    color: #7e22ce !important;
}
.select-item[data-disabled="true"], [role="option"][data-disabled="true"] {
    opacity: 0.45 !important;
    cursor: not-allowed !important;
    pointer-events: none !important;
}
"#;

#[component]
pub fn SelectPlayground() -> Element {
    rsx! {
        section {
            style: CARD_STYLE,
            style { "{SELECT_PLAYGROUND_CSS}" }
            h2 {
                style: "margin: 0; color: #581c87;",
                "Select and Ordered Overlay Selection"
            }
            p {
                style: MUTED_STYLE,
                "Headless WAI-ARIA Select primitive with single concrete focus ownership, synchronous 500ms typeahead, roving candidate highlight, and grouped overlays."
            }

            BasicFruitSelectSection {}
            GroupedSelectSection {}
            ScrollableViewportSection {}
            FormIntegrationSection {}
            BottomConstrainedSelectSection {}
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 1: Basic Fruit Select
// -------------------------------------------------------------------------

#[component]
fn BasicFruitSelectSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-basic");
    let selected_val = use_signal(|| Some("apple".to_string()));
    let is_open = use_signal(|| false);

    let def = Select::new(scope.clone())
        .with_value(selected_val())
        .with_open(is_open());

    let runtime = use_select_runtime(
        def,
        Some(move |val: Option<String>| {
            let mut s = selected_val;
            s.set(val);
        }),
        Some(move |open: bool| {
            let mut s = is_open;
            s.set(open);
        }),
    );

    let curr_val = selected_val().unwrap_or_else(|| "none".into());
    let open_state = if is_open() { "Open" } else { "Closed" };

    rsx! {
        div {
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "1. Basic Fruit Select"
                }
                div {
                    style: "display: flex; gap: 0.5rem;",
                    span { style: BADGE_STYLE, "Selected: {curr_val}" }
                    span { style: BADGE_STYLE, "State: {open_state}" }
                }
            }

            div {
                style: "position: relative; width: 240px;",
                SelectRoot {
                    runtime: runtime.clone(),
                    SelectTrigger {
                        class: "select-trigger-basic".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                        SelectValue {
                            placeholder: "Select a fruit...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContent {
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                        SelectViewport {
                            SelectItem {
                                value: "apple".to_string(),
                                text: "Apple".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Apple", is_selected: selected_val().as_deref() == Some("apple") }
                            }
                            SelectItem {
                                value: "banana".to_string(),
                                text: "Banana".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Banana", is_selected: selected_val().as_deref() == Some("banana") }
                            }
                            SelectItem {
                                value: "blueberry".to_string(),
                                text: "Blueberry".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Blueberry", is_selected: selected_val().as_deref() == Some("blueberry") }
                            }
                            SelectItem {
                                value: "cherry".to_string(),
                                text: "Cherry".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Cherry", is_selected: selected_val().as_deref() == Some("cherry") }
                            }
                            SelectItem {
                                value: "grapes".to_string(),
                                text: "Grapes".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Grapes", is_selected: selected_val().as_deref() == Some("grapes") }
                            }
                        }
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 2: Grouped Select with Labels
// -------------------------------------------------------------------------

#[component]
fn GroupedSelectSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-grouped");
    let selected_val = use_signal(|| Some("carrot".to_string()));
    let is_open = use_signal(|| false);

    let def = Select::new(scope.clone())
        .with_value(selected_val())
        .with_open(is_open());

    let runtime = use_select_runtime(
        def,
        Some(move |val: Option<String>| {
            let mut s = selected_val;
            s.set(val);
        }),
        Some(move |open: bool| {
            let mut s = is_open;
            s.set(open);
        }),
    );

    let curr_val = selected_val().unwrap_or_else(|| "none".into());

    rsx! {
        div {
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "2. Grouped Select with Category Labels"
                }
                span { style: BADGE_STYLE, "Choice: {curr_val}" }
            }

            div {
                style: "position: relative; width: 260px;",
                SelectRoot {
                    runtime: runtime.clone(),
                    SelectTrigger {
                        class: "select-trigger-grouped".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none;",
                        SelectValue {
                            placeholder: "Choose food...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContent {
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                        SelectViewport {
                            SelectGroup {
                                SelectLabel {
                                    id: "fruits-label".to_string(),
                                    class: "select-label".to_string(),
                                    span { style: "display: block; padding: 0.25rem 0.5rem; font-size: 0.75rem; font-weight: 700; color: #a855f7; text-transform: uppercase;", "Fruits" }
                                }
                                SelectItem {
                                    value: "apple".to_string(),
                                    text: "Apple".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Apple", is_selected: selected_val().as_deref() == Some("apple") }
                                }
                                SelectItem {
                                    value: "orange".to_string(),
                                    text: "Orange".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Orange", is_selected: selected_val().as_deref() == Some("orange") }
                                }
                            }
                            SelectSeparator {
                                class: "select-sep".to_string(),
                            }
                            SelectGroup {
                                SelectLabel {
                                    id: "vegetables-label".to_string(),
                                    class: "select-label".to_string(),
                                    span { style: "display: block; padding: 0.25rem 0.5rem; font-size: 0.75rem; font-weight: 700; color: #a855f7; text-transform: uppercase;", "Vegetables" }
                                }
                                SelectItem {
                                    value: "carrot".to_string(),
                                    text: "Carrot".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Carrot", is_selected: selected_val().as_deref() == Some("carrot") }
                                }
                                SelectItem {
                                    value: "broccoli".to_string(),
                                    text: "Broccoli".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Broccoli", is_selected: selected_val().as_deref() == Some("broccoli") }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 3: Scrollable Viewport with Disabled Items
// -------------------------------------------------------------------------

#[component]
fn ScrollableViewportSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-scroll");
    let selected_val = use_signal(|| None::<String>);
    let is_open = use_signal(|| false);

    let def = Select::new(scope.clone())
        .with_value(selected_val())
        .with_open(is_open());

    let runtime = use_select_runtime(
        def,
        Some(move |val: Option<String>| {
            let mut s = selected_val;
            s.set(val);
        }),
        Some(move |open: bool| {
            let mut s = is_open;
            s.set(open);
        }),
    );

    let curr_val = selected_val().unwrap_or_else(|| "none".into());

    rsx! {
        div {
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "3. Scrollable Viewport with Disabled Items"
                }
                span { style: BADGE_STYLE, "Choice: {curr_val}" }
            }

            div {
                style: "position: relative; width: 280px;",
                SelectRoot {
                    runtime: runtime.clone(),
                    SelectTrigger {
                        class: "select-trigger-scrollable".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none;",
                        SelectValue {
                            placeholder: "Select country...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                    }
                    SelectContent {
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none; max-height: 160px; overflow-y: auto;",
                        SelectViewport {
                            SelectItem {
                                value: "argentina".to_string(),
                                text: "Argentina".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Argentina", is_selected: selected_val().as_deref() == Some("argentina") }
                            }
                            SelectItem {
                                value: "brazil".to_string(),
                                text: "Brazil (Disabled)".to_string(),
                                disabled: true,
                                class: "select-item".to_string(),
                                ItemRow { text: "Brazil (Disabled)", is_selected: false }
                            }
                            SelectItem {
                                value: "canada".to_string(),
                                text: "Canada".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Canada", is_selected: selected_val().as_deref() == Some("canada") }
                            }
                            SelectItem {
                                value: "denmark".to_string(),
                                text: "Denmark".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Denmark", is_selected: selected_val().as_deref() == Some("denmark") }
                            }
                            SelectItem {
                                value: "egypt".to_string(),
                                text: "Egypt (Disabled)".to_string(),
                                disabled: true,
                                class: "select-item".to_string(),
                                ItemRow { text: "Egypt (Disabled)", is_selected: false }
                            }
                            SelectItem {
                                value: "france".to_string(),
                                text: "France".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "France", is_selected: selected_val().as_deref() == Some("france") }
                            }
                            SelectItem {
                                value: "germany".to_string(),
                                text: "Germany".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Germany", is_selected: selected_val().as_deref() == Some("germany") }
                            }
                            SelectItem {
                                value: "japan".to_string(),
                                text: "Japan".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Japan", is_selected: selected_val().as_deref() == Some("japan") }
                            }
                            SelectItem {
                                value: "korea".to_string(),
                                text: "Korea".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Korea", is_selected: selected_val().as_deref() == Some("korea") }
                            }
                        }
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 4: Form Integration with SelectHiddenInput
// -------------------------------------------------------------------------

#[component]
fn FormIntegrationSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-form");
    let selected_val = use_signal(|| Some("pro".to_string()));
    let is_open = use_signal(|| false);
    let submitted_value = use_signal(|| None::<String>);

    let def = Select::new(scope.clone())
        .with_value(selected_val())
        .with_open(is_open());

    let runtime = use_select_runtime(
        def,
        Some(move |val: Option<String>| {
            let mut s = selected_val;
            s.set(val);
        }),
        Some(move |open: bool| {
            let mut s = is_open;
            s.set(open);
        }),
    );

    let sub_disp = submitted_value().unwrap_or_else(|| "None yet".into());

    rsx! {
        div {
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "4. Native Form Integration"
                }
                span { style: BADGE_STYLE, "Submitted: {sub_disp}" }
            }

            form {
                onsubmit: move |evt| {
                    evt.prevent_default();
                    let mut s = submitted_value;
                    s.set(selected_val());
                },
                style: "display: flex; gap: 1rem; align-items: center;",
                div {
                    style: "position: relative; width: 220px;",
                    SelectRoot {
                        runtime: runtime.clone(),
                        name: "tier".to_string(),
                        SelectTrigger {
                            class: "select-trigger-form".to_string(),
                            style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none;",
                            SelectValue {
                                placeholder: "Choose tier...".to_string(),
                            }
                            span { style: "color: #9333ea; font-size: 0.75rem;", "▼" }
                        }
                        SelectContent {
                            style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                            SelectViewport {
                                SelectItem {
                                    value: "starter".to_string(),
                                    text: "Starter ($0)".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Starter ($0)", is_selected: selected_val().as_deref() == Some("starter") }
                                }
                                SelectItem {
                                    value: "pro".to_string(),
                                    text: "Pro ($29)".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Pro ($29)", is_selected: selected_val().as_deref() == Some("pro") }
                                }
                                SelectItem {
                                    value: "enterprise".to_string(),
                                    text: "Enterprise ($99)".to_string(),
                                    class: "select-item".to_string(),
                                    ItemRow { text: "Enterprise ($99)", is_selected: selected_val().as_deref() == Some("enterprise") }
                                }
                            }
                        }
                    }
                }
                button {
                    r#type: "submit",
                    id: "select-form-submit-btn",
                    style: "padding: 0.5rem 1rem; border-radius: 0.375rem; background-color: #9333ea; color: white; border: none; font-size: 0.875rem; font-weight: 500; cursor: pointer;",
                    "Submit Form"
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Helper: ItemRow
// -------------------------------------------------------------------------

#[component]
fn ItemRow(text: &'static str, is_selected: bool) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; width: 100%;",
            SelectItemText {
                span { "{text}" }
            }
            SelectItemIndicator {
                if is_selected {
                    span { style: "color: #9333ea; font-weight: bold; margin-left: 0.5rem;", "✓" }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 5: Bottom-Constrained Select (Collision Flip Proof)
// -------------------------------------------------------------------------

#[component]
fn BottomConstrainedSelectSection() -> Element {
    let scope = ScopeHandle::root("playground").child("select-flip");
    let selected_val = use_signal(|| Some("upward-1".to_string()));
    let is_open = use_signal(|| false);

    let def = Select::new(scope.clone())
        .with_value(selected_val())
        .with_open(is_open());

    let runtime = use_select_runtime(
        def,
        Some(move |val: Option<String>| {
            let mut s = selected_val;
            s.set(val);
        }),
        Some(move |open: bool| {
            let mut s = is_open;
            s.set(open);
        }),
    );

    let curr_val = selected_val().unwrap_or_else(|| "none".into());
    let open_state = if is_open() { "Open" } else { "Closed" };
    let current_side = runtime.side().as_str();

    rsx! {
        div {
            id: "select-flip-container",
            style: "border: 1px solid #e9d5ff; border-radius: 0.5rem; padding: 1.25rem; background-color: #faf5ff; margin-top: 2rem;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; color: #581c87;",
                    "5. Bottom-Constrained Select (Viewport Collision Flip)"
                }
                div {
                    style: "display: flex; gap: 0.5rem;",
                    span { style: BADGE_STYLE, "Selected: {curr_val}" }
                    span { style: BADGE_STYLE, "State: {open_state}" }
                    span {
                        id: "select-flip-side-badge",
                        style: BADGE_STYLE,
                        "side: {current_side}"
                    }
                }
            }
            p {
                style: MUTED_STYLE,
                "Positioned near viewport bottom. When space below is constrained (< content height), FloatingLayer flips placement to data-side=\"top\"."
            }

            div {
                style: "position: relative; width: 260px; margin-top: 1rem; margin-bottom: 0.5rem;",
                SelectRoot {
                    runtime: runtime.clone(),
                    SelectTrigger {
                        class: "select-trigger-flip".to_string(),
                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d8b4fe; border-radius: 0.375rem; background: white; font-size: 0.875rem; cursor: pointer; outline: none; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                        SelectValue {
                            placeholder: "Choose option...".to_string(),
                        }
                        span { style: "color: #9333ea; font-size: 0.75rem;", "▲/▼" }
                    }
                    SelectContent {
                        class: "select-content-flip".to_string(),
                        style: "z-index: 50; background: white; border: 1px solid #d8b4fe; border-radius: 0.375rem; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1); padding: 0.25rem; outline: none;",
                        SelectViewport {
                            SelectItem {
                                value: "upward-1".to_string(),
                                text: "Upward Option 1".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Upward Option 1", is_selected: selected_val().as_deref() == Some("upward-1") }
                            }
                            SelectItem {
                                value: "upward-2".to_string(),
                                text: "Upward Option 2".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Upward Option 2", is_selected: selected_val().as_deref() == Some("upward-2") }
                            }
                            SelectItem {
                                value: "upward-3".to_string(),
                                text: "Upward Option 3".to_string(),
                                class: "select-item".to_string(),
                                ItemRow { text: "Upward Option 3", is_selected: selected_val().as_deref() == Some("upward-3") }
                            }
                        }
                    }
                }
            }
        }
    }
}

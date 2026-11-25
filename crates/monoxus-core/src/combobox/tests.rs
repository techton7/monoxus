use crate::foundation::{
    overlay::{PlacementAlign, PlacementSide},
    shared::ScopeHandle,
};

use super::{
    attrs::{
        ComboboxContentAttrs, ComboboxGroupAttrs, ComboboxGroupHeadingAttrs, ComboboxInputAttrs,
        ComboboxItemAttrs, ComboboxScrollButtonAttrs, ComboboxTriggerAttrs, ComboboxViewportAttrs,
    },
    registry::ComboboxOptionRegistry,
    relationships::ComboboxRelationships,
    state::{
        ComboboxKeyOutcome, ComboboxState, apply_multiple_toggle, apply_single_toggle,
        compute_viewport_scroll_target, resolve_input_keydown,
    },
    types::{
        COMBOBOX_PARTS, ComboboxItemData, ComboboxMode, ComboboxNavKey, ComboboxPart,
        ComboboxScrollAlignment,
    },
};

fn sample_registry(count: usize) -> ComboboxOptionRegistry {
    let mut reg = ComboboxOptionRegistry::new();
    for i in 0..count {
        reg.register(
            format!("opt-{i}"),
            format!("Option {i}"),
            format!("cbx-item-opt-{i}"),
            i == 2,
        );
    }
    reg
}

#[test]
fn parts_and_relationships_are_deterministic() {
    assert_eq!(COMBOBOX_PARTS.len(), 14);
    assert_eq!(ComboboxPart::Input.as_str(), "input");
    assert_eq!(ComboboxPart::GroupHeading.as_str(), "group-heading");
    assert_eq!(ComboboxPart::ScrollUpButton.as_str(), "scroll-up-button");
    assert_eq!(ComboboxPart::ScrollDownButton.as_str(), "scroll-down-button");
    assert_eq!(ComboboxPart::HiddenInput.as_str(), "hidden-input");

    let scope = ScopeHandle::root("combobox").child("fruits");
    let rels = ComboboxRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.input_id(), scope.qualify("input"));
    assert_eq!(rels.trigger_id(), scope.qualify("trigger"));
    assert_eq!(rels.content_id(), scope.qualify("content"));
    assert_eq!(rels.viewport_id(), scope.qualify("viewport"));
    assert_eq!(rels.group_id("citrus"), scope.qualify("group-citrus"));
    assert_eq!(
        rels.group_heading_id("citrus"),
        scope.qualify("group-heading-citrus")
    );
    assert_eq!(rels.item_id("orange"), scope.qualify("item-orange"));
}

#[test]
fn single_toggle_respects_allow_deselect_and_clear_on_deselect() {
    // Select new item
    let selected = apply_single_toggle(None, "apple", "Apple", true, false);
    assert_eq!(selected.next_value.as_deref(), Some("apple"));
    assert_eq!(selected.next_input_value.as_deref(), Some("Apple"));
    assert!(selected.should_close);
    assert!(selected.changed);

    // Deselect with allow_deselect = true, clear_on_deselect = false
    let deselected_keep = apply_single_toggle(Some("apple"), "apple", "Apple", true, false);
    assert_eq!(deselected_keep.next_value, None);
    assert_eq!(deselected_keep.next_input_value, None);
    assert!(deselected_keep.should_close);
    assert!(deselected_keep.changed);

    // Deselect with allow_deselect = true, clear_on_deselect = true
    let deselected_clear = apply_single_toggle(Some("apple"), "apple", "Apple", true, true);
    assert_eq!(deselected_clear.next_value, None);
    assert_eq!(deselected_clear.next_input_value.as_deref(), Some(""));
    assert!(deselected_clear.should_close);
    assert!(deselected_clear.changed);

    // Toggle selected item with allow_deselect = false closes without deselecting
    let locked = apply_single_toggle(Some("apple"), "apple", "Apple", false, true);
    assert_eq!(locked.next_value.as_deref(), Some("apple"));
    assert_eq!(locked.next_input_value, None);
    assert!(locked.should_close);
    assert!(!locked.changed);
}

#[test]
fn multiple_toggle_updates_membership_and_clears_only_when_empty() {
    let first = apply_multiple_toggle(&[], "a", "Alpha", true);
    assert_eq!(first.next_values, vec!["a"]);
    assert_eq!(first.next_input_value.as_deref(), Some("Alpha"));

    let second = apply_multiple_toggle(&first.next_values, "b", "Beta", true);
    assert_eq!(second.next_values, vec!["a", "b"]);
    assert_eq!(second.next_input_value.as_deref(), Some("Beta"));

    // Removing one while another remains does not clear input
    let remove_one = apply_multiple_toggle(&second.next_values, "a", "Alpha", true);
    assert_eq!(remove_one.next_values, vec!["b"]);
    assert_eq!(remove_one.next_input_value, None);

    // Removing last item clears input when clear_on_deselect = true
    let remove_last = apply_multiple_toggle(&remove_one.next_values, "b", "Beta", true);
    assert!(remove_last.next_values.is_empty());
    assert_eq!(remove_last.next_input_value.as_deref(), Some(""));

    // Removing last item keeps input when clear_on_deselect = false
    let remove_last_keep = apply_multiple_toggle(&remove_one.next_values, "b", "Beta", false);
    assert!(remove_last_keep.next_values.is_empty());
    assert_eq!(remove_last_keep.next_input_value, None);
}

#[test]
fn registry_skips_disabled_items_and_caches_labels_after_unmount() {
    let mut reg = sample_registry(5);
    reg.seed_items(&[ComboboxItemData::new("seeded", "Seeded Label", false)]);
    assert_eq!(reg.label_for("seeded"), Some("Seeded Label"));
    assert_eq!(reg.label_for("opt-1"), Some("Option 1"));

    // Unmount opt-1; label cache must retain its label for closed-state hydration
    assert!(reg.unregister_by_dom_id("cbx-item-opt-1"));
    assert_eq!(reg.label_for("opt-1"), Some("Option 1"));
    assert!(!reg.contains_enabled("opt-1"));

    // Disabled opt-2 is skipped during navigation
    assert_eq!(
        reg.next_enabled(Some("opt-0"), false).map(|i| i.value.as_str()),
        Some("opt-3")
    );
    assert_eq!(
        reg.prev_enabled(Some("opt-3"), false).map(|i| i.value.as_str()),
        Some("opt-0")
    );
    // Loop wraps around
    assert_eq!(
        reg.next_enabled(Some("opt-4"), true).map(|i| i.value.as_str()),
        Some("opt-0")
    );
    assert_eq!(
        reg.prev_enabled(Some("opt-0"), true).map(|i| i.value.as_str()),
        Some("opt-4")
    );
    // Clamps when loop_focus = false
    assert_eq!(
        reg.next_enabled(Some("opt-4"), false).map(|i| i.value.as_str()),
        Some("opt-4")
    );
}

#[test]
fn keyboard_reducer_covers_arrows_page_alt_tab_escape_enter_home_end() {
    let reg = sample_registry(15);

    // Home/End pass through to native <input> caret
    assert_eq!(
        resolve_input_keydown(&reg, true, Some("opt-1"), None, ComboboxNavKey::Home, false, false, false),
        ComboboxKeyOutcome::default()
    );
    assert_eq!(
        resolve_input_keydown(&reg, true, Some("opt-1"), None, ComboboxNavKey::End, false, false, false),
        ComboboxKeyOutcome::default()
    );

    // ArrowDown when closed opens & highlights first enabled
    let open_down = resolve_input_keydown(&reg, false, None, None, ComboboxNavKey::ArrowDown, false, false, false);
    assert!(open_down.prevent_default);
    assert_eq!(open_down.next_open, Some(true));
    assert_eq!(open_down.next_highlighted, Some(Some("opt-0".to_string())));

    // ArrowUp when closed opens & highlights last enabled
    let open_up = resolve_input_keydown(&reg, false, None, None, ComboboxNavKey::ArrowUp, false, false, false);
    assert!(open_up.prevent_default);
    assert_eq!(open_up.next_open, Some(true));
    assert_eq!(open_up.next_highlighted, Some(Some("opt-14".to_string())));

    // PageDown moves +10 enabled items (skipping disabled opt-2)
    let paged_down = resolve_input_keydown(&reg, true, Some("opt-0"), None, ComboboxNavKey::PageDown, false, false, false);
    assert!(paged_down.prevent_default);
    assert_eq!(paged_down.next_highlighted, Some(Some("opt-11".to_string())));

    // PageUp moves -10 enabled items back to opt-0
    let paged_up = resolve_input_keydown(&reg, true, Some("opt-11"), None, ComboboxNavKey::PageUp, false, false, false);
    assert!(paged_up.prevent_default);
    assert_eq!(paged_up.next_highlighted, Some(Some("opt-0".to_string())));

    // Alt+ArrowUp closes with prevent_default
    let alt_up = resolve_input_keydown(&reg, true, Some("opt-0"), None, ComboboxNavKey::AltArrowUp, false, false, false);
    assert!(alt_up.prevent_default);
    assert_eq!(alt_up.next_open, Some(false));

    // Tab closes without prevent_default
    let tab = resolve_input_keydown(&reg, true, Some("opt-0"), None, ComboboxNavKey::Tab, false, false, false);
    assert!(!tab.prevent_default);
    assert_eq!(tab.next_open, Some(false));

    // Escape closes with prevent_default (preserving input focus on Blitz Native)
    let esc = resolve_input_keydown(&reg, true, Some("opt-0"), None, ComboboxNavKey::Escape, false, false, false);
    assert!(esc.prevent_default);
    assert_eq!(esc.next_open, Some(false));

    // Enter selects only when not composing (4B D-5)
    let enter_composing = resolve_input_keydown(&reg, true, Some("opt-0"), None, ComboboxNavKey::Enter, true, false, false);
    assert!(!enter_composing.prevent_default);
    assert!(!enter_composing.trigger_select);

    let enter_ready = resolve_input_keydown(&reg, true, Some("opt-0"), None, ComboboxNavKey::Enter, false, false, false);
    assert!(enter_ready.prevent_default);
    assert!(enter_ready.trigger_select);

    // Printable when closed opens menu without prevent_default
    let printable = resolve_input_keydown(&reg, false, None, None, ComboboxNavKey::Printable, false, false, false);
    assert!(!printable.prevent_default);
    assert_eq!(printable.next_open, Some(true));
}

#[test]
fn attrs_enforce_aria_disabled_4b_d6_and_boolean_data_omission() {
    let input_closed = ComboboxInputAttrs::new("in", "ct", false, Some("it-1".into()), false);
    assert_eq!(input_closed.role(), "combobox");
    assert_eq!(input_closed.aria_autocomplete(), "list");
    assert_eq!(input_closed.aria_expanded(), "false");
    assert_eq!(input_closed.aria_activedescendant(), None);
    assert_eq!(input_closed.data_state_str(), "closed");
    assert_eq!(input_closed.data_disabled(), None);

    let input_open = ComboboxInputAttrs::new("in", "ct", true, Some("it-1".into()), true);
    assert_eq!(input_open.aria_expanded(), "true");
    assert_eq!(input_open.aria_activedescendant(), Some("it-1"));
    assert_eq!(input_open.data_state_str(), "open");
    assert_eq!(input_open.data_disabled(), Some(""));

    let trigger = ComboboxTriggerAttrs::new("tr", "ct", true, false);
    assert_eq!(trigger.aria_haspopup(), "listbox");
    assert_eq!(trigger.aria_expanded(), "true");
    assert_eq!(trigger.data_disabled(), None);

    let content_single = ComboboxContentAttrs::new(
        "ct",
        ComboboxMode::Single,
        true,
        PlacementSide::Bottom,
        PlacementAlign::Start,
    );
    assert_eq!(content_single.role(), "listbox");
    assert_eq!(content_single.tabindex(), -1);
    assert_eq!(content_single.aria_multiselectable(), None);

    let content_multi = ComboboxContentAttrs::new(
        "ct",
        ComboboxMode::Multiple,
        true,
        PlacementSide::Bottom,
        PlacementAlign::Start,
    );
    assert_eq!(content_multi.aria_multiselectable(), Some("true"));

    // Enabled unselected unhighlighted item omits all boolean data-* and aria-disabled
    let item_idle = ComboboxItemAttrs::new("it-1", "a", "Alpha", false, false, false);
    assert_eq!(item_idle.role(), "option");
    assert_eq!(item_idle.aria_selected(), "false");
    assert_eq!(item_idle.aria_disabled(), None);
    assert_eq!(item_idle.data_disabled(), None);
    assert_eq!(item_idle.data_highlighted(), None);
    assert_eq!(item_idle.data_selected(), None);

    // Disabled item emits aria-disabled="true" (4B D-6) and suppresses data-highlighted
    let item_disabled = ComboboxItemAttrs::new("it-2", "b", "Beta", true, true, true);
    assert_eq!(item_disabled.aria_disabled(), Some("true"));
    assert_eq!(item_disabled.data_disabled(), Some(""));
    assert_eq!(item_disabled.data_highlighted(), None);
    assert_eq!(item_disabled.data_selected(), Some(""));

    let vp = ComboboxViewportAttrs::new("vp");
    assert_eq!(vp.role(), "presentation");

    let grp = ComboboxGroupAttrs::new("g1", Some("gh1".into()));
    assert_eq!(grp.role(), "group");
    assert_eq!(grp.aria_labelledby(), Some("gh1"));
    assert_eq!(ComboboxGroupHeadingAttrs::new("gh1").id(), "gh1");
    assert_eq!(ComboboxScrollButtonAttrs::new().aria_hidden(), "true");
}

#[test]
fn viewport_scroll_alignment_nearest_and_center_math() {
    // Item below visible viewport -> Nearest scrolls down
    assert_eq!(
        compute_viewport_scroll_target(100.0, 200.0, 100.0, 0.0, 210.0, 240.0, 30.0, ComboboxScrollAlignment::Nearest),
        Some(41.0)
    );
    // Item above visible viewport -> Nearest scrolls up
    assert_eq!(
        compute_viewport_scroll_target(100.0, 200.0, 100.0, 50.0, 80.0, 110.0, 30.0, ComboboxScrollAlignment::Nearest),
        Some(30.0)
    );
    // Item already visible -> Nearest returns None
    assert_eq!(
        compute_viewport_scroll_target(100.0, 200.0, 100.0, 20.0, 120.0, 150.0, 30.0, ComboboxScrollAlignment::Nearest),
        None
    );
    // Center aligns item midpoint to container midpoint
    assert_eq!(
        compute_viewport_scroll_target(100.0, 200.0, 100.0, 0.0, 180.0, 210.0, 30.0, ComboboxScrollAlignment::Center),
        Some(45.0)
    );
}

#[test]
fn combobox_state_builders_preserve_defaults_and_overrides() {
    let state = ComboboxState::new()
        .with_mode(ComboboxMode::Multiple)
        .with_disabled(true)
        .with_required(true)
        .with_loop_focus(true)
        .with_scroll_alignment(ComboboxScrollAlignment::Center)
        .with_allow_deselect(false)
        .with_clear_on_deselect(true)
        .with_name(Some("tags".into()))
        .with_form(Some("my-form".into()));

    assert!(state.mode().is_multiple());
    assert!(state.is_disabled());
    assert!(state.is_required());
    assert!(state.loop_focus());
    assert_eq!(state.scroll_alignment(), ComboboxScrollAlignment::Center);
    assert!(!state.allow_deselect());
    assert!(state.clear_on_deselect());
    assert_eq!(state.name(), Some("tags"));
    assert_eq!(state.form(), Some("my-form"));
}

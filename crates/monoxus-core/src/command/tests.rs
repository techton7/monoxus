use super::*;
use crate::foundation::{compose::Slottable, shared::ScopeHandle};

#[test]
fn score_exact_and_incomplete_and_non_match() {
    assert_eq!(compute_command_score("Calculator", "Calculator", &[]), 1.0);
    assert_eq!(compute_command_score("html", "html", &[]), 1.0);
    assert_eq!(compute_command_score("html5", "html", &[]), PENALTY_NOT_COMPLETE);
    assert_eq!(compute_command_score("Calculator", "xyz", &[]), 0.0);
    assert_eq!(compute_command_score("Calculator", "", &[]), PENALTY_NOT_COMPLETE);
    assert_eq!(compute_command_score("", "", &[]), 1.0);
}

#[test]
fn score_case_mismatch_and_word_jumps_and_keywords() {
    let exact_case = compute_command_score("HTML", "HM", &[]);
    let lower_case = compute_command_score("haml", "HM", &[]);
    assert!(exact_case > lower_case);

    // Space word jump scores higher than non-space gap jump, which scores higher than char jump
    let space_jump = compute_command_score("new file", "nf", &[]);
    let gap_jump = compute_command_score("new/file", "nf", &[]);
    let char_jump = compute_command_score("newfile", "nf", &[]);
    assert!(space_jump > gap_jump);
    assert!(gap_jump > char_jump);

    // Transposition
    let transposed = compute_command_score("ouch", "uc", &[]);
    assert!(transposed > 0.0);

    // Keywords concatenation
    let without_kw = compute_command_score("Calculator", "math", &[]);
    let with_kw = compute_command_score("Calculator", "math", &["math".to_string(), "add".to_string()]);
    assert_eq!(without_kw, 0.0);
    assert!(with_kw > 0.0);
}

#[test]
fn registry_filtering_ranking_group_hiding_and_force_mount() {
    let mut reg = CommandRegistry::new();
    reg.register_group("suggestions".into(), "g-sugg".into(), Some("h-sugg".into()), false);
    reg.register_group("settings".into(), "g-sett".into(), Some("h-sett".into()), false);

    reg.register_item("Calendar".into(), "i-cal".into(), Some("suggestions".into()), false, false, vec![]);
    reg.register_item("Calculator".into(), "i-calc".into(), Some("suggestions".into()), false, false, vec!["math".into()]);
    reg.register_item("Profile".into(), "i-prof".into(), Some("settings".into()), false, false, vec![]);
    reg.register_item("Pinned".into(), "i-pin".into(), Some("settings".into()), false, true, vec![]);

    // Empty search: all items visible, force_mount excluded from filtered.count
    reg.recompute_filtered_default("", true);
    assert_eq!(reg.filtered().count, 3);
    assert!(reg.is_group_visible("suggestions"));
    assert!(reg.is_group_visible("settings"));

    // Search "calc": Calculator > Calendar (or only Calculator matches)
    reg.recompute_filtered_default("calc", true);
    assert_eq!(reg.filtered().count, 1);
    assert!(reg.is_group_visible("suggestions"));
    // Settings has a force_mount item so it remains visible for its pinned item
    assert!(reg.is_group_visible("settings"));

    let ranked = reg.ranked_visible_items("calc", true);
    assert_eq!(ranked[0].value, "Calculator");
    assert!(ranked.iter().any(|it| it.value == "Pinned"));

    // Unregister Pinned -> settings group hides when searching "calc"
    assert!(reg.unregister_item("Pinned"));
    reg.recompute_filtered_default("calc", true);
    assert!(!reg.is_group_visible("settings"));
    assert!(!should_render_group(false, true, "calc", reg.is_group_visible("settings")));
    assert!(should_render_group(true, true, "calc", false));

    // Empty & separator visibility
    assert!(!should_render_empty(0, false, false));
    assert!(should_render_empty(0, true, false));
    assert!(!should_render_empty(1, true, false));
    assert!(should_render_empty(1, false, true));

    assert!(should_render_separator("", false));
    assert!(!should_render_separator("a", false));
    assert!(should_render_separator("a", true));
}

#[test]
fn navigation_1d_list_vim_group_jump_loop_and_disabled_skip() {
    let mut reg = CommandRegistry::new();
    reg.register_group("g1".into(), "g1".into(), Some("h1".into()), false);
    reg.register_group("g2".into(), "g2".into(), Some("h2".into()), false);

    reg.register_item("a".into(), "ia".into(), Some("g1".into()), false, false, vec![]);
    reg.register_item("b".into(), "ib".into(), Some("g1".into()), true, false, vec![]);
    reg.register_item("c".into(), "ic".into(), Some("g1".into()), false, false, vec![]);
    reg.register_item("d".into(), "id".into(), Some("g2".into()), false, false, vec![]);
    reg.recompute_filtered_default("", true);

    let none_mods = CommandNavModifiers::default();

    // Skips disabled "b"
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "a", CommandNavKey::ArrowDown, none_mods, None, false, true).as_deref(),
        Some("c")
    );
    // Home / End
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "c", CommandNavKey::Home, none_mods, None, false, true).as_deref(),
        Some("a")
    );
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "a", CommandNavKey::End, none_mods, None, false, true).as_deref(),
        Some("d")
    );
    // No loop at end
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "d", CommandNavKey::ArrowDown, none_mods, None, false, true),
        None
    );
    // Loop at end
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "d", CommandNavKey::ArrowDown, none_mods, None, true, true).as_deref(),
        Some("a")
    );
    // Vim bindings (requires ctrl + vim_bindings=true)
    let ctrl = CommandNavModifiers::new(false, false, true);
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "a", CommandNavKey::VimNext, ctrl, None, false, true).as_deref(),
        Some("c")
    );
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "a", CommandNavKey::VimNext, ctrl, None, false, false),
        None
    );
    // Alt + ArrowDown jumps to next group ("g2" -> "d")
    let alt = CommandNavModifiers::new(false, true, false);
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "a", CommandNavKey::ArrowDown, alt, None, false, true).as_deref(),
        Some("d")
    );
    // Meta + ArrowDown jumps to last item
    let meta = CommandNavModifiers::new(true, false, false);
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "a", CommandNavKey::ArrowDown, meta, None, false, true).as_deref(),
        Some("d")
    );
    // Left/Right ignored in 1D mode
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "a", CommandNavKey::ArrowRight, none_mods, None, false, true),
        None
    );
}

#[test]
fn navigation_2d_grid_columns() {
    let mut reg = CommandRegistry::new();
    for i in 1..=5 {
        reg.register_item(format!("i{i}"), format!("id-{i}"), None, false, false, vec![]);
    }
    reg.recompute_filtered_default("", true);
    let none_mods = CommandNavModifiers::default();

    // Grid with 2 columns:
    // Row 0: i1, i2
    // Row 1: i3, i4
    // Row 2: i5
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "i1", CommandNavKey::ArrowRight, none_mods, Some(2), false, true).as_deref(),
        Some("i2")
    );
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "i1", CommandNavKey::ArrowDown, none_mods, Some(2), false, true).as_deref(),
        Some("i3")
    );
    // Down from i4 (col 1) into shorter row 2 -> clamps to i5
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "i4", CommandNavKey::ArrowDown, none_mods, Some(2), false, true).as_deref(),
        Some("i5")
    );
    // Up from i3 -> i1
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "i3", CommandNavKey::ArrowUp, none_mods, Some(2), false, true).as_deref(),
        Some("i1")
    );
    // Loop from last row down -> row 0
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "i5", CommandNavKey::ArrowDown, none_mods, Some(2), true, true).as_deref(),
        Some("i1")
    );
}

#[test]
fn attrs_and_relationships_contract() {
    let scope = ScopeHandle::root("command").child("test");
    let rels = CommandRelationships::new(scope);
    assert!(rels.input_id().contains("input"));
    assert!(rels.viewport_id().contains("viewport"));

    let unselected = CommandItemAttrs::new("id-1", "Calendar", Some("g1".into()), false, false);
    assert_eq!(unselected.role, "option");
    assert_eq!(unselected.aria_disabled, "false");
    assert_eq!(unselected.aria_selected, "false");
    assert_eq!(unselected.data_disabled, None);
    assert_eq!(unselected.data_selected, None);

    let selected_disabled = CommandItemAttrs::new("id-2", "Billing", None, true, true);
    assert_eq!(selected_disabled.aria_disabled, "true");
    assert_eq!(selected_disabled.aria_selected, "true");
    assert_eq!(selected_disabled.data_disabled, Some(""));
    assert_eq!(selected_disabled.data_selected, Some(""));

    let loading = CommandLoadingAttrs::new("l1", 150.0, None);
    assert_eq!(loading.aria_valuenow, 100.0);
    assert_eq!(loading.aria_label, "Loading...");

    let group_hidden = CommandGroupAttrs::new("g1", "Settings", false);
    assert!(group_hidden.hidden);

    let (_, proj, _) = project_command_part_as_child(&unselected, "el", Slottable::new("body"));
    assert_eq!(proj, unselected);
}

#[test]
fn visual_order_matches_ranked_items_and_restores_on_clear() {
    let mut reg = CommandRegistry::new();
    reg.register_group("g-intro".into(), "g1".into(), Some("h1".into()), false);
    reg.register_group("g-delegate".into(), "g2".into(), Some("h2".into()), false);

    // Register an ungrouped item plus two groups where the second group has a stronger match for "d"
    reg.register_item("dashboard".into(), "i-dash".into(), None, false, false, vec![]);
    reg.register_item("Introduction".into(), "i-intro".into(), Some("g-intro".into()), false, false, vec![]);
    reg.register_item("Documentation".into(), "i-doc".into(), Some("g-intro".into()), false, false, vec![]);
    reg.register_item("Delegation".into(), "i-del".into(), Some("g-delegate".into()), false, false, vec![]);

    // Search "d": ungrouped "dashboard" is order 0; inside g-intro, "Documentation" outranks "Introduction"
    reg.recompute_filtered_default("d", true);
    let ranked: Vec<&str> = reg
        .ranked_visible_items("d", true)
        .into_iter()
        .map(|it| it.value.as_str())
        .collect();
    assert_eq!(ranked[0], "dashboard");
    assert_eq!(reg.item_visual_order("dashboard", "d", true), Some(0));

    let doc_ord = reg.item_visual_order("Documentation", "d", true).unwrap();
    let intro_ord = reg.item_visual_order("Introduction", "d", true).unwrap();
    assert!(doc_ord < intro_ord, "Documentation should precede Introduction visually");

    // Groups are offset after the 1 visible ungrouped item
    let g1_ord = reg.group_visual_order("g-intro", "d", true).unwrap();
    let g2_ord = reg.group_visual_order("g-delegate", "d", true).unwrap();
    assert!(g1_ord >= 1 && g2_ord >= 1);
    assert_ne!(g1_ord, g2_ord);

    // Search "del": only "Delegation" matches; "g-intro" and "dashboard" are hidden -> None
    reg.recompute_filtered_default("del", true);
    assert_eq!(reg.item_visual_order("dashboard", "del", true), None);
    assert_eq!(reg.group_visual_order("g-intro", "del", true), None);
    assert_eq!(reg.group_visual_order("g-delegate", "del", true), Some(0));
    assert_eq!(reg.item_visual_order("Delegation", "del", true), Some(0));

    // Clear search: registration order restored
    reg.recompute_filtered_default("", true);
    assert_eq!(reg.item_visual_order("Introduction", "", true), Some(0));
    assert_eq!(reg.item_visual_order("Documentation", "", true), Some(1));
    assert_eq!(reg.group_visual_order("g-intro", "", true), Some(1));
    assert_eq!(reg.group_visual_order("g-delegate", "", true), Some(2));
}

#[test]
fn style_merge_helpers_handle_grid_order_and_hidden_immunity() {
    assert_eq!(
        merge_flex_column_style(None, false).as_deref(),
        Some("display: flex; flex-direction: column;")
    );
    assert_eq!(
        merge_flex_column_style(Some("gap: 4px;"), false).as_deref(),
        Some("display: flex; flex-direction: column; gap: 4px;")
    );
    assert_eq!(
        merge_flex_column_style(Some("display: grid; grid-template-columns: repeat(3, 1fr);"), true).as_deref(),
        Some("display: grid; grid-template-columns: repeat(3, 1fr);")
    );
    assert_eq!(
        merge_ranked_item_style(false, Some(0), Some("display: flex;")).as_deref(),
        Some("display: none;")
    );
    assert_eq!(
        merge_ranked_item_style(true, Some(2), Some("color: red")).as_deref(),
        Some("color: red; order: 2;")
    );
    assert_eq!(
        merge_ranked_item_style(true, None, Some("color: red")).as_deref(),
        Some("color: red")
    );
}

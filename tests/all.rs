mod common;

#[cfg(feature = "native")]
#[path = "fixtures/scenarios/accordion.rs"]
pub(crate) mod playground_accordion;
#[cfg(feature = "native")]
#[path = "fixtures/scenarios/alert_dialog.rs"]
pub(crate) mod playground_alert_dialog;
#[cfg(feature = "native")]
#[path = "fixtures/scenarios/collapsible.rs"]
pub(crate) mod playground_collapsible;
#[cfg(feature = "native")]
#[path = "fixtures/scenarios/dialog.rs"]
pub(crate) mod playground_dialog;
#[cfg(feature = "native")]
#[path = "fixtures/scenarios/popover.rs"]
pub(crate) mod playground_popover;
#[cfg(feature = "native")]
#[allow(unused_imports, unused)]
#[path = "fixtures/scenarios/select/mod.rs"]
pub(crate) mod playground_select;
#[cfg(feature = "native")]
#[path = "fixtures/scenarios/tabs.rs"]
pub(crate) mod playground_tabs;
#[cfg(feature = "native")]
#[path = "fixtures/scenarios/toast_page.rs"]
pub(crate) mod playground_toast;
#[cfg(feature = "native")]
#[path = "fixtures/scenarios/tooltip.rs"]
pub(crate) mod playground_tooltip;

#[cfg(feature = "native")]
mod playground_fixtures {
    pub(crate) use super::{
        playground_accordion, playground_alert_dialog, playground_collapsible, playground_dialog,
        playground_popover, playground_select, playground_tabs, playground_toast,
        playground_tooltip,
    };
}

mod accordion;
mod checkbox;
mod dialog;
mod positioned_overlay;
mod radio_group;
mod select;
mod tabs;
mod toggle_group;

mod accessible_icon_test;
mod aspect_ratio_test;
mod auto_id_test;
mod avatar_test;
mod button_test;
mod calendar_test;
mod collapsible_test;
mod combobox_test;
mod command_test;
mod context_menu_test;
mod date_field_test;
mod date_picker_test;
mod date_range_field_test;
mod date_range_picker_test;
mod disclosure_dimensions_test;
mod dropdown_menu_test;
mod form_hidden_input_test;
mod foundation_phase_2_4_handoff;
mod foundation_phase_3_7_handoff;
mod hover_card_test;
mod label_test;
mod menubar_test;
mod meter_test;
mod navigation_menu_test;
mod overlay_nesting_test;
mod overlay_presence_phase_3_7_step_4;
mod pagination_test;
mod pin_input_test;
mod popover_test;
mod portal_parity_test;
mod progress_test;
mod range_calendar_test;
mod rating_group_test;
mod roving_focus_test;
mod scroll_area_test;
mod separator_test;
mod slider_test;
mod switch_test;
mod time_field_test;
mod time_range_field_test;
mod toast_test;
mod toggle_test;
mod toolbar_test;
mod tooltip_test;
mod visually_hidden_test;
mod z_index_hoisting_test;

#[cfg(feature = "native")]
mod native_accordion_test;
#[cfg(feature = "native")]
mod native_data_gesture_test;
#[cfg(feature = "native")]
mod native_harness_test;
#[cfg(feature = "native")]
mod native_non_floating_test;
#[cfg(feature = "native")]
mod native_overlay_test;
#[cfg(feature = "native")]
mod native_pin_input_test;
#[cfg(feature = "native")]
mod native_presence_exit_test;
#[cfg(feature = "native")]
mod native_select_test;

#[test]
fn all_test_files_are_included() {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    const ROOT_TESTS: &[&str] = &[
        "accessible_icon_test.rs",
        "aspect_ratio_test.rs",
        "auto_id_test.rs",
        "avatar_test.rs",
        "button_test.rs",
        "calendar_test.rs",
        "collapsible_test.rs",
        "combobox_test.rs",
        "command_test.rs",
        "context_menu_test.rs",
        "date_field_test.rs",
        "date_picker_test.rs",
        "date_range_field_test.rs",
        "date_range_picker_test.rs",
        "disclosure_dimensions_test.rs",
        "dropdown_menu_test.rs",
        "form_hidden_input_test.rs",
        "foundation_phase_2_4_handoff.rs",
        "foundation_phase_3_7_handoff.rs",
        "hover_card_test.rs",
        "label_test.rs",
        "menubar_test.rs",
        "meter_test.rs",
        "navigation_menu_test.rs",
        "native_accordion_test.rs",
        "native_data_gesture_test.rs",
        "native_harness_test.rs",
        "native_non_floating_test.rs",
        "native_overlay_test.rs",
        "native_pin_input_test.rs",
        "native_presence_exit_test.rs",
        "native_select_test.rs",
        "overlay_nesting_test.rs",
        "overlay_presence_phase_3_7_step_4.rs",
        "pagination_test.rs",
        "pin_input_test.rs",
        "popover_test.rs",
        "portal_parity_test.rs",
        "progress_test.rs",
        "range_calendar_test.rs",
        "rating_group_test.rs",
        "roving_focus_test.rs",
        "scroll_area_test.rs",
        "separator_test.rs",
        "slider_test.rs",
        "switch_test.rs",
        "time_field_test.rs",
        "time_range_field_test.rs",
        "toast_test.rs",
        "toggle_test.rs",
        "toolbar_test.rs",
        "tooltip_test.rs",
        "visually_hidden_test.rs",
        "z_index_hoisting_test.rs",
    ];
    const SUITE_ROOTS: &[&str] = &[
        "accordion/mod.rs",
        "checkbox/mod.rs",
        "dialog/mod.rs",
        "positioned_overlay/mod.rs",
        "radio_group/mod.rs",
        "select/mod.rs",
        "tabs/mod.rs",
        "toggle_group/mod.rs",
    ];

    fn collect_mod_roots(dir: &Path, output: &mut BTreeSet<PathBuf>) {
        for entry in fs::read_dir(dir).expect("read test directory") {
            let entry = entry.expect("read test entry");
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                if name == "common" || name == "fixtures" || name == "gui" {
                    continue;
                }
                collect_mod_roots(&path, output);
            } else if entry.file_name() == "mod.rs" {
                output.insert(path);
            }
        }
    }

    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let expected_root: BTreeSet<_> = ROOT_TESTS.iter().map(PathBuf::from).collect();
    let actual_root: BTreeSet<_> = fs::read_dir(&tests_dir)
        .expect("read tests directory")
        .map(|entry| entry.expect("read test entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .map(|path| path.file_name().unwrap().into())
        .filter(|name: &PathBuf| name != Path::new("all.rs"))
        .filter(|name: &PathBuf| name != Path::new("standalone_asset_test.rs"))
        .collect();
    assert_eq!(
        actual_root, expected_root,
        "all top-level integration sources must be included in tests/all.rs"
    );

    let expected_suite_roots: BTreeSet<_> = SUITE_ROOTS.iter().map(PathBuf::from).collect();
    let mut actual_suite_roots = BTreeSet::new();
    collect_mod_roots(&tests_dir, &mut actual_suite_roots);
    let actual_suite_roots: BTreeSet<PathBuf> = actual_suite_roots
        .into_iter()
        .map(|path| path.strip_prefix(&tests_dir).unwrap().to_path_buf())
        .collect();
    assert_eq!(
        actual_suite_roots, expected_suite_roots,
        "every non-common test-suite mod.rs must be routed by tests/all.rs"
    );
}

#[test]
fn all_primitives_have_gui_test_manifests() {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    const PRIMITIVES: &[&str] = &[
        "accessible_icon",
        "accordion",
        "alert_dialog",
        "aspect_ratio",
        "avatar",
        "button",
        "calendar",
        "checkbox",
        "collapsible",
        "combobox",
        "command",
        "context_menu",
        "date_field",
        "date_picker",
        "date_range_field",
        "date_range_picker",
        "dialog",
        "dropdown_menu",
        "hover_card",
        "label",
        "menubar",
        "meter",
        "navigation_menu",
        "pagination",
        "pin_input",
        "popover",
        "progress",
        "radio_group",
        "range_calendar",
        "rating_group",
        "scroll_area",
        "select",
        "separator",
        "slider",
        "switch",
        "tabs",
        "time_field",
        "time_range_field",
        "toast",
        "toggle",
        "toggle_group",
        "toolbar",
        "tooltip",
        "visually_hidden",
    ];

    let expected_primitives: BTreeSet<_> = PRIMITIVES.iter().map(PathBuf::from).collect();
    assert_eq!(
        expected_primitives.len(),
        44,
        "the primitive coverage guard must enumerate all 44 standalone primitives"
    );

    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let actual_primitives: BTreeSet<_> = fs::read_dir(&src_dir)
        .expect("read source directory")
        .map(|entry| entry.expect("read source entry"))
        .filter_map(|entry| {
            let path = entry.path();
            let name: PathBuf = entry.file_name().into();
            if path.is_dir() {
                if name == Path::new("foundation") || name == Path::new("menu") {
                    None
                } else {
                    Some(name)
                }
            } else if path.is_file()
                && path.extension().is_some_and(|extension| extension == "rs")
                && name != Path::new("lib.rs")
            {
                Some(path.file_stem().unwrap().into())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        actual_primitives, expected_primitives,
        "the GUI manifest registry must exactly cover all primitive modules"
    );

    let gui_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("gui");
    if !gui_dir.is_dir() {
        return;
    }
    let mut manifest_primitives = BTreeSet::new();
    let mut unregistered_non_manifest_dirs = BTreeSet::new();
    for entry in fs::read_dir(&gui_dir).expect("read GUI test directory") {
        let entry = entry.expect("read GUI test entry");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name: PathBuf = entry.file_name().into();
        if path.join("manifest.toml").is_file() {
            manifest_primitives.insert(name);
        } else if !expected_primitives.contains(&name) {
            unregistered_non_manifest_dirs.insert(name);
        }
    }
    assert_eq!(
        manifest_primitives, expected_primitives,
        "every primitive must have exactly one direct GUI test manifest directory"
    );
    assert!(
        unregistered_non_manifest_dirs.is_empty(),
        "every direct tests/gui subdirectory must be registered: {unregistered_non_manifest_dirs:?}"
    );
}

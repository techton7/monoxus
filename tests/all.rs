#[cfg(feature = "native")]
mod common;

#[cfg(feature = "native")]
#[path = "../examples/playground/accordion.rs"]
pub(crate) mod playground_accordion;
#[cfg(feature = "native")]
#[path = "../examples/playground/alert_dialog.rs"]
pub(crate) mod playground_alert_dialog;
#[cfg(feature = "native")]
#[path = "../examples/playground/collapsible.rs"]
pub(crate) mod playground_collapsible;
#[cfg(feature = "native")]
#[path = "../examples/playground/dialog.rs"]
pub(crate) mod playground_dialog;
#[cfg(feature = "native")]
#[path = "../examples/playground/popover.rs"]
pub(crate) mod playground_popover;
#[cfg(feature = "native")]
#[path = "../examples/playground/scenarios/select/mod.rs"]
pub(crate) mod playground_select;
#[cfg(feature = "native")]
#[path = "../examples/playground/tabs.rs"]
pub(crate) mod playground_tabs;
#[cfg(feature = "native")]
#[path = "../examples/playground/pages/toast_page.rs"]
pub(crate) mod playground_toast;
#[cfg(feature = "native")]
#[path = "../examples/playground/tooltip.rs"]
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
mod web_driver;

mod button_test;
mod collapsible_test;
mod form_hidden_input_test;
mod foundation_phase_2_4_handoff;
mod foundation_phase_3_7_handoff;
mod label_test;
mod overlay_presence_phase_3_7_step_4;
mod popover_test;
mod roving_focus_test;
mod separator_test;
mod switch_test;
mod toast_test;
mod toggle_test;
mod tooltip_test;

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
mod native_presence_exit_test;
#[cfg(feature = "native")]
mod native_select_test;

#[test]
fn all_integration_test_entrypoints_are_accounted_for() {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    const ROOT_TESTS: &[&str] = &[
        "button_test.rs",
        "collapsible_test.rs",
        "form_hidden_input_test.rs",
        "foundation_phase_2_4_handoff.rs",
        "foundation_phase_3_7_handoff.rs",
        "label_test.rs",
        "native_accordion_test.rs",
        "native_data_gesture_test.rs",
        "native_harness_test.rs",
        "native_non_floating_test.rs",
        "native_overlay_test.rs",
        "native_presence_exit_test.rs",
        "native_select_test.rs",
        "overlay_presence_phase_3_7_step_4.rs",
        "popover_test.rs",
        "roving_focus_test.rs",
        "separator_test.rs",
        "switch_test.rs",
        "toast_test.rs",
        "toggle_test.rs",
        "tooltip_test.rs",
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
        "web_driver/mod.rs",
    ];

    fn collect_mod_roots(dir: &Path, output: &mut BTreeSet<PathBuf>) {
        for entry in fs::read_dir(dir).expect("read test directory") {
            let entry = entry.expect("read test entry");
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                if name == "common" || name == "gui" {
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

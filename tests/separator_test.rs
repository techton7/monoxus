use dioxus::prelude::*;
#[allow(unused_imports)]
use monoxus::separator::{
    Separator, SeparatorAttributes, SeparatorOrientation, SeparatorProps, SeparatorRoot,
    project_separator_as_child,
};

#[cfg(feature = "native")]
use crate::common;

#[component]
fn SeparatorAllVariantsScenario() -> Element {
    rsx! {
        div {
            Separator {
                id: "sep-semantic-h",
                orientation: SeparatorOrientation::Horizontal,
                decorative: false,
            }
            Separator {
                id: "sep-semantic-v",
                orientation: SeparatorOrientation::Vertical,
                decorative: false,
            }
            Separator {
                id: "sep-decorative-h",
                orientation: SeparatorOrientation::Horizontal,
                decorative: true,
            }
            Separator {
                id: "sep-decorative-v",
                orientation: SeparatorOrientation::Vertical,
                decorative: true,
            }
        }
    }
}

#[test]
fn test_separator_vdom_renders_cleanly() {
    let mut dom = VirtualDom::new(SeparatorAllVariantsScenario);
    dom.rebuild_in_place();
}

#[test]
fn test_separator_attributes_4_state_matrix() {
    // 1. Semantic Horizontal: role="separator", aria-orientation=None, data-orientation="horizontal"
    let s1 = Separator::new().with_orientation(SeparatorOrientation::Horizontal).with_decorative(false);
    let a1 = s1.attributes();
    assert_eq!(a1.role, "separator");
    assert_eq!(a1.aria_orientation, None);
    assert_eq!(a1.data_orientation, "horizontal");
    assert_eq!(a1.data_separator_root, "");

    // 2. Semantic Vertical: role="separator", aria-orientation=Some("vertical"), data-orientation="vertical"
    let s2 = Separator::new().with_orientation(SeparatorOrientation::Vertical).with_decorative(false);
    let a2 = s2.attributes();
    assert_eq!(a2.role, "separator");
    assert_eq!(a2.aria_orientation, Some("vertical"));
    assert_eq!(a2.data_orientation, "vertical");
    assert_eq!(a2.data_separator_root, "");

    // 3. Decorative Horizontal: role="none", aria-orientation=None, data-orientation="horizontal"
    let s3 = Separator::new().with_orientation(SeparatorOrientation::Horizontal).with_decorative(true);
    let a3 = s3.attributes();
    assert_eq!(a3.role, "none");
    assert_eq!(a3.aria_orientation, None);
    assert_eq!(a3.data_orientation, "horizontal");
    assert_eq!(a3.data_separator_root, "");

    // 4. Decorative Vertical: role="none", aria-orientation=None, data-orientation="vertical"
    let s4 = Separator::new().with_orientation(SeparatorOrientation::Vertical).with_decorative(true);
    let a4 = s4.attributes();
    assert_eq!(a4.role, "none");
    assert_eq!(a4.aria_orientation, None);
    assert_eq!(a4.data_orientation, "vertical");
    assert_eq!(a4.data_separator_root, "");
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use super::common::native_fixture::*;

    #[test]
    fn test_native_separator_matrix_attributes() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let harness = mount_native_scenario(SeparatorAllVariantsScenario);

        // 1. Semantic Horizontal
        let sh_id = harness.query("#sep-semantic-h").expect("Semantic H mounted");
        {
            let base = harness.base();
            let node = base.get_node(sh_id).expect("Node exists");
            assert_eq!(node.attr("role".into()), Some("separator"));
            assert_eq!(node.attr("aria-orientation".into()), None, "Horizontal separator omits aria-orientation");
            assert_eq!(node.attr("data-orientation".into()), Some("horizontal"));
            assert_eq!(node.attr("data-separator-root".into()), Some(""));
        }

        // 2. Semantic Vertical
        let sv_id = harness.query("#sep-semantic-v").expect("Semantic V mounted");
        {
            let base = harness.base();
            let node = base.get_node(sv_id).expect("Node exists");
            assert_eq!(node.attr("role".into()), Some("separator"));
            assert_eq!(node.attr("aria-orientation".into()), Some("vertical"), "Vertical semantic separator emits aria-orientation='vertical'");
            assert_eq!(node.attr("data-orientation".into()), Some("vertical"));
            assert_eq!(node.attr("data-separator-root".into()), Some(""));
        }

        // 3. Decorative Horizontal
        let dh_id = harness.query("#sep-decorative-h").expect("Decorative H mounted");
        {
            let base = harness.base();
            let node = base.get_node(dh_id).expect("Node exists");
            assert_eq!(node.attr("role".into()), Some("none"));
            assert_eq!(node.attr("aria-orientation".into()), None, "Decorative separator omits aria-orientation");
            assert_eq!(node.attr("data-orientation".into()), Some("horizontal"));
            assert_eq!(node.attr("data-separator-root".into()), Some(""));
        }

        // 4. Decorative Vertical
        let dv_id = harness.query("#sep-decorative-v").expect("Decorative V mounted");
        {
            let base = harness.base();
            let node = base.get_node(dv_id).expect("Node exists");
            assert_eq!(node.attr("role".into()), Some("none"));
            assert_eq!(node.attr("aria-orientation".into()), None, "Decorative separator omits aria-orientation even when vertical");
            assert_eq!(node.attr("data-orientation".into()), Some("vertical"));
            assert_eq!(node.attr("data-separator-root".into()), Some(""));
        }
    }
}

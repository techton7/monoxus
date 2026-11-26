use crate::foundation::compose::project_attrs_as_child;
use crate::foundation::{
    compose::Slottable,
    roving_focus::RovingModifiers,
    shared::{CollectionRegistry, Direction, Orientation, ScopeHandle},
};

use super::*;

#[test]
fn part_inventory_and_relationships_are_deterministic() {
    assert_eq!(
        PaginationDefinition::parts()
            .iter()
            .map(PaginationPart::as_str)
            .collect::<Vec<_>>(),
        ["root", "prev-button", "next-button", "page"]
    );

    let scope = ScopeHandle::root("pagination").child("main");
    let rel = PaginationRelationships::new(scope.clone());
    assert_eq!(rel.root_id(), scope.token());
    assert_eq!(rel.prev_id(), scope.qualify("prev"));
    assert_eq!(rel.next_id(), scope.qualify("next"));
    assert_eq!(rel.page_id(4), scope.qualify("page-4"));
}

#[test]
fn compute_total_pages_handles_zero_and_remainder() {
    assert_eq!(compute_total_pages(0, 10), 1);
    assert_eq!(compute_total_pages(0, 0), 1);
    assert_eq!(compute_total_pages(100, 0), 100);
    assert_eq!(compute_total_pages(100, 10), 10);
    assert_eq!(compute_total_pages(101, 10), 11);
    assert_eq!(compute_total_pages(1, 10), 1);
}

#[test]
fn compute_range_handles_empty_middle_and_partial_last_pages() {
    assert_eq!(compute_range(1, 10, 0), PageRange { start: 0, end: 0 });
    assert_eq!(compute_range(1, 10, 100), PageRange { start: 1, end: 10 });
    assert_eq!(compute_range(2, 10, 100), PageRange { start: 11, end: 20 });
    assert_eq!(compute_range(10, 10, 95), PageRange { start: 91, end: 95 });
    assert_eq!(compute_range(99, 10, 95), PageRange { start: 91, end: 95 });
    assert_eq!(compute_range(0, 0, 5), PageRange { start: 1, end: 1 });
}

#[test]
fn compute_page_items_exhaustive_invariants() {
    for total_pages in 1usize..=20 {
        for sibling_count in 0..=2 {
            let sample_pages = [
                1,
                2.min(total_pages),
                (total_pages / 2).max(1),
                total_pages.saturating_sub(1).max(1),
                total_pages,
            ];
            for page in sample_pages {
                let items = compute_page_items(page, total_pages, sibling_count);
                assert!(!items.is_empty());
                assert_eq!(items.first().and_then(PageItem::page_value), Some(1));
                assert_eq!(
                    items.last().and_then(PageItem::page_value),
                    Some(total_pages)
                );

                let mut prev_was_ellipsis = false;
                let mut seen_pages = Vec::new();
                for item in &items {
                    match item {
                        PageItem::Ellipsis { .. } => {
                            assert!(
                                !prev_was_ellipsis,
                                "contiguous ellipsis for total={total_pages}, sibling={sibling_count}, page={page}: {items:?}"
                            );
                            prev_was_ellipsis = true;
                        }
                        PageItem::Page { value, key } => {
                            assert_eq!(key, &format!("page-{value}"));
                            assert!(!seen_pages.contains(value));
                            if let Some(last) = seen_pages.last() {
                                assert!(value > last);
                            }
                            seen_pages.push(*value);
                            prev_was_ellipsis = false;
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn compute_page_items_matches_expected_windows_and_deterministic_keys() {
    assert_eq!(
        compute_page_items(1, 10, 1),
        vec![
            PageItem::Page {
                value: 1,
                key: "page-1".into()
            },
            PageItem::Page {
                value: 2,
                key: "page-2".into()
            },
            PageItem::Page {
                value: 3,
                key: "page-3".into()
            },
            PageItem::Page {
                value: 4,
                key: "page-4".into()
            },
            PageItem::Ellipsis {
                key: "ellipsis-0".into()
            },
            PageItem::Page {
                value: 10,
                key: "page-10".into()
            },
        ]
    );

    assert_eq!(
        compute_page_items(5, 10, 1),
        vec![
            PageItem::Page {
                value: 1,
                key: "page-1".into()
            },
            PageItem::Ellipsis {
                key: "ellipsis-0".into()
            },
            PageItem::Page {
                value: 4,
                key: "page-4".into()
            },
            PageItem::Page {
                value: 5,
                key: "page-5".into()
            },
            PageItem::Page {
                value: 6,
                key: "page-6".into()
            },
            PageItem::Ellipsis {
                key: "ellipsis-1".into()
            },
            PageItem::Page {
                value: 10,
                key: "page-10".into()
            },
        ]
    );

    assert_eq!(
        compute_page_items(10, 10, 1),
        vec![
            PageItem::Page {
                value: 1,
                key: "page-1".into()
            },
            PageItem::Ellipsis {
                key: "ellipsis-0".into()
            },
            PageItem::Page {
                value: 7,
                key: "page-7".into()
            },
            PageItem::Page {
                value: 8,
                key: "page-8".into()
            },
            PageItem::Page {
                value: 9,
                key: "page-9".into()
            },
            PageItem::Page {
                value: 10,
                key: "page-10".into()
            },
        ]
    );
}

#[test]
fn attributes_publish_aria_current_and_omit_false_data_flags() {
    let scope = ScopeHandle::root("pagination").child("attrs");
    let def = PaginationDefinition::new(scope, 100)
        .with_per_page(10)
        .with_page(1);

    let root = def.root_attributes(None);
    assert_eq!(root.role(), "navigation");
    assert_eq!(root.aria_label(), "pagination");
    assert_eq!(root.dir(), "ltr");
    assert_eq!(root.data_orientation(), "horizontal");
    assert_eq!(root.data_monoxus_pagination_root(), "");

    let active_page = def.page_attributes(1, None, false);
    assert_eq!(active_page.aria_label(), "Page 1");
    assert_eq!(active_page.aria_current(), Some("page"));
    assert_eq!(active_page.data_selected(), Some(""));
    assert_eq!(active_page.data_disabled(), None);
    assert_eq!(active_page.aria_disabled(), None);
    assert!(!active_page.is_disabled());

    let inactive_page = def.page_attributes(2, None, false);
    assert_eq!(inactive_page.aria_current(), None);
    assert_eq!(inactive_page.data_selected(), None);
    assert_eq!(inactive_page.data_disabled(), None);

    let disabled_page = def.page_attributes(3, None, true);
    assert_eq!(disabled_page.aria_disabled(), Some("true"));
    assert_eq!(disabled_page.data_disabled(), Some(""));
    assert!(disabled_page.is_disabled());

    let prev_at_start = def.prev_button_attributes(None, false);
    assert!(prev_at_start.is_disabled());
    assert_eq!(prev_at_start.aria_disabled(), Some("true"));
    assert_eq!(prev_at_start.data_disabled(), Some(""));

    let next_at_start = def.next_button_attributes(None, false);
    assert!(!next_at_start.is_disabled());
    assert_eq!(next_at_start.aria_disabled(), None);
    assert_eq!(next_at_start.data_disabled(), None);

    let (target, projected_root, content) =
        project_attrs_as_child(&root, "custom-nav", Slottable::new("children"));
    assert_eq!(target, "custom-nav");
    assert_eq!(projected_root, root);
    assert_eq!(content, "children");
}

#[test]
fn directional_navigation_respects_rtl_orientation_loop_and_disabled_buttons() {
    let scope = ScopeHandle::root("pagination").child("nav");
    let def = PaginationDefinition::new(scope, 30)
        .with_per_page(10)
        .with_direction(Direction::Rtl)
        .with_orientation(Orientation::Horizontal)
        .with_loop_focus(true);

    let mut reg = CollectionRegistry::new();
    reg.register(
        PREV_ROVING_KEY.to_string(),
        PaginationItemRegistration {
            key: PREV_ROVING_KEY.to_string(),
            id: "prev".to_string(),
            disabled: true,
        },
    );
    for p in 1..=3 {
        let key = page_roving_key(p);
        reg.register(
            key.clone(),
            PaginationItemRegistration {
                key: key.clone(),
                id: key,
                disabled: false,
            },
        );
    }
    reg.register(
        NEXT_ROVING_KEY.to_string(),
        PaginationItemRegistration {
            key: NEXT_ROVING_KEY.to_string(),
            id: "next".to_string(),
            disabled: false,
        },
    );

    assert_eq!(
        def.navigate_key(
            &reg,
            Some("page-1"),
            "ArrowLeft",
            RovingModifiers::none()
        ),
        Some("page-2".to_string())
    );
    assert_eq!(
        def.navigate_key(
            &reg,
            Some("page-1"),
            "ArrowRight",
            RovingModifiers::none()
        ),
        Some("next".to_string())
    );
    assert_eq!(
        def.navigate_key(&reg, Some("next"), "Home", RovingModifiers::none()),
        Some("page-1".to_string())
    );
    assert_eq!(
        def.navigate_key(&reg, Some("page-1"), "End", RovingModifiers::none()),
        Some("next".to_string())
    );
}

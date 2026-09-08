use monoxus::{
    popover::{Popover, PopoverPart},
    tooltip::{Tooltip, TooltipPart},
};

#[test]
fn positioned_overlay_family_boundary_matches_the_phase_3_2_inventory() {
    let popover_parts: Vec<_> = Popover::parts().iter().map(PopoverPart::as_str).collect();
    let tooltip_parts: Vec<_> = Tooltip::parts().iter().map(TooltipPart::as_str).collect();

    assert_eq!(
        popover_parts,
        vec![
            "root", "trigger", "portal", "content", "arrow", "anchor", "close"
        ],
    );
    assert_eq!(
        tooltip_parts,
        vec!["root", "trigger", "portal", "content", "arrow", "provider"],
    );

    for omitted in [
        "overlay",
        "title",
        "description",
        "header",
        "footer",
        "hover-card",
        "context-menu",
        "dropdown-menu",
        "select",
        "combobox",
    ] {
        assert!(!popover_parts.contains(&omitted));
        assert!(!tooltip_parts.contains(&omitted));
    }
}

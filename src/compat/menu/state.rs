// [KEEP-MONOXUS-BRAND] Re-exports `ui_sdk::menu::state` with Monoxus/Radix `menu_css_vars` compat shim.
use ui_sdk::foundation::overlay::GeometryVars;

pub use ui_sdk::menu::state::{
    MenuContentState, SubmenuIntentMachine, TypeaheadBuffer, checkbox_group_item_state,
    first_enabled_item_id, last_enabled_item_id, match_next_typeahead_item,
    menu_css_vars_with_prefix, resolve_menu_navigation, resolve_menu_navigation_with_options,
    toggle_checkbox_group_value,
};

use super::types::MenuVariant;

const MONOXUS_CSS_PREFIX: &str = "monoxus";

const fn radix_compatibility_prefix(variant: MenuVariant) -> &'static str {
    match variant {
        MenuVariant::Menu => "radix-menu",
        MenuVariant::DropdownMenu => "radix-dropdown-menu",
        MenuVariant::ContextMenu => "radix-context-menu",
        MenuVariant::Menubar => "radix-menubar",
    }
}

/// Emit canonical `--monoxus-<variant>-*` and `--radix-<variant>-*` CSS variables without duplicates.
pub fn menu_css_vars(geometry: &GeometryVars, variant: MenuVariant) -> Vec<(String, String)> {
    let mut vars = menu_css_vars_with_prefix(geometry, variant, MONOXUS_CSS_PREFIX);

    for (name, value) in
        geometry.compatibility_alias_iter(radix_compatibility_prefix(variant), "trigger")
    {
        if !vars.iter().any(|(existing, _)| existing == &name) {
            vars.push((name, value));
        }
    }

    vars
}

#[cfg(test)]
mod tests {
    use ui_sdk::foundation::overlay::{FloatingLayer, PlacementSide, Rect, Size};

    use super::*;

    #[test]
    fn menu_css_vars_emits_monoxus_and_radix_aliases_without_duplicates() {
        for (variant, expected_radix) in [
            (MenuVariant::Menu, "radix-menu"),
            (MenuVariant::DropdownMenu, "radix-dropdown-menu"),
            (MenuVariant::ContextMenu, "radix-context-menu"),
            (MenuVariant::Menubar, "radix-menubar"),
        ] {
            let layer =
                FloatingLayer::new(PlacementSide::Bottom).with_namespace(variant.as_str());
            let geometry =
                layer.geometry_vars(Rect::new(10.0, 20.0, 80.0, 30.0), Size::new(160.0, 120.0));
            let vars = menu_css_vars(&geometry, variant);

            let monoxus_prefix = format!("--monoxus-{}", variant.as_str());
            let radix_prefix = format!("--{expected_radix}");

            assert!(vars
                .iter()
                .any(|(k, _)| k == &format!("{monoxus_prefix}-content-transform-origin")));
            assert!(vars
                .iter()
                .any(|(k, _)| k == &format!("{monoxus_prefix}-content-available-width")));
            assert!(vars
                .iter()
                .any(|(k, _)| k == &format!("{monoxus_prefix}-anchor-width")));
            assert!(vars
                .iter()
                .any(|(k, _)| k == &format!("{radix_prefix}-content-transform-origin")));
            assert!(vars
                .iter()
                .any(|(k, _)| k == &format!("{radix_prefix}-trigger-width")));

            let unique_count = {
                let mut keys: Vec<&str> = vars.iter().map(|(k, _)| k.as_str()).collect();
                keys.sort_unstable();
                keys.dedup();
                keys.len()
            };
            assert_eq!(unique_count, vars.len(), "duplicate CSS var emitted");
        }
    }
}

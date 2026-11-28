// [KEEP-MONOXUS-BRAND]
pub use ui_sdk::command::attrs::{
    CommandEmptyAttrs, CommandGroupAttrs, CommandGroupHeadingAttrs, CommandGroupItemsAttrs,
    CommandInputAttrs, CommandInputLabelAttrs, CommandItemAttrs, CommandLoadingAttrs,
    CommandRootAttrs, CommandSeparatorAttrs, CommandViewportAttrs, bool_to_empty_attr,
    merge_flex_column_style, merge_ranked_item_style,
};

// [KEEP-MONOXUS-BRAND] Preserve dual `--monoxus-command-list-height` / `--bits-command-list-height` CSS variables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandListAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_label: String,
    pub data_command_list: &'static str,
}

impl CommandListAttrs {
    pub fn new(id: impl Into<String>, aria_label: Option<&str>) -> Self {
        Self::from(ui_sdk::command::attrs::CommandListAttrs::new(id, aria_label))
    }

    pub fn format_height_style(height_px: Option<f64>, extra_style: Option<&str>) -> String {
        let mut style = String::new();
        if let Some(h) = height_px {
            style.push_str(&format!(
                "--monoxus-command-list-height: {h}px; --bits-command-list-height: {h}px;"
            ));
        }
        if let Some(extra) = extra_style {
            let trimmed = extra.trim();
            if !trimmed.is_empty() {
                if !style.is_empty() {
                    style.push(' ');
                }
                style.push_str(trimmed);
            }
        }
        style
    }

    pub fn format_height_style_with_prefix(
        prefix: &str,
        height_px: Option<f64>,
        extra_style: Option<&str>,
    ) -> String {
        ui_sdk::command::attrs::CommandListAttrs::format_height_style_with_prefix(
            prefix,
            height_px,
            extra_style,
        )
    }
}

impl From<ui_sdk::command::attrs::CommandListAttrs> for CommandListAttrs {
    fn from(attrs: ui_sdk::command::attrs::CommandListAttrs) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            aria_label: attrs.aria_label,
            data_command_list: attrs.data_command_list,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_list_attrs_preserves_dual_monoxus_and_bits_height_vars() {
        let attrs = CommandListAttrs::new("cmd-list", None);
        assert_eq!(attrs.id, "cmd-list");
        assert_eq!(attrs.role, "listbox");
        assert_eq!(attrs.aria_label, "Suggestions...");
        assert_eq!(attrs.data_command_list, "");

        assert_eq!(
            CommandListAttrs::format_height_style(Some(240.0), None),
            "--monoxus-command-list-height: 240px; --bits-command-list-height: 240px;"
        );
        assert_eq!(
            CommandListAttrs::format_height_style(Some(240.0), Some(" max-height: 300px; ")),
            "--monoxus-command-list-height: 240px; --bits-command-list-height: 240px; max-height: 300px;"
        );
        assert_eq!(
            CommandListAttrs::format_height_style(None, Some("gap: 4px;")),
            "gap: 4px;"
        );
        assert_eq!(CommandListAttrs::format_height_style(None, None), "");
        assert_eq!(
            CommandListAttrs::format_height_style_with_prefix("--ui", Some(120.0), None),
            "--ui-command-list-height: 120px;"
        );
    }
}

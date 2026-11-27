// [KEEP-MONOXUS-BRAND] Re-exports `ui_sdk::toast::types::*` with single type identity
// (so `ToastConfig.class_names` and `ToastClassNames` are identical types) while preserving
// Monoxus's Tailwind default class presets via explicit constructor helpers.

pub use ui_sdk::toast::types::*;

/// Returns the Monoxus Tailwind default slot classes for toast components.
pub fn monoxus_default_toast_class_names() -> ToastClassNames {
    ToastClassNames {
        toast: Some(
            "relative flex flex-col gap-2 rounded-xl border border-zinc-800 bg-zinc-950 p-4 text-zinc-50 shadow-xl"
                .to_string(),
        ),
        title: Some("text-sm font-semibold text-zinc-50".to_string()),
        description: Some("text-sm leading-5 text-zinc-400".to_string()),
        action_button: Some(
            "mt-1 rounded-md bg-zinc-50 px-3 py-1.5 text-xs font-semibold text-zinc-950"
                .to_string(),
        ),
        cancel_button: Some(
            "mt-1 rounded-md border border-zinc-700 px-3 py-1.5 text-xs font-medium text-zinc-300"
                .to_string(),
        ),
        close_button: Some(
            "absolute right-2 top-2 rounded-md px-2 py-1 text-zinc-400 hover:bg-zinc-900 hover:text-zinc-50"
                .to_string(),
        ),
        default: Some("border-zinc-800".to_string()),
        success: Some("border-emerald-800".to_string()),
        info: Some("border-sky-800".to_string()),
        warning: Some("border-amber-800".to_string()),
        error: Some("border-red-800".to_string()),
        loading: Some("border-violet-800".to_string()),
        expanded: None,
        collapsed_behind: None,
    }
}

/// Returns a [`ToastConfig`] initialized with [`monoxus_default_toast_class_names`].
pub fn monoxus_default_toast_config() -> ToastConfig {
    ToastConfig {
        class_names: monoxus_default_toast_class_names(),
        ..ToastConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_class_names_shares_single_type_identity_with_toast_config_and_preserves_monoxus_preset()
    {
        // Single type identity: ToastClassNames can be placed directly in ToastConfig literals.
        let headless_cfg = ToastConfig {
            class_names: ToastClassNames::default(),
            ..ToastConfig::default()
        };
        assert_eq!(headless_cfg.class_names.toast.as_deref(), Some(""));

        let monoxus_cfg = monoxus_default_toast_config();
        assert!(
            monoxus_cfg
                .class_names
                .toast
                .as_deref()
                .unwrap_or_default()
                .contains("bg-zinc-950")
        );
        assert_eq!(
            monoxus_cfg.class_names.success.as_deref(),
            Some("border-emerald-800")
        );
    }
}

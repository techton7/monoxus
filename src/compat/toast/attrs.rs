use super::types::ToastConfig;

pub use ui_sdk::toast::attrs::{ToastActionAttrs, ToastRootAttrs, ToastViewportAttrs};

// [KEEP-MONOXUS-BRAND] Preserve `data_radix_toast_announce_exclude` field for `monoxus::toast`.
/// Pure attribute publisher for close/dismiss buttons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastCloseAttrs {
    pub r#type: &'static str,
    pub aria_label: String,
    pub data_radix_toast_announce_exclude: &'static str,
}

impl ToastCloseAttrs {
    pub fn new(config: &ToastConfig) -> Self {
        Self::from(ui_sdk::toast::attrs::ToastCloseAttrs::new(config))
    }

    #[inline]
    pub fn announce_exclude(&self) -> bool {
        self.data_radix_toast_announce_exclude == "true"
    }
}

impl From<ui_sdk::toast::attrs::ToastCloseAttrs> for ToastCloseAttrs {
    fn from(attrs: ui_sdk::toast::attrs::ToastCloseAttrs) -> Self {
        Self {
            r#type: attrs.r#type,
            aria_label: attrs.aria_label,
            data_radix_toast_announce_exclude: attrs.data_toast_announce_exclude,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_close_attrs_preserves_radix_announce_exclude_field() {
        let config = ToastConfig::default();
        let attrs = ToastCloseAttrs::new(&config);
        assert_eq!(attrs.r#type, "button");
        assert_eq!(attrs.aria_label, config.close_button_aria_label);
        assert_eq!(attrs.data_radix_toast_announce_exclude, "true");
        assert!(attrs.announce_exclude());
    }
}

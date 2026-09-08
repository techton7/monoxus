use super::types::*;

/// Pure attribute publisher for the landmark `ToastViewport` element.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastViewportAttrs {
    pub role: &'static str,
    pub aria_label: String,
    pub tabindex: &'static str,
    pub dir: String,
    pub data_expanded: &'static str,
    pub style: String,
}

impl ToastViewportAttrs {
    pub fn new(config: &ToastConfig, expanded: bool, dir: impl Into<String>) -> Self {
        let mut styles = Vec::new();
        if let Some(ref top) = config.offset.top {
            styles.push(format!("--offset-top: {};", top));
        }
        if let Some(ref right) = config.offset.right {
            styles.push(format!("--offset-right: {};", right));
        }
        if let Some(ref bottom) = config.offset.bottom {
            styles.push(format!("--offset-bottom: {};", bottom));
        }
        if let Some(ref left) = config.offset.left {
            styles.push(format!("--offset-left: {};", left));
        }
        if let Some(ref top) = config.mobile_offset.top {
            styles.push(format!("--mobile-offset-top: {};", top));
        }
        if let Some(ref right) = config.mobile_offset.right {
            styles.push(format!("--mobile-offset-right: {};", right));
        }
        if let Some(ref bottom) = config.mobile_offset.bottom {
            styles.push(format!("--mobile-offset-bottom: {};", bottom));
        }
        if let Some(ref left) = config.mobile_offset.left {
            styles.push(format!("--mobile-offset-left: {};", left));
        }

        Self {
            role: "region",
            aria_label: config.container_aria_label.replace("{hotkey}", &config.hotkey),
            tabindex: "-1",
            dir: dir.into(),
            data_expanded: if expanded { "true" } else { "false" },
            style: styles.join(" "),
        }
    }
}

/// Pure attribute publisher for an individual `ToastRoot` card.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastRootAttrs {
    pub role: &'static str,
    pub aria_live: &'static str,
    pub aria_atomic: &'static str,
    pub data_state: &'static str,
    pub data_type: &'static str,
    pub data_visible: &'static str,
    pub data_front: &'static str,
    pub data_expanded: &'static str,
    pub data_index: String,
    pub data_testid: Option<String>,
}

impl ToastRootAttrs {
    pub fn new(
        toast_type: ToastType,
        phase: ToastPhase,
        index: usize,
        visible_limit: usize,
        test_id: Option<String>,
    ) -> Self {
        Self::with_options(toast_type, phase, index, visible_limit, test_id, false)
    }

    pub fn with_options(
        toast_type: ToastType,
        phase: ToastPhase,
        index: usize,
        visible_limit: usize,
        test_id: Option<String>,
        expanded: bool,
    ) -> Self {
        let (role, aria_live) = match toast_type {
            ToastType::Warning | ToastType::Error => ("alert", "assertive"),
            _ => ("status", "polite"),
        };

        Self {
            role,
            aria_live,
            aria_atomic: "true",
            data_state: phase.as_state_str(),
            data_type: toast_type.as_str(),
            data_visible: if index < visible_limit { "true" } else { "false" },
            data_front: if index == 0 { "true" } else { "false" },
            data_expanded: if expanded { "true" } else { "false" },
            data_index: index.to_string(),
            data_testid: test_id,
        }
    }

    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.data_expanded = if expanded { "true" } else { "false" };
        self
    }
}

/// Pure attribute publisher for action buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastActionAttrs {
    pub r#type: &'static str,
    pub aria_label: String,
}

impl ToastActionAttrs {
    pub fn new(alt_text: String) -> Self {
        Self {
            r#type: "button",
            aria_label: alt_text,
        }
    }
}

/// Pure attribute publisher for close/dismiss buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastCloseAttrs {
    pub r#type: &'static str,
    pub aria_label: String,
    pub data_radix_toast_announce_exclude: &'static str,
}

impl ToastCloseAttrs {
    pub fn new(config: &ToastConfig) -> Self {
        Self {
            r#type: "button",
            aria_label: config.close_button_aria_label.clone(),
            data_radix_toast_announce_exclude: "true",
        }
    }
}

use crate::foundation::state::DataState;

pub use ui_sdk::hover_card::attrs::{HoverCardPortalAttributes, HoverCardRootAttributes};

// [KEEP-MONOXUS-BRAND] Compat attribute wrappers preserving `data_monoxus_hover_card_*` markers.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverCardTriggerAttributes {
    pub id: String,
    pub href: Option<String>,
    pub target: Option<String>,
    pub rel: Option<String>,
    pub disabled: bool,
    pub aria_disabled: Option<&'static str>,
    pub tabindex: Option<i32>,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_monoxus_hover_card_trigger: &'static str,
}

impl HoverCardTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn rel(&self) -> Option<&str> {
        self.rel.as_deref()
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub const fn tabindex(&self) -> Option<i32> {
        self.tabindex
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub const fn data_monoxus_hover_card_trigger(&self) -> &'static str {
        self.data_monoxus_hover_card_trigger
    }

    pub fn from_parts(
        id: String,
        href: Option<String>,
        target: Option<String>,
        rel: Option<String>,
        disabled: bool,
        data_state: DataState,
    ) -> Self {
        Self::from(ui_sdk::hover_card::attrs::HoverCardTriggerAttributes::from_parts(
            id, href, target, rel, disabled, data_state,
        ))
    }
}

impl From<ui_sdk::hover_card::attrs::HoverCardTriggerAttributes> for HoverCardTriggerAttributes {
    fn from(attrs: ui_sdk::hover_card::attrs::HoverCardTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            href: attrs.href,
            target: attrs.target,
            rel: attrs.rel,
            disabled: attrs.disabled,
            aria_disabled: attrs.aria_disabled,
            tabindex: attrs.tabindex,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            data_monoxus_hover_card_trigger: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverCardContentAttributes {
    pub id: String,
    pub wrapper_id: String,
    pub tabindex: i32,
    pub data_state: DataState,
    pub data_side: &'static str,
    pub data_align: &'static str,
    pub data_monoxus_hover_card_content: &'static str,
}

impl HoverCardContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn wrapper_id(&self) -> &str {
        &self.wrapper_id
    }

    pub const fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_side(&self) -> &'static str {
        self.data_side
    }

    pub const fn data_align(&self) -> &'static str {
        self.data_align
    }

    pub const fn data_monoxus_hover_card_content(&self) -> &'static str {
        self.data_monoxus_hover_card_content
    }
}

impl From<ui_sdk::hover_card::attrs::HoverCardContentAttributes> for HoverCardContentAttributes {
    fn from(attrs: ui_sdk::hover_card::attrs::HoverCardContentAttributes) -> Self {
        Self {
            id: attrs.id,
            wrapper_id: attrs.wrapper_id,
            tabindex: attrs.tabindex,
            data_state: attrs.data_state,
            data_side: attrs.data_side,
            data_align: attrs.data_align,
            data_monoxus_hover_card_content: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverCardContentStaticAttributes {
    pub id: String,
    pub tabindex: i32,
    pub data_state: DataState,
    pub data_monoxus_hover_card_content: &'static str,
}

impl HoverCardContentStaticAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_monoxus_hover_card_content(&self) -> &'static str {
        self.data_monoxus_hover_card_content
    }
}

impl From<ui_sdk::hover_card::attrs::HoverCardContentStaticAttributes>
    for HoverCardContentStaticAttributes
{
    fn from(attrs: ui_sdk::hover_card::attrs::HoverCardContentStaticAttributes) -> Self {
        Self {
            id: attrs.id,
            tabindex: attrs.tabindex,
            data_state: attrs.data_state,
            data_monoxus_hover_card_content: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverCardArrowAttributes {
    pub id: String,
    pub data_state: DataState,
    pub data_side: &'static str,
    pub data_align: &'static str,
    pub data_monoxus_hover_card_arrow: &'static str,
}

impl HoverCardArrowAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }

    pub const fn data_side(&self) -> &'static str {
        self.data_side
    }

    pub const fn data_align(&self) -> &'static str {
        self.data_align
    }

    pub const fn data_monoxus_hover_card_arrow(&self) -> &'static str {
        self.data_monoxus_hover_card_arrow
    }
}

impl From<ui_sdk::hover_card::attrs::HoverCardArrowAttributes> for HoverCardArrowAttributes {
    fn from(attrs: ui_sdk::hover_card::attrs::HoverCardArrowAttributes) -> Self {
        Self {
            id: attrs.id,
            data_state: attrs.data_state,
            data_side: attrs.data_side,
            data_align: attrs.data_align,
            data_monoxus_hover_card_arrow: "",
        }
    }
}

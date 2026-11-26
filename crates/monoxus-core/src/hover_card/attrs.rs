use crate::foundation::{
    overlay::PortalHost,
    shared::data_disabled_attr,
    state::DataState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverCardRootAttributes {
    pub id: String,
    pub data_state: DataState,
}

impl HoverCardRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> &DataState {
        &self.data_state
    }
}

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
        Self {
            id,
            href: if disabled { None } else { href },
            target: if disabled { None } else { target },
            rel: if disabled { None } else { rel },
            disabled,
            aria_disabled: if disabled { Some("true") } else { None },
            tabindex: if disabled { Some(-1) } else { None },
            data_state,
            data_disabled: data_disabled_attr(disabled),
            data_monoxus_hover_card_trigger: "",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverCardPortalAttributes {
    pub host: PortalHost,
}

impl HoverCardPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
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


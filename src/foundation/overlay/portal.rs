use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PortalHost {
    Default,
    Inline,
    Named(Cow<'static, str>),
}

impl PortalHost {
    pub fn named(id: impl Into<Cow<'static, str>>) -> Self {
        Self::Named(id.into())
    }

    pub const fn inline() -> Self {
        Self::Inline
    }

    pub const fn default_host() -> Self {
        Self::Default
    }

    pub fn resolve(preferred: Option<Self>, inherited: Option<&Self>) -> Self {
        preferred.or_else(|| inherited.cloned()).unwrap_or_default()
    }

    pub const fn is_inline(&self) -> bool {
        matches!(self, Self::Inline)
    }

    pub const fn is_default_host(&self) -> bool {
        matches!(self, Self::Default)
    }

    pub const fn is_portalled(&self) -> bool {
        !self.is_inline()
    }

    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Default | Self::Inline => None,
            Self::Named(id) => Some(id.as_ref()),
        }
    }
}

impl Default for PortalHost {
    fn default() -> Self {
        Self::Default
    }
}

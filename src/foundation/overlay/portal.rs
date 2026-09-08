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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresenceState {
    Unmounted,
    Mounted,
    Suspended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Presence {
    desired_present: bool,
    retain_mount: bool,
    state: PresenceState,
}

impl Presence {
    pub const fn new(desired_present: bool) -> Self {
        Self {
            desired_present,
            retain_mount: false,
            state: if desired_present {
                PresenceState::Mounted
            } else {
                PresenceState::Unmounted
            },
        }
    }

    pub const fn with_retained_mount(mut self, retain_mount: bool) -> Self {
        self.retain_mount = retain_mount;
        self
    }

    pub const fn desired_present(&self) -> bool {
        self.desired_present
    }

    pub const fn retain_mount(&self) -> bool {
        self.retain_mount
    }

    pub const fn state(&self) -> PresenceState {
        self.state
    }

    pub const fn is_mounted(&self) -> bool {
        !matches!(self.state, PresenceState::Unmounted)
    }

    pub fn sync(&mut self, desired_present: bool) -> PresenceState {
        self.desired_present = desired_present;
        self.state = match desired_present {
            true => PresenceState::Mounted,
            false if self.retain_mount && self.is_mounted() => PresenceState::Suspended,
            false => PresenceState::Unmounted,
        };
        self.state
    }

    pub fn complete_unmount(&mut self) -> bool {
        if self.desired_present || self.state != PresenceState::Suspended {
            return false;
        }

        self.state = PresenceState::Unmounted;
        true
    }
}

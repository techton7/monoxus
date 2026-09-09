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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PresenceCloseCycleId(u64);

impl PresenceCloseCycleId {
    pub const fn get(self) -> u64 {
        self.0
    }

    #[allow(dead_code)]
    pub(crate) const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresenceControllerUpdate {
    desired_present: bool,
    state: PresenceState,
    active_close_cycle: Option<PresenceCloseCycleId>,
    started_close_cycle: Option<PresenceCloseCycleId>,
    invalidated_close_cycle: Option<PresenceCloseCycleId>,
}

impl PresenceControllerUpdate {
    pub const fn desired_present(&self) -> bool {
        self.desired_present
    }

    pub const fn state(&self) -> PresenceState {
        self.state
    }

    pub const fn should_render(&self) -> bool {
        !matches!(self.state, PresenceState::Unmounted)
    }

    pub const fn active_close_cycle(&self) -> Option<PresenceCloseCycleId> {
        self.active_close_cycle
    }

    pub const fn started_close_cycle(&self) -> Option<PresenceCloseCycleId> {
        self.started_close_cycle
    }

    pub const fn invalidated_close_cycle(&self) -> Option<PresenceCloseCycleId> {
        self.invalidated_close_cycle
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresenceController {
    presence: Presence,
    active_close_cycle: Option<PresenceCloseCycleId>,
    next_close_cycle: u64,
}

impl PresenceController {
    pub const fn new(desired_present: bool) -> Self {
        Self {
            presence: Presence::new(desired_present),
            active_close_cycle: None,
            next_close_cycle: 1,
        }
    }

    pub const fn with_retained_mount(mut self, retain_mount: bool) -> Self {
        self.presence = self.presence.with_retained_mount(retain_mount);
        self
    }

    pub const fn desired_present(&self) -> bool {
        self.presence.desired_present()
    }

    pub const fn retain_mount(&self) -> bool {
        self.presence.retain_mount()
    }

    pub const fn state(&self) -> PresenceState {
        self.presence.state()
    }

    pub const fn should_render(&self) -> bool {
        self.presence.is_mounted()
    }

    pub const fn active_close_cycle(&self) -> Option<PresenceCloseCycleId> {
        self.active_close_cycle
    }

    pub const fn has_active_close_cycle(&self) -> bool {
        self.active_close_cycle.is_some()
    }

    pub const fn presence(&self) -> &Presence {
        &self.presence
    }

    pub fn sync(&mut self, desired_present: bool) -> PresenceControllerUpdate {
        let previous_state = self.presence.state();
        let previous_cycle = self.active_close_cycle;
        let state = self.presence.sync(desired_present);
        let mut started_close_cycle = None;
        let mut invalidated_close_cycle = None;

        match state {
            PresenceState::Mounted => {
                if previous_state == PresenceState::Suspended {
                    invalidated_close_cycle = previous_cycle;
                }
                self.active_close_cycle = None;
            }
            PresenceState::Unmounted => {
                self.active_close_cycle = None;
            }
            PresenceState::Suspended => {
                if previous_state == PresenceState::Suspended && previous_cycle.is_some() {
                    self.active_close_cycle = previous_cycle;
                } else {
                    let cycle_id = self.next_close_cycle_id();
                    self.active_close_cycle = Some(cycle_id);
                    started_close_cycle = Some(cycle_id);
                }
            }
        }

        PresenceControllerUpdate {
            desired_present: self.presence.desired_present(),
            state,
            active_close_cycle: self.active_close_cycle,
            started_close_cycle,
            invalidated_close_cycle,
        }
    }

    pub fn complete_close_cycle(&mut self, cycle_id: PresenceCloseCycleId) -> bool {
        if self.active_close_cycle != Some(cycle_id) {
            return false;
        }

        let completed = self.presence.complete_unmount();
        if completed {
            self.active_close_cycle = None;
        }
        completed
    }

    fn next_close_cycle_id(&mut self) -> PresenceCloseCycleId {
        let cycle_id = PresenceCloseCycleId(self.next_close_cycle);
        self.next_close_cycle = self
            .next_close_cycle
            .checked_add(1)
            .expect("presence close cycle overflow");
        cycle_id
    }
}

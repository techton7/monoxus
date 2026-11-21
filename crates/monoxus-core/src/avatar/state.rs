use super::types::AvatarImageLoadingStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarStateMachine {
    status: AvatarImageLoadingStatus,
    generation: u64,
}

impl Default for AvatarStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarStateMachine {
    pub const fn new() -> Self {
        Self {
            status: AvatarImageLoadingStatus::Idle,
            generation: 0,
        }
    }

    pub const fn status(&self) -> AvatarImageLoadingStatus {
        self.status
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn begin_generation(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.status = AvatarImageLoadingStatus::Idle;
        self.generation
    }

    pub fn transition(
        &mut self,
        generation: u64,
        next: AvatarImageLoadingStatus,
    ) -> Option<AvatarImageLoadingStatus> {
        if generation != self.generation || next == self.status {
            return None;
        }
        self.status = next;
        Some(next)
    }

    pub const fn owns(&self, generation: u64) -> bool {
        generation == self.generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarFallbackState {
    generation: u64,
    elapsed: bool,
}

impl Default for AvatarFallbackState {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarFallbackState {
    pub const fn new() -> Self {
        Self {
            generation: 0,
            elapsed: false,
        }
    }

    pub fn begin(&mut self, immediately_visible: bool) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.elapsed = immediately_visible;
        self.generation
    }

    pub fn finish(&mut self, generation: u64) -> bool {
        if self.generation != generation {
            return false;
        }
        self.elapsed = true;
        true
    }

    pub const fn is_visible(&self, status: AvatarImageLoadingStatus) -> bool {
        self.elapsed && !matches!(status, AvatarImageLoadingStatus::Loaded)
    }
}

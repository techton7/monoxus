#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusScope<T> {
    root: T,
    branches: Vec<T>,
    restore_focus_to: Option<T>,
    autofocus_target: Option<T>,
    autofocus_enabled: bool,
    last_focused: Option<T>,
    trap_focus: bool,
    loop_focus: bool,
    active: bool,
    parent_paused: bool,
}

impl<T> FocusScope<T>
where
    T: Clone + PartialEq,
{
    pub fn new(root: T) -> Self {
        Self {
            root,
            branches: Vec::new(),
            restore_focus_to: None,
            autofocus_target: None,
            autofocus_enabled: true,
            last_focused: None,
            trap_focus: false,
            loop_focus: false,
            active: false,
            parent_paused: false,
        }
    }

    pub fn root(&self) -> &T {
        &self.root
    }

    pub fn branches(&self) -> &[T] {
        &self.branches
    }

    pub fn with_trap_focus(mut self, trap_focus: bool) -> Self {
        self.trap_focus = trap_focus;
        self
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub fn set_trap_focus(&mut self, trap_focus: bool) {
        self.trap_focus = trap_focus;
    }

    pub fn set_loop_focus(&mut self, loop_focus: bool) {
        self.loop_focus = loop_focus;
    }

    pub fn traps_focus(&self) -> bool {
        self.trap_focus
    }

    pub fn loops_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_parent_paused(&self) -> bool {
        self.parent_paused
    }

    pub fn set_autofocus_enabled(&mut self, enabled: bool) {
        self.autofocus_enabled = enabled;
    }

    pub fn autofocus_enabled(&self) -> bool {
        self.autofocus_enabled
    }

    pub fn set_autofocus_target(&mut self, target: Option<T>) {
        self.autofocus_target = target;
    }

    pub fn autofocus_target(&self) -> Option<&T> {
        self.autofocus_target.as_ref()
    }

    pub fn capture_restore_target(&mut self, target: Option<T>) {
        self.restore_focus_to = target;
    }

    pub fn restore_target(&self) -> Option<&T> {
        self.restore_focus_to.as_ref()
    }

    pub fn restore_focus(&self) -> Option<T> {
        self.restore_focus_to
            .clone()
            .or_else(|| self.last_focused.clone())
    }

    pub fn last_focused(&self) -> Option<&T> {
        self.last_focused.as_ref()
    }

    pub fn register_branch(&mut self, branch: T) -> bool {
        if self.root == branch || self.branches.iter().any(|candidate| candidate == &branch) {
            return false;
        }

        self.branches.push(branch);
        true
    }

    pub fn unregister_branch(&mut self, branch: &T) -> bool {
        let Some(position) = self
            .branches
            .iter()
            .position(|candidate| candidate == branch)
        else {
            return false;
        };

        self.branches.remove(position);
        true
    }

    pub fn contains(&self, node: &T) -> bool {
        &self.root == node || self.branches.iter().any(|branch| branch == node)
    }

    pub fn activate(&mut self) -> Option<T> {
        self.active = true;
        self.parent_paused = self.trap_focus;

        if !self.autofocus_enabled {
            return None;
        }

        let target = self
            .autofocus_target
            .clone()
            .filter(|node| self.contains(node))
            .or_else(|| self.last_focused.clone())
            .or_else(|| Some(self.root.clone()));

        if let Some(target) = target.clone() {
            self.last_focused = Some(target);
        }

        target
    }

    pub fn deactivate(&mut self) -> Option<T> {
        self.active = false;
        self.parent_paused = false;
        self.restore_focus()
    }

    pub fn focus(&mut self, node: T) -> bool {
        if !self.contains(&node) {
            return false;
        }

        self.last_focused = Some(node);
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FocusGuardSide {
    Before,
    After,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusGuards<T> {
    before: T,
    after: T,
    retain_count: usize,
}

impl<T> FocusGuards<T>
where
    T: PartialEq,
{
    pub const fn new(before: T, after: T) -> Self {
        Self {
            before,
            after,
            retain_count: 0,
        }
    }

    pub fn before(&self) -> &T {
        &self.before
    }

    pub fn after(&self) -> &T {
        &self.after
    }

    pub fn retain_count(&self) -> usize {
        self.retain_count
    }

    pub fn is_installed(&self) -> bool {
        self.retain_count > 0
    }

    pub fn retain(&mut self) -> usize {
        self.retain_count = self.retain_count.saturating_add(1);
        self.retain_count
    }

    pub fn release(&mut self) -> usize {
        self.retain_count = self.retain_count.saturating_sub(1);
        self.retain_count
    }

    pub fn contains(&self, node: &T) -> bool {
        &self.before == node || &self.after == node
    }

    pub fn side_of(&self, node: &T) -> Option<FocusGuardSide> {
        if &self.before == node {
            Some(FocusGuardSide::Before)
        } else if &self.after == node {
            Some(FocusGuardSide::After)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DismissEvent {
    Escape,
    PointerDownOutside,
    FocusOutside,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DismissLayer<T> {
    id: T,
    branches: Vec<T>,
    modal: bool,
    dismiss_on_escape: bool,
    dismiss_on_pointer_down_outside: bool,
    dismiss_on_focus_outside: bool,
}

impl<T> DismissLayer<T>
where
    T: PartialEq,
{
    pub fn new(id: T) -> Self {
        Self {
            id,
            branches: Vec::new(),
            modal: false,
            dismiss_on_escape: true,
            dismiss_on_pointer_down_outside: true,
            dismiss_on_focus_outside: true,
        }
    }

    pub fn id(&self) -> &T {
        &self.id
    }

    pub fn branches(&self) -> &[T] {
        &self.branches
    }

    pub fn with_modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    pub fn set_modal(&mut self, modal: bool) {
        self.modal = modal;
    }

    pub fn with_escape_dismiss(mut self, enabled: bool) -> Self {
        self.dismiss_on_escape = enabled;
        self
    }

    pub fn set_escape_dismiss(&mut self, enabled: bool) {
        self.dismiss_on_escape = enabled;
    }

    pub fn with_pointer_down_outside_dismiss(mut self, enabled: bool) -> Self {
        self.dismiss_on_pointer_down_outside = enabled;
        self
    }

    pub fn set_pointer_down_outside_dismiss(&mut self, enabled: bool) {
        self.dismiss_on_pointer_down_outside = enabled;
    }

    pub fn with_focus_outside_dismiss(mut self, enabled: bool) -> Self {
        self.dismiss_on_focus_outside = enabled;
        self
    }

    pub fn set_focus_outside_dismiss(&mut self, enabled: bool) {
        self.dismiss_on_focus_outside = enabled;
    }

    pub fn is_modal(&self) -> bool {
        self.modal
    }

    pub fn register_branch(&mut self, branch: T) -> bool {
        if self.id == branch || self.branches.iter().any(|candidate| candidate == &branch) {
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

    pub fn contains(&self, target: &T) -> bool {
        &self.id == target || self.branches.iter().any(|branch| branch == target)
    }

    pub fn is_topmost(&self, stack: &[T]) -> bool {
        stack.last().is_some_and(|top| top == &self.id)
    }

    pub fn should_dismiss_escape(&self, stack: &[T]) -> bool {
        self.dismiss_on_escape && self.is_topmost(stack)
    }

    pub fn should_dismiss_outside_pointer(&self, target: Option<&T>, stack: &[T]) -> bool {
        self.dismiss_on_pointer_down_outside
            && self.is_topmost(stack)
            && target.map(|target| !self.contains(target)).unwrap_or(true)
    }

    pub fn should_dismiss_outside_focus(&self, target: Option<&T>, stack: &[T]) -> bool {
        self.dismiss_on_focus_outside
            && self.is_topmost(stack)
            && target.map(|target| !self.contains(target)).unwrap_or(true)
    }

    pub fn blocks_outside_interaction(&self) -> bool {
        self.modal
    }
}

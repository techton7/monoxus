pub mod button;
pub mod link;
pub mod root;
pub mod separator;
pub mod toggle_group;
pub mod toggle_item;

pub use self::{
    button::{ToolbarButton, ToolbarButtonProps},
    link::{ToolbarLink, ToolbarLinkProps},
    root::{Toolbar, ToolbarRoot, ToolbarRootProps},
    separator::{ToolbarSeparator, ToolbarSeparatorProps},
    toggle_group::{ToolbarToggleGroup, ToolbarToggleGroupProps},
    toggle_item::{ToolbarToggleItem, ToolbarToggleItemProps},
};

use super::runtime::{ToolbarRuntime, ToolbarToggleGroupRuntime};

#[derive(Clone)]
pub struct ToolbarRootContext {
    pub runtime: ToolbarRuntime,
}

pub use ToolbarRootContext as ToolbarContext;

#[derive(Clone)]
pub struct ToolbarToggleGroupContext {
    pub runtime: ToolbarToggleGroupRuntime,
}

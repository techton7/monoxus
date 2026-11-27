pub mod next_button;
pub mod page;
pub mod prev_button;
pub mod root;

pub use self::{
    next_button::{PaginationNextButton, PaginationNextButtonProps},
    page::{PaginationPage, PaginationPageProps},
    prev_button::{PaginationPrevButton, PaginationPrevButtonProps},
    root::{PaginationRoot, PaginationRootProps},
};

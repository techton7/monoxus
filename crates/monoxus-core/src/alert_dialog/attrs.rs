use crate::dialog::DialogStateRequest;
use crate::foundation::{shared::data_disabled_attr, state::DataState};

macro_rules! button_attributes {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            pub(crate) id: String,
            pub(crate) data_state: DataState,
            pub(crate) close_request: DialogStateRequest,
            pub(crate) disabled: bool,
        }

        impl $name {
            pub fn new(id: impl Into<String>, data_state: DataState) -> Self {
                Self {
                    id: id.into(),
                    data_state,
                    close_request: DialogStateRequest::Close,
                    disabled: false,
                }
            }

            pub fn with_disabled(mut self, disabled: bool) -> Self {
                self.disabled = disabled;
                self
            }

            pub fn id(&self) -> &str {
                &self.id
            }

            pub fn data_state(&self) -> &DataState {
                &self.data_state
            }

            pub const fn close_request(&self) -> DialogStateRequest {
                self.close_request
            }

            pub const fn is_disabled(&self) -> bool {
                self.disabled
            }

            pub const fn data_disabled(&self) -> Option<&'static str> {
                data_disabled_attr(self.disabled)
            }
        }
    };
}

button_attributes!(AlertDialogActionAttributes);
button_attributes!(AlertDialogCancelAttributes);

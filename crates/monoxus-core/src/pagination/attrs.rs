use crate::foundation::{
    shared::data_disabled_attr,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationRootAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_label: String,
    pub dir: &'static str,
    pub data_orientation: &'static str,
    pub data_monoxus_pagination_root: &'static str,
}

impl PaginationRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_label(&self) -> &str {
        &self.aria_label
    }

    pub fn dir(&self) -> &'static str {
        self.dir
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_monoxus_pagination_root(&self) -> &'static str {
        self.data_monoxus_pagination_root
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationPageAttributes {
    pub id: String,
    pub r#type: &'static str,
    pub value: usize,
    pub aria_label: String,
    pub aria_current: Option<&'static str>,
    pub aria_disabled: Option<&'static str>,
    pub disabled: bool,
    pub data_value: String,
    pub data_selected: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_monoxus_pagination_page: &'static str,
}

impl PaginationPageAttributes {
    pub fn new(id: String, value: usize, is_selected: bool, disabled: bool) -> Self {
        Self {
            id,
            r#type: "button",
            value,
            aria_label: format!("Page {value}"),
            aria_current: is_selected.then_some("page"),
            aria_disabled: disabled.then_some("true"),
            disabled,
            data_value: value.to_string(),
            data_selected: data_disabled_attr(is_selected),
            data_disabled: data_disabled_attr(disabled),
            data_monoxus_pagination_page: "",
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn button_type(&self) -> &'static str {
        self.r#type
    }

    pub fn value(&self) -> usize {
        self.value
    }

    pub fn aria_label(&self) -> &str {
        &self.aria_label
    }

    pub fn aria_current(&self) -> Option<&'static str> {
        self.aria_current
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn is_selected(&self) -> bool {
        self.data_selected.is_some()
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn data_selected(&self) -> Option<&'static str> {
        self.data_selected
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_monoxus_pagination_page(&self) -> &'static str {
        self.data_monoxus_pagination_page
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationPrevButtonAttributes {
    pub id: String,
    pub r#type: &'static str,
    pub disabled: bool,
    pub aria_disabled: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_monoxus_pagination_prev: &'static str,
}

impl PaginationPrevButtonAttributes {
    pub fn new(id: String, disabled: bool) -> Self {
        Self {
            id,
            r#type: "button",
            disabled,
            aria_disabled: disabled.then_some("true"),
            data_disabled: data_disabled_attr(disabled),
            data_monoxus_pagination_prev: "",
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn button_type(&self) -> &'static str {
        self.r#type
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_monoxus_pagination_prev(&self) -> &'static str {
        self.data_monoxus_pagination_prev
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationNextButtonAttributes {
    pub id: String,
    pub r#type: &'static str,
    pub disabled: bool,
    pub aria_disabled: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_monoxus_pagination_next: &'static str,
}

impl PaginationNextButtonAttributes {
    pub fn new(id: String, disabled: bool) -> Self {
        Self {
            id,
            r#type: "button",
            disabled,
            aria_disabled: disabled.then_some("true"),
            data_disabled: data_disabled_attr(disabled),
            data_monoxus_pagination_next: "",
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn button_type(&self) -> &'static str {
        self.r#type
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_monoxus_pagination_next(&self) -> &'static str {
        self.data_monoxus_pagination_next
    }
}


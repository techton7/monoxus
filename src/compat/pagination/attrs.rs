// [KEEP-MONOXUS-BRAND]

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Serialize)]
pub struct PaginationRootAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_label: String,
    pub dir: &'static str,
    pub data_orientation: &'static str,
    pub data_monoxus_pagination_root: &'static str,
}

impl From<ui_sdk::pagination::attrs::PaginationRootAttributes> for PaginationRootAttributes {
    fn from(base: ui_sdk::pagination::attrs::PaginationRootAttributes) -> Self {
        Self {
            id: base.id,
            role: base.role,
            aria_label: base.aria_label,
            dir: base.dir,
            data_orientation: base.data_orientation,
            data_monoxus_pagination_root: base.data_pagination_root,
        }
    }
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
#[derive(serde::Serialize)]
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

impl From<ui_sdk::pagination::attrs::PaginationPageAttributes> for PaginationPageAttributes {
    fn from(base: ui_sdk::pagination::attrs::PaginationPageAttributes) -> Self {
        Self {
            id: base.id,
            r#type: base.r#type,
            value: base.value,
            aria_label: base.aria_label,
            aria_current: base.aria_current,
            aria_disabled: base.aria_disabled,
            disabled: base.disabled,
            data_value: base.data_value,
            data_selected: base.data_selected,
            data_disabled: base.data_disabled,
            data_monoxus_pagination_page: base.data_pagination_page,
        }
    }
}

impl PaginationPageAttributes {
    pub fn new(id: String, value: usize, is_selected: bool, disabled: bool) -> Self {
        ui_sdk::pagination::attrs::PaginationPageAttributes::new(id, value, is_selected, disabled)
            .into()
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
#[derive(serde::Serialize)]
pub struct PaginationPrevButtonAttributes {
    pub id: String,
    pub r#type: &'static str,
    pub disabled: bool,
    pub aria_disabled: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_monoxus_pagination_prev: &'static str,
}

impl From<ui_sdk::pagination::attrs::PaginationPrevButtonAttributes>
    for PaginationPrevButtonAttributes
{
    fn from(base: ui_sdk::pagination::attrs::PaginationPrevButtonAttributes) -> Self {
        Self {
            id: base.id,
            r#type: base.r#type,
            disabled: base.disabled,
            aria_disabled: base.aria_disabled,
            data_disabled: base.data_disabled,
            data_monoxus_pagination_prev: base.data_pagination_prev,
        }
    }
}

impl PaginationPrevButtonAttributes {
    pub fn new(id: String, disabled: bool) -> Self {
        ui_sdk::pagination::attrs::PaginationPrevButtonAttributes::new(id, disabled).into()
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
#[derive(serde::Serialize)]
pub struct PaginationNextButtonAttributes {
    pub id: String,
    pub r#type: &'static str,
    pub disabled: bool,
    pub aria_disabled: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_monoxus_pagination_next: &'static str,
}

impl From<ui_sdk::pagination::attrs::PaginationNextButtonAttributes>
    for PaginationNextButtonAttributes
{
    fn from(base: ui_sdk::pagination::attrs::PaginationNextButtonAttributes) -> Self {
        Self {
            id: base.id,
            r#type: base.r#type,
            disabled: base.disabled,
            aria_disabled: base.aria_disabled,
            data_disabled: base.data_disabled,
            data_monoxus_pagination_next: base.data_pagination_next,
        }
    }
}

impl PaginationNextButtonAttributes {
    pub fn new(id: String, disabled: bool) -> Self {
        ui_sdk::pagination::attrs::PaginationNextButtonAttributes::new(id, disabled).into()
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

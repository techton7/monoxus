use std::rc::Rc;

use dioxus::document::{self, Eval};
use dioxus::prelude::*;

pub use crate::foundation::compose::{
    compose_part_event_handlers, compose_part_refs, project_as_child, MountedHandle,
};

use crate::foundation::{
    browser::{
        recv_document_dismiss_event, recv_floating_auto_update_event, restore_focus_element_by_id,
        start_document_dismiss_monitor, start_floating_auto_update_monitor,
        stop_document_dismiss_monitor, stop_floating_auto_update_monitor, DocumentDismissEvent,
        FloatingAutoUpdateEvent,
    },
    overlay::{FloatingLayer, PlacementSide, PortalHost, Rect, Size},
    shared::ScopeHandle,
    state::DataState,
};

pub const SELECT_PARTS: [SelectPart; 14] = [
    SelectPart::Root,
    SelectPart::Trigger,
    SelectPart::Value,
    SelectPart::Icon,
    SelectPart::Portal,
    SelectPart::Content,
    SelectPart::Viewport,
    SelectPart::Group,
    SelectPart::Label,
    SelectPart::Item,
    SelectPart::ItemText,
    SelectPart::ItemIndicator,
    SelectPart::Separator,
    SelectPart::HiddenInput,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectPart {
    Root,
    Trigger,
    Value,
    Icon,
    Portal,
    Content,
    Viewport,
    Group,
    Label,
    Item,
    ItemText,
    ItemIndicator,
    Separator,
    HiddenInput,
}

impl SelectPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Value => "value",
            Self::Icon => "icon",
            Self::Portal => "portal",
            Self::Content => "content",
            Self::Viewport => "viewport",
            Self::Group => "group",
            Self::Label => "label",
            Self::Item => "item",
            Self::ItemText => "item-text",
            Self::ItemIndicator => "item-indicator",
            Self::Separator => "separator",
            Self::HiddenInput => "hidden-input",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectItemData {
    pub value: String,
    pub text: String,
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectRelationships {
    scope: ScopeHandle,
    root_id: String,
    trigger_id: String,
    content_id: String,
}

impl SelectRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            trigger_id: scope.qualify("trigger"),
            content_id: scope.qualify("content"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn trigger_id(&self) -> &str {
        &self.trigger_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn item_id(&self, value: &str) -> String {
        self.scope.qualify(&format!("item-{value}"))
    }

    pub fn group_id(&self, group_key: &str) -> String {
        self.scope.qualify(&format!("group-{group_key}"))
    }

    pub fn label_id(&self, group_key: &str) -> String {
        self.scope.qualify(&format!("label-{group_key}"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectRootAttributes {
    pub id: String,
    pub data_state: DataState,
    pub disabled: bool,
}

impl SelectRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectTriggerAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_haspopup: &'static str,
    pub aria_expanded: &'static str,
    pub aria_controls: String,
    pub aria_disabled: Option<&'static str>,
    pub data_state: DataState,
    pub disabled: bool,
    pub tabindex: i32,
}

impl SelectTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_haspopup(&self) -> &'static str {
        self.aria_haspopup
    }

    pub fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
    }

    pub fn aria_controls(&self) -> &str {
        &self.aria_controls
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectPortalAttributes {
    pub host: PortalHost,
}

impl SelectPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectContentAttributes {
    pub id: String,
    pub role: &'static str,
    pub tabindex: i32,
    pub aria_activedescendant: Option<String>,
    pub data_state: DataState,
    pub data_side: PlacementSide,
}

impl SelectContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn aria_activedescendant(&self) -> Option<&str> {
        self.aria_activedescendant.as_deref()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn data_side(&self) -> PlacementSide {
        self.data_side
    }

    pub fn data_side_str(&self) -> &'static str {
        self.data_side.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectItemAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_selected: &'static str,
    pub aria_disabled: Option<&'static str>,
    pub data_state: &'static str,
    pub is_highlighted: bool,
    pub disabled: bool,
    pub value: String,
}

impl SelectItemAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_selected(&self) -> &'static str {
        self.aria_selected
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn data_state(&self) -> &'static str {
        self.data_state
    }

    pub fn is_highlighted(&self) -> bool {
        self.is_highlighted
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Select {
    relationships: SelectRelationships,
    value: Option<String>,
    open: bool,
    allow_deselect: bool,
    disabled: bool,
    portal_host: PortalHost,
}

impl Select {
    pub fn parts() -> &'static [SelectPart] {
        &SELECT_PARTS
    }

    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            relationships: SelectRelationships::new(scope),
            value: None,
            open: false,
            allow_deselect: false,
            disabled: false,
            portal_host: PortalHost::default_host(),
        }
    }

    pub fn with_value(mut self, val: Option<String>) -> Self {
        self.value = val;
        self
    }

    pub fn with_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn with_allow_deselect(mut self, allow: bool) -> Self {
        self.allow_deselect = allow;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.portal_host = portal_host;
        self
    }

    pub fn portal_host(&self) -> &PortalHost {
        &self.portal_host
    }

    pub fn portal_attributes(&self) -> SelectPortalAttributes {
        SelectPortalAttributes {
            host: self.portal_host.clone(),
        }
    }

    pub fn relationships(&self) -> &SelectRelationships {
        &self.relationships
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn allows_deselect(&self) -> bool {
        self.allow_deselect
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn root_attributes(&self) -> SelectRootAttributes {
        SelectRootAttributes {
            id: self.relationships.root_id().to_owned(),
            data_state: if self.open { DataState::Open } else { DataState::Closed },
            disabled: self.disabled,
        }
    }

    pub fn trigger_attributes(&self) -> SelectTriggerAttributes {
        SelectTriggerAttributes {
            id: self.relationships.trigger_id().to_owned(),
            role: "combobox",
            aria_haspopup: "listbox",
            aria_expanded: if self.open { "true" } else { "false" },
            aria_controls: self.relationships.content_id().to_owned(),
            aria_disabled: if self.disabled { Some("true") } else { None },
            data_state: if self.open { DataState::Open } else { DataState::Closed },
            disabled: self.disabled,
            tabindex: if self.disabled { -1 } else { 0 },
        }
    }

    pub fn content_attributes(&self, activedescendant: Option<String>) -> SelectContentAttributes {
        self.content_attributes_with_side(activedescendant, PlacementSide::Bottom)
    }

    pub fn content_attributes_with_side(
        &self,
        activedescendant: Option<String>,
        side: PlacementSide,
    ) -> SelectContentAttributes {
        SelectContentAttributes {
            id: self.relationships.content_id().to_owned(),
            role: "listbox",
            tabindex: -1,
            aria_activedescendant: activedescendant,
            data_state: if self.open { DataState::Open } else { DataState::Closed },
            data_side: side,
        }
    }

    pub fn item_attributes(&self, item_val: &str, is_highlighted: bool, disabled: bool) -> SelectItemAttributes {
        let is_selected = self.value.as_deref() == Some(item_val);
        let eff_disabled = self.disabled || disabled;

        SelectItemAttributes {
            id: self.relationships.item_id(item_val),
            role: "option",
            aria_selected: if is_selected { "true" } else { "false" },
            aria_disabled: if eff_disabled { Some("true") } else { None },
            data_state: if is_selected { "checked" } else { "unchecked" },
            is_highlighted,
            disabled: eff_disabled,
            value: item_val.to_owned(),
        }
    }
}

pub type SelectValueChangeHandler = Rc<dyn Fn(Option<String>)>;
pub type SelectOpenChangeHandler = Rc<dyn Fn(bool)>;

#[derive(Clone, Copy, PartialEq)]
pub struct SelectRuntimeState {
    pub value: Signal<Option<String>>,
    pub open: Signal<bool>,
    pub highlighted_value: Signal<Option<String>>,
    pub items: Signal<Vec<SelectItemData>>,
    pub typeahead_buffer: Signal<String>,
    pub last_key_timestamp_ms: Signal<f64>,
    pub side: Signal<PlacementSide>,
    pub dismiss_monitor: Signal<Option<Eval>>,
    pub dismiss_loop_token: Signal<u64>,
    pub position_monitor: Signal<Option<Eval>>,
    pub position_loop_token: Signal<u64>,
}

#[derive(Clone)]
pub struct SelectRuntime {
    select: Select,
    on_value_change: Option<SelectValueChangeHandler>,
    on_open_change: Option<SelectOpenChangeHandler>,
    state: SelectRuntimeState,
}

impl PartialEq for SelectRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.select == other.select && self.state == other.state
    }
}

pub fn use_select_runtime<F1, F2>(
    select: Select,
    on_value_change: Option<F1>,
    on_open_change: Option<F2>,
) -> SelectRuntime
where
    F1: Fn(Option<String>) + 'static,
    F2: Fn(bool) + 'static,
{
    let synced_val_change: Option<SelectValueChangeHandler> =
        on_value_change.map(|f| Rc::new(f) as SelectValueChangeHandler);
    let synced_open_change: Option<SelectOpenChangeHandler> =
        on_open_change.map(|f| Rc::new(f) as SelectOpenChangeHandler);

    let initial_val = select.value().map(str::to_owned);
    let initial_open = select.is_open();

    let state = SelectRuntimeState {
        value: use_signal(|| initial_val),
        open: use_signal(|| initial_open),
        highlighted_value: use_signal(|| None),
        items: use_signal(Vec::new),
        typeahead_buffer: use_signal(String::new),
        last_key_timestamp_ms: use_signal(|| 0.0),
        side: use_signal(|| PlacementSide::Bottom),
        dismiss_monitor: use_signal(|| None),
        dismiss_loop_token: use_signal(|| 0),
        position_monitor: use_signal(|| None),
        position_loop_token: use_signal(|| 0),
    };

    let cleanup_state = state;
    dioxus::core::use_drop(move || {
        if let Some(monitor) = cleanup_state.dismiss_monitor.peek().clone() {
            let _ = stop_document_dismiss_monitor(monitor);
        }
        if let Some(monitor) = cleanup_state.position_monitor.peek().clone() {
            let _ = stop_floating_auto_update_monitor(monitor);
        }
    });

    let effect_state = state.clone();

    use_effect(use_reactive((&select,), move |(select,)| {
        let mut val_sig = effect_state.value;
        let incoming_val = select.value().map(str::to_owned);
        if *val_sig.peek() != incoming_val {
            val_sig.set(incoming_val);
        }
        let mut open_sig = effect_state.open;
        if *open_sig.peek() != select.is_open() {
            open_sig.set(select.is_open());
        }
    }));

    SelectRuntime {
        select,
        on_value_change: synced_val_change,
        on_open_change: synced_open_change,
        state,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn sync_items_with_document_order(content_id: &str, items: &mut Vec<SelectItemData>) {
    let script = format!(
        r#"(function() {{
            const root = document.getElementById({content_id:?});
            if (!root) return [];
            return Array.from(root.querySelectorAll('[role="option"]'))
                .map(el => el.getAttribute('data-value') || "");
        }})()"#
    );
    if let Ok(val) = js_sys::eval(&script) {
        let arr = js_sys::Array::from(&val);
        let order: Vec<String> = arr.iter().filter_map(|v| v.as_string()).collect();
        if !order.is_empty() {
            items.sort_by_key(|item| {
                order.iter().position(|v| v == &item.value).unwrap_or(usize::MAX)
            });
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_items_with_document_order(_content_id: &str, _items: &mut Vec<SelectItemData>) {}

impl SelectRuntime {
    pub fn select(&self) -> Select {
        self.select.clone()
    }

    pub fn value(&self) -> Option<String> {
        self.state.value.read().clone()
    }

    pub fn is_open(&self) -> bool {
        *self.state.open.read()
    }

    pub fn is_disabled(&self) -> bool {
        self.select.is_disabled()
    }

    pub fn highlighted_value(&self) -> Option<String> {
        self.state.highlighted_value.read().clone()
    }

    pub fn side(&self) -> PlacementSide {
        *self.state.side.read()
    }

    pub fn set_side(&self, side: PlacementSide) {
        let mut side_sig = self.state.side;
        if *side_sig.peek() != side {
            side_sig.set(side);
        }
    }

    pub fn relationships(&self) -> &SelectRelationships {
        self.select.relationships()
    }

    pub fn sync_dom_order(&self) {
        let content_id = self.relationships().content_id().to_owned();
        let mut items_sig = self.state.items;
        let mut list = items_sig.write();
        sync_items_with_document_order(&content_id, &mut list);
    }

    pub fn register_item(&self, val: &str, text: &str, disabled: bool) {
        let mut items = self.state.items;
        let mut list = items.write();
        if let Some(existing) = list.iter_mut().find(|i| i.value == val) {
            existing.text = text.to_owned();
            existing.disabled = disabled;
        } else {
            list.push(SelectItemData {
                value: val.to_owned(),
                text: text.to_owned(),
                disabled,
            });
        }
    }

    pub fn start_dismiss_monitor(&self) {
        self.stop_dismiss_monitor();

        let mut token_sig = self.state.dismiss_loop_token;
        let next_token = token_sig.peek().saturating_add(1);
        token_sig.set(next_token);

        let monitor = start_document_dismiss_monitor();
        let mut dm_sig = self.state.dismiss_monitor;
        dm_sig.set(Some(monitor));

        let runtime = self.clone();
        let trigger_id = self.relationships().trigger_id().to_owned();
        let content_id = self.relationships().content_id().to_owned();

        spawn(async move {
            let mut monitor = monitor;
            loop {
                if *runtime.state.dismiss_loop_token.peek() != next_token {
                    break;
                }

                match recv_document_dismiss_event(&mut monitor).await {
                    Ok(DocumentDismissEvent::Stopped) => break,
                    Ok(DocumentDismissEvent::Escape) => {
                        runtime.close_dropdown();
                        break;
                    }
                    Ok(DocumentDismissEvent::PointerDown { path_ids }) => {
                        let is_inside = path_ids.iter().any(|id| id == &trigger_id || id == &content_id);
                        if !is_inside {
                            runtime.close_dropdown_without_restore();
                            break;
                        }
                    }
                    Ok(DocumentDismissEvent::FocusIn { path_ids }) => {
                        let is_inside = path_ids.iter().any(|id| id == &trigger_id || id == &content_id);
                        if !is_inside {
                            runtime.close_dropdown_without_restore();
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    pub fn stop_dismiss_monitor(&self) {
        let current_monitor = self.state.dismiss_monitor.peek().clone();
        if let Some(monitor) = current_monitor {
            let mut dm_sig = self.state.dismiss_monitor;
            dm_sig.set(None);
            let _ = stop_document_dismiss_monitor(monitor);
        }
    }

    pub async fn measure_and_update_placement(&self, trigger_id: &str, content_id: &str) {
        let eval = document::eval(&format!(
            r#"return (() => {{
                const trigger = document.getElementById({trigger_id:?});
                const content = document.getElementById({content_id:?});
                if (!trigger) return null;
                const tRect = trigger.getBoundingClientRect();
                const cRect = (content && content.offsetHeight > 0)
                    ? content.getBoundingClientRect()
                    : {{ width: tRect.width, height: 180 }};
                return [
                    tRect.x, tRect.y, tRect.width, tRect.height,
                    cRect.width, cRect.height,
                    window.innerWidth, window.innerHeight
                ];
            }})()"#
        ));
        let result: Result<Option<[f64; 8]>, _> = eval.join().await;
        if let Ok(Some(arr)) = result {
            let tx = arr[0] as f32;
            let ty = arr[1] as f32;
            let tw = arr[2] as f32;
            let th = arr[3] as f32;
            let cw = arr[4] as f32;
            let ch = arr[5] as f32;
            let vpw = arr[6] as f32;
            let vph = arr[7] as f32;

            let anchor = Rect::new(tx, ty, tw, th);
            let content_size = Size::new(cw, ch);
            let viewport_size = Size::new(vpw, vph);

            let layer = FloatingLayer::new(PlacementSide::Bottom).with_side_offset(4.0);
            let placement = layer.position_with_available_size(anchor, content_size, viewport_size);
            self.set_side(placement.side());
        }
    }

    pub fn start_position_monitor(&self) {
        self.stop_position_monitor();

        let mut token_sig = self.state.position_loop_token;
        let next_token = token_sig.peek().saturating_add(1);
        token_sig.set(next_token);

        let trigger_id = self.relationships().trigger_id().to_owned();
        let content_id = self.relationships().content_id().to_owned();
        let monitor = start_floating_auto_update_monitor(&[&trigger_id], &content_id);
        let mut pm_sig = self.state.position_monitor;
        pm_sig.set(Some(monitor));

        let runtime = self.clone();
        spawn(async move {
            let mut monitor = monitor;

            // Immediate initial measurement
            runtime.measure_and_update_placement(&trigger_id, &content_id).await;

            loop {
                if *runtime.state.position_loop_token.peek() != next_token {
                    break;
                }

                match recv_floating_auto_update_event(&mut monitor).await {
                    Ok(FloatingAutoUpdateEvent::Scroll) | Ok(FloatingAutoUpdateEvent::Update) => {
                        runtime.measure_and_update_placement(&trigger_id, &content_id).await;
                    }
                    Ok(FloatingAutoUpdateEvent::Stopped) => break,
                    Err(_) => break,
                }
            }
        });
    }

    pub fn stop_position_monitor(&self) {
        let current_monitor = self.state.position_monitor.peek().clone();
        if let Some(monitor) = current_monitor {
            let mut pm_sig = self.state.position_monitor;
            pm_sig.set(None);
            let _ = stop_floating_auto_update_monitor(monitor);
        }
    }

    pub fn open_dropdown(&self) {
        if self.is_disabled() {
            return;
        }
        let mut open_sig = self.state.open;
        open_sig.set(true);

        if let Some(ref cb) = self.on_open_change {
            cb(true);
        }

        // Initialize highlighted value
        let current_val = self.value();
        let items = self.state.items.read();
        let init_highlight = current_val
            .as_ref()
            .and_then(|v| items.iter().find(|i| &i.value == v && !i.disabled).map(|i| i.value.clone()))
            .or_else(|| items.iter().find(|i| !i.disabled).map(|i| i.value.clone()));

        let mut hl_sig = self.state.highlighted_value;
        hl_sig.set(init_highlight);

        // Start document dismiss monitor (outside click / escape)
        self.start_dismiss_monitor();

        // Focus SelectContent directly per interact.md #1 (with preventScroll: true)
        let content_id = self.relationships().content_id().to_owned();
        restore_focus_element_by_id(&content_id);
    }

    pub fn close_dropdown_with_options(&self, restore_focus: bool) {
        let mut open_sig = self.state.open;
        open_sig.set(false);

        if let Some(ref cb) = self.on_open_change {
            cb(false);
        }

        let mut hl_sig = self.state.highlighted_value;
        hl_sig.set(None);
        let mut buf_sig = self.state.typeahead_buffer;
        buf_sig.set(String::new());

        // Stop document dismiss monitor and position monitor
        self.stop_dismiss_monitor();
        self.stop_position_monitor();

        if restore_focus {
            // Restore focus to SelectTrigger per interact.md #1 (with preventScroll: true)
            let trigger_id = self.relationships().trigger_id().to_owned();
            restore_focus_element_by_id(&trigger_id);
        }
    }

    pub fn close_dropdown(&self) {
        self.close_dropdown_with_options(true);
    }

    pub fn close_dropdown_without_restore(&self) {
        self.close_dropdown_with_options(false);
    }

    pub fn toggle(&self) {
        if self.is_open() {
            self.close_dropdown();
        } else {
            self.open_dropdown();
        }
    }

    pub fn select_item(&self, val: &str) {
        if self.is_disabled() {
            return;
        }
        let is_current = self.value().as_deref() == Some(val);
        let next_val = if self.select.allows_deselect() && is_current {
            None
        } else {
            Some(val.to_owned())
        };

        let mut val_sig = self.state.value;
        val_sig.set(next_val.clone());

        if let Some(ref cb) = self.on_value_change {
            cb(next_val);
        }

        self.close_dropdown();
    }

    pub fn highlight_next(&self) {
        let items = self.state.items.read();
        let enabled: Vec<_> = items.iter().filter(|i| !i.disabled).collect();
        if enabled.is_empty() {
            return;
        }

        let current = self.highlighted_value();
        let curr_idx = current
            .as_ref()
            .and_then(|v| enabled.iter().position(|i| &i.value == v));

        let next_idx = match curr_idx {
            Some(idx) => (idx + 1) % enabled.len(),
            None => 0,
        };

        let mut hl = self.state.highlighted_value;
        hl.set(Some(enabled[next_idx].value.clone()));
    }

    pub fn highlight_prev(&self) {
        let items = self.state.items.read();
        let enabled: Vec<_> = items.iter().filter(|i| !i.disabled).collect();
        if enabled.is_empty() {
            return;
        }

        let current = self.highlighted_value();
        let curr_idx = current
            .as_ref()
            .and_then(|v| enabled.iter().position(|i| &i.value == v));

        let prev_idx = match curr_idx {
            Some(0) => enabled.len() - 1,
            Some(idx) => idx - 1,
            None => enabled.len() - 1,
        };

        let mut hl = self.state.highlighted_value;
        hl.set(Some(enabled[prev_idx].value.clone()));
    }

    pub fn highlight_first(&self) {
        let items = self.state.items.read();
        if let Some(first) = items.iter().find(|i| !i.disabled) {
            let mut hl = self.state.highlighted_value;
            hl.set(Some(first.value.clone()));
        }
    }

    pub fn highlight_last(&self) {
        let items = self.state.items.read();
        if let Some(last) = items.iter().rev().find(|i| !i.disabled) {
            let mut hl = self.state.highlighted_value;
            hl.set(Some(last.value.clone()));
        }
    }

    pub fn handle_typeahead(&self, ch: char, now_ms: f64) {
        let last_time = *self.state.last_key_timestamp_ms.read();
        let mut buf = self.state.typeahead_buffer.read().clone();

        // 500ms synchronous threshold per interact.md #2
        if now_ms - last_time > 500.0 {
            buf.clear();
        }
        buf.push(ch);

        let mut buf_sig = self.state.typeahead_buffer;
        buf_sig.set(buf.clone());
        let mut time_sig = self.state.last_key_timestamp_ms;
        time_sig.set(now_ms);

        let prefix = buf.to_lowercase();
        let items = self.state.items.read();
        if let Some(matched) = items
            .iter()
            .find(|i| !i.disabled && i.text.to_lowercase().starts_with(&prefix))
        {
            let mut hl = self.state.highlighted_value;
            hl.set(Some(matched.value.clone()));
        }
    }

    pub fn handle_trigger_keydown(&self, event: &KeyboardEvent) {
        if self.is_disabled() || self.is_open() {
            return;
        }
        let key = event.key().to_string();
        match key.as_str() {
            "ArrowDown" | "ArrowUp" | " " | "Enter" => {
                event.prevent_default();
                self.open_dropdown();
            }
            _ => {}
        }
    }

    pub fn handle_content_keydown(&self, event: &KeyboardEvent, now_ms: f64) {
        if !self.is_open() {
            return;
        }
        self.sync_dom_order();
        let key = event.key().to_string();
        match key.as_str() {
            "ArrowDown" => {
                event.prevent_default();
                self.highlight_next();
            }
            "ArrowUp" => {
                event.prevent_default();
                self.highlight_prev();
            }
            "Home" => {
                event.prevent_default();
                self.highlight_first();
            }
            "End" => {
                event.prevent_default();
                self.highlight_last();
            }
            "Enter" | " " => {
                event.prevent_default();
                if let Some(cand) = self.highlighted_value() {
                    self.select_item(&cand);
                }
            }
            "Escape" => {
                event.prevent_default();
                self.close_dropdown();
            }
            "Tab" => {
                // Do NOT prevent_default(), let browser naturally advance tab focus!
                self.close_dropdown_without_restore();
            }
            _ => {
                if key.len() == 1 {
                    let ch = key.chars().next().unwrap();
                    if ch.is_alphanumeric() || ch.is_whitespace() {
                        self.handle_typeahead(ch, now_ms);
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Context & Declarative Components
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct SelectContext {
    pub runtime: SelectRuntime,
    pub name: Option<String>,
}

#[component]
pub fn SelectRoot(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] runtime: Option<SelectRuntime>,
    #[props(default)] value: Option<String>,
    #[props(default)] default_value: Option<String>,
    #[props(default)] on_value_change: Option<EventHandler<Option<String>>>,
    #[props(default = false)] open: bool,
    #[props(default)] on_open_change: Option<EventHandler<bool>>,
    #[props(default = false)] allow_deselect: bool,
    #[props(default = false)] disabled: bool,
    #[props(default)] name: Option<String>,
    children: Element,
) -> Element {
    let fallback_scope_id = id.clone().unwrap_or_else(|| "select".to_string());
    let fallback_val = value.clone().or(default_value);
    let fallback_select = Select::new(ScopeHandle::root("select").child(fallback_scope_id))
        .with_value(fallback_val)
        .with_open(open)
        .with_allow_deselect(allow_deselect)
        .with_disabled(disabled);

    let val_change = on_value_change.map(|h| move |v| h.call(v));
    let open_change = on_open_change.map(|h| move |o| h.call(o));
    let default_runtime = use_select_runtime(fallback_select, val_change, open_change);

    let runtime = runtime.unwrap_or(default_runtime);

    use_context_provider(|| SelectContext {
        runtime: runtime.clone(),
        name: name.clone(),
    });

    let attrs = runtime.select().root_attributes();

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            {children}
            if let Some(input_name) = name {
                SelectHiddenInput {
                    name: input_name,
                    value: runtime.value(),
                    disabled,
                }
            }
        }
    }
}

#[component]
pub fn SelectTrigger(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let attrs = ctx.runtime.select().trigger_attributes();

    rsx! {
        button {
            r#type: "button",
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            aria_haspopup: "{attrs.aria_haspopup()}",
            aria_expanded: "{attrs.aria_expanded()}",
            aria_controls: "{attrs.aria_controls()}",
            aria_disabled: attrs.aria_disabled(),
            tabindex: "{attrs.tabindex()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": "{attrs.data_state_str()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            onclick: {
                let runtime = ctx.runtime.clone();
                move |_| {
                    runtime.toggle();
                }
            },
            onkeydown: {
                let runtime = ctx.runtime.clone();
                move |evt| {
                    runtime.handle_trigger_keydown(&evt);
                }
            },
            {children}
        }
    }
}

#[component]
pub fn SelectValue(
    #[props(default)] placeholder: Option<String>,
    #[props(default)] class: Option<String>,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let current_val = ctx.runtime.value();
    let items = ctx.runtime.state.items.read();

    let display_text = current_val
        .as_ref()
        .and_then(|v| items.iter().find(|i| &i.value == v).map(|i| i.text.clone()))
        .or(current_val.clone())
        .or(placeholder.clone())
        .unwrap_or_default();

    let is_placeholder = current_val.is_none() && placeholder.is_some();

    rsx! {
        span {
            class: class.as_deref().unwrap_or_default(),
            "data-placeholder": if is_placeholder { "true" } else { "false" },
            "{display_text}"
        }
    }
}

#[component]
pub fn SelectIcon(
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        span {
            aria_hidden: "true",
            class: class.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn SelectPortal(
    #[props(default)] host: Option<PortalHost>,
    children: Element,
) -> Element {
    let _ = host;
    rsx! {
        {children}
    }
}

#[component]
pub fn SelectContent(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let is_open = ctx.runtime.is_open();
    let rels = ctx.runtime.relationships();
    let content_id = rels.content_id().to_owned();

    let runtime = ctx.runtime.clone();
    use_effect(use_reactive((&is_open,), {
        let cid = content_id.clone();
        let rt = runtime.clone();
        move |(open,)| {
            if open {
                restore_focus_element_by_id(&cid);
                rt.sync_dom_order();
                rt.start_position_monitor();
            } else {
                rt.stop_position_monitor();
            }
        }
    }));

    if !is_open {
        return rsx! {};
    }

    let hl = ctx.runtime.highlighted_value();
    let activedescendant = hl.map(|v| rels.item_id(&v));
    let side = ctx.runtime.side();
    let attrs = ctx.runtime.select().content_attributes_with_side(activedescendant, side);

    let side_placement_style = match side {
        PlacementSide::Top => "bottom: calc(100% + 4px) !important; top: auto !important;",
        _ => "top: calc(100% + 4px) !important; bottom: auto !important;",
    };
    let base_position_style = "position: absolute; left: 0; min-width: 100%;";
    let merged_style = if let Some(custom) = style {
        format!("{base_position_style} {side_placement_style} {custom}")
    } else {
        format!("{base_position_style} {side_placement_style}")
    };

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            tabindex: "{attrs.tabindex()}",
            aria_activedescendant: attrs.aria_activedescendant(),
            class: class.as_deref().unwrap_or_default(),
            style: "{merged_style}",
            "data-state": "{attrs.data_state_str()}",
            "data-side": "{attrs.data_side_str()}",
            onkeydown: {
                let runtime = ctx.runtime.clone();
                move |evt| {
                    #[cfg(target_arch = "wasm32")]
                    let now_ms = js_sys::Date::now();
                    #[cfg(not(target_arch = "wasm32"))]
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs_f64() * 1000.0)
                        .unwrap_or(0.0);

                    runtime.handle_content_keydown(&evt, now_ms);
                }
            },
            {children}
        }
    }
}

#[component]
pub fn SelectViewport(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or("overflow-y: auto;"),
            {children}
        }
    }
}

#[component]
pub fn SelectGroup(
    #[props(default)] class: Option<String>,
    #[props(default)] label_id: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div {
            role: "group",
            aria_labelledby: label_id.as_deref(),
            class: class.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn SelectLabel(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div {
            id: id.as_deref(),
            class: class.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn SelectItem(
    value: String,
    #[props(default = false)] disabled: bool,
    #[props(default)] text: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let item_val = value.clone();
    let label_text = text.clone().unwrap_or_else(|| value.clone());

    use_effect(use_reactive((&item_val, &label_text, &disabled), {
        let runtime = ctx.runtime.clone();
        move |(val, txt, dis)| {
            runtime.register_item(&val, &txt, dis);
            let mut hl_sig = runtime.state.highlighted_value;
            if hl_sig.peek().is_none() && !dis {
                if runtime.value().as_deref() == Some(&val) {
                    hl_sig.set(Some(val.clone()));
                }
            }
        }
    }));

    let is_hl = ctx.runtime.highlighted_value().as_deref() == Some(&value);
    let attrs = ctx.runtime.select().item_attributes(&value, is_hl, disabled);

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            aria_selected: "{attrs.aria_selected()}",
            aria_disabled: attrs.aria_disabled(),
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref(),
            "data-state": "{attrs.data_state()}",
            "data-highlighted": if attrs.is_highlighted() { "true" } else { "false" },
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            "data-value": "{attrs.value()}",
            onclick: {
                let runtime = ctx.runtime.clone();
                let click_val = value.clone();
                move |_| {
                    if !disabled && !runtime.is_disabled() {
                        runtime.select_item(&click_val);
                    }
                }
            },
            onpointerenter: {
                let runtime = ctx.runtime.clone();
                let hover_val = value.clone();
                move |_| {
                    if !disabled && !runtime.is_disabled() {
                        let mut hl = runtime.state.highlighted_value;
                        hl.set(Some(hover_val.clone()));
                    }
                }
            },
            {children}
        }
    }
}

#[component]
pub fn SelectItemText(
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        span {
            class: class.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn SelectItemIndicator(
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        span {
            aria_hidden: "true",
            class: class.as_deref().unwrap_or_default(),
            {children}
        }
    }
}

#[component]
pub fn SelectSeparator(
    #[props(default)] class: Option<String>,
) -> Element {
    rsx! {
        div {
            role: "separator",
            aria_hidden: "true",
            class: class.as_deref().unwrap_or_default(),
        }
    }
}

#[component]
pub fn SelectHiddenInput(
    name: String,
    value: Option<String>,
    #[props(default = false)] disabled: bool,
) -> Element {
    let val_str = value.unwrap_or_default();
    rsx! {
        input {
            r#type: "hidden",
            name: "{name}",
            value: "{val_str}",
            disabled: disabled,
            aria_hidden: "true",
        }
    }
}

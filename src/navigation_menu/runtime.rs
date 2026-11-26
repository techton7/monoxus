use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use dioxus::prelude::*;
use futures_timer::Delay;
use oxidase::dom::FocusOptions;
use oxidase::dom::observer::{CaptureEventKind, CaptureEventMask, observe_capture_events_with_mask};

use crate::foundation::{
    browser::{
        ResizeObserverBox, WatcherGuard, focus_element_by_id,
        focus_element_reliable as foundation_focus_element_reliable, get_element_offset_rect,
        get_tabbable_candidates, query_descendant_order, query_selector_all_by_id,
        remove_from_tab_order, restore_focus_element_by_id_with_options, restore_tab_order,
        spawn_root_task, start_resize_monitor,
    },
    overlay::{
        DismissCallbacks, DismissConfig, DismissEvent, DismissLayerHandle, WatcherSlot,
        use_dismissable_layer, use_watcher_slot,
    },
    roving_focus::RovingFocusRuntime,
    shared::{
        Direction, EscapeKeyDownEvent, FocusOutsideEvent, InteractOutsideEvent, Orientation,
        PointerDownOutsideEvent, ScopeHandle,
    },
};

use super::{
    attrs::{
        NavigationMenuAriaOwnsAttributes, NavigationMenuContentAttributes,
        NavigationMenuFocusProxyAttributes, NavigationMenuIndicatorAttributes,
        NavigationMenuItemAttributes, NavigationMenuLinkAttributes, NavigationMenuListAttributes,
        NavigationMenuListTrackAttributes, NavigationMenuRootAttributes,
        NavigationMenuSubAttributes, NavigationMenuTriggerAttributes,
        NavigationMenuViewportAttributes,
    },
    relationships::NavigationMenuRelationships,
    state::{
        NavigationMenuDef, NavigationMenuFocusProxyDecision, NavigationMenuHoverAction,
        NavigationMenuHoverMachine, NavigationMenuSubDef, NavigationMenuTimerCommand,
        compute_motion_direction, is_item_hover_enabled, resolve_focus_proxy_decision,
        should_dismiss_on_focus_outside,
    },
    types::{
        NAVIGATION_MENU_CLOSE_GRACE_MS, NavigationMenuActivationMode,
        NavigationMenuFocusProxySide, NavigationMenuIndicatorPosition, NavigationMenuMotion,
        NavigationMenuOutsideBehavior, NavigationMenuViewportSize,
    },
};

/// Render props passed to `render_child` on `NavigationMenuRoot`.
#[derive(Clone)]
pub struct NavigationMenuRootRenderProps {
    pub attrs: NavigationMenuRootAttributes,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Render props passed to `render_child` on `NavigationMenuSub`.
#[derive(Clone)]
pub struct NavigationMenuSubRenderProps {
    pub attrs: NavigationMenuSubAttributes,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Render props passed to `render_child` on `NavigationMenuList`.
#[derive(Clone)]
pub struct NavigationMenuListRenderProps {
    pub attrs: NavigationMenuListAttributes,
    pub track_attrs: NavigationMenuListTrackAttributes,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub indicator: Element,
}

/// Render props passed to `render_child` on `NavigationMenuItem`.
#[derive(Clone)]
pub struct NavigationMenuItemRenderProps {
    pub attrs: NavigationMenuItemAttributes,
    pub value: String,
    pub is_open: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Render props passed to `render_child` on `NavigationMenuTrigger`.
#[derive(Clone)]
pub struct NavigationMenuTriggerRenderProps {
    pub attrs: NavigationMenuTriggerAttributes,
    pub is_open: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Render props passed to `render_child` on `NavigationMenuContent`.
#[derive(Clone)]
pub struct NavigationMenuContentRenderProps {
    pub attrs: NavigationMenuContentAttributes,
    pub is_open: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Render props passed to `render_child` on `NavigationMenuLink`.
#[derive(Clone)]
pub struct NavigationMenuLinkRenderProps {
    pub attrs: NavigationMenuLinkAttributes,
    pub active: bool,
    pub is_focused: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Render props passed to `render_child` on `NavigationMenuIndicator`.
#[derive(Clone)]
pub struct NavigationMenuIndicatorRenderProps {
    pub attrs: NavigationMenuIndicatorAttributes,
    pub is_visible: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Render props passed to `render_child` on `NavigationMenuViewport`.
#[derive(Clone)]
pub struct NavigationMenuViewportRenderProps {
    pub attrs: NavigationMenuViewportAttributes,
    pub is_open: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FocusTrackerState {
    pub prev_focused_id: Option<String>,
    pub prev_focused_path: Vec<String>,
    pub current_focused_id: Option<String>,
    pub current_focused_path: Vec<String>,
}

impl FocusTrackerState {
    pub fn record_focus_in(&mut self, target_id: Option<String>, path_ids: Vec<String>) {
        self.prev_focused_id = self.current_focused_id.take();
        self.prev_focused_path = std::mem::take(&mut self.current_focused_path);
        self.current_focused_id = target_id;
        self.current_focused_path = path_ids;
    }

    pub fn ensure_focused(&mut self, element_id: &str, ancestor_ids: &[String]) {
        if self.current_focused_id.as_deref() == Some(element_id) {
            return;
        }
        let mut path = Vec::with_capacity(1 + ancestor_ids.len());
        path.push(element_id.to_string());
        for id in ancestor_ids {
            if id != element_id {
                path.push(id.clone());
            }
        }
        self.record_focus_in(Some(element_id.to_string()), path);
    }

    pub fn prev_within_any(&self, ids: &[String]) -> bool {
        if let Some(ref prev_id) = self.prev_focused_id
            && ids.iter().any(|id| id == prev_id)
        {
            return true;
        }
        self.prev_focused_path
            .iter()
            .any(|pid| ids.iter().any(|id| id == pid))
    }

    pub fn current_within(&self, target_id: &str) -> bool {
        self.current_focused_id.as_deref() == Some(target_id)
            || self.current_focused_path.iter().any(|id| id == target_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredTrigger {
    pub value: String,
    pub dom_id: String,
    pub open_on_hover: Option<bool>,
}

#[derive(Clone, PartialEq)]
pub struct ViewportContentData {
    pub value: String,
    pub content_id: String,
    pub trigger_id: String,
    pub focus_proxy_id: String,
    pub item_state: NavigationMenuItemContext,
    pub force_mount: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub as_child: bool,
    pub render_child: Option<Callback<NavigationMenuContentRenderProps, Element>>,
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    pub on_focus_outside: Option<EventHandler<FocusOutsideEvent>>,
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    pub escape_keydown_behavior: NavigationMenuOutsideBehavior,
    pub interact_outside_behavior: NavigationMenuOutsideBehavior,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq)]
pub struct IndicatorMountData {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub force_mount: bool,
    pub as_child: bool,
    pub render_child: Option<Callback<NavigationMenuIndicatorRenderProps, Element>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, Default, PartialEq)]
pub struct ContentDismissConfig {
    pub value: String,
    pub content_id: String,
    pub trigger_id: String,
    pub item_state: Option<NavigationMenuItemContext>,
    pub escape_keydown_behavior: NavigationMenuOutsideBehavior,
    pub interact_outside_behavior: NavigationMenuOutsideBehavior,
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    pub on_focus_outside: Option<EventHandler<FocusOutsideEvent>>,
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
}

/// Marker context distinguishing whether a `NavigationMenuLink` is inside `NavigationMenuContent`
/// or directly inside `NavigationMenuList`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavigationMenuInContent(pub bool);

/// Marker context indicating a `NavigationMenuRoot` content dismiss handler for nested submenus.
#[derive(Clone, Copy, PartialEq)]
pub struct NavigationMenuRootDismissContext {
    pub on_dismiss: Callback<()>,
}

/// Per-item runtime context shared among `Item`, `Trigger`, and `Content`.
#[derive(Clone, Copy, PartialEq)]
pub struct NavigationMenuItemContext {
    pub value: Signal<String>,
    pub open_on_hover: Signal<Option<bool>>,
    pub trigger_id: Signal<Option<String>>,
    pub content_id: Signal<Option<String>>,
    pub focus_proxy_id: Signal<String>,
    pub was_escape_close: Signal<bool>,
}

impl NavigationMenuItemContext {
    pub fn value_str(&self) -> String {
        self.value.read().clone()
    }

    pub fn open_on_hover_opt(&self) -> Option<bool> {
        *self.open_on_hover.read()
    }

    pub fn effective_trigger_id(&self, rels: &NavigationMenuRelationships) -> String {
        self.trigger_id
            .read()
            .clone()
            .unwrap_or_else(|| rels.trigger_id(&self.value.read()))
    }

    pub fn effective_content_id(&self, rels: &NavigationMenuRelationships) -> String {
        self.content_id
            .read()
            .clone()
            .unwrap_or_else(|| rels.content_id(&self.value.read()))
    }

    pub fn set_was_escape_close(&self, flag: bool) {
        let mut sig = self.was_escape_close;
        if sig.try_peek().is_ok_and(|cur| *cur != flag)
            && let Ok(mut w) = sig.try_write()
        {
            *w = flag;
        }
    }

    pub fn was_escape_close(&self) -> bool {
        self.was_escape_close
            .try_peek()
            .map(|v| *v)
            .unwrap_or(false)
    }

    pub fn on_entry_key_down(&self, rels: &NavigationMenuRelationships) {
        let content_id = self.effective_content_id(rels);
        focus_content_candidates(&content_id, NavigationMenuFocusProxySide::Start);
    }

    pub fn on_focus_proxy_enter(
        &self,
        rels: &NavigationMenuRelationships,
        side: NavigationMenuFocusProxySide,
    ) {
        let content_id = self.effective_content_id(rels);
        focus_content_candidates(&content_id, side);
    }

    pub fn on_root_content_close(&self, rels: &NavigationMenuRelationships) {
        let content_id = self.effective_content_id(rels);
        remove_from_tab_order(&content_id);
    }

    pub fn on_content_focus_outside(&self, rels: &NavigationMenuRelationships) {
        let content_id = self.effective_content_id(rels);
        remove_from_tab_order(&content_id);
    }
}

/// Shared runtime for `NavigationMenuRoot` and `NavigationMenuSub`.
#[derive(Clone)]
pub struct NavigationMenuContext {
    pub is_root_menu: bool,
    pub root_menu_id: Signal<String>,
    pub menu_dom_id: Signal<String>,
    pub relationships: NavigationMenuRelationships,
    pub value: Signal<String>,
    pub previous_value: Signal<String>,
    pub tracked_value: Signal<String>,
    pub on_value_change: CopyValue<Option<EventHandler<String>>>,
    pub dir: Signal<Direction>,
    pub orientation: Signal<Orientation>,
    pub activation_mode: Signal<NavigationMenuActivationMode>,
    pub disable_toggle: Signal<bool>,
    pub aria_label: Signal<Option<String>>,
    pub hover_machine: Option<Signal<NavigationMenuHoverMachine>>,
    pub alive: Rc<Cell<bool>>,
    pub focus_tracker: Rc<RefCell<FocusTrackerState>>,
    pub viewport_id: Signal<Option<String>>,
    pub viewport_contents: Signal<Vec<(String, ViewportContentData)>>,
    pub indicator_config: Signal<Option<IndicatorMountData>>,
    pub track_id: Signal<Option<String>>,
    pub triggers: Signal<Vec<RegisteredTrigger>>,
    pub contents: Signal<Vec<(String, String)>>,
    pub active_content_config: CopyValue<Option<ContentDismissConfig>>,
    pub branches: Signal<Vec<String>>,
    pub roving: RovingFocusRuntime,
    pub dismiss_layer: Option<DismissLayerHandle>,
    pub(crate) focus_capture_monitor: Option<WatcherSlot>,
    pub parent_root_dismiss: Option<Callback<()>>,
}

#[allow(clippy::too_many_arguments)]
pub fn use_navigation_menu_root_runtime(
    scope: ScopeHandle,
    dom_id: String,
    value_signal: Signal<String>,
    on_value_change: Option<EventHandler<String>>,
    delay_duration: u64,
    skip_delay_duration: u64,
    dir: Direction,
    orientation: Orientation,
    activation_mode: NavigationMenuActivationMode,
    aria_label: Option<String>,
) -> NavigationMenuContext {
    let relationships = NavigationMenuRelationships::new(scope.clone());
    let initial_val = value_signal();
    let def = NavigationMenuDef::new(scope)
        .with_is_root_menu(true)
        .with_value(initial_val.clone())
        .with_orientation(orientation)
        .with_dir(dir)
        .with_activation_mode(activation_mode)
        .with_delay_duration_ms(delay_duration)
        .with_skip_delay_duration_ms(skip_delay_duration)
        .with_aria_label(aria_label.clone());

    let mut initial_machine = NavigationMenuHoverMachine::new(
        delay_duration,
        skip_delay_duration,
        NAVIGATION_MENU_CLOSE_GRACE_MS,
    );
    if !initial_val.is_empty() {
        let _ = initial_machine.dispatch(NavigationMenuHoverAction::ValueCommitted {
            next_value: initial_val.clone(),
        });
    }

    let roving = RovingFocusRuntime::new(def.list_roving_config(), None, relationships.list_id());
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let focus_tracker = use_hook(|| Rc::new(RefCell::new(FocusTrackerState::default())));
    let root_menu_id = use_signal(|| dom_id.clone());
    let menu_dom_id = use_signal(|| dom_id.clone());
    let previous_value = use_signal(String::new);
    let tracked_value = use_signal(|| initial_val.clone());
    let on_value_change_slot = use_hook(|| CopyValue::new(on_value_change));
    let dir_sig = use_signal(|| def.dir());
    let orientation_sig = use_signal(|| def.orientation());
    let activation_mode_sig = use_signal(|| def.activation_mode());
    let disable_toggle = use_signal(|| false);
    let aria_label_sig = use_signal(|| aria_label);
    let hover_machine = Some(use_signal(|| initial_machine));
    let viewport_id = use_signal(|| None::<String>);
    let viewport_contents = use_signal(Vec::new);
    let indicator_config = use_signal(|| None);
    let track_id = use_signal(|| None);
    let triggers = use_signal(Vec::<RegisteredTrigger>::new);
    let contents = use_signal(Vec::<(String, String)>::new);
    let active_content_config = use_hook(|| CopyValue::new(Option::<ContentDismissConfig>::None));
    let branches = use_signal(Vec::<String>::new);
    let focus_capture_monitor = use_watcher_slot();

    // Start root capture-phase FocusIn observer once on mount using minimal mask.
    let tracker_for_capture = Rc::clone(&focus_tracker);
    use_hook(move || {
        let watcher = observe_capture_events_with_mask(CaptureEventMask::FOCUS_IN, move |event| {
            if matches!(event.kind, CaptureEventKind::FocusIn)
                && let Ok(mut ft) = tracker_for_capture.try_borrow_mut()
            {
                ft.record_focus_in(event.target_id, event.path_ids);
            }
        })
        .unwrap_or_else(|_| WatcherGuard::noop());
        focus_capture_monitor.replace(watcher);
    });

    let mut base_ctx = NavigationMenuContext {
        is_root_menu: true,
        root_menu_id,
        menu_dom_id,
        relationships,
        value: value_signal,
        previous_value,
        tracked_value,
        on_value_change: on_value_change_slot,
        dir: dir_sig,
        orientation: orientation_sig,
        activation_mode: activation_mode_sig,
        disable_toggle,
        aria_label: aria_label_sig,
        hover_machine,
        alive: Rc::clone(&alive),
        focus_tracker: Rc::clone(&focus_tracker),
        viewport_id,
        viewport_contents,
        indicator_config,
        track_id,
        triggers,
        contents,
        active_content_config,
        branches,
        roving,
        dismiss_layer: None,
        focus_capture_monitor: Some(focus_capture_monitor),
        parent_root_dismiss: None,
    };

    let is_open = !value_signal.read().is_empty();
    let mut boundary_ids: Vec<String> = Vec::new();
    for t in triggers.read().iter() {
        if !boundary_ids.contains(&t.dom_id) {
            boundary_ids.push(t.dom_id.clone());
        }
    }
    if let Some(ref vp_id) = *viewport_id.read()
        && !boundary_ids.contains(vp_id)
    {
        boundary_ids.push(vp_id.clone());
    }
    for (_, cid) in contents.read().iter() {
        if !boundary_ids.contains(cid) {
            boundary_ids.push(cid.clone());
        }
    }
    for b in branches.read().iter() {
        if !boundary_ids.contains(b) {
            boundary_ids.push(b.clone());
        }
    }

    let dismiss_config = DismissConfig {
        layer_id: dom_id,
        boundaries: boundary_ids,
        modal: false,
        disable_outside_pointer_events: false,
        listen_escape: true,
        listen_pointer_down_outside: true,
        listen_focus_outside: true,
    };

    let ctx_for_callbacks = base_ctx.clone();
    let dismiss_callbacks = use_hook(move || {
        let ctx_esc = ctx_for_callbacks.clone();
        let on_escape_key_down = EventHandler::new(move |evt: EscapeKeyDownEvent| {
            if !ctx_esc.is_alive()
                || ctx_esc
                    .value
                    .try_peek()
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            {
                evt.prevent_default();
                return;
            }
            let cfg = ctx_esc
                .active_content_config
                .try_read()
                .ok()
                .and_then(|c| c.clone())
                .unwrap_or_default();
            if let Some(handler) = cfg.on_escape_key_down {
                handler.call(evt.clone());
            }
            if !cfg.escape_keydown_behavior.should_close() {
                evt.prevent_default();
            }
        });

        let ctx_ptr = ctx_for_callbacks.clone();
        let on_pointer_down_outside = EventHandler::new(move |evt: PointerDownOutsideEvent| {
            if !ctx_ptr.is_alive()
                || ctx_ptr
                    .value
                    .try_peek()
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            {
                evt.prevent_default();
                return;
            }
            let cfg = ctx_ptr
                .active_content_config
                .try_read()
                .ok()
                .and_then(|c| c.clone())
                .unwrap_or_default();
            if let Some(handler) = cfg.on_pointer_down_outside {
                handler.call(evt.clone());
            }
            if let Some(handler) = cfg.on_interact_outside {
                handler.call(evt.clone());
            }
            if !cfg.interact_outside_behavior.should_close() {
                evt.prevent_default();
            }
        });

        let ctx_focus = ctx_for_callbacks.clone();
        let on_focus_outside = EventHandler::new(move |evt: FocusOutsideEvent| {
            if !ctx_focus.is_alive()
                || ctx_focus
                    .value
                    .try_peek()
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            {
                evt.prevent_default();
                return;
            }
            let cfg = ctx_focus
                .active_content_config
                .try_read()
                .ok()
                .and_then(|c| c.clone())
                .unwrap_or_default();

            let mut menu_ids = vec![
                ctx_focus
                    .root_menu_id
                    .try_peek()
                    .map(|s| s.clone())
                    .unwrap_or_default(),
            ];
            if let Ok(vp_guard) = ctx_focus.viewport_id.try_peek()
                && let Some(ref vp_id) = *vp_guard
            {
                menu_ids.push(vp_id.clone());
            }
            if let Ok(c_list) = ctx_focus.contents.try_peek() {
                for (_, cid) in c_list.iter() {
                    menu_ids.push(cid.clone());
                }
            }
            if let Ok(b_list) = ctx_focus.branches.try_peek() {
                for b in b_list.iter() {
                    menu_ids.push(b.clone());
                }
            }

            let focus_came_from_menu = ctx_focus
                .focus_tracker
                .try_borrow()
                .is_ok_and(|ft| ft.prev_within_any(&menu_ids));

            if !cfg.content_id.is_empty() {
                remove_from_tab_order(&cfg.content_id);
            }

            if !should_dismiss_on_focus_outside(false, focus_came_from_menu) {
                evt.prevent_default();
                return;
            }

            if let Some(handler) = cfg.on_focus_outside {
                handler.call(evt.clone());
            }
            if let Some(handler) = cfg.on_interact_outside {
                handler.call(evt.clone());
            }
            if !cfg.interact_outside_behavior.should_close() {
                evt.prevent_default();
            }
        });

        let ctx_dismiss = ctx_for_callbacks;
        let on_dismiss = EventHandler::new(move |event: DismissEvent| {
            if !ctx_dismiss.is_alive()
                || ctx_dismiss
                    .value
                    .try_peek()
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            {
                return;
            }
            let cfg = ctx_dismiss
                .active_content_config
                .try_read()
                .ok()
                .and_then(|c| c.clone())
                .unwrap_or_default();
            match event {
                DismissEvent::Escape => {
                    if let Some(ref item_state) = cfg.item_state {
                        item_state.set_was_escape_close(true);
                    }
                    let was_inside_content = !cfg.content_id.is_empty()
                        && (ctx_dismiss
                            .focus_tracker
                            .try_borrow()
                            .is_ok_and(|ft| ft.current_within(&cfg.content_id))
                            || is_active_element_inside(&cfg.content_id));
                    if !cfg.content_id.is_empty() {
                        remove_from_tab_order(&cfg.content_id);
                    }
                    ctx_dismiss.on_item_dismiss();
                    if was_inside_content && !cfg.trigger_id.is_empty() {
                        restore_focus_element_by_id_with_options(
                            &cfg.trigger_id,
                            FocusOptions::with_focus_visible(true, Some(true)),
                        );
                    }
                }
                DismissEvent::PointerDownOutside => {
                    if !cfg.content_id.is_empty() {
                        remove_from_tab_order(&cfg.content_id);
                    }
                    ctx_dismiss.on_item_dismiss();
                }
                DismissEvent::FocusOutside => {
                    ctx_dismiss.on_item_dismiss();
                }
            }
        });

        DismissCallbacks {
            on_escape_key_down: Some(on_escape_key_down),
            on_pointer_down_outside: Some(on_pointer_down_outside),
            on_focus_outside: Some(on_focus_outside),
            on_interact_outside: None,
            on_dismiss: Some(on_dismiss),
        }
    });

    let dismiss_layer = use_dismissable_layer(is_open, dismiss_config, dismiss_callbacks);
    base_ctx.dismiss_layer = Some(dismiss_layer);

    let cleanup_ctx = base_ctx.clone();
    let cleanup_alive = Rc::clone(&alive);
    use_drop(move || {
        cleanup_alive.set(false);
        if let Some(mut hm) = cleanup_ctx.hover_machine
            && let Ok(mut m) = hm.try_write()
        {
            let _ = m.cancel_all_timers();
        }
        if let Some(slot) = cleanup_ctx.focus_capture_monitor {
            slot.clear();
        }
    });

    base_ctx
}

#[allow(clippy::too_many_arguments)]
pub fn use_navigation_menu_sub_runtime(
    parent: &NavigationMenuContext,
    scope: ScopeHandle,
    dom_id: String,
    value_signal: Signal<String>,
    on_value_change: Option<EventHandler<String>>,
    orientation: Orientation,
    activation_mode: Option<NavigationMenuActivationMode>,
    disable_toggle: bool,
    parent_root_dismiss: Option<Callback<()>>,
) -> NavigationMenuContext {
    let relationships = NavigationMenuRelationships::new(scope.clone());
    let inherited_dir = (parent.dir)();
    let inherited_mode = (parent.activation_mode)();
    let def = NavigationMenuSubDef::new(scope, inherited_dir, inherited_mode)
        .with_orientation(orientation)
        .with_activation_mode(activation_mode)
        .with_disable_toggle(disable_toggle)
        .with_value(value_signal())
        .into_def();

    let roving = RovingFocusRuntime::new(def.list_roving_config(), None, relationships.list_id());
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let initial_val = value_signal();

    let ctx = NavigationMenuContext {
        is_root_menu: false,
        root_menu_id: parent.root_menu_id,
        menu_dom_id: use_signal(|| dom_id),
        relationships,
        value: value_signal,
        previous_value: use_signal(String::new),
        tracked_value: use_signal(|| initial_val),
        on_value_change: use_hook(|| CopyValue::new(on_value_change)),
        dir: use_signal(|| def.dir()),
        orientation: use_signal(|| def.orientation()),
        activation_mode: use_signal(|| def.activation_mode()),
        disable_toggle: use_signal(|| def.disable_toggle()),
        aria_label: use_signal(|| None),
        hover_machine: None,
        alive: Rc::clone(&alive),
        focus_tracker: Rc::clone(&parent.focus_tracker),
        viewport_id: use_signal(|| None),
        viewport_contents: use_signal(Vec::new),
        indicator_config: use_signal(|| None),
        track_id: use_signal(|| None),
        triggers: use_signal(Vec::new),
        contents: use_signal(Vec::new),
        active_content_config: use_hook(|| CopyValue::new(None)),
        branches: use_signal(Vec::new),
        roving,
        dismiss_layer: None,
        focus_capture_monitor: None,
        parent_root_dismiss,
    };

    let cleanup_alive = Rc::clone(&alive);
    use_drop(move || {
        cleanup_alive.set(false);
    });

    ctx
}

impl NavigationMenuContext {
    pub fn is_alive(&self) -> bool {
        self.alive.get() && self.value.try_peek().is_ok()
    }

    pub fn current_value(&self) -> String {
        self.value.read().clone()
    }

    pub fn previous_value_str(&self) -> String {
        self.previous_value.read().clone()
    }

    pub fn current_dir(&self) -> Direction {
        *self.dir.read()
    }

    pub fn current_orientation(&self) -> Orientation {
        *self.orientation.read()
    }

    pub fn current_activation_mode(&self) -> NavigationMenuActivationMode {
        *self.activation_mode.read()
    }

    pub fn has_viewport(&self) -> bool {
        self.viewport_id.read().is_some()
    }

    pub fn root_dom_id(&self) -> String {
        self.root_menu_id.read().clone()
    }

    pub fn menu_id(&self) -> String {
        self.menu_dom_id.read().clone()
    }

    pub fn def(&self) -> NavigationMenuDef {
        NavigationMenuDef::new(self.relationships.scope().clone())
            .with_is_root_menu(self.is_root_menu)
            .with_value(self.current_value())
            .with_previous_value(self.previous_value_str())
            .with_orientation(self.current_orientation())
            .with_dir(self.current_dir())
            .with_activation_mode(self.current_activation_mode())
            .with_disable_toggle(*self.disable_toggle.read())
            .with_aria_label(self.aria_label.read().clone())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sync_root_props(
        &self,
        dom_id: &str,
        on_value_change: Option<EventHandler<String>>,
        delay_duration: u64,
        skip_delay_duration: u64,
        dir: Direction,
        orientation: Orientation,
        activation_mode: NavigationMenuActivationMode,
        aria_label: Option<String>,
    ) {
        update_signal_if_changed(self.root_menu_id, dom_id.to_string());
        update_signal_if_changed(self.menu_dom_id, dom_id.to_string());
        let mut on_change_slot = self.on_value_change;
        if let Ok(mut w) = on_change_slot.try_write() {
            *w = on_value_change;
        }
        update_signal_if_changed(self.dir, dir);
        update_signal_if_changed(self.orientation, orientation);
        update_signal_if_changed(self.activation_mode, activation_mode);
        update_signal_if_changed(self.aria_label, aria_label);

        if let Some(mut hm) = self.hover_machine
            && let Ok(mut m) = hm.try_write()
        {
            m.update_durations(delay_duration, skip_delay_duration);
        }

        let current_val = self.value.try_peek().map(|v| v.clone()).unwrap_or_default();
        self.sync_external_value(current_val);
        self.roving.set_direction(dir);
        self.roving.set_orientation(orientation);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sync_sub_props(
        &self,
        dom_id: &str,
        on_value_change: Option<EventHandler<String>>,
        dir: Direction,
        orientation: Orientation,
        activation_mode: NavigationMenuActivationMode,
        disable_toggle: bool,
    ) {
        update_signal_if_changed(self.menu_dom_id, dom_id.to_string());
        let mut on_change_slot = self.on_value_change;
        if let Ok(mut w) = on_change_slot.try_write() {
            *w = on_value_change;
        }
        update_signal_if_changed(self.dir, dir);
        update_signal_if_changed(self.orientation, orientation);
        update_signal_if_changed(self.activation_mode, activation_mode);
        update_signal_if_changed(self.disable_toggle, disable_toggle);

        let current_val = self.value.try_peek().map(|v| v.clone()).unwrap_or_default();
        self.sync_external_value(current_val);
        self.roving.set_direction(dir);
        self.roving.set_orientation(orientation);
    }

    fn sync_external_value(&self, current_val: String) {
        let tracked = self
            .tracked_value
            .try_peek()
            .map(|v| v.clone())
            .unwrap_or_default();
        if tracked == current_val {
            return;
        }
        let mut prev_sig = self.previous_value;
        if let Ok(mut prev) = prev_sig.try_write() {
            *prev = tracked;
        }
        let mut tracked_sig = self.tracked_value;
        if let Ok(mut tr) = tracked_sig.try_write() {
            *tr = current_val.clone();
        }
        if let Some(mut hm) = self.hover_machine {
            let cmds = if let Ok(mut m) = hm.try_write() {
                m.dispatch(NavigationMenuHoverAction::ValueCommitted {
                    next_value: current_val,
                })
                .commands
            } else {
                Vec::new()
            };
            self.run_timer_commands(cmds);
        }
    }

    pub fn set_value(&self, next_value: String) {
        if !self.is_alive() {
            return;
        }
        let current = self
            .value
            .try_peek()
            .map(|v| v.clone())
            .unwrap_or_default();
        if current == next_value {
            return;
        }
        let mut prev_sig = self.previous_value;
        if let Ok(mut prev) = prev_sig.try_write() {
            *prev = current;
        }
        let mut tracked_sig = self.tracked_value;
        if let Ok(mut tr) = tracked_sig.try_write() {
            *tr = next_value.clone();
        }
        let mut val_sig = self.value;
        if let Ok(mut w) = val_sig.try_write() {
            *w = next_value.clone();
        }
        if let Some(mut hm) = self.hover_machine {
            let cmds = if let Ok(mut m) = hm.try_write() {
                m.dispatch(NavigationMenuHoverAction::ValueCommitted {
                    next_value: next_value.clone(),
                })
                .commands
            } else {
                Vec::new()
            };
            self.run_timer_commands(cmds);
        }
        if let Some(handler) = self.on_value_change.try_read().ok().and_then(|g| *g) {
            handler.call(next_value);
        }
    }

    pub fn on_trigger_enter(&self, item_value: &str, item_open_on_hover: Option<bool>) {
        let mode = self
            .activation_mode
            .try_peek()
            .map(|m| *m)
            .unwrap_or_default();
        if self.is_root_menu {
            self.dispatch_hover(NavigationMenuHoverAction::TriggerEnter {
                item_value: item_value.to_string(),
                activation_mode: mode,
                item_open_on_hover,
            });
        } else if is_item_hover_enabled(mode, item_open_on_hover) {
            self.set_value(item_value.to_string());
        }
    }

    pub fn on_trigger_leave(&self, item_open_on_hover: Option<bool>) {
        if !self.is_root_menu {
            return;
        }
        let mode = self
            .activation_mode
            .try_peek()
            .map(|m| *m)
            .unwrap_or_default();
        self.dispatch_hover(NavigationMenuHoverAction::TriggerLeave {
            activation_mode: mode,
            item_open_on_hover,
        });
    }

    pub fn on_content_enter(&self) {
        if !self.is_root_menu {
            return;
        }
        let mode = self
            .activation_mode
            .try_peek()
            .map(|m| *m)
            .unwrap_or_default();
        self.dispatch_hover(NavigationMenuHoverAction::ContentEnter {
            activation_mode: mode,
        });
    }

    pub fn on_content_leave(&self, item_open_on_hover: Option<bool>) {
        if !self.is_root_menu {
            return;
        }
        let mode = self
            .activation_mode
            .try_peek()
            .map(|m| *m)
            .unwrap_or_default();
        self.dispatch_hover(NavigationMenuHoverAction::ContentLeave {
            activation_mode: mode,
            item_open_on_hover,
        });
    }

    pub fn on_item_select(&self, item_value: &str) {
        if let Some(mut hm) = self.hover_machine
            && let Ok(mut m) = hm.try_write()
        {
            let _ = m.cancel_all_timers();
        }
        if item_value.is_empty() {
            self.set_value(String::new());
            return;
        }
        let current = self
            .value
            .try_peek()
            .map(|v| v.clone())
            .unwrap_or_default();
        let disable_toggle = self
            .disable_toggle
            .try_peek()
            .map(|d| *d)
            .unwrap_or(false);
        let next = if self.is_root_menu {
            if current == item_value {
                String::new()
            } else {
                item_value.to_string()
            }
        } else if disable_toggle || current != item_value {
            item_value.to_string()
        } else {
            String::new()
        };
        self.set_value(next);
    }

    pub fn on_item_dismiss(&self) {
        if let Some(mut hm) = self.hover_machine
            && let Ok(mut m) = hm.try_write()
        {
            let _ = m.cancel_all_timers();
        }
        self.set_value(String::new());
    }

    pub fn dismiss_root(&self) {
        if self.is_root_menu {
            self.on_item_dismiss();
        } else if let Some(ref parent_cb) = self.parent_root_dismiss {
            parent_cb.call(());
        } else {
            self.on_item_dismiss();
        }
    }

    pub fn active_item_open_on_hover(&self) -> bool {
        let current = self
            .value
            .try_peek()
            .map(|v| v.clone())
            .unwrap_or_default();
        if current.is_empty() {
            return false;
        }
        let mode = self
            .activation_mode
            .try_peek()
            .map(|m| *m)
            .unwrap_or_default();
        let item_override = self.triggers.try_peek().ok().and_then(|list| {
            list.iter()
                .find(|t| t.value == current)
                .and_then(|t| t.open_on_hover)
        });
        is_item_hover_enabled(mode, item_override)
    }

    pub fn active_item_open_on_hover_opt(&self) -> Option<bool> {
        let current = self
            .value
            .try_peek()
            .map(|v| v.clone())
            .unwrap_or_default();
        if current.is_empty() {
            return None;
        }
        self.triggers.try_peek().ok().and_then(|list| {
            list.iter()
                .find(|t| t.value == current)
                .and_then(|t| t.open_on_hover)
        })
    }

    fn dispatch_hover(&self, action: NavigationMenuHoverAction) {
        let Some(mut hm) = self.hover_machine else {
            return;
        };
        let transition = {
            let Ok(mut machine) = hm.try_write() else {
                return;
            };
            machine.dispatch(action)
        };
        self.run_timer_commands(transition.commands);
        if let Some(next_value) = transition.next_value {
            self.set_value(next_value);
        }
    }

    fn run_timer_commands(&self, commands: Vec<NavigationMenuTimerCommand>) {
        for cmd in commands {
            match cmd {
                NavigationMenuTimerCommand::StartOpenTimer {
                    item_value,
                    delay_ms,
                    generation,
                } => {
                    let ctx = self.clone();
                    spawn_root_task(async move {
                        Delay::new(Duration::from_millis(delay_ms)).await;
                        if !ctx.is_alive() {
                            return;
                        }
                        ctx.dispatch_hover(NavigationMenuHoverAction::OpenTimerFired {
                            item_value,
                            generation,
                        });
                    });
                }
                NavigationMenuTimerCommand::StartCloseTimer {
                    delay_ms,
                    generation,
                } => {
                    let ctx = self.clone();
                    spawn_root_task(async move {
                        Delay::new(Duration::from_millis(delay_ms)).await;
                        if !ctx.is_alive() {
                            return;
                        }
                        ctx.dispatch_hover(NavigationMenuHoverAction::CloseTimerFired {
                            generation,
                        });
                    });
                }
                NavigationMenuTimerCommand::StartSkipDelayTimer {
                    delay_ms,
                    generation,
                } => {
                    let ctx = self.clone();
                    spawn_root_task(async move {
                        Delay::new(Duration::from_millis(delay_ms)).await;
                        if !ctx.is_alive() {
                            return;
                        }
                        ctx.dispatch_hover(NavigationMenuHoverAction::SkipDelayTimerFired {
                            generation,
                        });
                    });
                }
                NavigationMenuTimerCommand::CancelOpenTimer
                | NavigationMenuTimerCommand::CancelCloseTimer
                | NavigationMenuTimerCommand::CancelSkipDelayTimer => {}
            }
        }
    }

    pub fn register_viewport(&self, viewport_id: String) {
        let already = self
            .viewport_id
            .try_peek()
            .is_ok_and(|cur| cur.as_deref() == Some(viewport_id.as_str()));
        if !already {
            let mut sig = self.viewport_id;
            if let Ok(mut w) = sig.try_write() {
                *w = Some(viewport_id);
            }
        }
    }

    pub fn unregister_viewport(&self, viewport_id: &str) {
        let matches = self
            .viewport_id
            .try_peek()
            .is_ok_and(|cur| cur.as_deref() == Some(viewport_id));
        if matches {
            let mut sig = self.viewport_id;
            if let Ok(mut w) = sig.try_write() {
                *w = None;
            }
        }
    }

    pub fn register_track_id(&self, track_id: String) {
        let already = self
            .track_id
            .try_peek()
            .is_ok_and(|cur| cur.as_deref() == Some(track_id.as_str()));
        if !already {
            let mut sig = self.track_id;
            if let Ok(mut w) = sig.try_write() {
                *w = Some(track_id);
            }
        }
    }

    pub fn unregister_track_id(&self, track_id: &str) {
        let matches = self
            .track_id
            .try_peek()
            .is_ok_and(|cur| cur.as_deref() == Some(track_id));
        if matches {
            let mut sig = self.track_id;
            if let Ok(mut w) = sig.try_write() {
                *w = None;
            }
        }
    }

    pub fn register_viewport_content(&self, entry: ViewportContentData) {
        let unchanged = self
            .viewport_contents
            .try_peek()
            .is_ok_and(|list| list.iter().any(|(v, e)| v == &entry.value && e == &entry));
        if unchanged {
            return;
        }
        let mut sig = self.viewport_contents;
        if let Ok(mut list) = sig.try_write() {
            if let Some(existing) = list.iter_mut().find(|(v, _)| v == &entry.value) {
                existing.1 = entry;
            } else {
                list.push((entry.value.clone(), entry));
            }
        }
    }

    pub fn unregister_viewport_content(&self, value: &str) {
        let exists = self
            .viewport_contents
            .try_peek()
            .is_ok_and(|list| list.iter().any(|(v, _)| v == value));
        if !exists {
            return;
        }
        let mut sig = self.viewport_contents;
        if let Ok(mut list) = sig.try_write() {
            list.retain(|(v, _)| v != value);
        }
    }

    pub fn register_indicator(&self, data: IndicatorMountData) {
        let unchanged = self
            .indicator_config
            .try_peek()
            .is_ok_and(|cur| cur.as_ref() == Some(&data));
        if unchanged {
            return;
        }
        let mut sig = self.indicator_config;
        if let Ok(mut w) = sig.try_write() {
            *w = Some(data);
        }
    }

    pub fn unregister_indicator(&self) {
        let has_data = self
            .indicator_config
            .try_peek()
            .is_ok_and(|cur| cur.is_some());
        if !has_data {
            return;
        }
        let mut sig = self.indicator_config;
        if let Ok(mut w) = sig.try_write() {
            *w = None;
        }
    }

    pub fn register_trigger(&self, value: &str, dom_id: &str, open_on_hover: Option<bool>) {
        let next = RegisteredTrigger {
            value: value.to_string(),
            dom_id: dom_id.to_string(),
            open_on_hover,
        };
        let unchanged = self
            .triggers
            .try_peek()
            .is_ok_and(|list| list.iter().any(|t| t == &next));
        if unchanged {
            return;
        }
        let mut sig = self.triggers;
        if let Ok(mut list) = sig.try_write() {
            if let Some(existing) = list.iter_mut().find(|t| t.value == value) {
                *existing = next;
            } else {
                list.push(next);
            }
        }
    }

    pub fn unregister_trigger(&self, value: &str) {
        let exists = self
            .triggers
            .try_peek()
            .is_ok_and(|list| list.iter().any(|t| t.value == value));
        if !exists {
            return;
        }
        let mut sig = self.triggers;
        if let Ok(mut list) = sig.try_write() {
            list.retain(|t| t.value != value);
        }
    }

    pub fn register_content_id(&self, value: &str, content_id: &str) {
        let unchanged = self
            .contents
            .try_peek()
            .is_ok_and(|list| list.iter().any(|(v, id)| v == value && id == content_id));
        if unchanged {
            return;
        }
        let mut sig = self.contents;
        if let Ok(mut list) = sig.try_write() {
            if let Some(existing) = list.iter_mut().find(|(v, _)| v == value) {
                existing.1 = content_id.to_string();
            } else {
                list.push((value.to_string(), content_id.to_string()));
            }
        }
    }

    pub fn unregister_content_id(&self, value: &str) {
        let exists = self
            .contents
            .try_peek()
            .is_ok_and(|list| list.iter().any(|(v, _)| v == value));
        if !exists {
            return;
        }
        let mut sig = self.contents;
        if let Ok(mut list) = sig.try_write() {
            list.retain(|(v, _)| v != value);
        }
    }

    pub fn set_active_content_config(&self, config: Option<ContentDismissConfig>) {
        let mut slot = self.active_content_config;
        if let Ok(mut w) = slot.try_write() {
            *w = config;
        }
    }

    pub fn register_branch(&self, branch: String) {
        if let Some(dl) = self.dismiss_layer {
            dl.register_branch(branch.clone());
        }
        let already = self
            .branches
            .try_peek()
            .is_ok_and(|list| list.iter().any(|b| b == &branch));
        if already {
            return;
        }
        let mut sig = self.branches;
        if let Ok(mut w) = sig.try_write() {
            w.push(branch);
        }
    }

    pub fn unregister_branch(&self, branch: &str) {
        if let Some(dl) = self.dismiss_layer {
            dl.unregister_branch(branch);
        }
        let exists = self
            .branches
            .try_peek()
            .is_ok_and(|list| list.iter().any(|b| b == branch));
        if !exists {
            return;
        }
        let mut sig = self.branches;
        if let Ok(mut w) = sig.try_write() {
            w.retain(|b| b != branch);
        }
    }

    pub fn trigger_id_for(&self, value: &str) -> Option<String> {
        self.triggers
            .read()
            .iter()
            .find(|t| t.value == value)
            .map(|t| t.dom_id.clone())
    }

    pub fn ordered_trigger_values(&self) -> Vec<String> {
        let list = self.triggers.read().clone();
        if list.len() <= 1 {
            return list.into_iter().map(|t| t.value).collect();
        }
        let root_id = self.roving.root_dom_id.peek().clone();
        if !root_id.is_empty() {
            let id_refs: Vec<&str> = list.iter().map(|t| t.dom_id.as_str()).collect();
            if let Ok(ordered_ids) = query_descendant_order(&root_id, &id_refs)
                && !ordered_ids.is_empty()
            {
                let mut ordered_values = Vec::with_capacity(list.len());
                for dom_id in ordered_ids {
                    if let Some(entry) = list.iter().find(|t| t.dom_id == dom_id) {
                        ordered_values.push(entry.value.clone());
                    }
                }
                for entry in &list {
                    if !ordered_values.contains(&entry.value) {
                        ordered_values.push(entry.value.clone());
                    }
                }
                return ordered_values;
            }
        }
        list.into_iter().map(|t| t.value).collect()
    }

    pub fn motion_for_item(
        &self,
        item_value: &str,
        prev_motion: Option<NavigationMenuMotion>,
    ) -> Option<NavigationMenuMotion> {
        let ordered = self.ordered_trigger_values();
        compute_motion_direction(
            &ordered,
            self.current_dir(),
            &self.current_value(),
            &self.previous_value_str(),
            item_value,
            prev_motion,
        )
    }

    pub fn record_element_focus(&self, element_id: &str) {
        let root_id = self.root_dom_id();
        if let Ok(mut ft) = self.focus_tracker.try_borrow_mut() {
            ft.ensure_focused(element_id, &[root_id]);
        }
    }

    pub fn handle_focus_proxy_focus(
        &self,
        item: &NavigationMenuItemContext,
        trigger_id: &str,
        content_id: &str,
    ) {
        let (last_id, from_content) = self
            .focus_tracker
            .try_borrow()
            .map(|ft| {
                (
                    ft.prev_focused_id.clone(),
                    ft.prev_within_any(&[content_id.to_string()]),
                )
            })
            .unwrap_or((None, false));

        match resolve_focus_proxy_decision(last_id.as_deref(), trigger_id, from_content) {
            NavigationMenuFocusProxyDecision::Enter(side) => {
                item.on_focus_proxy_enter(&self.relationships, side);
            }
            NavigationMenuFocusProxyDecision::Ignore => {}
        }
    }

    pub fn root_attributes(&self, id_override: Option<String>) -> NavigationMenuRootAttributes {
        self.def().root_attributes(id_override)
    }

    pub fn sub_attributes(&self, id_override: Option<String>) -> NavigationMenuSubAttributes {
        self.def().sub_attributes(id_override)
    }

    pub fn list_track_attributes(
        &self,
        id_override: Option<String>,
    ) -> NavigationMenuListTrackAttributes {
        self.def().list_track_attributes(id_override)
    }

    pub fn list_attributes(&self, id_override: Option<String>) -> NavigationMenuListAttributes {
        self.def().list_attributes(id_override)
    }

    pub fn item_attributes(
        &self,
        value: &str,
        id_override: Option<String>,
    ) -> NavigationMenuItemAttributes {
        self.def().item_attributes(value, id_override)
    }

    pub fn trigger_attributes(
        &self,
        value: &str,
        disabled: bool,
        id_override: Option<String>,
        content_id_override: Option<String>,
    ) -> NavigationMenuTriggerAttributes {
        self.def()
            .trigger_attributes(value, disabled, id_override, content_id_override)
    }

    pub fn focus_proxy_attributes(&self, value: &str) -> NavigationMenuFocusProxyAttributes {
        self.def().focus_proxy_attributes(value)
    }

    pub fn aria_owns_attributes(
        &self,
        value: &str,
        content_id_override: Option<String>,
    ) -> NavigationMenuAriaOwnsAttributes {
        self.def()
            .aria_owns_attributes(value, content_id_override)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn content_attributes(
        &self,
        value: &str,
        open: bool,
        motion: Option<NavigationMenuMotion>,
        inline_mode: bool,
        id_override: Option<String>,
        trigger_id_override: Option<String>,
        user_style: Option<&str>,
    ) -> NavigationMenuContentAttributes {
        self.def().content_attributes(
            value,
            open,
            motion,
            inline_mode,
            id_override,
            trigger_id_override,
            user_style,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn link_attributes(
        &self,
        key: &str,
        href: Option<String>,
        target: Option<String>,
        rel: Option<String>,
        active: bool,
        focused: bool,
        id_override: Option<String>,
    ) -> NavigationMenuLinkAttributes {
        self.def()
            .link_attributes(key, href, target, rel, active, focused, id_override)
    }

    pub fn indicator_attributes(
        &self,
        is_visible: bool,
        position: NavigationMenuIndicatorPosition,
        id_override: Option<String>,
        user_style: Option<&str>,
    ) -> NavigationMenuIndicatorAttributes {
        self.def()
            .indicator_attributes(is_visible, position, id_override, user_style)
    }

    pub fn viewport_attributes(
        &self,
        open: bool,
        size: Option<NavigationMenuViewportSize>,
        id_override: Option<String>,
        user_style: Option<&str>,
    ) -> NavigationMenuViewportAttributes {
        self.def()
            .viewport_attributes(open, size, id_override, user_style)
    }
}

pub fn active_element_id() -> Option<String> {
    oxidase::dom::Document::current()
        .and_then(|doc| doc.active_element())
        .and_then(|el| el.id())
}

pub fn is_active_element_inside(container_id: &str) -> bool {
    crate::foundation::browser::is_focus_within_element(container_id)
}

pub fn is_ignored_arrow_element_by_id(element_id: &str) -> bool {
    let Some(doc) = oxidase::dom::Document::current() else {
        return false;
    };
    let Some(el) = doc.element_by_id(element_id) else {
        return false;
    };
    matches!(el.tag_name().as_deref(), Some("input" | "textarea"))
}

pub fn first_data_focused_id(container_id: &str) -> Option<String> {
    query_selector_all_by_id(container_id, "[data-focused]")
        .into_iter()
        .find_map(|el| el.id())
}

pub fn focus_content_candidates(content_id: &str, side: NavigationMenuFocusProxySide) {
    restore_tab_order(content_id);
    let mut candidates = get_tabbable_candidates(content_id);
    if side == NavigationMenuFocusProxySide::End {
        candidates.reverse();
    }
    let _ = focus_first_candidate_id(&candidates);
}

pub fn focus_element_reliable(target_id: &str) {
    foundation_focus_element_reliable(target_id, FocusOptions::new(true));
}

pub fn focus_first_candidate_id(candidates: &[String]) -> bool {
    if candidates.is_empty() {
        return false;
    }
    let Some(doc) = oxidase::dom::Document::current() else {
        if let Some(first) = candidates.first() {
            focus_element_by_id(first);
            return true;
        }
        return false;
    };
    let prev_active_id = doc.active_element().and_then(|el| el.id());
    for id in candidates {
        if prev_active_id.as_deref() == Some(id.as_str()) {
            return true;
        }
        if let Some(el) = doc.element_by_id(id) {
            let _ = el.focus_sync_with_options(FocusOptions::new(true));
            if el.has_focus()
                || doc.active_element().and_then(|a| a.id()).as_deref() == Some(id.as_str())
            {
                return true;
            }
        }
    }
    if let Some(first) = candidates.first()
        && doc.element_by_id(first).is_some()
    {
        focus_element_by_id(first);
        return true;
    }
    false
}

/// Momentarily removes `aria-hidden`, focuses the `FocusProxy` guard, and restores `aria-hidden`
/// on the next frame so screen readers do not warn about hidden focused nodes.
pub fn handle_proxy_focus(proxy_id: &str) {
    let Some(doc) = oxidase::dom::Document::current() else {
        focus_element_by_id(proxy_id);
        return;
    };
    let Some(el) = doc.element_by_id(proxy_id) else {
        return;
    };
    let prev_aria_hidden = el.get_attribute("aria-hidden");
    el.remove_attribute("aria-hidden");
    let _ = el.focus_sync_with_options(FocusOptions::new(true));
    let pid = proxy_id.to_string();
    spawn_root_task(async move {
        let _ = oxidase::frame::next_frame().await;
        if let Some(doc) = oxidase::dom::Document::current()
            && let Some(el) = doc.element_by_id(&pid)
        {
            match prev_aria_hidden {
                Some(val) => el.set_attribute("aria-hidden", &val),
                None => el.set_attribute("aria-hidden", "true"),
            }
        }
    });
}

/// Measures untransformed CSSOM `offset_rect()` dimensions for `Viewport` sizing.
pub fn measure_viewport_offset_size(content_id: &str) -> Option<NavigationMenuViewportSize> {
    let rect = get_element_offset_rect(content_id)?;
    Some(NavigationMenuViewportSize {
        width: rect.width() as f64,
        height: rect.height() as f64,
    })
}

/// Measures untransformed CSSOM `offset_rect()` position/size for `Indicator` alignment.
pub fn measure_indicator_offset_position(
    trigger_id: &str,
    orientation: Orientation,
) -> Option<NavigationMenuIndicatorPosition> {
    let rect = get_element_offset_rect(trigger_id)?;
    Some(match orientation {
        Orientation::Vertical => NavigationMenuIndicatorPosition {
            size: rect.height() as f64,
            offset: rect.y() as f64,
        },
        Orientation::Horizontal | Orientation::Both => NavigationMenuIndicatorPosition {
            size: rect.width() as f64,
            offset: rect.x() as f64,
        },
    })
}

/// Observes `element_id` resize events and invokes `on_measure` with Dioxus scope safety.
pub fn start_offset_resize_observer(
    element_id: &str,
    mut on_measure: impl FnMut() + 'static,
) -> WatcherGuard {
    start_resize_monitor(element_id, ResizeObserverBox::BorderBox, move |_| {
        on_measure();
    })
}

fn update_signal_if_changed<T: PartialEq + 'static>(mut sig: Signal<T>, next: T) {
    if sig.try_peek().is_ok_and(|cur| *cur != next)
        && let Ok(mut w) = sig.try_write()
    {
        *w = next;
    }
}

pub fn advance_guard_token(mut sig: Signal<u64>) -> u64 {
    let next = sig.try_peek().map(|v| v.saturating_add(1)).unwrap_or(1);
    if let Ok(mut w) = sig.try_write() {
        *w = next;
    }
    next
}

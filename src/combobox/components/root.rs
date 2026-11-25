use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    form::use_form_control_sync_with_epoch,
    overlay::{
        FloatingReadiness, PlacementAlign, PlacementSide, PortalHost,
        unregister_active_overlay_layer,
    },
    shared::{ScopeHandle, use_stable_id},
};

use super::hidden_input::ComboboxHiddenInput;
use crate::combobox::{
    relationships::ComboboxRelationships,
    runtime::{ComboboxRuntime, ComboboxRuntimeState},
    state::ComboboxOptionRegistry,
    types::{ComboboxContext, ComboboxItemData, ComboboxMode, ComboboxScrollAlignment},
};

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub mode: ComboboxMode,
    #[props(default)]
    pub value: Option<Signal<Option<String>>>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<String>>>,
    #[props(default)]
    pub values: Option<Signal<Vec<String>>>,
    #[props(default)]
    pub default_values: Option<Vec<String>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default)]
    pub input_value: Option<Signal<String>>,
    #[props(default)]
    pub default_input_value: Option<String>,
    #[props(default)]
    pub on_input_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub open: Option<Signal<bool>>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default = false)]
    pub loop_focus: bool,
    #[props(default)]
    pub scroll_alignment: ComboboxScrollAlignment,
    #[props(default)]
    pub items: Vec<ComboboxItemData>,
    #[props(default = true)]
    pub allow_deselect: bool,
    #[props(default = false)]
    pub clear_on_deselect: bool,
    #[props(default)]
    pub portal_host: PortalHost,
    pub children: Element,
}

#[component]
pub fn ComboboxRoot(props: ComboboxRootProps) -> Element {
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = use_hook(|| ScopeHandle::root(stable_id.clone()));
    let relationships = use_hook(|| ComboboxRelationships::new(scope.clone()));

    let initial_input_value = {
        if let Some(ref iv) = props.default_input_value {
            iv.clone()
        } else if let Some(ref dv) = props.default_value {
            props
                .items
                .iter()
                .find(|i| &i.value == dv)
                .map(|i| i.label.clone())
                .unwrap_or_default()
        } else {
            String::new()
        }
    };

    // 4B D-4: Reactive config signals synced on every render (zero mount-time snapshot).
    let mut mode_sig = use_signal(|| props.mode);
    if *mode_sig.peek() != props.mode {
        mode_sig.set(props.mode);
    }
    let mut disabled_sig = use_signal(|| props.disabled);
    if *disabled_sig.peek() != props.disabled {
        disabled_sig.set(props.disabled);
    }
    let mut required_sig = use_signal(|| props.required);
    if *required_sig.peek() != props.required {
        required_sig.set(props.required);
    }
    let mut loop_focus_sig = use_signal(|| props.loop_focus);
    if *loop_focus_sig.peek() != props.loop_focus {
        loop_focus_sig.set(props.loop_focus);
    }
    let mut scroll_alignment_sig = use_signal(|| props.scroll_alignment);
    if *scroll_alignment_sig.peek() != props.scroll_alignment {
        scroll_alignment_sig.set(props.scroll_alignment);
    }
    let mut allow_deselect_sig = use_signal(|| props.allow_deselect);
    if *allow_deselect_sig.peek() != props.allow_deselect {
        allow_deselect_sig.set(props.allow_deselect);
    }
    let mut root_clear_on_deselect_sig = use_signal(|| props.clear_on_deselect);
    if *root_clear_on_deselect_sig.peek() != props.clear_on_deselect {
        root_clear_on_deselect_sig.set(props.clear_on_deselect);
    }
    let input_clear_on_deselect_sig = use_signal(|| None::<bool>);
    let mut name_sig = use_signal(|| props.name.clone());
    if *name_sig.peek() != props.name {
        name_sig.set(props.name.clone());
    }
    let mut form_sig = use_signal(|| props.form.clone());
    if *form_sig.peek() != props.form {
        form_sig.set(props.form.clone());
    }
    let mut portal_host_sig = use_signal(|| props.portal_host.clone());
    if *portal_host_sig.peek() != props.portal_host {
        portal_host_sig.set(props.portal_host.clone());
    }

    // 4B D-4: Reactive callback signals synced on every render.
    let mut on_open_change_sig = use_signal(|| props.on_open_change);
    if *on_open_change_sig.peek() != props.on_open_change {
        on_open_change_sig.set(props.on_open_change);
    }
    let mut on_open_change_complete_sig = use_signal(|| props.on_open_change_complete);
    if *on_open_change_complete_sig.peek() != props.on_open_change_complete {
        on_open_change_complete_sig.set(props.on_open_change_complete);
    }
    let mut on_value_change_sig = use_signal(|| props.on_value_change);
    if *on_value_change_sig.peek() != props.on_value_change {
        on_value_change_sig.set(props.on_value_change);
    }
    let mut on_values_change_sig = use_signal(|| props.on_values_change);
    if *on_values_change_sig.peek() != props.on_values_change {
        on_values_change_sig.set(props.on_values_change);
    }
    let mut on_input_value_change_sig = use_signal(|| props.on_input_value_change);
    if *on_input_value_change_sig.peek() != props.on_input_value_change {
        on_input_value_change_sig.set(props.on_input_value_change);
    }

    let open_sig = use_signal(|| props.default_open);
    let mut controlled_open_sig = use_signal(|| props.open);
    if *controlled_open_sig.peek() != props.open {
        controlled_open_sig.set(props.open);
    }

    let value_sig = use_signal(|| props.default_value.clone());
    let mut controlled_value_sig = use_signal(|| props.value);
    if *controlled_value_sig.peek() != props.value {
        controlled_value_sig.set(props.value);
    }

    let values_sig = use_signal(|| props.default_values.clone().unwrap_or_default());
    let mut controlled_values_sig = use_signal(|| props.values);
    if *controlled_values_sig.peek() != props.values {
        controlled_values_sig.set(props.values);
    }

    let input_value_sig = use_signal(|| initial_input_value.clone());
    let mut controlled_input_value_sig = use_signal(|| props.input_value);
    if *controlled_input_value_sig.peek() != props.input_value {
        controlled_input_value_sig.set(props.input_value);
    }

    let highlighted_value = use_signal(|| None::<String>);
    let pending_open_highlight = use_signal(|| None);
    let pending_input_reconcile = use_signal(|| 0_u64);
    let registry = use_signal(|| {
        let mut reg = ComboboxOptionRegistry::new();
        reg.seed_items(&props.items);
        reg
    });
    let registry_version = use_signal(|| 0_u64);
    let pressed_item = use_signal(|| None);
    let suppress_next_click = use_signal(|| false);
    let is_composing = use_signal(|| false);
    let user_has_scrolled = use_signal(|| false);
    let content_mounted = use_signal(|| false);
    let default_value_initial = use_signal(|| props.default_value.clone());
    let default_values_initial = use_signal(|| props.default_values.clone().unwrap_or_default());
    let default_input_value_initial = use_signal(|| initial_input_value);
    let mut reset_epoch_sig = use_signal(|| 0_u64);

    let input_id_sig = use_signal(|| relationships.input_id().to_owned());
    let trigger_id_sig = use_signal(|| relationships.trigger_id().to_owned());
    let content_id_sig = use_signal(|| relationships.content_id().to_owned());
    let viewport_id_sig = use_signal(|| relationships.viewport_id().to_owned());

    let input_handle = use_signal(|| None);
    let content_handle = use_signal(|| None);
    let placement = use_signal(|| None);
    let content_readiness = use_signal(|| FloatingReadiness::Measuring);

    let preferred_side = use_signal(|| PlacementSide::Bottom);
    let preferred_align = use_signal(|| PlacementAlign::Start);
    let side = use_signal(|| PlacementSide::Bottom);
    let align = use_signal(|| PlacementAlign::Start);
    let side_offset = use_signal(|| 4.0_f32);
    let align_offset = use_signal(|| 0.0_f32);
    let avoid_collisions = use_signal(|| true);
    let collision_boundary = use_signal(|| None::<String>);
    let collision_padding = use_signal(|| 0.0_f32);
    let arrow_padding = use_signal(|| 0.0_f32);
    let hide_when_detached = use_signal(|| false);
    let sticky = use_signal(|| None::<String>);
    let custom_anchor = use_signal(|| None::<String>);
    let force_mount = use_signal(|| false);

    let can_scroll_up = use_signal(|| false);
    let can_scroll_down = use_signal(|| false);

    let on_interact_outside = use_signal(|| None);
    let on_escape_key_down = use_signal(|| None);

    let dismiss_monitor = use_signal(|| None);
    let position_monitor = use_signal(|| None);

    let runtime = ComboboxRuntime {
        relationships: relationships.clone(),
        state: ComboboxRuntimeState {
            mode: mode_sig,
            disabled: disabled_sig,
            required: required_sig,
            loop_focus: loop_focus_sig,
            scroll_alignment: scroll_alignment_sig,
            allow_deselect: allow_deselect_sig,
            root_clear_on_deselect: root_clear_on_deselect_sig,
            input_clear_on_deselect: input_clear_on_deselect_sig,
            name: name_sig,
            form: form_sig,
            portal_host: portal_host_sig,
            on_open_change: on_open_change_sig,
            on_open_change_complete: on_open_change_complete_sig,
            on_value_change: on_value_change_sig,
            on_values_change: on_values_change_sig,
            on_input_value_change: on_input_value_change_sig,
            open: open_sig,
            controlled_open: controlled_open_sig,
            value: value_sig,
            controlled_value: controlled_value_sig,
            values: values_sig,
            controlled_values: controlled_values_sig,
            input_value: input_value_sig,
            controlled_input_value: controlled_input_value_sig,
            highlighted_value,
            pending_open_highlight,
            pending_input_reconcile,
            registry,
            registry_version,
            pressed_item,
            suppress_next_click,
            is_composing,
            user_has_scrolled,
            content_mounted,
            default_value_initial,
            default_values_initial,
            default_input_value_initial,
            reset_epoch: reset_epoch_sig,
            input_id: input_id_sig,
            trigger_id: trigger_id_sig,
            content_id: content_id_sig,
            viewport_id: viewport_id_sig,
            input_handle,
            content_handle,
            placement,
            content_readiness,
            preferred_side,
            preferred_align,
            side,
            align,
            side_offset,
            align_offset,
            avoid_collisions,
            collision_boundary,
            collision_padding,
            arrow_padding,
            hide_when_detached,
            sticky,
            custom_anchor,
            force_mount,
            can_scroll_up,
            can_scroll_down,
            on_interact_outside,
            on_escape_key_down,
            dismiss_monitor,
            position_monitor,
        },
    };

    runtime.seed_items(&props.items);

    // CBX-3 / DEF-CBX-09: Watch selection non-empty -> empty transitions so programmatic
    // deselection also clears `input_value` when `clear_on_deselect` is enabled.
    let has_selection = match runtime.mode() {
        ComboboxMode::Single => runtime.value().is_some(),
        ComboboxMode::Multiple => !runtime.values().is_empty(),
    };
    let mut prev_has_selection = use_signal(|| has_selection);
    {
        let rt = runtime.clone();
        use_effect(use_reactive((&has_selection,), move |(now_has_sel,)| {
            let was_has_sel = *prev_has_selection.peek();
            if was_has_sel != now_has_sel {
                prev_has_selection.set(now_has_sel);
                if was_has_sel
                    && !now_has_sel
                    && rt.clear_on_deselect()
                    && !rt.input_value_untracked().is_empty()
                {
                    rt.set_input_value(String::new(), true);
                }
            }
        }));
    }

    let current_input_id = runtime.current_input_id();
    let reset_rt = runtime.clone();
    let (_has_form, reset_epoch) = use_form_control_sync_with_epoch(
        &current_input_id,
        props.name.as_deref(),
        props.form.as_deref(),
        move || {
            reset_rt.handle_form_reset();
        },
    );
    if *reset_epoch_sig.peek() != reset_epoch {
        reset_epoch_sig.set(reset_epoch);
    }

    let drop_rt = runtime.clone();
    use_drop(move || {
        drop_rt.stop_dismiss_monitor();
        drop_rt.stop_position_monitor();
        let cid = drop_rt.current_content_id_untracked();
        unregister_active_overlay_layer(&cid);
    });

    use_context_provider(|| ComboboxContext {
        runtime: runtime.clone(),
    });

    let has_name = props.name.as_deref().is_some_and(|n| !n.is_empty());

    rsx! {
        {props.children}
        if has_name {
            ComboboxHiddenInput {}
        }
    }
}

pub use ComboboxRoot as Combobox;

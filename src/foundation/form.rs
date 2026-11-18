use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use super::browser::{
    FocusOptions, WatcherGuard, focus_element_sync_or_defer, probe_element_form_ancestor,
    spawn_root_task, start_form_reset_monitor,
};

pub use super::browser::collect_form_data;

/// Out-of-flow 1x1px visual clamp style overlaid adjacent to the headless trigger.
///
/// Prevents Flexbox/Grid item or `gap` distortion across Web and Blitz Native Taffy layout
/// while keeping native browser constraint validation popups anchored near the trigger.
pub const HIDDEN_INPUT_VISUAL_CLAMP_STYLE: &str = "position: absolute; pointer-events: none; opacity: 0; margin: 0; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; padding: 0; transform: translateX(-100%);";

/// HTML `<input type="...">` variants supported by the shared hidden form helper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HiddenInputKind {
    #[default]
    Checkbox,
    Radio,
    Hidden,
    Text,
}

impl HiddenInputKind {
    /// Returns the standard lowercase HTML `type` attribute token.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Hidden => "hidden",
            Self::Text => "text",
        }
    }

    /// Returns `true` when the input type participates in HTML `checked` state (`checkbox` or `radio`).
    pub const fn uses_checked_state(&self) -> bool {
        matches!(self, Self::Checkbox | Self::Radio)
    }
}

/// Pure attribute publisher for [`HiddenFormInput`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HiddenInputAttributes {
    pub id: Option<String>,
    pub r#type: &'static str,
    pub name: Option<String>,
    pub value: String,
    pub checked: Option<bool>,
    pub disabled: bool,
    pub required: bool,
    pub read_only: bool,
    pub form: Option<String>,
    pub autocomplete: Option<String>,
    pub aria_hidden: &'static str,
    pub tabindex: i32,
    pub style: &'static str,
}

impl HiddenInputAttributes {
    /// Derives normalized HTML attributes from [`HiddenFormInputProps`].
    pub fn from_props(props: &HiddenFormInputProps) -> Self {
        let checked = if props.kind.uses_checked_state() {
            Some(props.checked.unwrap_or(false))
        } else {
            None
        };
        Self {
            id: props.id.clone().filter(|id| !id.is_empty()),
            r#type: props.kind.as_str(),
            name: props.name.clone().filter(|name| !name.is_empty()),
            value: props.value.clone(),
            checked,
            disabled: props.disabled,
            required: props.required,
            read_only: props.read_only,
            form: props.form.clone().filter(|form| !form.is_empty()),
            autocomplete: props
                .autocomplete
                .clone()
                .filter(|autocomplete| !autocomplete.is_empty()),
            aria_hidden: "true",
            tabindex: -1,
            style: HIDDEN_INPUT_VISUAL_CLAMP_STYLE,
        }
    }
}

/// Pure helper determining whether a form-associated hidden input should mount immediately
/// prior to or alongside live DOM `<form>` ancestor detection.
pub fn should_render_hidden_input(
    name: Option<&str>,
    form: Option<&str>,
    has_ancestor_form: bool,
) -> bool {
    name.is_some_and(|n| !n.is_empty())
        || form.is_some_and(|f| !f.is_empty())
        || has_ancestor_form
}

/// Props for a single visually-clamped hidden `<input>` element.
#[derive(Props, Clone, PartialEq)]
pub struct HiddenFormInputProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub kind: HiddenInputKind,
    #[props(default)]
    pub name: Option<String>,
    #[props(default = String::from("on"))]
    pub value: String,
    #[props(default)]
    pub checked: Option<bool>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub form: Option<String>,
    #[props(default = Some(String::from("off")))]
    pub autocomplete: Option<String>,
    #[props(default)]
    pub focus_target_id: Option<String>,
    #[props(default = 0)]
    pub reset_epoch: u64,
}

/// Renders a single visually-clamped `<input>` sibling for native `<form>` `FormData`
/// serialization, constraint validation, and focus forwarding.
#[component]
pub fn HiddenFormInput(props: HiddenFormInputProps) -> Element {
    let attrs = HiddenInputAttributes::from_props(&props);
    let focus_target = props.focus_target_id.clone().filter(|id| !id.is_empty());

    let on_focus = move |_evt: FocusEvent| {
        if let Some(ref target_id) = focus_target {
            focus_element_sync_or_defer(target_id, FocusOptions::new(false));
        }
    };

    if let Some(is_checked) = attrs.checked {
        rsx! {
            for epoch in [props.reset_epoch] {
                input {
                    key: "{epoch}",
                    id: attrs.id.as_deref(),
                    r#type: attrs.r#type,
                    name: attrs.name.as_deref(),
                    value: "{attrs.value}",
                    checked: is_checked,
                    disabled: attrs.disabled,
                    required: attrs.required,
                    readonly: attrs.read_only,
                    form: attrs.form.as_deref(),
                    autocomplete: attrs.autocomplete.as_deref(),
                    "aria-hidden": attrs.aria_hidden,
                    tabindex: attrs.tabindex,
                    style: attrs.style,
                    onfocus: on_focus.clone(),
                }
            }
        }
    } else {
        rsx! {
            for epoch in [props.reset_epoch] {
                input {
                    key: "{epoch}",
                    id: attrs.id.as_deref(),
                    r#type: attrs.r#type,
                    name: attrs.name.as_deref(),
                    value: "{attrs.value}",
                    disabled: attrs.disabled,
                    required: attrs.required,
                    readonly: attrs.read_only,
                    form: attrs.form.as_deref(),
                    autocomplete: attrs.autocomplete.as_deref(),
                    "aria-hidden": attrs.aria_hidden,
                    tabindex: attrs.tabindex,
                    style: attrs.style,
                    onfocus: on_focus.clone(),
                }
            }
        }
    }
}

/// Props for rendering a keyed list of hidden `<input type="hidden">` elements
/// for multi-value controls (e.g., multi-thumb `slider` or multi-select `combobox`).
#[derive(Props, Clone, PartialEq)]
pub struct HiddenFormInputsProps {
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub values: Vec<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub form: Option<String>,
    #[props(default = Some(String::from("off")))]
    pub autocomplete: Option<String>,
    #[props(default)]
    pub focus_target_id: Option<String>,
    #[props(default = 0)]
    pub reset_epoch: u64,
}

/// Renders zero or more visually-clamped `<input type="hidden">` siblings for multi-value form controls.
#[component]
pub fn HiddenFormInputs(props: HiddenFormInputsProps) -> Element {
    if props.values.is_empty() {
        return rsx! {};
    }

    rsx! {
        for (idx, val) in props.values.iter().enumerate() {
            HiddenFormInput {
                key: "{props.reset_epoch}-{idx}-{val}",
                reset_epoch: props.reset_epoch,
                kind: HiddenInputKind::Hidden,
                name: props.name.clone(),
                value: val.clone(),
                disabled: props.disabled,
                required: props.required,
                read_only: props.read_only,
                form: props.form.clone(),
                autocomplete: props.autocomplete.clone(),
                focus_target_id: props.focus_target_id.clone(),
            }
        }
    }
}

/// Tracks whether `control_id` participates in a `<form>` (via `name`, explicit `form`, or a live
/// `<form>` ancestor) and subscribes to `<form>` reset events with automatic `use_drop` cleanup.
///
/// Returns `(should_render_hidden_input, reset_epoch)`, where `reset_epoch` increments on every
/// native `<form>` reset so callers can key [`HiddenFormInput`] and force a clean DOM `<input>`
/// remount even when component checked/value state was already at its default prior to reset.
pub fn use_form_control_sync_with_epoch(
    control_id: &str,
    name: Option<&str>,
    form: Option<&str>,
    on_reset: impl FnMut() + 'static,
) -> (bool, u64) {
    let reset_cb: Rc<RefCell<Box<dyn FnMut()>>> =
        use_hook(|| Rc::new(RefCell::new(Box::new(|| {}) as Box<dyn FnMut()>)));
    *reset_cb.borrow_mut() = Box::new(on_reset);

    let mut has_ancestor = use_signal(|| false);
    let reset_epoch = use_signal(|| 0_u64);
    let watcher_slot: Rc<RefCell<Option<WatcherGuard>>> = use_hook(|| Rc::new(RefCell::new(None)));
    let probe_task_slot: Rc<RefCell<Option<dioxus::core::Task>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    let cid_owned = control_id.to_string();
    let form_owned = form.filter(|f| !f.is_empty()).map(str::to_owned);
    let effect_watcher = Rc::clone(&watcher_slot);
    let effect_probe = Rc::clone(&probe_task_slot);
    let effect_cb = Rc::clone(&reset_cb);

    use_effect(use_reactive(
        (&cid_owned, &form_owned),
        move |(cid, explicit_form)| {
            if let Some(task) = effect_probe.borrow_mut().take() {
                task.cancel();
            }
            if let Some(prev) = effect_watcher.borrow_mut().take() {
                prev.stop();
            }

            if cid.is_empty() {
                if *has_ancestor.peek() {
                    has_ancestor.set(false);
                }
                return;
            }

            let bind_watcher = |watcher_ref: &Rc<RefCell<Option<WatcherGuard>>>,
                                cb_ref: &Rc<RefCell<Box<dyn FnMut()>>>,
                                mut epoch_sig: Signal<u64>,
                                element_id: &str,
                                form_id: Option<&str>| {
                if let Some(prev) = watcher_ref.borrow_mut().take() {
                    prev.stop();
                }
                let cb_handle = Rc::clone(cb_ref);
                let guard = start_form_reset_monitor(element_id, form_id, move || {
                    let next_epoch = epoch_sig.peek().wrapping_add(1);
                    epoch_sig.set(next_epoch);
                    if let Ok(mut cb) = cb_handle.try_borrow_mut() {
                        cb();
                    }
                });
                *watcher_ref.borrow_mut() = Some(guard);
            };

            match probe_element_form_ancestor(&cid) {
                Some(ancestor_detected) => {
                    if *has_ancestor.peek() != ancestor_detected {
                        has_ancestor.set(ancestor_detected);
                    }
                    if explicit_form.is_some() || ancestor_detected {
                        bind_watcher(
                            &effect_watcher,
                            &effect_cb,
                            reset_epoch,
                            &cid,
                            explicit_form.as_deref(),
                        );
                    }
                }
                None => {
                    if explicit_form.is_some() {
                        bind_watcher(
                            &effect_watcher,
                            &effect_cb,
                            reset_epoch,
                            &cid,
                            explicit_form.as_deref(),
                        );
                    }
                    let task_watcher = Rc::clone(&effect_watcher);
                    let task_cb = Rc::clone(&effect_cb);
                    let mut task_has_ancestor = has_ancestor;
                    let mut task_epoch = reset_epoch;
                    let task_cid = cid.clone();
                    let task_form = explicit_form.clone();
                    let task = spawn_root_task(async move {
                        for _ in 0..5 {
                            let _ = oxidase::frame::next_frame().await;
                            if let Some(ancestor_detected) = probe_element_form_ancestor(&task_cid)
                            {
                                if *task_has_ancestor.peek() != ancestor_detected {
                                    task_has_ancestor.set(ancestor_detected);
                                }
                                if task_form.is_some() || ancestor_detected {
                                    if let Some(prev) = task_watcher.borrow_mut().take() {
                                        prev.stop();
                                    }
                                    let cb_handle = Rc::clone(&task_cb);
                                    let guard = start_form_reset_monitor(
                                        &task_cid,
                                        task_form.as_deref(),
                                        move || {
                                            let next_epoch = task_epoch.peek().wrapping_add(1);
                                            task_epoch.set(next_epoch);
                                            if let Ok(mut cb) = cb_handle.try_borrow_mut() {
                                                cb();
                                            }
                                        },
                                    );
                                    *task_watcher.borrow_mut() = Some(guard);
                                }
                                break;
                            }
                        }
                    });
                    *effect_probe.borrow_mut() = Some(task);
                }
            }
        },
    ));

    let drop_watcher = Rc::clone(&watcher_slot);
    let drop_probe = Rc::clone(&probe_task_slot);
    use_drop(move || {
        if let Some(task) = drop_probe.borrow_mut().take() {
            task.cancel();
        }
        if let Some(guard) = drop_watcher.borrow_mut().take() {
            guard.stop();
        }
    });

    (
        should_render_hidden_input(name, form, *has_ancestor.read()),
        *reset_epoch.read(),
    )
}

/// Tracks whether `control_id` participates in a `<form>` (via `name`, explicit `form`, or a live
/// `<form>` ancestor) and subscribes to `<form>` reset events with automatic `use_drop` cleanup.
///
/// Returns `true` when the caller should render [`HiddenFormInput`].
pub fn use_form_control_sync(
    control_id: &str,
    name: Option<&str>,
    form: Option<&str>,
    on_reset: impl FnMut() + 'static,
) -> bool {
    use_form_control_sync_with_epoch(control_id, name, form, on_reset).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_input_kind_tokens_and_checked_flag() {
        assert_eq!(HiddenInputKind::Checkbox.as_str(), "checkbox");
        assert!(HiddenInputKind::Checkbox.uses_checked_state());

        assert_eq!(HiddenInputKind::Radio.as_str(), "radio");
        assert!(HiddenInputKind::Radio.uses_checked_state());

        assert_eq!(HiddenInputKind::Hidden.as_str(), "hidden");
        assert!(!HiddenInputKind::Hidden.uses_checked_state());

        assert_eq!(HiddenInputKind::Text.as_str(), "text");
        assert!(!HiddenInputKind::Text.uses_checked_state());
    }

    #[test]
    fn hidden_input_attributes_checkbox_and_radio_include_checked() {
        let cb_props = HiddenFormInputProps {
            id: Some("cb-hidden".to_string()),
            kind: HiddenInputKind::Checkbox,
            name: Some("terms".to_string()),
            value: "accepted".to_string(),
            checked: Some(true),
            disabled: false,
            required: true,
            read_only: false,
            form: Some("signup-form".to_string()),
            autocomplete: Some("off".to_string()),
            focus_target_id: Some("cb-trigger".to_string()),
            reset_epoch: 0,
        };

        let attrs = HiddenInputAttributes::from_props(&cb_props);
        assert_eq!(attrs.id.as_deref(), Some("cb-hidden"));
        assert_eq!(attrs.r#type, "checkbox");
        assert_eq!(attrs.name.as_deref(), Some("terms"));
        assert_eq!(attrs.value, "accepted");
        assert_eq!(attrs.checked, Some(true));
        assert!(!attrs.disabled);
        assert!(attrs.required);
        assert!(!attrs.read_only);
        assert_eq!(attrs.form.as_deref(), Some("signup-form"));
        assert_eq!(attrs.autocomplete.as_deref(), Some("off"));
        assert_eq!(attrs.aria_hidden, "true");
        assert_eq!(attrs.tabindex, -1);
        assert_eq!(attrs.style, HIDDEN_INPUT_VISUAL_CLAMP_STYLE);

        // Indeterminate / None checked defaults to Some(false) on checkbox/radio
        let indet_props = HiddenFormInputProps {
            checked: None,
            ..cb_props
        };
        assert_eq!(
            HiddenInputAttributes::from_props(&indet_props).checked,
            Some(false)
        );
    }

    #[test]
    fn hidden_input_attributes_hidden_and_text_omit_checked_and_empty_strings() {
        let hidden_props = HiddenFormInputProps {
            id: Some(String::new()),
            kind: HiddenInputKind::Hidden,
            name: Some(String::new()),
            value: "42".to_string(),
            checked: Some(true),
            disabled: true,
            required: false,
            read_only: true,
            form: Some(String::new()),
            autocomplete: Some(String::new()),
            focus_target_id: None,
            reset_epoch: 0,
        };

        let attrs = HiddenInputAttributes::from_props(&hidden_props);
        assert_eq!(attrs.id, None);
        assert_eq!(attrs.r#type, "hidden");
        assert_eq!(attrs.name, None);
        assert_eq!(attrs.value, "42");
        assert_eq!(attrs.checked, None);
        assert!(attrs.disabled);
        assert!(attrs.read_only);
        assert_eq!(attrs.form, None);
        assert_eq!(attrs.autocomplete, None);
    }

    #[test]
    fn should_render_hidden_input_truth_table() {
        assert!(!should_render_hidden_input(None, None, false));
        assert!(!should_render_hidden_input(Some(""), Some(""), false));
        assert!(should_render_hidden_input(Some("opt"), None, false));
        assert!(should_render_hidden_input(None, Some("checkout"), false));
        assert!(should_render_hidden_input(None, None, true));
    }
}

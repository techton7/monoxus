use monoxus::foundation::compose::project_attrs_as_child;
use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::foundation::{compose::Slottable, shared::ScopeHandle};
use monoxus::progress::{
    Indicator, PROGRESS_PARTS, Progress, ProgressPart, ProgressRoot, ProgressState,
    ProgressValidation, use_progress_context,
};

#[test]
fn public_progress_parts_and_aliases_are_stable() {
    assert_eq!(
        PROGRESS_PARTS,
        [ProgressPart::Root, ProgressPart::Indicator]
    );
    assert_eq!(PROGRESS_PARTS[0].as_str(), "root");
    assert_eq!(PROGRESS_PARTS[1].as_str(), "indicator");
}

#[test]
fn public_progress_contract_normalizes_and_omits_invalid_optional_values() {
    let scope = ScopeHandle::root("public-progress");
    let invalid = Progress::new(scope.clone(), Some(f64::NAN), 0.0);
    assert_eq!(invalid.max(), 100.0);
    assert_eq!(invalid.value(), None);
    assert_eq!(invalid.state(), ProgressState::Indeterminate);
    assert_eq!(
        invalid.validation_issues(),
        &[
            ProgressValidation::InvalidMax,
            ProgressValidation::InvalidValue
        ]
    );

    let attrs = invalid.root(None, false);
    assert_eq!(attrs.role, "progressbar");
    assert_eq!(attrs.aria_valuenow, None);
    assert_eq!(attrs.aria_valuetext, None);
    assert_eq!(attrs.data_value, None);
    assert_eq!(attrs.data_indeterminate, Some(""));
}

#[test]
fn public_progress_label_and_projection_contract() {
    let progress = Progress::new(ScopeHandle::root("public-progress"), Some(25.0), 100.0);
    let attrs = progress.root(Some("one quarter".to_string()), false);
    assert_eq!(attrs.aria_valuetext.as_deref(), Some("one quarter"));

    let (target, projected, content) =
        project_attrs_as_child(&attrs, "section", Slottable::new("child"));
    assert_eq!(target, "section");
    assert_eq!(projected.data_state, "loading");
    assert_eq!(content, "child");
}

#[component]
fn ProgressComponentScenario() -> Element {
    rsx! {
        Progress {
            id: "public-progress",
            value: 50.0,
            Indicator { id: "public-indicator" }
        }
    }
}

#[test]
fn progress_component_and_indicator_alias_mount() {
    let mut dom = VirtualDom::new(ProgressComponentScenario);
    dom.rebuild_in_place();
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ProgressProbeResult {
    root_role: Option<String>,
    root_now: Option<String>,
    root_text: Option<String>,
    root_state: Option<String>,
    indicator_state: Option<String>,
    indicator_value: Option<String>,
}

#[component]
fn ProgressProbe(result: Rc<RefCell<ProgressProbeResult>>) -> Element {
    let context = use_progress_context();
    let runtime = context.runtime();
    let root = runtime.root(None);
    let indicator = runtime.indicator();
    *result.borrow_mut() = ProgressProbeResult {
        root_role: Some(root.role.to_string()),
        root_now: root.aria_valuenow,
        root_text: root.aria_valuetext,
        root_state: Some(root.data_state.to_string()),
        indicator_state: Some(indicator.data_state.to_string()),
        indicator_value: indicator.data_value,
    };
    rsx! {}
}

#[component]
fn ProgressProbeScenario(result: Rc<RefCell<ProgressProbeResult>>) -> Element {
    rsx! {
        ProgressRoot {
            value: 25.0,
            max: 100.0,
            ProgressProbe { result }
        }
    }
}

#[test]
fn public_context_keeps_root_and_indicator_contract_synchronized() {
    let result = Rc::new(RefCell::new(ProgressProbeResult::default()));
    let mut dom = VirtualDom::new_with_props(
        ProgressProbeScenario,
        ProgressProbeScenarioProps {
            result: result.clone(),
        },
    );
    dom.rebuild_in_place();

    assert_eq!(
        *result.borrow(),
        ProgressProbeResult {
            root_role: Some("progressbar".to_string()),
            root_now: Some("25".to_string()),
            root_text: Some("25%".to_string()),
            root_state: Some("loading".to_string()),
            indicator_state: Some("loading".to_string()),
            indicator_value: Some("25".to_string()),
        }
    );
}

#[test]
fn progress_root_skips_unchanged_runtime_signal_update() {
    use std::cell::Cell;

    #[derive(Clone)]
    struct RenderCounter(Rc<Cell<usize>>);
    impl PartialEq for RenderCounter {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }

    #[component]
    fn ProgressCountProbe(counter: RenderCounter) -> Element {
        let ctx = use_progress_context();
        let _rt = ctx.runtime();
        counter.0.set(counter.0.get() + 1);
        rsx! {}
    }

    #[component]
    fn ProgressL2Host(
        counter: RenderCounter,
        tick_out: Rc<RefCell<Option<Signal<u32>>>>,
        val_out: Rc<RefCell<Option<Signal<f64>>>>,
    ) -> Element {
        let tick = use_signal(|| 0_u32);
        let val = use_signal(|| 50.0_f64);
        *tick_out.borrow_mut() = Some(tick);
        *val_out.borrow_mut() = Some(val);
        let _ = tick();

        rsx! {
            ProgressRoot {
                value: val(),
                max: 100.0,
                ProgressCountProbe { counter }
            }
        }
    }

    let counter = RenderCounter(Rc::new(Cell::new(0)));
    let tick_out = Rc::new(RefCell::new(None));
    let val_out = Rc::new(RefCell::new(None));

    let mut dom = VirtualDom::new_with_props(
        ProgressL2Host,
        ProgressL2HostProps {
            counter: counter.clone(),
            tick_out: tick_out.clone(),
            val_out: val_out.clone(),
        },
    );
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(counter.0.get(), 1);

    // Unchanged parent re-render must not write runtime_sig or force child re-render.
    let mut tick = tick_out.borrow().expect("tick captured");
    dom.in_scope(ScopeId::APP, || tick.set(1));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(
        counter.0.get(),
        1,
        "L-2: ProgressRoot unchanged parent re-render must not force child re-render"
    );

    // Changing value updates runtime_sig and re-renders child.
    let mut val = val_out.borrow().expect("val captured");
    dom.in_scope(ScopeId::APP, || val.set(75.0));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(
        counter.0.get(),
        2,
        "L-2: ProgressRoot value change must propagate to ProgressContext subscribers"
    );
}

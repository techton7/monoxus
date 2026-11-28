//! Declarative `<AgentBoundary>` subtree scoping with monotonic security
//! propagation (`hidden`, `secret`, `inert`), section concatenation, and
//! zero wrapper DOM elements.

use dioxus::prelude::*;
use ui_sdk::agent::AgentBoundaryScope;

use crate::foundation::state::sync_signal;

/// Reactive context holding the resolved [`AgentBoundaryScope`] for a subtree.
#[derive(Clone, Copy, PartialEq)]
pub struct AgentBoundaryContext(pub Signal<AgentBoundaryScope>);

/// Props for [`AgentBoundary`].
#[derive(Clone, PartialEq, Props)]
pub struct AgentBoundaryProps {
    /// Single section segment appended to the inherited boundary path.
    #[props(default, into)]
    pub section: Option<String>,
    /// Additional section segments appended after `section`.
    #[props(default)]
    pub sections: Vec<String>,
    /// Monotonically hides all descendant components from agent snapshots and queries.
    #[props(default = false)]
    pub hidden: bool,
    /// Monotonically redacts all descendant component values and prevents `Expect` queries.
    #[props(default = false)]
    pub secret: bool,
    /// Monotonically blocks all agent actions against descendant components.
    #[props(default = false)]
    pub inert: bool,
    /// Default confirmation prompt for descendant components.
    ///
    /// Empty or whitespace-only strings are ignored so a blank child `confirm`
    /// never masks a parent boundary's confirmation gate.
    #[props(default, into)]
    pub confirm: Option<String>,
    pub children: Element,
}

pub(crate) fn normalize_non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|trimmed| !trimmed.is_empty())
        .map(str::to_owned)
}

fn build_own_boundary_scope(
    section: Option<&str>,
    sections: &[String],
    hidden: bool,
    secret: bool,
    inert: bool,
    confirm: Option<&str>,
) -> AgentBoundaryScope {
    let mut combined_sections = Vec::with_capacity(sections.len() + usize::from(section.is_some()));
    if let Some(single) = normalize_non_empty(section) {
        combined_sections.push(single);
    }
    for segment in sections {
        if let Some(clean) = normalize_non_empty(Some(segment.as_str())) {
            combined_sections.push(clean);
        }
    }

    AgentBoundaryScope {
        sections: combined_sections,
        hidden,
        secret,
        inert,
        confirm: normalize_non_empty(confirm),
    }
}

/// Declarative zero-DOM boundary component that propagates section paths and
/// monotonic security defaults (`hidden`, `secret`, `inert`, `confirm`) to
/// descendant primitives.
#[component]
pub fn AgentBoundary(props: AgentBoundaryProps) -> Element {
    let parent = use_hook(|| try_consume_context::<AgentBoundaryContext>());
    let own = build_own_boundary_scope(
        props.section.as_deref(),
        &props.sections,
        props.hidden,
        props.secret,
        props.inert,
        props.confirm.as_deref(),
    );

    let merged = match parent {
        Some(parent_ctx) => {
            let parent_scope = (parent_ctx.0)();
            parent_scope.merge_child(&own)
        }
        None => own,
    };

    let mut scope_signal = use_signal(|| merged.clone());
    sync_signal(&mut scope_signal, merged);
    use_context_provider(|| AgentBoundaryContext(scope_signal));

    props.children
}

/// Subscribing read of the nearest enclosing [`AgentBoundaryScope`], if any.
pub fn use_agent_boundary_scope() -> Option<AgentBoundaryScope> {
    try_consume_context::<AgentBoundaryContext>().map(|ctx| (ctx.0)())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use dioxus::prelude::*;
    use ui_sdk::agent::AgentBoundaryScope;

    use super::{AgentBoundary, build_own_boundary_scope, use_agent_boundary_scope};

    #[test]
    fn build_own_boundary_scope_trims_and_drops_blank_confirm_and_sections() {
        let scope = build_own_boundary_scope(
            Some("  Billing  "),
            &[
                String::from("   "),
                String::from("Invoices"),
                String::from(""),
            ],
            false,
            true,
            false,
            Some("   \t\n  "),
        );

        assert_eq!(
            scope.sections,
            vec![String::from("Billing"), String::from("Invoices")]
        );
        assert!(!scope.hidden);
        assert!(scope.secret);
        assert!(!scope.inert);
        assert_eq!(scope.confirm, None);
    }

    #[test]
    fn nested_agent_boundaries_inherit_monotonically_and_preserve_parent_confirm() {
        let outer_captured: Rc<RefCell<Option<AgentBoundaryScope>>> = Rc::new(RefCell::new(None));
        let inner_blank_captured: Rc<RefCell<Option<AgentBoundaryScope>>> =
            Rc::new(RefCell::new(None));
        let inner_override_captured: Rc<RefCell<Option<AgentBoundaryScope>>> =
            Rc::new(RefCell::new(None));
        let outside_captured: Rc<RefCell<Option<AgentBoundaryScope>>> = Rc::new(RefCell::new(None));

        #[component]
        fn Probe(slot: Rc<RefCell<Option<AgentBoundaryScope>>>) -> Element {
            *slot.borrow_mut() = use_agent_boundary_scope();
            rsx! {}
        }

        #[component]
        fn App(
            outer_captured: Rc<RefCell<Option<AgentBoundaryScope>>>,
            inner_blank_captured: Rc<RefCell<Option<AgentBoundaryScope>>>,
            inner_override_captured: Rc<RefCell<Option<AgentBoundaryScope>>>,
            outside_captured: Rc<RefCell<Option<AgentBoundaryScope>>>,
        ) -> Element {
            rsx! {
                Probe { slot: outside_captured }
                AgentBoundary {
                    section: "Account",
                    sections: vec![String::from("Security")],
                    secret: true,
                    confirm: "Confirm security change?",
                    Probe { slot: outer_captured }
                    AgentBoundary {
                        section: "API Keys",
                        hidden: true,
                        secret: false,
                        inert: true,
                        confirm: "   ",
                        Probe { slot: inner_blank_captured }
                    }
                    AgentBoundary {
                        section: "Danger Zone",
                        confirm: "Permanently delete?",
                        Probe { slot: inner_override_captured }
                    }
                }
            }
        }

        let mut dom = VirtualDom::new_with_props(
            App,
            AppProps {
                outer_captured: Rc::clone(&outer_captured),
                inner_blank_captured: Rc::clone(&inner_blank_captured),
                inner_override_captured: Rc::clone(&inner_override_captured),
                outside_captured: Rc::clone(&outside_captured),
            },
        );
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert_eq!(*outside_captured.borrow(), None);

        let outer = outer_captured
            .borrow()
            .clone()
            .expect("outer scope captured");
        assert_eq!(
            outer.sections,
            vec![String::from("Account"), String::from("Security")]
        );
        assert!(!outer.hidden);
        assert!(outer.secret);
        assert!(!outer.inert);
        assert_eq!(outer.confirm.as_deref(), Some("Confirm security change?"));

        let inner_blank = inner_blank_captured
            .borrow()
            .clone()
            .expect("inner blank confirm scope captured");
        assert_eq!(
            inner_blank.sections,
            vec![
                String::from("Account"),
                String::from("Security"),
                String::from("API Keys"),
            ]
        );
        assert!(inner_blank.hidden);
        assert!(inner_blank.secret);
        assert!(inner_blank.inert);
        assert_eq!(
            inner_blank.confirm.as_deref(),
            Some("Confirm security change?")
        );

        let inner_override = inner_override_captured
            .borrow()
            .clone()
            .expect("inner override scope captured");
        assert_eq!(
            inner_override.sections,
            vec![
                String::from("Account"),
                String::from("Security"),
                String::from("Danger Zone"),
            ]
        );
        assert!(inner_override.secret);
        assert_eq!(
            inner_override.confirm.as_deref(),
            Some("Permanently delete?")
        );
    }

    #[test]
    fn agent_boundary_propagates_reactive_updates_without_wrapper_dom() {
        let captured: Rc<RefCell<Option<AgentBoundaryScope>>> = Rc::new(RefCell::new(None));
        let hidden_sig_holder: Rc<RefCell<Option<Signal<bool>>>> = Rc::new(RefCell::new(None));

        #[component]
        fn Probe(slot: Rc<RefCell<Option<AgentBoundaryScope>>>) -> Element {
            *slot.borrow_mut() = use_agent_boundary_scope();
            rsx! { span { "leaf" } }
        }

        #[component]
        fn App(
            captured: Rc<RefCell<Option<AgentBoundaryScope>>>,
            hidden_sig_holder: Rc<RefCell<Option<Signal<bool>>>>,
        ) -> Element {
            let hidden = use_signal(|| false);
            *hidden_sig_holder.borrow_mut() = Some(hidden);
            rsx! {
                AgentBoundary {
                    section: "Settings",
                    hidden: hidden(),
                    Probe { slot: captured }
                }
            }
        }

        let mut dom = VirtualDom::new_with_props(
            App,
            AppProps {
                captured: Rc::clone(&captured),
                hidden_sig_holder: Rc::clone(&hidden_sig_holder),
            },
        );
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert!(!captured.borrow().as_ref().expect("captured").hidden);

        let mut hidden_sig = hidden_sig_holder.borrow().expect("signal");
        hidden_sig.set(true);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert!(captured.borrow().as_ref().expect("captured").hidden);
    }
}

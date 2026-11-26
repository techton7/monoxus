use std::cell::{Cell, RefCell};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortalOp {
    Teleport {
        element_id: String,
        host_id: Option<String>,
    },
    Remove {
        element_id: String,
    },
}

thread_local! {
    static PORTAL_OP_RECORDING: Cell<bool> = const { Cell::new(false) };
    static PORTAL_OP_LOG: RefCell<Vec<PortalOp>> = const { RefCell::new(Vec::new()) };
}

#[doc(hidden)]
pub fn start_portal_op_recording() {
    PORTAL_OP_LOG.with(|log| log.borrow_mut().clear());
    PORTAL_OP_RECORDING.with(|flag| flag.set(true));
}

#[doc(hidden)]
pub fn take_portal_ops() -> Vec<PortalOp> {
    PORTAL_OP_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}

#[doc(hidden)]
pub fn stop_portal_op_recording() {
    PORTAL_OP_RECORDING.with(|flag| flag.set(false));
    PORTAL_OP_LOG.with(|log| log.borrow_mut().clear());
}

pub(crate) fn teleport_element_to_host(element_id: &str, host_id: Option<&str>) {
    if PORTAL_OP_RECORDING.with(|flag| flag.get()) {
        PORTAL_OP_LOG.with(|log| {
            log.borrow_mut().push(PortalOp::Teleport {
                element_id: element_id.to_owned(),
                host_id: host_id.map(str::to_owned),
            });
        });
    }
    let doc = oxidase::dom::document();
    doc.teleport_portal_to_host(element_id, host_id);
}

pub(crate) fn remove_element_by_id(element_id: &str) {
    if PORTAL_OP_RECORDING.with(|flag| flag.get()) {
        PORTAL_OP_LOG.with(|log| {
            log.borrow_mut().push(PortalOp::Remove {
                element_id: element_id.to_owned(),
            });
        });
    }
    let doc = oxidase::dom::document();
    doc.remove_portal_element_by_id(element_id);
}

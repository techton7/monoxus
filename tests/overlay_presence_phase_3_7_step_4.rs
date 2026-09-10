use monoxus::{
    alert_dialog::{AlertDialog, AlertDialogRuntime},
    dialog::{Dialog, DialogRuntime},
    foundation::{overlay::PresenceState, shared::ScopeHandle},
    popover::{Popover, PopoverRuntime},
    select::{Select, SelectRuntime},
    tooltip::{Tooltip, TooltipRuntime},
};

#[test]
fn phase_3_7_step_4_overlay_runtime_render_gate_handoff_is_public() {
    let _ = DialogRuntime::should_render_portal;
    let _ = DialogRuntime::should_render_overlay;
    let _ = DialogRuntime::should_render_content;
    let _ = AlertDialogRuntime::should_render_portal;
    let _ = AlertDialogRuntime::should_render_overlay;
    let _ = AlertDialogRuntime::should_render_content;
    let _ = PopoverRuntime::should_render_portal;
    let _ = PopoverRuntime::should_render_content;
    let _ = TooltipRuntime::should_render_portal;
    let _ = TooltipRuntime::should_render_content;
    let _ = SelectRuntime::should_render_portal;
    let _ = SelectRuntime::should_render_content;
}

#[test]
fn phase_3_7_step_4_overlay_models_still_publish_retained_mount_contract() {
    let dialog = Dialog::new(ScopeHandle::root("dialog").child("root"), true);
    let alert = AlertDialog::new(ScopeHandle::root("alert").child("root"), true);
    let popover = Popover::new(ScopeHandle::root("popover").child("root"), true);
    let tooltip = Tooltip::new(ScopeHandle::root("tooltip").child("root"), true);
    let select = Select::new(ScopeHandle::root("select").child("root")).with_open(true);

    for state in [
        dialog.lifecycle().presence().state(),
        alert.lifecycle().presence().state(),
        popover.lifecycle().presence().state(),
        tooltip.lifecycle().presence().state(),
        select.presence().state(),
    ] {
        assert_eq!(state, PresenceState::Mounted);
    }

    assert!(dialog.lifecycle().presence().retain_mount());
    assert!(alert.lifecycle().presence().retain_mount());
    assert!(popover.lifecycle().presence().retain_mount());
    assert!(tooltip.lifecycle().presence().retain_mount());
    assert!(select.presence().retain_mount());
}

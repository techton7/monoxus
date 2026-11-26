pub use crate::foundation::shared::{
    EscapeKeyDownEvent, InteractOutsideEvent as ComboboxInteractOutsideEvent,
};

use crate::foundation::browser::{
    get_element_client_rect, get_element_scroll_metrics, set_element_scroll_position,
    spawn_root_task,
};

use super::{
    runtime::ComboboxRuntime,
    state::compute_viewport_scroll_target,
    types::ComboboxScrollAlignment,
};

impl ComboboxRuntime {
    pub fn start_dismiss_monitor(&self) {
        self.state.dismiss.reset_interaction();
    }

    pub fn stop_dismiss_monitor(&self) {
        self.state.dismiss.stop();
    }

    pub fn start_position_monitor(&self) {
        self.state.floating.refresh();
    }

    pub fn stop_position_monitor(&self) {
        self.state.floating.clear();
    }

    pub fn recalculate_floating_position_sync(&self) -> bool {
        self.state.floating.recalculate_sync()
    }

    pub async fn recalculate_floating_position(&self) {
        self.state.floating.recalculate_with_async_fallback().await;
    }
}

pub(crate) fn scroll_item_in_viewport(
    viewport_id: &str,
    fallback_content_id: &str,
    item_id: &str,
    alignment: ComboboxScrollAlignment,
) {
    let viewport_id = viewport_id.to_owned();
    let fallback_content_id = fallback_content_id.to_owned();
    let item_id = item_id.to_owned();

    spawn_root_task(async move {
        for attempt in 0..3 {
            if try_scroll_item_in_viewport(
                &viewport_id,
                &fallback_content_id,
                &item_id,
                alignment,
            ) {
                break;
            }
            if attempt < 2 {
                let _ = oxidase::frame::next_frame().await;
            }
        }
    });
}

fn try_scroll_item_in_viewport(
    viewport_id: &str,
    fallback_content_id: &str,
    item_id: &str,
    alignment: ComboboxScrollAlignment,
) -> bool {
    let container_id = if get_element_client_rect(viewport_id).is_some() {
        viewport_id
    } else {
        fallback_content_id
    };

    let Some(container_rect) = get_element_client_rect(container_id) else {
        return false;
    };
    let Some(item_rect) = get_element_client_rect(item_id) else {
        return false;
    };
    let metrics = get_element_scroll_metrics(container_id).unwrap_or_default();

    if let Some(target_top) = compute_viewport_scroll_target(
        container_rect.top(),
        container_rect.bottom(),
        container_rect.height,
        metrics.scroll_top,
        item_rect.top(),
        item_rect.bottom(),
        item_rect.height,
        alignment,
    ) {
        set_element_scroll_position(container_id, metrics.scroll_left, target_top);
    }
    true
}

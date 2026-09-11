mod js {
    dioxus_js_interop::bind_js!("src/select/browser.ts"::*);
}

pub(crate) async fn get_document_option_order(content_id: &str) -> Vec<String> {
    js::get_document_option_order(content_id)
        .await
        .unwrap_or_default()
}

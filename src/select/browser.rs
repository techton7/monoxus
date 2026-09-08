use dioxus::prelude::*;
use dioxus_use_js::use_js;

use_js!("src/select/browser.js"::*);

pub(crate) async fn get_document_option_order(content_id: &str) -> Vec<String> {
    let content_id = content_id.to_string();
    let res: Result<Vec<String>, _> = getDocumentOptionOrder(content_id).await;
    res.unwrap_or_default()
}

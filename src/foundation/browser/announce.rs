//! Shared accessible polite live-region announcer (`S2-LD-09`).
//!
//! Lazily manages a single visually hidden `<div id="monoxus-live-announcer"
//! role="status" aria-live="polite" aria-atomic="true" data-monoxus-announcer="">`
//! attached to `document.body` on `wasm32`, excluded from modal DOM shielding
//! (`shield_external_dom`), and paired with a thread-local test recorder for
//! deterministic native `VirtualDom` verification.

use std::cell::RefCell;
use std::rc::Rc;

/// DOM element ID for the shared polite live-region announcer.
pub const LIVE_ANNOUNCER_ID: &str = "monoxus-live-announcer";

type RecorderBuf = Rc<RefCell<Vec<String>>>;

thread_local! {
    static ANNOUNCER_RECORDER: RefCell<Option<RecorderBuf>> = const { RefCell::new(None) };
}

/// RAII guard that captures calls to [`announce_polite`] on the current thread
/// and restores any previous recorder on drop.
pub struct AnnouncerRecorderGuard {
    buf: RecorderBuf,
    previous: Option<RecorderBuf>,
}

impl AnnouncerRecorderGuard {
    /// Returns a cloned snapshot of all recorded announcements.
    #[inline]
    pub fn messages(&self) -> Vec<String> {
        self.buf.borrow().clone()
    }

    /// Drains and returns all recorded announcements.
    #[inline]
    pub fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.buf.borrow_mut())
    }
}

impl Drop for AnnouncerRecorderGuard {
    fn drop(&mut self) {
        let prev = self.previous.take();
        let _ = ANNOUNCER_RECORDER.try_with(|slot| {
            *slot.borrow_mut() = prev;
        });
    }
}

/// Replaces the thread-local announcer recorder buffer, returning the previous
/// recorder (if any).
pub fn set_announcer_recorder(recorder: Option<RecorderBuf>) -> Option<RecorderBuf> {
    ANNOUNCER_RECORDER.with(|slot| std::mem::replace(&mut *slot.borrow_mut(), recorder))
}

/// Installs a thread-local announcer recorder and returns an RAII guard that
/// restores the previous recorder on drop.
pub fn capture_announcements() -> AnnouncerRecorderGuard {
    let buf = Rc::new(RefCell::new(Vec::new()));
    let previous = set_announcer_recorder(Some(Rc::clone(&buf)));
    AnnouncerRecorderGuard { buf, previous }
}

/// Announces `text` via the shared polite live region (`aria-live="polite"`).
///
/// Empty or whitespace-only messages are ignored.
pub fn announce_polite(text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    ANNOUNCER_RECORDER.with(|slot| {
        if let Some(buf) = slot.borrow().as_ref() {
            buf.borrow_mut().push(trimmed.to_owned());
        }
    });

    #[cfg(target_arch = "wasm32")]
    {
        let js_msg = encode_js_string(trimmed);
        let script = format!(
            r#"(function() {{
  if (typeof document === "undefined" || !document.body) return;
  var id = "{id}";
  var el = document.getElementById(id);
  if (!el) {{
    el = document.createElement("div");
    el.id = id;
    el.setAttribute("role", "status");
    el.setAttribute("aria-live", "polite");
    el.setAttribute("aria-atomic", "true");
    el.setAttribute("data-monoxus-announcer", "");
    el.style.position = "fixed";
    el.style.width = "1px";
    el.style.height = "1px";
    el.style.padding = "0";
    el.style.margin = "-1px";
    el.style.overflow = "hidden";
    el.style.clip = "rect(0, 0, 0, 0)";
    el.style.whiteSpace = "nowrap";
    el.style.border = "0";
    document.body.appendChild(el);
  }}
  var msg = {js_msg};
  el.textContent = "";
  if (typeof requestAnimationFrame === "function") {{
    requestAnimationFrame(function() {{ el.textContent = msg; }});
  }} else {{
    el.textContent = msg;
  }}
}})();"#,
            id = LIVE_ANNOUNCER_ID,
            js_msg = js_msg,
        );
        let _ = dioxus::document::eval(&script);
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn encode_js_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 2);
    out.push('"');
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::{
        LIVE_ANNOUNCER_ID, announce_polite, capture_announcements, encode_js_string,
    };

    #[test]
    fn announce_polite_records_non_empty_and_skips_whitespace() {
        let guard = capture_announcements();
        announce_polite("");
        announce_polite("   \n\t  ");
        announce_polite("Loading");
        announce_polite("  Saved changes  ");

        assert_eq!(guard.messages(), vec!["Loading", "Saved changes"]);
        assert_eq!(guard.take(), vec!["Loading", "Saved changes"]);
        assert!(guard.messages().is_empty());
    }

    #[test]
    fn nested_announcer_guard_restores_outer_recorder() {
        let outer = capture_announcements();
        announce_polite("outer-1");

        {
            let inner = capture_announcements();
            announce_polite("inner-1");
            assert_eq!(inner.messages(), vec!["inner-1"]);
        }

        announce_polite("outer-2");
        assert_eq!(outer.messages(), vec!["outer-1", "outer-2"]);
    }

    #[test]
    fn js_string_encoder_escapes_special_characters() {
        assert_eq!(LIVE_ANNOUNCER_ID, "monoxus-live-announcer");
        assert_eq!(
            encode_js_string("hi \"quoted\" <tag>\n\\"),
            "\"hi \\\"quoted\\\" \\u003ctag\\u003e\\n\\\\\""
        );
    }
}

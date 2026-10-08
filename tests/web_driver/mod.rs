//! Web Verification Automation Runner for Monoxus Playground.
//!
//! Provides automation and driver helpers for verifying Web target (`wasm32`)
//! primitives using `ego-browser` against `dx serve --example playground --web --open false --port 8080`.
//!
//! # Invariants:
//! 1. All wait/timeout durations are strictly in **seconds** (e.g. 0.3s, 0.5s), NEVER milliseconds.
//! 2. All browser interactions execute via compact IIFEs without auxiliary `.js` files.
//! 3. Single `taskSpace` session per verification run, with proper cleanup.

use std::process::Command;

/// Configuration for the Web Driver runner.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WebDriverConfig {
    pub host: String,
    pub port: u16,
    pub default_wait_sec: f32,
}

impl Default for WebDriverConfig {
    fn default() -> Self {
        Self {
            host: "http://127.0.0.1".to_string(),
            port: 8080,
            default_wait_sec: 0.3,
        }
    }
}

impl WebDriverConfig {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            ..Default::default()
        }
    }

    pub fn base_url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn nav_link_id(component: &str) -> String {
        let slug = component.to_lowercase().replace(' ', "-");
        format!("nav-link-{slug}")
    }
}

/// Script Generator: Formats compact IIFEs for in-browser execution via `page.evaluate(...)`.
pub struct ScriptBuilder;

impl ScriptBuilder {
    /// Formats a compact JS IIFE string.
    pub fn iife(body: &str) -> String {
        format!("(() => {{ {body} }})()")
    }

    /// Formats a sub-second wait helper call (e.g. `await wait(0.3);`).
    pub fn wait_call(seconds: f32) -> String {
        format!("await wait({seconds:.2});")
    }

    /// JS snippet to click a navigation link by component slug.
    pub fn navigate_nav_link(component: &str) -> String {
        let nav_id = WebDriverConfig::nav_link_id(component);
        Self::iife(&format!(
            "const btn = document.querySelector('#{nav_id}'); if (btn) btn.click();"
        ))
    }

    /// JS snippet to query the currently active element ID (`document.activeElement.id`).
    pub fn eval_active_element_id() -> String {
        Self::iife("return document.activeElement ? document.activeElement.id : '';")
    }

    /// JS snippet to query an element's `data-state` attribute.
    pub fn get_data_state(selector: &str) -> String {
        Self::iife(&format!(
            "const el = document.querySelector('{selector}'); return el ? el.getAttribute('data-state') : null;"
        ))
    }

    /// JS snippet to focus and click an element matching `selector`.
    pub fn click_element(selector: &str) -> String {
        Self::iife(&format!(
            "const el = document.querySelector('{selector}'); if (el) {{ el.focus(); el.click(); }}"
        ))
    }

    /// JS snippet to dispatch KeyDown and KeyUp events with explicit key and code.
    pub fn dispatch_key(selector: &str, key: &str, code: &str) -> String {
        Self::iife(&format!(
            "const el = document.querySelector('{selector}'); if (el) {{ \
                el.dispatchEvent(new KeyboardEvent('keydown', {{ key: '{key}', code: '{code}', bubbles: true }})); \
                el.dispatchEvent(new KeyboardEvent('keyup', {{ key: '{key}', code: '{code}', bubbles: true }})); \
            }}"
        ))
    }

    /// Compact JS actions for Collapsible web verification:
    /// Navigates #nav-link-collapsible, clicks #collapsible-trigger, verifies data-state="open", and checks focus.
    pub fn verify_collapsible_actions() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            Self::navigate_nav_link("collapsible"),
            Self::wait_call(0.3),
            Self::click_element("#collapsible-trigger"),
            Self::wait_call(0.3),
            Self::iife(
                "const el = document.querySelector('#collapsible-trigger'); \
                 const state = el ? el.getAttribute('data-state') : null; \
                 const activeId = document.activeElement ? document.activeElement.id : ''; \
                 console.log(JSON.stringify({ component: 'collapsible', dataState: state, activeId }));"
            )
        )
    }

    /// Compact JS actions for Accordion web verification:
    /// Navigates #nav-link-accordion, clicks #accordion-trigger-1, and verifies data-state="open".
    pub fn verify_accordion_actions() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            Self::navigate_nav_link("accordion"),
            Self::wait_call(0.3),
            Self::click_element("#accordion-trigger-1"),
            Self::wait_call(0.3),
            Self::iife(
                "const el = document.querySelector('#accordion-trigger-1'); \
                 const state = el ? el.getAttribute('data-state') : null; \
                 console.log(JSON.stringify({ component: 'accordion', dataState: state }));"
            )
        )
    }

    /// Compact JS actions for Tabs web verification:
    /// Navigates #nav-link-tabs, clicks #tabs-trigger-2, verifies data-state="active", verifies active element and content.
    pub fn verify_tabs_actions() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            Self::navigate_nav_link("tabs"),
            Self::wait_call(0.3),
            Self::click_element("#tabs-trigger-2"),
            Self::wait_call(0.3),
            Self::iife(
                "const el = document.querySelector('#tabs-trigger-2'); \
                 const state = el ? el.getAttribute('data-state') : null; \
                 const activeId = document.activeElement ? document.activeElement.id : ''; \
                 const content2 = document.querySelector('#tabs-content-2'); \
                 const contentState = content2 ? content2.getAttribute('data-state') : null; \
                 console.log(JSON.stringify({ component: 'tabs', dataState: state, activeId, contentState }));"
            )
        )
    }

    /// Compact JS actions for Popover web verification:
    /// Navigates #nav-link-popover, clicks #popover-trigger, verifies open state & content, clicks #popover-close, verifies dismissal.
    pub fn verify_popover_actions() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            Self::navigate_nav_link("popover"),
            Self::wait_call(0.3),
            Self::click_element("#popover-trigger"),
            Self::wait_call(0.3),
            Self::iife(
                "const trigger = document.querySelector('#popover-trigger'); \
                 const content = document.querySelector('#popover-content'); \
                 const triggerState = trigger ? trigger.getAttribute('data-state') : null; \
                 const contentState = content ? content.getAttribute('data-state') : null; \
                 console.log(JSON.stringify({ component: 'popover', step: 'open', triggerState, contentState, contentMounted: !!content }));"
            ),
            Self::click_element("#popover-close"),
            Self::wait_call(0.3),
            Self::iife(
                "const content = document.querySelector('#popover-content'); \
                 console.log(JSON.stringify({ component: 'popover', step: 'closed', contentMounted: !!content }));"
            ),
        )
    }

    /// Compact JS actions for Tooltip web verification:
    /// Navigates #nav-link-tooltip, triggers hover/focus on #tooltip-trigger, waits for delay, verifies #tooltip-content & #tooltip-arrow.
    pub fn verify_tooltip_actions() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            Self::navigate_nav_link("tooltip"),
            Self::wait_call(0.3),
            Self::iife(
                "const trigger = document.querySelector('#tooltip-trigger'); \
                 if (trigger) { \
                     trigger.focus(); \
                     trigger.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true })); \
                 }"
            ),
            Self::wait_call(0.5),
            Self::iife(
                "const trigger = document.querySelector('#tooltip-trigger'); \
                 const content = document.querySelector('#tooltip-content'); \
                 const arrow = document.querySelector('#tooltip-arrow'); \
                 const triggerState = trigger ? trigger.getAttribute('data-state') : null; \
                 const contentState = content ? content.getAttribute('data-state') : null; \
                 console.log(JSON.stringify({ \
                     component: 'tooltip', \
                     triggerState, \
                     contentState, \
                     contentMounted: !!content, \
                     arrowMounted: !!arrow \
                 }));"
            ),
        )
    }

    /// Compact JS actions for Dialog web verification:
    /// Navigates #nav-link-dialog, clicks #dialog-trigger, verifies overlay and content, clicks #dialog-close, verifies dismissal.
    pub fn verify_dialog_actions() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            Self::navigate_nav_link("dialog"),
            Self::wait_call(0.3),
            Self::click_element("#dialog-trigger"),
            Self::wait_call(0.3),
            Self::iife(
                "const overlay = document.querySelector('#dialog-overlay'); \
                 const content = document.querySelector('#dialog-content'); \
                 const state = content ? content.getAttribute('data-state') : null; \
                 console.log(JSON.stringify({ \
                     component: 'dialog', \
                     step: 'open', \
                     overlayMounted: !!overlay, \
                     contentMounted: !!content, \
                     state \
                 }));"
            ),
            Self::click_element("#dialog-close"),
            Self::wait_call(0.3),
            Self::iife(
                "const content = document.querySelector('#dialog-content'); \
                 console.log(JSON.stringify({ \
                     component: 'dialog', \
                     step: 'closed', \
                     contentMounted: !!content \
                 }));"
            ),
        )
    }

    /// Compact JS actions for AlertDialog web verification:
    /// Navigates #nav-link-alert-dialog, clicks #alert-dialog-trigger, verifies content and cancel, clicks #alert-dialog-cancel, verifies dismissal.
    pub fn verify_alert_dialog_actions() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            Self::navigate_nav_link("alert-dialog"),
            Self::wait_call(0.3),
            Self::click_element("#alert-dialog-trigger"),
            Self::wait_call(0.3),
            Self::iife(
                "const content = document.querySelector('#alert-dialog-content'); \
                 const cancel = document.querySelector('#alert-dialog-cancel'); \
                 const state = content ? content.getAttribute('data-state') : null; \
                 console.log(JSON.stringify({ \
                     component: 'alert-dialog', \
                     step: 'open', \
                     contentMounted: !!content, \
                     cancelMounted: !!cancel, \
                     state \
                 }));"
            ),
            Self::click_element("#alert-dialog-cancel"),
            Self::wait_call(0.3),
            Self::iife(
                "const content = document.querySelector('#alert-dialog-content'); \
                 console.log(JSON.stringify({ \
                     component: 'alert-dialog', \
                     step: 'closed', \
                     contentMounted: !!content \
                 }));"
            ),
        )
    }
}

/// Headless browser execution driver using `ego-browser`.
#[allow(dead_code)]
pub struct EgoWebDriver {
    config: WebDriverConfig,
}

#[allow(dead_code)]
impl EgoWebDriver {
    pub fn new(config: WebDriverConfig) -> Self {
        Self { config }
    }

    pub fn default_local() -> Self {
        Self::new(WebDriverConfig::default())
    }

    pub fn config(&self) -> &WebDriverConfig {
        &self.config
    }

    /// Generates a complete standalone Node.js script for `ego-browser` adhering to strict invariants.
    pub fn generate_runner_script(&self, task_name: &str, scenario_actions_js: &str) -> String {
        let base_url = self.config.base_url();
        let default_wait = self.config.default_wait_sec;

        format!(
            r#"const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));
const task = await taskSpace("{task_name}");
const page = task.page("p1");
await page.goto("{base_url}");
await wait({default_wait:.2});

{scenario_actions_js}

await task.finish({{ keep: [] }});
"#
        )
    }

    /// Generates runner script for Collapsible web verification.
    pub fn generate_collapsible_verification_script(&self) -> String {
        self.generate_runner_script(
            "verify-monoxus-collapsible",
            &ScriptBuilder::verify_collapsible_actions(),
        )
    }

    /// Generates runner script for Accordion web verification.
    pub fn generate_accordion_verification_script(&self) -> String {
        self.generate_runner_script(
            "verify-monoxus-accordion",
            &ScriptBuilder::verify_accordion_actions(),
        )
    }

    /// Generates runner script for Tabs web verification.
    pub fn generate_tabs_verification_script(&self) -> String {
        self.generate_runner_script("verify-monoxus-tabs", &ScriptBuilder::verify_tabs_actions())
    }

    /// Generates runner script for Popover web verification.
    pub fn generate_popover_verification_script(&self) -> String {
        self.generate_runner_script(
            "verify-monoxus-popover",
            &ScriptBuilder::verify_popover_actions(),
        )
    }

    /// Generates runner script for Tooltip web verification.
    pub fn generate_tooltip_verification_script(&self) -> String {
        self.generate_runner_script(
            "verify-monoxus-tooltip",
            &ScriptBuilder::verify_tooltip_actions(),
        )
    }

    /// Generates runner script for Dialog web verification.
    pub fn generate_dialog_verification_script(&self) -> String {
        self.generate_runner_script(
            "verify-monoxus-dialog",
            &ScriptBuilder::verify_dialog_actions(),
        )
    }

    /// Generates runner script for AlertDialog web verification.
    pub fn generate_alert_dialog_verification_script(&self) -> String {
        self.generate_runner_script(
            "verify-monoxus-alert-dialog",
            &ScriptBuilder::verify_alert_dialog_actions(),
        )
    }

    /// Executes a generated script through `ego-browser nodejs`.
    pub fn run_script(&self, script: &str) -> Result<String, String> {
        let mut child = Command::new("ego-browser")
            .arg("nodejs")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn ego-browser: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin
                .write_all(script.as_bytes())
                .map_err(|e| format!("Failed to write script to ego-browser stdin: {e}"))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("ego-browser failed to execute: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ego-browser exited with error: {stderr}"));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_driver_config_new() {
        let cfg = WebDriverConfig::new(3000);
        assert_eq!(cfg.port, 3000);
        assert_eq!(cfg.base_url(), "http://127.0.0.1:3000");
    }

    #[test]
    fn test_script_builder_nav_link_id() {
        assert_eq!(WebDriverConfig::nav_link_id("collapsible"), "nav-link-collapsible");
        assert_eq!(WebDriverConfig::nav_link_id("alert dialog"), "nav-link-alert-dialog");
        assert_eq!(WebDriverConfig::nav_link_id("Accordion"), "nav-link-accordion");
    }

    #[test]
    fn test_script_builder_compact_iifes() {
        let nav_script = ScriptBuilder::navigate_nav_link("collapsible");
        assert!(nav_script.starts_with("(() => {"));
        assert!(nav_script.ends_with("})()"));
        assert!(nav_script.contains("#nav-link-collapsible"));

        let active_id_script = ScriptBuilder::eval_active_element_id();
        assert!(active_id_script.contains("document.activeElement.id"));

        let state_script = ScriptBuilder::get_data_state("#collapsible-trigger");
        assert!(state_script.contains("getAttribute('data-state')"));
        assert!(state_script.contains("#collapsible-trigger"));

        let click_script = ScriptBuilder::click_element("#collapsible-trigger");
        assert!(click_script.contains(".click()"));
        assert!(click_script.contains("#collapsible-trigger"));

        let key_script = ScriptBuilder::dispatch_key("#collapsible-trigger", "Enter", "Enter");
        assert!(key_script.contains("KeyboardEvent"));
        assert!(key_script.contains("'Enter'"));
    }

    #[test]
    fn test_wait_durations_are_strictly_seconds() {
        let wait_call = ScriptBuilder::wait_call(0.5);
        assert_eq!(wait_call, "await wait(0.50);");
        // Verify no millisecond integer literals (e.g. 500) are generated
        assert!(!wait_call.contains("500"));
    }

    #[test]
    fn test_runner_script_generation_invariants() {
        let driver = EgoWebDriver::default_local();
        let script = driver.generate_runner_script("test-task", "// actions here");

        // Strict invariant 1: wait function converts seconds to ms (s * 1000)
        assert!(script.contains("const wait = (s) => new Promise((r) => setTimeout(r, s * 1000));"));

        // Strict invariant 2: sub-second wait
        assert!(script.contains("await wait(0.30);"));

        // Strict invariant 3: clean session finish
        assert!(script.contains("await task.finish({ keep: [] });"));

        // Points to expected URL
        assert!(script.contains("http://127.0.0.1:8080"));
    }

    #[test]
    fn test_non_floating_verification_script_generators() {
        let driver = EgoWebDriver::default_local();

        // 1. Collapsible script invariants
        let col_script = driver.generate_collapsible_verification_script();
        assert!(col_script.contains("#nav-link-collapsible"));
        assert!(col_script.contains("#collapsible-trigger"));
        assert!(col_script.contains("dataState"));
        assert!(col_script.contains("activeId"));
        assert!(!col_script.contains("500"));

        // 2. Accordion script invariants
        let acc_script = driver.generate_accordion_verification_script();
        assert!(acc_script.contains("#nav-link-accordion"));
        assert!(acc_script.contains("#accordion-trigger-1"));
        assert!(acc_script.contains("dataState"));
        assert!(!acc_script.contains("500"));

        // 3. Tabs script invariants
        let tabs_script = driver.generate_tabs_verification_script();
        assert!(tabs_script.contains("#nav-link-tabs"));
        assert!(tabs_script.contains("#tabs-trigger-2"));
        assert!(tabs_script.contains("#tabs-content-2"));
        assert!(tabs_script.contains("dataState"));
        assert!(tabs_script.contains("contentState"));
        assert!(!tabs_script.contains("500"));
    }

    #[test]
    fn test_overlay_and_floating_verification_script_generators() {
        let driver = EgoWebDriver::default_local();

        // 1. Popover script invariants: strictly sub-second wait, IIFE, correct IDs
        let pop_script = driver.generate_popover_verification_script();
        assert!(pop_script.contains("#nav-link-popover"));
        assert!(pop_script.contains("#popover-trigger"));
        assert!(pop_script.contains("#popover-content"));
        assert!(pop_script.contains("#popover-close"));
        assert!(pop_script.contains("(() => {"));
        assert!(pop_script.contains("await wait(0.30);"));
        assert!(!pop_script.contains("300"));
        assert!(!pop_script.contains("500"));

        // 2. Tooltip script invariants: strictly sub-second wait (0.50 for delay), IIFE, correct IDs
        let tt_script = driver.generate_tooltip_verification_script();
        assert!(tt_script.contains("#nav-link-tooltip"));
        assert!(tt_script.contains("#tooltip-trigger"));
        assert!(tt_script.contains("#tooltip-content"));
        assert!(tt_script.contains("#tooltip-arrow"));
        assert!(tt_script.contains("(() => {"));
        assert!(tt_script.contains("await wait(0.50);"));
        assert!(!tt_script.contains("450"));
        assert!(!tt_script.contains("500"));

        // 3. Dialog script invariants: strictly sub-second wait, IIFE, correct IDs
        let dlg_script = driver.generate_dialog_verification_script();
        assert!(dlg_script.contains("#nav-link-dialog"));
        assert!(dlg_script.contains("#dialog-trigger"));
        assert!(dlg_script.contains("#dialog-overlay"));
        assert!(dlg_script.contains("#dialog-content"));
        assert!(dlg_script.contains("#dialog-close"));
        assert!(dlg_script.contains("(() => {"));
        assert!(dlg_script.contains("await wait(0.30);"));
        assert!(!dlg_script.contains("300"));

        // 4. AlertDialog script invariants: strictly sub-second wait, IIFE, correct IDs
        let alert_script = driver.generate_alert_dialog_verification_script();
        assert!(alert_script.contains("#nav-link-alert-dialog"));
        assert!(alert_script.contains("#alert-dialog-trigger"));
        assert!(alert_script.contains("#alert-dialog-content"));
        assert!(alert_script.contains("#alert-dialog-cancel"));
        assert!(alert_script.contains("(() => {"));
        assert!(alert_script.contains("await wait(0.30);"));
        assert!(!alert_script.contains("300"));
    }
}

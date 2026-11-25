use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
const THEME_STORAGE_KEY: &str = "monoxus-theme";

/// Visual color scheme mode for the documentation site.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }

    pub const fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

/// Shared context holding the active theme signal.
#[derive(Clone, Copy)]
pub struct ThemeContext(pub Signal<ThemeMode>);

impl ThemeContext {
    pub fn mode(&self) -> ThemeMode {
        (self.0)()
    }

    pub fn is_dark(&self) -> bool {
        self.mode().is_dark()
    }

    #[allow(dead_code)]
    pub fn set(&mut self, mode: ThemeMode) {
        self.0.set(mode);
    }

    pub fn toggle(&mut self) {
        let next = self.mode().toggle();
        self.0.set(next);
    }
}

/// Provides light/dark theme state synchronized with `localStorage` and `<html>` `.dark`.
#[component]
pub fn ThemeProvider(children: Element) -> Element {
    let theme_signal = use_signal(read_initial_theme);

    use_effect(move || {
        apply_theme_to_dom(theme_signal());
    });

    use_context_provider(|| ThemeContext(theme_signal));

    rsx! {
        {children}
    }
}

/// Consumes the active [`ThemeContext`].
pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>()
}

/// Copies `text` to the browser clipboard when running on `wasm32`.
pub fn copy_to_clipboard(text: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.navigator().clipboard().write_text(text);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = text;
    }
}

fn read_initial_theme() -> ThemeMode {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(saved)) = storage.get_item(THEME_STORAGE_KEY) {
                    return match saved.as_str() {
                        "dark" => ThemeMode::Dark,
                        _ => ThemeMode::Light,
                    };
                }
            }
            if let Ok(Some(mql)) = window.match_media("(prefers-color-scheme: dark)") {
                if mql.matches() {
                    return ThemeMode::Dark;
                }
            }
        }
    }
    ThemeMode::Light
}

fn apply_theme_to_dom(mode: ThemeMode) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(root) = document.document_element() {
                    let class_list = root.class_list();
                    if mode.is_dark() {
                        let _ = class_list.add_1("dark");
                    } else {
                        let _ = class_list.remove_1("dark");
                    }
                }
            }
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(THEME_STORAGE_KEY, mode.as_str());
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = mode;
    }
}

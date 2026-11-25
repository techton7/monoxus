#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TocItem {
    pub id: &'static str,
    pub title: &'static str,
    pub level: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkdownDoc {
    pub section: &'static str,
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub html: &'static str,
    pub raw_md: &'static str,
    pub toc: &'static [TocItem],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchEntry {
    pub title: &'static str,
    pub description: &'static str,
    pub section: &'static str,
    pub path: &'static str,
    pub keywords: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavItem {
    pub title: &'static str,
    pub path: &'static str,
    pub badge: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavGroup {
    pub title: &'static str,
    pub items: &'static [NavItem],
}

include!(concat!(env!("OUT_DIR"), "/docs_gen.rs"));

static SIDEBAR_GROUPS: &[NavGroup] = &[
    NavGroup {
        title: "Overview",
        items: &[
            NavItem {
                title: "Introduction",
                path: "/docs/overview/introduction",
                badge: None,
            },
            NavItem {
                title: "Getting Started",
                path: "/docs/overview/getting-started",
                badge: None,
            },
            NavItem {
                title: "Accessibility",
                path: "/docs/overview/accessibility",
                badge: None,
            },
            NavItem {
                title: "Roadmap",
                path: "/docs/overview/roadmap",
                badge: Some("30+"),
            },
        ],
    },
    NavGroup {
        title: "Guides",
        items: &[
            NavItem {
                title: "Styling",
                path: "/docs/guides/styling",
                badge: None,
            },
            NavItem {
                title: "Composition",
                path: "/docs/guides/composition",
                badge: None,
            },
        ],
    },
    NavGroup {
        title: "Components",
        items: &[
            NavItem {
                title: "Button",
                path: "/docs/components/button",
                badge: None,
            },
            NavItem {
                title: "Dialog",
                path: "/docs/components/dialog",
                badge: None,
            },
            NavItem {
                title: "Select",
                path: "/docs/components/select",
                badge: None,
            },
            NavItem {
                title: "Tabs",
                path: "/docs/components/tabs",
                badge: None,
            },
            NavItem {
                title: "Tooltip",
                path: "/docs/components/tooltip",
                badge: None,
            },
        ],
    },
];

pub fn all_markdown_docs() -> &'static [MarkdownDoc] {
    ALL_MARKDOWN_DOCS
}

pub fn find_markdown_doc(section: &str, slug: &str) -> Option<&'static MarkdownDoc> {
    ALL_MARKDOWN_DOCS
        .iter()
        .find(|doc| doc.section == section && doc.slug == slug)
}

pub fn sidebar_groups() -> &'static [NavGroup] {
    SIDEBAR_GROUPS
}

pub fn flat_nav_items() -> Vec<NavItem> {
    let mut items = Vec::new();
    for group in SIDEBAR_GROUPS {
        for item in group.items {
            if !items.iter().any(|existing: &NavItem| existing.path == item.path) {
                items.push(*item);
            }
        }
    }
    items
}

pub fn prev_next_nav(current_path: &str) -> (Option<NavItem>, Option<NavItem>) {
    let items = flat_nav_items();
    let Some(idx) = items.iter().position(|item| item.path == current_path) else {
        return (None, None);
    };
    let prev = if idx > 0 { Some(items[idx - 1]) } else { None };
    let next = if idx + 1 < items.len() {
        Some(items[idx + 1])
    } else {
        None
    };
    (prev, next)
}

pub fn search_docs(query: &str) -> Vec<&'static SearchEntry> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return ALL_SEARCH_ENTRIES.iter().collect();
    }
    let terms: Vec<&str> = q.split_whitespace().collect();
    let mut scored: Vec<(usize, &'static SearchEntry)> = ALL_SEARCH_ENTRIES
        .iter()
        .filter_map(|entry| {
            let title_lc = entry.title.to_ascii_lowercase();
            let desc_lc = entry.description.to_ascii_lowercase();
            let kw_lc = entry.keywords.to_ascii_lowercase();
            let sec_lc = entry.section.to_ascii_lowercase();

            let mut score = 0usize;
            for term in &terms {
                let mut matched = false;
                if title_lc.contains(term) {
                    score += if title_lc == *term { 20 } else { 10 };
                    matched = true;
                }
                if desc_lc.contains(term) {
                    score += 4;
                    matched = true;
                }
                if kw_lc.contains(term) {
                    score += 2;
                    matched = true;
                }
                if sec_lc.contains(term) {
                    score += 1;
                    matched = true;
                }
                if !matched {
                    return None;
                }
            }
            Some((score, entry))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.title.cmp(b.1.title)));
    scored.into_iter().map(|(_, entry)| entry).collect()
}

pub fn highlight_rust_html(code: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while",
    ];
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len * 2);
    let mut i = 0;

    while i < len {
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            let start = i;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            push_span(&mut out, "hl-cm", &code[start..i]);
            continue;
        }
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(len);
            push_span(&mut out, "hl-cm", &code[start..i]);
            continue;
        }
        if bytes[i] == b'#'
            && i + 1 < len
            && (bytes[i + 1] == b'[' || (bytes[i + 1] == b'!' && i + 2 < len && bytes[i + 2] == b'['))
        {
            let start = i;
            let mut depth = 0usize;
            while i < len {
                if bytes[i] == b'[' {
                    depth += 1;
                } else if bytes[i] == b']' {
                    depth = depth.saturating_sub(1);
                    i += 1;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                i += 1;
            }
            push_span(&mut out, "hl-attr", &code[start..i]);
            continue;
        }
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            push_span(&mut out, "hl-str", &code[start..i]);
            continue;
        }
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < len
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.')
            {
                i += 1;
            }
            push_span(&mut out, "hl-num", &code[start..i]);
            continue;
        }
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let ident = &code[start..i];
            if i < len && bytes[i] == b'!' {
                i += 1;
                push_span(&mut out, "hl-mac", &code[start..i]);
            } else if KEYWORDS.contains(&ident) {
                push_span(&mut out, "hl-kw", ident);
            } else if ident.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                push_span(&mut out, "hl-ty", ident);
            } else if i < len && bytes[i] == b'(' {
                push_span(&mut out, "hl-fn", ident);
            } else {
                out.push_str(&html_escape(ident));
            }
            continue;
        }

        let ch = code[i..].chars().next().unwrap();
        out.push_str(&html_escape(&code[i..i + ch.len_utf8()]));
        i += ch.len_utf8();
    }

    out
}

fn push_span(out: &mut String, cls: &str, text: &str) {
    out.push_str("<span class=\"");
    out.push_str(cls);
    out.push_str("\">");
    out.push_str(&html_escape(text));
    out.push_str("</span>");
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

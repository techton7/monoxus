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
                badge: Some("44"),
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
        items: SIDEBAR_COMPONENT_ITEMS,
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

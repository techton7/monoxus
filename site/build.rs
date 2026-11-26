use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[allow(dead_code)]
#[path = "src/api_schema/mod.rs"]
mod api_schema;

struct TocEntry {
    id: String,
    title: String,
    level: u8,
}

struct CompiledDoc {
    section: String,
    slug: String,
    title: String,
    description: String,
    html: String,
    raw_md: String,
    toc: Vec<TocEntry>,
    keywords: String,
}

const CSS_TOKENS_SNIPPET: &str = r#"/* Monoxus primitives expose semantic state via data-* attributes */
[data-state="open"]   { /* open / active / expanded styling */ }
[data-state="closed"] { /* closed / collapsed styling */ }
[data-disabled]       { opacity: 0.5; pointer-events: none; }

/* Tailwind v4 theme variables used in demos */
@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-muted: var(--muted);
  --color-dark: var(--dark);
  --color-border-card: var(--border-card);
}"#;

fn main() {
    println!("cargo:rerun-if-changed=content");
    println!("cargo:rerun-if-changed=src/api_schema");
    println!("cargo:rerun-if-changed=src/demos");
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let content_dir = manifest_dir.join("content");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let public_dir = manifest_dir.join("public");
    let tailwind_css = manifest_dir.join("assets").join("tailwind.css");

    if !tailwind_css.exists() {
        if let Some(parent) = tailwind_css.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&tailwind_css, "/* placeholder until dx builds tailwind */\n")
            .expect("write placeholder assets/tailwind.css");
    }

    let mut md_files = Vec::new();
    collect_markdown_files(&content_dir, &mut md_files);
    md_files.sort();

    let mut docs = Vec::new();
    for path in md_files {
        let rel = path
            .strip_prefix(&content_dir)
            .expect("content file under content/");
        let mut components = rel.components();
        let section = components
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("overview")
            .to_string();
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("index")
            .to_string();

        let raw_file = fs::read_to_string(&path).expect("read markdown file");
        let (title, description, body) = parse_frontmatter(&raw_file);
        let (html, toc, keywords) = compile_markdown(body, &title, &description);

        let clean_md = format!("# {title}\n\n> {description}\n\n{}", body.trim());
        docs.push(CompiledDoc {
            section,
            slug,
            title,
            description,
            html,
            raw_md: clean_md,
            toc,
            keywords,
        });
    }

    write_docs_gen(&out_dir.join("docs_gen.rs"), &manifest_dir, &docs);
    write_public_outputs(&manifest_dir, &public_dir, &docs);
}

fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

fn parse_frontmatter(input: &str) -> (String, String, &str) {
    let trimmed = input.trim_start_matches('\u{feff}');
    if let Some(rest) = trimmed.strip_prefix("---\n") {
        if let Some(end_idx) = rest.find("\n---\n") {
            let fm = &rest[..end_idx];
            let body = &rest[end_idx + 5..];
            let mut title = String::new();
            let mut description = String::new();
            for line in fm.lines() {
                if let Some(val) = line.strip_prefix("title:") {
                    title = val.trim().trim_matches('"').to_string();
                } else if let Some(val) = line.strip_prefix("description:") {
                    description = val.trim().trim_matches('"').to_string();
                }
            }
            return (title, description, body);
        }
    }
    (String::new(), String::new(), input)
}

fn slugify(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev_dash = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

fn compile_markdown(body: &str, title: &str, description: &str) -> (String, Vec<TocEntry>, String) {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_HEADING_ATTRIBUTES;
    let parser = Parser::new_ext(body, options);

    let mut events_out: Vec<Event<'_>> = Vec::new();
    let mut toc: Vec<TocEntry> = Vec::new();
    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    let mut heading_keywords: Vec<String> = Vec::new();

    let mut in_heading: Option<(HeadingLevel, Option<CowStr<'_>>, Vec<CowStr<'_>>, Vec<(CowStr<'_>, Option<CowStr<'_>>)>)> = None;
    let mut heading_events: Vec<Event<'_>> = Vec::new();
    let mut heading_text = String::new();

    let mut in_code_block: Option<String> = None;
    let mut code_buffer = String::new();

    for event in parser {
        if let Some(ref lang) = in_code_block {
            match event {
                Event::Text(t) => code_buffer.push_str(&t),
                Event::End(TagEnd::CodeBlock) => {
                    let highlighted = highlight_code(lang, &code_buffer);
                    let lang_attr = html_escape(lang);
                    let html = format!(
                        r#"<pre class="code-block" data-lang="{lang_attr}"><code>{highlighted}</code></pre>"#
                    );
                    events_out.push(Event::Html(html.into()));
                    in_code_block = None;
                    code_buffer.clear();
                }
                _ => {}
            }
            continue;
        }

        if let Some((level, orig_id, classes, attrs)) = in_heading.take() {
            match event {
                Event::End(TagEnd::Heading(_)) => {
                    let level_num = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    let mut final_id = orig_id.map(|s| s.into_string());
                    if matches!(level, HeadingLevel::H2 | HeadingLevel::H3) {
                        let base_slug = final_id.take().unwrap_or_else(|| slugify(&heading_text));
                        let count = slug_counts.entry(base_slug.clone()).or_insert(0);
                        *count += 1;
                        let unique_slug = if *count == 1 {
                            base_slug
                        } else {
                            format!("{base_slug}-{count}")
                        };
                        heading_keywords.push(heading_text.trim().to_string());
                        toc.push(TocEntry {
                            id: unique_slug.clone(),
                            title: heading_text.trim().to_string(),
                            level: level_num,
                        });
                        final_id = Some(unique_slug);
                    }
                    events_out.push(Event::Start(Tag::Heading {
                        level,
                        id: final_id.map(CowStr::from),
                        classes,
                        attrs,
                    }));
                    events_out.extend(heading_events.drain(..));
                    events_out.push(Event::End(TagEnd::Heading(level)));
                    heading_text.clear();
                }
                other => {
                    match &other {
                        Event::Text(t) | Event::Code(t) => heading_text.push_str(t),
                        Event::SoftBreak | Event::HardBreak => heading_text.push(' '),
                        _ => {}
                    }
                    heading_events.push(other);
                    in_heading = Some((level, orig_id, classes, attrs));
                }
            }
            continue;
        }

        match event {
            Event::Start(Tag::Heading {
                level,
                id,
                classes,
                attrs,
            }) => {
                in_heading = Some((level, id, classes, attrs));
                heading_events.clear();
                heading_text.clear();
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = Some(lang.trim().to_string());
                code_buffer.clear();
            }
            other => events_out.push(other),
        }
    }

    let mut html_out = String::new();
    pulldown_cmark::html::push_html(&mut html_out, events_out.into_iter());

    let mut keywords = format!("{title} {description}");
    if !heading_keywords.is_empty() {
        keywords.push(' ');
        keywords.push_str(&heading_keywords.join(" "));
    }

    (html_out, toc, keywords)
}

fn write_docs_gen(out_path: &Path, manifest_dir: &Path, docs: &[CompiledDoc]) {
    let component_docs = api_schema::all_component_docs();
    let demos_dir = manifest_dir.join("src").join("demos");

    let mut out = String::new();
    out.push_str("pub static ALL_MARKDOWN_DOCS: &[MarkdownDoc] = &[\n");
    for doc in docs {
        out.push_str("    MarkdownDoc {\n");
        out.push_str(&format!("        section: {:?},\n", doc.section));
        out.push_str(&format!("        slug: {:?},\n", doc.slug));
        out.push_str(&format!("        title: {:?},\n", doc.title));
        out.push_str(&format!("        description: {:?},\n", doc.description));
        out.push_str(&format!("        html: {:?},\n", doc.html));
        out.push_str(&format!("        raw_md: {:?},\n", doc.raw_md));
        out.push_str("        toc: &[\n");
        for item in &doc.toc {
            out.push_str(&format!(
                "            TocItem {{ id: {:?}, title: {:?}, level: {} }},\n",
                item.id, item.title, item.level
            ));
        }
        out.push_str("        ],\n");
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");

    out.push_str("pub static ALL_SEARCH_ENTRIES: &[SearchEntry] = &[\n");
    for doc in docs {
        let path = format!("/docs/{}/{}", doc.section, doc.slug);
        let section_label = match doc.section.as_str() {
            "overview" => "Overview",
            "guides" => "Guides",
            other => other,
        };
        out.push_str("    SearchEntry {\n");
        out.push_str(&format!("        title: {:?},\n", doc.title));
        out.push_str(&format!("        description: {:?},\n", doc.description));
        out.push_str(&format!("        section: {:?},\n", section_label));
        out.push_str(&format!("        path: {:?},\n", path));
        out.push_str(&format!("        keywords: {:?},\n", doc.keywords));
        out.push_str("    },\n");
    }
    for comp in component_docs {
        let path = format!("/docs/components/{}", comp.slug);
        out.push_str("    SearchEntry {\n");
        out.push_str(&format!("        title: {:?},\n", comp.title));
        out.push_str(&format!("        description: {:?},\n", comp.description));
        out.push_str("        section: \"Components\",\n");
        out.push_str(&format!("        path: {:?},\n", path));
        out.push_str(&format!("        keywords: {:?},\n", comp.keywords));
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");

    out.push_str("pub static SIDEBAR_COMPONENT_ITEMS: &[NavItem] = &[\n");
    for comp in component_docs {
        let path = format!("/docs/components/{}", comp.slug);
        out.push_str("    NavItem {\n");
        out.push_str(&format!("        title: {:?},\n", comp.title));
        out.push_str(&format!("        path: {:?},\n", path));
        out.push_str("        badge: None,\n");
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");

    out.push_str(&format!(
        "pub const CSS_TOKENS_SNIPPET: &str = {:?};\n",
        CSS_TOKENS_SNIPPET
    ));
    out.push_str(&format!(
        "pub const HIGHLIGHTED_CSS_TOKENS: &str = {:?};\n\n",
        highlight_css(CSS_TOKENS_SNIPPET)
    ));

    out.push_str("pub fn highlighted_anatomy(slug: &str) -> &'static str {\n");
    out.push_str("    match slug {\n");
    for comp in component_docs {
        let highlighted = highlight_rust(comp.anatomy);
        out.push_str(&format!("        {:?} => {:?},\n", comp.slug, highlighted));
    }
    out.push_str("        _ => \"\",\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out.push_str("pub fn highlighted_demo(slug: &str) -> &'static str {\n");
    out.push_str("    match slug {\n");
    for comp in component_docs {
        let demo_path = demos_dir.join(format!("{}_demo.rs", comp.slug));
        let demo_src = fs::read_to_string(&demo_path).unwrap_or_default();
        let highlighted = highlight_rust(&demo_src);
        out.push_str(&format!("        {:?} => {:?},\n", comp.slug, highlighted));
    }
    out.push_str("        _ => \"\",\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    fs::write(out_path, out).expect("write docs_gen.rs");
}

fn write_public_outputs(manifest_dir: &Path, public_dir: &Path, docs: &[CompiledDoc]) {
    let _ = fs::create_dir_all(public_dir);

    for doc in docs {
        let target_dir = public_dir.join("docs").join(&doc.section);
        let _ = fs::create_dir_all(&target_dir);
        let target_file = target_dir.join(format!("{}.md", doc.slug));
        fs::write(target_file, &doc.raw_md).expect("write static markdown");
    }

    let components_dir = public_dir.join("docs").join("components");
    let _ = fs::create_dir_all(&components_dir);
    let demos_dir = manifest_dir.join("src").join("demos");
    for comp in api_schema::all_component_docs() {
        let demo_path = demos_dir.join(format!("{}_demo.rs", comp.slug));
        let demo_src = fs::read_to_string(&demo_path).unwrap_or_default();
        let comp_md = api_schema::component_to_markdown(comp, &demo_src);
        let target_file = components_dir.join(format!("{}.md", comp.slug));
        fs::write(target_file, comp_md).expect("write component static markdown");
    }

    let mut llms = String::new();
    llms.push_str("# Monoxus\n\n");
    llms.push_str("> Headless, accessible, composable UI primitives for Dioxus on Web and Blitz Native.\n\n");

    llms.push_str("## Overview\n\n");
    for doc in docs.iter().filter(|d| d.section == "overview") {
        llms.push_str(&format!(
            "- [{}](/docs/overview/{}.md): {}\n",
            doc.title, doc.slug, doc.description
        ));
    }

    llms.push_str("\n## Guides\n\n");
    for doc in docs.iter().filter(|d| d.section == "guides") {
        llms.push_str(&format!(
            "- [{}](/docs/guides/{}.md): {}\n",
            doc.title, doc.slug, doc.description
        ));
    }

    llms.push_str("\n## Components\n\n");
    for comp in api_schema::all_component_docs() {
        llms.push_str(&format!(
            "- [{}](/docs/components/{}.md): {}\n",
            comp.title, comp.slug, comp.description
        ));
    }

    fs::write(public_dir.join("llms.txt"), llms).expect("write public/llms.txt");

    let mut json = String::from("[\n");
    let mut first = true;
    for doc in docs {
        if !first {
            json.push_str(",\n");
        }
        first = false;
        let path = format!("/docs/{}/{}", doc.section, doc.slug);
        let section_label = if doc.section == "overview" {
            "Overview"
        } else if doc.section == "guides" {
            "Guides"
        } else {
            &doc.section
        };
        json.push_str(&format!(
            "  {{\"title\": {}, \"description\": {}, \"section\": {}, \"path\": {}, \"keywords\": {}}}",
            json_escape(&doc.title),
            json_escape(&doc.description),
            json_escape(section_label),
            json_escape(&path),
            json_escape(&doc.keywords),
        ));
    }
    for comp in api_schema::all_component_docs() {
        if !first {
            json.push_str(",\n");
        }
        first = false;
        let path = format!("/docs/components/{}", comp.slug);
        json.push_str(&format!(
            "  {{\"title\": {}, \"description\": {}, \"section\": \"Components\", \"path\": {}, \"keywords\": {}}}",
            json_escape(comp.title),
            json_escape(comp.description),
            json_escape(&path),
            json_escape(comp.keywords),
        ));
    }
    json.push_str("\n]\n");

    fs::write(public_dir.join("search-index.json"), json).expect("write public/search-index.json");
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
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

fn push_span(out: &mut String, cls: &str, text: &str) {
    out.push_str("<span class=\"");
    out.push_str(cls);
    out.push_str("\">");
    out.push_str(&html_escape(text));
    out.push_str("</span>");
}

fn highlight_code(lang: &str, code: &str) -> String {
    match lang.to_ascii_lowercase().as_str() {
        "rust" | "rs" => highlight_rust(code),
        "toml" => highlight_toml(code),
        "css" => highlight_css(code),
        "bash" | "sh" | "zsh" | "shell" => highlight_bash(code),
        _ => html_escape(code),
    }
}

fn highlight_rust(code: &str) -> String {
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
        // Line comment
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            let start = i;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            push_span(&mut out, "hl-cm", &code[start..i]);
            continue;
        }
        // Block comment
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
        // Attribute #[...] or #![...]
        if bytes[i] == b'#' && (i + 1 < len && (bytes[i + 1] == b'[' || (bytes[i + 1] == b'!' && i + 2 < len && bytes[i + 2] == b'['))) {
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
        // String literal
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
        // Number
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') {
                i += 1;
            }
            push_span(&mut out, "hl-num", &code[start..i]);
            continue;
        }
        // Identifier / keyword / macro / type / fn
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

        // Single UTF-8 char
        let ch = code[i..].chars().next().unwrap();
        out.push_str(&html_escape(&code[i..i + ch.len_utf8()]));
        i += ch.len_utf8();
    }

    out
}

fn highlight_toml(code: &str) -> String {
    let mut out = String::with_capacity(code.len() * 2);
    for (idx, line) in code.split('\n').enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            push_span(&mut out, "hl-cm", line);
        } else if trimmed.starts_with('[') {
            push_span(&mut out, "hl-ty", line);
        } else if let Some(eq_pos) = line.find('=') {
            let key = &line[..eq_pos];
            let rest = &line[eq_pos..];
            push_span(&mut out, "hl-attr", key);
            out.push('=');
            highlight_toml_value(&rest[1..], &mut out);
        } else {
            out.push_str(&html_escape(line));
        }
    }
    out
}

fn highlight_toml_value(val: &str, out: &mut String) {
    let bytes = val.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            push_span(out, "hl-str", &val[start..i]);
        } else if bytes[i] == b'#' {
            push_span(out, "hl-cm", &val[i..]);
            break;
        } else if val[i..].starts_with("true") || val[i..].starts_with("false") {
            let kw = if val[i..].starts_with("true") { "true" } else { "false" };
            push_span(out, "hl-kw", kw);
            i += kw.len();
        } else if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            push_span(out, "hl-num", &val[start..i]);
        } else {
            let ch = val[i..].chars().next().unwrap();
            out.push_str(&html_escape(&val[i..i + ch.len_utf8()]));
            i += ch.len_utf8();
        }
    }
}

fn highlight_css(code: &str) -> String {
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len * 2);
    let mut i = 0;
    while i < len {
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
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            let start = i;
            i += 1;
            while i < len {
                if bytes[i] == q {
                    i += 1;
                    break;
                }
                i += 1;
            }
            push_span(&mut out, "hl-str", &code[start..i]);
            continue;
        }
        if bytes[i] == b'[' {
            let start = i;
            while i < len && bytes[i] != b']' {
                i += 1;
            }
            if i < len {
                i += 1;
            }
            push_span(&mut out, "hl-attr", &code[start..i]);
            continue;
        }
        if bytes[i] == b'-' && i + 1 < len && bytes[i + 1] == b'-' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
                i += 1;
            }
            push_span(&mut out, "hl-ty", &code[start..i]);
            continue;
        }
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.' || bytes[i] == b'%') {
                i += 1;
            }
            push_span(&mut out, "hl-num", &code[start..i]);
            continue;
        }
        let ch = code[i..].chars().next().unwrap();
        out.push_str(&html_escape(&code[i..i + ch.len_utf8()]));
        i += ch.len_utf8();
    }
    out
}

fn highlight_bash(code: &str) -> String {
    let mut out = String::with_capacity(code.len() * 2);
    for (idx, line) in code.split('\n').enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        if line.trim_start().starts_with('#') {
            push_span(&mut out, "hl-cm", line);
        } else {
            let bytes = line.as_bytes();
            let mut i = 0;
            let mut first_word = true;
            while i < bytes.len() {
                if bytes[i] == b'"' || bytes[i] == b'\'' {
                    let q = bytes[i];
                    let start = i;
                    i += 1;
                    while i < bytes.len() && bytes[i] != q {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                    push_span(&mut out, "hl-str", &line[start..i]);
                    first_word = false;
                } else if bytes[i] == b'-' && (i == 0 || bytes[i - 1].is_ascii_whitespace()) {
                    let start = i;
                    while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    push_span(&mut out, "hl-attr", &line[start..i]);
                    first_word = false;
                } else if first_word && !bytes[i].is_ascii_whitespace() {
                    let start = i;
                    while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    push_span(&mut out, "hl-fn", &line[start..i]);
                    first_word = false;
                } else {
                    let ch = line[i..].chars().next().unwrap();
                    out.push_str(&html_escape(&line[i..i + ch.len_utf8()]));
                    i += ch.len_utf8();
                }
            }
        }
    }
    out
}

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use monoxus_site::api_schema::{all_component_docs, component_to_markdown};
use monoxus_site::docs_data::{
    all_markdown_docs, find_markdown_doc, highlight_rust_html, prev_next_nav, search_docs,
    sidebar_groups,
};
use syn::{Fields, FnArg, Item, Pat, Type};

fn monoxus_src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../src")
        .canonicalize()
        .expect("locate ../src")
}

fn collect_rs_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path.to_path_buf());
        }
        return;
    }
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            collect_rs_files(&entry.path(), out);
        }
    }
}

fn strip_raw_prefix(name: &str) -> String {
    name.strip_prefix("r#").unwrap_or(name).to_string()
}

fn extract_type_ident(ty: &Type) -> Option<String> {
    if let Type::Path(tp) = ty {
        tp.path.segments.last().map(|seg| seg.ident.to_string())
    } else {
        None
    }
}

fn collect_component_props(files: &[PathBuf]) -> HashMap<String, HashSet<String>> {
    let mut structs: HashMap<String, HashSet<String>> = HashMap::new();
    let mut fns: Vec<(String, Vec<(String, Option<String>)>)> = Vec::new();

    for file_path in files {
        let src = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", file_path.display()));
        let parsed = syn::parse_file(&src)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", file_path.display()));

        for item in parsed.items {
            match item {
                Item::Struct(s) => {
                    let mut fields = HashSet::new();
                    if let Fields::Named(named) = s.fields {
                        for field in named.named {
                            if let Some(ident) = field.ident {
                                fields.insert(strip_raw_prefix(&ident.to_string()));
                            }
                        }
                    }
                    structs.insert(s.ident.to_string(), fields);
                }
                Item::Fn(f) => {
                    if !matches!(f.vis, syn::Visibility::Public(_)) {
                        continue;
                    }
                    let mut params = Vec::new();
                    for input in f.sig.inputs {
                        if let FnArg::Typed(pat_type) = input {
                            if let Pat::Ident(pat_ident) = *pat_type.pat {
                                let name = strip_raw_prefix(&pat_ident.ident.to_string());
                                let ty_name = extract_type_ident(&pat_type.ty);
                                params.push((name, ty_name));
                            }
                        }
                    }
                    fns.push((f.sig.ident.to_string(), params));
                }
                _ => {}
            }
        }
    }

    let mut result = HashMap::new();
    for (fn_name, params) in fns {
        let props = if params.len() == 1 && params[0].0 == "props" {
            if let Some(ref struct_name) = params[0].1 {
                structs.get(struct_name).cloned().unwrap_or_default()
            } else {
                HashSet::new()
            }
        } else {
            params.into_iter().map(|(name, _)| name).collect()
        };
        result.insert(fn_name, props);
    }
    result
}

#[test]
fn api_schema_matches_monoxus_source() {
    let base = monoxus_src_dir();
    let targets = [
        base.join("button.rs"),
        base.join("dialog/components.rs"),
        base.join("tabs/components.rs"),
        base.join("select"),
        base.join("tooltip/components"),
    ];

    let mut rs_files = Vec::new();
    for target in &targets {
        assert!(
            target.exists(),
            "expected monoxus source path to exist: {}",
            target.display()
        );
        collect_rs_files(target, &mut rs_files);
    }

    let parsed_components = collect_component_props(&rs_files);
    let docs = all_component_docs();
    assert_eq!(docs.len(), 5, "expected 5 MVP component docs");

    for comp in docs {
        for part in comp.parts {
            let actual_props = parsed_components.get(part.name).unwrap_or_else(|| {
                panic!(
                    "documented part `{}` in `{}` was not found as a `pub fn` in monoxus",
                    part.name, comp.slug
                )
            });

            for prop in part.props {
                let normalized = strip_raw_prefix(prop.name);
                assert!(
                    actual_props.contains(&normalized),
                    "documented prop `{}` on `{}` (`{}`) does not exist in monoxus source; found: {:?}",
                    prop.name,
                    part.name,
                    comp.slug,
                    actual_props
                );
            }
        }

        let md = component_to_markdown(comp, "");
        assert!(md.contains(comp.title));
    }
}

#[test]
fn public_llms_and_search_index_exist_and_non_empty() {
    let public_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("public");

    let llms = fs::read_to_string(public_dir.join("llms.txt"))
        .expect("public/llms.txt should be generated by build.rs");
    assert!(!llms.trim().is_empty(), "public/llms.txt must not be empty");
    assert!(llms.contains("Button"));
    assert!(llms.contains("Dialog"));
    assert!(llms.contains("Select"));
    assert!(llms.contains("Tabs"));
    assert!(llms.contains("Tooltip"));

    let search_json = fs::read_to_string(public_dir.join("search-index.json"))
        .expect("public/search-index.json should be generated by build.rs");
    assert!(
        !search_json.trim().is_empty(),
        "public/search-index.json must not be empty"
    );
    assert!(search_json.starts_with('['));

    for doc in all_markdown_docs() {
        let static_md = public_dir
            .join("docs")
            .join(doc.section)
            .join(format!("{}.md", doc.slug));
        assert!(
            static_md.exists(),
            "expected static markdown file at {}",
            static_md.display()
        );
    }

    for comp in all_component_docs() {
        let static_md = public_dir
            .join("docs")
            .join("components")
            .join(format!("{}.md", comp.slug));
        assert!(
            static_md.exists(),
            "expected component static markdown file at {}",
            static_md.display()
        );
    }
}

#[test]
fn docs_data_navigation_search_and_highlighting_work() {
    assert_eq!(all_markdown_docs().len(), 6);
    assert!(find_markdown_doc("overview", "introduction").is_some());
    assert!(find_markdown_doc("overview", "roadmap").is_some());
    assert!(find_markdown_doc("guides", "styling").is_some());
    assert_eq!(sidebar_groups().len(), 3);

    let (prev, next) = prev_next_nav("/docs/overview/getting-started");
    assert_eq!(prev.map(|i| i.path), Some("/docs/overview/introduction"));
    assert_eq!(next.map(|i| i.path), Some("/docs/overview/accessibility"));

    let results = search_docs("dialog");
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Dialog");

    let highlighted = highlight_rust_html("pub fn Demo() -> Element { let x = 1; }");
    assert!(highlighted.contains("hl-kw"));
    assert!(highlighted.contains("hl-ty"));
    assert!(highlighted.contains("hl-num"));
}

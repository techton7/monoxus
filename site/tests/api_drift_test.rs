use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use monoxus_site::api_schema::{all_component_docs, component_to_markdown};
use monoxus_site::docs_data::{
    HIGHLIGHTED_CSS_TOKENS, all_markdown_docs, find_markdown_doc, highlighted_anatomy,
    highlighted_demo, prev_next_nav, search_docs, sidebar_groups,
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

fn has_component_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        path.is_ident("component")
            || (path.segments.len() == 2
                && path.segments[0].ident == "dioxus"
                && path.segments[1].ident == "component")
    })
}

fn is_excluded_component_fn(slug: &str, fn_name: &str) -> bool {
    slug == "dialog" && matches!(fn_name, "AlertDialogAction" | "AlertDialogCancel")
}

fn extract_type_ident(ty: &Type) -> Option<String> {
    if let Type::Path(tp) = ty {
        tp.path.segments.last().map(|seg| seg.ident.to_string())
    } else {
        None
    }
}

struct ParsedComponentSource {
    pub_fns: HashMap<String, HashSet<String>>,
    component_fns: HashSet<String>,
}

fn collect_component_props(files: &[PathBuf]) -> ParsedComponentSource {
    let mut structs: HashMap<String, HashSet<String>> = HashMap::new();
    let mut fns: Vec<(String, Vec<(String, Option<String>)>)> = Vec::new();
    let mut component_fns: HashSet<String> = HashSet::new();

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
                                fields.insert(ident.to_string());
                            }
                        }
                    }
                    structs.insert(s.ident.to_string(), fields);
                }
                Item::Fn(f) => {
                    if !matches!(f.vis, syn::Visibility::Public(_)) {
                        continue;
                    }
                    let fn_name = f.sig.ident.to_string();
                    if has_component_attr(&f.attrs) {
                        component_fns.insert(fn_name.clone());
                    }
                    let mut params = Vec::new();
                    for input in f.sig.inputs {
                        if let FnArg::Typed(pat_type) = input {
                            if let Pat::Ident(pat_ident) = *pat_type.pat {
                                let name = pat_ident.ident.to_string();
                                let ty_name = extract_type_ident(&pat_type.ty);
                                params.push((name, ty_name));
                            }
                        }
                    }
                    fns.push((fn_name, params));
                }
                _ => {}
            }
        }
    }

    let mut pub_fns = HashMap::new();
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
        pub_fns.insert(fn_name, props);
    }
    ParsedComponentSource {
        pub_fns,
        component_fns,
    }
}

#[test]
fn api_schema_matches_monoxus_source() {
    let base = monoxus_src_dir();
    let docs = all_component_docs();
    assert!(
        !docs.is_empty(),
        "expected at least one documented component in api_schema"
    );

    let mut all_undocumented_parts = Vec::new();
    let mut all_undocumented_props = Vec::new();

    for comp in docs {
        let rel = comp
            .source_path
            .strip_prefix("src/")
            .unwrap_or(comp.source_path);
        let target = if rel.ends_with("/mod.rs") {
            base.join(Path::new(rel).parent().expect("mod.rs parent dir"))
        } else {
            base.join(rel)
        };
        assert!(
            target.exists(),
            "expected monoxus source path for `{}` to exist: {}",
            comp.slug,
            target.display()
        );

        let mut rs_files = Vec::new();
        collect_rs_files(&target, &mut rs_files);
        rs_files.sort();
        rs_files.dedup();

        let parsed = collect_component_props(&rs_files);
        let mut documented_parts = HashSet::new();

        for part in comp.parts {
            documented_parts.insert(part.name);

            let actual_props = parsed.pub_fns.get(part.name).unwrap_or_else(|| {
                panic!(
                    "documented part `{}` in `{}` was not found as a `pub fn` in {}",
                    part.name,
                    comp.slug,
                    target.display()
                )
            });

            let mut documented_props = HashSet::new();
            for prop in part.props {
                assert!(
                    actual_props.contains(prop.name),
                    "documented prop `{}` on `{}` (`{}`) does not exist in monoxus source; found: {:?}",
                    prop.name,
                    part.name,
                    comp.slug,
                    actual_props
                );
                documented_props.insert(prop.name);
            }

            let mut undocumented: Vec<String> = actual_props
                .iter()
                .filter(|prop| !documented_props.contains(prop.as_str()))
                .cloned()
                .collect();
            undocumented.sort();
            if !undocumented.is_empty() {
                all_undocumented_props
                    .push(format!("`{}` (`{}`): {:?}", part.name, comp.slug, undocumented));
            }
        }

        let mut undocumented_parts: Vec<String> = parsed
            .component_fns
            .iter()
            .filter(|name| !is_excluded_component_fn(comp.slug, name))
            .filter(|name| !documented_parts.contains(name.as_str()))
            .cloned()
            .collect();
        undocumented_parts.sort();
        if !undocumented_parts.is_empty() {
            all_undocumented_parts.push(format!("`{}`: {:?}", comp.slug, undocumented_parts));
        }

        let md = component_to_markdown(comp, "");
        assert!(md.contains(comp.title));
    }

    assert!(
        all_undocumented_parts.is_empty(),
        "undocumented public #[component] functions found:\n{}",
        all_undocumented_parts.join("\n")
    );
    assert!(
        all_undocumented_props.is_empty(),
        "undocumented public props found:\n{}",
        all_undocumented_props.join("\n")
    );
}

#[test]
fn public_llms_and_search_index_exist_and_non_empty() {
    let public_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("public");

    let llms = fs::read_to_string(public_dir.join("llms.txt"))
        .expect("public/llms.txt should be generated by build.rs");
    assert!(!llms.trim().is_empty(), "public/llms.txt must not be empty");
    for comp in all_component_docs() {
        assert!(
            llms.contains(comp.title),
            "public/llms.txt missing component {}",
            comp.title
        );
    }

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
    assert_eq!(sidebar_groups()[2].items.len(), all_component_docs().len());

    let (prev, next) = prev_next_nav("/docs/overview/getting-started");
    assert_eq!(prev.map(|i| i.path), Some("/docs/overview/introduction"));
    assert_eq!(next.map(|i| i.path), Some("/docs/overview/accessibility"));

    let results = search_docs("dialog");
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Dialog");

    for comp in all_component_docs() {
        let anatomy_html = highlighted_anatomy(comp.slug);
        assert!(
            anatomy_html.contains("hl-kw") && anatomy_html.contains("hl-ty"),
            "pre-highlighted anatomy for `{}` missing highlight spans",
            comp.slug
        );
        let demo_html = highlighted_demo(comp.slug);
        assert!(
            demo_html.contains("hl-kw") && demo_html.contains("hl-ty"),
            "pre-highlighted demo for `{}` missing highlight spans",
            comp.slug
        );
    }

    assert!(
        HIGHLIGHTED_CSS_TOKENS.contains("hl-attr") && HIGHLIGHTED_CSS_TOKENS.contains("hl-ty"),
        "pre-highlighted CSS tokens snippet should use CSS highlight rules"
    );
}

// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// An item in the Karpal API index.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ApiItem {
    pub name: String,
    pub kind: &'static str,
    pub crate_name: String,
    pub path: String,
    pub signature: Option<String>,
    pub docs: Option<String>,
    pub summary: Option<String>,
    pub supertraits: Vec<String>,
    pub subtraits: Vec<String>,
    pub methods: Vec<String>,
    pub implementors: Vec<String>,
    pub trait_impls: Vec<String>,
}

/// An in-memory index of all public API items in the Karpal workspace.
pub struct WorkspaceIndex {
    pub items: Vec<ApiItem>,
    pub crate_descriptions: BTreeMap<String, String>,
}

impl WorkspaceIndex {
    /// Build the index by walking the workspace source tree.
    pub fn build(workspace_root: &Path) -> Self {
        let mut items = Vec::new();
        let mut crate_descriptions = BTreeMap::new();

        // Find all crate directories (those containing Cargo.toml and a src/ dir)
        for entry in WalkDir::new(workspace_root)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "Cargo.toml" {
                let crate_dir = entry.path().parent().unwrap();
                let src_dir = crate_dir.join("src");
                if src_dir.is_dir() {
                    let crate_name = crate_dir.file_name().unwrap().to_string_lossy().to_string();

                    // Skip non-library crates
                    let cargo_toml = fs::read_to_string(entry.path()).unwrap_or_default();
                    if !cargo_toml.contains("[lib]") && cargo_toml.contains("[[bin]]") {
                        continue;
                    }

                    // Extract crate description from Cargo.toml
                    if let Some(desc) = extract_description(&cargo_toml) {
                        crate_descriptions.insert(crate_name.clone(), desc);
                    }

                    // Parse source files in src/
                    for src_entry in WalkDir::new(&src_dir)
                        .max_depth(2)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if src_entry.path().extension().is_some_and(|e| e == "rs") {
                            let source = fs::read_to_string(src_entry.path()).unwrap_or_default();
                            let rel_path = src_entry
                                .path()
                                .strip_prefix(workspace_root)
                                .unwrap_or(src_entry.path())
                                .to_string_lossy()
                                .to_string();

                            parse_source(&source, &crate_name, &rel_path, &mut items);
                        }
                    }
                }
            }
        }

        // Build cross-references
        build_cross_references(&mut items);

        WorkspaceIndex {
            items,
            crate_descriptions,
        }
    }

    /// Search for items matching a query (name substring, case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&ApiItem> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&ApiItem> = self
            .items
            .iter()
            .filter(|item| item.name.to_lowercase().contains(&query_lower))
            .collect();
        results.sort_by_key(|item| &item.name);
        results
    }

    /// Find an item by exact name.
    pub fn find(&self, name: &str) -> Option<&ApiItem> {
        self.items.iter().find(|item| item.name == name)
    }
}

/// Extract the description field from a Cargo.toml string.
fn extract_description(cargo_toml: &str) -> Option<String> {
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("description = ") {
            let desc = trimmed
                .strip_prefix("description = \"")
                .or_else(|| trimmed.strip_prefix("description = \""))
                .and_then(|s| s.strip_suffix('"'));
            if let Some(d) = desc {
                return Some(d.to_string());
            }
        }
    }
    None
}

/// Parse a Rust source file and extract public API items.
fn parse_source(source: &str, crate_name: &str, path: &str, items: &mut Vec<ApiItem>) {
    // Simple heuristic-based extraction — we look for `pub` items
    // without needing full syn parsing for the MVP.
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // pub trait Name
        if line.starts_with("pub trait ")
            && let Some(name) = extract_name(line, "pub trait ")
        {
            let doc = extract_doc_comment(&lines, i);
            let summary = doc.as_ref().and_then(|d| first_sentence(d));
            let supertraits = extract_supertraits(line);
            let methods = extract_trait_methods(&lines, &mut i, &name);
            items.push(ApiItem {
                name: name.clone(),
                kind: "trait",
                crate_name: crate_name.to_string(),
                path: format!("{}:{}", path, find_line_number(&lines, i)),
                signature: Some(line.to_string()),
                docs: doc,
                summary,
                supertraits,
                subtraits: Vec::new(),
                methods,
                implementors: Vec::new(),
                trait_impls: Vec::new(),
            });
        }

        // pub struct Name
        if line.starts_with("pub struct ")
            && let Some(name) = extract_name(line, "pub struct ")
        {
            let doc = extract_doc_comment(&lines, i);
            let summary = doc.as_ref().and_then(|d| first_sentence(d));
            items.push(ApiItem {
                name: name.clone(),
                kind: "struct",
                crate_name: crate_name.to_string(),
                path: format!("{}:{}", path, find_line_number(&lines, i)),
                signature: Some(line.to_string()),
                docs: doc,
                summary,
                supertraits: Vec::new(),
                subtraits: Vec::new(),
                methods: Vec::new(),
                implementors: Vec::new(),
                trait_impls: Vec::new(),
            });
        }

        // pub enum Name
        if line.starts_with("pub enum ")
            && let Some(name) = extract_name(line, "pub enum ")
        {
            let doc = extract_doc_comment(&lines, i);
            let summary = doc.as_ref().and_then(|d| first_sentence(d));
            items.push(ApiItem {
                name: name.clone(),
                kind: "enum",
                crate_name: crate_name.to_string(),
                path: format!("{}:{}", path, find_line_number(&lines, i)),
                signature: Some(line.to_string()),
                docs: doc,
                summary,
                supertraits: Vec::new(),
                subtraits: Vec::new(),
                methods: Vec::new(),
                implementors: Vec::new(),
                trait_impls: Vec::new(),
            });
        }

        // pub type Name = ...;
        if line.starts_with("pub type ")
            && let Some(name) = extract_name(line, "pub type ")
        {
            let doc = extract_doc_comment(&lines, i);
            let summary = doc.as_ref().and_then(|d| first_sentence(d));
            items.push(ApiItem {
                name: name.clone(),
                kind: "type alias",
                crate_name: crate_name.to_string(),
                path: format!("{}:{}", path, find_line_number(&lines, i)),
                signature: Some(line.to_string()),
                docs: doc,
                summary,
                supertraits: Vec::new(),
                subtraits: Vec::new(),
                methods: Vec::new(),
                implementors: Vec::new(),
                trait_impls: Vec::new(),
            });
        }

        // pub fn name(...)
        if line.starts_with("pub fn ")
            && let Some(name) = extract_fn_name(line)
        {
            let doc = extract_doc_comment(&lines, i);
            let summary = doc.as_ref().and_then(|d| first_sentence(d));
            items.push(ApiItem {
                name,
                kind: "function",
                crate_name: crate_name.to_string(),
                path: format!("{}:{}", path, find_line_number(&lines, i)),
                signature: Some(line.to_string()),
                docs: doc,
                summary,
                supertraits: Vec::new(),
                subtraits: Vec::new(),
                methods: Vec::new(),
                implementors: Vec::new(),
                trait_impls: Vec::new(),
            });
        }

        // impl Trait for Type — track trait impls
        if line.starts_with("impl ")
            && line.contains(" for ")
            && !line.contains('<')
            && let Some((trait_name, type_name)) = extract_impl(line)
        {
            // Find the type and add this trait impl
            if let Some(type_item) = items.iter_mut().find(|item| item.name == type_name) {
                type_item.trait_impls.push(trait_name.clone());
            }
            // Find the trait and add this implementor
            if let Some(trait_item) = items.iter_mut().find(|item| item.name == trait_name)
                && !trait_item.implementors.contains(&type_name)
            {
                trait_item.implementors.push(type_name.clone());
            }
        }

        i += 1;
    }
}

fn extract_name(line: &str, prefix: &str) -> Option<String> {
    let rest = line.strip_prefix(prefix)?;
    let name = rest
        .split(|c: char| c.is_whitespace() || c == '<' || c == '{' || c == ':' || c == ';')
        .next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn extract_fn_name(line: &str) -> Option<String> {
    let rest = line.strip_prefix("pub fn ")?;
    let name = rest.split('(').next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.trim().to_string())
    }
}

fn extract_supertraits(line: &str) -> Vec<String> {
    if let Some(after_colon) = line.split(':').nth(1) {
        after_colon
            .split('+')
            .map(|s| s.trim().trim_end_matches('{').trim().to_string())
            .filter(|s| !s.is_empty() && !s.starts_with('{'))
            .collect()
    } else {
        Vec::new()
    }
}

fn extract_impl(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("impl ")?;
    // Handle "impl TraitName for TypeName"
    let parts: Vec<&str> = rest.split(" for ").collect();
    if parts.len() == 2 {
        let trait_name = parts[0].trim().to_string();
        let type_name = parts[1].trim().trim_end_matches('{').trim().to_string();
        Some((trait_name, type_name))
    } else {
        None
    }
}

fn extract_trait_methods(lines: &[&str], i: &mut usize, _trait_name: &str) -> Vec<String> {
    let mut methods = Vec::new();
    // Skip ahead to find the trait body
    while *i < lines.len() && !lines[*i].contains('{') {
        *i += 1;
    }
    *i += 1; // skip the opening brace
    let mut depth = 1;
    while *i < lines.len() && depth > 0 {
        let line = lines[*i].trim();
        if line.contains('{') {
            depth += line.matches('{').count();
        }
        if line.contains('}') {
            depth -= line.matches('}').count();
        }
        if depth > 0 && (line.starts_with("fn ") || line.starts_with("type ")) {
            let sig = line.trim_end_matches('{').trim().to_string();
            if !sig.is_empty() {
                methods.push(sig);
            }
        }
        *i += 1;
    }
    methods
}

fn extract_doc_comment(lines: &[&str], idx: usize) -> Option<String> {
    if idx == 0 {
        return None;
    }
    let mut docs = Vec::new();
    let mut j = idx;
    while j > 0 {
        j -= 1;
        let line = lines[j].trim();
        if line.starts_with("///") {
            docs.push(line.strip_prefix("///").unwrap_or(line).trim().to_string());
        } else if line.starts_with("//!") {
            docs.push(line.strip_prefix("//!").unwrap_or(line).trim().to_string());
        } else if line.is_empty() || line.starts_with("#[") {
            continue;
        } else {
            break;
        }
    }
    docs.reverse();
    if docs.is_empty() {
        None
    } else {
        Some(docs.join(" "))
    }
}

fn first_sentence(doc: &str) -> Option<String> {
    let first = doc.split(". ").next().unwrap_or(doc);
    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}

fn find_line_number(_lines: &[&str], idx: usize) -> usize {
    idx + 1
}

/// Build cross-references between items.
fn build_cross_references(items: &mut [ApiItem]) {
    // Build trait → subtrait relationships from supertrait data
    // For each trait T, find all items whose supertraits include T
    // and add them as subtraits of T.
    let trait_names: Vec<(String, Vec<String>)> = items
        .iter()
        .filter(|i| i.kind == "trait")
        .map(|i| (i.name.clone(), i.supertraits.clone()))
        .collect();

    for (trait_name, _supertraits) in &trait_names {
        // Find all traits that list this one as a supertrait
        let subtraits: Vec<String> = items
            .iter()
            .filter(|i| i.supertraits.contains(trait_name))
            .map(|i| i.name.clone())
            .collect();

        if let Some(item) = items.iter_mut().find(|i| &i.name == trait_name) {
            item.subtraits = subtraits;
        }
    }
}

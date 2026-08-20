// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Imported-symbol analysis — which catalog symbols a target project
//! actually consumes (the deferred 19-B/19-C slice).
//!
//! [`analyze_imports`] parses every `use` statement under a target project
//! root (a real `syn` parse, same walker conventions as the catalog
//! extractor) and resolves each leaf against a [`Catalog`]:
//!
//! - **resolved** — the import names a real catalog item, reported as a
//!   qualified `<crate>::<item>` symbol ref (the overlay's reference format),
//!   with the item kind, local names (aliases included), per-file spread and
//!   occurrence counts. Resolution is leaf-tolerant: intermediate module
//!   segments (`use karpal_core::functor::Functor`) do not block matching,
//!   mirroring re-export tolerance.
//! - **unresolved** — the import names a catalog crate but no item of that
//!   crate (the drift signal: a stale reference to a removed/renamed item).
//! - **globs** — `use a::b::*` imports, recorded by path but never expanded
//!   (a glob does not name items).
//!
//! Imports of crates outside the catalog (`std`, external deps) are ignored
//! entirely, and bare module imports (`use karpal_core::functor;`) are
//! neither resolved items nor unresolved symbols.
//!
//! The overlay join ([`ImportsReport::concepts_used`]) then answers the
//! discovery question: which curated mathematical concepts does this project
//! actually use?

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use syn::visit::Visit;
use syn::{ItemUse, UseTree};
use walkdir::WalkDir;

use crate::catalog::{Catalog, ItemKind};
use crate::overlay::{ConceptOverlay, ConceptRecord};

/// The result of analyzing a project's imports against a catalog.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportsReport {
    /// Catalog items the project imports, sorted by `symbol_ref`.
    pub resolved: Vec<ResolvedImport>,
    /// Imports naming a catalog crate but no item of it, sorted by `path`.
    pub unresolved: Vec<UnresolvedImport>,
    /// Glob imports naming a catalog crate, sorted by `path`.
    pub globs: Vec<GlobImport>,
}

/// One resolved catalog item, aggregated across every import site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedImport {
    /// Qualified catalog reference, `<crate>::<item>`.
    pub symbol_ref: String,
    /// The name bound locally (the bare item name if any import used it,
    /// otherwise the lexicographically first alias).
    pub local_name: String,
    /// The catalog item kind, when the item lives in the cataloged
    /// workspace. `None` when the symbol resolves through a re-export whose
    /// origin is an external crate (e.g. a cross-repo dependency) — the
    /// re-export site is then the catalog-visible anchor.
    pub kind: Option<ItemKind>,
    /// Distinct files importing the item.
    pub files: usize,
    /// Total import occurrences (aliases included).
    pub occurrences: usize,
}

/// An import of a catalog crate that resolved to no item — the drift signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedImport {
    /// The import path as written (crate in `_` form).
    pub path: String,
    /// The name bound locally.
    pub local_name: String,
    /// Distinct files carrying the import.
    pub files: usize,
}

/// A `use …::*` import of a catalog crate, recorded but never expanded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobImport {
    /// The glob path as written (`crate::module::*`).
    pub path: String,
    /// Distinct files carrying the glob.
    pub files: usize,
}

/// One flattened `use` leaf: the full path segments, an optional rename, and
/// whether the leaf was a glob.
#[derive(Debug, Clone)]
struct UseLeaf {
    segments: Vec<String>,
    alias: Option<String>,
    glob: bool,
}

/// Collects [`ItemUse`] leaves across a whole file, remembering the current
/// file for per-file aggregation.
#[derive(Default)]
struct UseCollector {
    file: String,
    leaves: Vec<(String, UseLeaf)>,
}

impl<'ast> Visit<'ast> for UseCollector {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        let mut segments = Vec::new();
        flatten(
            &item.tree,
            &mut segments,
            &mut self.leaves,
            &self.file.clone(),
        );
    }
}

/// Recursively flatten a [`UseTree`] into path-qualified leaves.
fn flatten(
    tree: &UseTree,
    segments: &mut Vec<String>,
    out: &mut Vec<(String, UseLeaf)>,
    file: &str,
) {
    match tree {
        UseTree::Path(path) => {
            segments.push(path.ident.to_string());
            flatten(&path.tree, segments, out, file);
        }
        UseTree::Name(name) => {
            let mut leaf_segment = segments.clone();
            leaf_segment.push(name.ident.to_string());
            out.push((
                file.to_string(),
                UseLeaf {
                    segments: leaf_segment,
                    alias: None,
                    glob: false,
                },
            ));
        }
        UseTree::Rename(rename) => {
            let mut leaf_segment = segments.clone();
            leaf_segment.push(rename.ident.to_string());
            out.push((
                file.to_string(),
                UseLeaf {
                    segments: leaf_segment,
                    alias: Some(rename.rename.to_string()),
                    glob: false,
                },
            ));
        }
        UseTree::Glob(_) => {
            out.push((
                file.to_string(),
                UseLeaf {
                    segments: segments.clone(),
                    alias: None,
                    glob: true,
                },
            ));
        }
        UseTree::Group(group) => {
            for inner in &group.items {
                flatten(inner, segments, out, file);
            }
        }
    }
}

/// Analyze the imports under `root` against `catalog`.
///
/// Read-only: parses `.rs` files with `syn`, skipping `target/` and `.git`.
/// Unparseable files are skipped (mirroring the catalog extractor).
#[must_use]
pub fn analyze_imports(root: &Path, catalog: &Catalog) -> ImportsReport {
    // `_`-form crate name (as written in `use` paths) → package name.
    let crate_lookup: BTreeMap<String, &str> = catalog
        .crates
        .keys()
        .map(|k| (k.replace('-', "_"), k.as_str()))
        .collect();

    let mut collector = UseCollector::default();
    for entry in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| !is_pruned(entry))
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(file) = syn::parse_file(&source) else {
            continue;
        };
        collector.file = entry.path().to_string_lossy().into_owned();
        collector.visit_file(&file);
    }

    // Aggregation state, keyed for deterministic output.
    let mut resolved: BTreeMap<String, AggResolved> = BTreeMap::new();
    let mut unresolved: BTreeMap<String, AggUnresolved> = BTreeMap::new();
    let mut globs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for (file, leaf) in &collector.leaves {
        let Some(first) = leaf.segments.first() else {
            continue;
        };
        // Only imports of catalog crates are in scope.
        let Some((_, package)) = crate_lookup.get_key_value(first.as_str()) else {
            continue;
        };

        if leaf.glob {
            let key = format!("{}::*", leaf.segments.join("::"));
            globs.entry(key).or_default().insert(file.clone());
            continue;
        }

        let local = leaf
            .segments
            .last()
            .cloned()
            .unwrap_or_else(|| first.clone());
        let alias = leaf.alias.clone().unwrap_or_else(|| local.clone());

        // Bare module import (a trailing `self` or a module name): not an
        // item and not drift.
        let module_import = if local == "self" {
            true
        } else {
            is_module_of(catalog, package, &local)
        };
        if module_import {
            continue;
        }

        // Direct item hit, then re-export resolution (intra-crate rename or
        // cross-crate re-export), then external re-export (an origin crate
        // outside the cataloged workspace — resolved at the re-export site).
        let (symbol_ref, kind) = match catalog
            .crates
            .get(*package)
            .and_then(|c| c.items.iter().find(|item| item.name == local))
        {
            Some(record) => (
                format!("{}::{}", record.crate_name, record.name),
                Some(record.kind.clone()),
            ),
            None => match resolve_via_reexport(catalog, package, &local) {
                Some(Resolution::InCatalog(record)) => (
                    format!("{}::{}", record.crate_name, record.name),
                    Some(record.kind.clone()),
                ),
                Some(Resolution::ExternalReexport) => (format!("{}::{}", package, local), None),
                None => {
                    unresolved
                        .entry(leaf.segments.join("::"))
                        .or_insert(AggUnresolved {
                            local_name: alias,
                            files: BTreeSet::new(),
                        })
                        .files
                        .insert(file.clone());
                    continue;
                }
            },
        };
        let agg = resolved.entry(symbol_ref).or_insert_with(|| AggResolved {
            kind,
            local_names: BTreeSet::new(),
            files: BTreeSet::new(),
            occurrences: 0,
        });
        agg.local_names.insert(alias);
        agg.files.insert(file.clone());
        agg.occurrences += 1;
    }

    ImportsReport {
        resolved: resolved
            .into_iter()
            .map(|(symbol_ref, agg)| {
                let bare = symbol_ref
                    .rsplit("::")
                    .next()
                    .unwrap_or(symbol_ref.as_str())
                    .to_string();
                let local_name = agg
                    .local_names
                    .iter()
                    .find(|n| n.as_str() == bare)
                    .cloned()
                    .or_else(|| agg.local_names.iter().next().cloned())
                    .unwrap_or_default();
                ResolvedImport {
                    symbol_ref,
                    local_name,
                    kind: agg.kind,
                    files: agg.files.len(),
                    occurrences: agg.occurrences,
                }
            })
            .collect(),
        unresolved: unresolved
            .into_iter()
            .map(|(path, agg)| UnresolvedImport {
                path,
                local_name: agg.local_name,
                files: agg.files.len(),
            })
            .collect(),
        globs: globs
            .into_iter()
            .map(|(path, files)| GlobImport {
                path,
                files: files.len(),
            })
            .collect(),
    }
}

/// Per-symbol aggregation for resolved imports.
struct AggResolved {
    kind: Option<ItemKind>,
    local_names: BTreeSet<String>,
    files: BTreeSet<String>,
    occurrences: usize,
}

/// Per-path aggregation for unresolved imports.
struct AggUnresolved {
    local_name: String,
    files: BTreeSet<String>,
}

/// Resolve an import leaf through the crate's re-export table: find a
/// re-export bound to `local`, take its origin's defining segment, and look
/// for that item in the re-exporting crate first (the common intra-crate
/// rename) then across the whole catalog (the cross-crate re-export case).
fn resolve_via_reexport<'a>(
    catalog: &'a Catalog,
    package: &str,
    local: &str,
) -> Option<Resolution<'a>> {
    let record = catalog.crates.get(package)?;
    let reexport = record.reexports.iter().find(|r| r.name == local)?;
    let target = reexport.origin.rsplit("::").next()?.to_string();
    let in_crate = record.items.iter().find(|item| item.name == target);
    let resolution = in_crate.map(Resolution::InCatalog).or_else(|| {
        catalog
            .crates
            .values()
            .flat_map(|c| c.items.iter())
            .find(|item| item.name == target)
            .map(Resolution::InCatalog)
    });
    // An external origin crate (outside the cataloged workspace) still
    // resolves — at the re-export site.
    Some(resolution.unwrap_or(Resolution::ExternalReexport))
}

/// The outcome of re-export resolution.
enum Resolution<'a> {
    /// The defining item lives in the catalog.
    InCatalog(&'a crate::catalog::ItemRecord),
    /// The origin crate is external to the workspace catalog; the re-export
    /// site is the anchor.
    ExternalReexport,
}

/// Is `name` a public module of `package` (by module-path leaf)?
fn is_module_of(catalog: &Catalog, package: &str, name: &str) -> bool {
    catalog.crates.get(package).is_some_and(|record| {
        record
            .modules
            .iter()
            .any(|m| m.path.rsplit("::").next() == Some(name))
    })
}

/// Walker prune rule, mirroring the catalog extractor: `target/` and `.git/`.
fn is_pruned(entry: &walkdir::DirEntry) -> bool {
    entry.depth() > 0
        && entry.file_type().is_dir()
        && matches!(entry.file_name().to_str(), Some("target") | Some(".git"))
}

impl ImportsReport {
    /// Join the resolved imports against a concept overlay: the curated
    /// mathematical concepts this project actually uses, sorted by concept
    /// id. This is the discovery answer to "what is this project doing,
    /// categorically?"
    pub fn concepts_used<'o>(&self, overlay: &'o ConceptOverlay) -> Vec<&'o ConceptRecord> {
        let mut by_id: BTreeMap<&str, &ConceptRecord> = BTreeMap::new();
        for import in &self.resolved {
            if let Some(concept) = overlay.concept_for_symbol(&import.symbol_ref) {
                by_id.insert(concept.id.as_str(), concept);
            }
        }
        by_id.into_values().collect()
    }
}

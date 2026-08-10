// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The workspace extractor — walks a Karpal workspace and builds a [`Catalog`]
//! from the public API surface via a real `syn` AST walk.
//!
//! Slice 1 catalogues **public traits** (plus crate metadata and modules). A
//! real `syn` parse replaces `karpal-index`'s string scanning, preserving
//! generic bounds, supertraits, associated items, and method shape. Functions,
//! types, and macros arrive in later slices; the `visit`-style top-level walk
//! here is the pattern they will extend.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::{Attribute, ItemEnum, ItemFn, ItemStruct, ItemTrait, ItemType, TraitItem};
use walkdir::WalkDir;

use crate::catalog::{
    Catalog, CrateRecord, EnumRecord, FunctionRecord, ItemKind, ItemRecord, MethodRecord,
    ModuleRecord, StructRecord, TraitRecord, TypeAliasRecord,
};

/// Extract a structural catalog from a workspace root.
///
/// Walks the workspace, finds every crate directory (`Cargo.toml` + `src/`),
/// and parses its public API surface. Returns an empty catalog for a path with
/// no crates.
pub fn extract_workspace(root: &Path) -> Catalog {
    let mut catalog = Catalog::new();
    for cargo_toml in find_crate_manifests(root) {
        let Some(record) = extract_crate(&cargo_toml) else {
            continue;
        };
        catalog.crates.insert(record.name.clone(), record);
    }
    catalog
}

/// Discover every `Cargo.toml` whose directory also contains a `src/` tree.
/// `target/` build output and `.git` are pruned so only source crates are
/// catalogued.
fn find_crate_manifests(root: &Path) -> Vec<PathBuf> {
    let mut manifests = Vec::new();
    for entry in WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_entry(|entry| !is_build_dir(entry))
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "Cargo.toml" {
            continue;
        }
        let dir = entry.path().parent().expect("Cargo.toml has a parent");
        if dir.join("src").is_dir() {
            manifests.push(entry.path().to_path_buf());
        }
    }
    manifests.sort();
    manifests
}

/// Whether a directory entry is build output or VCS metadata to skip.
fn is_build_dir(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && matches!(entry.file_name().to_str(), Some("target") | Some(".git"))
}

/// Parse one crate's manifest + source tree into a [`CrateRecord`].
fn extract_crate(cargo_toml: &Path) -> Option<CrateRecord> {
    let crate_dir = cargo_toml.parent()?;
    let src_dir = crate_dir.join("src");
    let meta = parse_manifest(cargo_toml);
    if meta.name.is_empty() {
        return None;
    }
    let crate_root = meta.name.replace('-', "_");
    let mut modules = Vec::new();
    let mut items = Vec::new();
    for source in rust_sources(&src_dir) {
        let module_path = match module_path_for(&src_dir, &source, &crate_root) {
            Some(path) => path,
            None => continue,
        };
        let Ok(content) = std::fs::read_to_string(&source) else {
            continue;
        };
        let file = match syn::parse_file(&content) {
            Ok(file) => file,
            Err(_) => continue, // skip unparseable files rather than aborting the walk
        };
        let docs = extract_doc(&file.attrs);
        modules.push(ModuleRecord {
            path: module_path.clone(),
            docs,
        });
        for item in &file.items {
            if let Some(record) = extract_item(item, &meta.name, &module_path) {
                items.push(record);
            }
        }
    }
    modules.sort_by(|a, b| a.path.cmp(&b.path));
    items.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.module_path.cmp(&b.module_path))
    });
    Some(CrateRecord {
        name: meta.name,
        version: meta.version,
        description: meta.description,
        features: meta.features,
        dependencies: meta.dependencies,
        modules,
        items,
    })
}

/// Crate metadata parsed from a `Cargo.toml` manifest.
#[derive(Default)]
struct ManifestMeta {
    name: String,
    version: Option<String>,
    description: Option<String>,
    features: BTreeMap<String, Vec<String>>,
    dependencies: BTreeMap<String, String>,
}

/// Parse crate metadata from a `Cargo.toml`.
fn parse_manifest(cargo_toml: &Path) -> ManifestMeta {
    let Ok(content) = std::fs::read_to_string(cargo_toml) else {
        return ManifestMeta::default();
    };
    let Ok(table) = content.parse::<toml::Table>() else {
        return ManifestMeta::default();
    };
    let package = table.get("package").and_then(toml::Value::as_table);
    let name = package
        .and_then(|p| p.get("name"))
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let version = package
        .and_then(|p| p.get("version"))
        .and_then(toml::Value::as_str)
        .map(str::to_string);
    let description = package
        .and_then(|p| p.get("description"))
        .and_then(toml::Value::as_str)
        .map(str::to_string);
    let features = table
        .get("features")
        .and_then(toml::Value::as_table)
        .map(|features_table| {
            features_table
                .iter()
                .map(|(key, value)| {
                    let list = value
                        .as_array()
                        .map(|array| {
                            array
                                .iter()
                                .filter_map(toml::Value::as_str)
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default();
                    (key.clone(), list)
                })
                .collect()
        })
        .unwrap_or_default();
    let dependencies = table
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .map(|deps_table| {
            deps_table
                .iter()
                .map(|(key, value)| {
                    let req = match value {
                        toml::Value::String(s) => s.clone(),
                        toml::Value::Table(t) => t
                            .get("version")
                            .and_then(toml::Value::as_str)
                            .or_else(|| t.get("path").and_then(toml::Value::as_str))
                            .map(str::to_string)
                            .unwrap_or_default(),
                        other => other.to_string(),
                    };
                    (key.clone(), req)
                })
                .collect()
        })
        .unwrap_or_default();
    ManifestMeta {
        name,
        version,
        description,
        features,
        dependencies,
    }
}

/// Every `.rs` file beneath a crate's `src/` directory, deterministically ordered.
fn rust_sources(src_dir: &Path) -> Vec<PathBuf> {
    let mut sources: Vec<PathBuf> = WalkDir::new(src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .map(|entry| entry.path().to_path_buf())
        .collect();
    sources.sort();
    sources
}

/// Compute the fully-qualified module path for a source file.
fn module_path_for(src_dir: &Path, file: &Path, crate_root: &str) -> Option<String> {
    let rel = file.strip_prefix(src_dir).ok()?;
    let mut rel = rel.to_path_buf();
    rel.set_extension("");
    let components: Vec<String> = rel
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    let mut segments = Vec::new();
    for (index, component) in components.iter().enumerate() {
        let is_last = index == components.len() - 1;
        if is_last && matches!(component.as_str(), "lib" | "mod" | "main") {
            continue;
        }
        segments.push(component.clone());
    }
    if segments.is_empty() {
        Some(crate_root.to_string())
    } else {
        Some(format!("{}::{}", crate_root, segments.join("::")))
    }
}

/// Build the trait payload from a parsed `syn::ItemTrait`.
fn extract_trait_kind(trait_item: &ItemTrait) -> TraitRecord {
    let supertraits = trait_item
        .supertraits
        .iter()
        .filter_map(|bound| {
            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                Some(normalize_tokens(
                    &trait_bound.path.to_token_stream().to_string(),
                ))
            } else {
                None
            }
        })
        .collect();
    let mut associated_items = Vec::new();
    let mut methods = Vec::new();
    for member in &trait_item.items {
        match member {
            TraitItem::Const(const_item) => associated_items.push(const_item.ident.to_string()),
            TraitItem::Type(type_item) => associated_items.push(type_item.ident.to_string()),
            TraitItem::Fn(fn_item) => {
                methods.push(MethodRecord {
                    name: fn_item.sig.ident.to_string(),
                    signature: normalize_tokens(&fn_item.sig.to_token_stream().to_string()),
                    is_required: fn_item.default.is_none(),
                });
            }
            _ => {}
        }
    }
    TraitRecord {
        supertraits,
        generics: generics_of(&trait_item.generics),
        associated_items,
        methods,
    }
}

/// Build the function payload from a parsed `syn::ItemFn`.
fn extract_function(fn_item: &ItemFn) -> FunctionRecord {
    FunctionRecord {
        signature: normalize_tokens(&fn_item.sig.to_token_stream().to_string()),
        is_async: fn_item.sig.asyncness.is_some(),
        is_const: fn_item.sig.constness.is_some(),
        is_unsafe: matches!(fn_item.sig.safety, syn::Safety::Unsafe(_)),
    }
}

/// Build the struct payload from a parsed `syn::ItemStruct`.
fn extract_struct(struct_item: &ItemStruct) -> StructRecord {
    let fields = match &struct_item.fields {
        syn::Fields::Named(named) => named
            .named
            .iter()
            .map(|field| {
                field
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default()
            })
            .collect(),
        _ => Vec::new(),
    };
    StructRecord {
        generics: generics_of(&struct_item.generics),
        derives: derives(&struct_item.attrs),
        fields,
    }
}

/// Build the enum payload from a parsed `syn::ItemEnum`.
fn extract_enum(enum_item: &ItemEnum) -> EnumRecord {
    let variants = enum_item
        .variants
        .iter()
        .map(|variant| variant.ident.to_string())
        .collect();
    EnumRecord {
        generics: generics_of(&enum_item.generics),
        derives: derives(&enum_item.attrs),
        variants,
    }
}

/// Build the type-alias payload from a parsed `syn::ItemType`.
fn extract_type_alias(type_item: &ItemType) -> TypeAliasRecord {
    TypeAliasRecord {
        generics: generics_of(&type_item.generics),
        aliased_type: normalize_tokens(&type_item.ty.to_token_stream().to_string()),
    }
}

/// Dispatch one top-level item to its kind-specific payload, returning a
/// catalogued [`ItemRecord`] for public items and `None` otherwise.
fn extract_item(item: &syn::Item, crate_name: &str, module_path: &str) -> Option<ItemRecord> {
    let (name, kind, attrs): (String, ItemKind, &[Attribute]) = match item {
        syn::Item::Trait(t) if is_public(&t.vis) => (
            t.ident.to_string(),
            ItemKind::Trait(extract_trait_kind(t)),
            &t.attrs,
        ),
        syn::Item::Fn(f) if is_public(&f.vis) => (
            f.sig.ident.to_string(),
            ItemKind::Function(extract_function(f)),
            &f.attrs,
        ),
        syn::Item::Struct(s) if is_public(&s.vis) => (
            s.ident.to_string(),
            ItemKind::Struct(extract_struct(s)),
            &s.attrs,
        ),
        syn::Item::Enum(e) if is_public(&e.vis) => (
            e.ident.to_string(),
            ItemKind::Enum(extract_enum(e)),
            &e.attrs,
        ),
        syn::Item::Type(t) if is_public(&t.vis) => (
            t.ident.to_string(),
            ItemKind::TypeAlias(extract_type_alias(t)),
            &t.attrs,
        ),
        _ => return None,
    };
    Some(ItemRecord {
        name,
        crate_name: crate_name.to_string(),
        module_path: module_path.to_string(),
        kind,
        docs: extract_doc(attrs),
        cfg: cfg_gate(attrs),
    })
}

/// `#[derive(...)]` trait names on an item, in source order.
fn derives(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .filter_map(|attr| attr.meta.require_list().ok())
        .filter_map(|list| {
            list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .ok()
        })
        .flat_map(|paths| {
            paths.into_iter().map(|path| {
                path.segments
                    .last()
                    .map(|segment| segment.ident.to_string())
                    .unwrap_or_else(|| path.to_token_stream().to_string())
            })
        })
        .collect()
}

/// Normalized generic-parameter string for `syn::Generics`, or `None` if empty.
fn generics_of(generics: &syn::Generics) -> Option<String> {
    let normalized = normalize_tokens(&generics.to_token_stream().to_string());
    (!normalized.is_empty()).then_some(normalized)
}

/// Whether a `syn::Visibility` denotes a public item.
fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// Collect `#[doc = "..."]` attribute lines into a single doc string.
fn extract_doc(attrs: &[Attribute]) -> Option<String> {
    let lines: Vec<String> = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| attr.meta.require_name_value().ok())
        .filter_map(|meta| {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &meta.value
            {
                Some(s.value().trim_start().to_string())
            } else {
                None
            }
        })
        .collect();
    let joined = lines.join("\n");
    let trimmed = joined.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// Reconstruct a raw `cfg(...)` gate string, if present.
fn cfg_gate(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("cfg"))
        .map(|attr| attr.meta.to_token_stream().to_string())
}

/// Collapse the spacing `quote`-style token streams add, so generics and
/// signatures are stored compactly (e.g. `<B>` rather than `< B >`).
fn normalize_tokens(raw: &str) -> String {
    raw.replace("< ", "<")
        .replace(" >", ">")
        .replace(" ,", ",")
        .replace(", ", ",")
        .replace(" ::", "::")
        .replace(":: ", "::")
        .replace(" :", ":")
        .replace(": ", ":")
        .replace(" &", "&")
        .replace("& ", "&")
        .replace(" (", "(")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(") ", ")")
        .replace(" ;", ";")
        .replace("-> ", "->")
        .replace(" ->", "->")
        .replace("  ", " ")
        .trim()
        .to_string()
}

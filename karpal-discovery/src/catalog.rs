// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The structural catalog data model.
//!
//! Every map is a [`BTreeMap`] so that serialization is byte-stable and
//! content-hashable, mirroring `amari-discovery`'s catalog discipline. These are provisional, plain-`serde` types: they carry **no**
//! dependency on the lonis `Block` contract, which wraps only the *output*
//! layer built in later slices.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A complete structural catalog of a Karpal workspace.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Catalog {
    /// Workspace crates keyed by crate name, deterministically ordered.
    pub crates: BTreeMap<String, CrateRecord>,
}

impl Catalog {
    /// Construct an empty catalog.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of catalogued crates.
    #[must_use]
    pub fn crate_count(&self) -> usize {
        self.crates.len()
    }

    /// Total number of catalogued items across all crates.
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.crates.values().map(|record| record.items.len()).sum()
    }

    /// Find the first item named `name` anywhere in the catalog.
    pub fn find_item(&self, name: &str) -> Option<&ItemRecord> {
        self.crates
            .values()
            .flat_map(|record| &record.items)
            .find(|item| item.name == name)
    }

    /// Canonical SHA-256 content hash of the catalog's JSON serialization.
    ///
    /// Because every collection is ordered, identical catalogs produce
    /// identical hashes. This is the drift-detection primitive the later
    /// hardening slice will check into CI.
    #[must_use]
    pub fn content_hash(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("catalog serializes");
        hex::encode(Sha256::digest(&bytes))
    }
}

/// One workspace crate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrateRecord {
    /// Crate name (package name, with `-` kept as in `Cargo.toml`).
    pub name: String,
    /// Package version, if declared.
    pub version: Option<String>,
    /// One-line description, if declared.
    pub description: Option<String>,
    /// Feature flags and the features they enable.
    pub features: BTreeMap<String, Vec<String>>,
    /// Workspace/crate dependencies (name → version requirement or path).
    pub dependencies: BTreeMap<String, String>,
    /// Public modules reachable from the crate root.
    pub modules: Vec<ModuleRecord>,
    /// Public top-level items declared in this crate.
    pub items: Vec<ItemRecord>,
}

/// A public module path within a crate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleRecord {
    /// Fully-qualified module path, e.g. `karpal_core::functor`.
    pub path: String,
    /// Doc comment, if any.
    pub docs: Option<String>,
}

/// A public top-level item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRecord {
    /// Item name (trait/type/function identifier).
    pub name: String,
    /// Owning crate name.
    pub crate_name: String,
    /// Module the item lives in (`crate_name` for root, `crate_name::module` otherwise).
    pub module_path: String,
    /// The kind-specific payload.
    pub kind: ItemKind,
    /// Doc comment, if any.
    pub docs: Option<String>,
    /// Raw `cfg` gate string if the item is feature/config gated, else `None`.
    pub cfg: Option<String>,
}

/// Kind-specific payload for a catalogued item.
///
/// `Trait` is the only variant in this slice; `Function`, `Type`, and `Macro`
/// arrive in later slices. The enum is `#[serde(tag = "kind")]` so adding
/// variants is additive on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ItemKind {
    /// A public trait.
    Trait(TraitRecord),
}

/// A public trait.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraitRecord {
    /// Direct supertraits (as written, e.g. `Functor`).
    pub supertraits: Vec<String>,
    /// Raw generic parameters, e.g. `<A, B>`, if any.
    pub generics: Option<String>,
    /// Names of associated types and constants.
    pub associated_items: Vec<String>,
    /// Methods (required and provided).
    pub methods: Vec<MethodRecord>,
}

/// One trait method.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MethodRecord {
    /// Method name.
    pub name: String,
    /// Reconstructed signature string.
    pub signature: String,
    /// Whether the method is required (`true`) or has a default body (`false`).
    pub is_required: bool,
}

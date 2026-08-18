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

    /// Every type that implements `trait_name`, across all crates, sorted and
    /// de-duplicated. Powers the discovery query "implementors of X".
    #[must_use]
    pub fn implementors_of(&self, trait_name: &str) -> Vec<String> {
        let mut names: Vec<String> = self
            .crates
            .values()
            .flat_map(|record| &record.impls)
            .filter(|implementation| implementation.trait_name == trait_name)
            .map(|implementation| implementation.implementor.clone())
            .collect();
        names.sort();
        names.dedup();
        names
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
    /// Trait implementations found in this crate (`impl Trait for Type`).
    pub impls: Vec<ImplRecord>,
    /// Public modules reachable from the crate root.
    pub modules: Vec<ModuleRecord>,
    /// `pub use` re-export leaves (including renames), sorted by name. Glob
    /// and `std`/`core`/`alloc` re-exports are skipped.
    #[serde(default)]
    pub reexports: Vec<ReexportRecord>,
    /// Public top-level items declared in this crate.
    pub items: Vec<ItemRecord>,
}

/// A public const item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstRecord {
    /// The const's type, rendered as written, when parseable.
    pub ty: Option<String>,
}

/// One trait implementation (`impl Trait for Type`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImplRecord {
    /// The implementing type's name (leading segment).
    pub implementor: String,
    /// The implemented trait's name (last path segment).
    pub trait_name: String,
}

/// One `pub use` re-export leaf: the locally-bound name (a rename when
/// `as` was used) and the origin path as written.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReexportRecord {
    /// The name bound locally (post-rename).
    pub name: String,
    /// The re-exported path as written (e.g. `self::MonteCarloVerifier`,
    /// `karpal_proof_derive::VerifySemigroup`).
    pub origin: String,
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
/// `Trait`, the core value/type/function kinds (slice 2), and macros
/// (slice 4): declarative `macro_rules!` plus the three procedural flavors.
/// The enum is `#[non_exhaustive]` and `#[serde(tag = "kind")]` so adding
/// variants is additive on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ItemKind {
    /// A public trait.
    Trait(TraitRecord),
    /// A public free function.
    Function(FunctionRecord),
    /// A public struct.
    Struct(StructRecord),
    /// A public const.
    Const(ConstRecord),
    /// A public enum.
    Enum(EnumRecord),
    /// A public type alias.
    TypeAlias(TypeAliasRecord),
    /// A macro definition (declarative or procedural).
    Macro(MacroRecord),
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

/// A public free function.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionRecord {
    /// Reconstructed signature string.
    pub signature: String,
    /// Whether the function is `async`.
    pub is_async: bool,
    /// Whether the function is `const`.
    pub is_const: bool,
    /// Whether the function is `unsafe`.
    pub is_unsafe: bool,
}

/// A public struct.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructRecord {
    /// Raw generic parameters, if any.
    pub generics: Option<String>,
    /// `#[derive(...)]` trait names, in source order.
    pub derives: Vec<String>,
    /// Named field identifiers (empty for tuple/unit structs).
    pub fields: Vec<String>,
}

/// A public enum.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnumRecord {
    /// Raw generic parameters, if any.
    pub generics: Option<String>,
    /// `#[derive(...)]` trait names, in source order.
    pub derives: Vec<String>,
    /// Variant identifiers, in source order.
    pub variants: Vec<String>,
}

/// A public type alias.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeAliasRecord {
    /// Raw generic parameters, if any.
    pub generics: Option<String>,
    /// The aliased type, as written.
    pub aliased_type: String,
}

/// The flavor of a macro definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroFlavor {
    /// `macro_rules! name { … }` — declarative.
    #[default]
    Declarative,
    /// `#[proc_macro] pub fn name` — function-like procedural.
    Function,
    /// `#[proc_macro_attribute] pub fn name` — attribute procedural.
    Attribute,
    /// `#[proc_macro_derive(Trait, …)] pub fn name` — derive procedural.
    Derive,
}

/// A macro definition (declarative or procedural).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacroRecord {
    /// The macro flavor.
    pub flavor: MacroFlavor,
    /// For [`MacroFlavor::Derive`], the trait name being derived (from
    /// `#[proc_macro_derive(Trait, …)]`); `None` for other flavors.
    pub derives: Option<String>,
    /// Helper attributes declared by a derive macro
    /// (`#[proc_macro_derive(Trait, attributes(a, b))]`), in source order.
    pub helper_attributes: Vec<String>,
}

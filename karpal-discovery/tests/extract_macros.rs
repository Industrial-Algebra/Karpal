// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for cataloguing macros — declarative (`macro_rules!`)
//! and procedural (function/attribute/derive) — slice 4 of the structural
//! catalog.

use std::path::{Path, PathBuf};

use karpal_discovery::{Catalog, ItemKind, MacroFlavor, extract::extract_workspace};

const CRATE_TOML: &str = "\
[package]
name = \"fixture-macros\"
version = \"0.1.0\"
edition = \"2024\"
";

// `extern crate proc_macro` + `use proc_macro::TokenStream` parse fine under
// syn (the catalog never compiles, only parses) — so a single fixture can
// carry all four macro flavors.
const LIB_RS: &str = "\
//! A fixture crate for macros.

extern crate proc_macro;
use proc_macro::TokenStream;

/// An exported declarative macro — catalogued.
#[macro_export]
macro_rules! greet {
    () => {};
}

/// A private macro_rules! — neither exported nor pub — skipped.
macro_rules! internal {
    () => {};
}

/// A function-like proc macro.
#[proc_macro]
pub fn make_thing(input: TokenStream) -> TokenStream {
    input
}

/// An attribute proc macro.
#[proc_macro_attribute]
pub fn annotate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// A derive proc macro with a helper attribute.
#[proc_macro_derive(Derivable, attributes(helper))]
pub fn derive_thing(input: TokenStream) -> TokenStream {
    input
}

/// A plain function — must still catalogue as Function, not Macro.
pub fn not_a_macro() {}
";

/// Build the fixture workspace, return the catalog.
fn fixture_catalog() -> (tempfile::TempDir, Catalog) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), CRATE_TOML).unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), LIB_RS).unwrap();
    let catalog = extract_workspace(dir.path());
    (dir, catalog)
}

/// Unpack a catalogued macro, panicking with a clear message if absent or
/// not the Macro kind.
fn macro_record<'a>(catalog: &'a Catalog, name: &str) -> &'a karpal_discovery::MacroRecord {
    let item = catalog
        .find_item(name)
        .unwrap_or_else(|| panic!("`{name}` should be catalogued"));
    match &item.kind {
        ItemKind::Macro(record) => record,
        other => panic!("`{name}` catalogued as {other:?}, expected Macro"),
    }
}

#[test]
fn declarative_macro_rules_is_catalogued() {
    let (_dir, catalog) = fixture_catalog();
    let record = macro_record(&catalog, "greet");
    assert_eq!(record.flavor, MacroFlavor::Declarative);
    assert_eq!(record.derives, None);
    assert!(record.helper_attributes.is_empty());
}

#[test]
fn private_macro_rules_is_skipped() {
    let (_dir, catalog) = fixture_catalog();
    assert!(
        catalog.find_item("internal").is_none(),
        "a non-exported, non-pub macro_rules! must not be catalogued"
    );
}

#[test]
fn function_proc_macro_is_catalogued() {
    let (_dir, catalog) = fixture_catalog();
    let record = macro_record(&catalog, "make_thing");
    assert_eq!(record.flavor, MacroFlavor::Function);
}

#[test]
fn attribute_proc_macro_is_catalogued() {
    let (_dir, catalog) = fixture_catalog();
    let record = macro_record(&catalog, "annotate");
    assert_eq!(record.flavor, MacroFlavor::Attribute);
}

#[test]
fn derive_proc_macro_carries_trait_and_helpers() {
    let (_dir, catalog) = fixture_catalog();
    let record = macro_record(&catalog, "derive_thing");
    assert_eq!(record.flavor, MacroFlavor::Derive);
    assert_eq!(record.derives.as_deref(), Some("Derivable"));
    assert_eq!(record.helper_attributes, vec!["helper"]);
}

#[test]
fn plain_function_is_still_function_not_macro() {
    let (_dir, catalog) = fixture_catalog();
    let item = catalog
        .find_item("not_a_macro")
        .expect("plain fn catalogued");
    assert!(matches!(item.kind, ItemKind::Function(_)), "not a macro");
}

// -- real-workspace assertions (the actual Karpal fixtures) --

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root resolves")
}

#[test]
fn workspace_derive_macros_are_catalogued() {
    let catalog = extract_workspace(&workspace_root());
    // karpal-proof-derive: `#[proc_macro_derive(VerifySemigroup, attributes(verify))]`
    // on `pub fn derive_verify_semigroup`.
    let record = macro_record(&catalog, "derive_verify_semigroup");
    assert_eq!(record.flavor, MacroFlavor::Derive);
    assert_eq!(record.derives.as_deref(), Some("VerifySemigroup"));
    assert_eq!(record.helper_attributes, vec!["verify"]);
}

#[test]
fn workspace_attribute_macro_is_catalogued() {
    let catalog = extract_workspace(&workspace_root());
    // karpal-verify-derive: `#[proc_macro_attribute] pub fn export_obligations`.
    let record = macro_record(&catalog, "export_obligations");
    assert_eq!(record.flavor, MacroFlavor::Attribute);
}

#[test]
fn workspace_declarative_macro_is_catalogued() {
    let catalog = extract_workspace(&workspace_root());
    // karpal-core/src/macros.rs: `#[macro_export] macro_rules! do_`.
    let record = macro_record(&catalog, "do_");
    assert_eq!(record.flavor, MacroFlavor::Declarative);
}

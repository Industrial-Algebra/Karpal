// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for the workspace extractor: a hermetic fixture crate is
//! written to a tempdir and the extracted catalog is checked for shape and
//! visibility filtering.

use std::fs;

use karpal_discovery::{Catalog, ItemKind, extract::extract_workspace};
use tempfile::TempDir;

const CRATE_TOML: &str = "\
[package]
name = \"fixture-crate\"
version = \"0.1.0\"
edition = \"2024\"
description = \"a fixture\"

[dependencies]
";

/// A fixture `lib.rs`: a public trait with a supertrait, generics, an
/// associated type, one required and one provided method, plus a private
/// trait that must be filtered out.
const LIB_RS: &str = "\
//! A fixture crate.

/// A documented public trait.
pub trait Demo<B>: SuperTrait {
    /// the target type
    type Target;

    /// a required method
    fn do_it<A>(&self, a: A) -> Self::Target;

    /// a provided method
    fn default_method(&self) -> u32 {
        42
    }
}

trait PrivateTrait {
    fn hidden(&self);
}
";

fn write_fixture() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let crate_dir = dir.path().join("fixture-crate");
    let src = crate_dir.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(crate_dir.join("Cargo.toml"), CRATE_TOML).expect("write Cargo.toml");
    fs::write(src.join("lib.rs"), LIB_RS).expect("write lib.rs");
    dir
}

#[test]
fn extracts_public_trait_with_full_shape() {
    let dir = write_fixture();
    let catalog: Catalog = extract_workspace(dir.path());

    assert_eq!(catalog.crate_count(), 1, "exactly one crate");
    let krate = catalog
        .crates
        .get("fixture-crate")
        .expect("fixture-crate present");
    assert_eq!(krate.version.as_deref(), Some("0.1.0"));
    assert_eq!(krate.description.as_deref(), Some("a fixture"));

    let demo = catalog.find_item("Demo").expect("Demo present");
    assert_eq!(demo.crate_name, "fixture-crate");
    assert_eq!(demo.module_path, "fixture_crate");
    assert_eq!(demo.docs.as_deref(), Some("A documented public trait."));
    let ItemKind::Trait(tr) = &demo.kind else {
        panic!("Demo is a trait");
    };
    assert_eq!(tr.supertraits, vec!["SuperTrait".to_string()]);
    assert_eq!(tr.generics.as_deref(), Some("<B>"));
    assert!(
        tr.associated_items.contains(&"Target".to_string()),
        "associated type Target recorded: {tr:?}"
    );
    let required = tr
        .methods
        .iter()
        .find(|m| m.name == "do_it")
        .expect("do_it method");
    assert!(required.is_required, "do_it is required");
    let provided = tr
        .methods
        .iter()
        .find(|m| m.name == "default_method")
        .expect("default_method");
    assert!(!provided.is_required, "default_method is provided");
}

#[test]
fn private_traits_are_filtered_out() {
    let dir = write_fixture();
    let catalog = extract_workspace(dir.path());
    assert!(
        catalog.find_item("PrivateTrait").is_none(),
        "no private items"
    );
    assert_eq!(catalog.item_count(), 1, "only the public trait");
}

#[test]
fn empty_workspace_yields_empty_catalog() {
    let dir = TempDir::new().expect("tempdir");
    let catalog = extract_workspace(dir.path());
    assert_eq!(catalog.crate_count(), 0);
    assert_eq!(catalog.item_count(), 0);
}

#[test]
fn extraction_is_deterministic_across_runs() {
    let a = extract_workspace(write_fixture().path());
    let b = extract_workspace(write_fixture().path());
    assert_eq!(a.content_hash(), b.content_hash());
}

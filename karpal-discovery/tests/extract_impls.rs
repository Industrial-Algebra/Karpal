// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Tests for the trait implementation graph: `impl Trait for Type` blocks are
//! catalogued and queryable via `Catalog::implementors_of`.

use std::fs;

use karpal_discovery::extract::extract_workspace;
use tempfile::TempDir;

const CRATE_TOML: &str = "\
[package]
name = \"fixture-impls\"
version = \"0.1.0\"
edition = \"2024\"
";

const LIB_RS: &str = "\
//! A fixture crate.

/// A trait with several implementors.
pub trait Demo {
    fn thing(&self);
}

pub struct Alpha;
impl Demo for Alpha {
    fn thing(&self) {}
}

pub struct Beta;
impl Demo for Beta {
    fn thing(&self) {}
}

/// No implementation of Demo.
pub struct Gamma;

// An inherent impl is not a trait impl and must not appear in the trait graph.
impl Alpha {
    fn inherent(&self) {}
}

// A second trait, to ensure implementors_of is scoped per trait.
pub trait Other {}
impl Other for Beta {}
";

fn write_fixture() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let crate_dir = dir.path().join("fixture-impls");
    let src = crate_dir.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(crate_dir.join("Cargo.toml"), CRATE_TOML).expect("write Cargo.toml");
    fs::write(src.join("lib.rs"), LIB_RS).expect("write lib.rs");
    dir
}

#[test]
fn implementors_of_returns_all_implementors() {
    let catalog = extract_workspace(write_fixture().path());
    let mut implementors = catalog.implementors_of("Demo");
    implementors.sort();
    assert_eq!(
        implementors,
        vec!["Alpha".to_string(), "Beta".to_string()],
        "Alpha and Beta implement Demo"
    );
}

#[test]
fn non_implementors_are_excluded() {
    let catalog = extract_workspace(write_fixture().path());
    let implementors = catalog.implementors_of("Demo");
    assert!(
        !implementors.contains(&"Gamma".to_string()),
        "Gamma does not implement Demo"
    );
}

#[test]
fn implementors_are_scoped_per_trait() {
    let catalog = extract_workspace(write_fixture().path());
    assert_eq!(catalog.implementors_of("Other"), vec!["Beta".to_string()]);
}

#[test]
fn unknown_trait_has_no_implementors() {
    let catalog = extract_workspace(write_fixture().path());
    assert!(catalog.implementors_of("NoSuchTrait").is_empty());
}

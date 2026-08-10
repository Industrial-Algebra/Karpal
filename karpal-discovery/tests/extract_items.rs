// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for cataloguing functions, structs, enums, and type
//! aliases — the non-trait public item kinds added in slice 2.

use karpal_discovery::{ItemKind, extract::extract_workspace};
use std::fs;
use tempfile::TempDir;

const CRATE_TOML: &str = "\
[package]
name = \"fixture-items\"
version = \"0.1.0\"
edition = \"2024\"
";

const LIB_RS: &str = "\
//! A fixture crate.

/// A documented public function.
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

/// An async function.
pub async fn fetch() -> u32 {
    0
}

fn hidden_fn() -> u32 {
    0
}

/// A public struct with named fields.
#[derive(Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

struct HiddenStruct;

/// A public enum.
#[derive(Clone)]
pub enum Shape {
    Circle(f64),
    Square { side: f64 },
    Triangle,
}

/// A public type alias.
pub type Distance = f64;
";

fn write_fixture() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let crate_dir = dir.path().join("fixture-items");
    let src = crate_dir.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(crate_dir.join("Cargo.toml"), CRATE_TOML).expect("write Cargo.toml");
    fs::write(src.join("lib.rs"), LIB_RS).expect("write lib.rs");
    dir
}

#[test]
fn extracts_public_functions_with_flags() {
    let catalog = extract_workspace(write_fixture().path());
    let add = catalog.find_item("add").expect("add present");
    let ItemKind::Function(rec) = &add.kind else {
        panic!("add is a function: {:?}", add.kind);
    };
    assert!(
        rec.signature.starts_with("fn add"),
        "signature: {}",
        rec.signature
    );
    assert!(!rec.is_async);
    assert!(!rec.is_const);
    assert!(!rec.is_unsafe);

    let fetch = catalog.find_item("fetch").expect("fetch present");
    let ItemKind::Function(rec) = &fetch.kind else {
        panic!("fetch is a function");
    };
    assert!(rec.is_async, "fetch is async");
}

#[test]
fn extracts_public_struct_with_derives_and_fields() {
    let catalog = extract_workspace(write_fixture().path());
    let point = catalog.find_item("Point").expect("Point present");
    let ItemKind::Struct(rec) = &point.kind else {
        panic!("Point is a struct: {:?}", point.kind);
    };
    assert!(rec.derives.contains(&"Clone".to_string()));
    assert!(rec.derives.contains(&"Debug".to_string()));
    assert_eq!(rec.fields, vec!["x".to_string(), "y".to_string()]);
}

#[test]
fn extracts_public_enum_with_variants() {
    let catalog = extract_workspace(write_fixture().path());
    let shape = catalog.find_item("Shape").expect("Shape present");
    let ItemKind::Enum(rec) = &shape.kind else {
        panic!("Shape is an enum: {:?}", shape.kind);
    };
    assert_eq!(
        rec.variants,
        vec![
            "Circle".to_string(),
            "Square".to_string(),
            "Triangle".to_string()
        ]
    );
    assert!(rec.derives.contains(&"Clone".to_string()));
}

#[test]
fn extracts_public_type_alias() {
    let catalog = extract_workspace(write_fixture().path());
    let distance = catalog.find_item("Distance").expect("Distance present");
    let ItemKind::TypeAlias(rec) = &distance.kind else {
        panic!("Distance is a type alias: {:?}", distance.kind);
    };
    assert_eq!(rec.aliased_type, "f64");
}

#[test]
fn private_items_are_filtered() {
    let catalog = extract_workspace(write_fixture().path());
    assert!(catalog.find_item("hidden_fn").is_none());
    assert!(catalog.find_item("HiddenStruct").is_none());
}

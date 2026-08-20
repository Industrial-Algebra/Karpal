// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Tests for the catalog data model: serde round-trips, ordering determinism,
//! and the content-hash drift primitive.

use std::collections::BTreeMap;

use karpal_discovery::{Catalog, CrateRecord, ItemKind, ItemRecord, MethodRecord, TraitRecord};

fn sample_catalog() -> Catalog {
    let mut catalog = Catalog::new();
    let mut krate = CrateRecord {
        name: "karpal-core".to_string(),
        version: Some("0.9.0".to_string()),
        description: Some("core".to_string()),
        features: BTreeMap::new(),
        dependencies: BTreeMap::new(),
        impls: Vec::new(),
        modules: Vec::new(),
        items: Vec::new(),
        reexports: Vec::new(),
    };
    krate.items.push(ItemRecord {
        name: "Functor".to_string(),
        crate_name: "karpal-core".to_string(),
        module_path: "karpal_core".to_string(),
        kind: ItemKind::Trait(TraitRecord {
            supertraits: Vec::new(),
            generics: Some("<A, B>".to_string()),
            associated_items: vec!["Target".to_string()],
            methods: vec![MethodRecord {
                name: "fmap".to_string(),
                signature: "fn fmap(...)".to_string(),
                is_required: true,
            }],
        }),
        docs: Some("Functor".to_string()),
        cfg: None,
    });
    catalog.crates.insert("karpal-core".to_string(), krate);
    catalog
}

#[test]
fn serde_round_trips_preserving_shape() {
    let catalog = sample_catalog();
    let json = serde_json::to_string(&catalog).expect("serialize");
    let back: Catalog = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.crate_count(), 1);
    assert_eq!(back.item_count(), 1);
    let item = back.find_item("Functor").expect("Functor present");
    let ItemKind::Trait(tr) = &item.kind else {
        panic!("expected trait kind");
    };
    assert_eq!(tr.associated_items, vec!["Target".to_string()]);
    assert_eq!(tr.methods.len(), 1);
    assert_eq!(tr.methods[0].name, "fmap");
}

#[test]
fn identical_catalogs_have_identical_hashes() {
    let a = sample_catalog();
    let b = sample_catalog();
    assert_eq!(a.content_hash(), b.content_hash());
}

#[test]
fn different_catalogs_have_different_hashes() {
    let mut a = sample_catalog();
    let b = sample_catalog();
    a.crates
        .get_mut("karpal-core")
        .expect("crate")
        .items
        .push(ItemRecord {
            name: "Monad".to_string(),
            crate_name: "karpal-core".to_string(),
            module_path: "karpal_core".to_string(),
            kind: ItemKind::Trait(TraitRecord::default()),
            docs: None,
            cfg: None,
        });
    assert_ne!(a.content_hash(), b.content_hash());
}

#[test]
fn insertion_order_does_not_change_hash() {
    // Two catalogs built with crates inserted in different orders must hash
    // identically: BTreeMap ordering, not insertion ordering, governs output.
    let mut first = Catalog::new();
    first.crates.insert("zzz".to_string(), empty_crate("zzz"));
    first.crates.insert("aaa".to_string(), empty_crate("aaa"));

    let mut second = Catalog::new();
    second.crates.insert("aaa".to_string(), empty_crate("aaa"));
    second.crates.insert("zzz".to_string(), empty_crate("zzz"));

    assert_eq!(first.content_hash(), second.content_hash());
}

fn empty_crate(name: &str) -> CrateRecord {
    CrateRecord {
        name: name.to_string(),
        ..Default::default()
    }
}

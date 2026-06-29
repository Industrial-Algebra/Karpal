// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use karpal_verify::ObligationBundle;
use karpal_verify_derive::export_obligations;

struct Additive;

#[export_obligations(
    crate_name = "example",
    item_path = "Additive",
    carrier = "Int",
    semigroup(op = "combine"),
    monoid(op = "combine", identity = "empty")
)]
impl Additive {}

#[test]
fn macro_exports_bundle() {
    let bundle: ObligationBundle = Additive::karpal_obligation_bundle();
    assert_eq!(bundle.name, "Additive");
    assert_eq!(bundle.origin.crate_name, "example");
    assert_eq!(bundle.origin.item_path, "Additive");
    assert_eq!(bundle.obligations().len(), 3);
    assert_eq!(bundle.obligations()[0].name, "associativity");
}

struct GroupLike;

#[export_obligations(
    crate_name = "example",
    item_path = "GroupLike",
    carrier = "Int",
    group(op = "combine", identity = "empty", inverse = "invert")
)]
impl GroupLike {}

#[test]
fn macro_exports_group_bundle() {
    let bundle = GroupLike::karpal_obligation_bundle();
    let names = bundle
        .obligations()
        .iter()
        .map(|obligation| obligation.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(bundle.obligations().len(), 5);
    assert!(names.contains(&"left_inverse"));
    assert!(names.contains(&"right_inverse"));
}

struct SemiringLike;

#[export_obligations(
    crate_name = "example",
    item_path = "SemiringLike",
    carrier = "Int",
    semiring(add = "add", zero = "zero", mul = "mul", one = "one")
)]
impl SemiringLike {}

#[test]
fn macro_exports_semiring_bundle() {
    let bundle = SemiringLike::karpal_obligation_bundle();
    let names = bundle
        .obligations()
        .iter()
        .map(|obligation| obligation.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(bundle.obligations().len(), 9);
    assert!(names.contains(&"left_distributivity"));
    assert!(names.contains(&"right_distributivity"));
}

struct RingLike;

#[export_obligations(
    crate_name = "example",
    item_path = "RingLike",
    carrier = "Int",
    ring(add = "add", zero = "zero", neg = "neg", mul = "mul", one = "one")
)]
impl RingLike {}

#[test]
fn macro_exports_ring_bundle() {
    let bundle = RingLike::karpal_obligation_bundle();
    let names = bundle
        .obligations()
        .iter()
        .map(|obligation| obligation.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(bundle.obligations().len(), 11);
    assert!(names.contains(&"add_left_inverse"));
    assert!(names.contains(&"add_right_inverse"));
}

struct LatticeLike;

#[export_obligations(
    crate_name = "example",
    item_path = "LatticeLike",
    carrier = "Bool",
    lattice(meet = "meet", join = "join")
)]
impl LatticeLike {}

#[test]
fn macro_exports_lattice_bundle() {
    let bundle = LatticeLike::karpal_obligation_bundle();
    let names = bundle
        .obligations()
        .iter()
        .map(|obligation| obligation.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(bundle.obligations().len(), 7);
    assert!(names.contains(&"meet_associativity"));
    assert!(names.contains(&"absorption"));
}

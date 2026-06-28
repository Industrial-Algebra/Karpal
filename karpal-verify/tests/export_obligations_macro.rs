// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use karpal_verify::{ObligationBundle, export_obligations};

struct AdditiveViaReexport;

#[export_obligations(
    crate_name = "example",
    item_path = "AdditiveViaReexport",
    carrier = "Int",
    monoid(op = "combine", identity = "empty")
)]
impl AdditiveViaReexport {}

#[test]
fn reexported_macro_exports_bundle() {
    let bundle: ObligationBundle = AdditiveViaReexport::karpal_obligation_bundle();
    assert_eq!(bundle.obligations().len(), 3);
    assert_eq!(bundle.origin.crate_name, "example");
}

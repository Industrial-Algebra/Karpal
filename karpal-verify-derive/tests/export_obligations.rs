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

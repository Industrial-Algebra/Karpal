use karpal_diagram::Diagram;
use karpal_diagram::coherence::{ByNormalization, equivalent_proved};
use karpal_proof::rewrite::Rewrite;

struct MarkerA;
struct MarkerB;

#[test]
fn equivalent_proved_returns_witness_when_diagrams_normalize_equally() {
    let a = Diagram::swap(1, 2).then(Diagram::swap(2, 1));
    let b = Diagram::identity(3);

    let witness: Rewrite<MarkerA, MarkerB, ByNormalization> =
        equivalent_proved::<MarkerA, MarkerB>(&a, &b).expect("should prove equivalence");

    // The witness is a ZST — its existence is the proof
    let _ = witness;
}

#[test]
fn equivalent_proved_returns_none_when_diagrams_differ() {
    let a = Diagram::box_("f", 1, 1);
    let b = Diagram::box_("g", 1, 1);

    assert!(equivalent_proved::<MarkerA, MarkerB>(&a, &b).is_none());
}

#[test]
fn equivalent_proved_handles_yanking_equivalence() {
    let a = Diagram::identity(1);
    let yanked = Diagram::cup(1)
        .parallel(Diagram::identity(1))
        .then(Diagram::identity(1).parallel(Diagram::cap(1)));

    let witness: Rewrite<MarkerA, MarkerB, ByNormalization> =
        equivalent_proved::<MarkerA, MarkerB>(&a, &yanked.normalize())
            .expect("normalized yanking equals identity");

    let _ = witness;
}

#[test]
fn equivalent_proved_handles_associativity_equivalence() {
    let a = Diagram::box_("f", 1, 1)
        .then(Diagram::box_("g", 1, 1))
        .then(Diagram::box_("h", 1, 1));
    let b = Diagram::box_("f", 1, 1).then(Diagram::box_("g", 1, 1).then(Diagram::box_("h", 1, 1)));

    let witness: Rewrite<MarkerA, MarkerB, ByNormalization> =
        equivalent_proved::<MarkerA, MarkerB>(&a, &b).expect("associativity should normalize");

    let _ = witness;
}

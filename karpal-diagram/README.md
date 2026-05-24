# karpal-diagram

Monoidal categories and string diagrams for the Karpal ecosystem.

`karpal-diagram` begins Phase 13 of the Karpal roadmap with:

- monoidal category traits: `Tensor`, `Braiding`, `Symmetry`, `Trace`
- coherence law witnesses: `PentagonIdentity`, `TriangleIdentity`, `HexagonIdentity`
- diagrammatic rewriting: `ByNormalization`, `ByYanking`, `equivalent_proved()`, `prove_yanking()`
- verification integration: `CoherenceCertificate`, `coherence_certificates()`
- a small string-diagram DSL
- compact-closed cup/cap nodes with basic yanking normalization
- text and SVG rendering helpers
- diagram normalization for simple equivalence checking
- normalization tracing for rewrite/debug visibility

## Example

```rust
use karpal_arrow::{Arrow, FnA};
use karpal_diagram::{Braiding, Diagram, Tensor, Trace};

let double = FnA::arr(|x: i32| x * 2);
let increment = FnA::arr(|x: i32| x + 1);
let parallel = FnA::tensor(double, increment);
assert_eq!(parallel((3, 4)), (6, 5));

let swap = FnA::braid::<i32, bool>();
assert_eq!(swap((7, true)), (true, 7));

let traced = FnA::trace::<i32, i32, i32>(FnA::arr(|(input, feedback)| {
    (input + feedback, feedback + 1)
}));
assert_eq!(traced(7), 7);

let diagram = Diagram::box_("double", 1, 1)
    .parallel(Diagram::box_("increment", 1, 1))
    .then(Diagram::swap(1, 1));
assert!(diagram.render_text().contains("swap[1|1]"));

let trace = Diagram::identity(1)
    .then(Diagram::box_("double", 1, 1))
    .normalize_with_trace();
assert!(trace.applied(karpal_diagram::NormalizationRule::ElideIdentitySequenceStage));
assert!(diagram.render_normalization_trace().contains("normalization trace"));

let yanking = Diagram::cup(1)
    .parallel(Diagram::identity(1))
    .then(Diagram::identity(1).parallel(Diagram::cap(1)));
let yanking_trace = yanking.normalize_with_trace();
assert_eq!(yanking_trace.normalized, Diagram::identity(1));
assert!(yanking_trace.applied(karpal_diagram::NormalizationRule::YankCupCap));

// Type-level coherence witness
use karpal_diagram::coherence::verify_pentagon;
use karpal_proof::rewrite::Rewrite;
let _proof: Rewrite<(((i32, u8), bool), String), (i32, (u8, (bool, String))), _> =
    verify_pentagon::<i32, u8, bool, String>();

// Verification certificates for all coherence laws
use karpal_diagram::coherence::coherence_certificates;
let certs = coherence_certificates();
assert_eq!(certs.len(), 3); // pentagon, triangle, hexagon
```

## License

AGPL-3.0-or-later

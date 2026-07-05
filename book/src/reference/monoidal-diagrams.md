# Monoidal Diagrams

String diagrams, monoidal categories, coherence witnesses, and verification — `karpal-diagram` (Phase 13).


## Monoidal Category Traits

`karpal-diagram` provides four monoidal category traits built on `karpal-arrow`:

| Trait      | Super-trait | Key method                                            |
|------------|-------------|-------------------------------------------------------|
| `Tensor`   | `Arrow`     | `tensor(left, right)`, associator, left/right unitors |
| `Braiding` | `Tensor`    | `braid<A,B>()` — swap tensor factors                  |
| `Symmetry` | `Braiding`  | `braid ∘ braid = id`                                  |
| `Trace`    | `Tensor`    | `trace(morphism)` — close a feedback wire             |

``` rust
use karpal_arrow::FnA;
use karpal_diagram::{Braiding, Tensor, Trace};

// Tensor product
let parallel = FnA::tensor(
    FnA::arr(|x: i32| x * 2),
    FnA::arr(|x: i32| x + 1),
);
assert_eq!(parallel((3, 4)), (6, 5));

// Braiding
let swap = FnA::braid::();
assert_eq!(swap((7, true)), (true, 7));

// Trace (close feedback)
let traced = FnA::trace::(FnA::arr(|(a, d)| (a + d, d)));
assert_eq!(traced(7), 7);
```


## String Diagram DSL

`Diagram` is a runtime string-diagram representation with these node kinds:

- `Identity` — arity-n identity wire
- `Box { label }` — labelled morphism
- `Sequence(a, b)` — vertical composition (`a.then(b)`)
- `Parallel(a, b)` — horizontal composition (`a.parallel(b)`)
- `Swap { left, right }` — braiding node
- `Cup { arity }` — compact-closed unit (I → A\* ⊗ A)
- `Cap { arity }` — compact-closed counit (A ⊗ A\* → I)

``` rust
use karpal_diagram::Diagram;

let circuit = Diagram::box_("f", 1, 1)
    .parallel(Diagram::box_("g", 1, 1))
    .then(Diagram::swap(1, 1))
    .then(Diagram::box_("h", 2, 2));

// Text rendering
println!("{}", circuit.render_text());

// SVG rendering
println!("{}", circuit.render_svg());
```


## Diagram Normalization

Diagrams normalize to a canonical form using these rewrite rules:

| Rule                         | Effect                                  |
|------------------------------|-----------------------------------------|
| `FlattenSequence`            | Flatten nested `Sequence` nodes         |
| `FlattenParallel`            | Flatten nested `Parallel` nodes         |
| `ElideIdentitySequenceStage` | Remove identity in sequence             |
| `CollapseIdentityParallel`   | Collapse all-identity parallel branches |
| `CancelAdjacentSwaps`        | swap(A,B) ; swap(B,A) → id              |
| `YankCupCap`                 | (cup ⊗ id) ; (id ⊗ cap) → id            |

``` rust
let yanked = Diagram::cup(1)
    .parallel(Diagram::identity(1))
    .then(Diagram::identity(1).parallel(Diagram::cap(1)));

let trace = yanked.normalize_with_trace();
assert_eq!(trace.normalized, Diagram::identity(1));
assert!(trace.applied(NormalizationRule::YankCupCap));

// Equivalence checking via normalization
let a = Diagram::swap(1, 2).then(Diagram::swap(2, 1));
assert!(a.equivalent_to(&Diagram::identity(3)));
```


## Type-Level Coherence Witnesses

Monoidal coherence laws are encoded as `karpal-proof::Justifies` witnesses:

| Witness            | Law                                   |
|--------------------|---------------------------------------|
| `PentagonIdentity` | (α⊗id) ; α ; (id⊗α) = α ; α           |
| `TriangleIdentity` | ρ⊗id = α ; (id⊗λ)                     |
| `HexagonIdentity`  | braid ; α⁻¹ ; braid ; α⁻¹ = α ; braid |

``` rust
use karpal_diagram::coherence::verify_hexagon;
use karpal_proof::rewrite::Rewrite;

let _proof: Rewrite<((i32, u8), bool), ((u8, bool), i32), _> =
    verify_hexagon::();
```


## Diagrammatic Rewriting Bridge

Runtime diagram normalization connects to type-level proofs via `ByNormalization` and `ByYanking`:

``` rust
use karpal_diagram::coherence::{equivalent_proved, prove_yanking, ByYanking};
use karpal_proof::rewrite::Rewrite;

// Prove equivalence via normalization
let a = Diagram::swap(1, 2).then(Diagram::swap(2, 1));
let witness: Rewrite<_, _, _> =
    equivalent_proved::<(), ()>(&a, &Diagram::identity(3)).unwrap();

// Prove yanking
let yank_proof: Rewrite<_, _, ByYanking> = prove_yanking::<(), ()>(2);
```


## Verification Integration

Coherence certificates connect to `karpal-verify`:

``` rust
use karpal_diagram::coherence::coherence_certificates;

let certs = coherence_certificates();
assert_eq!(certs.len(), 3); // pentagon, triangle, hexagon
for cert in &certs {
    assert_eq!(cert.backend, "karpal-diagram-coherence");
}
```



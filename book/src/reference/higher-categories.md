# Higher Categories

2-categories, enriched categories, bicategories, FFunctor/FMonad — `karpal-higher` (Phase 15).


## TwoCategory

A strict 2-category has objects, 1-morphisms between objects, and 2-morphisms between parallel 1-morphisms:

``` rust
use karpal_higher::{TwoCategory, Cat};

// Cat: objects = types, 1-morphisms = Box, 2-morphisms = ()
let id = Cat::id1::();
assert_eq!(id(42), 42);

let f: Box i32> = Box::new(|x| x + 1);
let g: Box i32> = Box::new(|x| x * 2);
let gf = Cat::compose1(f, g);
assert_eq!(gf(5), 12);
```


## Bicategory

A bicategory weakens associativity and unitality to isomorphism, with an associator and left/right unitors:

``` rust
use karpal_higher::{Bicategory, Cat};

// Associator: (f ∘ g) ∘ h ≅ f ∘ (g ∘ h)
let _alpha = Cat::associator::();

// Left unitor: id ∘ f ≅ f
let _lambda = Cat::left_unitor::();

// Right unitor: f ∘ id ≅ f
let _rho = Cat::right_unitor::();
```


## EnrichedCategory

Categories enriched over a monoidal base V, where hom-objects carry algebraic structure:

``` rust
use karpal_higher::{EnrichedCategory, SetCategory, SetEnrichment};

// Enriched over Set: ordinary category
let id = SetCategory::id::();
assert_eq!(id(42), 42);

let f: Box i32> = Box::new(|x| x + 1);
let g: Box i32> = Box::new(|x| x * 2);
let gf = SetCategory::compose(f, g);
assert_eq!(gf(5), 12);
```


## FFunctor / FMonad

Functors between 2-categories and monads in the endofunctor 2-category:

``` rust
use karpal_higher::{FFunctor, IdentityFFunctor, TwoCategory};

// Identity FFunctor preserves 1-morphisms and 2-morphisms
let m = IdentityFFunctor::<Cat>::map_morphism::<i32, i32>(Cat::id1());
```


## Coherence Witnesses

Type-level witnesses for bicategory coherence laws via `karpal-proof::Justifies`:

| Witness                      | Law                                           |
|------------------------------|-----------------------------------------------|
| `InterchangeIdentity`        | `(α ∘ᵥ β) ∘ₕ (γ ∘ᵥ δ) = (α ∘ₕ γ) ∘ᵥ (β ∘ₕ δ)` |
| `BicategoryPentagonIdentity` | Associator pentagon coherence                 |
| `BicategoryTriangleIdentity` | Unitor-triangle coherence                     |

``` rust
use karpal_higher::verify_interchange;
let _proof = verify_interchange();
```


## Verification Integration

Coherence certificates connect to `karpal-verify`:

``` rust
use karpal_higher::higher_coherence_certificates;

let certs = higher_coherence_certificates();
assert_eq!(certs.len(), 3); // interchange, pentagon, triangle
for cert in &certs {
    assert_eq!(cert.backend, "karpal-higher-coherence");
}
```



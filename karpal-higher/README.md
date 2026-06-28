# karpal-higher

2-categories, enriched categories, and bicategories for the Industrial Algebra ecosystem.

`karpal-higher` implements Phase 15 of the Karpal roadmap:

- **TwoCategory** — strict 2-categories with objects, 1-morphisms, and 2-morphisms
- **Bicategory** — weakened 2-categories with associator and unitors as isomorphisms
- **EnrichedCategory** — categories enriched over a monoidal base (Set, Monoid)
- **FFunctor / FMonad** — functors and monads at the 2-categorical level
- **Coherence witnesses** — interchange, pentagon, and triangle identities as type-level proofs
- **Verification integration** — certificates for coherence laws via karpal-verify

## Example

```rust
use karpal_higher::{TwoCategory, Cat};

// Cat: the strict 2-category of categories
let id: Box<dyn Fn(i32) -> i32> = Cat::id1();
assert_eq!(id(42), 42);

let f: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
let g: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
let gf = Cat::compose1(f, g);
assert_eq!(gf(5), 12); // (5+1)*2

// Coherence witnesses
use karpal_higher::verify_interchange;
let _proof = verify_interchange();

// Verification certificates
use karpal_higher::higher_coherence_certificates;
let certs = higher_coherence_certificates();
assert_eq!(certs.len(), 3); // interchange, pentagon, triangle
```

## License

AGPL-3.0-or-later

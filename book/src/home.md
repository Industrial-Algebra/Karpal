# Karpal

> Higher-Kinded Types and algebraic structures for Rust

[Get Started](./getting-started.md) | [Browse Reference](./reference/functor-family.md) | [GitHub](https://github.com/Industrial-Algebra/Karpal)

---

## Features

### Type-Safe Abstractions

HKT encoding via GATs lets you write functions generic over `Option`, `Result`, `Vec`, and any container that supports mapping, sequencing, or folding.

### Complete Hierarchy

Functor through Monad, Alt through Alternative, Foldable, Traversable, Comonad, and contravariant duals — all with property-based law verification.

### Profunctor Optics

Lens, Prism, and composition powered by the profunctor hierarchy. Build reusable, first-class field accessors and pattern matchers.

### Ergonomic Macros

`do_!` flattens nested `.and_then()` chains. `ado_!` combines independent computations. Both work with any Monad or Applicative.

### Proof & Verification

`karpal-proof` provides law witnesses and refinement types, while `karpal-verify` exports obligations to SMT and Lean with explicit trust boundaries.

## Quick Example

Flatten nested error handling with `do_!`:

```rust
use karpal_std::prelude::*;

// Without do_! — rightward drift with every step
fn process(input: &str) -> Option<String> {
    parse_id(input).and_then(|id| {
        lookup_user(id).and_then(|user| {
            check_permissions(&user).and_then(|role| {
                Some(format!("{} logged in as {:?}", user.name, role))
            })
        })
    })
}

// With do_! — reads top-to-bottom
fn process(input: &str) -> Option<String> {
    do_! { OptionF;
        id = parse_id(input);
        user = lookup_user(id);
        role = check_permissions(&user);
        Some(format!("{} logged in as {:?}", user.name, role))
    }
}
```

## Workspace

| Crate               | Description                                                                               |
|---------------------|-------------------------------------------------------------------------------------------|
| `karpal-core`       | HKT encoding, functor hierarchy, Semigroup, Monoid, macros                                |
| `karpal-profunctor` | Profunctor, Strong, Choice, FnP                                                           |
| `karpal-optics`     | Profunctor optics: Lens, Prism, composition                                               |
| `karpal-arrow`      | Arrow hierarchy: Category, Arrow, ArrowChoice, Kleisli, Cokleisli                         |
| `karpal-free`       | Free constructions: Coyoneda, Yoneda, Free Monad, Cofree Comonad                          |
| `karpal-recursion`  | Recursion schemes: Fix, cata, ana, hylo, para, histo, chrono                              |
| `karpal-algebra`    | Abstract algebra: Group, Ring, Field, Lattice, Module, VectorSpace                        |
| `karpal-effect`     | Monad transformers and static-bound functor hierarchy                                     |
| `karpal-proof`      | Law witnesses, refinement types, rewrite evidence, and derive-based law checks            |
| `karpal-verify`     | External verification bridge: obligations, exporters, runners, reporting, and trust model |
| `karpal-diagram`    | Monoidal categories and string diagrams                                                   |
| `karpal-schubert-types` | Schubert intersection types                                                           |
| `karpal-higher`     | 2-categories, enriched categories, bicategories                                           |
| `karpal-std`        | Standard prelude re-exports                                                               |

`karpal-core`, `karpal-profunctor`, `karpal-arrow`, `karpal-free`, `karpal-recursion`, `karpal-algebra`, `karpal-effect`, `karpal-proof`, and the modeling/export portions of `karpal-verify` are `no_std` compatible with optional `std`/`alloc` feature gates.

## Documentation Map

| Need                                                | Where to start                                                                |
|-----------------------------------------------------|-------------------------------------------------------------------------------|
| Core HKT and trait hierarchy                        | [Getting Started](./getting-started.md) and [Architecture](./architecture-full.md) |
| Detailed typeclass APIs                             | [Reference pages](./reference/functor-family.md)                              |
| `karpal-proof` law witnesses and refinement types   | [Proof & Verification](./reference/proof-verification.md)                     |
| `karpal-verify` exporters, runners, and trust model | [Proof & Verification](./reference/proof-verification.md)                     |
| CI artifact/report workflow                         | [Verification CI Workflow](./reference/verification-ci.md)                    |
| Serialized artifact schemas and compatibility       | [Verification Schemas](./reference/verification-schemas.md)                   |
| End-to-end verification walkthrough                 | [Verification Workflow](./examples/verification-workflow.md)                  |
| Importing verified evidence into domain APIs        | [Verified Domain API](./examples/verified-domain-api.md)                      |

---

Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).

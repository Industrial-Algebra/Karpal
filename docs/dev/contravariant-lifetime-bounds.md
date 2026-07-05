# Contravariant Lifetime Bounds: Why `'static` and What We Can Do About It

**Status:** Resolved — lifetime-aware variant added
**Date:** 2026-07-01
**Issue:** [#93](https://github.com/Industrial-Algebra/Karpal/issues/93)
**Surfaced by:** Proserpina documentation critique (Batch 3)

## Background

The contravariant and profunctor type families in Karpal require `'static`
lifetime bounds on their type parameters. This breaks the categorical duality
with the covariant hierarchy (Functor → Monad), which works with arbitrary
lifetimes.

The issue manifests as: a predicate `Box<dyn Fn(&str) -> bool>` cannot be
`contramap`'d because `A = &str` does not satisfy `A: 'static`.

## Root Cause

The `'static` bound originates from `Box<dyn Fn>` trait objects.

When the `HKT` trait encodes `type Of<T> = Box<dyn Fn(T) -> bool>`, the `dyn`
trait object has an implicit `'static` lifetime:

```rust
// PredicateF::Of<A> expands to:
Box<dyn Fn(A) -> bool + 'static>
//                     ^^^^^^^^ implicit
```

For this type to be valid, `A` must satisfy `A: 'static`. The `dyn` trait
object cannot reference data with a shorter lifetime — this is a fundamental
property of Rust's type system, not a design choice.

### Verification

```rust
// This COMPILES (A = i32: 'static):
let pred: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
PredicateF::contramap(pred, |s: &str| s.len() as i32);  // ✅

// This DOES NOT COMPILE (A = &str is not 'static):
let pred: Box<dyn Fn(&str) -> bool> = Box::new(|s| !s.is_empty());
PredicateF::contramap(pred, |len: usize| ...);
// error[E0310]: the parameter type `A` may not live long enough
```

## Asymmetry with the Covariant Hierarchy

The covariant `Functor` trait has NO `'static` bounds:

```rust
pub trait Functor: HKT {
    fn fmap<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> B) -> Self::Of<B>;
}
```

This works because canonical covariant types (`Option<T>`, `Vec<T>`,
`Result<T, E>`) are **owned containers** — they don't use `dyn` trait objects.
`Option<&str>` is perfectly valid.

However, covariant types that DO use `Box<dyn Fn>` (like `StoreF` and
`TracedF` comonads) also can't implement `Functor`. The codebase documents
this:

```rust
// Note: StoreF and TracedF cannot implement the generic Functor trait because
// Box<dyn Fn> requires 'static bounds that the trait signature doesn't allow.
// They get their own fmap via the Extend/Comonad implementation.
```

So the asymmetry is:
- **Covariant**: no `'static` on trait → `Box<dyn Fn>` types excluded
- **Contravariant**: `'static` on trait → `Box<dyn Fn>` types included, but borrowed data excluded

Both tradeoffs are real. The contravariant side chose `'static` because ALL
natural contravariant types use `Box<dyn Fn>` (predicates, comparators,
serializers are all function types).

## Solution: Lifetime-Parameterized HKT

Rust's GATs support lifetime parameters:

```rust
pub trait HKTLt {
    type Of<'a, T: 'a>;
}
```

This allows the associated type to carry a lifetime, enabling:

```rust
impl HKTLt for PredicateFLt {
    type Of<'a, T: 'a> = Box<dyn Fn(T) -> bool + 'a>;
    //                                                ^^ parameterized!
}
```

The lifetime-aware `ContravariantLt` trait:

```rust
pub trait ContravariantLt: HKTLt {
    fn contramap<'a, A: 'a, B: 'a>(
        fa: Self::Of<'a, A>,
        f: impl Fn(B) -> A + 'a,
    ) -> Self::Of<'a, B>;
}
```

This removes the `'static` requirement entirely. Predicates on borrowed
data now work:

```rust
let len_check: Box<dyn Fn(i32) -> bool> = Box::new(|n| n > 3);
let string_pred = PredicateFLt::contramap(len_check, |s: String| s.len() as i32);
assert!(string_pred(String::from("hello")));
```

## Trade-off Matrix

| Encoding | `'static` required | Borrowed data | API complexity |
|----------|-------------------|---------------|----------------|
| `HKT { type Of<T> }` | Yes | No | Simple |
| `HKTLt { type Of<'a, T> }` | No | Yes | Slightly more complex |

Both are provided:
- [`Contravariant`](../karpal-core/src/contravariant.rs) — simple API, `'static` only
- [`ContravariantLt`](../karpal-core/src/contravariant_lt.rs) — lifetime-aware, supports borrowed data

## Remaining HRTB Limitation

The lifetime-aware variant resolves the `'static` issue but still faces a
separate Rust limitation: higher-ranked trait bounds (HRTB) mismatch.

When `contramap` produces `Box<dyn Fn(B) -> bool + 'a>` where `B` contains
references (e.g., `B = (&str, i32)`), the resulting trait object uses a
specific lifetime `'a` rather than the HRTB `for<'x>`. This causes a type
mismatch when the consumer expects `dyn for<'x> Fn(...)`.

This is a general Rust type system limitation with HRTBs and GATs, not
specific to Karpal. It affects cases where the function argument type itself
contains references. For owned argument types, the lifetime-aware variant
works perfectly.

## Broader Significance

This limitation applies to the entire contravariant and profunctor hierarchy
(`Divisible`, `Decide`, `Profunctor`, `Strong`, `Choice`, `Traversing`).
Each could have a lifetime-aware counterpart using `HKTLt`. For now, only
`ContravariantLt` is provided as a proof of concept. The pattern can be
extended to the full hierarchy if demand materializes.

This is the same class of limitation as the FreeAp `fold_map` investigation:
Rust's type system cannot fully express the category-theoretic abstractions
that higher-ranked polymorphism enables in Haskell.

## References

- [Issue #93](https://github.com/Industrial-Algebra/Karpal/issues/93) — original report
- [Proserpina critique report](../reviews/batch3-optics-review.md) — Batch 3, finding B4
- `karpal-core/src/contravariant.rs` — the `'static`-bounded implementation
- `karpal-core/src/contravariant_lt.rs` — the lifetime-aware implementation
- [FreeAp fold_map exploration](./freeap-fold-map-exploration.md) — related GAT limitation

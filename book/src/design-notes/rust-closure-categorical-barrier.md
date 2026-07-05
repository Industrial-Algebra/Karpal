# Rust Closure Traits as a Category-Theoretic Barrier

**Status:** Design rationale — open problem  
**Date:** 2026-07-01  
**Related:** Issues [#95](https://github.com/Industrial-Algebra/Karpal/issues/95), [#98](https://github.com/Industrial-Algebra/Karpal/issues/98)  
**Investigations:** [FreeAp fold_map](./freeap-fold-map-exploration.md), [ReaderTF/StateTF ApplicativeSt](#98)

## Summary

Rust's closure trait hierarchy (`FnOnce` → `FnMut` → `Fn`) encodes a
semantic distinction that has no counterpart in category theory: *how many
times a computation may be invoked*. Most categorical constructs are defined
in a lazy setting (Haskell, or mathematical Set) where a value `A → B` can
be consumed repeatedly without cost. Rust's strict-call-by-value semantics,
combined with ownership, make this distinction a **first-class type-system
barrier** that blocks several natural category-theoretic encodings.

This document identifies the precise mechanism, catalogues the blocked
constructions, and outlines what future language features might provide
an escape hatch.

## The Mechanism

### FnOnce vs Fn: The Ownership Barrier

In category theory, a function `f: A → B` is a value. It can be applied
zero, one, or many times. The notion of "how many times" does not exist.

In Rust, a closure literal `|x| body` is classified by how its captured
environment is used:

| Trait | Captures by | Callable | Signature |
|-------|------------|----------|-----------|
| `FnOnce` | Value (consuming) | Once | `call_once(self, args)` |
| `FnMut` | Mutable reference | Many | `call_mut(&mut self, args)` |
| `Fn` | Immutable reference | Many | `call(&self, args)` |

The key friction: when a closure captures an owned value `a: A` by move,
it is `FnOnce` — the value is consumed on invocation and cannot be
reproduced. For the closure to be `Fn`, `a` must be `Clone`, so that
`a.clone()` can produce a fresh value on each invocation.

This means:

```
move |x| consume(a, x)  ⟹  FnOnce  (captures a by move, consumes it)
move |x| a.clone()      ⟹  Fn      (captures a by move, clones it)
```

### Why This Matters Categorically

Many category-theoretic constructions involve *producing a value of type
`G::Of<A>`* inside a context where the value may need to be produced
multiple times. In Haskell (lazy), producing a value has no cost — it's
a thunk. In Rust (strict), producing a value consumes its ingredients,
and reproducing it requires either cloning or sharing via `Rc`/`Arc`.

## Constructions Blocked

### 1. FreeAp `fold_map`: Natural Transformations Through Existentials

The canonical signature:

```haskell
foldMap :: Applicative g => (forall x. f x -> g x) -> FreeAp f a -> g a
```

Rust cannot express this because:
1. `(forall x. f x -> g x)` is a **rank-2 polymorphic** natural transformation
2. The intermediate type `B` in `Ap (f b) (...)` is **existentially quantified**
3. Dispatching a rank-2 function through an existential type requires
   monomorphization — which `dyn Trait` cannot provide
4. Even if monomorphization worked, the natural transformation must be
   callable *at every node in the tree* (multiple invocations), but each
   invocation consumes the effect value — making it `FnOnce` when the
   recursive walk needs `Fn`

**Categorical gap:** `foldMap` works in Haskell because:
- `forall x` is first-class (System F)
- Existential types are first-class
- Lazy evaluation makes the number of invocations irrelevant

Rust lacks all three. The four alternative encodings we explored (generic
node trait, Church encoding, recursive monomorphic, list-based) each fail
for different sub-reasons of the same fundamental barrier.

### 2. ReaderTF/StateTF `ApplicativeSt`: Pure in a Closure Context

The problem:

```rust
// ReaderTF::Of<A> = Box<dyn Fn(E) -> M::Of<A>>

impl ApplicativeSt for ReaderTF<E, M> {
    fn pure_st<A: 'static>(a: A) -> Box<dyn Fn(E) -> M::Of<A>> {
        // We must produce M::Of<A> on EVERY invocation of the closure.
        // But M::pure_st(a) consumes a. After the first call, a is gone.
        //
        // Without A: Clone, the closure can only be FnOnce.
    }
}
```

**Categorical gap:** In category theory, `pure: a → Reader e a` is a
natural transformation. The resulting `Reader e a` is a value. Applying
it to different environments is free — it's just function application.
In Rust, the "result" IS the function (a `Box<dyn Fn>`), and every
invocation must produce a *fresh* `M::Of<A>`. Since `pure` consumes `a`,
only the first invocation succeeds.

The same applies to `StateTF`: `pure(a)(s) = M::pure_st((s, a))` consumes
`a`, making the closure `FnOnce`.

### 3. Other Latent Issues

The same pattern would appear in:

- **ContT (continuation monad transformer):** `pure_st(a) = |k| k(a)` —
  `a` must be cloneable for `k` to be called multiple times with the same
  continuation
- **Any free construction with generic interpreters:** The interpreter
  must be applied at every node, consuming contextual data each time
- **Cofree comonad with function-valued tails:** Similar closure capture
  issues when extracting repeatedly

## The Common Thread

All these failures share a structural property:

```
Categorical context:     "produce G<A> once, return it"
Rust representation:     "return a Fn closure that produces G<A> on each call"
Conflict:                Fn requires reproducibility, but production consumes.
```

This maps onto the categorical coalgebra/algebra distinction:
- **Coalgebraically:** `A → G<A>` (produce once, observe/consume)
- **Algebraically:** `(E → G<A>)` (produce on demand, potentially many times)

Rust can express the coalgebraic version (`FnOnce`), but not the algebraic
version (`Fn`) without `Clone`. Most category-theoretic constructs
implicitly assume algebraic access (values can be observed repeatedly).

## Why This Is Not Fixable in Current Rust

The Rust project has explored several avenues that would help, but none
are close to stabilization:

| Feature | Status | Would it help? |
|---------|--------|---------------|
| `impl for<X> Fn(X) -> Y` (rank-N closures) | Not proposed | Would help FreeAp, but not Fn/FnOnce |
| Existential types (`exists X. ...`) | Not in roadmap | Would help FreeAp |
| `FnOnce` → `Fn` upcast (with Clone) | Not in trait system | Would bridge some cases |
| Lazy evaluation / thunks | Rejected | Would solve the root cause |
| `for<'a>` in trait objects (lifetime-bounded dyn) | Partially available via GATs | Already used in ContravariantLt (#93) |

## Escape Hatches We Explored and Used

1. **Lifetime-parameterized GATs** (`type Of<'a, T>`): Used for
   ContravariantLt (#93). Solves the `'static` constraint but not
   the `Fn`/`FnOnce` issue.

2. **Blanket impls** (`impl<F: Functor> FunctorSt for F`): Used for
   the St hierarchy (#97). Bridges parallel typeclass families but
   doesn't solve the underlying representation problem.

3. **Standalone functions with stronger bounds:** Used for
   `reader_t_pure`/`state_t_pure` (#98). The functions require
   `Clone` because they clone on each invocation. The trait
   doesn't require it because of the ripple effects (#98
   investigation). This is a pragmatic compromise.

4. **Iterative convergence instead of knot-tying:** Used for
   `loop_fixpoint` (#94). Replaces Haskell's lazy `loop` with
   an iterative fixpoint that terminates when the feedback
   stabilizes.

5. **Honest removal:** `OptionF` as Comonad (#92) and `CokleisliF<OptionF>`
   (#92). Removed instances that mathematically cannot exist because
   totality is violated.

## If an Alternative Encoding Emerges

The document exists so that when Rust's type system evolves — whether
through HKT, rank-N closures, first-class existentials, or a `for<X>`
quantifier — we can revisit these constructions. The specific things to
watch for:

### Future Feature: `for<X>` Quantified Closures

```rust
// Hypothetical: rank-N closure through trait objects
trait NatTrans<F: HKT, G: HKT> {
    fn transform<X>(fx: F::Of<X>) -> G::Of<X>
    where for<X>  // NOTE: hypothetical syntax
}

// Then fold_map could be:
impl<F: HKT, A> FreeAp<F, A> {
    fn fold_map<G: Applicative>(
        self,
        nt: &dyn for<X> Fn(F::Of<X>) -> G::Of<X>,  // hypothetical
    ) -> G::Of<A>
}
```

This would require Rust to support rank-N closures through `dyn` — a
significant extension to the trait system.

### Future Feature: First-Class Existentials

```rust
// Hypothetical: existential type
enum FreeAp<F: HKT, A> {
    Pure(A),
    Ap(exists B. F::Of<B>, FreeAp<F, B -> A>),  // hypothetical
}
```

This would allow the existential type `B` to be recovered at pattern-match
time, enabling monomorphization of `fold_map`.

### Future Feature: Lazy/Thunk Evaluation

```rust
// Hypothetical: GAT-based lazy value
trait Lazy {
    type Of<'a, T> = ???;  // some thunk-like representation
}
```

If Rust gained lazy evaluation primitives, the `Fn`/`FnOnce` distinction
would become irrelevant for many categorical constructs — producing a
value would not consume it, and `pure` would be `Fn` by default.

## Relationship to Other Karpal Design Documents

- [FreeAp fold_map exploration](./freeap-fold-map-exploration.md) —
  detailed investigation of four alternative encodings
- [Contravariant lifetime bounds](https://github.com/Industrial-Algebra/Karpal/blob/develop/docs/dev/contravariant-lifetime-bounds.md) —
  the `'static`/`Box<dyn Fn>` limitation and lifetime-aware GAT workaround

## References

- Rust closure trait hierarchy: [`FnOnce`](https://doc.rust-lang.org/std/ops/trait.FnOnce.html),
  [`FnMut`](https://doc.rust-lang.org/std/ops/trait.FnMut.html),
  [`Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html)
- GAT stabilization: [RFC 1598](https://rust-lang.github.io/rfcs/1598-generic_associated_types.html)
- `for<>` lifetime syntax: [Reference](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds)

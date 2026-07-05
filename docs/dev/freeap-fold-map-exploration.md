# FreeAp `fold_map`: Why It's Impossible in Rust (And What We Can Do Instead)

**Status:** Design rationale — closed investigation
**Date:** 2026-07-01
**Issue:** [#95](https://github.com/Industrial-Algebra/Karpal/issues/95)
**Surfaced by:** Proserpina documentation critique (Batch 4)

## Background

The Free Applicative (`FreeAp`) is a standard construction in category theory
and functional programming. In Haskell, its signature is:

```haskell
data FreeAp f a
  = Pure a
  | forall b. Ap (f b) (FreeAp f (b -> a))

foldMap :: Applicative g => (forall x. f x -> g x) -> FreeAp f a -> g a
```

The `forall b` in the `Ap` constructor is an **existential type** — each node
hides its own intermediate type `b`. The `forall x` in `foldMap` is a
**rank-2 universal** — the natural transformation must work for *all* types.

Together, these two quantifiers create a tension that is easy to express in
Haskell (which has full System F polymorphism) but — as this investigation
demonstrates — **fundamentally impossible to express in current Rust**.

## The Problem

Karpal's `FreeAp` uses GAT-based HKT encoding with an existential `dyn` trait:

```rust
trait FreeApNode<F: HKT + 'static, A: 'static> {
    fn retract_node(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative;
    fn count_effects(&self) -> usize;
}

pub enum FreeAp<F: HKT + 'static, A: 'static> {
    Pure(A),
    Ap(Box<dyn FreeApNode<F, A>>),
}
```

Each `Ap` node erases its intermediate type `B` behind `dyn FreeApNode<F, A>`.
This works for `retract` (interpret into `F` itself) because `F::ap` and
`F::pure` are available at the trait level.

But `fold_map` needs to call `nt.transform::<B>()` where `B` is erased.
Rust cannot monomorphize a generic function through a `dyn` trait object.
**Generic methods and `dyn` dispatch are mutually exclusive in Rust's type
system.**

## Approaches Explored

### Approach 1: Generic method on the node trait

**Idea:** Add `fold_map` as a generic method to `FreeApNode`.

```rust
trait FreeApNode<F: HKT + 'static, A: 'static> {
    fn fold_map<G: Applicative, NT: NatTrans<F, G>>(&self, nt: &NT) -> G::Of<A>;
}

trait NatTrans<F: HKT, G: HKT> {
    fn transform<B>(fb: F::Of<B>) -> G::Of<B>;
}
```

**Result: ❌ Does not compile.**

`fold_map<G, NT>` is a generic method. Generic methods make a trait
**non-dyn-compatible** (`dyn FreeApNode` won't compile). Without `dyn`, we
cannot erase intermediate types, which means we cannot build heterogeneous
trees.

This is the circular trap:
- To erase `B`, we need `dyn`.
- To dispatch `nt<B>`, we need monomorphization.
- `dyn` and generic methods are mutually exclusive.

### Approach 2: Church encoding with erased interpreter

**Idea:** Represent `FreeAp` as its own fold — a closure that takes an
interpreter and produces the result.

```rust
trait ErasedInterp<F> {
    fn pure_erased(&self, val: Box<dyn Any>) -> Box<dyn Any>;
    fn ap_erased(&self, ff: Box<dyn Any>, fa: Box<dyn Any>) -> Box<dyn Any>;
    fn lift_erased(&self, fb: Box<dyn Any>) -> Box<dyn Any>;
}

pub struct FreeApC<F: 'static, A: 'static> {
    run: Box<dyn FnOnce(&dyn ErasedInterp<F>) -> Box<dyn Any>>,
}
```

**Result: ❌ Does not compile.**

The interpreter's `pure_erased` receives `Box<dyn Any>` and must construct
`G::Of<A>`. But `A` is erased at runtime — the interpreter has no way to
recover the concrete type from `Box<dyn Any>` to call `G::pure`.

Similarly, `ap_erased` receives two erased boxes but cannot determine the
function/argument types to perform the applicative application.

**Root cause:** `Box<dyn Any>` erases types that the interpreter needs to
construct correctly-typed results. Type erasure is incompatible with type
construction.

### Approach 3: Recursive monomorphic encoding

**Idea:** Constrain all effects to the same type `X`, avoiding the need for
existentials.

```rust
pub enum FreeApMono<F: HKT + 'static, X: 'static, A: 'static> {
    Pure(A),
    Ap {
        effect: F::Of<X>,
        kont: Box<FreeApMono<F, X, Box<dyn Fn(X) -> A>>>,
    },
}
```

**Result: ❌ Does not compile — infinite type recursion.**

The `Ap` variant stores `FreeApMono<F, X, Box<dyn Fn(X) -> A>>`, which creates
an infinite type chain at the type checker level:

```
FreeApMono<F, X, A>
  contains FreeApMono<F, X, Box<dyn Fn(X) -> A>>
    contains FreeApMono<F, X, Box<dyn Fn(X) -> Box<dyn Fn(X) -> A>>>
      contains FreeApMono<F, X, Box<dyn Fn(X) -> Box<dyn Fn(X) -> Box<dyn Fn(X) -> A>>>>
        contains ... (infinite)
```

Compiler error:
```
error[E0320]: overflow while adding drop-check rules for `FreeApMono<F, X, A>`
  = note: overflowed on `FreeApMono<F, X, Box<dyn Fn(X) -> Box<dyn Fn(X) -> Box<...>>>>`
```

The recursive applicative structure (`Ap(F<B>, FreeAp<F, B->A>)`) **must** be
broken with existentials. There is no way to represent a recursive applicative
tree without either existentials (which block `fold_map`) or infinite types.

### Approach 4: Non-recursive list-based encoding

**Idea:** Flatten effects to a `Vec<F::Of<X>>` and combine with a pure
function. Since applicatives have independent effects (unlike monads), this
is semantically valid for the monomorphic case.

```rust
pub struct FreeApSeq<F: HKT + 'static, X: 'static, A: 'static> {
    effects: Vec<F::Of<X>>,
    combine: Box<dyn Fn(&[X]) -> A>,
}
```

`fold_map` would: (1) apply NT to each effect, (2) sequence the `G` effects
via `Applicative`, (3) map `combine` over the result.

**Result: ⚠️ Theoretically viable, but hits ownership friction.**

The `sequence` operation (turn `Vec<G::Of<X>>` into `G::Of<Vec<X>>`) requires
threading an accumulator through `G::ap`. The accumulator closure captures a
`Vec<X>` by move, making it `FnOnce` — but `G::ap` requires `Box<dyn Fn>`,
which must be callable multiple times.

This can be worked around with `Rc<RefCell<Vec<X>>>`, but that introduces
runtime overhead and complexity, violating Karpal's zero-cost principle.

**Additional trade-offs:**
- Only supports monomorphic effects (all `F::Of<X>`)
- Loses the applicative composition structure (flat list, not a tree)
- The `combine: Fn(&[X]) -> A` interface is awkward (positional, not curried)

## The Fundamental Barrier

All four approaches fail for the same underlying reason:

> **Rust's type system cannot express rank-N polymorphism (`forall x. f x -> g x`)
> dispatched through existential types (`forall b. ...`).**

In type-theoretic terms:
- `fold_map` requires a **negative** occurrence of `forall x` (the natural
  transformation must be polymorphic in `x`)
- The `Ap` constructor requires a **positive** occurrence of `exists b` (the
  intermediate type is hidden)
- Rust's trait system supports neither rank-N types nor first-class
  existentials — both are approximated via `dyn`, which requires monomorphic
  (non-generic) methods

This is not a bug or an oversight. It is a fundamental property of Rust's
type system as of 2026. The limitation is shared by all GAT-based HKT
encodings in Rust.

## What the Current Encoding Provides

| Capability | Status |
|-----------|--------|
| `retract()` — interpret into `F` itself | ✅ Works |
| `count_effects()` — static analysis of effect tree | ✅ Works |
| `fmap()` — functor map over result | ✅ Works |
| `ap()` — applicative composition | ✅ Works |
| `fold_map()` — interpret into arbitrary `G` via NT | ❌ Impossible (fundamental) |
| Heterogeneous effect types | ✅ Supported |
| Applicative law verification | ✅ 4 proptest laws |

## Recommendation

1. **The current encoding is correct.** It provides the maximum set of
   capabilities possible in Rust's type system.

2. **The documentation already explains the limitation** and provides the
   workaround: `fold_map nt ≡ retract . hoist nt`.

3. **For monomorphic use cases** where `fold_map` is genuinely needed,
   users can build a `Vec<F::Of<X>>` and sequence it directly via their
   target `Applicative`. This is straightforward in application code:

   ```rust
   let effects: Vec<F::Of<X>> = vec![...];
   let g_effects: Vec<G::Of<X>> = effects.into_iter().map(nt).collect();
   let sequenced = G::sequence(g_effects);  // if G: Traversable
   let result = G::fmap(sequenced, combine);
   ```

4. **A `FreeApSeq` type** (Approach 4) could be added as a separate crate
   or module if demand materializes. It would provide `fold_map` for the
   monomorphic case at the cost of generality and zero-cost guarantees.

## Broader Significance

This investigation is relevant beyond Karpal. Any Rust library attempting
to encode category-theoretic abstractions with free constructions will hit
this same wall. The findings here apply to:

- Free monads with generic interpreters
- Church-encoded data types with polymorphic folds
- Any construction requiring rank-N polymorphism through existentials

The documentation of this limitation serves as a reference for the broader
Rust category-theory ecosystem.

## References

- [Issue #95](https://github.com/Industrial-Algebra/Karpal/issues/95) — original report
- [Proserpina critique report](../reviews/batch4-algebra-review.md) — Batch 4, finding B6
- `karpal-free/src/free_ap.rs` — the implementation
- Haskell `Control.Applicative.Free` — the reference implementation that CAN express `foldMap`

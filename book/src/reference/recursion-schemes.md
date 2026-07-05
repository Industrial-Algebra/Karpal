# Recursion Schemes

Recursion schemes provide structured, composable patterns for folding and unfolding recursive data. They live in the `karpal-recursion` crate and depend on `karpal-core` (HKT, Functor) and `karpal-free` (Cofree, Free).

## Overview

| Type / Function | Category             | Key idea                                                     |
|-----------------|----------------------|--------------------------------------------------------------|
| `Fix<F>`        | Fixed point          | Ties the recursive knot: `Fix<F> ≅ F<Fix<F>>`                |
| `Mu<F>`         | Least fixed point    | Type alias for `Fix<F>` (Rust can't enforce finiteness)      |
| `Nu<F, Seed>`   | Greatest fixed point | Seed + coalgebra; lazy observation of corecursive structures |
| `cata`          | Fold                 | Catamorphism — fold bottom-up with `F<A> → A`                |
| `ana`           | Unfold               | Anamorphism — unfold top-down with `A → F<A>`                |
| `hylo`          | Refold               | Hylomorphism — unfold then fold, no intermediate `Fix`       |
| `para`          | Fold+                | Paramorphism — fold with access to original subterms         |
| `apo`           | Unfold+              | Apomorphism — unfold with early termination via `Either`     |
| `histo`         | Fold++               | Histomorphism — fold with full history via `Cofree`          |
| `futu`          | Unfold++             | Futumorphism — multi-step unfold via `Free`                  |
| `zygo`          | Composite            | Zygomorphism — fold with auxiliary fold in parallel          |
| `chrono`        | Composite            | Chronomorphism — `futu` ; `histo` in a single pass           |

## Fixed Points


### Fix\<F\>

The fixed point of a functor. Ties the recursive knot so that `Fix<F> ≅ F<Fix<F>>`.


#### Definition

``` rust
pub struct Fix<F: HKT>(Rc<F::Of<Fix<F>>>);

// Unconditional Clone via Rc reference counting
impl<F: HKT> Clone for Fix<F> { ... }

pub type Mu<F> = Fix<F>;
```

#### Key Methods

``` rust
// Wrap one layer
Fix::new(f: F::Of<Fix<F>>) -> Fix<F>

// Unwrap one layer (consuming)
fix.unfix() -> F::Of<Fix<F>>  // where F::Of<Fix<F>>: Clone

// Borrow one layer
fix.unfix_ref() -> &F::Of<Fix<F>>
```

#### Design: Rc vs Box

`Fix` uses `Rc` instead of `Box` for indirection. This makes `Fix<F>: Clone` unconditional (just a reference count bump), which is essential for paramorphism — it needs to both preserve and consume each subterm. Rust's trait solver cannot prove coinductive Clone bounds like `Fix<OptionF>: Clone ↔ Option<Fix<OptionF>>: Clone`, so `Box` would make `Clone` impossible.

#### Example: Natural Numbers

``` rust
use karpal_recursion::{Fix, cata, ana};
use karpal_core::hkt::OptionF;

// None = Zero, Some(n) = Succ(n)
let three: Fix<OptionF> = Fix::new(Some(Fix::new(Some(Fix::new(None)))));

// Or build with ana:
let five: Fix<OptionF> = ana(
    |n: u32| if n == 0 { None } else { Some(n - 1) },
    5,
);

// Fold with cata:
let count = cata::<OptionF, u32>(
    |layer| match layer {
        None => 0,
        Some(n) => n + 1,
    },
    five,
);
assert_eq!(count, 5);
```


### Nu\<F, Seed\>

Greatest fixed point — a seed paired with a coalgebra for lazy observation.


#### Definition

``` rust
pub struct Nu<F: HKT, Seed> {
    pub seed: Seed,
    pub coalgebra: Box<dyn Fn(&Seed) -> F::Of<Seed>>,
}
```

#### Key Methods

``` rust
Nu::new(seed, coalgebra) -> Nu<F, Seed>
nu.observe() -> F::Of<Seed>   // apply coalgebra once
nu.to_fix() -> Fix<F>         // fully unfold via ana
```

#### Example

``` rust
use karpal_recursion::Nu;
use karpal_core::hkt::OptionF;

let countdown = Nu::<OptionF, u32>::new(3, |&s| {
    if s == 0 { None } else { Some(s - 1) }
});
assert_eq!(countdown.observe(), Some(2));
```


## Recursion Schemes


### cata — Catamorphism

Fold a recursive structure bottom-up. The fundamental "tear down" operation.


#### Signature

``` rust
pub fn cata<F: HKT + Functor, A>(
    alg: impl Fn(F::Of<A>) -> A,
    fix: Fix<F>,
) -> A
```

#### Example: Sum Natural Numbers

``` rust
let n = ana(|s: u32| if s == 0 { None } else { Some(s - 1) }, 5);
let sum = cata::<OptionF, u32>(
    |layer| match layer {
        None => 0,
        Some(acc) => acc + 1,
    },
    n,
);
assert_eq!(sum, 5);
```

#### Laws

- `cata(Fix::new, x) == x` — folding with the constructor is identity


### ana — Anamorphism

Unfold a recursive structure top-down from a seed.


#### Signature

``` rust
pub fn ana<F: HKT + Functor, A>(
    coalg: impl Fn(A) -> F::Of<A>,
    seed: A,
) -> Fix<F>
```

#### Example: Build Natural Numbers

``` rust
let three: Fix<OptionF> = ana(
    |n: u32| if n == 0 { None } else { Some(n - 1) },
    3,
);
```

#### Laws

- `cata(alg, ana(coalg, seed)) == hylo(alg, coalg, seed)`


### hylo — Hylomorphism

Unfold then fold in a single pass — no intermediate `Fix` is allocated.


#### Signature

``` rust
pub fn hylo<F: HKT + Functor, A, B>(
    alg: impl Fn(F::Of<B>) -> B,
    coalg: impl Fn(A) -> F::Of<A>,
    seed: A,
) -> B
```

#### Key Property

`hylo(alg, coalg, seed) == cata(alg, ana(coalg, seed))` but more efficient — the intermediate data structure is "deforested" away.


### para — Paramorphism

Fold with access to original subterms. The algebra receives both the folded result and the original sub-structure.


#### Signature

``` rust
pub fn para<F: HKT + Functor, A>(
    alg: impl Fn(F::Of<(Fix<F>, A)>) -> A,
    fix: Fix<F>,
) -> A
```

#### Example: Factorial

``` rust
let factorial = para::<OptionF, u64>(
    |layer| match layer {
        None => 1,           // 0! = 1
        Some((sub, acc)) => {
            let n = count(&sub) + 1;  // read current number from subterm
            (n as u64) * acc
        }
    },
    nat(5),
);
assert_eq!(factorial, 120);
```

#### Laws

- When ignoring the `Fix<F>` subterm, `para` degenerates to `cata`


### apo — Apomorphism

Unfold with early termination. The coalgebra can short-circuit by injecting a pre-built `Fix`.


#### Signature

``` rust
pub fn apo<F: HKT + Functor, A>(
    coalg: impl Fn(A) -> F::Of<Either<Fix<F>, A>>,
    seed: A,
) -> Fix<F>
```

#### Key Idea

`Either::Right(seed)` continues unfolding, `Either::Left(fix)` embeds an already-built subtree directly. When always returning `Right`, `apo` degenerates to `ana`.


### histo — Histomorphism

Fold with full history via `Cofree`. At each step, access all previously-computed results.


#### Signature

``` rust
pub fn histo<F: HKT + Functor, A>(
    alg: impl Fn(&F::Of<Cofree<F, A>>) -> A,
    fix: Fix<F>,
) -> A
```

#### Example: Fibonacci

``` rust
let fib = histo::<OptionF, u64>(
    |layer| match layer {
        None => 0,                      // fib(0) = 0
        Some(cofree) => {
            let prev = cofree.head;      // fib(n-1)
            match cofree.tail.as_ref() {
                None => 1,               // fib(1) = 1
                Some(gc) => prev + gc.head,  // fib(n-1) + fib(n-2)
            }
        }
    },
    nat(10),
);
assert_eq!(fib, 55);
```

#### Design Note

The algebra takes `&F::Of<Cofree<F, A>>` (a reference) rather than ownership. This avoids needing `Clone` on the recursive `Cofree` structure, which Rust's trait solver cannot prove coinductively.


### futu — Futumorphism

Multi-step unfold via `Free`. Generate multiple layers of structure at once.


#### Signature

``` rust
pub fn futu<F: HKT + Functor, A>(
    coalg: impl Fn(A) -> F::Of<Free<F, A>>,
    seed: A,
) -> Fix<F>
```

#### Key Idea

`Free::Pure(seed)` continues with one more coalgebra application. `Free::Roll(f)` injects multiple layers at once. When always returning `Pure`, `futu` degenerates to `ana`.


### zygo — Zygomorphism

Fold with an auxiliary fold running in parallel.


#### Signature

``` rust
pub fn zygo<F: HKT + Functor, A, B>(
    aux: impl Fn(F::Of<B>) -> B,
    alg: impl Fn(F::Of<(B, A)>) -> A,
    fix: Fix<F>,
) -> A
where F::Of<(B, A)>: Clone
```

#### Key Idea

Two algebras run simultaneously: `aux` computes a helper value `B` at each layer, and `alg` has access to both `B` and the primary result `A` from sub-structures.


### chrono — Chronomorphism

Combines futumorphism and histomorphism in a single pass.


#### Signature

``` rust
pub fn chrono<F: HKT + Functor, A, B>(
    alg: impl Fn(&F::Of<Cofree<F, B>>) -> B,
    coalg: impl Fn(A) -> F::Of<Free<F, A>>,
    seed: A,
) -> B
```

#### Key Idea

Conceptually `chrono = histo . futu` — multi-step unfolding with history-aware folding — but computed in a single pass without building an intermediate `Fix`.


## Either\<L, R\>


### Either\<L, R\>

A simple sum type used by apomorphism for early termination.


#### Definition

``` rust
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

// Methods: either, map_left, map_right
```


## Scheme Relationships

The schemes form a lattice of generality:

``` text
        cata ────────── hylo ────────── ana
         │                                │
    para (+ subterms)              apo (+ early stop)
         │                                │
   histo (+ full history)         futu (+ multi-step)
         │                                │
         └──────── chrono ────────────────┘
                     │
                zygo (+ aux fold)
```

Each scheme on the left is the dual of the corresponding scheme on the right. Moving down adds more power (and constraints). `hylo` sits in the middle as the deforested composition of any fold and unfold.

## Implementation Patterns

- **`&dyn Fn` recursion** — all inner helper functions use `&dyn Fn` for the algebra/coalgebra to break monomorphization recursion (same pattern as `Free::fmap_inner`)
- **`Rc` in Fix** — enables `Clone` without coinductive proofs; essential for `para`
- **Reference algebras** — `histo` and `chrono` take `&F::Of<Cofree<F, A>>` to avoid cloning Cofree
- **`F::Of<Fix<F>>: Clone`** — required by schemes that call `unfix()` (cata, para, histo, zygo) for the `Rc::try_unwrap` fallback path

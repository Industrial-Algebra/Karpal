# Arrow Family

Arrows generalize functions to computations with structured inputs and outputs. They extend the [Profunctor](profunctor-family.md) idea -- a two-parameter type constructor with composition -- into a full algebra of composable pipelines with product routing, sum routing, application, looping, and failure.

The arrow family lives in the `karpal-arrow` crate and builds on the `HKT2` encoding from `karpal-core`.

## Trait Hierarchy

``` rust
HKT2
 +-> Semigroupoid          compose(f, g)
     +-> Category           id()
         +-> Arrow           arr(f), first, second, split, fanout
              |-> ArrowChoice    left, right, splat, fanin
              |-> ArrowApply     app  (~ Monad)
              |-> ArrowLoop      loop_arrow  (D: Default)
              +-> ArrowZero      zero_arrow
                   +-> ArrowPlus  plus(f, g)
```

- **Semigroupoid** -- composable morphisms (associative composition).
- **Category** -- adds an identity morphism.
- **Arrow** -- lifts pure functions and routes through products (tuples).
- **ArrowChoice** -- routes through sum types (`Result`).
- **ArrowApply** -- first-class arrow application, equivalent in power to Monad.
- **ArrowLoop** -- feedback/fixpoint combinator using `D: Default` for strict evaluation.
- **ArrowZero** -- a failing/empty arrow.
- **ArrowPlus** -- associative choice between arrows.

## Traits


### Semigroupoid

Morphisms that can be composed associatively.


#### Signature

``` rust
/// Semigroupoid: morphisms that can be composed.
///
/// Laws:
/// - Associativity: compose(f, compose(g, h)) == compose(compose(f, g), h)
pub trait Semigroupoid: HKT2 {
    fn compose<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Self::P<B, C>,
        g: Self::P<A, B>,
    ) -> Self::P<A, C>;
}
```

`compose` chains two morphisms: given `g: A -> B` and `f: B -> C`, produce `f . g: A -> C`. Note the argument order -- `f` comes first, matching mathematical convention (`f` after `g`).

#### Laws


Associativity

Composition is associative:

``` rust
P::compose(f, P::compose(g, h)) == P::compose(P::compose(f, g), h)
```


### Category

A Semigroupoid with an identity morphism.


#### Signature

``` rust
/// Category: a Semigroupoid with an identity morphism.
///
/// Laws:
/// - Left identity:  compose(id(), f) == f
/// - Right identity: compose(f, id()) == f
pub trait Category: Semigroupoid {
    fn id<A: Clone + 'static>() -> Self::P<A, A>;
}
```

`id` produces the identity morphism that, when composed with any other morphism, yields that morphism unchanged.

#### Laws


Left Identity

``` rust
P::compose(P::id(), f) == f
```


Right Identity

``` rust
P::compose(f, P::id()) == f
```


### Arrow

A Category that can lift pure functions and operate on products (tuples).


#### Signature

``` rust
/// Arrow: a Category that can lift pure functions and operate on products.
///
/// Laws:
/// - arr(id) == id()
/// - arr(|a| g(f(a))) == compose(arr(g), arr(f))
/// - first(arr(f)) == arr(|(a, c)| (f(a), c))
/// - first(compose(f, g)) == compose(first(f), first(g))
pub trait Arrow: Category {
    /// Lift a pure function into an arrow.
    fn arr<A: Clone + 'static, B: Clone + 'static>(
        f: impl Fn(A) -> B + 'static,
    ) -> Self::P<A, B>;

    /// Apply an arrow to the first component of a pair, passing the second through.
    fn first<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<(A, C), (B, C)>;

    /// Apply an arrow to the second component of a pair.
    fn second<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<(C, A), (C, B)> { ... }

    /// `***`: apply two arrows in parallel on a product.
    fn split<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static, D: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<C, D>,
    ) -> Self::P<(A, C), (B, D)> { ... }

    /// `&&&`: feed input to two arrows and collect results as a pair.
    fn fanout<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<A, C>,
    ) -> Self::P<A, (B, C)> { ... }
}
```

`arr` lifts any pure function into the arrow. `first` applies an arrow to the first component of a tuple, passing the second through unchanged. `second`, `split`, and `fanout` have default implementations built from `first`, `compose`, and `arr`.

#### Laws


arr preserves identity

``` rust
P::arr(|a| a) == P::id()
```


arr preserves composition

``` rust
P::arr(|a| g(f(a))) == P::compose(P::arr(g), P::arr(f))
```


first/arr coherence

``` rust
P::first(P::arr(f)) == P::arr(|(a, c)| (f(a), c))
```


first distributes over compose

``` rust
P::first(P::compose(f, g)) == P::compose(P::first(f), P::first(g))
```


#### Derived Operations

- `second(pab)` -- default implementation swaps the tuple components, applies `first`, and swaps back.
- `split(f, g)` -- Haskell's `***` operator. Applies `f` to the first component and `g` to the second: `compose(second(g), first(f))`.
- `fanout(f, g)` -- Haskell's `&&&` operator. Duplicates the input and applies `f` and `g` in parallel: `compose(split(f, g), arr(|a| (a.clone(), a)))`.


### ArrowChoice

An Arrow that can route through sum types (`Result`).


#### Signature

``` rust
/// ArrowChoice: an Arrow that can route through sum types.
///
/// Uses `Result<L, R>` as the sum type, consistent with karpal-profunctor's Choice.
///
/// Laws:
/// - left(arr(f)) == arr(|r| r.map(f))
/// - left(compose(f, g)) == compose(left(f), left(g))
/// - compose(arr(Ok), f) == compose(left(f), arr(Ok))
pub trait ArrowChoice: Arrow {
    /// Route the Ok branch through the arrow, passing Err through.
    fn left<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<Result<A, C>, Result<B, C>>;

    /// Route the Err branch through the arrow, passing Ok through.
    fn right<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<Result<C, A>, Result<C, B>> { ... }

    /// `+++`: apply f on Ok, g on Err.
    fn splat<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static, D: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<C, D>,
    ) -> Self::P<Result<A, C>, Result<B, D>> { ... }

    /// `|||`: merge two arrows, one for each branch of Result.
    fn fanin<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Self::P<A, C>,
        g: Self::P<B, C>,
    ) -> Self::P<Result<A, B>, C> { ... }
}
```

`left` routes the `Ok` branch through the arrow, passing the `Err` branch through unchanged. `right`, `splat`, and `fanin` have default implementations. Karpal uses `Result<L, R>` as the sum type, consistent with [Choice](profunctor-family.md#choice) in `karpal-profunctor`.

#### Laws


left/arr coherence

``` rust
P::left(P::arr(f)) == P::arr(|r| r.map(f))
```


left distributes over compose

``` rust
P::left(P::compose(f, g)) == P::compose(P::left(f), P::left(g))
```


#### Derived Operations

- `right(pab)` -- mirrors the `Result`, applies `left`, and mirrors back.
- `splat(f, g)` -- Haskell's `+++` operator. Applies `f` to `Ok` and `g` to `Err`: `compose(right(g), left(f))`.
- `fanin(f, g)` -- Haskell's `|||` operator. Merges both branches into a single output: `compose(merge, splat(f, g))`.


### ArrowApply

An Arrow that can apply arrows from within the computation. Equivalent in power to Monad.


#### Signature

``` rust
/// ArrowApply: an Arrow that can apply arrows from within the computation.
///
/// Equivalent in power to Monad (ArrowApply ~ Monad via Kleisli).
pub trait ArrowApply: Arrow {
    fn app<A: Clone + 'static, B: Clone + 'static>()
        -> Self::P<(Self::P<A, B>, A), B>;
}
```

`app` takes a pair of an arrow and an input value, and applies the arrow to the value. This gives arrows the ability to choose which arrow to run at runtime, making `ArrowApply` equivalent in power to `Monad` (via the Kleisli correspondence).


### ArrowLoop

An Arrow with a loop/fixpoint combinator.


#### Signature

``` rust
/// ArrowLoop: an Arrow with a loop/fixpoint combinator.
///
/// Takes an arrow from `(A, D)` to `(B, D)` and produces an arrow from `A` to `B`,
/// where `D` is the "feedback" type threaded through the loop.
///
/// In Haskell, `loop` relies on laziness to tie the knot. Rust is strict, so
/// `D: Default` provides the initial feedback seed and the implementation uses
/// single-pass evaluation.
pub trait ArrowLoop: Arrow {
    fn loop_arrow<A: Clone + 'static, B: Clone + 'static, D: Default + Clone + 'static>(
        f: Self::P<(A, D), (B, D)>,
    ) -> Self::P<A, B>;
}
```

`loop_arrow` takes an arrow from `(A, D)` to `(B, D)` and produces an arrow from `A` to `B`, where `D` is the feedback type. In Haskell, `loop` relies on laziness to tie the knot. Since Rust is strict, the `D: Default` bound provides the initial feedback seed and the implementation uses single-pass evaluation.


### ArrowZero

An Arrow with a zero (failing/empty) morphism.


#### Signature

``` rust
/// ArrowZero: an Arrow with a zero (failing/empty) morphism.
///
/// Laws:
/// - compose(zero_arrow(), f) == zero_arrow()  (left absorption)
pub trait ArrowZero: Arrow {
    fn zero_arrow<A: Clone + 'static, B: Clone + 'static>() -> Self::P<A, B>;
}
```

`zero_arrow` produces an arrow that always fails or returns empty. It absorbs any composition from the left: composing anything after a `zero_arrow` still yields `zero_arrow`.

#### Laws


Left Absorption

``` rust
P::compose(P::zero_arrow(), f) == P::zero_arrow()
```


### ArrowPlus

An ArrowZero with an associative choice operation.


#### Signature

``` rust
/// ArrowPlus: an ArrowZero with an associative choice operation.
///
/// Laws:
/// - Associativity: plus(plus(f, g), h) == plus(f, plus(g, h))
/// - Left identity:  plus(zero_arrow(), f) == f
/// - Right identity: plus(f, zero_arrow()) == f
pub trait ArrowPlus: ArrowZero {
    fn plus<A: Clone + 'static, B: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<A, B>,
    ) -> Self::P<A, B>;
}
```

`plus` combines two arrows: if the first succeeds, use its result; otherwise fall back to the second. Together with `zero_arrow`, this forms a monoid over arrows.

#### Laws


Associativity

``` rust
P::plus(P::plus(f, g), h) == P::plus(f, P::plus(g, h))
```


Left Identity

``` rust
P::plus(P::zero_arrow(), f) == f
```


Right Identity

``` rust
P::plus(f, P::zero_arrow()) == f
```


## Concrete Implementations


### FnA (Function Arrow)

Marker type whose `P<A, B>` is `Box<dyn Fn(A) -> B>` -- the canonical function arrow.


#### Definition

``` rust
pub struct FnA;

impl HKT2 for FnA {
    type P<A, B> = Box<dyn Fn(A) -> B>;
}
```

`FnA` is a zero-sized marker type equivalent to `FnP` in `karpal-profunctor` but independent (no cross-crate dependency). It is the most natural arrow: a boxed function from `A` to `B`.

#### Implemented traits

| Trait          | Method          | Implementation                                                         |
|----------------|-----------------|------------------------------------------------------------------------|
| `Semigroupoid` | `compose(f, g)` | `Box::new(move |a| f(g(a)))`                                           |
| `Category`     | `id()`          | `Box::new(|a| a)`                                                      |
| `Arrow`        | `arr(f)`        | `Box::new(f)`                                                          |
| `Arrow`        | `first(pab)`    | `Box::new(move |(a, c)| (pab(a), c))`                                  |
| `Arrow`        | `second(pab)`   | `Box::new(move |(c, a)| (c, pab(a)))`                                  |
| `ArrowChoice`  | `left(pab)`     | `Box::new(move |r| match r { Ok(a) => Ok(pab(a)), Err(c) => Err(c) })` |
| `ArrowChoice`  | `right(pab)`    | `Box::new(move |r| match r { Ok(c) => Ok(c), Err(a) => Err(pab(a)) })` |
| `ArrowApply`   | `app()`         | `Box::new(|(f, a)| f(a))`                                              |
| `ArrowLoop`    | `loop_arrow(f)` | `Box::new(move |a| { let (b, _) = f((a, D::default())); b })`          |

`FnA` does **not** implement `ArrowZero` or `ArrowPlus` because a plain function `A -> B` has no notion of failure or empty result.

#### Feature gate

`FnA` requires the `alloc` feature because `Box<dyn Fn>` requires heap allocation.

#### Example

``` rust
use karpal_arrow::{Arrow, ArrowChoice, Category, Semigroupoid, FnA};

// Lift pure functions into arrows
let double = FnA::arr(|x: i32| x * 2);
let add_one = FnA::arr(|x: i32| x + 1);

// Compose: (x * 2) then (+ 1)
let pipeline = FnA::compose(add_one, double);
assert_eq!(pipeline(5), 11); // (5 * 2) + 1

// fanout: feed the same input to two arrows
let double = FnA::arr(|x: i32| x * 2);
let negate = FnA::arr(|x: i32| -x);
let both = FnA::fanout(double, negate);
assert_eq!(both(5), (10, -5));

// split: apply two arrows in parallel on a tuple
let to_str = FnA::arr(|x: i32| x.to_string());
let double = FnA::arr(|x: i32| x * 2);
let par = FnA::split(to_str, double);
assert_eq!(par((42, 5)), ("42".to_string(), 10));

// ArrowChoice: route through Result branches
let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
let routed = FnA::left::<i32, i32, &str>(double);
assert_eq!(routed(Ok(5)), Ok(10));
assert_eq!(routed(Err("nope")), Err("nope"));
```


### KleisliF\<M\> (Kleisli Arrow)

Kleisli arrow for a Monad `M`: `P<A, B> = Box<dyn Fn(A) -> M::Of<B>>`.


#### Definition

``` rust
pub struct KleisliF<M: HKT>(PhantomData<M>);

impl<M: HKT> HKT2 for KleisliF<M> {
    type P<A, B> = Box<dyn Fn(A) -> M::Of<B>>;
}
```

`KleisliF<M>` wraps effectful functions `A -> M<B>` as arrows. It implements the full Arrow hierarchy when `M: Chain + Applicative + Functor`, and additionally implements `ArrowZero` and `ArrowPlus` when `M: Plus`.

#### Implemented traits

| Trait          | Constraint on `M`               | Key operation                                 |
|----------------|---------------------------------|-----------------------------------------------|
| `Semigroupoid` | `Chain + Applicative`           | `compose(f, g) = |a| M::chain(g(a), &f)`      |
| `Category`     | `Chain + Applicative`           | `id() = |a| M::pure(a)`                       |
| `Arrow`        | `Chain + Applicative + Functor` | `arr(f) = |a| M::pure(f(a))`                  |
| `ArrowChoice`  | `Chain + Applicative + Functor` | `left(pab) = |Ok(a)| M::fmap(pab(a), Ok)`     |
| `ArrowApply`   | `Chain + Applicative + Functor` | `app() = |(f, a)| f(a)`                       |
| `ArrowZero`    | `+ Plus`                        | `zero_arrow() = |_| M::zero()`                |
| `ArrowPlus`    | `+ Plus`                        | `plus(f, g) = |a| M::alt(f(a.clone()), g(a))` |

#### Example

``` rust
use karpal_arrow::{Arrow, ArrowZero, ArrowPlus, Semigroupoid, KleisliF};
use karpal_core::hkt::OptionF;

type KOpt = KleisliF<OptionF>;

// Kleisli arrows: functions returning Option
let safe_double: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x * 2));
let safe_add_one: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x + 1));

// Compose: chains through the monad (short-circuits on None)
let pipeline = KOpt::compose(safe_add_one, safe_double);
assert_eq!(pipeline(5), Some(11)); // Some(10) -> Some(11)

// Short-circuit on failure
let fail: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|_| None);
let after_fail = KOpt::compose(safe_add_one, fail);
assert_eq!(after_fail(5), None);

// ArrowZero: an arrow that always returns None
let z = KOpt::zero_arrow::<i32, i32>();
assert_eq!(z(42), None);

// ArrowPlus: try the first arrow, fall back to the second
let attempt = KOpt::plus(fail, safe_double);
assert_eq!(attempt(5), Some(10)); // first fails, second succeeds
```


### CokleisliF\<W\> (Cokleisli Arrow)

Cokleisli arrow for a Comonad `W`: `P<A, B> = Box<dyn Fn(W::Of<A>) -> B>`.


#### Definition

``` rust
pub struct CokleisliF<W: HKT>(PhantomData<W>);

impl<W: HKT> HKT2 for CokleisliF<W> {
    type P<A, B> = Box<dyn Fn(W::Of<A>) -> B>;
}
```

`CokleisliF<W>` wraps context-consuming functions `W<A> -> B` as arrows. Composition requires `W::Of<A>: Clone`, which cannot be expressed generically with GATs. Instead, the `impl_cokleisli!` macro generates `Semigroupoid` and `Category` impls for specific comonads.

#### The `impl_cokleisli!` macro

``` rust
/// Generate Semigroupoid + Category impls for `CokleisliF<$W>` where
/// `$W::Of<A>` is known to be `Clone` for `A: Clone`.
///
/// Usage: `impl_cokleisli!(IdentityF, OptionF, NonEmptyVecF, EnvF<E>);`
#[macro_export]
macro_rules! impl_cokleisli {
    ($W:ty) => { /* generates Semigroupoid + Category impls */ };
}
```

Pre-generated instances are provided for `IdentityF`, `OptionF`, and `NonEmptyVecF`. A separate `impl_cokleisli_env!` macro handles `EnvF<E>` with specific environment types (pre-generated for `i32` and `String`).

#### Implemented traits

| Trait          | Key operation                                               |
|----------------|-------------------------------------------------------------|
| `Semigroupoid` | `compose(f, g) = |wa| f(W::extend(wa, |wa| g(wa.clone())))` |
| `Category`     | `id() = |wa| W::extract(&wa)`                               |

`CokleisliF` only implements `Semigroupoid` and `Category`. It does not implement the full `Arrow` hierarchy because lifting a pure function `A -> B` into `W<A> -> B` would require `Comonad::extract` in `arr`, but the product-routing operations (`first`, `second`) would need `W<(A, C)>` to be decomposable, which is not generally possible for all comonads.


## Design Notes

### `Clone + 'static` bounds

All type parameters in the arrow hierarchy require `Clone + 'static` bounds. The `'static` bound is necessary because all concrete implementations use `Box<dyn Fn>`, which requires captured closures (and the types they close over) to be `'static`. The `Clone` bound is needed for operations like `fanout` (which duplicates the input) and Kleisli composition (which needs to clone values threaded through monadic binds).

### ArrowLoop and `D: Default`

In Haskell, `ArrowLoop`'s `loop` relies on laziness to "tie the knot" -- the feedback value `d` is defined in terms of itself. Rust is strict, so this is not possible. Instead, Karpal's `loop_arrow` requires `D: Default` to provide an initial seed for the feedback channel. The implementation performs a single pass: it feeds `(a, D::default())` into the arrow and returns the `B` component of the output, discarding the feedback.

### CokleisliF and `impl_cokleisli!`

The challenge with `CokleisliF<W>` is that composition requires `W::Of<A>: Clone`, but this bound cannot be expressed generically with GATs -- you cannot write `where W::Of<A>: Clone` in a blanket impl because the compiler cannot verify this for all possible `W`. The `impl_cokleisli!` macro solves this by generating implementations for each specific comonad where the `Clone` bound is known to hold.

### Operator naming

Haskell uses symbolic operators for several arrow combinators. Since Rust does not support custom operators, Karpal uses descriptive names:

| Haskell operator | Karpal name | Description                                        |
|------------------|-------------|----------------------------------------------------|
| `>>>`            | `compose`   | Sequential composition (reversed argument order)   |
| `***`            | `split`     | Apply two arrows in parallel on a product          |
| `&&&`            | `fanout`    | Feed one input to two arrows, collect results      |
| `+++`            | `splat`     | Apply two arrows to the two branches of a sum      |
| `|||`            | `fanin`     | Merge two arrows from sum branches into one output |


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



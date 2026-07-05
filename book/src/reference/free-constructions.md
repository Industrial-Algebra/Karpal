# Free Constructions

Free constructions generate algebraic structure "for free" from a type constructor. They live in the `karpal-free` crate and build on the HKT encoding and typeclass hierarchy from `karpal-core`.

## Overview

| Type                 | What it gives you       | Key idea                                                                 |
|----------------------|-------------------------|--------------------------------------------------------------------------|
| `Coyoneda<F, A, B>`  | Free Functor            | `fmap` without `F: Functor`, deferred until `lower()`                    |
| `Yoneda<F, A>`       | Map fusion              | O(1) map composition via CPS; `lift` requires `F: Functor`               |
| `Free<F, A>`         | Free Monad              | Build monadic programs as data, interpret with `fold_map`                |
| `Cofree<F, A>`       | Cofree Comonad          | Annotated trees/streams; `F` determines branching shape                  |
| `Freer<F, A>`        | Free Monad (no Functor) | Like `Free` but no `F: Functor` until `fold_map`                         |
| `Lan<G, H, A, B>`    | Left Kan Extension      | Generalises Coyoneda; `fmap` composes extract functions                  |
| `Ran` (trait)        | Right Kan Extension     | CPS form `∀R. (A → G R) → H R`; generalises Codensity                    |
| `Codensity<F, A>`    | CPS Monad               | `pure`/`chain` without bounds; `to_monad` needs `F: Applicative + Chain` |
| `Density<W, A>`      | CPS Comonad             | `extract`/`fmap` without bounds on `W`                                   |
| `Day<F, G, A, B, C>` | Day Convolution         | Pairs two functors with a combining function; interprets via two NTs     |
| `FreeAp<F, A>`       | Free Applicative        | Static analysis of effects before interpretation; `retract` into `F`     |
| `FreeAlt<F, A>`      | Free Alternative        | Choice among applicative branches; `zero`/`alt`/`retract`                |

## Types


### Coyoneda\<F, A, B\>

The free functor -- makes any type constructor into a Functor by deferring `fmap` as function composition.


#### Definition

``` rust
pub struct Coyoneda<F: HKT, A, B> {
    f: Box<dyn Fn(B) -> A>,   // accumulated transform
    fb: F::Of<B>,              // the original value
    _marker: PhantomData<F>,
}

pub struct CoyonedaF<F: HKT>(PhantomData<F>);
```

`Coyoneda<F, A, B>` stores an `F<B>` together with a function `B → A`. The type parameter `B` is the original "base" type from `lift`. Calling `fmap` composes onto the stored function (changing `A` but keeping `B` fixed). Only `lower()` applies the composed function via a single `F::fmap`.

#### Key Methods

``` rust
impl<F: HKT, A: 'static> Coyoneda<F, A, A> {
    /// Lift F<A> into Coyoneda. No Functor bound needed.
    pub fn lift(fa: F::Of<A>) -> Self;
}

impl<F: HKT, A: 'static, B: 'static> Coyoneda<F, A, B> {
    /// Map without Functor bound -- composes onto stored function.
    pub fn fmap<C: 'static>(self, g: impl Fn(A) -> C + 'static) -> Coyoneda<F, C, B>;

    /// Apply the stored function via F::fmap, producing F<A>.
    /// This is the only operation that requires F: Functor.
    pub fn lower(self) -> F::Of<A> where F: Functor;
}
```

#### Example

``` rust
use karpal_free::{Coyoneda, CoyonedaF};
use karpal_core::hkt::OptionF;

// Chain multiple fmaps -- no Functor needed yet
let co = Coyoneda::<OptionF, _, _>::lift(Some(1))
    .fmap(|x| x + 1)
    .fmap(|x| x * 10)
    .fmap(|x| x + 5);

// Only lower() needs Functor -- applies all maps at once
let result = co.lower();
assert_eq!(result, Some(25)); // (1+1)*10+5
```


### Yoneda\<F, A\>

The Yoneda lemma as a data type -- O(1) map composition via CPS.


#### Definition

``` rust
pub struct Yoneda<F: HKT + Functor + 'static, A: 'static> {
    inner: Box<dyn YonedaLower<F, A>>,
}

pub struct YonedaF<F: HKT + Functor + 'static>(PhantomData<F>);
```

`Yoneda<F, A>` is similar to Coyoneda, but `lift` requires `F: Functor`. The benefit is **map fusion**: chaining N `fmap` calls composes the functions before applying them to `F`, so `lower()` makes only one pass through the structure.

#### Key Methods

``` rust
impl<F: HKT + Functor + 'static, A: Clone + 'static> Yoneda<F, A> {
    /// Lift F<A> into Yoneda. Requires F: Functor (unlike Coyoneda).
    pub fn lift(fa: F::Of<A>) -> Self;
}

impl<F: HKT + Functor + 'static, A: 'static> Yoneda<F, A> {
    /// Lower back to F<A> by applying accumulated transformations.
    pub fn lower(self) -> F::Of<A>;

    /// Map a function -- composed into the CPS, deferred until lower().
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> Yoneda<F, B>;
}
```

#### Coyoneda vs Yoneda

|                           | Coyoneda                        | Yoneda                              |
|---------------------------|---------------------------------|-------------------------------------|
| `lift` requires Functor?  | No                              | Yes                                 |
| `fmap` requires Functor?  | No                              | No                                  |
| `lower` requires Functor? | Yes                             | No (already lifted)                 |
| Use case                  | Make non-Functor types mappable | Fuse chains of maps for performance |

#### Example

``` rust
use karpal_free::{Yoneda, YonedaF};
use karpal_core::hkt::OptionF;

// Yoneda fuses multiple fmaps into a single pass
let result = Yoneda::<OptionF, i32>::lift(Some(42))
    .fmap(|x| x * 2)
    .fmap(|x| format!("val={}", x))
    .lower();
assert_eq!(result, Some("val=84".to_string()));
```


### Free\<F, A\>

The Free Monad -- build monadic computations as data structures, then interpret them.


#### Definition

``` rust
/// Pure(a)             -- a finished computation returning a
/// Roll(F<Free<F, A>>) -- one layer of effect wrapping a continuation
pub enum Free<F: HKT, A> {
    Pure(A),
    Roll(Box<F::Of<Free<F, A>>>),
}

/// HKT marker -- implements HKT + Functor
pub struct FreeF<F: HKT>(PhantomData<F>);
```

`Free<F, A>` represents a program where `F` describes the available effects and `A` is the result type. Programs are built with `pure` and `lift_f`, composed with `chain`, and interpreted with `fold_map` using a [natural transformation](bifunctor-natural.md) into any target monad.

#### Key Methods

``` rust
impl<F: HKT, A> Free<F, A> {
    /// Wrap a pure value.
    pub fn pure(a: A) -> Self;
}

impl<F: HKT + Functor, A> Free<F, A> {
    /// Lift a single effect F<A> into the free monad.
    pub fn lift_f(fa: F::Of<A>) -> Self;

    /// Map a function over the result.
    pub fn fmap<B>(self, f: impl Fn(A) -> B) -> Free<F, B>;

    /// Monadic bind -- sequence with a continuation.
    pub fn chain<B>(self, f: impl Fn(A) -> Free<F, B>) -> Free<F, B>;

    /// Interpret into target monad M via natural transformation NT: F ~> M.
    pub fn fold_map<M, NT>(self) -> M::Of<A>
    where
        M: Applicative + Chain,
        NT: NaturalTransformation<F, M>;
}
```

#### Trait Implementations

| Marker     | Trait     | Notes                     |
|------------|-----------|---------------------------|
| `FreeF<F>` | `HKT`     | `Of<T> = Free<F, T>`      |
| `FreeF<F>` | `Functor` | Delegates to `Free::fmap` |

`FreeF` does **not** implement `Apply`, `Chain`, or `Monad`. See [Design Notes](#design-no-monad) for why.

#### Laws


Functor Identity

``` rust
free.fmap(|a| a) == free
```


Functor Composition

``` rust
free.fmap(|a| g(f(a))) == free.fmap(f).fmap(g)
```


Monad Left Identity

``` rust
Free::pure(a).chain(f) == f(a)
```


Monad Right Identity

``` rust
m.chain(Free::pure) == m
```


#### Example

``` rust
use karpal_free::{Free, FreeF};
use karpal_core::hkt::OptionF;
use karpal_core::natural::NaturalTransformation;

// Define a natural transformation: Option ~> Option (identity)
struct OptionId;
impl NaturalTransformation<OptionF, OptionF> for OptionId {
    fn transform<A>(fa: Option<A>) -> Option<A> { fa }
}

// Build a computation: lift an effect, then chain
let program = Free::<OptionF, i32>::lift_f(Some(3))
    .chain(|x| Free::lift_f(Some(x * 10)));

// Interpret into Option via the natural transformation
let result = program.fold_map::<OptionF, OptionId>();
assert_eq!(result, Some(30));
```


### Cofree\<F, A\>

The Cofree Comonad -- annotated trees/streams where `F` determines the branching structure.


#### Definition

``` rust
/// Each node carries a value (head) and subtrees (tail).
/// The choice of F determines the shape:
///   OptionF  -> a non-empty list (finite stream)
///   VecF     -> a rose tree
///   IdentityF -> an infinite stream
pub struct Cofree<F: HKT, A> {
    pub head: A,
    pub tail: Box<F::Of<Cofree<F, A>>>,
}

/// HKT marker -- implements HKT + Functor + Extend + Comonad
pub struct CofreeF<F: HKT>(PhantomData<F>);
```

`Cofree<F, A>` is the dual of the Free Monad. Where Free builds up effects layer by layer, Cofree builds up **context** -- every node in the tree carries a value and can see its entire subtree.

#### Key Methods

``` rust
impl<F: HKT, A> Cofree<F, A> {
    /// Create a node with the given head and tail.
    pub fn new(head: A, tail: F::Of<Cofree<F, A>>) -> Self;

    /// Extract the head value.
    pub fn extract(&self) -> A where A: Clone;
}

impl<F: HKT + Functor, A> Cofree<F, A> {
    /// Map a function over all head values.
    pub fn fmap<B>(self, f: impl Fn(A) -> B) -> Cofree<F, B>;

    /// Apply a context-aware function to every position.
    /// f receives the entire sub-cofree rooted at each node.
    pub fn extend<B>(self, f: impl Fn(&Cofree<F, A>) -> B) -> Cofree<F, B>
    where A: Clone;

    /// Build a Cofree from a seed and an unfolding function.
    /// f(seed) returns (head, F<Seed>) -- the value and seeds for subtrees.
    pub fn unfold<Seed>(seed: Seed, f: impl Fn(&Seed) -> (A, F::Of<Seed>)) -> Self;
}
```

#### Trait Implementations

| Marker       | Trait     | Notes                                    |
|--------------|-----------|------------------------------------------|
| `CofreeF<F>` | `HKT`     | `Of<T> = Cofree<F, T>`                   |
| `CofreeF<F>` | `Functor` | Delegates to `Cofree::fmap`              |
| `CofreeF<F>` | `Extend`  | Context-aware mapping over all positions |
| `CofreeF<F>` | `Comonad` | `extract` returns the head value         |

#### Laws


Comonad: extract after extend

`extract(extend(w, f)) == f(w)`

``` rust
let extended = CofreeF::<OptionF>::extend(w, f);
CofreeF::<OptionF>::extract(&extended) == f(&w)
```


Comonad: extend with extract

`extend(w, extract) == w`

``` rust
CofreeF::<OptionF>::extend(w, CofreeF::<OptionF>::extract).head == w.head
```


#### Example

``` rust
use karpal_free::{Cofree, CofreeF};
use karpal_core::hkt::OptionF;

// Unfold a countdown stream: 3, 2, 1, 0
let stream = Cofree::<OptionF, i32>::unfold(3, |&seed| {
    if seed <= 0 { (seed, None) }
    else { (seed, Some(seed - 1)) }
});
assert_eq!(stream.head, 3);

// Extend: at each position, sum the current and next head
let sums = stream.extend(|w| {
    let next = w.tail.as_ref().as_ref().map(|c| c.head).unwrap_or(0);
    w.head + next
});
assert_eq!(sums.head, 3 + 2); // 5
```


### Freer\<F, A\>

The Freer Monad -- like Free but requires no `F: Functor` until interpretation.


#### Definition

``` rust
/// Pure(a)        -- a finished computation
/// Impure(step)   -- an effect step with erased continuation type
pub enum Freer<F: HKT + 'static, A: 'static> {
    Pure(A),
    Impure(Box<dyn FreerStep<F, A>>),
}

pub struct FreerF<F: HKT + 'static>(PhantomData<F>);
```

`Freer<F, A>` stores computations as a tree of effect steps, each containing `∃B. (F B, B → Freer F A)`. The intermediate type `B` is erased via a dyn-safe trait, deferring the `Functor` requirement to `fold_map`. Use Freer when `F` is not a Functor, or when you want to build computations without that constraint.

#### Key Methods

``` rust
impl<F: HKT + 'static, A: 'static> Freer<F, A> {
    /// Wrap a pure value. No F: Functor required.
    pub fn pure(a: A) -> Self;

    /// Lift a single effect F<A>. No F: Functor required.
    pub fn lift_f(fa: F::Of<A>) -> Self;

    /// Map a function over the result. No F: Functor required.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> Freer<F, B>;

    /// Monadic bind. No F: Functor required.
    pub fn chain<B: 'static>(self, f: impl Fn(A) -> Freer<F, B> + 'static) -> Freer<F, B>;

    /// Interpret into target monad M via natural transformation NT: F ~> M.
    /// This is the only operation requiring F: Functor.
    pub fn fold_map<M, NT>(self) -> M::Of<A>
    where
        F: Functor,
        M: Applicative + Chain,
        NT: NaturalTransformation<F, M>;
}
```

#### Free vs Freer

|                                    | Free                                     | Freer                |
|------------------------------------|------------------------------------------|----------------------|
| `lift_f`/`chain` require Functor?  | Yes                                      | No                   |
| `fold_map` requires Functor?       | Yes                                      | Yes                  |
| Implements `HKT`/`Functor` traits? | Yes (`FreeF`)                            | No (GAT limitation)  |
| Use case                           | `F: Functor` available, want trait impls | `F` is not a Functor |

#### Laws


Monad Left Identity

``` rust
Freer::pure(a).chain(f) == f(a)
```


Monad Right Identity

``` rust
m.chain(Freer::pure) == m
```


#### Example

``` rust
use karpal_free::Freer;
use karpal_core::hkt::OptionF;
use karpal_core::natural::NaturalTransformation;

struct OptionId;
impl NaturalTransformation<OptionF, OptionF> for OptionId {
    fn transform<A>(fa: Option<A>) -> Option<A> { fa }
}

// Build without Functor constraint
let program = Freer::<OptionF, i32>::lift_f(Some(1))
    .chain(|x| Freer::lift_f(Some(x + 1)))
    .chain(|x| Freer::lift_f(Some(x * 10)));

// Functor only needed at interpretation time
let result = program.fold_map::<OptionF, OptionId>();
assert_eq!(result, Some(20)); // (1+1)*10
```


### Lan\<G, H, A, B\>

Left Kan Extension -- `∃B. (G B → A, H B)`.


#### Definition

``` rust
pub struct Lan<G: HKT, H: HKT, A, B> {
    extract_fn: Box<dyn Fn(G::Of<B>) -> A>,
    source: H::Of<B>,
    _marker: PhantomData<G>,
}

pub struct LanF<G: HKT, H: HKT, B>(PhantomData<(G, H, B)>);
```

`Lan<G, H, A, B>` encodes a value `H B` together with a way to extract `A` from `G B`. `fmap` composes onto the extract function with no bounds on G or H. When `G = IdentityF`, Lan is isomorphic to [Coyoneda](#coyoneda).

#### Key Methods

``` rust
impl<G: HKT + 'static, H: HKT + 'static, A: 'static, B: 'static> Lan<G, H, A, B> {
    /// Construct from a source and extract function.
    pub fn new(source: H::Of<B>, f: impl Fn(G::Of<B>) -> A + 'static) -> Self;

    /// Map over the result type. No bounds on G or H required.
    pub fn fmap<C: 'static>(self, f: impl Fn(A) -> C + 'static) -> Lan<G, H, C, B>;

    /// Collapse using a natural transformation NT: H ~> G.
    pub fn lower<NT: NaturalTransformation<H, G>>(self) -> A;
}

/// When G = IdentityF: convert to Coyoneda.
impl<H: HKT + 'static, A: 'static, B: 'static> Lan<IdentityF, H, A, B> {
    pub fn to_coyoneda(self) -> Coyoneda<H, A, B>;
}
```

#### Example

``` rust
use karpal_free::Lan;
use karpal_core::hkt::{IdentityF, OptionF};
use karpal_core::natural::NaturalTransformation;

// Lan<IdentityF, OptionF, String, i32>:
//   source: Option<i32>, extract: i32 -> String
let lan = Lan::<IdentityF, OptionF, String, i32>::new(
    Some(42),
    |x: i32| format!("val={x}"),
);

// Convert to Coyoneda (Lan with G=IdentityF is Coyoneda)
let coy = lan.to_coyoneda();
let result = coy.lower();
assert_eq!(result, Some("val=42".to_string()));
```


### Ran (trait)

Right Kan Extension -- `∀R. (A → G R) → H R`.


#### Definition

``` rust
/// Ran is a trait because the universal quantifier (forall R)
/// requires a generic method, which cannot be made object-safe.
pub trait Ran<G: HKT, H: HKT> {
    /// The input type (A in forall R. (A -> G R) -> H R).
    type Input;

    /// Run: given a continuation k: A -> G R, produce H R.
    fn run_ran<R>(&self, k: impl Fn(Self::Input) -> G::Of<R>) -> H::Of<R>;
}

/// Map over a Ran's input, producing a new Ran.
pub fn ran_fmap<G, H, A, B, T, F>(ran: T, f: F) -> RanMapped<G, H, A, B, T, F>;
```

`Ran` is the dual of [Lan](#lan). Where Lan uses an existential (hidden type), Ran uses a universal (works for all types). In Rust, this means Ran must be a **trait** rather than a concrete type, since trait objects cannot have generic methods. When `G = H = F`, Ran specialises to [Codensity](#codensity).

#### Example

``` rust
use karpal_free::{Ran, ran_fmap};
use karpal_core::hkt::OptionF;

struct SimpleRan(i32);

impl Ran<OptionF, OptionF> for SimpleRan {
    type Input = i32;
    fn run_ran<R>(&self, k: impl Fn(i32) -> Option<R>) -> Option<R> {
        k(self.0)
    }
}

// Basic usage
let result = SimpleRan(42).run_ran(|x| Some(x * 2));
assert_eq!(result, Some(84));

// ran_fmap transforms the input
let mapped = ran_fmap(SimpleRan(10), |x| x + 5);
let result = mapped.run_ran(|x| Some(x * 2));
assert_eq!(result, Some(30)); // (10+5)*2
```


### Codensity\<F, A\>

The Codensity Monad -- CPS transform of a type constructor. `pure`, `fmap`, `chain` need no bounds on `F`.


#### Definition

``` rust
/// Codensity<F, A> = forall R. (A -> F R) -> F R
/// Implemented as a dyn-safe computation tree (Pure/Map/Bind layers).
pub struct Codensity<F: HKT + 'static, A: 'static> {
    inner: Box<dyn CodensityInner<F, A>>,
}

pub struct CodensityF<F: HKT + 'static>(PhantomData<F>);
```

`Codensity<F, A>` is the right Kan extension of `F` along itself (`Ran F F A`), specialised into a concrete type. The key property: `pure`, `fmap`, and `chain` require **no bounds on `F`**. Only `to_monad` needs `F: Applicative + Chain`. Wrapping a free monad in Codensity can improve asymptotic performance of left-associated binds.

#### Key Methods

``` rust
impl<F: HKT + 'static, A: 'static> Codensity<F, A> {
    /// Wrap a pure value. No bounds on F.
    pub fn pure(a: A) -> Self;

    /// Map a function. No bounds on F.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> Codensity<F, B>;

    /// Monadic bind. No bounds on F.
    pub fn chain<B: 'static>(self, f: impl Fn(A) -> Codensity<F, B> + 'static)
        -> Codensity<F, B>;

    /// Collapse to F<A>. Requires F: Applicative + Chain.
    pub fn to_monad(self) -> F::Of<A>
    where F: Applicative + Chain;
}
```

#### Laws


Monad Left Identity

``` rust
Codensity::pure(a).chain(f).to_monad() == f(a).to_monad()
```


Monad Right Identity

``` rust
m.chain(Codensity::pure).to_monad() == m.to_monad()
```


Monad Associativity

``` rust
(m.chain(f)).chain(g).to_monad()
  == m.chain(|x| f(x).chain(g)).to_monad()
```


#### Example

``` rust
use karpal_free::Codensity;
use karpal_core::hkt::OptionF;

// Build computation -- no bounds on OptionF needed here
let computation = Codensity::<OptionF, i32>::pure(1)
    .chain(|x| Codensity::pure(x + 1))
    .chain(|x| Codensity::pure(x * 10))
    .chain(|x| Codensity::pure(x + 5));

// Only to_monad needs F: Applicative + Chain
let result = computation.to_monad();
assert_eq!(result, Some(25)); // (1+1)*10+5
```


### Density\<W, A\>

The Density Comonad -- CPS dual of Codensity. `extract` and `fmap` need no bounds on `W`.


#### Definition

``` rust
/// Density<W, A> = exists S. (W S -> A, W S)
/// The left Kan extension of W along itself (Lan W W A).
pub struct Density<W: HKT + 'static, A: 'static> {
    inner: Box<dyn DensityDyn<W, A>>,
}

pub struct DensityF<W: HKT + 'static>(PhantomData<W>);
```

`Density<W, A>` stores a value `W S` together with a function `&W::Of<S> → A`. The state type `S` is existentially hidden via a dyn-safe trait. Both `extract` and `fmap` require **no bounds on `W`**.

#### Key Methods

``` rust
impl<W: HKT + 'static, A: 'static> Density<W, A> {
    /// Construct from a source and extract function.
    pub fn lift<S: 'static>(
        source: W::Of<S>,
        f: impl Fn(&W::Of<S>) -> A + 'static,
    ) -> Self;

    /// Extract the value. No bounds on W.
    pub fn extract(&self) -> A;

    /// Map a function. No bounds on W.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> Density<W, B>;
}
```

#### Laws


Functor Identity

``` rust
d.fmap(|a| a).extract() == d.extract()
```


Functor Composition

``` rust
d.fmap(|a| g(f(a))).extract() == d.fmap(f).fmap(g).extract()
```


#### Example

``` rust
use karpal_free::Density;
use karpal_core::hkt::OptionF;

// Lift a value with an extract function
let d = Density::<OptionF, i32>::lift(
    Some(42),
    |opt| opt.unwrap(),
);
assert_eq!(d.extract(), 42);

// fmap composes onto extract -- no bounds on OptionF
let mapped = d.fmap(|x| format!("val={x}"));
assert_eq!(mapped.extract(), "val=42");
```


### Day\<F, G, A, B, C\>

Day Convolution -- pairs two functors with a combining function.


#### Definition

``` rust
/// Day f g a ≅ ∃b c. (f b, g c, b → c → a)
/// B and C are exposed as type parameters (Rust lacks existential types).
pub struct Day<F: HKT, G: HKT, A, B, C> {
    f_val: F::Of<B>,
    g_val: G::Of<C>,
    combine: Box<dyn Fn(B, C) -> A>,
    _marker: PhantomData<(F, G)>,
}

pub struct DayF<F: HKT + 'static, G: HKT + 'static>(PhantomData<(F, G)>);
```

`Day<F, G, A, B, C>` stores an `F<B>` value, a `G<C>` value, and a function `(B, C) → A`. `fmap` composes onto the combining function with no bounds on F or G. `run_day` interprets both sides into a target Applicative using two natural transformations.

#### Key Methods

``` rust
impl<F, G, A, B: Clone, C: Clone> Day<F, G, A, B, C> {
    /// Construct from two functor values and a combining function.
    pub fn new(f_val: F::Of<B>, g_val: G::Of<C>,
               combine: impl Fn(B, C) -> A + 'static) -> Self;

    /// Map over the result. No bounds on F or G required.
    pub fn fmap<D: 'static>(self, f: impl Fn(A) -> D + 'static) -> Day<F, G, D, B, C>;

    /// Interpret into target Applicative M using two natural transformations.
    pub fn run_day<M, NF, NG>(self) -> M::Of<A>
    where
        M: Applicative,
        NF: NaturalTransformation<F, M>,
        NG: NaturalTransformation<G, M>;
}
```

#### Laws


Functor Identity

``` rust
day.fmap(|a| a).run_day() == day.run_day()
```


Functor Composition

``` rust
day.fmap(|a| g(f(a))).run_day() == day.fmap(f).fmap(g).run_day()
```


#### Example

``` rust
use karpal_free::Day;
use karpal_core::hkt::OptionF;
use karpal_core::natural::NaturalTransformation;

struct OptionId;
impl NaturalTransformation<OptionF, OptionF> for OptionId {
    fn transform<A>(fa: Option<A>) -> Option<A> { fa }
}

// Pair two Option values with multiplication
let day = Day::<OptionF, OptionF, i32, i32, i32>::new(
    Some(3), Some(4), |a, b| a * b,
);
let result = day.run_day::<OptionF, OptionId, OptionId>();
assert_eq!(result, Some(12));

// fmap composes onto the combining function
let day = Day::<OptionF, OptionF, i32, i32, i32>::new(
    Some(2), Some(5), |a, b| a + b,
).fmap(|x| x * 3);
let result = day.run_day::<OptionF, OptionId, OptionId>();
assert_eq!(result, Some(21)); // (2+5)*3
```


### FreeAp\<F, A\>

The Free Applicative -- build applicative computations as data for static analysis before interpretation.


#### Definition

``` rust
/// Pure(a)   -- a finished computation
/// Ap(node)  -- an effect step (existentially quantified)
pub enum FreeAp<F: HKT + 'static, A: 'static> {
    Pure(A),
    Ap(Box<dyn FreeApNode<F, A>>),
}

pub struct FreeApF<F: HKT + 'static>(PhantomData<F>);
```

`FreeAp<F, A>` stores a computation tree where effects from `F` can be **statically analyzed** before interpretation. Unlike `Free<F, A>` (the free monad), effects in `FreeAp` do not depend on the results of previous effects -- they form a tree, not a chain.

#### Key Methods

``` rust
impl<F: HKT + 'static, A: 'static> FreeAp<F, A> {
    /// Wrap a pure value.
    pub fn pure(a: A) -> Self;

    /// Lift a single effect F<A>. Requires A: Clone (for Apply::ap).
    pub fn lift_f(fa: F::Of<A>) -> Self where A: Clone;

    /// Map a function over the result. No bounds on F required.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> FreeAp<F, B>;

    /// Applicative ap: apply wrapped functions to values.
    pub fn ap<B: 'static>(
        ff: FreeAp<F, Box<dyn Fn(A) -> B>>, fa: FreeAp<F, A>,
    ) -> FreeAp<F, B> where A: Clone;

    /// Collapse into F's own Applicative. Requires F: Applicative.
    pub fn retract(self) -> F::Of<A> where F: Applicative;

    /// Count the number of lift_f effects in this tree.
    pub fn count_effects(&self) -> usize;
}
```

#### FreeAp vs Free

|                     | Free (Monad)                                    | FreeAp (Applicative)                  |
|---------------------|-------------------------------------------------|---------------------------------------|
| Effect dependencies | Later effects depend on earlier results         | All effects are independent           |
| Static analysis     | Not possible (effects depend on runtime values) | Yes: `count_effects`, tree inspection |
| Interpretation      | `fold_map` via natural transformation           | `retract` into F's Applicative        |
| Power               | More powerful (monadic sequencing)              | Less powerful but more analyzable     |

#### Laws


Functor Identity

``` rust
fa.fmap(|a| a).retract() == fa.retract()
```


Applicative Identity

``` rust
FreeAp::ap(FreeAp::pure(Box::new(|x| x)), fa).retract() == fa.retract()
```


Applicative Homomorphism

``` rust
FreeAp::ap(FreeAp::pure(f), FreeAp::pure(x)).retract()
  == FreeAp::pure(f(x)).retract()
```


#### Example

``` rust
use karpal_free::FreeAp;
use karpal_core::hkt::OptionF;

// Build a computation tree
let fa = FreeAp::<OptionF, i32>::lift_f(Some(10));
let fb = fa.fmap(|x| x + 5);

// Static analysis: count effects before running
assert_eq!(fb.count_effects(), 1);

// Interpret by collapsing into Option's Applicative
let result = fb.retract();
assert_eq!(result, Some(15));

// Apply wrapped functions
let ff = FreeAp::<OptionF, Box<dyn Fn(i32) -> i32>>::pure(
    Box::new(|x| x * 2) as Box<dyn Fn(i32) -> i32>
);
let fa = FreeAp::<OptionF, i32>::lift_f(Some(21));
let result = FreeAp::ap(ff, fa).retract();
assert_eq!(result, Some(42));
```


### FreeAlt\<F, A\>

The Free Alternative -- choice among zero or more applicative computations.


#### Definition

``` rust
/// FreeAlt f a ≅ [FreeAp f a]
/// An empty list is zero (failure), multiple elements represent choice.
pub struct FreeAlt<F: HKT + 'static, A: 'static> {
    alternatives: Vec<FreeAp<F, A>>,
}

pub struct FreeAltF<F: HKT + 'static>(PhantomData<F>);
```

`FreeAlt<F, A>` wraps a list of `FreeAp<F, A>` branches. An empty list represents `zero` (failure/no alternatives). `alt` combines two alternatives by concatenating their branches. `retract` collapses into `F`'s own Alternative by folding branches with `F::alt`.

#### Key Methods

``` rust
impl<F: HKT + 'static, A: 'static> FreeAlt<F, A> {
    /// Wrap a pure value (single branch).
    pub fn pure(a: A) -> Self;

    /// Lift a single effect (single branch).
    pub fn lift_f(fa: F::Of<A>) -> Self where A: Clone;

    /// The empty alternative (zero / failure).
    pub fn zero() -> Self;

    /// Combine two alternatives (choice).
    pub fn alt(self, other: FreeAlt<F, A>) -> Self;

    /// Map a function over all branches.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> FreeAlt<F, B>;

    /// Collapse into F's own Alternative.
    pub fn retract(self) -> F::Of<A> where F: Alternative;

    /// Count branches.
    pub fn count_alternatives(&self) -> usize;

    /// Count total effects across all branches.
    pub fn count_effects(&self) -> usize;
}
```

#### Laws


Alt Associativity

``` rust
(a.alt(b)).alt(c).retract() == a.alt(b.alt(c)).retract()
```


Plus Left Identity

``` rust
FreeAlt::zero().alt(x).retract() == x.retract()
```


Plus Right Identity

``` rust
x.alt(FreeAlt::zero()).retract() == x.retract()
```


#### Example

``` rust
use karpal_free::FreeAlt;
use karpal_core::hkt::OptionF;

// Build alternatives
let a = FreeAlt::<OptionF, i32>::lift_f(None);
let b = FreeAlt::<OptionF, i32>::lift_f(Some(42));
let combined = a.alt(b);

// Inspect before interpreting
assert_eq!(combined.count_alternatives(), 2);
assert_eq!(combined.count_effects(), 2);

// retract: collapses using Option's alt (picks first Some)
let result = combined.retract();
assert_eq!(result, Some(42));

// zero is the identity for alt
let z = FreeAlt::<OptionF, i32>::zero();
assert_eq!(z.retract(), None);
```


## Design Notes

### Three families of free constructions

**Exposed type parameters** (Coyoneda, Lan, Day): These types keep type parameters visible (e.g. `B` in `Coyoneda<F, A, B>`, or `B, C` in `Day<F, G, A, B, C>`) representing the original values inside the functors. This is the simplest encoding and avoids trait objects for the core structure, though `fmap` closures still require `Box<dyn Fn>`.

**Trait objects with dyn-safe traits** (Yoneda, Codensity, Density, Freer, FreeAp): These types erase an internal type parameter via `Box<dyn Trait>`. This requires `F: 'static` and `A: 'static` bounds everywhere. Due to a Rust GAT limitation (you cannot add `T: 'static` to `type Of<T>` in a trait impl when the trait definition doesn't have it), marker types like `CodensityF`, `FreerF`, `FreeApF`, and `FreeAltF` **cannot implement** the `HKT` or `Functor` traits. Use the inherent methods instead.

**Recursive enums/structs** (Free, Cofree): These use direct recursion (not trait objects), so they avoid the `'static` limitation entirely. `FreeF` implements `HKT + Functor`; `CofreeF` implements the full `HKT + Functor + Extend + Comonad` chain.

**Composite wrappers** (FreeAlt): Built on top of other free constructions. `FreeAlt<F, A>` is simply `Vec<FreeAp<F, A>>`, providing Alternative structure by combining applicative branches.

### Kan extension relationships

Several of these types are specialisations of Kan extensions:

| Kan extension             | Specialisation | Result              |
|---------------------------|----------------|---------------------|
| `Lan<IdentityF, F, A, B>` | G = Identity   | Coyoneda\<F, A, B\> |
| `Lan<W, W, A, S>`         | G = H = W      | Density\<W, A\>     |
| `Ran<F, F>`               | G = H = F      | Codensity\<F, A\>   |

### Why Free doesn't implement the Monad trait

`FreeF` implements `Functor` but not `Apply`, `Chain`, or `Monad`. The problem is `Apply::ap`: in the `Roll` case, each child of the function tree needs its own copy of the argument tree. The trait only provides `A: Clone`, not `Free<F, A>: Clone`. Adding bounds to an impl that the trait doesn't have is not allowed in Rust. The inherent `chain` method avoids this issue since it doesn't need to clone its argument.

### Why Cofree doesn't implement Clone

A generic `Clone` impl for `Cofree<F, A>` would require `F::Of<Cofree<F, A>>: Clone`, which triggers **infinite recursion** in the compiler's trait resolution (coinductive reasoning, which Rust doesn't support). For the same reason, `Cofree` doesn't provide a `duplicate` method (which would need `Clone` internally). Use `extend` directly instead.

### The `&dyn Fn` recursion pattern

Recursive methods like `fmap` and `extend` on Free/Cofree pass their closure to child nodes. Naively passing `&f` causes each recursive level to add a reference layer (`&Fn`, `&&Fn`, `&&&Fn`, ...), hitting the monomorphization recursion limit. The solution: public methods accept `impl Fn`, then immediately delegate to a private `_inner` method that takes `&dyn Fn`. Since the type is erased behind `dyn`, the recursive calls all use the same concrete type.

``` rust
// Public API: accepts impl Fn
pub fn fmap<B>(self, f: impl Fn(A) -> B) -> Cofree<F, B> {
    self.fmap_inner(&f)  // erase to &dyn Fn
}

// Private: fixed type at every recursion level
fn fmap_inner<B>(self, f: &dyn Fn(A) -> B) -> Cofree<F, B> {
    Cofree {
        head: f(self.head),
        tail: Box::new(F::fmap(*self.tail, |child| child.fmap_inner(f))),
    }
}
```

### Why FreeAp uses `retract` instead of `fold_map`

In Haskell, the primary eliminator for free applicatives is `foldMap :: (forall x. f x -> g x) -> Ap f a -> g a`, which interprets into any target Applicative via a natural transformation. In Rust, this is **impossible** through dyn dispatch.

The issue: `FreeAp::Ap` erases an intermediate type `B` behind a trait object (`Box<dyn FreeApNode<F, A>>`). Calling `NT::transform<B>(effect)` requires compile-time monomorphisation for the specific `B`, but `B` has been erased. Rust cannot dispatch a generic function through a type-erased interface.

`retract` avoids this because it collapses into `F` itself -- `F` is already a type parameter of the trait. To interpret into a *different* Applicative `M`, apply your natural transformation at each `lift_f` call site, then `retract`:

``` rust
// fold_map nt ≡ retract . hoist nt
// In practice: apply NT when building, then retract
let free_m: FreeAp<M, A> = FreeAp::lift_f(NT::transform(effect));
let result: M::Of<A> = free_m.retract();
```

### Why Ran is a trait, not a struct

Ran encodes a universally quantified function: `∀R. (A → G R) → H R`. In Rust, this requires a generic method `run_ran<R>`, which cannot be made object-safe. Unlike Codensity (which restricts to a single eliminator `to_monad`), Ran preserves the full generality of `∀R`, so it must remain a trait. Use `ran_fmap` for functor-like mapping over Ran implementations.

## See Also

- [**Functor Family**](functor-family.md) -- the traits that Free/Cofree/FreeAp build on (Functor, Apply, Applicative, Chain, Monad)
- [**Alt Family**](alt-family.md) -- Alt, Plus, and Alternative, which FreeAlt's `retract` requires
- [**Comonad Family**](comonad-family.md) -- Extend and Comonad, which Cofree implements
- [**Bifunctor & Natural**](bifunctor-natural.md) -- NaturalTransformation, used by `Free::fold_map`, `Freer::fold_map`, `Lan::lower`, and `Day::run_day`


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



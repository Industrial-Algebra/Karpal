# Adjunctions & Category Theory

Advanced category-theoretic constructions in `karpal-core`. An adjunction F ⊣ U is the fundamental relationship that gives rise to monads and comonads. This module also includes functor composition, ends, coends, dinatural transformations, the continuation monad, and profunctor-level adjunctions.

## Overview

| Concept                   | Module       | Key idea                                                                                      |
|---------------------------|--------------|-----------------------------------------------------------------------------------------------|
| `Adjunction<F, U>`        | `adjunction` | F ⊣ U: `unit: A → U(F(A))`, `counit: F(U(B)) → B`                                             |
| `ComposeF<F, G>`          | `compose`    | Functor composition: `(F . G)(A) = F(G(A))`                                                   |
| `DinaturalTransformation` | `dinatural`  | Transform between profunctor diagonals: `P(A,A) → Q(A,A)`                                     |
| `End<P>`                  | `end`        | Universal quantification: `∀A. P(A,A)`                                                        |
| `Coend<P, A>`             | `coend`      | Existential quantification: `∃A. P(A,A)`                                                      |
| `ContravariantAdjunction` | `adjunction` | Adjunction between contravariant functors; `ContF<R> ⊣ ContF<R>` gives the continuation monad |
| `ProfunctorAdjunction`    | `adjunction` | Adjunction in the category of profunctors                                                     |

## Adjunction


### Adjunction\<F, U\>

The fundamental relationship between a left adjoint F and right adjoint U.


#### Signature

``` rust
pub trait Adjunction<F: HKT, U: HKT> {
    fn unit<A: Clone + 'static>(a: A) -> U::Of<F::Of<A>>;
    fn counit<B: 'static>(fub: F::Of<U::Of<B>>) -> B;
}
```

The trait is bounded by `HKT` rather than `Functor` because some right adjoints (like `ReaderF<E>`) cannot implement the generic `Functor` trait due to `'static` limitations on `Box<dyn Fn>`.

#### Laws (Triangle Identities)


Left Triangle

``` rust
// counit(F::fmap(fa, unit)) == fa
// "Going up then back down is identity on F"
```


Right Triangle

``` rust
// U::fmap(unit(a), counit) == a
// "Going up then back down is identity on U"
```


#### Derived Operations

``` rust
// left_adjunct: (F(A) -> B) -> (A -> U(B))
fn left_adjunct(f, a) = U::fmap(unit(a), f)

// right_adjunct: (A -> U(B)) -> (F(A) -> B)
fn right_adjunct(f, fa) = counit(F::fmap(fa, f))
```

#### Instances

| Witness       | F (left)    | U (right)    | Feature  |
|---------------|-------------|--------------|----------|
| `IdentityAdj` | `IdentityF` | `IdentityF`  | `no_std` |
| `CurryAdj<E>` | `EnvF<E>`   | `ReaderF<E>` | `alloc`  |


## Monad & Comonad from Adjunctions

Every adjunction F ⊣ U gives rise to both a monad and a comonad:

- **Monad on U . F** — `pure = unit`, `join = U(counit)`
- **Comonad on F . U** — `extract = counit`, `duplicate = F(unit)`


### CurryAdj\<E\> — Product/Exponential Adjunction

`EnvF<E> ⊣ ReaderF<E>`: the canonical adjunction giving State and Store.


#### How It Works

``` text
EnvF<E>::Of<A>   = (E, A)            -- product ("pairing with environment")
ReaderF<E>::Of<A> = Box<dyn Fn(E) -> A>  -- exponential ("function from environment")

unit(a) = |e| (e, a)                   -- embed value into reader of pairs
counit((e, f)) = f(e)                  -- apply the function to the environment
```

#### State Monad (U . F = ReaderF . EnvF)

The composed functor `ReaderF<E> . EnvF<E>` gives `Of<A> = Box<dyn Fn(E) → (E, A)>` — exactly the State monad, where the environment `E` is threaded and potentially modified.

``` rust
use karpal_core::adjunction::*;

// State monad: E -> (E, A) where E is mutable state
let get = state_get::<i32>();               // |e| (e, e)
let put = |s| state_put(s);                  // |_| (s, ())
let modify = state_modify(|e: i32| e + 1);  // |e| (e+1, ())

// Pure wraps a value without touching state
let pure_42 = state_pure::<i32, _>(42);
assert_eq!(pure_42(0), (0, 42));

// Chain sequences state-passing computations
let program = state_chain(
    state_get::<i32>(),
    |x| state_chain(
        state_modify(move |e: i32| e + x),
        |_| state_get::<i32>(),
    ),
);
assert_eq!(program(10), (20, 20));  // get 10, add 10, get 20
```

#### Store Comonad (F . U = EnvF . ReaderF)

The composed functor `EnvF<E> . ReaderF<E>` gives `Of<A> = (E, Box<dyn Fn(E) → A>)` — the Store comonad, holding a position and a function to look up values.

``` rust
use karpal_core::adjunction::*;

// Store: (position, lookup_function)
let store: (i32, Box<dyn Fn(i32) -> i32>) =
    (5, Box::new(|e| e * e));

assert_eq!(store_pos(&store), 5);       // current position
assert_eq!(store_peek(3, &store), 9);    // lookup(3) = 9
assert_eq!(store_extract(store), 25);   // lookup(5) = 25 (moves store)
```


## ReaderF\<E\>


### ReaderF\<E\>

The Reader functor: `Of<T> = Box<dyn Fn(E) → T>`. Right adjoint of `EnvF<E>`.


`ReaderF<E>` cannot implement the generic `Functor` trait because `Box<dyn Fn>` requires `'static` bounds that the trait signature doesn't allow. Instead, it provides equivalent functionality via **inherent methods** with `'static` bounds on the impl block (the "Lan workaround").

#### Inherent Methods

``` rust
impl<E: Clone + 'static> ReaderF<E> {
    fn fmap<A: 'static, B: 'static>(
        fa: Box<dyn Fn(E) -> A>,
        f: impl Fn(A) -> B + 'static,
    ) -> Box<dyn Fn(E) -> B>;

    fn pure<A: Clone + 'static>(a: A) -> Box<dyn Fn(E) -> A>;

    fn chain<A: 'static, B: 'static>(
        fa: Box<dyn Fn(E) -> A>,
        f: impl Fn(A) -> Box<dyn Fn(E) -> B> + 'static,
    ) -> Box<dyn Fn(E) -> B>;

    fn ask() -> Box<dyn Fn(E) -> E>;

    fn local<A: 'static>(
        f: impl Fn(E) -> E + 'static,
        reader: Box<dyn Fn(E) -> A>,
    ) -> Box<dyn Fn(E) -> A>;
}
```

#### Examples

``` rust
use karpal_core::hkt::ReaderF;

// Reader monad: shared read-only environment
let reader = ReaderF::<String>::chain(
    ReaderF::ask(),
    |env: String| ReaderF::pure(env.len()),
);
assert_eq!(reader("hello".to_string()), 5);
```

**State vs Reader:** Reader's `chain` passes the *same* environment to both computations (`|e| f(reader(e))(e)`), while State threads *modified* state (`|e| let (e', a) = m(e); f(a)(e')`). They are different monads arising from the same adjunction.


## Functor Composition


### ComposeF\<F, G\>

Compose two type constructors: `(F . G)(A) = F(G(A))`.


#### Signature

``` rust
pub struct ComposeF<F, G>(PhantomData<(F, G)>);

impl<F: HKT, G: HKT> HKT for ComposeF<F, G> {
    type Of<T> = F::Of<G::Of<T>>;
}

impl<F: Functor, G: Functor> Functor for ComposeF<F, G> {
    fn fmap<A, B>(fga: F::Of<G::Of<A>>, f: impl Fn(A) -> B) -> F::Of<G::Of<B>> {
        F::fmap(fga, |ga| G::fmap(ga, &f))
    }
}
```

#### Examples

``` rust
use karpal_core::compose::ComposeF;
use karpal_core::functor::Functor;
use karpal_core::hkt::{OptionF, VecF};

// Option<Option<i32>> as a composed functor
let val: Option<Option<i32>> = Some(Some(42));
let result = ComposeF::<OptionF, OptionF>::fmap(val, |x| x + 1);
assert_eq!(result, Some(Some(43)));

// Vec<Option<i32>> -- fmap reaches through both layers
let val: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
let result = ComposeF::<VecF, OptionF>::fmap(val, |x| x * 10);
assert_eq!(result, vec![Some(10), None, Some(30)]);
```

Functor composition is the key building block for adjunction-derived monads and comonads: the State monad is `ComposeF<ReaderF<E>, EnvF<E>>` and the Store comonad is `ComposeF<EnvF<E>, ReaderF<E>>`.


## Dinatural Transformation


### DinaturalTransformation\<P, Q\>

A transformation between two profunctors on the diagonal: `P(A,A) → Q(A,A)`.


#### Signature

``` rust
pub trait DinaturalTransformation<P: HKT2, Q: HKT2> {
    fn transform<A: 'static>(paa: P::P<A, A>) -> Q::P<A, A>;
}
```

A dinatural transformation is to profunctors what a natural transformation is to functors. Where a natural transformation has components `α_A: F(A) → G(A)`, a dinatural transformation has components `α_A: P(A,A) → Q(A,A)` that work on the diagonal of a profunctor (both type parameters equal).

#### Instances

| Witness       | Description                                      |
|---------------|--------------------------------------------------|
| `DinaturalId` | Identity: `P(A,A) → P(A,A)` for any profunctor P |

#### Examples

``` rust
use karpal_core::dinatural::*;
use karpal_core::hkt::TupleF;

let val: (i32, i32) = (1, 2);
let result = <DinaturalId as DinaturalTransformation<TupleF, TupleF>>
    ::transform::<i32>(val);
assert_eq!(result, (1, 2));
```


## Ends & Coends


### End\<P\>

Universal quantification over a profunctor's diagonal: `∀A. P(A,A)`.


#### Signature

``` rust
pub trait End<P: HKT2> {
    fn run<A: 'static>(&self) -> P::P<A, A>;
}
```

An end is a value that, when asked for any type `A`, can produce a `P(A,A)`. This is the categorical analogue of `forall A. P(A,A)` from System F. The `End` trait is not dyn-compatible (it has a generic method), so implementations must be concrete types.

Ends appear in categorical constructions like the Yoneda lemma (`∫_A [A, F(A)] ≅ F`) and are the natural setting for parametric polymorphism.


### Coend\<P, A\>

Existential quantification over a profunctor's diagonal: `∃A. P(A,A)`.


#### Signature

``` rust
pub struct Coend<P: HKT2, A> {
    pub value: P::P<A, A>,
}

impl<P: HKT2, A> Coend<P, A> {
    pub fn new(value: P::P<A, A>) -> Self;
    pub fn elim<R>(self, f: impl FnOnce(P::P<A, A>) -> R) -> R;
}
```

A coend packages a `P(A,A)` value where the type `A` is "existentially hidden". Since Rust lacks existential types, `A` is exposed as a type parameter. The `elim` method provides CPS-style elimination.

#### Examples

``` rust
use karpal_core::coend::Coend;
use karpal_core::hkt::TupleF;

let c = Coend::<TupleF, i32>::new((42, 42));
let sum = c.elim(|(a, b)| a + b);
assert_eq!(sum, 84);
```


## Contravariant Adjunction


### ContravariantAdjunction\<F, G\>

An adjunction between contravariant functors, giving rise to the continuation monad.


#### Signature

``` rust
pub trait ContravariantAdjunction<F: HKT, G: HKT> {
    fn unit<A: Clone + 'static>(a: A) -> G::Of<F::Of<A>>;
    fn counit<B: Clone + 'static>(b: B) -> F::Of<G::Of<B>>;
}
```

For contravariant functors, the composition `G . F` is *covariant* (two contravariants compose to give a covariant functor). The primary instance is the self-adjunction of `ContF<R>`, which gives the continuation monad.

#### ContF\<R\> — The Continuation Functor

``` rust
pub struct ContF<R>(PhantomData<R>);

// Of<A> = Box<dyn Fn(A) -> R>
// Generalizes PredicateF (which is ContF<bool>)
```

#### Instances

| Witness      | F          | G          | Resulting Monad              |
|--------------|------------|------------|------------------------------|
| `ContAdj<R>` | `ContF<R>` | `ContF<R>` | `(A → R) → R` (Continuation) |

`ContAdj<R>` is self-adjoint: `unit` and `counit` are the same operation (`|k| k(a)`), embedding a value into CPS form.

#### Continuation Monad Helpers

``` rust
use karpal_core::adjunction::*;

// Pure: embed a value into CPS
let m = cont_pure::<i32, _>(42);
assert_eq!(cont_run(&*m, |x| x + 1), 43);

// Fmap: transform inside CPS
let mapped = cont_fmap(|x: i32| x * 3, cont_pure(10));
assert_eq!(cont_run(&*mapped, |x| x + 1), 31);  // (10 * 3) + 1

// Chain (bind): sequence CPS computations
let chained = cont_chain(cont_pure(5), |x| cont_pure(x + 10));
assert_eq!(cont_run(&*chained, |x| x * 2), 30);  // (5 + 10) * 2

// call/cc: call-with-current-continuation
let m = cont_call_cc::<i32, i32, i32>(|escape| {
    // escape(10) short-circuits, ignoring the rest
    let escaped = escape(10);
    cont_chain(escaped, |_| cont_pure(999))  // never reached
});
assert_eq!(cont_run(&*m, |x| x), 10);  // not 999!
```


## Profunctor Adjunction


### ProfunctorAdjunction\<F, U\>

An adjunction in the category of profunctors.


#### Signature

``` rust
/// Type-level functor on profunctors
pub trait ProfunctorFunctor {
    type Applied<P: HKT2>: HKT2;
}

/// Adjunction between profunctor functors
pub trait ProfunctorAdjunction<F: ProfunctorFunctor, U: ProfunctorFunctor> {
    fn unit<P: HKT2, A: 'static, B: 'static>(
        pab: P::P<A, B>,
    ) -> <U::Applied<F::Applied<P>> as HKT2>::P<A, B>;

    fn counit<Q: HKT2, A: 'static, B: 'static>(
        fuqab: <F::Applied<U::Applied<Q>> as HKT2>::P<A, B>,
    ) -> Q::P<A, B>;
}
```

`ProfunctorFunctor` maps profunctors to profunctors using GATs as an HKT3-like encoding. A `ProfunctorAdjunction` witnesses a left/right adjoint pair at the profunctor level, with unit and counit that are natural transformations between profunctors.

#### Instances

| Witness                 | F                     | U                     |
|-------------------------|-----------------------|-----------------------|
| `ProfunctorIdentityAdj` | `ProfunctorIdentityF` | `ProfunctorIdentityF` |

The identity instance maps every profunctor to itself. Non-trivial instances like `Pastro ⊣ Tambara` require profunctor transformer types and are planned for future phases.

#### Examples

``` rust
use karpal_core::adjunction::*;
use karpal_core::hkt::TupleF;

// Identity profunctor adjunction: roundtrip is identity
let val: (i32, String) = (42, "hello".into());
let roundtrip = ProfunctorIdentityAdj::counit::<TupleF, i32, String>(
    ProfunctorIdentityAdj::unit::<TupleF, i32, String>(val.clone()),
);
assert_eq!(roundtrip, val);
```


## Design Notes

- **HKT not Functor in Adjunction trait** — `ReaderF<E>` can't implement generic `Functor` (GAT can't add `'static` bounds that the trait lacks). The trait uses `HKT` bounds; standalone helper functions add `Functor` bounds where needed.
- **Lan workaround for ReaderF** — `'static` bounds go on the `impl` block rather than the trait, allowing `Box<dyn Fn>` usage without changing trait signatures. This pattern was pioneered in the Lan/Ran implementations.
- **State ≠ Reader** — Though both derive from `CurryAdj<E>`, the State monad threads modified state while Reader shares the same environment. State's `chain` is `|e| let (e', a) = m(e); f(a)(e')`; Reader's is `|e| f(reader(e))(e)`.
- **Rc for continuation closures** — `cont_chain` and `cont_call_cc` use `Rc` to share closures across multiple `Fn` invocations. This is required because `Fn` closures must be callable multiple times but captured values are moved.
- **End is not dyn-compatible** — The `run<A>` method is generic, so `End` cannot be used as a trait object. Concrete types must implement it.
- **Coend exposes A** — Rust lacks existential types, so the "hidden" type parameter is exposed. The `elim` method provides CPS-style consumption.
- **`no_std` support** — `IdentityAdj`, `ComposeF`, `DinaturalTransformation`, `End`, `Coend`, `ProfunctorFunctor`, and `ProfunctorAdjunction` all work without `std` or `alloc`. `CurryAdj`, `ContAdj`, `ReaderF`, and continuation helpers require `alloc`.


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



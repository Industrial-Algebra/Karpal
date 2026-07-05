# Comonad Family

Dual of the monad hierarchy: comonadic context and extraction.

Where a Monad lets you *inject* values into a context and *sequence* context-producing computations, a Comonad lets you *extract* values from a context and *extend* context-consuming functions across an entire structure. The comonad family in Karpal consists of five traits arranged in a linear hierarchy with three specialized branches.

## Hierarchy


Functor → Extend → Comonad → ComonadEnv  
Functor → Extend → Comonad → ComonadStore \*  
Functor → Extend → Comonad → ComonadTraced \*


**\* Design note:** `ComonadStore` and `ComonadTraced` require `HKT` (not `Comonad`) as their supertrait. This is because `StoreF` and `TracedF` use `Box<dyn Fn>` internally, which imposes `'static` bounds that are incompatible with the generic `Functor` signature. Since `Functor` is a supertrait of `Extend` and `Comonad`, these types cannot implement the full comonad chain. Instead, they provide their own `extract` method directly on the trait, defined as a default method in terms of `peek`/`trace`.

## Extend


### Extend

The dual of Chain. Enables cooperative, context-aware computation.


#### Signature

``` rust
pub trait Extend: Functor {
    fn extend<A, B>(wa: Self::Of<A>, f: impl Fn(&Self::Of<A>) -> B) -> Self::Of<B>
    where
        A: Clone;

    fn duplicate<A>(wa: Self::Of<A>) -> Self::Of<Self::Of<A>>
    where
        A: Clone,
        Self::Of<A>: Clone;
}
```

Given a value in context `W<A>` and a function `&W<A> -> B` that can inspect the full context, `extend` applies that function at every "position" in the structure, producing `W<B>`. The `duplicate` method has a default implementation: `Self::extend(wa, |w| w.clone())`.

#### Laws


Associativity


extend(f, extend(g, w)) == extend(\|w\| f(&extend(g, w.clone())), w)


#### Instances

| Type constructor | `Of<A>`          | Notes                                               |
|------------------|------------------|-----------------------------------------------------|
| `IdentityF`      | `A`              | Trivially applies `f` to the value                  |
| `OptionF`        | `Option<A>`      | Applies `f` if `Some`; returns `None` otherwise     |
| `NonEmptyVecF`   | `NonEmptyVec<A>` | Applies `f` to each suffix (alloc-gated)            |
| `EnvF<E>`        | `(E, A)`         | Applies `f` to the pair, preserving the environment |

#### Example

``` rust
use karpal_core::prelude::*;

// NonEmptyVec extend: apply a summary function to each suffix
let nev = NonEmptyVec::new(1, vec![2, 3]);
let sums = NonEmptyVecF::extend(nev, |w| w.iter().sum::<i32>());
// Suffixes: [1,2,3], [2,3], [3]  =>  Sums: 6, 5, 3
assert_eq!(sums, NonEmptyVec::new(6, vec![5, 3]));

// Option extend
let doubled = OptionF::extend(Some(3), |opt| match opt {
    Some(x) => x * 2,
    None => 0,
});
assert_eq!(doubled, Some(6));

// duplicate: embed the structure inside itself
let nested = OptionF::duplicate(Some(42));
assert_eq!(nested, Some(Some(42)));
```


## Comonad


### Comonad

The categorical dual of Monad. Extract a value from context.


#### Signature

``` rust
pub trait Comonad: Extend {
    fn extract<A: Clone>(wa: &Self::Of<A>) -> A;
}
```

A Comonad can `extract` a value from a context and `extend` a context-aware function over the entire structure. Where `Monad::pure` injects a value into a minimal context, `Comonad::extract` pulls a value out of an existing context.

#### Laws


Left identity


extract(&extend(w, f)) == f(&w)


Right identity


extend(w, \|w\| extract(w)) == w


Associativity is inherited from `Extend`.

#### Instances

| Type constructor | `Of<A>`          | Notes                                                 |
|------------------|------------------|-------------------------------------------------------|
| `IdentityF`      | `A`              | Returns the value directly                            |
| `OptionF`        | `Option<A>`      | Panics on `None` (partial comonad)                    |
| `NonEmptyVecF`   | `NonEmptyVec<A>` | Returns the head element (alloc-gated)                |
| `EnvF<E>`        | `(E, A)`         | Returns the `A` component, discarding the environment |

#### Example

``` rust
use karpal_core::prelude::*;

// Extract from NonEmptyVec: always returns the head
let nev = NonEmptyVec::new(10, vec![20, 30]);
assert_eq!(NonEmptyVecF::extract(&nev), 10);

// Extract from Env: discards the environment, keeps the value
assert_eq!(EnvF::<&str>::extract(&("config", 42)), 42);

// Left identity law in action:
let f = |w: &NonEmptyVec<i32>| w.head + 1;
let extended = NonEmptyVecF::extend(nev.clone(), f);
assert_eq!(NonEmptyVecF::extract(&extended), f(&nev));
```


## ComonadEnv


### ComonadEnv\<E\>

A Comonad with access to an environment value. Dual of Reader/MonadReader.


#### Signature

``` rust
pub trait ComonadEnv<E>: Comonad {
    fn ask<A>(wa: &Self::Of<A>) -> E;
    fn local<A>(wa: Self::Of<A>, f: impl Fn(E) -> E) -> Self::Of<A>;
}
```

`ask` retrieves the environment from the comonadic value. `local` transforms the environment while leaving the focus value unchanged.

#### Laws


Local preserves extract


extract(local(wa, f)) == extract(wa)


#### Instances

| Type constructor | `Of<A>`  | Notes                                             |
|------------------|----------|---------------------------------------------------|
| `EnvF<E>`        | `(E, A)` | `ask` returns `E`; `local` transforms `E` via `f` |

#### Example

``` rust
use karpal_core::prelude::*;

let w = ("hello", 42);

// ask: retrieve the environment
assert_eq!(EnvF::<&str>::ask(&w), "hello");

// local: transform the environment, keep the value
let w2 = (10i32, "value");
let result = EnvF::<i32>::local(w2, |e| e * 2);
assert_eq!(result, (20, "value"));

// Law: local does not change the extracted value
assert_eq!(
    EnvF::<i32>::extract(&EnvF::<i32>::local((5, 99), |e| e + 1)),
    EnvF::<i32>::extract(&(5, 99))
);
```


## ComonadStore


### ComonadStore\<S\>

A comonad with a notion of position and peeking. Dual of State.


#### Signature

``` rust
pub trait ComonadStore<S>: HKT {
    fn pos<A>(wa: &Self::Of<A>) -> S;
    fn peek<A>(s: S, wa: &Self::Of<A>) -> A;

    /// Extract the focused value (equivalent to `peek(pos(wa), wa)`).
    fn extract<A>(wa: &Self::Of<A>) -> A
    where
        S: Clone;
}
```

`pos` returns the current position (index, key, cursor) within the store. `peek` retrieves the value at an arbitrary position. The default `extract` method is defined as `peek(pos(wa), wa)`.

**Design note:** `ComonadStore` requires `HKT` rather than `Comonad` as its supertrait. `StoreF<S>` is represented as `(Box<dyn Fn(S) -> A>, S)`, which requires `'static` bounds on `S`. The generic `Functor` trait does not carry this bound, so `StoreF` cannot implement `Functor` and therefore cannot implement `Extend` or `Comonad`. The `extract` method is provided directly on this trait instead.

#### Laws


Peek-pos identity


peek(pos(wa), wa) == extract(wa)


#### Instances

| Type constructor | `Of<A>`                    | Notes                                      |
|------------------|----------------------------|--------------------------------------------|
| `StoreF<S>`      | `(Box<dyn Fn(S) -> A>, S)` | Alloc-gated; requires `S: Clone + 'static` |

#### Example

``` rust
use karpal_core::prelude::*;

// A Store is a pair of (lookup function, current position)
let store: (Box<dyn Fn(i32) -> String>, i32) =
    (Box::new(|s| format!("value_{}", s)), 42);

// pos: get the current position
assert_eq!(StoreF::<i32>::pos(&store), 42);

// peek: look up the value at any position
assert_eq!(StoreF::<i32>::peek(10, &store), "value_10");

// extract: peek at the current position
assert_eq!(StoreF::<i32>::extract(&store), "value_42");
```


## ComonadTraced


### ComonadTraced\<M: Monoid\>

A comonad with a monoidal trace/accumulator. Dual of Writer.


#### Signature

``` rust
pub trait ComonadTraced<M: Monoid>: HKT {
    fn trace<A>(m: M, wa: &Self::Of<A>) -> A;

    /// Extract the focused value (equivalent to `trace(M::empty(), wa)`).
    fn extract<A>(wa: &Self::Of<A>) -> A;
}
```

`trace` queries the comonadic value with a monoidal input. The default `extract` method traces with the monoidal identity (`M::empty()`), yielding the "current" value without any accumulated trace.

**Design note:** Like `ComonadStore`, this trait requires `HKT` rather than `Comonad` as its supertrait. `TracedF<M>` is represented as `Box<dyn Fn(M) -> A>`, which imposes `'static` bounds incompatible with the generic `Functor` signature.

#### Laws


Identity trace


trace(M::empty(), wa) == extract(wa)


#### Instances

| Type constructor | `Of<A>`               | Notes                                               |
|------------------|-----------------------|-----------------------------------------------------|
| `TracedF<M>`     | `Box<dyn Fn(M) -> A>` | Alloc-gated; requires `M: Monoid + Clone + 'static` |

#### Example

``` rust
use karpal_core::prelude::*;

// A Traced comonad is a function from a monoid to a value
let w: Box<dyn Fn(i32) -> String> = Box::new(|m| format!("traced_{}", m));

// trace: query with a specific monoidal value
assert_eq!(TracedF::<i32>::trace(5, &w), "traced_5");

// extract: trace with the monoidal identity (i32::empty() == 0)
assert_eq!(TracedF::<i32>::extract(&w), "traced_0");
```


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



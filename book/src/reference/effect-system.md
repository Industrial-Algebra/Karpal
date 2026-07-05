# Effect System & Monad Transformers

The `karpal-effect` crate provides monad transformers — composable building blocks for stacking effects (errors, state, environment, logging) on top of any inner monad. It also introduces `FunctorSt`, `ApplicativeSt`, and `ChainSt` — variants of the functor hierarchy with `'static` bounds required by Rust's `Box<dyn Fn>`.

## Overview

| Transformer      | Representation                    | Effect                                                    |
|------------------|-----------------------------------|-----------------------------------------------------------|
| `ExceptTF<E, M>` | `M::Of<Result<A, E>>`             | Error handling — short-circuits on `Err`                  |
| `WriterTF<W, M>` | `M::Of<(A, W)>`                   | Log accumulation — `W` must be a `Monoid`                 |
| `ReaderTF<E, M>` | `Box<dyn Fn(E) -> M::Of<A>>`      | Shared environment — every computation reads the same `E` |
| `StateTF<S, M>`  | `Box<dyn Fn(S) -> M::Of<(S, A)>>` | Mutable state — state is threaded through computations    |

All four transformers implement `HKT`, `FunctorSt`, `ChainSt`, and `MonadTrans`. `ExceptTF` and `WriterTF` additionally implement `ApplicativeSt`.

## Static Type Classes

The standard `Functor` / `Applicative` / `Chain` traits in `karpal-core` do not have `'static` bounds on their type parameters. Monad transformers that use `Box<dyn Fn>` internally need these bounds, so `karpal-effect` introduces parallel traits with the suffix `St`.


### FunctorSt / ApplicativeSt / ChainSt

Mirror traits with `'static` bounds for transformer compatibility.


#### Signatures

``` rust
pub trait FunctorSt: HKT {
    fn fmap_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> B + 'static,
    ) -> Self::Of<B>;
}

pub trait ApplicativeSt: FunctorSt {
    fn pure_st<A: 'static>(a: A) -> Self::Of<A>;
}

pub trait ChainSt: FunctorSt {
    fn chain_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> Self::Of<B> + 'static,
    ) -> Self::Of<B>;
}
```

#### Base Instances

| Type         | `FunctorSt` | `ApplicativeSt` | `ChainSt` |
|--------------|-------------|-----------------|-----------|
| `OptionF`    | Yes         | Yes             | Yes       |
| `ResultF<E>` | Yes         | Yes             | Yes       |
| `IdentityF`  | Yes         | Yes             | Yes       |
| `VecF`       | Yes         | Yes             | Yes       |

These implementations are trivial — for `OptionF`, `fmap_st` is just `fa.map(f)`. The `'static` bound matches what `Box<dyn Fn>` requires, so base types that work with boxed closures satisfy it automatically.


### MonadTrans

Lift an inner monad computation into a transformer stack.


#### Signature

``` rust
pub trait MonadTrans<M: HKT>: HKT {
    fn lift<A: 'static>(ma: M::Of<A>) -> Self::Of<A>
    where
        M::Of<A>: Clone;
}
```

`lift` embeds an `M` computation into the transformer without adding any effect. The `Clone` bound on `M::Of<A>` is needed by closure-based transformers (ReaderT, StateT) whose inner function may be called multiple times.

#### Law


lift preserves pure

``` rust
lift(M::pure_st(a)) == pure(a)
```


#### Examples

``` rust
use karpal_effect::{MonadTrans, ExceptTF, WriterTF, ReaderTF, StateTF};
use karpal_core::hkt::OptionF;

// Lift Some(42) into ExceptT — produces Some(Ok(42))
let lifted = ExceptTF::<&str, OptionF>::lift(Some(42));
assert_eq!(lifted, Some(Ok(42)));

// Lift Some(42) into WriterT — produces Some((42, ""))
let lifted = WriterTF::<String, OptionF>::lift(Some(42));
assert_eq!(lifted, Some((42, String::new())));

// Lift Some(42) into ReaderT — ignores the environment
let lifted = ReaderTF::<i32, OptionF>::lift(Some(42));
assert_eq!(lifted(999), Some(42));

// Lift Some(42) into StateT — passes state through unchanged
let lifted = StateTF::<i32, OptionF>::lift(Some(42));
assert_eq!(lifted(99), Some((99, 42)));
```


## Monad Transformers


### ExceptTF\<E, M\>

Adds error handling to an inner monad. Equivalent to `EitherT` / `ExceptT` in Haskell.


#### Representation

``` rust
pub struct ExceptTF<E, M>(PhantomData<(E, M)>);

// ExceptTF<E, M>::Of<A> = M::Of<Result<A, E>>
impl<E: 'static, M: HKT> HKT for ExceptTF<E, M> {
    type Of<A> = M::Of<Result<A, E>>;
}
```

This is the simplest transformer — the inner monad wraps `Result<A, E>` directly. No closures, no `Box<dyn Fn>`.

#### Trait Implementations

| Trait           | Bounds on `M`                |
|-----------------|------------------------------|
| `FunctorSt`     | `M: FunctorSt`               |
| `ApplicativeSt` | `M: ApplicativeSt`           |
| `ChainSt`       | `M: ChainSt + ApplicativeSt` |
| `MonadTrans<M>` | `M: FunctorSt`               |

#### Operations

``` rust
// pure: wrap a value in Ok inside the inner monad
fn except_t_pure<E, M: ApplicativeSt, A>(a: A) -> M::Of<Result<A, E>>;

// fmap: apply a function to the Ok value
fn except_t_fmap<E, M: FunctorSt, A, B>(fa, f) -> M::Of<Result<B, E>>;

// chain: short-circuits on Err
fn except_t_chain<E, M: ChainSt + ApplicativeSt, A, B>(fa, f) -> M::Of<Result<B, E>>;

// throw: produce an error
fn except_t_throw<E, M: ApplicativeSt, A>(e: E) -> M::Of<Result<A, E>>;

// catch: handle an error with a recovery function
fn except_t_catch<E, M: ChainSt + ApplicativeSt, A>(fa, handler) -> M::Of<Result<A, E>>;
```

#### Examples

``` rust
use karpal_effect::except_t::*;
use karpal_core::hkt::OptionF;

// Success path
let val = except_t_pure::<&str, OptionF, _>(10);
let result = except_t_chain::<&str, OptionF, _, _>(
    val, |x| Some(Ok(x + 5))
);
assert_eq!(result, Some(Ok(15)));

// Error short-circuit
let err: Option<Result<i32, &str>> = Some(Err("fail"));
let result = except_t_chain::<&str, OptionF, _, _>(
    err, |x| Some(Ok(x + 10))
);
assert_eq!(result, Some(Err("fail")));

// Error recovery
let recovered = except_t_catch::<&str, OptionF, i32>(
    Some(Err("bad")), |_| Some(Ok(42))
);
assert_eq!(recovered, Some(Ok(42)));
```


### WriterTF\<W, M\>

Adds log accumulation to an inner monad. The log type must be a Monoid.


#### Representation

``` rust
pub struct WriterTF<W, M>(PhantomData<(W, M)>);

// WriterTF<W, M>::Of<A> = M::Of<(A, W)>
impl<W: 'static, M: HKT> HKT for WriterTF<W, M> {
    type Of<A> = M::Of<(A, W)>;
}
```

Like ExceptT, the representation is a direct wrapper — no closures. The log `W` is paired with the value inside the inner monad. Logs are combined using `Semigroup::combine` when chaining.

#### Trait Implementations

| Trait           | Bounds on `W` / `M`                              |
|-----------------|--------------------------------------------------|
| `FunctorSt`     | `M: FunctorSt`                                   |
| `ApplicativeSt` | `W: Monoid`, `M: ApplicativeSt`                  |
| `ChainSt`       | `W: Semigroup + Clone`, `M: ChainSt + FunctorSt` |
| `MonadTrans<M>` | `W: Monoid`, `M: FunctorSt`                      |

#### Operations

``` rust
fn writer_t_pure<W: Monoid, M: ApplicativeSt, A>(a: A) -> M::Of<(A, W)>;
fn writer_t_tell<W, M: ApplicativeSt>(w: W) -> M::Of<((), W)>;
fn writer_t_listen<W: Clone, M: FunctorSt, A>(fa) -> M::Of<((A, W), W)>;
fn writer_t_pass<W, M: FunctorSt, A>(fa) -> M::Of<(A, W)>;
```

#### Examples

``` rust
use karpal_effect::writer_t::*;
use karpal_core::hkt::OptionF;

// tell appends to the log
let told = writer_t_tell::<String, OptionF>("hello".to_string());
assert_eq!(told, Some(((), "hello".to_string())));

// chain accumulates logs via Semigroup::combine
let m1 = writer_t_tell::<String, OptionF>("a".to_string());
let result = writer_t_chain::<String, OptionF, _, _>(m1, |()| {
    writer_t_tell::<String, OptionF>("b".to_string())
});
assert_eq!(result, Some(((), "ab".to_string())));

// listen exposes the log alongside the value
let val: Option<(i32, String)> = Some((42, "log".to_string()));
let listened = writer_t_listen::<String, OptionF, i32>(val);
assert_eq!(listened, Some(((42, "log".to_string()), "log".to_string())));
```


### ReaderTF\<E, M\>

Adds a shared, read-only environment to an inner monad.


#### Representation

``` rust
pub struct ReaderTF<E, M>(PhantomData<(E, M)>);

// ReaderTF<E, M>::Of<A> = Box<dyn Fn(E) -> M::Of<A>>
impl<E: 'static, M: HKT + 'static> HKT for ReaderTF<E, M> {
    type Of<A> = Box<dyn Fn(E) -> M::Of<A>>;
}
```

ReaderT wraps a function from environment to inner monad. The environment is **shared** (not threaded) — each chained computation receives the same environment value.

#### Trait Implementations

| Trait           | Bounds                             |
|-----------------|------------------------------------|
| `FunctorSt`     | `M: FunctorSt + 'static`           |
| `ChainSt`       | `E: Clone`, `M: ChainSt + 'static` |
| `MonadTrans<M>` | `M: FunctorSt + 'static`           |

**Note:** `ApplicativeSt` is **not** implemented for `ReaderTF`. The trait's `pure_st` method cannot produce a `Box<dyn Fn(E) -> M::Of<A>>` without being able to clone `A`. Adding a blanket `A: Clone` bound to `ApplicativeSt::pure_st` would impose that requirement on *every* `ApplicativeSt` implementation (including `ExceptTF`), preventing its use with non-`Clone` values. Instead, use the standalone `reader_t_pure` function when you specifically need a `Clone` `A`; it requires `A: Clone` explicitly.

#### Operations

``` rust
fn reader_t_pure<E, M: ApplicativeSt, A: Clone>(a: A) -> Box<dyn Fn(E) -> M::Of<A>>;
fn reader_t_ask<E: Clone, M: ApplicativeSt>() -> Box<dyn Fn(E) -> M::Of<E>>;
fn reader_t_local<E, M: HKT, A>(f: impl Fn(E) -> E, reader) -> Box<dyn Fn(E) -> M::Of<A>>;
fn reader_t_reader<E, M: ApplicativeSt, A>(f: impl Fn(E) -> A) -> Box<dyn Fn(E) -> M::Of<A>>;
fn reader_t_run<E, M: HKT, A>(reader, env: E) -> M::Of<A>;
```

#### Examples

``` rust
use karpal_effect::reader_t::*;
use karpal_core::hkt::OptionF;

// ask: read the environment
let r = reader_t_ask::<i32, OptionF>();
assert_eq!(r(42), Some(42));

// chain shares the environment between computations
let r = reader_t_chain::<i32, OptionF, _, _>(
    reader_t_ask::<i32, OptionF>(),
    |x| {
        let x_captured = x;
        reader_t_fmap::<i32, OptionF, _, _>(
            reader_t_ask::<i32, OptionF>(),
            move |e| e + x_captured,
        )
    },
);
assert_eq!(r(10), Some(20));  // 10 + 10

// local: modify the environment for a sub-computation
let r = reader_t_ask::<i32, OptionF>();
let localized = reader_t_local::<i32, OptionF, i32>(|e| e + 100, r);
assert_eq!(localized(5), Some(105));
```


### StateTF\<S, M\>

Adds mutable state to an inner monad. State is threaded through computations.


#### Representation

``` rust
pub struct StateTF<S, M>(PhantomData<(S, M)>);

// StateTF<S, M>::Of<A> = Box<dyn Fn(S) -> M::Of<(S, A)>>
impl<S: 'static, M: HKT + 'static> HKT for StateTF<S, M> {
    type Of<A> = Box<dyn Fn(S) -> M::Of<(S, A)>>;
}
```

Unlike ReaderT, the state is **threaded** (modified) — each chained computation receives the updated state from the previous one. The output includes both the new state and the result.

#### Trait Implementations

| Trait           | Bounds                               |
|-----------------|--------------------------------------|
| `FunctorSt`     | `M: FunctorSt + 'static`             |
| `ChainSt`       | `S: Clone`, `M: ChainSt + 'static`   |
| `MonadTrans<M>` | `S: Clone`, `M: FunctorSt + 'static` |

Like ReaderT, `ApplicativeSt` is **not** implemented — use the standalone `state_t_pure` function (which requires `A: Clone`).

#### Operations

``` rust
fn state_t_pure<S: Clone, M: ApplicativeSt, A: Clone>(a: A) -> Box<dyn Fn(S) -> M::Of<(S, A)>>;
fn state_t_get<S: Clone, M: ApplicativeSt>() -> Box<dyn Fn(S) -> M::Of<(S, S)>>;
fn state_t_put<S: Clone, M: ApplicativeSt>(new_state: S) -> Box<dyn Fn(S) -> M::Of<(S, ())>>;
fn state_t_modify<S: Clone, M: ApplicativeSt>(f: impl Fn(S) -> S) -> Box<dyn Fn(S) -> M::Of<(S, ())>>;
fn state_t_run<S, M: HKT, A>(state, initial: S) -> M::Of<(S, A)>;
```

#### Examples

``` rust
use karpal_effect::state_t::*;
use karpal_core::hkt::OptionF;

// get reads the current state
let g = state_t_get::<i32, OptionF>();
assert_eq!(g(42), Some((42, 42)));

// put replaces the state
let p = state_t_put::<i32, OptionF>(99);
assert_eq!(p(0), Some((99, ())));

// chain threads state: get 10, modify +10, get 20
let program = state_t_chain::<i32, OptionF, _, _>(
    state_t_get::<i32, OptionF>(),
    |x| state_t_chain::<i32, OptionF, _, _>(
        state_t_modify::<i32, OptionF>(move |s| s + x),
        |_| state_t_get::<i32, OptionF>(),
    ),
);
assert_eq!(program(10), Some((20, 20)));

// Inner monad can short-circuit (OptionF with None)
let guarded = state_t_chain::<i32, OptionF, _, _>(
    state_t_get::<i32, OptionF>(),
    |x| -> Box<dyn Fn(i32) -> Option<(i32, i32)>> {
        if x > 100 {
            state_t_pure::<i32, OptionF, _>(x)
        } else {
            Box::new(|_| None)
        }
    },
);
assert_eq!(guarded(10), None);
assert_eq!(guarded(200), Some((200, 200)));
```


## Reader vs State

|                    | ReaderT                                      | StateT                                       |
|--------------------|----------------------------------------------|----------------------------------------------|
| Environment        | Shared (read-only)                           | Threaded (mutable)                           |
| Chain semantics    | Both computations see the same `E`           | Second sees updated `S` from first           |
| `ask` / `get`      | Always returns the original environment      | Returns current (potentially modified) state |
| `local` / `modify` | Scoped change (only affects sub-computation) | Permanent change (visible to all subsequent) |

## Design Notes

- **Why separate `FunctorSt` / `ChainSt` traits?** — Rust's `Box<dyn Fn>` requires `'static` bounds on captured types. Adding `'static` to the main `Functor` trait would unnecessarily constrain non-transformer code. The `St` family provides a parallel hierarchy that coexists cleanly.
- **Why no `ApplicativeSt` for ReaderT and StateT?** — `pure_st` must produce a `Box<dyn Fn(E) -> M::Of<A>>` from a single `A`. The closure may be called multiple times, so `A` must be cloneable. But adding `A: Clone` to the trait would impose that requirement globally on every `ApplicativeSt` implementation, preventing use with non-`Clone` values. The solution: standalone `reader_t_pure` / `state_t_pure` functions with explicit `A: Clone` bounds.
- **Why `M: 'static` on closure-based transformers?** — `Box<dyn Fn(E) -> M::Of<A>>` has an implicit `'static` lifetime bound, which propagates to `M::Of<A>`. Adding `M: 'static` to the HKT impl ensures the associated type satisfies this bound.
- **Rc for closure sharing** — `reader_t_fmap` and `reader_t_chain` wrap the user-provided function in `Rc` because the outer closure (which is `Fn`, not `FnOnce`) may be called multiple times, and each call creates an inner closure that needs its own reference.
- **`no_std` compatible** — the `MonadTrans` trait and its definition work in `no_std`. The transformers themselves require `alloc` (for `Box` and `Rc`).


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



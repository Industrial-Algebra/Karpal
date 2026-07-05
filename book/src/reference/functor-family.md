# Functor Family

The covariant functor hierarchy: Functor through Monad.

## Hierarchy

The Functor family forms a linear chain of increasingly powerful abstractions. Each trait extends the one above it:

``` rust
Functor           // fmap: lift A -> B into F<A> -> F<B>
  |
  v
Apply             // ap: apply F<A -> B> to F<A>, producing F<B>
  |
  v
Applicative       // pure: lift a value A into F<A>
  |
  +--- Chain      // chain: monadic bind (flatMap)
  |      |
  v      v
  Monad           // Applicative + Chain (blanket impl, no extra methods)
```

`Monad` is provided as a blanket implementation: any type that implements both `Applicative` and `Chain` automatically implements `Monad`. There is no need to write an explicit `impl Monad for ...` block.


### Functor

Covariant functor: lifts a function `A -> B` into `F<A> -> F<B>`.


#### Signature

``` rust
pub trait Functor: HKT {
    fn fmap<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> B) -> Self::Of<B>;
}
```

#### Laws


**Identity:** Mapping the identity function is a no-op.  
`F::fmap(fa, |x| x) == fa`


**Composition:** Mapping two functions sequentially is the same as mapping their composition.  
`F::fmap(F::fmap(fa, f), g) == F::fmap(fa, |x| g(f(x)))`


#### Instances

| Type constructor | Notes                                              |
|------------------|----------------------------------------------------|
| `OptionF`        | Delegates to `Option::map`                         |
| `ResultF<E>`     | Delegates to `Result::map`                         |
| `VecF`           | Requires `alloc` or `std` feature                  |
| `IdentityF`      | Applies `f` directly: `f(fa)`                      |
| `NonEmptyVecF`   | Requires `alloc` or `std` feature                  |
| `EnvF<E>`        | Maps over the second element of the tuple `(E, A)` |

#### Example

``` rust
use karpal_std::prelude::*;

// Concrete usage
let doubled = OptionF::fmap(Some(5), |x| x * 2);
assert_eq!(doubled, Some(10));

let lengths = VecF::fmap(vec!["hello", "world"], |s| s.len());
assert_eq!(lengths, vec![5, 5]);

// Generic over any Functor
fn increment<F: Functor>(fa: F::Of<i32>) -> F::Of<i32> {
    F::fmap(fa, |x| x + 1)
}

assert_eq!(increment::<OptionF>(Some(9)), Some(10));
assert_eq!(increment::<VecF>(vec![1, 2]), vec![2, 3]);
```


### Apply

A Functor that can apply a wrapped function to a wrapped value.


#### Signature

``` rust
pub trait Apply: Functor {
    fn ap<A, B, F>(ff: Self::Of<F>, fa: Self::Of<A>) -> Self::Of<B>
    where
        A: Clone,
        F: Fn(A) -> B;
}
```

The `A: Clone` bound is required because some instances (such as `VecF`) apply multiple functions to each value, consuming the value more than once.

#### Laws


**Associative composition:** Applying composed functions is the same as composing applications.  
`ap(ap(fmap(compose, f), g), x) == ap(f, ap(g, x))`


#### Instances

| Type constructor | Notes                                                  |
|------------------|--------------------------------------------------------|
| `OptionF`        | Applies function if both are `Some`; otherwise `None`  |
| `ResultF<E>`     | Applies function if both are `Ok`; first `Err` wins    |
| `VecF`           | Cartesian product: each function applied to each value |
| `IdentityF`      | Direct application: `ff(fa)`                           |
| `NonEmptyVecF`   | Cartesian product (requires `alloc` or `std`)          |

#### Example

``` rust
use karpal_std::prelude::*;

// Apply a wrapped function to a wrapped value
let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
let result = OptionF::ap(f, Some(21));
assert_eq!(result, Some(42));

// Vec: cartesian product of functions and values
let fs: Vec<fn(i32) -> i32> = vec![|x| x + 1, |x| x * 10];
let result = VecF::ap(fs, vec![1, 2, 3]);
assert_eq!(result, vec![2, 3, 4, 10, 20, 30]);
```


### Applicative

An Apply that can lift a pure value into the functor.


#### Signature

``` rust
pub trait Applicative: Apply {
    fn pure<A>(a: A) -> Self::Of<A>;
}
```

#### Laws


**Identity:** Applying a pure identity function is a no-op.  
`ap(pure(id), v) == v`


**Homomorphism:** Lifting a function and a value, then applying, is the same as lifting the result directly.  
`ap(pure(f), pure(x)) == pure(f(x))`


**Interchange:** The order of lifting does not matter when the value is pure.  
`ap(u, pure(y)) == ap(pure(|f| f(y)), u)`


#### Instances

| Type constructor | `pure(a)` returns           |
|------------------|-----------------------------|
| `OptionF`        | `Some(a)`                   |
| `ResultF<E>`     | `Ok(a)`                     |
| `VecF`           | `vec![a]`                   |
| `IdentityF`      | `a`                         |
| `NonEmptyVecF`   | `NonEmptyVec::singleton(a)` |

#### Example

``` rust
use karpal_std::prelude::*;

// Lift a value into any Applicative context
let opt: Option<i32> = OptionF::pure(42);
assert_eq!(opt, Some(42));

let v: Vec<i32> = VecF::pure(42);
assert_eq!(v, vec![42]);

// Generic lifting
fn wrap<F: Applicative>(x: i32) -> F::Of<i32> {
    F::pure(x)
}

assert_eq!(wrap::<OptionF>(7), Some(7));
assert_eq!(wrap::<VecF>(7), vec![7]);
```


### Chain

An Apply with monadic bind (flatMap). Enables sequential computations where each step depends on the previous result.


#### Signature

``` rust
pub trait Chain: Apply {
    fn chain<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> Self::Of<B>) -> Self::Of<B>;
}
```

Note that the function `f` returns `Self::Of<B>`, not just `B`. This is what distinguishes `chain` from `fmap`: the callback itself produces a wrapped value, and `chain` flattens the result.

#### Laws


**Associativity:** Chaining is associative -- nesting does not matter.  
`chain(chain(m, f), g) == chain(m, |x| chain(f(x), g))`


#### Instances

| Type constructor | Notes                                                             |
|------------------|-------------------------------------------------------------------|
| `OptionF`        | Delegates to `Option::and_then`                                   |
| `ResultF<E>`     | Delegates to `Result::and_then`                                   |
| `VecF`           | `flat_map`: each element produces a Vec, results are concatenated |
| `IdentityF`      | Direct application: `f(fa)`                                       |
| `NonEmptyVecF`   | Concatenates non-empty results (requires `alloc` or `std`)        |

#### Example

``` rust
use karpal_std::prelude::*;

// Option: short-circuits on None
fn safe_sqrt(x: f64) -> Option<f64> {
    if x >= 0.0 { Some(x.sqrt()) } else { None }
}

let result = OptionF::chain(Some(16.0), safe_sqrt);
assert_eq!(result, Some(4.0));

let result = OptionF::chain(Some(-1.0), safe_sqrt);
assert_eq!(result, None);

// Vec: flatMap (each element expands into a list)
let result = VecF::chain(vec![1, 2, 3], |x| vec![x, x * 10]);
assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
```


### Monad

Applicative + Chain. A blanket implementation with no extra methods.


#### Signature

``` rust
pub trait Monad: Applicative + Chain {}

impl<F: Applicative + Chain> Monad for F {}
```

`Monad` is a **marker trait**. It adds no new methods; it simply certifies that a type implements both `Applicative` (for `pure`) and `Chain` (for `chain`). The blanket `impl` means you never write `impl Monad for MyType` -- just implement `Applicative` and `Chain`, and `Monad` comes for free.

#### Laws

In addition to the Applicative and Chain laws, a Monad must satisfy:


**Left identity:** Lifting a value with `pure` then chaining is the same as calling the function directly.  
`chain(pure(a), f) == f(a)`


**Right identity:** Chaining with `pure` is a no-op.  
`chain(m, pure) == m`


#### Instances

Every type that implements both `Applicative` and `Chain` is automatically a `Monad`:

| Type constructor | Notes                                    |
|------------------|------------------------------------------|
| `OptionF`        | Blanket impl                             |
| `ResultF<E>`     | Blanket impl                             |
| `VecF`           | Blanket impl (requires `alloc` or `std`) |
| `IdentityF`      | Blanket impl                             |
| `NonEmptyVecF`   | Blanket impl (requires `alloc` or `std`) |

#### Example

``` rust
use karpal_std::prelude::*;

// Use Monad as a trait bound to require both pure and chain
fn bind_and_wrap<M: Monad>(x: i32) -> M::Of<String>
where
    M::Of<i32>: Clone,
{
    M::chain(M::pure(x), |n| M::pure(format!("value: {}", n)))
}

assert_eq!(bind_and_wrap::<OptionF>(42), Some("value: 42".to_string()));

// The do_! macro desugars into chain calls, so it requires Monad
let result = do_! { OptionF;
    x = Some(10);
    y = Some(x + 20);
    Some(x + y)
};
assert_eq!(result, Some(40));
```


## See Also

- [**Alt Family**](alt-family.md) -- the Alt / Plus / Alternative branch, which extends Functor in a different direction (choice and failure).
- [**Macros**](macros.md) -- the `do_!` and `ado_!` macros that provide ergonomic syntax for Chain and Applicative computations.
- [**Foldable & Traversable**](foldable-traversable.md) -- folding and traversing structures, which combine naturally with Applicative.
- [**Getting Started**](../getting-started.md) -- a tutorial introduction to HKTs, Functor, and monadic notation.


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



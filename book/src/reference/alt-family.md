# Alt Family

Fallback and choice combinators: Alt, Plus, Alternative.

## Hierarchy


Functor  →  Alt  →  Plus  →  (+ Applicative) →  Alternative  (blanket)


The Alt branch of the functor hierarchy provides combinators for expressing fallback and choice. `Alt` gives an associative choice operation, `Plus` adds an identity element (zero/empty), and `Alternative` combines `Plus` with `Applicative` via a blanket impl.

## Alt


### Alt


A Functor with an associative choice operation.


#### Signature

``` rust
pub trait Alt: Functor {
    fn alt<A>(fa1: Self::Of<A>, fa2: Self::Of<A>) -> Self::Of<A>;
}
```

`alt` takes two values of the same functor type and returns one, preferring the first when both are "successful." The exact semantics depend on the instance: for `OptionF` it is `.or()`, for `VecF` it is concatenation.

#### Laws


Associativity


alt(alt(a, b), c) == alt(a, alt(b, c))


Distributivity


fmap(f, alt(a, b)) == alt(fmap(f, a), fmap(f, b))


#### Instances

| Type constructor | Behaviour of `alt`                                                                |
|------------------|-----------------------------------------------------------------------------------|
| `OptionF`        | `fa1.or(fa2)` — returns the first `Some`, or `None` if both are `None`            |
| `ResultF<E>`     | `fa1.or(fa2)` — returns the first `Ok`, or the second value if the first is `Err` |
| `VecF`           | Concatenation — extends `fa1` with all elements of `fa2`                          |
| `NonEmptyVecF`   | Concatenation — appends the head and tail of `fa2` onto `fa1`                     |

`VecF` and `NonEmptyVecF` require the `alloc` or `std` feature.

#### Example

``` rust
use karpal_std::prelude::*;

// Fallback: try the first source, fall back to the second
let primary: Option<i32> = None;
let fallback: Option<i32> = Some(42);

let result = OptionF::alt(primary, fallback);
assert_eq!(result, Some(42));

// When both are present, the first wins
let result = OptionF::alt(Some(1), Some(2));
assert_eq!(result, Some(1));

// Vec: concatenation
let combined = VecF::alt(vec![1, 2], vec![3, 4]);
assert_eq!(combined, vec![1, 2, 3, 4]);
```


## Plus


### Plus


An Alt with a zero/empty element.


#### Signature

``` rust
pub trait Plus: Alt {
    fn zero<A>() -> Self::Of<A>;
}
```

`zero` produces the identity element for `alt`. Combined with the Alt laws, this gives a monoid structure over the functor type.

#### Laws


Left identity


alt(zero(), a) == a


Right identity


alt(a, zero()) == a


Annihilation


fmap(f, zero()) == zero()


#### Instances

| Type constructor | `zero()` returns            |
|------------------|-----------------------------|
| `OptionF`        | `None`                      |
| `VecF`           | `Vec::new()` (empty vector) |

`ResultF<E>` does **not** implement `Plus` because there is no way to produce a `Result<A, E>` without an `E` value. `NonEmptyVecF` also lacks an instance because a non-empty vector cannot be empty by definition.

`VecF` requires the `alloc` or `std` feature.

#### Example

``` rust
use karpal_std::prelude::*;

// zero() for Option is None
let empty: Option<i32> = OptionF::zero();
assert_eq!(empty, None);

// zero() for Vec is an empty vector
let empty_vec: Vec<i32> = VecF::zero();
assert_eq!(empty_vec, Vec::<i32>::new());

// Left identity: alt(zero(), a) == a
let a = Some(10);
assert_eq!(OptionF::alt(OptionF::zero(), a), a);

// Right identity: alt(a, zero()) == a
assert_eq!(OptionF::alt(a, OptionF::zero()), a);
```


## Alternative


### Alternative


Applicative + Plus with no extra methods (blanket impl).


#### Signature

``` rust
pub trait Alternative: Applicative + Plus {}

impl<F: Applicative + Plus> Alternative for F {}
```

`Alternative` is a marker trait that combines `Applicative` and `Plus`. It introduces no new methods — any type that implements both `Applicative` and `Plus` automatically implements `Alternative` via the blanket impl.

#### Laws

Alternative inherits all laws from Alt, Plus, and Applicative, and adds two of its own:


Distributivity


ap(alt(f, g), x) == alt(ap(f, x), ap(g, x))


Annihilation


ap(zero(), x) == zero()


#### Instances

| Type constructor | Notes                                                                                                            |
|------------------|------------------------------------------------------------------------------------------------------------------|
| `OptionF`        | Implements both `Applicative` and `Plus`, so `Alternative` is provided automatically                             |
| `VecF`           | Implements both `Applicative` and `Plus`, so `Alternative` is provided automatically (requires `alloc` or `std`) |

#### Example

``` rust
use karpal_std::prelude::*;

// Alternative lets you combine choice (Alt/Plus) with
// applicative computation (Applicative).

// Distributivity: ap(alt(f, g), x) == alt(ap(f, x), ap(g, x))
let f: Option<fn(i32) -> i32> = Some(|a| a + 1);
let g: Option<fn(i32) -> i32> = Some(|a| a * 2);
let x = Some(10);

let left  = OptionF::ap(OptionF::alt(f, g), x);
let right = OptionF::alt(OptionF::ap(f, x), OptionF::ap(g, x));
assert_eq!(left, right);  // Both are Some(11)

// Annihilation: ap(zero(), x) == zero()
let no_fn: Option<fn(i32) -> i32> = OptionF::zero();
let result = OptionF::ap(no_fn, Some(5));
assert_eq!(result, None);
```


## See Also

- [**Functor Family**](functor-family.md) — the Functor → Apply → Applicative → Chain → Monad branch that Alt builds upon.
- [**Semigroup & Monoid**](algebraic.md) — the value-level analogue: Semigroup provides an associative `combine`, Monoid adds an `empty` identity, mirroring the Alt/Plus relationship at the functor level.
- [**Foldable & Traversable**](foldable-traversable.md) — traits for collapsing and sequencing containers, which compose naturally with Alt and Plus.


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



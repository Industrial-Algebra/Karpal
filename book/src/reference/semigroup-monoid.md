# Semigroup & Monoid

Algebraic typeclasses for combining values.

Semigroup and Monoid are the foundational algebraic abstractions in Karpal. A `Semigroup` provides an associative binary operation for combining two values of the same type. A `Monoid` extends `Semigroup` with an identity element, enabling operations like folding an empty collection to a default value.


### Semigroup

A type with an associative binary operation.


#### Signature

``` rust
/// A type with an associative binary operation.
pub trait Semigroup {
    fn combine(self, other: Self) -> Self;
}
```

The `combine` method takes ownership of both values and produces a new value of the same type. Because it consumes `self`, there is no hidden aliasing -- the implementation is free to reuse allocations (and Karpal's `String` and `Vec` implementations do exactly that).

#### Laws


Associativity

For all `a`, `b`, `c` of type `T: Semigroup`:

``` rust
a.combine(b).combine(c) == a.combine(b.combine(c))
```

The grouping of operations does not matter. This is the only law a `Semigroup` must satisfy.


#### Instances

| Type                              | Behavior of `combine`                                                     | Feature gate     |
|-----------------------------------|---------------------------------------------------------------------------|------------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | Addition (`self + other`)                                                 | none (`no_std`)  |
| `u8`, `u16`, `u32`, `u64`, `u128` | Addition (`self + other`)                                                 | none (`no_std`)  |
| `f32`, `f64`                      | Addition (`self + other`)                                                 | none (`no_std`)  |
| `String`                          | Concatenation (`push_str`)                                                | `std` or `alloc` |
| `Vec<T>`                          | Concatenation (`extend`)                                                  | `std` or `alloc` |
| `Option<T: Semigroup>`            | Combines inner values if both are `Some`; keeps the `Some` side otherwise | none (`no_std`)  |
| `NonEmptyVec<T>`                  | Concatenation (head + tails merged)                                       | `std` or `alloc` |

#### Examples

``` rust
use karpal_core::semigroup::Semigroup;

// Numeric addition
assert_eq!(3i32.combine(4), 7);

// String concatenation
assert_eq!(
    "hello ".to_string().combine("world".to_string()),
    "hello world"
);

// Vec concatenation
assert_eq!(vec![1, 2].combine(vec![3, 4]), vec![1, 2, 3, 4]);

// Option lifts the inner Semigroup
assert_eq!(Some(3i32).combine(Some(4)), Some(7));
assert_eq!(Some(3i32).combine(None), Some(3));
assert_eq!(None::<i32>.combine(Some(4)), Some(4));
```


### Monoid

A Semigroup with an identity element.


#### Signature

``` rust
use crate::semigroup::Semigroup;

/// A `Semigroup` with an identity element.
pub trait Monoid: Semigroup {
    fn empty() -> Self;
}
```

The `empty` method returns the identity element for the type's `combine` operation. Combining any value with `empty()` (on either side) must return that value unchanged.

#### Laws


Left Identity

For all `a` of type `T: Monoid`:

``` rust
T::empty().combine(a) == a
```


Right Identity

For all `a` of type `T: Monoid`:

``` rust
a.combine(T::empty()) == a
```


Together with the `Semigroup` associativity law, these two laws make `(T, combine, empty)` a monoid in the algebraic sense.

#### Instances

| Type                              | `empty()` value                | Feature gate     |
|-----------------------------------|--------------------------------|------------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | `0`                            | none (`no_std`)  |
| `u8`, `u16`, `u32`, `u64`, `u128` | `0`                            | none (`no_std`)  |
| `f32`, `f64`                      | `0.0`                          | none (`no_std`)  |
| `String`                          | `String::new()` (empty string) | `std` or `alloc` |
| `Vec<T>`                          | `Vec::new()` (empty vec)       | `std` or `alloc` |
| `Option<T: Semigroup>`            | `None`                         | none (`no_std`)  |

Note that `NonEmptyVec<T>` implements `Semigroup` but **not** `Monoid` -- by definition it always contains at least one element, so there is no valid identity value.

#### Examples

``` rust
use karpal_core::semigroup::Semigroup;
use karpal_core::monoid::Monoid;

// Numeric identity
assert_eq!(i32::empty(), 0);
assert_eq!(i32::empty().combine(42), 42);
assert_eq!(42i32.combine(i32::empty()), 42);

// String identity
assert_eq!(String::empty(), "");

// Vec identity
assert_eq!(Vec::<i32>::empty(), Vec::<i32>::new());

// Option identity
assert_eq!(Option::<i32>::empty(), None);
```


## Foldable and Monoid

The `Monoid` trait plays a central role in the [Foldable](foldable-traversable.md) typeclass. `Foldable` defines `fold_map`, which maps each element of a structure through a function that returns a `Monoid`, then combines all the results using `combine` and `empty`:

``` rust
pub trait Foldable: HKT {
    fn fold_right<A, B>(fa: Self::Of<A>, init: B, f: impl Fn(A, B) -> B) -> B;

    fn fold_map<A, M: Monoid>(fa: Self::Of<A>, f: impl Fn(A) -> M) -> M {
        Self::fold_right(fa, M::empty(), |a, acc| f(a).combine(acc))
    }
}
```

The default implementation of `fold_map` starts with `M::empty()` as the initial accumulator and folds right, combining each mapped element with the accumulator. Because `Monoid` guarantees associativity and identity, the result is well-defined regardless of the folding direction.

#### Example: summing a collection

``` rust
use karpal_core::prelude::*;

// fold_map with the identity function sums the elements,
// because i32's Semigroup instance uses addition.
let total = VecF::fold_map(vec![1, 2, 3], |a: i32| a);
assert_eq!(total, 6);
```

#### Example: collecting strings

``` rust
use karpal_core::prelude::*;

// Map each number to its string representation, then combine.
// String's Semigroup concatenates, and its Monoid starts from "".
let result = VecF::fold_map(vec![1, 2, 3], |a: i32| a.to_string());
assert_eq!(result, "123".to_string());
```

This pattern -- map then combine -- is the essence of `fold_map` and is the reason `Monoid` is so important in functional programming. Any time you need to reduce a collection to a single summary value, `Monoid` provides the structure to do it generically.


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



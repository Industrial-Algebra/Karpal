# Abstract Algebra

Higher algebraic structures built on top of [Semigroup & Monoid](algebraic.md). These traits live in the `karpal-algebra` crate and extend the hierarchy with groups, rings, fields, lattices, and vector spaces.

## Overview

| Trait            | Extends        | Key idea                                                         |
|------------------|----------------|------------------------------------------------------------------|
| `Group`          | `Monoid`       | Every element has an inverse: `a.combine(a.invert()) == empty()` |
| `AbelianGroup`   | `Group`        | Marker — operation is commutative                                |
| `Semiring`       | (independent)  | Two operations (add/mul) with distribution and annihilation      |
| `Ring`           | `Semiring`     | Additive inverses: `a.add(a.negate()) == zero()`                 |
| `Field`          | `Ring`         | Multiplicative inverses for non-zero elements                    |
| `Lattice`        | (independent)  | Join (supremum) and meet (infimum) with absorption               |
| `BoundedLattice` | `Lattice`      | Top and bottom elements                                          |
| `Module<R>`      | `AbelianGroup` | Scalar multiplication over a ring                                |
| `VectorSpace<F>` | `Module<F>`    | Module over a field                                              |

## Trait Hierarchy

``` text
Semigroup (karpal-core)         Semiring (independent)      Lattice (independent)
  |                               |                           |
Monoid (karpal-core)            Ring                        BoundedLattice
  |                               |
Group                           Field
  |
AbelianGroup (marker)           Module<R: Ring>: AbelianGroup
                                  |
                                VectorSpace<F: Field>
```

The `Semigroup → Monoid → Group` chain extends the existing `karpal-core` hierarchy. `Semiring` and `Lattice` are independent hierarchies — they define their own operations to avoid the "which operation is the semigroup?" ambiguity.

## Newtype Wrappers

The default `Semigroup` for numeric types uses addition. Newtype wrappers in `karpal-core` let you select a different combining strategy. This is the standard approach to the "a type can be a monoid in multiple ways" problem.


### Sum & Product

Select additive or multiplicative combining.


#### Definition

``` rust
pub struct Sum<T>(pub T);      // Semigroup: +, Monoid: 0
pub struct Product<T>(pub T);  // Semigroup: *, Monoid: 1
```

#### Examples

``` rust
use karpal_core::{Sum, Product, Semigroup, Monoid};
use karpal_core::Foldable;
use karpal_core::hkt::VecF;

// Sum uses addition
assert_eq!(Sum(3i32).combine(Sum(4)), Sum(7));
assert_eq!(Sum::<i32>::empty(), Sum(0));

// Product uses multiplication
assert_eq!(Product(3i32).combine(Product(4)), Product(12));
assert_eq!(Product::<i32>::empty(), Product(1));

// Fold a list as a product instead of a sum
let product = VecF::fold_map(vec![1, 2, 3, 4], |x| Product(x));
assert_eq!(product, Product(24));
```

#### Instances

| Wrapper           | Semigroup          | Monoid `empty()`              |
|-------------------|--------------------|-------------------------------|
| `Sum<T: Add>`     | `self.0 + other.0` | `Sum(0)` / `Sum(0.0)`         |
| `Product<T: Mul>` | `self.0 * other.0` | `Product(1)` / `Product(1.0)` |


### Min & Max

Select minimum or maximum combining for ordered types.


#### Definition

``` rust
pub struct Min<T>(pub T);  // Semigroup: min, Monoid: T::MAX
pub struct Max<T>(pub T);  // Semigroup: max, Monoid: T::MIN
```

#### Examples

``` rust
use karpal_core::{Min, Max, Semigroup, Monoid};

assert_eq!(Min(3i32).combine(Min(7)), Min(3));
assert_eq!(Max(3i32).combine(Max(7)), Max(7));

// Monoid identity: combining with empty returns the other value
assert_eq!(Min::<i32>::empty().combine(Min(5)), Min(5));  // MAX.min(5) = 5
assert_eq!(Max::<i32>::empty().combine(Max(5)), Max(5));  // MIN.max(5) = 5
```

`Monoid` is implemented for all integer types (`i8`–`i128`, `u8`–`u128`) using `T::MAX` for `Min` and `T::MIN` for `Max`. Floats have `Semigroup` but not `Monoid` because `f64::INFINITY` is debatable and NaN breaks the identity law.


### First & Last

Select the first or last `Some` value.


#### Definition

``` rust
pub struct First<T>(pub T);  // Only Semigroup+Monoid for Option<T>
pub struct Last<T>(pub T);   // Only Semigroup+Monoid for Option<T>
```

#### Examples

``` rust
use karpal_core::{First, Last, Semigroup, Monoid};

// First picks the first Some value
assert_eq!(First(Some(1)).combine(First(Some(2))), First(Some(1)));
assert_eq!(First(None::<i32>).combine(First(Some(2))), First(Some(2)));

// Last picks the last Some value
assert_eq!(Last(Some(1)).combine(Last(Some(2))), Last(Some(2)));
assert_eq!(Last(Some(1)).combine(Last(None)), Last(Some(1)));

// Monoid identity is None (neutral under "pick the Some")
assert_eq!(First::<Option<i32>>::empty(), First(None));
```

`First` and `Last` are only implemented for `Option<T>`, matching Haskell's `Data.Monoid.First`/`Last`. A blanket `First<T>` would conflict with the `Option` specialization due to Rust's coherence rules.


## Group Hierarchy


### Group

A Monoid where every element has an inverse.


#### Signature

``` rust
pub trait Group: Monoid {
    fn invert(self) -> Self;

    // Provided: a.combine(b.invert())
    fn combine_inverse(self, other: Self) -> Self { ... }
}
```

#### Laws


Left Inverse

``` rust
a.invert().combine(a) == Self::empty()
```


Right Inverse

``` rust
a.combine(a.invert()) == Self::empty()
```


#### Instances

| Type                                | `invert`                 |
|-------------------------------------|--------------------------|
| `i8`, `i16`, `i32`, `i64`, `i128`   | Negation (`-self`)       |
| `f32`, `f64`                        | Negation (`-self`)       |
| `(A, B)` where `A: Group, B: Group` | Component-wise inversion |

Unsigned integers are not groups — they have no additive inverse.

#### Examples

``` rust
use karpal_algebra::Group;
use karpal_core::{Semigroup, Monoid};

assert_eq!(5i32.invert(), -5);
assert_eq!(5i32.combine(5i32.invert()), 0);  // right inverse
assert_eq!(10i32.combine_inverse(3), 7);      // 10 + (-3) = 7
```


### AbelianGroup

A Group whose operation is commutative. Marker trait — commutativity verified by property tests.


#### Signature

``` rust
pub trait AbelianGroup: Group {}
```


Commutativity

``` rust
a.combine(b) == b.combine(a)
```


All `Group` instances in Karpal are abelian (addition is commutative). The marker trait exists so that `Module` can require it — modules are defined over abelian groups.

#### Instances

All signed integers (`i8`–`i128`), `f32`, `f64`, and `(A, B)` where both components are `AbelianGroup`.


## Semiring / Ring / Field

These traits model the algebraic structures from abstract algebra, with two operations: addition and multiplication. They are **independent** of `Semigroup` — they define their own method names to avoid the ambiguity of which operation the semigroup uses.


### Semiring

A type with addition (commutative monoid) and multiplication (monoid), where multiplication distributes over addition.


#### Signature

``` rust
pub trait Semiring: Sized + Clone + PartialEq {
    fn zero() -> Self;
    fn one() -> Self;
    fn add(self, other: Self) -> Self;
    fn mul(self, other: Self) -> Self;
}
```

#### Laws


Additive Commutative Monoid

``` rust
a.add(b) == b.add(a)                    // commutativity
a.add(b).add(c) == a.add(b.add(c))      // associativity
Self::zero().add(a) == a                 // identity
```


Multiplicative Monoid

``` rust
a.mul(b).mul(c) == a.mul(b.mul(c))      // associativity
Self::one().mul(a) == a                  // identity
```


Distribution & Annihilation

``` rust
a.mul(b.add(c)) == a.mul(b).add(a.mul(c))  // left distribution
a.add(b).mul(c) == a.mul(c).add(b.mul(c))  // right distribution
Self::zero().mul(a) == Self::zero()         // annihilation
```


#### Instances

| Type              | `zero` / `one`   | `add` / `mul` |
|-------------------|------------------|---------------|
| All integer types | `0` / `1`        | `+` / `*`     |
| `f32`, `f64`      | `0.0` / `1.0`    | `+` / `*`     |
| `bool`            | `false` / `true` | OR / AND      |

The `bool` semiring is the classic example: OR is "addition" (with identity `false`), AND is "multiplication" (with identity `true`), AND distributes over OR, and `false && x == false` (annihilation).

#### Examples

``` rust
use karpal_algebra::Semiring;

assert_eq!(3i32.add(4), 7);
assert_eq!(3i32.mul(4), 12);
assert_eq!(i32::zero(), 0);
assert_eq!(i32::one(), 1);

// Boolean semiring
assert_eq!(false.add(true), true);   // OR
assert_eq!(true.mul(false), false);  // AND
```


### Ring

A Semiring with additive inverses (negation).


#### Signature

``` rust
pub trait Ring: Semiring {
    fn negate(self) -> Self;

    // Provided: self.add(other.negate())
    fn sub(self, other: Self) -> Self { ... }
}
```


Additive Inverse

``` rust
a.add(a.negate()) == Self::zero()
```


#### Instances

| Type                              | `negate` |
|-----------------------------------|----------|
| `i8`, `i16`, `i32`, `i64`, `i128` | `-self`  |
| `f32`, `f64`                      | `-self`  |

Unsigned integers and `bool` are **not** rings — they have no additive inverse.

#### Examples

``` rust
use karpal_algebra::{Ring, Semiring};

assert_eq!(5i32.negate(), -5);
assert_eq!(5i32.add(5i32.negate()), 0);  // additive inverse
assert_eq!(10i32.sub(3), 7);             // sub = add(negate)
```


### Field

A Ring with multiplicative inverses for all non-zero elements.


#### Signature

``` rust
pub trait Field: Ring {
    fn reciprocal(self) -> Self;

    // Provided: self.mul(other.reciprocal())
    fn div(self, other: Self) -> Self { ... }
}
```


Multiplicative Inverse (non-zero)

``` rust
// For a != zero():
a.mul(a.reciprocal()) == Self::one()
```


#### Instances

| Type  | `reciprocal` |
|-------|--------------|
| `f32` | `1.0 / self` |
| `f64` | `1.0 / self` |

Integers are not fields — `1 / 2` is not an integer. Only floating-point types have a multiplicative inverse.

#### Examples

``` rust
use karpal_algebra::{Field, Ring, Semiring};

let half = 2.0f64.reciprocal();
assert!((half - 0.5).abs() < 1e-10);

let result = 10.0f64.div(4.0);
assert!((result - 2.5).abs() < 1e-10);
```


## Lattice Hierarchy


### Lattice

A type with join (supremum) and meet (infimum) operations satisfying absorption.


#### Signature

``` rust
pub trait Lattice: Sized {
    fn join(self, other: Self) -> Self;  // supremum (least upper bound)
    fn meet(self, other: Self) -> Self;  // infimum (greatest lower bound)
}
```

#### Laws


Associativity

``` rust
a.join(b.join(c)) == a.join(b).join(c)
a.meet(b.meet(c)) == a.meet(b).meet(c)
```


Commutativity

``` rust
a.join(b) == b.join(a)
a.meet(b) == b.meet(a)
```


Idempotency

``` rust
a.join(a) == a
a.meet(a) == a
```


Absorption

``` rust
a.join(a.meet(b)) == a
a.meet(a.join(b)) == a
```


#### Instances

| Type              | `join`     | `meet`     |
|-------------------|------------|------------|
| All integer types | `max`      | `min`      |
| `bool`            | OR         | AND        |
| `f32`, `f64`      | `f64::max` | `f64::min` |

#### Examples

``` rust
use karpal_algebra::Lattice;

assert_eq!(3i32.join(5), 5);   // max
assert_eq!(3i32.meet(5), 3);   // min

// Absorption: a.join(a.meet(b)) == a
assert_eq!(3i32.join(3i32.meet(5)), 3);

// Bool lattice
assert_eq!(false.join(true), true);   // OR
assert_eq!(false.meet(true), false);  // AND
```


### BoundedLattice

A Lattice with top (greatest) and bottom (least) elements.


#### Signature

``` rust
pub trait BoundedLattice: Lattice {
    fn top() -> Self;
    fn bottom() -> Self;
}
```


Identity

``` rust
a.join(Self::bottom()) == a   // bottom is join identity
a.meet(Self::top()) == a      // top is meet identity
```


#### Instances

| Type              | `top`    | `bottom` |
|-------------------|----------|----------|
| All integer types | `T::MAX` | `T::MIN` |
| `bool`            | `true`   | `false`  |

`f32` and `f64` intentionally do **not** implement `BoundedLattice` — `INFINITY` is debatable as a top element, and `NaN` breaks the lattice laws (it is not comparable to itself).


## Module & VectorSpace


### Module\<R: Ring\>

An abelian group with scalar multiplication over a ring.


#### Signature

``` rust
pub trait Module<R: Ring>: AbelianGroup {
    fn scale(self, scalar: R) -> Self;
}
```

The ring `R` is a generic parameter, not an associated type. This allows a single vector type to be a module over different rings.

#### Laws


Module Laws

``` rust
a.scale(R::one()) == a                            // identity
a.scale(r).scale(s) == a.scale(r.mul(s))           // compatibility
a.combine(b).scale(r) == a.scale(r).combine(b.scale(r))  // distribution over group
a.scale(r.add(s)) == a.scale(r).combine(a.scale(s))      // distribution over ring
```


#### Instances

| Type     | Scalar ring | `scale`                       |
|----------|-------------|-------------------------------|
| `f32`    | `f32`       | `self * scalar`               |
| `f64`    | `f64`       | `self * scalar`               |
| `(F, F)` | `F: Field`  | Component-wise multiplication |

#### Examples

``` rust
use karpal_algebra::{Module, Semiring};
use karpal_core::Semigroup;

// Scalar field as 1D module
assert!((3.0f64.scale(2.0) - 6.0).abs() < 1e-10);

// 2D vector as module
let v = (1.0f64, 2.0).scale(3.0);
assert!((v.0 - 3.0).abs() < 1e-10);
assert!((v.1 - 6.0).abs() < 1e-10);

// Distribution: (a + b) * r == a*r + b*r
let a = (1.0f64, 2.0);
let b = (3.0, 4.0);
let left = a.combine(b).scale(2.0);
let right = a.scale(2.0).combine(b.scale(2.0));
assert!((left.0 - right.0).abs() < 1e-10);
```


### VectorSpace\<F: Field\>

A Module over a Field. Marker trait guaranteeing scalar division.


#### Signature

``` rust
pub trait VectorSpace<F: Field>: Module<F> {}
```

A vector space inherits all module laws. The key additional guarantee is that scalars form a `Field`, so scalar division is available. Any field is a one-dimensional vector space over itself.

#### Instances

`f32` over `f32`, `f64` over `f64`, and `(F, F)` over any `F: Field`.

#### Examples

``` rust
use karpal_algebra::{VectorSpace, Module, Semiring};
use karpal_core::Semigroup;

// Generic function over any vector space
fn linear_combination<V: VectorSpace<f64> + Semigroup>(
    a: V, sa: f64, b: V, sb: f64,
) -> V {
    a.scale(sa).combine(b.scale(sb))
}

// Standard basis vectors in R^2
let e1 = (1.0f64, 0.0);
let e2 = (0.0f64, 1.0);
let v = e1.scale(3.0).combine(e2.scale(4.0));
assert!((v.0 - 3.0).abs() < 1e-10);
assert!((v.1 - 4.0).abs() < 1e-10);
```


## Design Notes

- **Semiring is independent of Semigroup** — a type can be a semigroup under addition (the default) while also being a semiring with both add and mul. The newtypes (`Sum`, `Product`) solve "which is the semigroup?" for `Foldable`; the semiring/ring/field hierarchy provides both operations simultaneously.
- **Integer impls use standard operators** — matching the existing `Semigroup` pattern. Property tests use bounded ranges (`-100..100`) to avoid overflow.
- **Float equality** — all float-related property tests use epsilon tolerance (`1e-6` to `1e-10`) rather than exact equality.
- **No `BoundedLattice` for floats** — NaN is not comparable to itself, so `NaN.meet(top()) != NaN`. Rather than special-case NaN handling, we leave the impl out.
- **`no_std` compatible** — all traits in `karpal-algebra` work without `std` or `alloc`.


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



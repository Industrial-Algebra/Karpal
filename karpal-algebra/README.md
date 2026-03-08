# karpal-algebra

Abstract algebra structures for Rust: Group, Ring, Field, Lattice, Module,
and VectorSpace.

## What's inside

### Trait hierarchy

```
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

### Group hierarchy

| Trait | Extends | Key method | Instances |
|-------|---------|-----------|-----------|
| `Group` | `Monoid` | `invert(self) -> Self` | signed integers, f32, f64, (A, B) |
| `AbelianGroup` | `Group` | marker trait | all Group types |

### Semiring / Ring / Field

| Trait | Extends | Key methods | Instances |
|-------|---------|------------|-----------|
| `Semiring` | (independent) | `zero`, `one`, `add`, `mul` | all numerics, bool |
| `Ring` | `Semiring` | `negate`, `sub` | signed integers, f32, f64 |
| `Field` | `Ring` | `reciprocal`, `div` | f32, f64 |

### Lattice

| Trait | Extends | Key methods | Instances |
|-------|---------|------------|-----------|
| `Lattice` | (independent) | `join`, `meet` | all integers, bool, f32, f64 |
| `BoundedLattice` | `Lattice` | `top`, `bottom` | all integers, bool |

### Module / VectorSpace

| Trait | Extends | Key method | Instances |
|-------|---------|-----------|-----------|
| `Module<R: Ring>` | `AbelianGroup` | `scale(self, R) -> Self` | f32, f64, (F, F) |
| `VectorSpace<F: Field>` | `Module<F>` | marker trait | f32, f64, (F, F) |

### Example: Group and Ring

```rust
use karpal_algebra::{Group, Ring, Semiring};
use karpal_core::{Semigroup, Monoid};

// Group: every element has an inverse
assert_eq!(5i32.invert(), -5);
assert_eq!(5i32.combine(5i32.invert()), 0);
assert_eq!(10i32.combine_inverse(3), 7);

// Ring: two operations with additive inverse
assert_eq!(5i32.negate(), -5);
assert_eq!(10i32.sub(3), 7);
```

### Example: VectorSpace

```rust
use karpal_algebra::{VectorSpace, Module, Semiring};
use karpal_core::Semigroup;

// 2D vectors as a vector space
let e1 = (1.0f64, 0.0);
let e2 = (0.0f64, 1.0);
let v = e1.scale(3.0).combine(e2.scale(4.0));
assert!((v.0 - 3.0).abs() < 1e-10);
assert!((v.1 - 4.0).abs() < 1e-10);
```

### Newtype wrappers (in karpal-core)

Select alternative Semigroup/Monoid instances:

| Wrapper | Semigroup | Monoid `empty()` |
|---------|-----------|-----------------|
| `Sum<T>` | `+` | `Sum(0)` |
| `Product<T>` | `*` | `Product(1)` |
| `Min<T>` | `min` | `Min(T::MAX)` |
| `Max<T>` | `max` | `Max(T::MIN)` |
| `First<Option<T>>` | first `Some` | `First(None)` |
| `Last<Option<T>>` | last `Some` | `Last(None)` |

```rust
use karpal_core::{Sum, Product, Semigroup, Monoid, Foldable};
use karpal_core::hkt::VecF;

let product = VecF::fold_map(vec![1, 2, 3, 4], |x| Product(x));
assert_eq!(product, Product(24));
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Implies `alloc` |
| `alloc` | no      | Enables `alloc`-gated features |

All traits are `no_std` compatible with no feature gates.

## License

MIT OR Apache-2.0

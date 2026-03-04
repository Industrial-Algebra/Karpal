# karpal-core

Core algebraic structures for Rust: HKT encoding, a full functor hierarchy
(Functor through Monad), Semigroup, Monoid, and `do_!`/`ado_!` notation macros.

## What's inside

### HKT encoding

GAT-based Higher-Kinded Type encoding with zero dependencies:

```rust
use karpal_core::hkt::{HKT, HKT2, OptionF, ResultF, VecF, ResultBF, TupleF};

// HKT  — OptionF::Of<T> = Option<T>, ResultF<E>::Of<T> = Result<T, E>, VecF::Of<T> = Vec<T>
// HKT2 — ResultBF::P<A, B> = Result<B, A>, TupleF::P<A, B> = (A, B)
```

### Functor hierarchy

| Trait | Supertrait | Instances |
|-------|-----------|-----------|
| Functor | HKT | OptionF, ResultF, VecF |
| Apply | Functor | OptionF, ResultF, VecF |
| Applicative | Apply | OptionF, ResultF, VecF |
| Chain | Apply | OptionF, ResultF, VecF |
| Monad | Applicative + Chain | blanket impl |
| Alt | Functor | OptionF, ResultF, VecF |
| Plus | Alt | OptionF, VecF |
| Alternative | Applicative + Plus | blanket impl |
| Foldable | HKT | OptionF, ResultF, VecF |
| Traversable | Functor + Foldable | OptionF, ResultF, VecF |
| FunctorFilter | Functor | OptionF, VecF |
| Selective | Applicative | OptionF |
| Contravariant | HKT | PredicateF |
| Bifunctor | HKT2 | ResultBF, TupleF |
| NaturalTransformation | HKT | OptionToVec, VecHeadToOption |

### Functor

```rust
use karpal_core::{Functor, hkt::OptionF};

let result = OptionF::fmap(Some(21), |x| x * 2);
assert_eq!(result, Some(42));
```

### Applicative / Chain / Monad

```rust
use karpal_core::{Applicative, Chain};
use karpal_core::hkt::OptionF;

let result = OptionF::ap(Some(|x: i32| x + 1), Some(41));
assert_eq!(result, Some(42));

let result = OptionF::chain(Some(3), |x| Some(x * 2));
assert_eq!(result, Some(6));
```

### do! / ado! macros

```rust
use karpal_core::{do_, ado_};
use karpal_core::hkt::OptionF;
use karpal_core::Applicative;

let result = do_! { OptionF;
    x = Some(1);
    y = Some(x + 1);
    OptionF::pure(x + y)
};
assert_eq!(result, Some(3));

let result = ado_! { OptionF;
    x = Some(10);
    y = Some(20);
    yield x + y
};
assert_eq!(result, Some(30));
```

### Semigroup

```rust
use karpal_core::Semigroup;

assert_eq!(3i32.combine(4), 7);
assert_eq!("hello ".to_string().combine("world".to_string()), "hello world");
assert_eq!(Some(3).combine(Some(4)), Some(7));
```

Instances: all numeric types (additive), `String`, `Vec<T>`, `Option<T: Semigroup>`.

### Monoid

```rust
use karpal_core::Monoid;

assert_eq!(i32::empty(), 0);
assert_eq!(String::empty(), "");
assert_eq!(Vec::<i32>::empty(), vec![]);
```

Instances match Semigroup instances.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Enables `Vec`, `String`, `PredicateF` instances |
| `alloc` | no      | Same instances via `alloc` (for `no_std`) |

## License

MIT OR Apache-2.0

# karpal-std

Standard prelude for the Karpal ecosystem.

Re-exports the most commonly used types and traits from `karpal-core`,
`karpal-profunctor`, and `karpal-optics` for ergonomic single-import usage.

## Usage

```rust
use karpal_std::prelude::*;
```

This gives you access to:

- **HKT encoding**: `HKT`, `HKT2`, `OptionF`, `ResultF`, `VecF`, `ResultBF`, `TupleF`
- **Functor hierarchy**: `Functor`, `Apply`, `Applicative`, `Chain`, `Monad`,
  `Alt`, `Plus`, `Alternative`, `Foldable`, `Traversable`, `FunctorFilter`,
  `Selective`, `Bifunctor`, `Contravariant`, `NaturalTransformation`
- **Algebraic typeclasses**: `Semigroup`, `Monoid`
- **Profunctor**: `Profunctor`, `Strong`, `Choice`, `FnP`
- **Optics**: `Lens`, `SimpleLens`, `Prism`, `SimplePrism`, `ComposedLens`

For qualified access, the individual crates are also re-exported:

```rust
use karpal_std::karpal_core;
use karpal_std::karpal_profunctor;
use karpal_std::karpal_optics;
```

The `do_!` and `ado_!` macros are available directly:

```rust
use karpal_std::do_;
use karpal_std::ado_;
```

## License

MIT OR Apache-2.0

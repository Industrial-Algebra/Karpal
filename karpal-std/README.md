# karpal-std

Standard prelude for the Karpal ecosystem.

Re-exports the most commonly used types and traits from all Karpal crates
for ergonomic single-import usage.

## Usage

```rust
use karpal_std::prelude::*;
```

This gives you access to:

- **HKT encoding**: `HKT`, `HKT2`, `OptionF`, `ResultF`, `VecF`, `ResultBF`, `TupleF`, and more
- **Functor hierarchy**: `Functor`, `Apply`, `Applicative`, `Chain`, `Monad`,
  `Alt`, `Plus`, `Alternative`, `Foldable`, `Traversable`, `FunctorFilter`,
  `Selective`, `Bifunctor`, `Contravariant`, `NaturalTransformation`
- **Comonad hierarchy**: `Comonad`, `ComonadEnv`, `ComonadStore`, `ComonadTraced`, `Extend`
- **Algebraic typeclasses**: `Semigroup`, `Monoid`
- **Newtype wrappers**: `Sum`, `Product`, `Min`, `Max`, `First`, `Last`
- **Abstract algebra**: `Group`, `AbelianGroup`, `Semiring`, `Ring`, `Field`,
  `Lattice`, `BoundedLattice`, `Module`, `VectorSpace`
- **Adjunctions**: `Adjunction`, `ComposeF`, `End`, `Coend`, `DinaturalTransformation`
- **Effect system**: `FunctorSt`, `ApplicativeSt`, `ChainSt`, `MonadTrans`,
  `ExceptTF`, `WriterTF`, `ReaderTF`, `StateTF`
- **Proof system**: `Proven`, `Property`, `Rewrite`, `NonEmpty`, `Positive`,
  and law-verification derive macros
- **External verification**: `Obligation`, `ObligationBundle`, `Term`, `Sort`,
  `Certificate`, `Certified`, `SmtLib2`, `Lean4`, artifact layouts, configs,
  dry-run invocation plans, runner abstractions, execution results, and
  verification reports
- **Profunctor**: `Profunctor`, `Strong`, `Choice`, `Traversing`, `FnP`, `ForgetF`, `TaggedF`
- **Optics**: `Iso`, `Lens`, `Prism`, `Traversal`, `Fold`, `Getter`, `Setter`, `Review`, and composed variants
- **Arrow hierarchy**: `Semigroupoid`, `Category`, `Arrow`, `ArrowChoice`, `ArrowApply`,
  `ArrowLoop`, `ArrowZero`, `ArrowPlus`, `FnA`, `KleisliF`, `CokleisliF`
- **Free constructions**: `Free`, `Cofree`, `Freer`, `Coyoneda`, `Yoneda`, `Day`,
  `FreeAp`, `FreeAlt`, `Codensity`, `Density`, `Lan`, `Ran`
- **Recursion schemes**: `Fix`, `Mu`, `Nu`, `cata`, `ana`, `hylo`, `para`, `apo`,
  `histo`, `futu`, `zygo`, `chrono`
- **Macros**: `do_!`, `ado_!`

For qualified access, the individual crates are also re-exported:

```rust
use karpal_std::karpal_core;
use karpal_std::karpal_profunctor;
use karpal_std::karpal_optics;
use karpal_std::karpal_arrow;
use karpal_std::karpal_free;
use karpal_std::karpal_recursion;
use karpal_std::karpal_algebra;
use karpal_std::karpal_effect;
use karpal_std::karpal_proof;
use karpal_std::karpal_verify;
```

The `do_!` and `ado_!` macros are available directly:

```rust
use karpal_std::do_;
use karpal_std::ado_;
```

## License

MIT OR Apache-2.0

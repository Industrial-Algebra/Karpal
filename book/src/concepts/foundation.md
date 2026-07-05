# Mathematical Foundation

Karpal is built on category theory — the mathematics of structure and composition.

## HKT Encoding

Higher-Kinded Types (HKTs) are types that take other types as parameters: `F<A>` rather than just `A`. Rust doesn't natively support HKTs, but GATs (Generic Associated Types, stable since Rust 1.65) provide a zero-dependency encoding:

```rust
pub trait HKT {
    type Of<T>;
}
```

A marker type like `OptionF` implements `HKT` with `type Of<T> = Option<T>`. This lets us write traits that are generic over the "shape" of a container.

## The Functor Hierarchy

The core abstraction is the functor hierarchy:

```
Functor → Apply → Applicative
                 ↓
          Chain → Monad
```

Each level adds capabilities:
- **Functor**: map over a container (`fmap`)
- **Apply**: combine two containers (`ap`)
- **Applicative**: create a pure value (`pure`)
- **Chain**: sequence operations (`chain` / `bind`)
- **Monad**: full sequential computation

## Algebraic Structure

Beyond the functor hierarchy, Karpal provides algebraic typeclasses:

- **Semigroup / Monoid**: associative combine + identity
- **Group / AbelianGroup**: monoid + inverse
- **Semiring / Ring / Field**: two operations with distributivity
- **Lattice / BoundedLattice**: join + meet (poset with all suprema/infima)
- **HeytingAlgebra**: bounded lattice with implication (intuitionistic logic)

The Heyting algebra is the foundation for structured emptiness — the idea that "why something is empty" carries information.

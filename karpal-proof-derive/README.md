# karpal-proof-derive

Derive macros for algebraic law verification in the Karpal ecosystem.

This crate generates `#[cfg(test)]` property tests for algebraic laws such as:

- semigroup associativity
- monoid identity
- group inverse laws
- semiring laws
- lattice laws

It is usually consumed through `karpal-proof`'s default `derive` feature, but it can also be used directly.

## Example

```rust,ignore
use karpal_proof_derive::VerifySemigroup;

#[derive(Clone, Debug, PartialEq, VerifySemigroup)]
#[verify(strategy = "0u32..100")]
struct MyWrapper(u32);
```

## License

MIT OR Apache-2.0

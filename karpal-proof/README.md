# karpal-proof

Algebraic law witnesses, rewrite evidence, and refinement types for the Karpal ecosystem.

`karpal-proof` provides the in-Rust proof layer used by Karpal:

- `Proven<P, T>` for values paired with evidence markers
- property markers for algebraic laws
- `Rewrite` witnesses for equational reasoning
- refinement types such as `NonEmpty<T>` and `Positive<T>`
- optional derive-macro integration via `karpal-proof-derive`

## Features

- `std` *(default)*
- `alloc`
- `derive` *(default)*

## Example

```rust
use karpal_proof::{IsAssociative, Proven};

let value = unsafe { Proven::<IsAssociative, i32>::axiom(42) };
assert_eq!(*value, 42);
```

See the workspace `README.md` and `docs/reference/proof-verification.html` for the broader proof and external verification story.

## License

MIT OR Apache-2.0

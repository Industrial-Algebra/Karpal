# Installation

## Requirements

- Nightly Rust (for GAT-based HKT encoding)
- Rust 2024 edition

```sh
rustup default nightly
```

## Adding Karpal to Your Project

### Full Prelude

```toml
[dependencies]
karpal-std = "0.7"
```

```rust
use karpal_std::prelude::*;
```

### Individual Crates

```toml
[dependencies]
karpal-core = "0.7"      # HKT, Functor hierarchy, Semigroup, Monoid
karpal-optics = "0.7"     # Lens, Prism, Traversal, Fold
karpal-proof = "0.7"      # Proven<P,T>, Rewrite witnesses
karpal-verify = "0.7"     # SMT-LIB2, Lean 4, Kani verification
karpal-diagram = "0.7"    # Monoidal categories, string diagrams
karpal-higher = "0.7"     # 2-categories, enriched categories
karpal-schubert-types = "0.7"  # Schubert intersection types
```

## no_std Support

Most crates are `no_std` compatible with optional `std`/`alloc` feature gates:

```toml
[dependencies]
karpal-core = { version = "0.7", default-features = false, features = ["alloc"] }
```

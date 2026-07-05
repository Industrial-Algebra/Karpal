# Feature Flags

Each crate supports `std` and `alloc` feature gates for `no_std` compatibility.

## Default Features

By default, all crates enable `std`:

```toml
karpal-core = "0.7"  # enables std by default
```

## no_std with alloc

```toml
karpal-core = { version = "0.7", default-features = false, features = ["alloc"] }
```

## no_std without alloc

Core traits work without any allocator:

```toml
karpal-core = { version = "0.7", default-features = false }
```

## Special Features

| Crate | Feature | Effect |
|-------|---------|--------|
| `karpal-verify` | `amari` | Statistical verification via amari-flynn |
| `karpal-proof` | `derive` | `#[derive(VerifySemigroup)]` etc. |
| `karpal-verify` | `derive` | `#[export_obligations]` macro |

## Exceptions

- `karpal-schubert-types` is std-only (depends on `amari-enumerative`)
- `karpal-index` is a binary crate (not published to crates.io)

## Per-Crate `no_std` Status

| Crate | `no_std` (core only) | `alloc` | `std` | Notes |
|-------|----------------------|---------|-------|-------|
| `karpal-core` | ✅ | ✅ | ✅ | HKT encoding, functor hierarchy, Semigroup/Monoid work without alloc |
| `karpal-profunctor` | ✅ | ✅ | ✅ | Profunctor, Strong, Choice, FnP |
| `karpal-optics` | ✅ | ✅ | ✅ | Lens, Prism, composition |
| `karpal-arrow` | ✅ | ✅ | ✅ | Arrow hierarchy |
| `karpal-free` | ✅ | ✅ | ✅ | Free constructions (alloc required for most) |
| `karpal-recursion` | ✅ | ✅ | ✅ | Recursion schemes |
| `karpal-algebra` | ✅ | ✅ | ✅ | Abstract algebra |
| `karpal-effect` | ✅ | ✅ | ✅ | Monad transformers |
| `karpal-proof` | ✅ | ✅ | ✅ | Law witnesses, refinement types |
| `karpal-verify` | ❌ | ❌ | ✅ | Verification bridge (process spawning, filesystem) |
| `karpal-verify-derive` | ❌ | ❌ | ✅ | Proc-macro crate (requires std) |
| `karpal-proof-derive` | ❌ | ❌ | ✅ | Proc-macro crate (requires std) |
| `karpal-diagram` | ✅ | ✅ | ✅ | String diagrams |
| `karpal-schubert-types` | ❌ | ❌ | ✅ | Depends on `amari-enumerative` |
| `karpal-higher` | ✅ | ✅ | ✅ | 2-categories, enriched categories |
| `karpal-std` | ❌ | ❌ | ✅ | Prelude re-exports (pulls in all crates) |

CI verifies this via `cargo build --no-default-features -p karpal-core -p karpal-profunctor` on every push.

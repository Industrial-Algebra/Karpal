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

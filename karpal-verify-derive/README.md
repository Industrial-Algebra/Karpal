# karpal-verify-derive

Attribute macros for exporting Karpal verification obligations from Rust items.

Prefer using this crate through `karpal-verify`'s macro re-export:

```rust,ignore
use karpal_verify::export_obligations;

struct Additive;

#[export_obligations(
    crate_name = "example",
    item_path = "Additive",
    carrier = "Int",
    monoid(op = "combine", identity = "empty")
)]
impl Additive {}

let bundle = Additive::karpal_obligation_bundle();
```

Supported explicit families:

- `semigroup(op = "combine")`
- `monoid(op = "combine", identity = "empty")`
- `group(op = "combine", identity = "empty", inverse = "invert")`
- `semiring(add = "add", zero = "zero", mul = "mul", one = "one")`
- `ring(add = "add", zero = "zero", neg = "neg", mul = "mul", one = "one")`
- `lattice(meet = "meet", join = "join")`

The macro currently uses explicit metadata rather than inspecting trait impl bodies.

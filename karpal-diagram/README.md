# karpal-diagram

Monoidal categories and string diagrams for the Karpal ecosystem.

`karpal-diagram` begins Phase 13 of the Karpal roadmap with:

- monoidal category traits: `Tensor`, `Braiding`, `Symmetry`
- a small string-diagram DSL
- text and SVG rendering helpers
- diagram normalization for simple equivalence checking

## Example

```rust
use karpal_arrow::{Arrow, FnA};
use karpal_diagram::{Braiding, Diagram, Tensor};

let double = FnA::arr(|x: i32| x * 2);
let increment = FnA::arr(|x: i32| x + 1);
let parallel = FnA::tensor(double, increment);
assert_eq!(parallel((3, 4)), (6, 5));

let swap = FnA::braid::<i32, bool>();
assert_eq!(swap((7, true)), (true, 7));

let diagram = Diagram::box_("double", 1, 1)
    .parallel(Diagram::box_("increment", 1, 1))
    .then(Diagram::swap(1, 1));
assert!(diagram.render_text().contains("swap[1|1]"));
```

## License

MIT OR Apache-2.0

# karpal-optics

Profunctor optics for Rust: composable, type-safe accessors for nested data
structures.

## What's inside

### Lens

A lens focuses on a single field within a larger structure, providing
get, set, and over operations:

```rust
use karpal_optics::Lens;

struct Point { x: f64, y: f64 }

let x_lens = Lens::new(
    |p: &Point| p.x,
    |p, x| Point { x, ..p },
);

let p = Point { x: 1.0, y: 2.0 };
assert_eq!(x_lens.get(&p), 1.0);

let moved = x_lens.over(p, |x| x + 10.0);
assert_eq!(moved.x, 11.0);
```

### Profunctor transform

The `transform` method connects lenses to the profunctor hierarchy.
Given any `Strong` profunctor, it lifts a `P<A, B>` into `P<S, T>`:

```rust
use karpal_optics::Lens;
use karpal_profunctor::FnP;

struct Person { name: String, age: u32 }

let age_lens = Lens::new(
    |p: &Person| p.age,
    |p, age| Person { age, ..p },
);

let increment: Box<dyn Fn(u32) -> u32> = Box::new(|a| a + 1);
let birthday = age_lens.transform::<FnP>(increment);

let alice = Person { name: "Alice".into(), age: 30 };
assert_eq!(birthday(alice).age, 31);
```

### Optic

Marker trait for all optic types. `Lens` implements `Optic`.

## License

MIT OR Apache-2.0

# karpal-optics

Profunctor optics for Rust: composable, type-safe accessors for nested data
structures.

## What's inside

### Lens

A lens isolates a single field inside a struct, giving you `get`, `set`, and
`over` as first-class values you can store, pass to functions, and compose.

Where plain field access breaks down is when you have nested structs and want
to build reusable field modifiers:

```rust
use karpal_optics::Lens;

struct Sensor { id: u32, reading: f64, unit: String }

let reading = Lens::new(
    |s: &Sensor| s.reading,
    |s, r| Sensor { reading: r, ..s },
);

let raw = Sensor { id: 1, reading: 98.6, unit: "F".into() };

// get — extract the focused field
assert_eq!(reading.get(&raw), 98.6);

// over — apply a transformation (e.g. unit conversion)
let celsius = reading.over(raw, |f| (f - 32.0) * 5.0 / 9.0);
assert!((celsius.reading - 37.0).abs() < 0.01);
```

### Profunctor transform — first-class field modifiers

`transform` is where lenses connect to the profunctor hierarchy. Given
any `Strong` profunctor `P<A, B>`, it lifts it into `P<S, T>` — turning
a function on the *field* into a function on the *whole struct*.

The result is a `Box<dyn Fn(Sensor) -> Sensor>` you can store, pass around,
or compose with other functions — something you can't do by writing
`sensor.reading = new_value` inline:

```rust
use karpal_optics::Lens;
use karpal_profunctor::FnP;

struct Sensor { id: u32, reading: f64, unit: String }

let reading = Lens::new(
    |s: &Sensor| s.reading,
    |s, r| Sensor { reading: r, ..s },
);

// Build a reusable calibration pipeline — it's just a Box<dyn Fn>
let calibrate: Box<dyn Fn(f64) -> f64> = Box::new(|r| r * 1.02 + 0.5);
let calibrate_sensor = reading.transform::<FnP>(calibrate);

// Apply it anywhere — the function carries "which field" knowledge
let s1 = Sensor { id: 1, reading: 100.0, unit: "C".into() };
let s2 = Sensor { id: 2, reading: 200.0, unit: "C".into() };
assert!((calibrate_sensor(s1).reading - 102.5).abs() < 0.001);
assert!((calibrate_sensor(s2).reading - 204.5).abs() < 0.001);

// You can build multiple transforms from the same lens
let clamp: Box<dyn Fn(f64) -> f64> = Box::new(|r| r.clamp(0.0, 100.0));
let clamp_sensor = reading.transform::<FnP>(clamp);

let out_of_range = Sensor { id: 3, reading: 999.0, unit: "C".into() };
assert_eq!(clamp_sensor(out_of_range).reading, 100.0);
```

### Why not just write a method?

You could write `impl Sensor { fn calibrate(self) -> Self { ... } }` — and
for a single struct, that's fine. Lenses pay off when:

- You have many structs with similar fields (multiple sensor types, nested
  configs) and want to reuse the same transformation logic across them.
- You're building a pipeline of field transformations that gets assembled
  at runtime (e.g., user-configured data processing steps).
- You want to abstract over "which field" — pass a lens as a parameter,
  letting the caller decide what to focus on.

### Optic

Marker trait for all optic types. `Lens` implements `Optic`.

## License

MIT OR Apache-2.0

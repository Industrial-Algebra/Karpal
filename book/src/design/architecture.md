# Architecture

## Design Principles

- **GAT-based HKT encoding**: `trait HKT { type Of<T>; }` — clean, zero-dependency
- **Static Land over Fantasy Land**: traits with associated functions (not methods on values)
- **Law verification built in**: every trait ships with proptest-based law tests
- **`no_std` first**: core and profunctor crates work without an allocator
- **Composition over completeness**: each phase delivers a usable layer before the next begins
- **Structured emptiness**: zeros carry provenance — *why* something is empty matters

## Phase Completion

| Phase | Crate(s) | Status |
|-------|----------|--------|
| 1–11 | core through proof | ✅ Complete |
| 12 | karpal-verify | ✅ Complete |
| 13 | karpal-diagram | ✅ Complete |
| 14 | karpal-schubert-types (A–D) | ✅ Complete |
| 15 | karpal-higher | ✅ Complete |
| 16A | HeytingAlgebra | ✅ Complete |
| 16B–D | Topos theory | 🔲 Planned |
| 17 | E2E validation | 🔲 Planned |
| 18 | Ecosystem verification | 🔲 Planned |

## License

Apache-2.0 + CLA. See [CONTRIBUTING.md](https://github.com/Industrial-Algebra/Karpal/blob/develop/CONTRIBUTING.md).

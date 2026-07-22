# Karpal — Project Guidelines

Karpal is a Higher-Kinded Type (HKT) library for the Industrial Algebra ecosystem.

## Workspace Structure

```
karpal/
├── karpal-core/         # HKT encoding, Functor, Semigroup, Monoid (no_std compatible)
├── karpal-profunctor/   # Profunctor, Strong, Choice, FnP (no_std compatible)
├── karpal-optics/       # Profunctor optics: Lens (Phase 1), Prism (Phase 2)
└── karpal-std/          # Standard prelude re-exports (stub, Phase 2)
```

## HKT Encoding

GAT-based marker types: `trait HKT { type Of<T>; }` — stable since Rust 1.65, zero dependencies.
Two-parameter variant: `trait HKT2 { type P<A, B>; }` for profunctors.

## Toolchain

- **Nightly Rust** (edition 2024) is required — pinned via `rust-toolchain.toml`
- Components: `rustfmt`, `clippy`

## Coding Conventions

- Idiomatic Rust: prefer ownership over references where practical, leverage the type system
- TDD approach: write tests alongside or before implementation
- Use **phantom types** and **algebraic type patterns** extensively for type-level programming
- `rayon` for CPU-bound concurrency
- `wgpu` for GPU acceleration
- Keep `unsafe` blocks minimal and well-documented

## Git Workflow (Gitflow)

See **`AGENTS.md`** for the full branch-and-release discipline (the source of
truth). Summary of the hard rules:

- Branch prefixes: `feature/`, `chore/`, `fix/`, `docs/`, `release/v*`, `hotfix/*`
- **Never push directly to `main` or `develop`** — always branch + PR (Rule 1)
- Feature/fix/chore branches → PR to `develop`
- Release PRs: `release/v*` → `main`; **always backmerge `main → develop` after** (Rule 2)
- Release-only commits (CHANGELOG date, version bump) live on `release/*` (Rule 3)
- Hotfixes (untagged) → PR to `main` for production fixes that mustn't wait or republish crates

## Pre-commit Hooks

Local hooks run (in order, fail-fast):
1. `cargo fmt --check --all`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo test --workspace`

Shareable hooks live in `.githooks/`. After cloning, run:
```sh
./scripts/setup-hooks.sh
```

## CI/CD

GitHub Actions CI (`.github/workflows/ci.yml`) runs on all pushes and PRs to `develop`/`main`:
- `cargo fmt --check --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo build --no-default-features -p karpal-core -p karpal-profunctor` (no_std verification)

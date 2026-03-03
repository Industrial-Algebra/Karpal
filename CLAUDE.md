# Karpal — Project Guidelines

Karpal is a Higher-Kinded Type (HKT) library for the Industrial Algebra ecosystem.

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

- Branch prefixes: `feature/`, `chore/`, `fix/`, `refactor/`, `docs/`
- Feature branches → PR to `develop`
- Release PRs: `develop` → `main`
- Never push directly to `main` or `develop`

## Pre-commit Hooks

Local hooks run (in order, fail-fast):
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`

Shareable hooks live in `.githooks/`. After cloning, run:
```sh
./scripts/setup-hooks.sh
```

## CI/CD

GitHub Actions CI (`.github/workflows/ci.yml`) runs on all pushes and PRs to `develop`/`main`:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`

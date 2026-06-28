# Contributing to Karpal

Thank you for contributing to Karpal. This repository uses separate development
and support lines so that new Industrial Algebra work can move forward while
existing `0.4.x` users still have a maintained bugfix path.

## Branches and release lines

| Branch | Release line | License | Purpose |
|--------|--------------|---------|---------|
| `develop` | `0.6.x` and later | `Apache-2.0` | Active development, new features, roadmap work, and ordinary bugfixes |
| `main` | latest released line | `Apache-2.0` | Release branch; changes arrive through release PRs |
| `support/0.4.x` | `0.4.x` | `MIT OR Apache-2.0` | Selective bugfix maintenance for the final permissively licensed line |

Starting with `0.6.0`, Karpal is licensed under `Apache-2.0`. Releases `0.5.x`
were temporarily licensed under `AGPL-3.0-or-later`; this has been corrected.
Earlier `0.4.x` releases remain available under `MIT OR Apache-2.0`.

## Choosing the right base branch

Please target PRs as follows:

- **New features / roadmap work**: target `develop`.
- **Bugfixes only relevant to current development**: target `develop`.
- **Important bugfixes affecting `0.4.x` users**: target `support/0.4.x`.
- **Bugfixes affecting both lines**: prefer a minimal fix on `support/0.4.x`
  first, then forward-port the fix to `develop`.

If you are unsure which branch to target, open an issue or draft PR and ask.

## CLA

All contributors must sign the [CLA](https://github.com/Industrial-Algebra/.github/blob/main/CLA.md).

## `0.4.x` maintenance policy

The `0.4.x` line is selectively maintained. It exists so downstream users who
need the earlier permissive license can continue receiving important fixes.

Accepted for `support/0.4.x`:

- security fixes
- correctness fixes
- serious regression fixes
- compatibility fixes needed by existing users
- small test/documentation updates directly tied to accepted bugfixes

Not accepted for `support/0.4.x`:

- new public APIs
- new crates or roadmap phases
- broad refactors not required for a bugfix
- feature work that belongs to `0.6.x` or later


## Licensing of contributions

By opening a PR, you agree that your contribution is submitted under the CLA
and the Apache-2.0 license of the target branch.

## Local verification

Before submitting a PR, run:

```sh
cargo fmt --check --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For feature-specific changes, also run any relevant feature checks documented in
crate READMEs or CI.

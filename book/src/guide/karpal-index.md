# Type Discovery with karpal-index

`karpal-index` is a CLI binary that lets AI agents (and humans) discover Karpal's types and operations through progressive drill-down.

## Commands

### Search

```bash
$ karpal-index search Functor
Functor                        trait           Covariant functor: lifts a function A->B into F<A>->F<B>
FunctorFilter                  trait           FunctorFilter: a Functor that can filter elements
```

### Detail

```bash
$ karpal-index detail Functor
Functor [trait]
  crate: karpal-core
  supertraits: HKT
  methods:
    - fn fmap<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> B) -> Self::Of<B>;
  implementors:
    - OptionF
    - VecF
    - IdentityF
```

### Hierarchy

```bash
$ karpal-index hierarchy Semigroup
Semigroup [trait]
  subtraits:
    - Monoid
  implementors:
    - String
```

### JSON Output

All commands support `--json` for programmatic consumption:

```bash
$ karpal-index search Functor --json
[{"name":"Functor","kind":"trait","crate_name":"karpal-core",...}]
```

## Usage

```sh
cargo run --bin karpal-index -- search Functor
# or install:
cargo install --path . --bin karpal-index
karpal-index search Functor
```

The binary reads the workspace source tree at runtime — no pre-built index needed.

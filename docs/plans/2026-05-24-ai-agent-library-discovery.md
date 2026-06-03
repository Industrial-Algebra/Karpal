# AI-Agent Library Discovery via Per-Project CLI

## Problem

AI coding agents (Claude Code, Codex, Gemini CLI, etc.) need to discover and
use Karpal's types, traits, and operations effectively. IA-MCP has proven
insufficient for two reasons:

1. **Context bloat** — MCP tool responses are too large, filling the agent's
   context window with data it doesn't immediately need
2. **Flat discovery** — Even with a current manifest, agents overlook features
   at first glance because the library's structure isn't progressively disclosed

CLI tools have worked better because they support progressive drill-down
(`search` → `list` → `detail`) with minimal output per step.

## Design

Each foundational library (Karpal, Amari, Orlando) ships a small CLI binary
(e.g., `karpal-index`) that agents invoke via bash. The binary reads the
library's source tree at runtime and exposes a progressive-discovery command set.

### Command model

```
karpal-index search <query>    → names + one-line descriptions
karpal-index detail <name>     → signature, trait bounds, impls, docs
karpal-index hierarchy <trait> → super/subtraits, implementors
karpal-index example <name>    → usage example
karpal-index crates            → workspace crate list
karpal-index graph             → dependency/trait hierarchy overview
```

All commands support `--json` for structured output. Default output is
human-readable but concise.

### Agent workflow

```
$ karpal-index search "Functor"
Functor           — core functor trait
FunctorFilter     — filterable functors
Contravariant     — contravariant functor

$ karpal-index detail Functor
trait Functor {
    type Target<B>;       // HKT target
    fn fmap<A,B>(f: impl Fn(A)->B, fa: Self::Target<A>) -> Self::Target<B>;
}
Implementors: OptionF, VecF, ResultF, IdentityF, ...

$ karpal-index example Functor
let result: Option<i32> = OptionF::fmap(|x| x * 2, Some(21));
assert_eq!(result, Some(42));
```

### Index content

The index captures structured API metadata with cross-references:

- **Traits**: name, crate, module path, supertraits, associated types,
  required/provided methods, doc summary
- **Types**: name, crate, module path, fields/variants, trait impls, doc summary
- **Functions**: name, crate, module path, signature, doc summary
- **Macros**: name, crate, module path, invocation syntax, doc summary
- **Impls**: impl trait for type, where clause, feature gate
- **Crates**: name, description, feature flags, dependencies, public modules

### Index generation (source-tree reader, Approach B)

The CLI binary reads `.rs` source files from the crate source tree at
runtime. This avoids embed-time staleness and works for any repo checkout.
The index is built on first invocation and cached.

Implementation: a Rust source parser using `syn` that walks the workspace,
extracts public API items from each crate, and builds the in-memory index.

### Distribution

The binary lives in the workspace root as a default member crate
(`karpal-index/`). Agents invoke it via:

```
cargo run --bin karpal-index -- search Functor
```

Or install it once:

```
cargo install --path . --bin karpal-index
karpal-index search Functor
```

## Scope

Initial scope: `karpal-index` for the Karpal workspace. The pattern is
reusable — `amari-index` and `orlando-index` would follow the same design
with library-specific content.

## Open questions

- Whether to parse source with `syn` at runtime or pre-generate a static
  index file via a build script
- Whether to also provide an MCP wrapper that reads the same index file
  (for agents that prefer MCP over CLI)

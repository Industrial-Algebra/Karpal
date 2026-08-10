# karpal-discovery — Subprocess-Hosting Spike: Findings

Date: 2026-08-10
Status: spike (validated end-to-end)
Branch: `feature/discovery-subprocess-spike`
Depends on: Lonis @ `3f3ffdc` (ADR-0001/0002/0003 landed)

## Goal

Dogfood Lonis's `SubprocessTool` (ADR-0003) from the consumer side — host a
`karpal` binary through it — to surface protocol/contract requirements while
Lonis progresses in parallel.

## What was built

- A `lonis` feature on `karpal-discovery` (optional git deps on `lonis-schema`
  + `lonis-core`, pinned to Lonis develop HEAD). The default, lonis-free crate
  is unchanged and stays publishable.
- `src/payload.rs`: `KarpalPayload: BlockPayload` (one variant, `Search`),
  carrying real catalog data.
- `src/main.rs`: the `karpal` binary (`required-features = ["lonis"]`) speaking
  the ADR-0003 wire protocol — JSON on stdin, blocks on stdout, structured
  `ToolError` on stderr.
- `tests/subprocess_hosting.rs`: hosts `karpal` via `SubprocessTool` and asserts
  the typed payload round-trips through the seam.

## Result — the full vertical validates end-to-end

`karpal search`, hosted by `SubprocessTool`, emits `Block<KarpalPayload>`; the
host parses it as `Vec<Block<BlockKind>>` with the vertical payload landing
**losslessly** in `BlockKind::Extension { kind: "karpal_search", data }`,
`Functor` present in results; structured errors propagate with the right
exit code. **21 tests** (`--features lonis`); **19** default; fmt/clippy/doc
clean both ways; workspace + no_std gate unaffected.

## Requirements surfaced (for Lonis)

1. **Vertical payload enums must serialize as `{"kind": …, "data": …}`
   (adjacently tagged).** `BlockKind`'s custom `Deserialize` reads the wire form
   `{"kind", "data"}` and maps unknown kinds to `Extension`. An
   *internally*-tagged payload (`#[serde(tag = "kind")]`) would **not**
   round-trip through the seam — its fields would land flattened, not under
   `data`. Lonis should document this or offer a derive so vertical authors
   don't discover it by failure. (`KarpalPayload` uses
   `#[serde(tag = "kind", content = "data")]`.)

2. **The serde `kind` tag must equal `BlockPayload::kind_name()`, and should be
   namespaced.** The host sees the serde `kind` tag (it becomes
   `Extension.kind`), *not* `kind_name()`. If they diverge, the in-process
   `kind_name()` and the wire/host kind disagree. They must agree, and the kind
   should be namespaced (`karpal_search`, not `search`) to avoid collisions
   across verticals. A Lonis derive could generate both consistently from one
   declaration.

3. **Cross-repo pre-publication deps must be git deps, not path deps.** A
   relative path dep to a sibling repo (`../../Lonis/…`) breaks across git
   worktree locations (it resolves relative to the worktree, not the canonical
   tree) and isn't portable or committable. karpal-discovery uses git deps
   pinned to a rev. (Mitigation for local dev: a `[patch]` override — but that
   path is worktree-relative too, so it's machine-local only.)

4. **The target a tool operates on must arrive via stdin input or argv, not env
   or cwd.** `SubprocessTool` clears the environment (PATH only) and uses a
   neutral cwd (temp dir). So `karpal search` takes `{"workspace": …, "query":
   …}` on stdin. Any tool that operates on an external target has the same
   constraint — worth calling out in the protocol guide.

## Validations (things that worked, not requirements)

- The ADR-0002 erased seam round-trips vertical payloads **losslessly** — the
  host never knows `KarpalPayload`, only `BlockKind`, and the typed data is
  preserved byte-for-byte in `Extension.data`.
- `ToolError` propagation is end-to-end: the bin's structured stderr + exit
  code surfaces as the host's `ToolError { kind, exit_code }`.
- **Optional git deps don't touch the default build** — Cargo fetches/builds
  them only when the feature is on, so the publishable lonis-free core stays
  intact (19 default tests, no lonis fetch).

## Open / future

- **Many operations → provider model.** Today one `SubprocessTool` is registered
  per argv prefix (`karpal search`). With karpal-discovery's full operation set
  (search/detail/implementors/inspect/recommend/plan/probe), per-operation
  registration is verbose. ADR-0003's future `SubprocessProvider` (one
  executable, `call <tool>`) is the cleaner host integration — and `lonis`
  itself is already close to a conforming provider.
- **Publishability.** The `lonis` feature (and the `karpal` bin) are
  unpublishable until `lonis-schema` reaches crates.io. The default crate is
  unaffected. When lonis-schema publishes, switch the git deps to crates.io and
  consider default-enabling the feature.
- **Rev tracking.** The git dep is pinned to `3f3ffdc`; bump as Lonis progresses.

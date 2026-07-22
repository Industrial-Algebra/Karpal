# AGENTS.md — Karpal

Guidance for AI agents (and humans) working in this repo. The single source of
truth for **branch discipline and release mechanics**. See `CLAUDE.md` for
project context, toolchain, and coding conventions.

## Gitflow — read this before touching `develop` or `main`

Karpal uses IA's release-oriented gitflow. **Two hard rules have each caused real
damage in sibling repos (Schubert). Follow them.**

### Branch model

```
            feature/* ──PR──▶ develop ──release PR──▶ release/v* ──PR──▶ main ──tag/release──▶ crates.io
              fix/*              ▲                                                                     │
              chore/*            └──────────────── backmerge (merge commit) ────────────────────────┘
            hotfix/* ──PR──▶ main ──backmerge──▶ develop   (production fixes that mustn't wait)
```

| Branch        | Purpose                                              | Lands via         |
|---------------|------------------------------------------------------|-------------------|
| `main`        | What shipped. Protected. Every commit is a release.  | `release/*` or `hotfix/*` PR only |
| `develop`     | Integration for the *next* release. Protected.       | `feature/*`/`fix/*`/`chore/*` PR only |
| `feature/*`   | One PR's work, off `develop`.                        | PR → `develop`    |
| `fix/*`       | Bug fixes, same lifecycle as `feature/*`.            | PR → `develop`    |
| `chore/*`     | Docs/tooling/config that rides the next release.     | PR → `develop`    |
| `release/v*`  | Release-only commits (CHANGELOG date, version bump). | PR → `main`       |
| `hotfix/*`    | Urgent production fix off `main` (see below).        | PR → `main`, then backmerge |

### Rule 1 — Never push directly to a protected branch

`develop` and `main` receive changes **only via merged PRs**. No `git push` to
either — not "just a one-line fix", not "last-minute release tweak", not "it's
faster". Branch it, PR it, let CI run. The PR flow is what runs CI before code
lands. (Install the opt-in `pre-push` hook from the ia-gitflow skill to enforce
this at the machine.)

### Rule 2 — Every merge to `main` is followed by a `main → develop` backmerge

After a release PR **or a hotfix** merges to `main`, immediately backmerge
`main` into `develop` using a **merge commit, never a squash**. This is the last
step of releasing, not an optional chore. A squash-merged release with no
backmerge diverges the graphs; the divergence is invisible until the *next*
release PR, where it surfaces as a confusing conflict. If you tagged/published,
you owe `develop` a backmerge.

### Rule 3 — Release-only commits live on a `release/*` branch

Dating the CHANGELOG, final version touches — these belong on `release/v*` so
they're reviewed in the release PR, not pushed to `develop` (Rule 1) or buried in
the squash. The backmerge carries them to `develop`.

### Hotfixes (production fixes that mustn't wait for a release)

For urgent production fixes (broken docs deploy, critical runtime bug) that must
reach `main` immediately without a full release cycle:

1. Branch `hotfix/*` off `origin/main`.
2. Apply the minimal fix. Verify.
3. PR `hotfix/*` → `main`. Merge with a merge commit.
4. **Backmerge `main → develop`** (Rule 2 still applies).

**Critical nuance — crates.io publishing runs on GitHub release publication
(`on: release: types: [published]`), NOT on tag pushes or main pushes.** So an
**untagged** hotfix merge to `main` deploys docs/CI but does **not** republish
crates. Only create a GitHub release (`gh release create`) when you intend to
publish. Use this to ship docs/deploy/config fixes without a spurious crate
version bump.

If a fix must also ship to crates.io, cut a proper `release/v*` patch instead.

## Verification matrix

Before declaring a PR mergeable or a release shipped, run these and read the
output — evidence before claims:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features          # plus each feature combo the CI matrix covers
cargo doc --all-features --no-deps # zero "unresolved link"
cargo build --no-default-features -p karpal-core -p karpal-profunctor -p karpal-topos  # no_std gate
```

`cargo fmt --all` formats feature-gated files too — never use standalone
`rustfmt --edition` (it disagrees with cargo-fmt's style).

## Release workflow

1. **Version bump** on a branch off `develop`; PR to `develop`; merge. (All
   versions come from `[workspace.package]`; bump there + every `version = "..."`
   workspace-dep ref in each crate `Cargo.toml`.) Leave CHANGELOG `## [X.Y.Z] — Unreleased`.
2. **Cut `release/v<ver>`** off the updated `develop`.
3. **Date the CHANGELOG** on the release branch: `## [X.Y.Z] — <YYYY-MM-DD>`.
   Verify (the matrix above).
4. **Release PR** `release/v<ver> → main`. If it conflicts, a prior release
   skipped its backmerge — merge `origin/main` in, resolve to the release
   branch's (superset) content, re-verify.
5. **Merge** to `main` (merge commit).
6. **Tag** `v<ver>` on the merge commit and push the tag.
7. **`gh release create v<ver>`** — this triggers `publish.yml`, which publishes
   all crates to crates.io in dependency order (with index-wait sleeps). It
   also triggers the Netlify docs deploy build hook.
8. **Backmerge** `main → develop` (Rule 2) via a PR, **merge commit**.

## Publish workflow notes

- `publish.yml` triggers on `release: [published]` (and `workflow_dispatch`).
  It does **not** fire on tag push or branch push.
- Crates publish in dependency order: `karpal-core` first, then dependents.
  `karpal-topos` (depends only on `karpal-core`) publishes right after core.
- `continue-on-error` on already-published crates makes republish idempotent.
- The last step fires the Netlify build hook to redeploy the docs.

## Docs deploy

- Netlify serves `book/book/` (the English mdBook) at `karpal.industrialalgebra.com`.
- The Japanese mdBook deploys at `/book-ja/` (copied into `book/book/book-ja/` by the build command in `netlify.toml`).
- Netlify auto-deploys on pushes to `main` (production branch). Docs fixes can
  ship via an untagged hotfix to `main` (see Hotfixes) without a crate release.

## Common pitfalls

| Shortcut | Symptom | Fix |
|---|---|---|
| Push straight to `develop`/`main` | Protected branch red; CI never ran | Rule 1 — always branch + PR |
| Release squash-merged, no backmerge | Next release PR conflicts vs `main` | Rule 2 — backmerge (merge commit) every time |
| Date CHANGELOG by pushing to `develop` | Violates Rule 1; re-stales if reverted | Rule 3 — date it on `release/*` |
| Backmerge as a squash | Graphs still don't join | Backmerge with a merge commit |
| Cherry-pick fix straight to `main` | Violates "main = releases only" | Use `hotfix/*` branch + PR (or ride a release) |
| `gh release create` without meaning to publish | Unintended crates.io version | Only create releases when publishing; docs fixes ship untagged |
| Tag on `develop` not `main` | `publish.yml` doesn't fire | Tag the `main` merge commit |

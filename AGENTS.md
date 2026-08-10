# AGENTS.md — Karpal

Guidance for AI agents (and humans) working in this repo. The single source of
truth for **branch discipline and release mechanics**. See `CLAUDE.md` for
project context, toolchain, and coding conventions.

## Gitflow — read this before touching `develop` or `main`

Karpal uses IA's release-oriented gitflow: work lands on `develop` via reviewed
PRs, releases cut a `release/*` branch onto `main`, a GitHub release triggers the
publish workflow, and a **mandatory `main → develop` backmerge** rejoins the
graphs afterward.

The discipline exists because two specific shortcuts have each caused real
damage in sibling repos (Schubert v0.3.0/v0.4.0). **Follow the hard rules.** A
release that skips the backmerge looks shipped but silently diverges `main` from
`develop`, surfacing as a conflict at the *next* release.

### Branch model

```
            feature/* ──PR──▶ develop ──release PR──▶ release/v* ──PR──▶ main ──release──▶ crates.io
              fix/*              ▲                                                                 │
              chore/*            └────────────────── backmerge (merge commit) ────────────────┘
            hotfix/* ──PR──▶ main ──backmerge──▶ develop   (production fixes that mustn't wait)
```

| Branch        | Purpose                                              | Lands via         |
|---------------|------------------------------------------------------|-------------------|
| `main`        | What shipped. Protected. Every commit is a release.  | `release/*` or `hotfix/*` PR only |
| `develop`     | Integration for the *next* release. Protected.       | `feature/*`/`fix/*`/`chore/*` PR only |
| `feature/*`   | One PR's worth of work, off `develop`.               | PR → `develop`    |
| `fix/*`       | Bug fixes, same lifecycle as `feature/*`.            | PR → `develop`    |
| `chore/*`     | Docs/tooling/config that rides the next release.     | PR → `develop`    |
| `release/v*`  | Release-only commits (CHANGELOG date, version bump). | PR → `main`       |
| `hotfix/*`    | Urgent production fix off `main` (see below).        | PR → `main`, then backmerge |

### Rule 1 — Never push directly to a protected branch

`develop` and `main` receive changes **only via merged PRs**. No `git push` to
either — not "just a one-line fix", not "last-minute release tweak", not "it's
faster". Branch it, PR it, let CI run. The PR flow is what runs CI before code
lands. (Install the opt-in `pre-push` hook below to enforce this at the machine.)

**Why:** Schubert v0.4.0 work was pushed straight to `develop`. CI never ran on
it. `develop` went red (broken format, dead doc-links, an ungated example) and
stayed red until the first proper PR surfaced it — blocking the release. The PR
flow isn't ceremony; it's what runs CI before code lands.

### Rule 2 — Every merge to `main` is followed by a `main → develop` backmerge

After a release PR **or a hotfix** merges to `main`, immediately backmerge
`main` into `develop` using a **merge commit, never a squash**. This is the last
step of releasing, not an optional chore.

**Why:** Release PRs are commonly squash-merged (one tidy commit on `main`). A
squash creates a `main`-only commit that `develop`'s graph never contains, so
`main` and `develop` diverge the instant the squash lands. The divergence is
invisible until the *next* release PR — where it surfaces as a confusing "how
did this conflict?" against `main`. If you tagged/published, you owe `develop` a
backmerge.

### Rule 3 — Release-only commits live on a `release/*` branch

Dating the CHANGELOG, final version touches — these belong on `release/v*` so
they're **reviewed** (in the release PR) rather than pushed straight to
`develop` (which would violate Rule 1) or buried in the squash. After the
release, the backmerge carries them to `develop`.

## Workflows

### Feature / fix / chore work

```sh
git checkout develop && git pull
git checkout -b feature/<short-scope>      # or fix/* or chore/*
# ... work, commit ...
git push -u origin feature/<short-scope>
gh pr create --base develop --head feature/<short-scope>
```

After green CI, merge (squash or merge commit — either is fine for features; the
backmerge rule only governs the release step).

### Rebasing a PR onto an updated `develop`

If `develop` moved while your PR was open, rebase and force-push your own branch:

```sh
git fetch origin
git rebase origin/develop
git push --force-with-lease   # Karpal is single-remote (GitHub only); this works cleanly
```

(Karpal has a single `origin` on GitHub — no Forgejo mirror — so
`--force-with-lease` is reliable. Sibling repos with a dual-push remote need
plain `--force` for rebased own branches.)

### Releasing

1. **Version bump** — on a branch off `develop`; PR to `develop`; merge. (All
   versions come from `[workspace.package]`; bump there + every
   `version = "..."` workspace-dep ref in each crate `Cargo.toml`.) Leave the
   CHANGELOG as `## [X.Y.Z] — Unreleased`.
2. **Cut `release/v<ver>`** off the updated `develop`:
   `git checkout -b release/v<ver> origin/develop`.
3. **Date the CHANGELOG** on the release branch: `## [X.Y.Z] — <YYYY-MM-DD>`.
   Commit. Run the **verification matrix** below.
4. **Release PR** `release/v<ver> → main`. If it conflicts, see *Reconciling a
   conflicting release PR* below.
5. **Merge** the release PR to `main` (merge commit).
6. **Tag** `v<ver>` on the merge commit and push the tag.
7. **`gh release create v<ver>`** — this triggers `publish.yml`, which publishes
   all crates to crates.io in dependency order (with index-wait sleeps). It also
   fires the Netlify docs deploy build hook.
8. **Backmerge** `main → develop` (Rule 2) via a PR, **merge commit**.

### Reconciling a conflicting release PR

A release PR that "shouldn't" conflict means a prior release skipped its
backmerge. Diagnose:

```sh
git merge-base --is-ancestor origin/main origin/develop \
  && echo "clean (FF possible)" || echo "DIVERGED — backmerge was skipped"
git log --oneline origin/develop..origin/main   # what main has that develop lacks
git diff --stat origin/main origin/develop      # expect a few metadata files only
```

Almost always `develop`'s tree is a **strict superset** of `main`'s (it holds the
prior release's content plus new work). Verify per-file with `git diff` before
resolving. Resolve conflicted metadata files (CHANGELOG, Cargo.toml, Cargo.lock,
README) to the release branch's content, regenerate `Cargo.lock`, and run the
**full verification matrix** before committing. Don't assume the superset —
confirm with `git diff` per file.

### Hotfixes (production fixes that mustn't wait for a release)

For urgent production fixes (broken docs deploy, critical runtime bug) that must
reach `main` immediately without a full release cycle:

1. Branch `hotfix/*` off `origin/main`.
2. Apply the minimal fix. Verify (the matrix below).
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

## Publish workflow notes

- `publish.yml` triggers on `release: [published]` (and `workflow_dispatch`).
  It does **not** fire on tag push or branch push.
- Crates publish in dependency order: `karpal-core` first, then dependents.
  `karpal-topos` (depends only on `karpal-core`) publishes right after core.
- `continue-on-error` on already-published crates makes republish idempotent.
- The last step fires the Netlify build hook to redeploy the docs.

## Docs deploy

- Netlify serves `book/book/` (the English mdBook) at `karpal.industrialalgebra.com`.
- The Japanese mdBook deploys at `/book-ja/` (copied into `book/book/book-ja/` by
  the build command in `netlify.toml`).
- mdBook renders HTML from the markdown source (`book/src/`, `book-ja/src/`).
  **Never commit build output** (`book/book/`, `book-ja/book/`) — both are
  `.gitignore`d; Netlify regenerates them on every deploy.
- Netlify auto-deploys on pushes to `main` (production branch). Docs fixes can
  ship via an untagged hotfix to `main` (see Hotfixes) without a crate release.

## Optional enforcement — pre-push hook

A local `pre-push` hook blocks accidental pushes to `develop`/`main` (Rule 1 at
the machine, not just the mind). Opt-in per clone:

```sh
cat > .git/hooks/pre-push <<'EOF'
#!/usr/bin/env bash
while read local_ref local_sha remote_ref remote_sha; do
  case "$remote_ref" in
    refs/heads/develop|refs/heads/main|refs/heads/master)
      echo "ia-gitflow: direct push to $remote_ref blocked (use a PR)." >&2
      exit 1 ;;
  esac
done
EOF
chmod +x .git/hooks/pre-push
```

This catches the most common slip. It does **not** replace Rule 2 (the
backmerge) — that's a release-process check, not a push check. The repo also
ships a `.githooks/pre-commit` (fmt/clippy/test); enable it with
`./scripts/setup-hooks.sh`.

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
| Commit mdBook build output | Repo bloats with generated HTML/CSS/JS | Keep `book/book/` + `book-ja/book/` gitignored |

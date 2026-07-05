# mdBook Documentation Critique — Consolidated Triage

**Tool:** Proserpina v0.2.1, 5-critic panel (Devil's Advocate, Methodologist, Red Team, Domain Expert, Editor)
**Date:** 2026-07-01
**Scope:** All 45 mdBook pages, batched into 6 themed review docs
**Total findings:** 62 (9 blockers, 28 major, 14 minor, 11 info)

## Severity Summary

| Batch | Blocker | Major | Minor | Info | Total |
|-------|---------|-------|-------|------|-------|
| 1. Core & Getting Started | 1 | 6 | 1 | 3 | 11 |
| 2. Functor Hierarchy | 1 | 3 | 2 | 3 | 9 |
| 3. Optics & Profunctor | 1 | 4 | 2 | 2 | 9 |
| 4. Arrow, Free, Algebra | 3 | 6 | 3 | 1 | 13 |
| 5. Proof & Verification | 2 | 5 | 2 | 1 | 10 |
| 6. Examples & Architecture | 1 | 4 | 4 | 1 | 10 |

---

## Blockers (9) — Must Address

### DOC-FIXABLE (fix the docs now)

#### B1: Conflicting "Getting Started" pages with contradictory version/license
- **Batch:** 1 (Core)
- **Issue:** Two Getting Started pages exist — `getting-started.md` (scaffold, says v0.7 + Apache) and `getting-started-full.md` (ported HTML, says v0.1 + AGPL). The ported page has stale version and license info.
- **Action:** Merge into one page. Update version to 0.7, license to Apache-2.0 + CLA. Remove the duplicate.
- **Effort:** 30 min

#### B2: Certified→Proven `unsafe` conversion creates illusory security boundary
- **Batch:** 6 (Examples)
- **Issue:** The verified-domain-api example shows `let _: Proven<...> = unsafe { imported.into_proven() };` — this allows forging external verification with no signature, checksum, or replay protection.
- **Action:** This is **by design** (the trust boundary is documented in `docs/dev/phase-12-trust-model.md`), but the example doesn't explain *why* it's unsafe or what the trust model actually is. Add a prominent warning box to the example explaining the trust boundary.
- **Effort:** 20 min (doc clarification)

### CODE-LEVEL (track as issues, document as known limitations)

#### B3: `OptionF::extract` panics on None
- **Batch:** 2 (Functor)
- **Issue:** The Comonad `extract` implementation panics on `None`, violating totality. This is an inherent limitation of `Option` as a Comonad.
- **Action:** Document as a known partiality. Consider returning `Result` or requiring `Some` at the type level.
- **Note:** This is an existing design decision, not a regression.

#### B4: `'static` lifetime bounds on contravariant/profunctor types
- **Batch:** 3 (Optics)
- **Issue:** Contravariant and profunctor types require `'static`, breaking claimed duality with covariant hierarchy (which works with arbitrary lifetimes).
- **Action:** Document as a known limitation of the GAT encoding.

#### B5: ArrowLoop discards the feedback value
- **Batch:** 4 (Algebra)
- **Issue:** ArrowLoop's `loop_` discards the feedback, not implementing a true fixpoint.
- **Action:** Investigate the implementation. May be a stub or known limitation.

#### B6: FreeAp `fold_map` cannot be implemented
- **Batch:** 4 (Algebra)
- **Issue:** Rust can't dispatch generics through erased types for FreeAp's primary eliminator.
- **Action:** Document as a known limitation.

#### B7: Derived State monad from `ReaderF . EnvF` doesn't thread state
- **Batch:** 4 (Algebra)
- **Issue:** The adjunction-derived State acts as a Reader, not threading modified state.
- **Action:** Investigate the adjunction implementation.

#### B8: FunctorSt/ChainSt hierarchy creates silent incompatibility
- **Batch:** 5 (Proof)
- **Issue:** Types implementing only base `Functor` can't be used with transformer-generic code expecting `FunctorSt`.
- **Action:** Document the transformer typeclass hierarchy explicitly.

#### B9: ReaderTF/StateTF omit ApplicativeSt
- **Batch:** 5 (Proof)
- **Issue:** These transformers skip the ApplicativeSt level, violating the categorical hierarchy.
- **Action:** Investigate whether this is intentional (partial implementation) or a gap.

---

## Major Findings (28) — Should Address

### Documentation Fixes (high value, low effort)

1. **Version/license staleness** — Ported HTML docs reference v0.5.0, AGPL. Need to update to v0.7, Apache-2.0 throughout.
2. **No bridge from basic to advanced** — Getting Started jumps from `do_!` to SMT/Lean verification with no motivation paragraph.
3. **Structured Emptiness not connected** — The concept section doesn't show integration with Functor/Monad/do_!.
4. **HKT practical value not demonstrated** — No compelling example showing what Karpal adds over standard Rust.
5. **Nightly Rust requirement** — Not clear which specific nightly features are needed.
6. **no_std claim undocumented** — No per-crate, per-feature breakdown of no_std support.
7. **Missing performance discussion** — No benchmarks or zero-cost abstraction claims.
8. **Clone-heavy lenses** — Every lens getter clones; no reference-returning alternative documented.
9. **Static Land pattern discoverability** — `OptionF::fmap(...)` breaks Rust conventions; no extension trait alternative.
10. **SVG diagram accessibility** — Raw SVG won't render in many viewers; needs ASCII fallback.

### Design/Implementation Concerns (document as known limitations)

11. **Property testing ≠ formal proof** — Docs conflate the two. Need clear distinction.
12. **Proven<T> is a documentation label** — Not a cryptographic/formal guarantee. Should be explicit.
13. **`Box<dyn Fn>` in optics composition** — Allocation per composition; performance cliff undocumented.
14. **Certificate model has no crypto** — Arbitrary strings accepted as solver output.
15. **do_! macro hygiene** — `=` syntax collides with assignment; no pattern matching.
16. **Academic bloat** — Many esoteric structures with no demonstrated practical application.

---

## Recommended Action Plan

### Immediate (doc fixes for 0.7.0)
- [ ] Merge duplicate Getting Started pages, fix version/license (B1)
- [ ] Update all ported pages: v0.5.0→v0.7, AGPL→Apache-2.0 references
- [ ] Add trust boundary warning to verified-domain-api example (B2)
- [ ] Add motivation bridge paragraph before verification section
- [ ] Add HKT value proposition example (non-trivial, 50+ lines)
- [ ] Document nightly feature requirements specifically
- [ ] Add no_std per-crate status table
- [ ] Separate "stable core" from "experimental" in workspace overview

### Track as GitHub issues
- [ ] B3-B9: Code-level blockers (create issues with `documentation` + `bug` labels)
- [ ] Certificate cryptographic signing (security enhancement)
- [ ] Performance benchmarks
- [ ] Extension traits for Static Land discoverability

### Defer to 0.8.0+
- [ ] do_! macro pattern matching support
- [ ] Reference-returning lens getters
- [ ] Full formal semantics for Proven<T>
- [ ] Stable Rust migration path assessment

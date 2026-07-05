# Critique Report

**Subject:** `/tmp/karpal-review/batch5-proof.md`

**Findings:** 10 (2 blocker, 5 major, 2 minor, 1 info)

## 1. [blocker] The separate `FunctorSt`/`ChainSt` hierarchy creates a silent incompatibility surface where types implementing only the base `Functor` cannot be used with transformer-generic code.

- **Category:** parallel-trait-incoherence
- **Location:** §Static Type Classes
- **Quote:** > "The standard `Functor` / `Applicative` / `Chain` traits in `karpal-core` do not have `'static` bounds... so `karpal-effect` introduces parallel traits with the suffix `St`."
- **Suggested change:** Provide conversion functions or blanket impls between the two hierarchies, or document the compatibility story explicitly with guidance on which traits to implement when.
- **Raised by:** Red Team, Devil's Advocate

## 2. [blocker] ReaderTF and StateTF omit ApplicativeSt, violating the categorical hierarchy where every monad must be an applicative functor.

- **Category:** applicative-omission
- **Location:** §ReaderTF, §StateTF, §Design Notes
- **Quote:** > "Why no ApplicativeSt for ReaderT and StateT? — pure_st must produce a Box<dyn Fn(E) -> M::Of<A>> from a single A. The closure may be called multiple times, so A must be cloneable. But adding A: Clone to the trait would impose that requirement globally... The solution: standalone reader_t_pure / state_t_pure functions with explicit A: Clone bounds."
- **Suggested change:** Implement ApplicativeSt by constraining pure_st at the method level (`fn pure_st<A: Clone + 'static>(a: A) -> Self::Of<A>`) rather than omitting the trait entirely. Alternatively, document this as a known design trade-off with a clear migration path.
- **Raised by:** Devil's Advocate, Methodologist, Domain Expert

## 3. [major] Monad transformer laws are stated but not verified for any implementation.

- **Category:** missing-law-verification
- **Location:** §MonadTrans
- **Quote:** > "The law `lift(M::pure_st(a)) == pure(a)`"
- **Suggested change:** Add testable law formulations for all monad transformer laws (naturality, composition, stacking) and document the test suite as part of the CI pipeline.
- **Raised by:** Methodologist, Domain Expert, Red Team

## 4. [major] The Certified-to-Proven conversion uses `unsafe` for logical correctness assertions, not memory safety, which is an abuse of the safe/unsafe contract and provides no actual semantic guarantee.

- **Category:** unsound-unsafe-boundary
- **Location:** §Explicit trust boundary
- **Quote:** > "Crossing into Proven<P, T> remains an explicit unsafe action."
- **Suggested change:** Replace `unsafe` with a designated `assume_unchecked` method whose `# Safety` section explicitly documents the required logical invariant, or use a procedural macro that audits conversions at compile-time.
- **Raised by:** Devil's Advocate, Red Team, Methodologist

## 5. [major] The string diagram normalization rule set lacks proof of termination and confluence, so equivalence checking via normalization may be unsound.

- **Category:** strong-normalization-unproven
- **Location:** §Diagram Normalization
- **Quote:** > "Diagrams normalize to a canonical form using these rewrite rules... The proposal claims normalization produces 'a canonical form,' but doesn't prove termination, confluence, or uniqueness."
- **Suggested change:** Provide a ranking measure (e.g., decreasing size or depth) that strictly decreases with each rewrite step, and document critical pair analysis or restrict equivalence checking to decidable fragments.
- **Raised by:** Methodologist, Devil's Advocate, Red Team

## 6. [major] The `Clone` bound on `MonadTrans::lift` is imposed globally but is only needed by closure-based transformers, creating a leaky abstraction that prevents non-Clone inner monad values from working with any transformer.

- **Category:** clone-bound-leakage
- **Location:** §MonadTrans
- **Quote:** > "lift embeds an M computation into the transformer without adding any effect. The Clone bound on M::Of<A> is needed by closure-based transformers (ReaderT, StateT)."
- **Suggested change:** Remove the `Clone` bound from the trait and add it only to the specific `lift` implementations for `ReaderTF` and `StateTF` via a where clause.
- **Raised by:** Devil's Advocate, Domain Expert, Red Team

## 7. [major] The Littlewood-Richardson computation in `check_intersection` is #P-complete with no documented time bounds or fallback, enabling denial-of-service via large partitions.

- **Category:** computation-unbounded
- **Location:** §Intersection
- **Quote:** > "check_intersection(a, b) computes the intersection product via amari-enumerative... The `Underdetermined` result's semantics are unspecified."
- **Suggested change:** Document the algorithm's complexity and correctness guarantees, add a configurable timeout or iteration limit, and handle `Underdetermined` in a way that cannot produce `Positive` (a false positive).
- **Raised by:** Red Team, Methodologist

## 8. [minor] The Schubert Types section uses specialized algebraic geometry terminology (Grassmannian, Littlewood-Richardson) without explanation, making it inaccessible to most readers.

- **Category:** insufficient-context
- **Location:** §Schubert Types
- **Quote:** > "Types are Schubert classes σ_λ in a Grassmannian Gr(k, n), and type compatibility is computed via Littlewood-Richardson intersection coefficients."
- **Suggested change:** Add a "Prerequisites" subsection with at least 2–3 paragraphs explaining the geometric intuition and the mapping from Rust types to partitions, or provide links to accessible references.
- **Raised by:** Editor, Domain Expert, Methodologist

## 9. [minor] The documentation oscillates between detailed implementation notes and high-level summaries without a consistent template, and critical design decisions are buried in late sections.

- **Category:** documentation-structure
- **Location:** §Effect System, §Proof & Verification, §Monoidal Diagrams, §Schubert Types
- **Quote:** > "The explanation for why ApplicativeSt is not implemented for ReaderTF and StateTF is placed deep in the Design Notes section."
- **Suggested change:** Enforce a consistent section template (Overview → Core Concepts → API → Examples → Design Notes) across all four areas, and move the Reader/State ApplicativeSt justification to a prominent callout after the trait signatures.
- **Raised by:** Editor, Methodologist

## 10. [info] No performance characterization is provided for closure-based transformers (ReaderT/StateT), including allocation costs, Rc overhead, and cache behavior.

- **Category:** performance-omission
- **Location:** §ReaderTF, §StateTF, §Design Notes
- **Quote:** > "Rc for closure sharing... each call creates an inner closure that needs its own reference."
- **Suggested change:** Add a performance section discussing allocation patterns, Rc refcount overhead, benchmark results, and guidance on when to use transformer stacks vs. simpler alternatives (e.g., `State` vs. `StateT` over `Identity`).
- **Raised by:** Domain Expert, Red Team


_Reproducibility: seed `10084613847725414410`_

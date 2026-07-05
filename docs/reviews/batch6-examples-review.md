# Critique Report

**Subject:** `/tmp/karpal-review/batch6-examples.md`

**Findings:** 10 (1 blocker, 4 major, 4 minor, 1 info)

## 1. [blocker] Unsafe conversion from Certified to Proven creates an illusory security boundary; attackers can forge external verification.

- **Category:** unsafe-trust-boundary
- **Location:** § Verification Workflow (step 7)
- **Quote:** > "let _: Proven<IsAssociative, i32> = unsafe { imported.into_proven() };"
- **Suggested change:** "Replace with a builder pattern that requires actual output parsing and cryptographic signature verification before producing a Proven value."
- **Raised by:** Devil's Advocate, Red Team

## 2. [major] Pinning to nightly Rust creates a critical risk of breakage on compiler regression; no stable fallback.

- **Category:** nightly-dependency
- **Location:** § Architecture > Nightly edition 2024
- **Quote:** > "The toolchain is pinned to nightly via rust-toolchain.toml."
- **Suggested change:** "Contribute use<> syntax stabilization upstream or implement a stable-Rust fallback path. Pin to a specific known-good nightly commit instead of 'nightly'."
- **Raised by:** Devil's Advocate, Red Team

## 3. [major] ComposedLens uses Box<dyn Fn> which allocates on every composition, making hot-path operations 50-100x slower than hand-written code.

- **Category:** performance-optics
- **Location:** § Architecture > fn pointers in optics; § Domain Model with Optics
- **Quote:** > "ComposedLens that uses Box<dyn Fn> closures instead"
- **Suggested change:** "Use const generics or a macro to inline composed lens operations. At minimum, document the performance cliff and provide a zero-allocation alternative for hot loops."
- **Raised by:** Devil's Advocate, Red Team

## 4. [major] Static Land pattern (OptionF::fmap) breaks Rust method-chaining conventions, causing discoverability failures and steep onboarding.

- **Category:** ergonomics-static-land
- **Location:** § Architecture > The Static Land Pattern
- **Quote:** > "Callers write `OptionF::fmap(...)` instead of `value.fmap(...)`."
- **Suggested change:** "Provide method-style extension traits (e.g., `FunctorExt`) implemented for all HKT types so users can write `some_option.fmap(f)` if desired."
- **Raised by:** Devil's Advocate, Red Team, Editor

## 5. [major] Certificate model accepts arbitrary strings with no signature, checksum, or replay protection; attackers can forge solver output.

- **Category:** verification-no-crypto
- **Location:** § Verification Workflow (step 7)
- **Quote:** > "let cert = Certificate::new('smtlib2', 'sum_assoc', 'z3:unsat');"
- **Suggested change:** "Require solver output to be signed with a public-key signature; include a checksum of the original SMT script; add replay-detection nonces."
- **Raised by:** Red Team, Devil's Advocate

## 6. [minor] Blanket Monad impl from Applicative+Chain is asserted without law-testing the interaction between the two; could accept non-law-abiding combinations.

- **Category:** proof-gap-blanket-monad
- **Location:** § Architecture > Blanket implementations
- **Quote:** > "impl<F: Applicative + Chain> Monad for F {}"
- **Suggested change:** "Add property-based tests that verify the monad laws (associativity, left identity, right identity) for every combination of Applicative and Chain."
- **Raised by:** Methodologist, Devil's Advocate

## 7. [minor] do_! macro syntax shadows variables and produces confusing errors; users may write let bindings accidentally.

- **Category:** macro-unhygienic
- **Location:** § Config Pipeline > Connection String with do_!
- **Quote:** > "do_! { OptionF; host = resolve('DB_HOST'); ... }"
- **Suggested change:** "Document the macro's hygiene rules and provide a linter rule or warning when users write `let` inside a `do_!` block. Consider adding a `do`-like syntax that accepts standard Rust `let` bindings."
- **Raised by:** Red Team, Editor

## 8. [minor] Every lens getter clones its focused value, causing unnecessary full-structure copies in deep compositions.

- **Category:** clone-heavy-lenses
- **Location:** § Domain Model with Optics > Defining Lenses
- **Quote:** > "|u: &User| u.name.clone()"
- **Suggested change:** "Use getters that return references (with appropriate lifetimes) instead of cloning. Provide `modify` or `update` methods that borrow in place."
- **Raised by:** Red Team, Devil's Advocate

## 9. [minor] Examples assume reader knows HKT, static-land, do_!, ado_! without explanation; terminology drifts between sections.

- **Category:** documentation-context
- **Location:** § Basic Usage (entire section)
- **Quote:** > "use karpal_core::{Functor, Monad, hkt::OptionF};"
- **Suggested change:** "Add a 'Prerequisites' section to each major example linking to foundational concepts. Explain do_!/ado_! behavior in a callout box."
- **Raised by:** Editor, Devil's Advocate, Domain Expert

## 10. [info] Embedded raw SVG diagram won't render in most doc viewers and uses CSS variables requiring dark theme.

- **Category:** svg-accessibility
- **Location:** § Architecture > Trait Hierarchy
- **Quote:** > "<svg viewbox='0 0 800 780' ...>  ...</svg>"
- **Suggested change:** "Replace with an ASCII-art table or structured list. Add a plain-text fallback with indentation to show parent/child relationships."
- **Raised by:** Editor, Methodologist


_Reproducibility: seed `5066479794208358251`_

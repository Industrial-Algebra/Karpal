# Phase 12 Trust Model

`karpal-verify` adds an external-proof boundary to Karpal. That boundary must
stay explicit.

## Principle

There are now four qualitatively different kinds of evidence in the project:

1. **Type-level / impossible**
   - Rust trait bounds and phantom witnesses
   - Example: `Proven<IsMonoid, T>::from_monoid(value)`
2. **Statistical / rare**
   - proptest, Monte Carlo, Hoeffding-style confidence arguments
3. **Runtime / emergent**
   - dynamic checks and discovered counterexamples
4. **External / imported**
   - SMT solver results, Lean proofs, or other prover artifacts

These should not be conflated.

## Rule

External evidence must not silently produce `Proven<P, T>`.

Instead, imported evidence first becomes:

- `Certified<B, P, T>`

where:

- `B` names the backend (`SmtCertificate`, `LeanCertificate`, ...)
- `P` is the claimed property
- `T` is the wrapped value

Only an explicit `unsafe` conversion may cross into:

- `Proven<P, T>`

This keeps trust reviewable in code and makes imported assumptions searchable.

## Why `unsafe`

`Proven::axiom` already marks unchecked proof introduction as unsafe.
`karpal-verify` follows the same policy:

- external certificates are useful
- external certificates are not compiler-checked by Rust itself
- therefore they must remain an explicit trust boundary

## Intended review flow

1. Generate `Obligation`
2. Export to SMT-LIB2 or Lean 4
3. Discharge the obligation externally
4. Store backend metadata in `Certificate`
5. Import as `Certified<B, P, T>`
6. Convert to `Proven<P, T>` only at carefully-audited boundaries

## Non-goal

This crate does **not** claim that all SMT or Lean outputs are equally trusted.
The certificate metadata is intentionally lightweight today; later phases can
add stronger provenance, digests, replay data, or signed artifacts.

# Verified Domain API

This example shows how `karpal-proof` and `karpal-verify` fit together at an API boundary: a domain type accepts `Proven<P, T>` internally, but externally produced evidence must first cross through `Certified<B, P, T>` and an explicit trust handoff.

## Domain goal

Suppose your domain wants to work only with values whose combine operation is known to be associative. Instead of accepting a raw `T`, you can require `Proven<IsAssociative, T>`.

``` rust
use karpal_proof::{IsAssociative, Proven};

#[derive(Debug, Clone)]
struct VerifiedAccumulator<T> {
    inner: Proven<IsAssociative, T>,
}

impl<T> VerifiedAccumulator<T> {
    fn new(inner: Proven<IsAssociative, T>) -> Self {
        Self { inner }
    }
}
```

This is the `karpal-proof` style: the domain API states the required law as a type-level precondition.

## Rust-native entry point

If the value already comes from a trusted Rust-side witness constructor, the API is straightforward:

``` rust
use karpal_proof::Proven;

let proven = Proven::from_semigroup(5i32);
let acc = VerifiedAccumulator::new(proven);
```

Here the value enters through Rust-native evidence. No external trust boundary is involved.

## External certificate entry point

Now consider the case where associativity was established by an external prover. `karpal-verify` deliberately prevents that evidence from silently becoming `Proven<...>`.

``` rust
use karpal_proof::{IsAssociative, Proven};
use karpal_verify::{Certificate, Certified, SmtCertificate};

let cert = Certificate::new("smtlib2", "sum_assoc", "z3:unsat");
let certified = unsafe {
    Certified::<SmtCertificate, IsAssociative, i32>::assume(5, cert)
};

// Still not a Proven<...> value.
let proven: Proven<IsAssociative, i32> = unsafe { certified.into_proven() };
let acc = VerifiedAccumulator::new(proven);
```

The two explicit `unsafe` steps are the point: code review can find and audit imported trust boundaries.

## Boundary design pattern

A useful pattern is to keep the unsafe conversion at one narrow boundary function and expose only safe APIs elsewhere:

``` rust
use karpal_proof::{IsAssociative, Proven};
use karpal_verify::{Certified, SmtCertificate};

fn import_associative_i32(
    certified: Certified<SmtCertificate, IsAssociative, i32>,
) -> VerifiedAccumulator<i32> {
    let proven = unsafe { certified.into_proven() };
    VerifiedAccumulator::new(proven)
}
```

This keeps the imported-proof decision explicit and localized.

## Why this matters

- **`karpal-proof`** gives your domain rich law-aware APIs.
- **`karpal-verify`** lets external provers feed those APIs without erasing trust provenance.
- **The combination** means you can keep public APIs principled while still integrating with SMT and full Lean workflows, including project-aware execution, structured diagnostics, and archived verification artifacts.

## Recommended usage

1.  Design internal domain APIs around `Proven<P, T>` and refinement wrappers.
2.  Model and discharge external obligations with `karpal-verify`.
3.  Import certificates as `Certified<B, P, T>`.
4.  Convert to `Proven<P, T>` only in a small, audited boundary layer.

For the broader export/execution workflow, see [Verification Workflow](verification-workflow.md). For the API overview, see [Proof & Verification](../reference/proof-verification.md). For CI/report/archive details, see [Verification CI Workflow](../reference/verification-ci.md), and for serialized compatibility details see [Verification Schemas](../reference/verification-schemas.md).


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).



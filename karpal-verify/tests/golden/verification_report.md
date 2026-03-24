# Verification Report

- Schema version: `1`
- Bundle: `sum_monoid`
- Root: `target/karpal-verify-golden`
- Successes: 0
- Failures: 3

| Obligation | Status | Artifact | Lean theorem | Lean diagnostics | SMT certificate | Lean certificate |
|---|---|---|---|---|---|---|
| `associativity` | `DryRun` | `target/karpal-verify-golden/smt/associativity.smt2` | `KarpalVerify.associativity` | `-` | `-` | `-` |
| `left_identity` | `DryRun` | `target/karpal-verify-golden/smt/left_identity.smt2` | `KarpalVerify.left_identity` | `-` | `-` | `-` |
| `right_identity` | `DryRun` | `target/karpal-verify-golden/smt/right_identity.smt2` | `KarpalVerify.right_identity` | `-` | `-` | `-` |

Lean module: `KarpalVerify`
Lean theorems: 3
Lean imports: 0
Lean aliases: 0
Lean diagnostics: 0
Lean theorem failures: 0
Lean certificate: `-`

Report files:
- Schema version: `1`
- JSON: `target/karpal-verify-golden/summary.json`
- Markdown: `target/karpal-verify-golden/summary.md`
- Lean diagnostics JSON: `target/karpal-verify-golden/summary.lean-diagnostics.json`
- Lean manifest: `target/karpal-verify-golden/lean/KarpalVerify.manifest.json`

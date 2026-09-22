# D-0005 — R-0022: the red-first deviation, and AC4's sampled clause

- **Date:** 2026-09-22
- **Decided by:** **Gustavo**, 2026-09-22 — both dispositions approved as
  drafted. Raised by the architect review of PR #96 and written up by the main
  session so neither item lived only in a PR description.
- **Touches:** [R-0022](../requirements/0022-gate2-witness.md),
  [SPEC-0022](../specs/0022-gate2-witness.md) §2.1,
  [PR #96](https://github.com/westerngazoo/ufl/pull/96)

## 1. Red-first was not observed for `witness.rs`

CLAUDE.md §1.4 says a failing test exists before the code that satisfies it,
"no exceptions." For R-0022 the main session wrote `crates/ufl-evolve/src/witness.rs`
while the qa agent was still deriving the acceptance suite, so the suite compiled
and went green on its first run. That is a process failure, not a code defect,
and it is the main session's.

**What was done instead.** The qa agent reconstructed the gate four ways and
reverted every mutation, verifying `witness.rs` byte-identical afterwards by
sha256:

| reconstruction | result |
|---|---|
| module moved out of the tree | `error[E0583]: file not found for module witness` |
| rotor plane `E12` → `Basis(5)` | 5 tests fail; **`structure` still passes** |
| `fk_motor` → `Basis(1)` (a reflection) | `structure` fails naming guard 1/3 EVEN |
| `measure`'s denominator `2n` → `n` | **only** `metric_is_per_component` fails |

The second row is worth keeping on the record: it is independent confirmation of
§2.1's division of labour — the structural guard proves *a* proper rigid motion,
and only the numerics pin *which*.

The committed history does show red → green: the acceptance suite lands first,
in a commit that does not compile, and the module lands second. That ordering was
reconstructed deliberately after the fact; it was not how the work happened.

**Why this is recorded rather than waived.** D-0002 established that a
substituted verification is acceptable *when the substitution is named and its
evidence is shown*. The same standard applies to a substituted red: the gate was
re-established by mutation rather than observed by construction, and a reader of
the history should be told which.

## 2. AC4 is partially met: one clause of the guard is sampled, not proved

R-0022 **AC4** requires the derivation's structural claim be "verified
independently of its numerics." SPEC-0022 §2.1's guard has three clauses:

| clause | kind |
|---|---|
| `typecheck(fk_motor, ctx) == Ok({0,2,4})` — even | **structural**: all inputs, nothing evaluated |
| `typecheck(fk_witness, ctx) == Ok({3})` — point space | **structural** |
| `M ∗ M̃ == 1` — unit | **sampled**: 409 poses, worst 4.44e-16 (2 ulp), worst residue 1.11e-16 |

The third is numeric because `ufl-geo` has no versor judgement to appeal to —
`grade.rs:79-82` says so in terms: *"Not 'is a versor': `Basis(8) = e₀` is null
and non-invertible, yet the flag is `true` and sound."* Widening the pose sweep
cannot close this; 409 poses and 4,000,000 poses are the same kind of evidence.

**The construction that would close it**, identified by the architect review and
not built here: a syntactic predicate over the motor tree — every factor an `Exp`
of a scalar-chain times a bivector from a whitelist whose square is evaluated
once, with `GeoProduct` the only combinator. Unitness then follows in ℝ by
induction: `(cos + sin B̂)(cos − sin B̂) = cos² − sin²B̂² = 1` for `B̂² = −1`,
`(1 + uB)(1 − uB) = 1` for null `B`, and a product of unit versors is unit. The
sweep would then measure f64 rounding rather than establish unitness.

**Disposition (approved).** AC4 is *partially met* for R-0022, with the
honest label **"unit by algebraic identity, f64 residual 2 ulp"** in §2.1, and
defer the syntactic predicate. Grounds: the predicate belongs in `ufl-geo`
beside `typecheck`, and SPEC-0011 §7's 2026-06-21 decision freezes that crate;
building it inside `ufl-evolve` would put a grade-system judgement in the wrong
crate to satisfy one requirement.

The algebra also bounds how much is actually at risk. For even
`M = s + B + pI` in `Cl(3,0,1)`, `M M̃ = (s² − ⟨B²⟩₀) + (2sp − ⟨B²⟩₄)I`, so
"even ∧ `M M̃ = 1`" is two equations on an 8-dimensional space — the
6-dimensional motor group, up to sign. The sampled clause is pinning a
codimension-2 condition, not standing in for an open-ended one.

## Follow-up

The syntactic versor predicate is **deferred, not dropped** — tracked as
[#97](https://github.com/westerngazoo/ufl/issues/97). It lands when `ufl-geo`
reopens; until then §2.1's label stands and the sampled clause is disclosed
wherever the guard is described.

## What this does not decide

Neither item changes what the witness measures. The Gate-2 result — 25 nodes,
4 `Param`s, flat at ≈1.4e-16 against an 80-digit reference across nine orders of
magnitude of `|θ|` — is independent of both.

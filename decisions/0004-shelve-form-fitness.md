# D-0004 — R-0021 (`FormFitness`) is shelved, and what survives it

- **Date:** 2026-09-16
- **Decided by:** Gustavo, on the three-lens round-1 result
- **Touches:** [R-0021](../requirements/0021-form-fitness.md),
  [SPEC-0021](../specs/0021-form-fitness.md), [#66](https://github.com/westerngazoo/ufl/issues/66)

## The decision

**Shelved, not killed.** The requirement's ACs were approved; the spec then failed
its own three-lens. Architect **REQUEST CHANGES** (3 blocking), hater **NEEDS
WORK** (3 blocking), nice-guy **STRONG WORK** — but both critical lenses
independently concluded the spec as drafted should not be built.

## Why

1. **Three of the spec's own claims were falsified by measurement** (SPEC-0021
   §7.1): the C1/C3 security argument (an `eml`-encoded zero writes the form with
   no verifier binding at all), the "two probes pin the meaning" claim
   (`(not (= residual one))` passes both and accepts every residual ≠ 1), and the
   envelope guard (wrong at `i64::MIN`, and unnecessary — `r as f64 == 0.0 ⟺
   r == 0` for all `i64`, fuzzed 200,261 values).
2. **The named beneficiary is closed.** R-0021 exists to make C3 *structural* for
   the Rung-4 meta-loop. R-0015 is a **documented negative** — no headroom window
   on any substrate, the probe was never armed. Infrastructure for a loop that may
   never run.
3. **It does not generalise** (R-0021 §1.1): the form language has no ordering
   relation, so form-as-acceptance works only for a lane whose criterion is an
   exact `== 0`.
4. **Opportunity cost.** R-0022 — the Gate-2 witness — has an external audience
   and was ready to build. R-0021's value is legible only to someone who already
   accepts the two-language thesis.

## What survives, and where it lives

Shelving a requirement must not lose what it found. Three things outlived it:

| finding | where it now lives |
|---|---|
| **The form language has no ordering relation** — #66's KILL condition, which *does* fire, just not for matmul. The missing form is one ordering head, and it is not a drop-in because `Value` has no total order. | **R-0021 §1.1**, hoisted to the top of the shelved requirement so it is findable without reading the spec |
| **VHT has a hole in the existing seam** — a `CheatingProposer` built from public API only made 500,000 verifier consultations against a `Ledger` reporting 5 | [#94](https://github.com/westerngazoo/ufl/issues/94) — a live issue about `main`, independent of R-0021 |
| **`solved` is called once per generation, not per candidate** (132 vs 7,200 on the sweep); a form discharge is 283 ns and **0.489%** of run wall-clock | SPEC-0021 §7.2 — retires the "forms are too slow" objection for any future attempt |

And one rule worth carrying forward, derivable from two independently-written
lanes: **the cost type owns the binding's totality; the form only compares.**

## What would un-shelve it

SPEC-0021 §7.4's rescope — ~100 lines: a `pub(crate)` `FormFitness`, the form
carrying **both** conjuncts of `RankDecomposition::discharge` (not just the
residual one, which is what `MatmulFitness::solved` checks and what the drafted
form transcribed), a probe set of `{0, 1, −1, 7}`, no envelope guard, and AC6
reported per-run rather than per-candidate.

The trigger is **Rung-4 earning headroom** — i.e. R-0015 being reopened on new
evidence. Until then this is the right thing not to build.

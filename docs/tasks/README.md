# Task set — the evolutive self-eval language

Enumerated task briefs for the dev team, produced by the 2026-06-30 five-lens
review of the deployed state (origin/main @ `3d5422f`, post-R-0014-merge) against
the goal: **UFL becomes a language that evaluates its own programs as first-class
data and evolves its own programs — and eventually its own search operators —
under the verifier-held, can't-fool-itself discipline.** Each file is
self-contained: context, concrete work, a falsifiable acceptance gate, and what it
must NOT claim. Constitution applies to every task (requirement + spec before
code, test-first, no lib panics, typed errors — `CLAUDE.md`).

## ⚠ Read first — urgent findings from the triage

1. **PR #38 is a revert bomb.** It carries a MERGEABLE badge but its stale tree
   **deletes the R-0014 merge** (`generic.rs`, the seam tests, `requirements/0014`,
   both theory docs). Do not merge; close with a supersession note (T2).
2. **The exact verifier has a silent-wrong regression.** Merged #46 made
   `Tensor::add_at` silently no-op on out-of-bounds
   (`crates/ufl-tensor/src/tensor.rs:40-45`) while `reconstruct()` — the core of
   the exact verifier — calls it. Silent-wrong inside the verifier is the one
   failure mode the discipline forbids. Fixed as a precondition inside T1.
3. **No CI exists** — the constitution's §5 hard merge gate is aspirational; two
   open PRs sit failing `fmt --check` undetected, and the fleet produced finding
   (1) unnoticed. T3 closes this.
4. **Junk on main:** a checked-in binary `test_size_hint` (+ `.rs`) at repo root
   (slipped in around #48). Removed by T2.

## The thesis (one paragraph)

Exactly one rung of the staircase exists as committed code today (the answer-blind
`Proposer`/`Fitness` seam). Everything above it is prose, deleted pilots, or a red
test. So: **P0 — bank the proven assets and make the substrate honest** (flip-graph
green, PR fleet to zero, CI, the staircase written as numbered requirements,
quote/eval/raise on one coherent depth contract, a hardened generic seam);
**P1 — climb** (memetic lane, held-out statistics, fitness-as-forms, then the
make-or-break Rung-4 probe: a bounded MoveForm DSL under a pre-registered 2-SE
held-out gate that decides whether operator-semantics evolution is real);
**P2 — deepen** (bounded in-language eval, grade-harness tower). Rung-5 (the Lisp
substrate) is deliberately absent — it is earned only by a Rung-4 positive.

## The tasks

| # | Task | Priority | Rung | Depends on |
|---|------|----------|------|------------|
| [01](01-bank-the-flip-graph-land-r-0013-gate-0-green.md) | Bank the flip-graph: land R-0013 Gate-0 green | **P0** | 0 | — |
| [02](02-triage-the-pr-fleet-merge-close-de-junk-refr.md) | Triage the PR fleet: merge, close, de-junk, refresh ROADMAP | **P0** | 0 | — |
| [03](03-add-ci-enforcing-the-constitution-s-merge-ga.md) | Add CI enforcing the constitution's merge gate | **P0** | 0 | — |
| [04](04-write-r-0015-and-r-0016-number-the-staircase.md) | Write R-0015 and R-0016: number the staircase, pre-register gates | **P0** | 1+4 | — |
| [05](05-implement-quote-eval-raise-reflection-rung-1.md) | Implement quote/eval/raise — reflection rung 1 | **P0** | 1 | T4, T6 |
| [06](06-adopt-one-depth-contract-across-read-print-l.md) | Adopt one depth contract across read/print/lower/eval/Drop | **P0** | 1 | — |
| [07](07-write-spec-0014-harden-run-generic-errors-sc.md) | Write SPEC-0014; harden run_generic: errors, screen, eval ledger | **P0** | 0 | — |
| [08](08-land-ufl-evolve-memetic-engine-refiner-seam.md) | Land ufl-evolve memetic engine: Refiner seam + typecheck pruning | P1 | 0 | T2, T7 |
| [09](09-commit-the-c2-c6-held-out-split-se-harness-a.md) | Commit the C2/C6 held-out split + SE harness as real code | P1 | 0 | T7 |
| [10](10-add-formfitness-verifier-held-fitness-as-dis.md) | Add FormFitness: verifier-held fitness as discharged UFL forms | P1 | 3 | T7 |
| [11](11-run-the-rung-4-probe-moveform-dsl-meta-loop.md) | Run the Rung-4 probe: MoveForm DSL, meta-loop, 2-SE gate | P1 | 4 | T1, T4, T7, T9 |
| [12](12-discharge-r-0014-ac3-eml-nand-truth-table-ma.md) | Discharge R-0014 AC3: eml-NAND truth table + matmul-entry probe | P1 | 2 | T7 |
| [13](13-un-shelve-r-0005-spec-guarded-recursion-boun.md) | Un-shelve R-0005; spec guarded recursion; bounded in-UFL eval | P2 | 2 | T5, T12 |
| [14](14-deepen-the-grade-harness-fuzz-coverage-blade.md) | Deepen the grade harness: fuzz coverage, blade support, contracts | P2 | — | — |

## Sequencing — the waves (parallelize inside a wave)

- **Wave 0 (start now, fully parallel):** T2 (triage — close the #38 bomb first,
  merge #33), T3 (CI, so every later PR is machine-gated), T1 (flip-graph, rebased
  onto main), T4 (the two requirements — pure discussion + docs). T6's requirement
  discussion starts here too.
- **Wave 1:** T7 (SPEC-0014 + seam hardening) and T6's implementation; then T5
  (quote/eval/raise) once T4 is Accepted and the depth contract exists — quote
  must not land on an asymmetric read/print codec.
- **Wave 2 (parallel fan-out once T7 lands):** T8, T9, T10, T12.
- **Wave 3 — the convergence point:** T11, the program's **decision node**. A
  positive unlocks a future Rung-5 requirement (Chez↔Rust FFI banking + process
  isolation). A documented negative **kills Rung-5 permanently** and redirects
  effort to object-level scaling (R-0013's T₃ record attempt) plus the reflection
  line (T5/T13), which stands on its own merits either way.
- **Wave 4 (opportunistic):** T13 after T5+T12; T14 in any slack.

## Deliberately not tasked (no silent drops)

- **Rung-5 / the Lisp substrate (Chez FFI, process isolation):** gated behind a
  T11 positive by the repo's own non-sequitur caveat
  (`theory/two-language-substrate.md`) — opening it now would pre-commit what the
  discipline forbids.
- **Scored-genome cache / checkpointing:** no measured duplicate-eval evidence;
  T7's eval ledger is exactly the instrumentation to revisit.
- **Quoted-Sexpr as the Rung-4 genome:** would couple T5 onto T11's critical path
  for no evidentiary gain; recorded in T11 as the follow-on join if the probe pays.
- **PRs #34/#36:** carried only as CLOSE verdicts inside T2 (cold path / zero-diff).

All remaining review quick-wins are folded into named task bodies (ROADMAP refresh
→ T2; `add_at` fail-loud + SPEC-0013 §2.1 amendment → T1; #38 salvage → T6;
`is_versor` doc + `Tensor::zeros` expect() → T14; trees≅sexprs theory note → T5).

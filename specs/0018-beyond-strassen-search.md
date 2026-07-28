# SPEC-0018 — Rectangular flip-graph + the plateau walk (beyond-⟨2,2,2⟩)

- **Realizes:** [R-0018](../requirements/0018-beyond-strassen-search.md) (Accepted) —
  a certified matmul reduction beyond the ⟨2,2,2⟩ special case, via a square-embedded
  rectangular flip-graph + the existing eager-reduce loop. The falsifiable gate —
  **⟨2,2,3⟩ 12→11 certified ternary** — is **MET** (§3).
- **Status:** **Accepted** (2026-07-24) — three-lens complete and all findings
  folded: nice-guy *STRONG WORK*; architect *approve-with-changes* (verified the
  square-embedding is **isomorphic**, not merely type-compatible); hater *NEEDS-WORK
  on the rationale while **winning the gate*** → the borrowed "10⁸/fixed-rank" claim
  replaced by the measured "eager-reduce @10⁶" reality (§0, §2, R-0018 §3c). AC2 is
  **met** (⟨2,2,3⟩ 12→11 certified, independently reproduced). Implemented in PR #77;
  Gustavo holds final approval there.
- **Deferred (recorded, PR #77 architect finding 5):** §4 T5 (the plateau
  level-ratio diagnostic) is **not implemented**. The §2.1 ratios (~6–12% level-1;
  ≈1:135,000 ternary) are therefore **one-off pilot observations, not committed
  measurements** — cited as such, never as regression-guarded facts. Landing T5 is a
  follow-up. Related measured correction: the `ENVELOPE` cap is **strongly binding**
  (66–79% of frontier draws refused, peak |c| ≈ 65,533), which SPEC-0013's "does not
  constrain the walk" comment denied — corrected in `flipgraph.rs`, and flagged as an
  open question for the §5 lift (the envelope must become a *workspace policy*, not a
  module constant, before a non-matmul workspace reuses the walk).
- **Crate:** `ufl-discovery` only (extend `flipgraph` with one constructor +
  `reduce_matmul_rect`). **No `ufl-tensor` change** — the square-embedding reuses the
  existing square `Tensor`/`for_target` (§1.3, verified).
- **Depends on:** SPEC-0013 (the flip-graph — its primitives are already
  dim-agnostic), R-0014's `RankDecomposition` (the exact verifier).

---

## 0. What is measured (the gate is WON)

The early pilot (R-0018 §3b) diagnosed the 12→11 reduction as *behind a plateau*
(no single flip from naive lowers the rank) — correct. Its conclusion ("needs
10⁸–10⁹ flips and a fixed-rank walk") was **borrowed from KM's ⟨3,3,3⟩ record scale,
not measured** for ⟨2,2,3⟩ — and the hater caught it (R-0018 §3c). **Measured, and
reproduced independently:** the existing **eager-reduce loop** crosses ⟨2,2,3⟩ 12→11
and certifies a **rank-11 ternary scheme at 10⁶ flips** (seed 3); a fixed-rank walk
is measured *worse* (0/32). So the two load-bearing changes are small: **(1)**
rectangular targets via a **square-embedding** (§1 — verified, no `ufl-tensor`
change), and **(2)** run the *existing* loop over the embedded naive at a modest
budget, with its `is_ternary()` early-stop as the terminal filter (§2.1). No new
science, no new driver, no record-scale compute — the certified beyond-Strassen
scheme falls out of the proven primitives.

## 1. Rectangular support via square-embedding (the enabling infra)

### 1.1 The square-embedding — no `ufl-tensor` change, no `IntScheme` change

**VERIFIED (2026-07-24, before this revision).** ⟨m,n,p⟩ has unequal slot lengths
(`m·n`/`n·p`/`m·p`), which `ufl_tensor::Triple::new` **rejects** as `Ragged`
(`scheme.rs:47`) — the wall the first draft flagged. The clean fix is to **pad every
slot to `d = max(m·n, n·p, m·p)`**, embedding ⟨m,n,p⟩ into a **square `d×d×d`**
tensor with *structural zeros* in the unused dimensions. Then:

- all slots are length `d` → `Triple::new` accepts them (no `Ragged`);
- the target is a genuine `d×d×d` `Tensor` → the **existing square**
  `reconstruct` + `RankDecomposition::for_target(target, rank).discharge` certify it
  **with no `ufl-tensor` change** (measured: `for_target(padded⟨2,2,3⟩, 12)
  .discharge(padded-naive) = Ok(true)`; a one-coefficient corruption → `Ok(false)`);
- `IntScheme` keeps its single `dim = d`; `reconstruct_int`/`target_int` stay square;
  the flip **moves** are unchanged (already dim-agnostic — they touch only the
  triples). **So the whole rectangular generalization is ONE new constructor**
  (`naive_embedded(m,n,p)`), not a per-slot-dims refactor. ⟨2,2,2⟩ is untouched
  (`naive_embedded(2,2,2) == naive(2)`, `d=4`) — the T-square-identical gate.

### 1.2 The embedded naive

- `naive_embedded(m, n, p) -> IntScheme` — `d = max(m·n, n·p, m·p)`; one `0/1`
  triple per `(i,j,k)`, `i<m, j<n, k<p`: `u = e_{i·n+j}`, `v = e_{j·p+k}`,
  `w = e_{i·p+k}`, **each a unit vector in length `d`** (the used index < its true
  slot length ≤ d; higher dims structurally zero). Rank `m·n·p`. The structural
  zeros stay zero under every move (flips are linear combos of existing triples,
  whose padded dims are zero), so the walk never leaves the embedded subspace.
- `target_embedded(m,n,p) = reconstruct(naive_embedded).to_tensor` — one source of
  truth; the `reconstruct_int == target` debug invariant (SPEC-0013 §2.4) holds
  unchanged over the `d³` flat image.

*(A note the nice-guy surfaced, worth keeping: because the embedded slots have
genuinely different **used** lengths, the rectangular tests are a **stronger**
slot-hygiene check than square ever was — any accidental slot-crossing that
equal-length square silently tolerated surfaces here. T-rect-naive-exact
retroactively hardens confidence in the certified square primitives.)*

### 1.3 Certification (RESOLVED — the verifier needs no extension)

The caller discharges through `RankDecomposition::for_target(target_embedded, rank)`
(§1.1, measured). `Tensor` is square `(d,d,d)` and already supported; the
`Ragged`-guard-in-`Triple::new` (the "Guard Inside the Candidate" convention) is
sidestepped by padding, not weakened. VHT preserved: the proposer never holds the
verifier; a tensor-breaking move can only *fail* to certify.

## 2. The search — the measured method (R-0018 §3c)

**MEASURED, gate WON (2026-07-24, reproduced independently, seed 3).** The existing
eager-reduce loop — behaviour-identical to the certified square path, now factored
into a shared `walk()` — run over `naive_embedded(2,2,3)`
with `FlipConfig::pinned()`, crosses ⟨2,2,3⟩ 12→11 and returns a **certified rank-11
ternary scheme** (`for_target(target_embedded, 11).discharge = Ok(true)`) at **10⁶
flips**. A fixed-rank walk was measured **worse** (0/32); the earlier "fixed-rank at
10⁸" plan (borrowed from KM's ⟨3,3,3⟩) is withdrawn. So the search is a **budget +
constructor change on the proven loop, not a new driver**:

- **`reduce_matmul_rect(m, n, p, target_rank, seed, budget, config)`** — the existing
  loop over **`naive_embedded(m,n,p)`** (§1.2), verifying against `target_embedded`,
  returning a certified ternary `Scheme` **for the caller to discharge** through
  `for_target` (the driver never self-certifies — VHT) or `NotFound { best_rank }`.

### 2.1 The two-level plateau (why the `is_ternary` early-stop is load-bearing)

The 12→11 crossing has two levels (**pilot-observed, not regression-guarded** —
T5 is deferred, see the Status block):
- **Level 1 — reach rank 11.** ~6–12% of seeds do so at 10⁶; hard seeds cross by 10⁸.
  (This is why *budget helps* — more attempts at the level-1 crossing.)
- **Level 2 — land on a *ternary* rank-11 state.** The rank-11 plateau is
  **~99.999% non-ternary (≈1:135,000)**. The loop's `s.rank() ≤ target ∧
  is_ternary()` early-stop (`flipgraph.rs:473`) **is the terminal filter** — it
  samples the rank-11 plateau every step and stops on the rare ternary moment. More
  steps on the rank-11 plateau ⇒ more ternary draws. **This early-stop is
  load-bearing and must be preserved** — a "detect-drop-only" fixed-rank walk (the
  withdrawn plan) would sit at rank 11 *forever* (rank 10 for ⟨2,2,3⟩ does not exist)
  and never terminate. §4 T5 *would* guard these ratios; it is **deferred** (Status block), so treat the
  numbers as one-off observations.

### 2.2 Budget + config (measured cost, pre-registered)

- Per-flip cost measured **~1.5–2 µs** (⟨2,2,2⟩ 0.68 µs / ⟨3,3,3⟩ 8.19 µs endpoints;
  the ~`R²·d` model predicts ~1.5 µs at ⟨2,2,3⟩) — so **10⁶ ≈ 1.5 s, 10⁸ ≈ 2.5 min**.
  The debug `reconstruct` invariant is `debug_assert!` (off in release — confirmed);
  the release per-flip work is `shared_factor_pairs` + `reduce` only.
- **Config = `FlipConfig::pinned()`** (`stall_window: 400, perturb_flips: 6`) — the
  config that *won* the gate; no sweep is needed for the pinned-seed gate. A
  `(stall_window, perturb_flips)` sweep is an *optional* robustness study (§4 T5),
  re-pinned against a **level-1-crossing seed**, not blindly over non-crossing seeds.

## 3. The gate — WON (AC2 met), pinned like `r_0013`

**Gate (AC2) — MET.** `reduce_matmul_rect(2,2,3, 11, SEED, budget, pinned())` returns
a rank-11 ternary `Scheme` that `RankDecomposition::for_target(target_embedded, 11)
.discharge` certifies **`Ok(true)`** — a certified beyond-Strassen reduction. The
committed e2e **pins a level-1-crossing seed** (measured: seed 3 certifies at 10⁶,
~1.5 s — the `r_0013` seed-5 pattern), so the gate is *deterministic and fast*, plus
a documented `#[ignore]` **robustness run** over a seed block at 10⁸ that reports the
honest hit-count (~6–12% cross level 1). The certified scheme is banked in
`theory/discovery-results.md` (§4 T-gate) — *committed code, not a claim*.

**Existence — SETTLED favorably (not open).** A `{−1,0,1}` rank-11 ⟨2,2,3⟩ scheme
demonstrably exists: the **Strassen-7 ⊕ matvec-4 block** construction (⟨2,2,2⟩ ⊕
⟨2,2,1⟩) certifies `Ok(true)` (corruption `Ok(false)`). So the gate was never
testing a non-existent target; the only question was reachability, now answered.

**Escalation ladder (AC3) — rung 1 succeeded, so 2/3 are moot; retained honestly:**
1. **Eager-reduce loop from `naive_embedded`, pinned config** reaches a certified
   rank-11 → **DONE** (the discovery-direction win, reproduced).
2. *(Not needed — rung 1 succeeded.)* A start-from-known perturb-recover result would
   require a **pre-registered kick size and threshold**: recovery from the known
   optimum is measured k=1 → 11/16, k=3 → 3/16, **k=6 → 0/16** (the walk's own kick),
   so "holds" is only non-trivial for small `k`. Not claimed as a win here.
3. *(Not reached.)* A documented negative (no crossing at the pre-registered budget)
   would have banked the bounded plateau limit. It did cross.

## 4. Tests (TDD — written first, red)

`crates/ufl-discovery/tests/r_0018_rect.rs`:

1. **T-square-identical** — `naive_embedded(n,n,n)` and the rectangular
   reconstruct/verify reproduce the SPEC-0013 square path byte-identically for
   ⟨2,2,2⟩ (the certified rank-7 sweep `r_0013_flipgraph` stays green).
2. **T-rect-naive-exact** — `naive_embedded(m,n,p)` reconstructs to the exact ⟨m,n,p⟩
   tensor for ⟨2,2,2⟩/⟨2,2,3⟩/⟨2,3,3⟩; `reduce` never raises rank; every `flip_at`
   preserves the rectangular tensor (fuzz).
3. **T-verifier-rect** — `for_target(t223, 11).discharge` accepts a known rank-11
   ternary ⟨2,2,3⟩ scheme and **rejects** a one-coefficient corruption (the
   verifier-is-sole-judge contract, rectangular).
4. **T-ternary-exists** (§3 pre-check) — the encoded known rank-11 scheme certifies.
5. **T-plateau-diagnostic** (`#[ignore]`, release) — at a mid budget, report the
   best-rank distribution + the level-1/level-2 ratios (§2.1) as a regression guard.
6. **The gate** (`#[ignore]`, release, the experiment) — `reduce_matmul_rect(2,2,3,
   11, seed, 10⁸, gate_config)` over the pre-registered seeds; print each seed's
   certified rank + the summary N/seeds; **the committed assertion is not "11
   found"** — a documented negative is a valid AC3c result; the test asserts the
   gate *ran at the pre-registered budget/seeds and recorded a verdict*.

## 5. Generality (the vision — noted, NOT built yet)

R-0018's success would make this the first instance of a general correctness-first
discovery engine — the `ufl-search` seam's shape (workspace + correctness-preserving
moves + cost + unreachable exact verifier), which already hosts the GA and memetic
lanes. **Per CLAUDE.md §2 (no premature abstraction), the plateau walk stays
matmul-specific in `ufl-discovery` until the gate is met.** *If* AC2 succeeds, a
follow-up requirement lifts the walk into `ufl-search` as `run_walk<Workspace, Move>`.
**The named nearest second domain is boolean-circuit minimization via the
R-0014-AC3 eml-NAND verifier (nice-guy)** — a recombination of two in-repo artifacts:
R-0014 AC3's **exact, finite truth-table** oracle + this plateau walk, with workspace
= NAND trees (transiently non-canonical, exactly the flip-graph's off-the-ternary-set
trick), moves = truth-table-preserving rewrites, cost = gate count (the rank
analogue). It **preserves the exact-verifier differentiator** where the geometric
lane weakens it (`ufl-geo` equivalence is only *numerical* over sampled multivectors).
This "prove the instance, then lift" is UFL's *demonstrated* method (SPEC-0014
hardened the `run_generic` traits in `ufl-discovery` before the physical `ufl-search`
crate; SPEC-0011M placed `Refiner` in `ufl-search` only after the geometric instance
worked) — building the abstraction before the instance works is the anti-pattern this
project rejects.

### Open questions for the three-lens
1. **RESOLVED (measured):** the existing **eager-reduce** loop crosses ⟨2,2,3⟩
   12→11 at 10⁶ (certified, seed 3); a fixed-rank walk is *worse* (0/32). No fork.
   *(superseded question:)* ~~does the existing eager-reduce loop
   cross the plateau at 10⁸, or must the search wander at fixed rank? (Measured in
   T-plateau-diagnostic; the spec should not pre-decide.)
2. **RESOLVED (2026-07-24):** no `ufl-tensor` change — the **square-embedding**
   (§1.1) passes `Triple::new` and certifies via the existing `for_target` (measured
   `Ok(true)`; corruption `Ok(false)`).
3. **Budget honesty:** 10⁸ ≈ 4–5 min/seed (measured ~2–3 µs/flip); the gate
   pre-registers 10⁸ + a small seed block, with 10⁹ as a documented opt-in second
   run. Is that the right ceiling, or should it escalate 10⁷→10⁸→10⁹ and report the
   first crossing?
4. **The `{−1,0,1}` constraint:** the flip workspace is unrestricted `i64`; only the
   *final* state must be ternary. Is a known rank-11 ⟨2,2,3⟩ scheme ternary, and is
   the ternary end-state reachable, or does the plateau live off the ternary set?

## 6. Non-goals

Beating ⟨3,3,3⟩ (measured out of reach). No RL/LLM mutation, no GPU, no numerical
schemes. No general `run_walk` abstraction yet (§5). No change to the merged square
⟨2,2,2⟩ behaviour (T-square-identical guards it).

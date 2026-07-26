# SPEC-0018 — Rectangular flip-graph + the plateau walk (beyond-⟨2,2,2⟩)

- **Realizes:** [R-0018](../requirements/0018-beyond-strassen-search.md) (ACs agreed
  2026-07-24) — a certified matmul reduction beyond the ⟨2,2,2⟩ special case, via
  rectangular support + a record-scale plateau walk. The falsifiable gate is
  **⟨2,2,3⟩ 12→11 certified**.
- **Status:** **Draft — round 1 three-lens: nice-guy STRONG WORK; architect + hater
  died on infra errors but both surfaced the `Triple`-equal-length feasibility wall,
  now RESOLVED via the verified square-embedding (§1.1, §1.3).** Revised 2026-07-24
  (square-embedding, measured budget cost, boolean-minimization second domain).
  Architect + hater **re-review pending** on this revision.
- **Crate:** `ufl-discovery` (extend `flipgraph`), `ufl-tensor` (rectangular
  `Tensor`/verifier — feasibility flagged for the lens, §1.3).
- **Depends on:** SPEC-0013 (the flip-graph — its primitives are already
  dim-agnostic), R-0014's `RankDecomposition` (the exact verifier).

---

## 0. What the de-risk already settled (so the spec targets the right thing)

R-0018 §3b measured, before this spec: **⟨2,2,3⟩ naive-start random-flip 0/12 and
greedy steepest-descent 0/20**, and **no single flip from naive lowers the rank** —
the 12→11 reduction is *behind a plateau*. Our pilot ran ~10⁵ flips; the
Kauers–Moosbauer method crosses these plateaus with **10⁸–10⁹ flips**. So the two
load-bearing changes are: **(1)** the flip-graph must accept **rectangular** targets
(⟨2,2,3⟩ is not square), and **(2)** the search must run at a **record-scale budget**
with a plateau policy tuned for long constant-rank wandering. Neither is new science
— both are engineering the existing, proven primitives to the regime the method
actually needs.

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

## 2. The plateau walk (the search)

The existing `reduce_matmul_with` loop **is** a plateau walk — it accepts
rank-preserving flips and perturbs the best-so-far on a stall. The changes are
budget and policy, not structure:

- **`reduce_matmul_rect(m, n, p, target_rank, seed, budget, config)`** — the same
  loop over `naive_rect(m,n,p)`, verifying against the rectangular target,
  returning a certified ternary `Scheme` (via `to_scheme` + the caller's
  `for_target` discharge) or `NotFound { best_rank }`.
- **Budget to the regime the method needs — measured cost, not guessed.** Per-flip
  cost was timed (architect): ⟨2,2,2⟩ 0.68 µs, ⟨3,3,3⟩ 8.19 µs; ⟨2,2,3⟩ (embedded
  d=6, rank ~11–12) brackets to **~2–3 µs/flip**. So **10⁸ ≈ 4–5 min per seed**
  (feasible); a *seed block* of 8 → ~40 min; a 10⁹ escalation → ~40 min–hours per
  seed. The gate pre-registers `budget = 10⁸` and a **small seed block** (the e2e is
  `#[ignore]`, release); the 10⁹ escalation is a *documented, opt-in* second run, not
  the default — the honest wall-clock is stated so "run the gate" is actually
  runnable. **The debug `reconstruct` invariant must be `debug_assert!`** (off in
  release) or the per-flip cost explodes — confirmed it is (`flipgraph.rs`).
- **A plateau-tuned `FlipConfig`:** the pilot used `pinned() = {stall_window: 400,
  perturb_flips: 6}`. §2.7 pre-registers a small sweep of `(stall_window,
  perturb_flips)` for the gate — the plateau-walk literature favours *large*
  stall windows (wander far before perturbing) and *small* kicks; the pinned gate
  config is frozen from the sweep before the certified run.
- **The eager-reduce question (an explicit spec fork, §5 OQ1).** Today's loop
  eager-`reduce`s after every flip. The KM method walks at *fixed rank* and treats a
  reduction as a rare event. §2.7's **T-plateau-diagnostic** measures, at 10⁸: does
  eager-reduce ever cross the ⟨2,2,3⟩ plateau, or must the loop switch to a
  fixed-rank walk (accept rank-equal flips, detect a reduction only when the whole
  scheme drops)? The answer is *measured* in the pilot phase, not assumed here — if
  eager-reduce crosses, the change is budget-only; if not, §2.6 adds the fixed-rank
  variant. Either way the gate is the arbiter.

## 3. The gate + the escalation ladder (AC2/AC3)

**Gate (AC2):** `reduce_matmul_rect(2,2,3, 11, seed, 10⁸, gate_config)` returns a
scheme that `RankDecomposition::for_target(t223, 11).discharge` certifies `Ok(true)`,
for **at least one** pre-registered seed, within a laptop run. Pre-register the seed
block and the config; report the honest count.

**Escalation ladder (AC3), pre-registered, every rung a result:**
1. **Fixed budget 10⁸, plateau walk from naive** reaches rank-11 → **done** (the
   discovery-direction win).
2. Else **start-from-a-known ⟨2,2,3⟩ rank-11 scheme + perturb-and-recover** holds
   (local search returns to 11 after a `k`-flip kick) → the *navigability* result:
   the landscape near the optimum is connected, only the cold-start is hard.
3. Else **documented negative** (R-0018 AC3c): the correctness-first flip-graph does
   not cross the ⟨2,2,3⟩ plateau at 10⁸ flips — banked in
   `theory/discovery-results.md` with the measured best-rank and the plateau
   diagnostic. Bounded, diagnosed, real.

**Ternary-existence pre-check (before the gate is armed):** confirm a `{−1,0,1}`
rank-11 ⟨2,2,3⟩ scheme *exists* (encode a known Hopcroft–Kerr/Smirnov scheme and
verify `for_target(...).discharge == Ok(true)` + `is_ternary`). If none exists, the
gate re-scopes to the smallest ternary-reachable reduction and says so — measured,
not assumed (the discipline).

## 4. Tests (TDD — written first, red)

`crates/ufl-discovery/tests/r_0018_rect.rs`:

1. **T-square-identical** — `naive_rect(n,n,n)` and the rectangular
   reconstruct/verify reproduce the SPEC-0013 square path byte-identically for
   ⟨2,2,2⟩ (the certified rank-7 sweep `r_0013_flipgraph` stays green).
2. **T-rect-naive-exact** — `naive_rect(m,n,p)` reconstructs to the exact ⟨m,n,p⟩
   tensor for ⟨2,2,2⟩/⟨2,2,3⟩/⟨2,3,3⟩; `reduce` never raises rank; every `flip_at`
   preserves the rectangular tensor (fuzz).
3. **T-verifier-rect** — `for_target(t223, 11).discharge` accepts a known rank-11
   ternary ⟨2,2,3⟩ scheme and **rejects** a one-coefficient corruption (the
   verifier-is-sole-judge contract, rectangular).
4. **T-ternary-exists** (§3 pre-check) — the encoded known rank-11 scheme certifies.
5. **T-plateau-diagnostic** (`#[ignore]`, release) — at a mid budget, report the
   best-rank distribution and whether eager-reduce ever crosses (informs §2.6/OQ1).
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
1. **Eager-reduce vs fixed-rank walk (§2.6):** does the existing eager-reduce loop
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

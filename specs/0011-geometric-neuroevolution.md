# SPEC-0011 — Geometric Neuroevolution (`ufl-evolve`)

- **Status:** Draft — **ready for acceptance** (2026-06-22 — three-lens closed:
  nice-guy STRONG WORK, **architect APPROVE** (re-review), hater NEEDS WORK →
  findings addressed: evolvability made an explicit §2.8 pre-implementation gate,
  the "6/6" downgraded to an unverified prior, the grade-seed-bias restricted to
  answer-blind input grades. The §2.6 de-risk ran; headline reframed to
  **equivariant generalization** (owner). Awaiting owner acceptance.)
- **Realizes:** R-0011 (scope: the geometric headline, reframed — §2.5)
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-21
- **Depends on:** R-0010 (`ufl-geo` — `GeoExpr`, `eval`, `grade`/`typecheck`),
  R-0009 (`ufl-ga` — the `Cl(3,0,1)` kernel + `Point` readout), R-0008
  (`ufl-discovery` — the proposer-agnostic / verifier-exact seam pattern).
- **Crate:** `crates/ufl-evolve` (new — the evolver over `ufl-geo`)

## 1. Motivation

SPEC-0011 realizes [R-0011](../requirements/0011-geometric-neuroevolution.md):
the **evolver** over the `GeoExpr` AST — the M5 payoff. Two acceptance gates: a
**validation gate** (rediscover the rotor sandwich) and the **headline**.

**The headline was reframed by the §2.6 de-risk (2026-06-22).** The original
"beat an MLP on inverse kinematics with ~0.07% of the params" is retired for two
measured reasons: (1) literal IK (target→joint-config) is **inexpressible** in the
`GeoExpr` form set (it needs `acos`/`atan2`/division/`Normalize`/`Log`, none of
which exist), and (2) a *fair* MLP baseline fits the forward map with **~50
params, not 4547** — the param-count headline was a ~30× strawman. What the
de-risk *proved* is stronger and honest: a **4-parameter geometric program
computes the rigid-body map exactly and equivariantly** (RMSE `2.5e-16`
in-distribution **and** out-of-distribution), while the smallest fair MLP only
*approximates* in-distribution and **collapses out-of-distribution** (OOD RMSE
floor ~0.3, up to ~145× its in-distribution error). So the headline is
**exactness + equivariance + generalization**, demonstrated by *evolution* rather
than gradient training — the one thing the CliffordNet/GATr/Haynes family does not
do. The win metric is the **OOD generalization gap**, with the (honest, modest)
~10× param advantage secondary.

Two load-bearing resolutions:
- **Q1 (motors) → no `ufl-geo` extension.** The de-risk built the rigid-body
  forward map (a `Sandwich` of an `R·T·R·T` motor chain) in the *current* forms
  and it is **machine-exact** (§2.6). Motors are `GeoProduct(Exp(eucl-bivector),
  Exp(e₀-bivector))`; the Mv→pose **readout** lives in the verifier (`ufl_ga::
  Point`). A `Normalize`/`Log` extension (which would unlock *literal* IK) is
  **deferred to a future requirement** (§2.7), not needed for this headline.
- **Q2 (crate/seam) → a new `ufl-evolve` crate** with a **generic** seam: R-0008's
  accept step was a boolean `Predicate::discharge`; R-0011 needs a generic genome
  (`GeoExpr`) + a **real-valued, totally-ordered `Fitness`**. Reuse the *pattern*
  (seed → score → select → vary, `SplitMix64`, trajectory, elitism), not the
  matmul types.

## 2. Design

### 2.1 The crate + the generic seam

```rust
// crates/ufl-evolve  (→ ufl-geo, ufl-ga, ufl-prng)

/// A genome's score. Higher is better; the engine maximizes. TOTAL ORDER:
/// non-finite scores (NaN/±inf, reachable via the readout — §2.3) are mapped to
/// the worst rank, never compared raw (an `f64` `sort` on NaN silently corrupts
/// selection — a three-lens finding). `score` returns `Fit`, an ordered wrapper.
pub trait Fitness {
    fn score(&self, genome: &GeoExpr) -> Fit;   // Fit: Ord; NaN/inf ⇒ Fit::WORST
}

pub trait Proposer {
    fn seed(&self, rng: &mut SplitMix64) -> Vec<GeoExpr>;
    fn vary(&self, ranked: &[(GeoExpr, Fit)], rng: &mut SplitMix64) -> Vec<GeoExpr>;
}

pub struct EvolveConfig { pub seed: u64, pub generations: usize, pub population: usize,
                          pub elitism: usize, pub lambda_parsimony: f64, /* operator rates … */ }

pub enum Outcome { Solved { genome: GeoExpr, generation: usize, fitness: Fit },
                   Exhausted { best: GeoExpr, fitness: Fit, trajectory: Vec<Fit> } }

pub fn run<F: Fitness, P: Proposer>(cfg: &EvolveConfig, fit: &F, prop: &P)
    -> Result<Outcome, EvolveError>;
```

The loop mirrors `ufl-discovery::engine::run`. **`SplitMix64` is lifted to a new
shared `ufl-prng` crate** with a *tested* public API — `next_u64`, `below(n)`,
**`f64_unit()`**, **`normal(μ,σ)`** (the tree-GA needs uniform-index *and*
Gaussian-float sampling; `ufl-discovery`'s `SplitMix64` exposes neither today —
a three-lens finding) — and both `ufl-discovery` and `ufl-evolve` depend on it
(no determinism-critical duplication). *Verifier-Held Transparency*: only `run`
holds `fit`; the proposer is answer-blind.

### 2.2 The genetic operators (closed, grade-pruned, grade-seeded)

`GeoExpr` is the chromosome; operators always yield a **well-formed** tree (the
enum is valid by construction; `eval`/`grade` totality absorbs out-of-range
leaves):

- **Point mutation** — swap a node's operator (same arity), or perturb a leaf
  (`Param += normal(0,σ)`; resample a `Basis`/`Var`). **`Exp` `Param`s are range-
  bounded** (a Euclidean-bivector `exp` is bounded `cos/sin`, but an e₀-bivector
  `exp` is linear-unbounded and a mixed "screw" `exp` hits a Taylor path —
  unbounded magnitudes just score worst via §2.3, but bounding the sampler keeps
  the search well-conditioned).
- **Subtree replacement** / **subtree crossover** — depth-bounded, **size-capped**
  (a runaway `Sandwich` nests exponentially — the §2.8 pilot hit an 11 MB render
  without a cap).
- **Local `Param` refinement on the elites (memetic step)** — the §2.8 pilot's
  load-bearing finding: a pure tree-GA reliably assembles the right *structure*
  but lands on the wrong *constants* (e.g. the sandwich shape with a mis-tuned
  angle). Hill-climbing the `Param` leaves of each elite ("fit the constants once
  the shape is right") is what crosses the exact bar. The proposer is therefore
  **memetic** (structure search + numeric refinement), not a pure GA — the
  evidenced form of R-0011's "stronger proposer."

**Grade used two ways (the R-0010 payoff), neither as an entropy penalty:**
1. **Pruning filter** — a grade-**incoherent** candidate (`typecheck` → `∅` grade,
   e.g. a projection onto an absent grade) is **rejected and resampled before
   `eval`** (the decidable signal R-0010 built; an ∅-grade tree can only be zero).
2. **Seeding bias** (optional; *answer-blind*) — the proposer may bias sampling
   using only the task's **input** grades (public, from the task signature — e.g.
   a grade-1 vector input → favor vector-typed subtrees), **never the target/output
   grade**. Biasing on the *target* grade would hand the answer-blind proposer a
   hint (a Verifier-Held-Transparency relaxation, a three-lens finding); it is
   **off by default**, and any gate run that enables it must disclose it. The sound,
   default role of grade is the pruning *filter* (use 1).

> **Note (three-lens):** the spec deliberately **drops the grade-*entropy* fitness
> term** from the original design. Over a `GradeSet` (a bitmask) it degenerates to
> `log₂|grade|`, *and* it would **penalize the multi-grade `{0,2,4}` motors the
> task requires** — actively misdirecting search. Grade earns its keep as a
> *filter* and a *seed bias*, not a regularizer (§7).

### 2.3 The fitness + the readout (total, NaN-safe)

```
fitness(g) = accuracy(g) − λₚ · parsimony(g)
```

- **accuracy** — `−RMSE` of the genome's output vs the task target over the
  dataset (Gate 2) or `−‖eval(g) − e₂‖` (Gate 1).
- **parsimony** — `node_count(g)` (favors small programs; the headline *is* a tiny
  exact program — the de-risk's exact FK is 4 `Param`s / 25 nodes).
- **The Mv→position readout is defined, total, and NaN-safe.** The verifier reads
  a pose from the genome's output `Mv` via `ufl_ga::Point::from_multivector(mv).
  to_euclidean()`, which divides by the homogeneous weight (blade `e₁₂₃`). A
  random genome routinely outputs a non-point `Mv` (zero weight) → the readout is
  **guarded**: if `|weight| ≤ ε` or any coordinate is non-finite, the sample
  contributes the **worst per-sample error** (and the genome's `Fit` is
  `WORST`-bounded), never a `NaN` into the mean. (Verified necessary: a scalar or
  bivector output reads as `NaN` unguarded — a three-lens finding.)

### 2.4 Gate 1 — rediscover the rotor sandwich (AC4, validation)

The task forces a **general rotation**, not a constant: the genome takes an input
vector `Var("v")` and must reproduce the `e₁→e₂` rotation *applied to v*, scored
over several test vectors — `fitness = −(1/|V|) Σ_v ‖eval(g, {v}) − rot(v)‖`
(a constant like `Basis(2)` fails — it ignores `v`). The global optimum is
`Sandwich(Exp(Param·e₁₂), Var("v"))` with `Param ≈ −τ/8`. Answer-blind, the tree-GA
must converge in a **robust fraction of independent seeds** (threshold set by qa).
Expressibility is the R-0009/R-0010 keystone (`hello_geo`). **Honesty guards
(three-lens):** the seed-node sampling distribution is reported alongside the
success rate, and a **uniform-over-arity control** must also find it (so "success"
isn't re-sampling a `Sandwich`/`Exp`-biased prior). **Whether the GA actually finds
this is not assumed — it is gated by the §2.8 evolvability pilot.**

### 2.5 Gate 2 — the equivariant-generalization headline (AC5)

**Task.** A 2-link planar arm (L1=1.0, L2=0.7), forward map `(t1,t2) → (x,y)`
(`x = L1·cos t1 + L2·cos(t1+t2)`, `y = …`). Scalar angle inputs are `Var`-bound
grade-0 `Mv`s; the evolver discovers the motor-chain `GeoExpr` (the de-risk's
hand-built witness, §2.6, is the target structure). Datasets: TRAIN/TEST
in-distribution `[−2,2]²`, **OOD `[2,3]²`** (the extrapolation band). Metric:
RMSE on `(x,y)`.

**The MLP baseline is fair, by construction (three-lens / AC6).** A pure-Rust MLP
(`2→H→2`, tanh, Adam), trained on the same data; **`H` is swept and the *smallest*
network reaching the evolved program's in-distribution error is reported** (not a
padded 4547-param net) — with its parameter count, in-distribution RMSE, **and OOD
RMSE**. The de-risk's fair baseline: ~32 params @ test-RMSE 0.05, ~52 @ 0.01.

**The claim (AC5) — the OOD generalization gap, not param count.** The evolved
`GeoExpr` matches the map **exactly** (RMSE ≈ machine-ε) **on both** the in-dist
and OOD sets, with a small param budget (de-risk: 4 `Param`s); the fair MLP only
approximates in-distribution and **collapses OOD** (de-risk: OOD floor ~0.3, ~70–
145× its in-dist error, regardless of width). Reported honestly with seeds, run
counts, and the per-set RMSE table. The ~10× param advantage is noted as
secondary.

**Honest scope (three-lens).** §2.6 proved the target is *expressible*; whether the
tree-GA can *evolve* the full 25-node exact FK (harder than Gate 1's sandwich) is
**not assumed** — it is gated by the §2.8 evolvability pilot, and the novelty of
R-0011 (that evolution *discovers* the structure, not that a hand-built witness
exists) lives entirely there. **AC5's primary, must-deliver claim is the
*expressibility proof* + the *fair-MLP OOD comparison*** (both evidenced); evolving
the full FK structure is an **explicit stretch**, gated by §2.8. A documented
partial/negative — the search finds a good-but-inexact program — **satisfies AC6**
(the R-0008 discipline). The spec does not pre-bless evolution as expected.

### 2.6 The expressibility de-risk — DONE (the §2.6 gate, run 2026-06-22)

Run in-repo against the real `ufl-geo`/`ufl-ga` kernel (throwaway, repo left
clean). Results (the evidence the reframe rests on):

- **Forward map is expressible and machine-exact.** The hand-built FK `GeoExpr`
  — `Sandwich(GeoProduct(R(t1)·T(L1), R(t2)·T(L2)), origin)` with rotors
  `Exp(−0.5·tᵢ·e₁₂)` and translators `Exp(+0.5·Lᵢ·e₀₁)`, origin = `Basis(7)`
  (e₁₂₃) — is **4 `Param`s, 25 nodes**, RMSE **2.5e-16 in-distribution and OOD**.
  This is the promotable witness (it lands as `tests/fk_expressible.rs` when the
  crate is built).
- **Literal IK is inexpressible** — `acos`/`atan2`/division/`Normalize`/`Log` are
  absent; the versor route needs normalization (`V·~V = 5.0` scalar, no `1/√`).
  Confirmed; this is why the headline reframed and the §2.7 extension is deferred.
- **Conventions pinned** (for the witness): garust uses `exp(−0.5θ·plane)`, so the
  rotor is `Exp(−0.5·t·e₁₂)`; the translator is `Exp(+0.5·d·e₀₁)` and **`exp(d·e₀ᵢ)`
  moves `2d`** (the half-distance gotcha); the readout weight stays exactly 1 for
  rigid motors (0/2000 readout failures), but the §2.3 guard is mandatory for the
  *evolver's* non-rigid intermediates.

### 2.7 Deferred — the `Normalize`/`Log` extension (a future literal-IK requirement)

Literal IK (and direct rotor construction) need `Normalize`/`Log` forms that
`GeoExpr` lacks. Adding them is a clean future requirement (additive `GeoExpr`
variants + their `eval`/`grade` rules, three-lens'd, the R-0010 soundness gate
`realized(eval) ⊆ grade` extended in lockstep). **Out of R-0011 scope** — the
reframed headline needs none of it.

### 2.8 The evolvability pilot — a pre-implementation gate (the §2.6 sibling)

§2.6 proved the target is *expressible*; this gate proves it is *evolvable* —
the actual R-0011 thesis, currently **unevidenced in-repo** (the "6/6 sandwich
rediscovery" is an *unverified external prior* — a throwaway Python run, not yet
reproduced here; it is **not** cited as evidence and Gate-1's threshold is set
fresh by qa, three-lens). **Before the full evolver is built**, a throwaway in-repo
pilot must report hard numbers:
- **(a) Gate-1 structure discovery** — a minimal tree-GA rediscovers the general
  rotation (§2.4), answer-blind, across ≥12 seeds: the success rate, generations-
  to-solve, the winning shapes, the seed-node distribution, **and** the
  uniform-arity control. This reproduces (or refutes) the "6/6" in-repo.
- **(b) Gate-2 param-tunability** — with the §2.6 FK structure fixed, a simple
  optimizer recovers the 4 `Param`s from random inits (the smooth-landscape half of
  evolvability): success rate + evals-to-converge.
**Decision rule:** strong pilot → the full evolver + AC4/AC5 structure-evolution
proceed with evidence; weak/failed pilot → AC5 falls back to its primary deliverable
(expressibility + the fair-MLP OOD comparison), full-structure evolution is
recorded as an open negative, and the proposer (or the genotype) is reconsidered
*before* sinking the build. This is the R-0008 falsification discipline — and the
"Reachability-Before-Search" convention (nice-guy) — applied to the *search*, not
just the target.

**Gate-1 pilot RESULT (run 2026-06-23, throwaway, repo clean).** An answer-blind
tree-GA (uniform-over-arity sampling, no `Sandwich` prior, `typecheck` grade
filter) **discovered exact rotations by search** and the prototype printer
translated them back to GA notation — the whole-thesis loop closed end-to-end on
the smallest case:
- **Success: 3/12 seeds to machine precision** (best `−3.7e-17`), **5/12** within
  `1e-3`; the other 7 collapse to a `−0.5` local optimum. Real basin, **not** a
  clean sweep.
- **Varied valid discoveries**, not a memorized template: the canonical sandwich
  `R v R̃` (via a `GradeLift` route), a *composed double rotor* (stacked
  sandwiches summing to the turn), and a *wedge/inner rotation identity* whose
  discovered constant `−0.785 ≈ −τ/8` is the exact half-angle. Two of three strict
  winners didn't use a top-level `Sandwich` at all.
- **Honesty guards held:** a constant (`Basis(2)`) scores `−1.09` (the task forces
  structure); op-sampling is uniform `1/8`.
- **Two engineering musts surfaced:** the **memetic `Param`-refinement** step
  (§2.2 — the reason 3/12 not 9/12) and the **size cap**; plus a fitness note —
  the residual magnitude must be `√Σcoeff²`, **not** garust's metric-blind `norm()`
  (it zeros `e₀`-blades — the SPEC-0010 trap). This satisfies the §2.8(a) gate
  with a documented, reproducible positive (modulo the memetic upgrade now folded
  into §2.2).

## 3. Code outline

`crates/ufl-evolve/src/`: `lib.rs` (`#![forbid(unsafe_code)]`), `engine.rs`
(`run`, `EvolveConfig`, `Outcome`, `Fit`, `EvolveError`), `proposer.rs` (the
tree-GA + operators + grade-seed bias), `fitness.rs` (`Fitness` + parsimony +
the NaN-safe readout), `tasks/` (`sandwich.rs` Gate 1, `fk.rs` Gate 2 + the
fair-MLP baseline). New `crates/ufl-prng` (lifted `SplitMix64` + `f64_unit`/
`normal`, tested). `examples/evolve_sandwich.rs` (Gate 1 live — also the headline
demo + a behavioral regression oracle). `tests/fk_expressible.rs` (the §2.6
witness, promoted) lands **before** the engine.

## 4. Non-goals

- The agentic "GA-VisAgent" proposer; the Strassen/matmul attempt (R-0011 §4).
- **Literal inverse kinematics** + the `Normalize`/`Log` forms (§2.7, deferred).
- A grade-entropy fitness term (dropped — §2.2 note).
- New `ufl-ga` kernel work; a `GeoExpr` textual reader (still deferred).

## 5. Open questions — resolved

| R-0011 §5 question | Resolution |
|---|---|
| Q1 — motors for IK | **No `ufl-geo` extension** — the forward map is expressible + machine-exact (§2.6); readout in the verifier. The `Normalize`/`Log` extension for *literal* IK is deferred (§2.7). |
| Q2 — crate + seam | **New `ufl-evolve` crate**; generic genome + a **totally-ordered** real-valued `Fitness`; `SplitMix64` lifted to a shared **`ufl-prng`** with `f64_unit`/`normal`. |
| Q3 — the task | The 2-link planar-arm **forward map** with an OOD band; the headline is the **equivariant OOD generalization gap** (§2.5), not param count. |
| Q4 — MLP baseline | Pure-Rust, **smallest-at-the-evolved-error** via a width sweep, in-dist **and** OOD RMSE reported (fair by construction). |
| Q5 — fitness + operators | `accuracy − λₚ·parsimony`; grade as **filter + seed bias**, *not* entropy; `Exp` `Param`s bounded; NaN-safe readout + total `Fit` order. |

## 6. Acceptance criteria

- [ ] **AC1 — Evolver on the generic seam.** `run` reaches genomes only via
  `Proposer::{seed,vary}`, scores only via `Fitness::score` (a total `Fit` order);
  `ufl-prng` `SplitMix64` makes runs reproducible (a fixed seed reproduces a
  trajectory).
- [ ] **AC2 — Closed, grade-pruned operators.** Operators always yield a
  well-formed `GeoExpr`; a grade-incoherent candidate is rejected by `typecheck`
  before scoring. (Unit + a fuzz: every produced genome `typecheck`s or is filtered.)
- [ ] **AC3 — Fitness, total + NaN-safe.** `accuracy − λₚ·parsimony` implemented;
  the Mv→pose readout is guarded (non-point/non-finite ⇒ worst per-sample error);
  `Fit` is a total order (NaN/inf ⇒ `WORST`), so selection never sees a raw NaN.
  (Unit: a NaN-producing genome ranks last, never corrupts the sort.)
- [ ] **AC4 — Gate 1: rediscover the sandwich (gated by §2.8).** Answer-blind, the
  evolver discovers the general-rotation program (§2.4) within ε in ≥ the qa-set
  fraction of seeds; the seed-node distribution **and** a uniform-arity control are
  reported. The §2.8 pilot reproduces this in-repo first (the "6/6" is not assumed).
  (e2e.)
- [ ] **AC5 — Gate 2: the equivariant-generalization headline.** **Primary (must
  deliver):** the **expressibility proof** (the §2.6 4-`Param` exact FK witness) +
  the **fair-MLP OOD comparison** — the **smallest fair MLP** at the evolved/witness
  in-dist error, reported with params + in-dist + **OOD** RMSE; the headline is the
  **OOD gap** (geometric ≈ exact everywhere, MLP collapses OOD). **Stretch (gated by
  §2.8):** an *evolved* (not hand-built) `GeoExpr` matches the map within ε on both
  sets. A documented stretch-negative satisfies AC6.
- [ ] **AC6 — Honest reporting.** Fair (smallest-at-error) MLP baseline; seeds/
  run-counts/success-rates disclosed; a failed/partial gate documented (an honest
  negative satisfies AC6).

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-21 | **Q1: no `ufl-geo` extension** — motors via `Exp` of e₀-bivectors; readout in the verifier. | Confirmed machine-exact by the §2.6 de-risk; keeps the merged crate frozen. |
| 2026-06-21 | **Q2: new `ufl-evolve` crate**; generic genome + real-valued fitness; `SplitMix64` lifted to `ufl-prng`. | R-0008's seam is welded to matmul `Scheme` + boolean discharge; the float/normal samplers the tree-GA needs don't exist on the current PRNG (three-lens). |
| 2026-06-22 | **Headline reframed to equivariant OOD generalization** (away from "IK with ~0.07% params"). | The §2.6 de-risk: literal IK is inexpressible, **and** a fair MLP is ~50 params (the 4547 was a ~30× strawman). The measured, honest win is exactness + equivariance: geometric RMSE `2.5e-16` in-dist **and** OOD vs the MLP's ~0.3 OOD floor (up to 145×). Owner-approved. |
| 2026-06-22 | **Grade-entropy dropped from the fitness**; grade kept as a filter + seed bias. | Over a `GradeSet` it is `log₂|grade|`, and it would penalize the `{0,2,4}` motors the task needs — a misdirection (three-lens). Pruning + seeding are the sound uses. |
| 2026-06-22 | **Fitness is a total order; the readout is NaN-safe.** | `f64` is not `Ord`; the readout `NaN`s on the non-point `Mv`s a random genome routinely produces, and a raw-NaN `sort` silently corrupts elitism (three-lens, demonstrated). |
| 2026-06-22 | **MLP baseline = smallest-at-the-evolved-error + OOD reported.** | Prevents the strawman the de-risk exposed; makes AC5 a real, falsifiable gate. |
| 2026-06-22 | **`Normalize`/`Log` extension (literal IK) deferred to a future requirement** (§2.7). | Not needed for the reframed headline; keeps R-0011 shippable without re-opening `ufl-geo`. |
| 2026-06-22 | **Evolvability made an explicit §2.8 pre-implementation gate** (hater re-review). | The R-0011 thesis is that evolution *discovers* structure; §2.6 only proved *expressibility*. The hater is right that a hand-built witness + a blessed honest-negative "can't lose." So evolvability is de-risked (a Gate-1 structure-discovery pilot + a Gate-2 param-tunability pilot) *before* the full build — the §2.6/"Reachability-Before-Search" discipline applied to the search. AC5's primary deliverable is expressibility + the fair-MLP comparison; evolved structure is a §2.8-gated stretch. |
| 2026-06-22 | **"6/6 sandwich rediscovery" downgraded to an unverified external prior** (not cited as evidence; reproduced in-repo by §2.8). | It is a throwaway Python run with no in-repo artifact (hater); using it to calibrate Gate-1's threshold or imply Gate-2 tractability would be an unfalsifiable citation. Gate-1's threshold is set fresh by qa. |
| 2026-06-22 | **Grade-seed-bias restricted to *input* grades, off by default; target-grade biasing is a disclosed Transparency relaxation, not taken.** | Biasing the answer-blind proposer on the *target* grade leaks a hint (`GradeCtx::declare` is keyed by input var, not a target channel — hater). Grade's sound default role is the pruning filter. |

## 8. Companion edits (this branch)

- `crates/ufl-evolve`, `crates/ufl-prng` — new crates; workspace `members += `.
- The de-risk witness `tests/fk_expressible.rs` (the promotable 4-`Param` FK).

## Changelog

- 2026-06-21 — created (Draft); resolved Q1–Q5; made the §2.6 expressibility
  de-risk a pre-implementation gate.
- 2026-06-22 — **three-lens applied** (nice-guy STRONG WORK; architect REQUEST
  CHANGES; hater NEEDS WORK — both ran probes against the real kernel). **§2.6
  de-risk run**, and the **headline reframed to equivariant OOD generalization**
  (owner decision). Revisions: dropped the grade-entropy fitness term (kept grade
  as filter + seed bias); defined a total `Fit` order + a NaN-safe readout; lifted
  `SplitMix64` to `ufl-prng` with `f64_unit`/`normal`; made the MLP baseline
  smallest-at-error + OOD; bounded `Exp` `Param`s; Gate-1 seed-distribution
  disclosure + control; deferred the `Normalize`/`Log` extension to a future
  literal-IK requirement; recorded the de-risk evidence + the promotable FK
  witness.
- 2026-06-22 (later) — **three-lens closed**: architect **APPROVE** on re-review
  (all six prior findings verified resolved); hater **NEEDS WORK** → addressed by
  making evolvability an explicit §2.8 pre-implementation gate, downgrading the
  "6/6" to an unverified prior, and restricting the grade-seed-bias to answer-blind
  input grades. **Status → ready for owner acceptance.**

# SPEC-0008 — Discovery Engine (loop validation + blind-proposer falsification)

- **Status:** Draft (revised after the empirical de-risk + first three-lens)
- **Realizes:** R-0008
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-12
- **Depends on:** R-0007 (`RankDecomposition` + the `residual()` seam), R-0006
  (`Scheme`/`Triple`)
- **Crate:** `crates/ufl-discovery` (extended; no new external dependency)

## 1. Motivation

SPEC-0008 builds the seeded genetic **discovery engine** and validates it end to
end, with its scope set by the **empirical de-risk** in
[`papers-review.md`](../ufl-discovery/papers-review.md) §4a: three independent
blind methods cannot drive 2×2 matmul to exact residual 0 (rank-7 0/10), but a
blind GA recovers a **planted solvable target 8/10**. So:

- **R-0008 validates the *loop and the seam*** — proposer → `residual` →
  `discharge` accept → certificate — on the **planted target** (a problem a
  blind proposer provably solves).
- **R-0008 documents the *blind-proposer falsification* on matmul** — the
  engine runs on Strassen, plateaus, and the trajectory is recorded as the
  honest diagnostic (AC6's purpose). This *motivates* R-0011's stronger
  proposer rather than pretending blind GA suffices.
- **Rediscovering Strassen relocates to R-0011**, where the proposer is upgraded
  (memetic / agentic — the GA-VisAgent pattern). The **proposer-agnostic seam**
  makes that a proposer swap with the verifier untouched.

Design principle (now load-bearing): **Verifier-Held Transparency** — a blind
GA, an agent, or a coin flip may *propose*; only an exact `Predicate::discharge`
may *accept* (`docs/conventions.md`).

## 2. Design

### 2.1 PRNG — in-crate `SplitMix64` (no `rand`)

```rust
pub struct SplitMix64 { state: u64 }
impl SplitMix64 {
    pub fn new(seed: u64) -> Self { Self { state: seed } }
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize { (self.next_u64() % n as u64) as usize }
    fn ternary(&mut self) -> i8 { [-1, 0, 1][self.below(3)] } // bias negligible for search
}
```

### 2.2 `residual()` — fitness IS the verifier's arithmetic (AC2)

The SPEC-0007 §4 seam is built: `residual` is the grading primitive; `discharge`
derives from it (R-0007 tests stay green — `discharge`'s observable behaviour,
incl. *DimMismatch regardless of rank*, is unchanged).

```rust
impl RankDecomposition {
    pub fn residual(&self, scheme: &Scheme) -> Result<i64, SchemeError> {
        let recon = reconstruct(scheme);
        error(&recon, &self.target).ok_or(SchemeError::DimMismatch {
            n: self.n, expected: self.target.dim(), got: recon.dim(),
        })
    }
    // discharge(s) ≡ Ok(self.residual(s)? == 0 && s.rank() == self.rank)
}
```

### 2.3 Genotype vs phenotype — `Genome`, total `express`, `ufl-tensor` untouched

The proposer owns a mutable **genome**; the **phenotype** is a `Scheme` built
through `ufl-tensor`'s validating constructors. `express` is **total and loud** —
a malformed genome is a typed error, never a silently shorter scheme (the
review's silent-truncation finding):

```rust
pub struct Genome { pub triples: Vec<[Vec<i8>; 3]> } // rank × (u,v,w), each length d = n²

fn express(g: &Genome) -> Result<Scheme, SchemeError> {
    let mut s = Scheme::new();
    for [u, v, w] in &g.triples {
        s.push(Triple::new(u.clone(), v.clone(), w.clone())?)?; // ? — no swallow
    }
    Ok(s)
}
```

`ufl-tensor` stays a **pure leaf, byte-untouched** (validity enforced at
`express` by *Guard Inside the Candidate*). The **certificate** (AC5) is the
expressed `Scheme`.

### 2.4 `GaProposer` — the blind proposer (concrete, with pinned config)

```rust
/// Pinned defaults — the configuration that recovered the planted target 8/10
/// in the de-risk (papers-review §4a). Pre-registered so AC3's gate is honest.
pub struct GaConfig {
    pub population: usize,      // 300
    pub tournament_size: usize, // 5
    pub mutation_count: usize,  // 2
    pub elitism: usize,         // 4  (REQUIRED ≥ 1 — see AC6)
}
```

- **Seed:** each genome entry uniform `{-1,0,+1}`, `rank` triples of length `d`.
- **Selection:** tournament — `tournament_size` random indices, lowest residual
  wins; **on a tie the lower index wins** (so AC1 determinism is total).
- **Crossover:** uniform over the `rank` triples (the scheme is an unordered
  sum).
- **Mutation:** copy, set `mutation_count` random entries to `rng.ternary()`.
- **Elitism:** the best `elitism ≥ 1` genomes survive unchanged (the monotone-best
  guarantee AC6 depends on). `GaProposer::new` rejects `elitism == 0` and
  `population == 0` with a typed error.

Operators are mutation + crossover only (the merge/split third timescale stays
deferred to R-0011 / AC6's verdict).

### 2.5 The engine — `run`, `Config`, `Outcome`

```rust
pub struct Config { pub predicate: RankDecomposition, pub generations: usize,
                    pub seed: u64, pub ga: GaConfig }
pub enum Outcome {
    Found { scheme: Scheme, generation: usize },
    Exhausted { best: Scheme, best_residual: i64, trajectory: Vec<i64> },
}

pub fn run(config: &Config) -> Result<Outcome, EngineError> {
    config.validate()?;                       // population ≥ 1, generations ≥ 1, elitism ≥ 1
    let mut rng = SplitMix64::new(config.seed);
    let proposer = GaProposer::new(/* d, rank, */ config.ga)?;
    let mut pop = proposer.seed(&mut rng);
    let mut trajectory = Vec::with_capacity(config.generations);
    let mut scored = score(&config.predicate, pop)?;     // Vec<(Genome, i64)>, sorted
    for gen in 0..config.generations {
        // acceptance is the verifier, evaluated up-front (no fall-through):
        if scored[0].1 == 0 {
            let s = express(&scored[0].0)?;
            debug_assert!(config.predicate.discharge(&s)? );  // residual 0 ∧ fixed rank ⇒ true
            return Ok(Outcome::Found { scheme: s, generation: gen });
        }
        trajectory.push(scored[0].1);          // best of THIS scored generation
        pop = proposer.vary(&scored, &mut rng);
        scored = score(&config.predicate, pop)?;
    }
    // final population is scored; best is scored[0] — same generation the last
    // trajectory entry will describe after one more push:
    trajectory.push(scored[0].1);
    Ok(Outcome::Exhausted {
        best: express(&scored[0].0)?, best_residual: scored[0].1, trajectory,
    })
}
```

`score` expresses each genome and grades by `residual` (the `?` propagates the
*impossible* dim error rather than hiding it). The genome is fixed-rank `R` by
construction, so `residual == 0` already implies `discharge == Ok(true)`; the
`debug_assert` documents that invariant without making it control flow.

**Determinism (AC1):** one seeded `SplitMix64` threads `seed` → every `vary`;
`Outcome` is built only from `Vec`/`Scheme`/`i64` (no `HashMap`, no float order),
and the tournament tie-break is index-deterministic.

### 2.6 The proposer seam (named, not abstracted) — and its two real leaks

`run` reaches candidates only via `proposer.{seed,vary}` and accepts only via
`predicate.discharge`. No `Proposer` trait is built (a trait is earned by *two*
instances — the SPEC-0007 rule; R-0011 brings the second). Honest naming of what
R-0011 generalizes, beyond the `Genome` type:

1. **`express` is proposer-owned** — it is the genotype→phenotype map, so it
   belongs behind the future `Proposer` trait (today a free fn over the concrete
   `Genome`), not a fixed engine function.
2. **The `i64` fitness type is part of the seam** — R-0011's geometric predicate
   has a *dense* fitness (pose error), not an integer residual; `run`'s `i64` is
   a concrete commitment the trait must generalize (an associated `Fitness: Ord`).

These are *named* now so R-0011's generalization is a known, bounded change, not
a surprise.

## 3. Code outline

New modules in `crates/ufl-discovery`: `prng.rs` (§2.1), `genome.rs`
(`Genome`, total `express`, §2.3), `proposer.rs` (`GaProposer`, `GaConfig`,
§2.4), `engine.rs` (`Config`, `Outcome`, `EngineError`, `run`, §2.5);
`predicate.rs` gains `residual()` and routes `discharge` through it (§2.2).
`lib.rs` re-exports `run`, `Config`, `Outcome`, `GaConfig`, `RankDecomposition`.
An `examples/hello_discovery.rs` runs the engine on the planted target and
re-discharges the found certificate (the `hello_*` convention; AC5 made
tangible).

## 4. Non-goals

- The geometric genotype, the agent proposer, the `Proposer` trait, **and
  Strassen rediscovery** — all **R-0011**.
- Phase 2 (n ≥ 3), `egg`, `rayon`, learned guidance, record-beating — unchanged.
- Buffer-reuse micro-optimization — only if profiling demands.

## 5. Open questions — resolved

| Question | Resolution |
|---|---|
| PRNG | In-crate `SplitMix64`. |
| `residual()` placement | On `RankDecomposition`; `discharge` derived. |
| Budget (the hater's blocking finding) | **Pre-registered from the de-risk:** population 300, generations 1500, tournament 5, mutation 2, elitism 4 — the config measured to recover the planted target 8/10. AC3's ≥6/10 gate sits below the measurement with margin. |
| CI shape | **Smoke gate** (planted recovery, seed 0 + determinism) is always-on. The **planted ladder** (AC3, seeds 0..=9) and the **matmul experiment** (AC4) are `#[ignore]`-d — qa runs `cargo test -- --ignored` for sign-off and records results in the `ufl-discovery/` writeup (decision log). |
| Operators | Mutation + uniform triple-crossover + tournament + elitism(≥1). |

## 6. Acceptance criteria

- [ ] **AC1 — Determinism.** Same `(seed, config)` ⇒ identical `Outcome` (run
  twice, compare).
- [ ] **AC2 — Fitness is the verifier's arithmetic.** Grading uses
  `RankDecomposition::residual`; `discharge` is *defined in terms of* it. A test
  pins `discharge(s) == Ok(true) ⟺ residual(s) == Ok(0) && rank matches`, and the
  R-0007 acceptance suite stays green.
- [ ] **AC3 — Loop validation on a solvable known-answer instance.** For a
  **planted** target (sum of `K = 5` fixed ternary triples) at search rank 5, the
  engine returns `Found` for **≥ 6 of seeds 0..=9** (`#[ignore]` ladder;
  evidence-based — measured 8/10). Exercises `Found`, the certificate, and
  determinism end-to-end.
- [ ] **AC4 — Blind-proposer falsification, documented.** The engine runs on
  `n = 2`, ranks 7 and 8, seeds 0..=9; the outcomes and best-residual
  trajectories are recorded in a `ufl-discovery/` writeup (`#[ignore]` ladder).
  The honest result (the plateau, per §4a — or, if blind GA surprises us, a
  discovery) is written up *either way*. This is the falsifiable experiment, not
  a guaranteed negative.
- [ ] **AC5 — Certificates.** Every `Found.scheme` re-discharges `Ok(true)`
  through a **freshly constructed** `RankDecomposition`.
- [ ] **AC6 — Diagnostics.** `Exhausted` carries the per-generation
  best-residual `trajectory`; with `elitism ≥ 1` it is monotone non-increasing
  (a test asserts it). No genome ever truncates (`express` is total — a test
  asserts every scored phenotype has rank `R`), so the trajectory reflects
  *landscape*, not engine bugs.
- [ ] **AC-smoke (merge gate, always-on).** Planted recovery at seed 0 +
  determinism — fast, deterministic, in the `cargo test` suite.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-12 | **Proposer-agnostic / verifier-exact** seam; build `GaProposer` only, no `Proposer` trait (earned by R-0011's second instance). | Viability: blind GA won't scale; transparency must live in the verifier. |
| 2026-06-12 | `residual()` on `RankDecomposition`; `discharge` derived. | AC2 "fitness == verifier" structural (SPEC-0007 §4 seam). |
| 2026-06-12 | Genotype/phenotype split; `express` **total** (`Result`, no swallow); `ufl-tensor` untouched. | The review's silent-truncation finding — a malformed genome must be a typed error, not degraded fitness that pollutes AC6's diagnostics. |
| 2026-06-12 | **Re-scoped after the empirical de-risk:** AC3 = loop validation on the planted target (blind GA solves it, 8/10); AC4 = documented matmul falsification; **Strassen rediscovery → R-0011**. | Three blind methods fail exact 2×2 matmul (papers-review §4a). Forcing it into R-0008 means a real solver or the agent proposer — collapsing R-0011 into R-0008. The honest gate is what a blind proposer *can* validate + the diagnosed wall. |
| 2026-06-12 | **Pre-registered budget** (population 300 / gen 1500 / tour 5 / mut 2 / elite 4) from the de-risk; AC3 ≥6/10 with margin under the measured 8/10. | The hater's blocking finding — an "honest canary" needs a specified budget; this one is *measured*, not guessed. |
| 2026-06-12 | `elitism ≥ 1` and `population/generations ≥ 1` validated in construction; tournament ties broken by lower index. | AC6 monotonicity precondition; AC1 determinism totality (architect + hater minor findings). |
| 2026-06-12 | **QA-runs-ignored contract:** AC3/AC4 are `#[ignore]`-d (out of the merge gate) but **mandatory QA sign-off** tests — qa runs `--ignored` over seeds 0..=9 and records the pass count + trajectories in the `ufl-discovery/` writeup. `main`'s always-on gate is the smoke test; the statistical ACs are verified-by-QA, not unverified. Gustavo accepts that `main` carries no always-on guarantee of AC3/AC4. | architect process finding: distinguish the merge gate from the AC-verification gate; make the contract explicit so `#[ignore]` is not misread as untested. |
| 2026-06-12 | Named the seam's two real leaks (`express` proposer-owned; `i64` fitness type generic). | The hater's "names two fewer leaks than exist" — R-0011's change is bounded and known. |

## Changelog

- 2026-06-12 — created (Draft).
- 2026-06-12 — revised after the empirical de-risk (papers-review §4a) and the
  first three-lens: re-scoped to loop-validation + documented matmul
  falsification (Strassen → R-0011); pre-registered the budget from the
  measurement; `express` made total; `elitism ≥ 1` / config validation /
  tie-break pinned; the QA-runs-ignored contract recorded; the seam's two leaks
  named; `hello_discovery` example added.

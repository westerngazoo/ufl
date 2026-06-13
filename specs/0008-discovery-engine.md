# SPEC-0008 — Discovery Engine (seeded GA, proposer-agnostic / verifier-exact)

- **Status:** Draft
- **Realizes:** R-0008
- **Author:** Gustavo Delgadillo (Goose) — drafted with Claude
- **Created:** 2026-06-12
- **Depends on:** R-0007 (`RankDecomposition` — the verifier + the `residual()`
  seam), R-0006 (`Scheme`/`Triple`)
- **Crate:** `crates/ufl-discovery` (extended; no new external dependency)

## 1. Motivation

SPEC-0008 realizes [R-0008](../requirements/0008-discovery-engine.md): a seeded
genetic search over decomposition `Scheme`s whose **accept step is the R-0007
discharge**. It is the **engine-validation step** of the geometric-neuroevolution
program — prove the search + predicate-discharge loop on the *known-answer*
Strassen problem before the genotype generalizes to geometric ASTs (R-0011).

Two principles from the viability analysis ([`papers-review.md`](../ufl-discovery/papers-review.md) §4)
shape the design:

1. **Proposer-agnostic, verifier-exact.** Transparency belongs to the *verifier*,
   not the proposer. A blind GA (here), an LLM agent (R-0011), or anything may
   *propose* candidates; only an exact `Predicate::discharge` may *accept* one.
   The engine names this seam; it builds only the blind-GA proposer.
2. **The 2×2 canary is a real falsification — do not soften it.** If blind GA
   cannot clear rank-7 (AC4) in budget, that is the honest signal that a blind
   proposer is too weak, caught for one requirement's cost. Strassen's scheme
   appears **only** in tests (the expected-output oracle), never in the engine.

## 2. Design

### 2.1 PRNG — in-crate `SplitMix64` (no `rand` dependency)

```rust
/// Deterministic, seeded splitmix64 (~12 lines). Keeps UFL dependency-free and
/// makes AC1 determinism trivially auditable.
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
    fn ternary(&mut self) -> i8 { [-1, 0, 1][self.below(3)] } // bias negligible & deterministic
}
```

### 2.2 `residual()` — the fitness IS the verifier's arithmetic (AC2)

The SPEC-0007 §4 breadcrumb is built: `residual()` becomes the grading
primitive, and `discharge` is refactored to derive from it — so fitness and
acceptance are provably the same computation (the R-0007 tests stay green; the
behaviour of `discharge` is unchanged).

```rust
impl RankDecomposition {
    /// ‖reconstruct(scheme) − T_n‖² against the cached target. Err(DimMismatch)
    /// on dim/n mismatch — the same total contract as discharge.
    pub fn residual(&self, scheme: &Scheme) -> Result<i64, SchemeError> {
        let recon = reconstruct(scheme);
        error(&recon, &self.target).ok_or(SchemeError::DimMismatch {
            n: self.n, expected: self.target.dim(), got: recon.dim(),
        })
    }
}
// discharge now derives from residual (one computation, same contract):
//   fn discharge(&self, s) -> Result<bool, _> {
//       Ok(self.residual(s)? == 0 && s.rank() == self.rank)
//   }
```

### 2.3 Genotype vs phenotype — the genome, `express`, and `ufl-tensor` untouched

The proposer owns a raw **genome** (the mutable representation); the **phenotype**
is a `Scheme`, built only through `ufl-tensor`'s validating public constructors
(`Triple::new` / `Scheme::push` — *Guard Inside the Candidate*, so a malformed
candidate is unconstructible). `ufl-tensor` stays a **pure leaf, byte-untouched**.

```rust
/// rank × (u, v, w), each a length-d = n² ternary vector. The proposer mutates
/// genomes; the engine `express`es each to a Scheme for scoring.
pub struct Genome { pub triples: Vec<[Vec<i8>; 3]> }

fn express(g: &Genome) -> Scheme {
    let mut s = Scheme::new();
    for [u, v, w] in &g.triples {
        // u,v,w are constructed ternary & equal-length by the proposer, so these
        // never error; the Result is mapped, never unwrapped in lib code.
        if let Ok(t) = Triple::new(u.clone(), v.clone(), w.clone()) { let _ = s.push(t); }
    }
    s
}
```

The **certificate** (AC5) is the expressed `Scheme` (the phenotype), re-dischargeable
by anyone through a freshly constructed `RankDecomposition`.

### 2.4 `GaProposer` — the blind proposer (the one built here)

```rust
pub struct GaProposer { d: usize, rank: usize, cfg: GaConfig }
// GaConfig: population, tournament_size, mutation_count, elitism

impl GaProposer {
    /// Initial population: each entry drawn uniform {-1,0,+1}.
    fn seed(&self, rng: &mut SplitMix64) -> Vec<Genome>;
    /// Next generation from scored parents: elitism (carry best k unchanged) +
    /// tournament-selected parents recombined by uniform triple-crossover, then
    /// point mutation (flip `mutation_count` random entries to a random ternary).
    fn vary(&self, scored: &[(Genome, i64)], rng: &mut SplitMix64) -> Vec<Genome>;
}
```

- **Selection:** tournament — pick `tournament_size` random indices, keep the
  lowest residual.
- **Crossover:** uniform over the `rank` triples (the scheme is an unordered sum,
  so triple-level mixing is natural).
- **Mutation:** copy, then set `mutation_count` random entries to `rng.ternary()`.
- **Elitism:** the best `elitism` genomes survive unchanged (monotone best).

Operators are **mutation + crossover only** (the PRD's third timescale —
merge/split rewrites — is deferred; AC6's diagnostics decide if rank-7 needs it).

### 2.5 The engine — `run`, `Config`, `Outcome`

```rust
pub struct Config { pub n: usize, pub rank: usize, pub generations: usize,
                    pub seed: u64, pub ga: GaConfig }

pub enum Outcome {
    /// A discovery: the certificate scheme + when it was found.
    Found { scheme: Scheme, generation: usize },
    /// Budget exhausted: best phenotype + its residual + the per-generation
    /// best-residual trajectory (AC6 falsification diagnostics).
    Exhausted { best: Scheme, best_residual: i64, trajectory: Vec<i64> },
}

pub fn run(config: &Config) -> Result<Outcome, SchemeError> {
    let predicate = RankDecomposition::new(config.n, config.rank);
    let proposer = GaProposer::new(config.n * config.n, config.rank, config.ga);
    let mut rng = SplitMix64::new(config.seed);
    let mut pop = proposer.seed(&mut rng);
    let mut trajectory = Vec::with_capacity(config.generations);
    for gen in 0..config.generations {
        let mut scored = Vec::with_capacity(pop.len());
        for g in pop {
            let s = express(&g);
            scored.push((g, predicate.residual(&s)?)); // Err only on impossible dim
        }
        if let Some((g, _)) = scored.iter().find(|(_, r)| *r == 0) {
            let s = express(g);
            if predicate.discharge(&s)? {              // exact acceptance (verifier)
                return Ok(Outcome::Found { scheme: s, generation: gen });
            }
        }
        trajectory.push(scored.iter().map(|(_, r)| *r).min().unwrap_or(i64::MAX));
        pop = proposer.vary(&scored, &mut rng);
    }
    // … assemble Exhausted from the best of the final population …
}
```

**Determinism (AC1):** the single seeded `SplitMix64` threads through `seed` and
every `vary`; same `(seed, config)` ⇒ identical population sequence ⇒ identical
`Outcome`.

### 2.6 The proposer seam (named, not abstracted)

`run` reaches candidates *only* through `proposer.seed()` / `proposer.vary()`,
and accepts *only* through `predicate.discharge()`. That is the
**proposer-agnostic / verifier-exact** boundary. SPEC-0008 builds exactly one
proposer (`GaProposer`); it does **not** introduce a `Proposer` trait — by the
same rule SPEC-0007 applied to `Predicate` (a trait is earned by *two* real
instances, not one). R-0011 introduces `trait Proposer { type Genome; … }`,
impls it for `GaProposer` **and** the agentic proposer (the GA-VisAgent pattern),
and makes `run` generic — a mechanical change because the method shapes
(`seed`/`vary`) are already right. The seam survives; the abstraction waits for
its second instance.

## 3. Code outline

New modules in `crates/ufl-discovery`: `prng.rs` (§2.1), `genome.rs`
(`Genome` + `express`, §2.3), `proposer.rs` (`GaProposer` + `GaConfig`, §2.4),
`engine.rs` (`Config`, `Outcome`, `run`, §2.5); `predicate.rs` gains `residual()`
and routes `discharge` through it (§2.2). `lib.rs` re-exports `run`, `Config`,
`Outcome`, `GaConfig`.

## 4. Non-goals

- The geometric genotype, the agent proposer, the `Proposer` trait — **R-0011**.
- Phase 2 (n ≥ 3), `egg`, `rayon`, learned guidance, record-beating — unchanged
  from R-0008 §4.
- Buffer-reuse micro-optimization of `express`/`residual` — only if profiling on
  the ignored ladder demands it.

## 5. Open questions — resolved

| R-0008 §5 question | Resolution |
|---|---|
| PRNG dependency | **In-crate `SplitMix64`** (§2.1) — no `rand`; AC1 auditable. |
| `residual()` placement | **On `RankDecomposition`**, with `discharge` derived from it (§2.2) — AC2 "same computation" is structural. |
| CI shape | **Smoke gate** (seed 0, rank 8, small budget + a determinism test) runs in the merge-gated suite; the **acceptance ladder** (AC3 rank-8 ≥9/10, AC4 rank-7 ≥3/10) is `#[ignore]`-d — qa runs it for sign-off, results recorded in the writeup. No wall-clock assertions. |
| Operator minimality | **Mutation + uniform triple-crossover + tournament + elitism** (§2.4); merge/split deferred to AC6's verdict. |

## 6. Acceptance criteria

- [ ] **AC1 — Determinism.** `run` with the same `(seed, config)` returns an
  identical `Outcome` (asserted by running twice and comparing).
- [ ] **AC2 — Fitness is the verifier's arithmetic.** Grading uses
  `RankDecomposition::residual`; acceptance uses `discharge`, which is *defined
  in terms of* `residual` (one cached-target computation). A test pins
  `discharge(s) == Ok(true) ⟺ residual(s) == Ok(0) && rank matches`.
- [ ] **AC3 — Phase-0 gate.** For `n = 2`, rank 8: `run` returns `Found` within
  the budget for **≥ 9 of seeds 0..=9** (`#[ignore]` ladder).
- [ ] **AC4 — Phase-1 rediscovery.** For `n = 2`, rank 7: `Found` within budget
  for **≥ 3 of seeds 0..=9**, with **no Strassen fixture in the engine path**
  (`#[ignore]` ladder). The found scheme need not be Strassen's. *Not softened
  to mask a blind-GA failure.*
- [ ] **AC5 — Certificates.** Every `Found.scheme` re-discharges `Ok(true)`
  through a **freshly constructed** `RankDecomposition`.
- [ ] **AC6 — Falsification diagnostics.** `Exhausted` carries the
  per-generation best-residual `trajectory`; a test asserts it is recorded and
  monotone non-increasing (elitism guarantees the best never worsens).
- [ ] **AC-smoke (merge gate).** A fast deterministic test: seed 0, rank 8,
  small budget → `Found`; plus the AC1 determinism test. Always-on CI.

## 7. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-12 | **Proposer-agnostic / verifier-exact** seam: `run` reaches candidates only via `proposer.{seed,vary}` and accepts only via `discharge`; build `GaProposer` only, no `Proposer` trait yet. | Viability analysis: blind GA won't scale, the agent is the scalable proposer; transparency must live in the verifier. Trait deferred until R-0011's second instance (the SPEC-0007 rule). |
| 2026-06-12 | `residual()` on `RankDecomposition`; `discharge` derived from it. | Makes AC2's "fitness == verifier" structural, not asserted — the SPEC-0007 §4 seam. R-0007 tests stay green (discharge unchanged). |
| 2026-06-12 | Genotype (`Genome`) distinct from phenotype (`Scheme`); `ufl-tensor` untouched. | The proposer owns a mutable representation; validity is enforced at `express` by `ufl-tensor`'s constructors — no need to widen `ufl-tensor`'s API or read private entries. |
| 2026-06-12 | In-crate `SplitMix64`; smoke-gate-in-CI + ignored acceptance ladder; mutation+crossover only. | Dependency discipline; CI stays fast and non-flaky (Structural Frugality); minimal operators with AC6 deciding escalation. |
| 2026-06-12 | AC4 kept at ≥3/10 and **not softened**; Strassen only in tests. | The honest canary — a blind-proposer failure must be *visible*, not engineered around. |

## Changelog

- 2026-06-12 — created (Draft).
